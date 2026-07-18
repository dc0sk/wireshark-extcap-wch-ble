mod pcap;
mod usb;

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use usb::{CaptureConfig, McuDevice, PhyMode, WCH_PID_BLE_MCU, WCH_VID};

const ADV_CH: [u8; 3] = [37, 38, 39];

static HELP_TEXT: &str = "\
WCH BLE Analyzer Pro — Wireshark extcap plugin

Usage: wch-ble-extcap [extcap options]

Extcap options (used by Wireshark):
  --extcap-interfaces       List available capture interfaces
  --extcap-interface <if>   List DLTs for a specific interface
  --extcap-dlts             List DLT types for the interface
  --extcap-config           List configuration options
  --extcap-capture          Start capturing (pcap to stdout)
  --extcap-filter <expr>    Capture filter (passed from Wireshark)
  --fifo <path>             FIFO path for pcap output (alternative to stdout)

Plugin options (passed via extcap config):
  --channel <n>             BLE advertising channel: 37, 38, 39, or 0=all (default: 0)
  --phy <n>                 PHY: 1=1M (default), 2=2M, 3=CodedS8, 4=CodedS2
  -v                        Verbose: print packets to stderr
  -h, --help                Show this help
";

fn ble_ch_to_rf(ch: u8) -> u8 {
    match ch {
        37 => 0,
        38 => 12,
        39 => 39,
        c if c <= 10 => c + 1,
        c => c + 2,
    }
}

fn extcap_interfaces() {
    // extcap format: <name>;<display>;<help>
    println!(
        "wch-ble-extcap;WCH BLE Analyzer Pro;WCH BLE Analyzer Pro BLE 5.1 sniffer (3x CH582F MCUs)"
    );
}

fn extcap_dlts() {
    // extcap format: <interface>;<number>;<dlt>;<name>
    println!("wch-ble-extcap;1;256;DLT_BLUETOOTH_LE_LL_WITH_PHDR");
}

fn extcap_config() {
    // arg format: arg {number=N}{call=--X}{display=Y}{type=Z}{tooltip=T}
    // Double braces {{}} to escape them from Rust's format! macro
    println!("arg {{number=0}}{{call=--channel}}{{display=Advertising Channel}}{{type=selector}}{{tooltip=BLE advertising channel (0=all, 37, 38, 39)}}");
    println!("value {{arg=0}}{{value=0}}{{display=All channels (auto)}}");
    println!("value {{arg=0}}{{value=37}}{{display=Channel 37 (2402 MHz)}}");
    println!("value {{arg=0}}{{value=38}}{{display=Channel 38 (2426 MHz)}}");
    println!("value {{arg=0}}{{value=39}}{{display=Channel 39 (2480 MHz)}}");
    println!("arg {{number=1}}{{call=--phy}}{{display=PHY Mode}}{{type=selector}}{{tooltip=Physical layer modulation}}");
    println!("value {{arg=1}}{{value=1}}{{display=1M (default)}}");
    println!("value {{arg=1}}{{value=2}}{{display=2M}}");
    println!("value {{arg=1}}{{value=3}}{{display=CodedS8 (Long Range, 125 kbps)}}");
    println!("value {{arg=1}}{{value=4}}{{display=CodedS2 (Long Range, 500 kbps)}}");
}

fn extcap_capture(verbose: bool, channel: u8, phy: u8) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = rusb::Context::new()?;
    let devs = usb::find_devices(&ctx);
    if devs.is_empty() {
        eprintln!("No WCH BLE Analyzer MCUs found (VID 0x{:04X} / PID 0x{:04X}).", WCH_VID, WCH_PID_BLE_MCU);
        eprintln!("Check USB connection and udev rules.");
        std::process::exit(1);
    }
    eprintln!("Found {} MCU device(s).", devs.len());

    let mut mcus: Vec<McuDevice> = devs;
    let mut opened = 0;
    for dev in &mut mcus {
        match usb::open_device(dev, &ctx) {
            Ok(()) => {
                eprintln!("Opened bus={} addr={}", dev.bus, dev.addr);
                opened += 1;
            }
            Err(e) => {
                eprintln!("open bus={} addr={}: {}", dev.bus, dev.addr, e);
            }
        }
    }
    if opened == 0 {
        eprintln!("Could not open any device.");
        std::process::exit(1);
    }

    let phy_mode = match phy {
        2 => PhyMode::Phy2M,
        3 => PhyMode::CodedS8,
        4 => PhyMode::CodedS2,
        _ => PhyMode::Phy1M,
    };

    let n_devs = mcus.len();
    for (i, dev) in mcus.iter().enumerate() {
        if !dev.is_open {
            continue;
        }
        let mut cfg = CaptureConfig {
            channel: channel,
            phy: phy_mode,
        };
        // Auto-assign one adv channel per MCU
        if cfg.channel == 0 && n_devs > 1 && i < 3 {
            cfg.channel = ADV_CH[i];
        }

        if let Err(e) = usb::start_capture(dev, &cfg) {
            eprintln!("start_capture bus={} addr={}: {}", dev.bus, dev.addr, e);
        } else if n_devs > 1 && cfg.channel != 0 {
            eprintln!("  MCU {} (bus={} addr={}): BLE ch{}", i, dev.bus, dev.addr, cfg.channel);
        }
    }

    // Signal handling
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let _ = ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    });

    // Write pcap header to stdout
    let stdout = io::stdout();
    let mut out = stdout.lock();
    pcap::write_pcap_header(&mut out)?;

    eprintln!("Capturing... press Ctrl+C to stop.");

    let mut pkt_count: u64 = 0;

    // Main capture loop: drain + idle
    while running.load(Ordering::SeqCst) {
        let mut any_data = false;

        // Phase 1: drain
        for dev in mcus.iter_mut() {
            if !dev.is_open {
                continue;
            }
            loop {
                let n = usb::read_packets(dev, &mut |hdr, pdu| {
                    pkt_count += 1;
                    let rf_ch = ble_ch_to_rf(hdr.channel_index);

                    if verbose {
                        eprintln!(
                            "[{:>12} us] ch{:02}  {:<22}  rssi {:4} dBm  AA {:08X}  {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}  PDU[{}]: {}",
                            hdr.timestamp_us,
                            hdr.channel_index,
                            usb::pkt_type_name(hdr.pkt_type),
                            hdr.rssi,
                            hdr.access_addr,
                            hdr.src_addr[5], hdr.src_addr[4], hdr.src_addr[3],
                            hdr.src_addr[2], hdr.src_addr[1], hdr.src_addr[0],
                            pdu.len(),
                            pdu.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ")
                        );
                    }

                    if let Err(e) = pcap::write_pcap_packet(&mut out, rf_ch, hdr.rssi, hdr.access_addr, pdu) {
                        eprintln!("pcap write error: {}", e);
                    }
                });
                match n {
                    Ok(0) => break,
                    Ok(_) => any_data = true,
                    Err(_) => break,
                }
            }
        }

        // Phase 2: idle wait
        if !any_data && running.load(Ordering::SeqCst) {
            for dev in mcus.iter_mut() {
                if !dev.is_open {
                    continue;
                }
                let _ = usb::read_packets(dev, &mut |hdr, pdu| {
                    pkt_count += 1;
                    let rf_ch = ble_ch_to_rf(hdr.channel_index);

                    if verbose {
                        eprintln!(
                            "[{:>12} us] ch{:02}  {:<22}  rssi {:4} dBm  AA {:08X}  {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}  PDU[{}]: {}",
                            hdr.timestamp_us,
                            hdr.channel_index,
                            usb::pkt_type_name(hdr.pkt_type),
                            hdr.rssi,
                            hdr.access_addr,
                            hdr.src_addr[5], hdr.src_addr[4], hdr.src_addr[3],
                            hdr.src_addr[2], hdr.src_addr[1], hdr.src_addr[0],
                            pdu.len(),
                            pdu.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ")
                        );
                    }

                    if let Err(e) = pcap::write_pcap_packet(&mut out, rf_ch, hdr.rssi, hdr.access_addr, pdu) {
                        eprintln!("pcap write error: {}", e);
                    }
                });
            }
        }
    }

    // Stop and clean up
    eprintln!("\nStopping capture ({} packets)...", pkt_count);
    for dev in mcus.iter_mut() {
        if !dev.is_open {
            continue;
        }
        let _ = usb::stop_capture(dev);
        eprintln!(
            "  bus={} addr={}: rx={} err={}",
            dev.bus, dev.addr, dev.rx_count, dev.err_count
        );
        usb::close_device(dev);
    }

    out.flush()?;
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprint!("{}", HELP_TEXT);
        std::process::exit(1);
    }

    let mut extcap_mode = false;
    let mut verbose = false;
    let mut channel: u8 = 0;
    let mut phy: u8 = 1;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--extcap-interfaces" => {
                extcap_interfaces();
                return;
            }
            "--extcap-dlts" => {
                extcap_dlts();
                return;
            }
            "--extcap-interface" | "--extcap-if" => {
                // Next arg is interface name, skip it
                extcap_mode = true;
                i += 1;
            }
            "--extcap-config" => {
                extcap_config();
                return;
            }
            "--extcap-capture" => {
                extcap_mode = true;
            }
            "--extcap-filter" => {
                i += 1; // skip filter expression
            }
            "--fifo" => {
                i += 1; // skip fifo path (we write to stdout)
            }
            "--channel" => {
                i += 1;
                if i < args.len() {
                    channel = args[i].parse().unwrap_or(0);
                }
            }
            "--phy" => {
                i += 1;
                if i < args.len() {
                    phy = args[i].parse().unwrap_or(1);
                }
            }
            "-v" => {
                verbose = true;
            }
            "-h" | "--help" => {
                eprint!("{}", HELP_TEXT);
                return;
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                eprint!("{}", HELP_TEXT);
                std::process::exit(1);
            }
        }
        i += 1;
    }

    if extcap_mode {
        if let Err(e) = extcap_capture(verbose, channel, phy) {
            eprintln!("Fatal: {}", e);
            std::process::exit(1);
        }
    } else {
        eprint!("{}", HELP_TEXT);
    }
}
