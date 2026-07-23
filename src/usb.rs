use rusb::{Context, DeviceHandle, UsbContext};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// Wireshark surfaces anything an extcap plugin writes to stderr in an error
/// dialog, so protocol diagnostics stay quiet unless -v was passed.
static VERBOSE: AtomicBool = AtomicBool::new(false);

pub fn set_verbose(on: bool) {
    VERBOSE.store(on, Ordering::SeqCst);
}

fn verbose() -> bool {
    VERBOSE.load(Ordering::SeqCst)
}

pub const WCH_VID: u16 = 0x1A86;
pub const WCH_PID_BLE_MCU: u16 = 0x8009;

const EP_BULK_IN: u8 = 0x82;
const EP_BULK_OUT: u8 = 0x02;
const BULK_TRANSFER_SIZE: usize = 0x2800;
const BULK_TIMEOUT: Duration = Duration::from_millis(1000);

const WCH_MAGIC: u8 = 0xAA;
const CMD_IDENTIFY: u8 = 0x84;
const CMD_BLE_CONFIG: u8 = 0x81;
const CMD_SCAN_START: u8 = 0xA1;

const FRAME_MAGIC: u8 = 0x55;
const FRAME_TYPE_DATA: u8 = 0x10;
const FRAME_TYPE_STS: u8 = 0x01;
const MIN_DATA_PAYLOAD: usize = 18;
const BLE_ADV_AA: u32 = 0x8E89BED6;

const IAP_STR: [u8; 15] = *b"BLEAnalyzer&IAP";

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PktHeader {
    pub rssi: i8,
    pub pkt_type: u8,
    pub direction: u8,
    pub access_addr: u32,
    pub src_addr: [u8; 6],
    pub dst_addr: [u8; 6],
    pub timestamp_us: u64,
    pub interval_us: u64,
    pub pkt_index: u64,
    pub channel_index: u8,
}

pub struct McuDevice {
    pub bus: u8,
    pub addr: u8,
    pub handle: Option<DeviceHandle<Context>>,
    pub is_open: bool,
    pub rx_count: u64,
    pub err_count: u64,
    ts_prev_us: u32,
    ts_hi_us: u64,
    pkt_seq: u64,
}

impl McuDevice {
    fn new(bus: u8, addr: u8) -> Self {
        Self {
            bus,
            addr,
            handle: None,
            is_open: false,
            rx_count: 0,
            err_count: 0,
            ts_prev_us: 0,
            ts_hi_us: 0,
            pkt_seq: 0,
        }
    }

    fn bulk_write(&self, data: &[u8]) -> rusb::Result<usize> {
        let handle = self.handle.as_ref().ok_or(rusb::Error::NoDevice)?;
        handle.write_bulk(EP_BULK_OUT, data, BULK_TIMEOUT)
    }

    fn bulk_read(&self, buf: &mut [u8]) -> rusb::Result<(usize, Duration)> {
        let handle = self.handle.as_ref().ok_or(rusb::Error::NoDevice)?;
        let timeout = BULK_TIMEOUT;
        let len = handle.read_bulk(EP_BULK_IN, buf, timeout)?;
        Ok((len, Duration::ZERO))
    }
}

pub fn find_devices(ctx: &Context) -> Vec<McuDevice> {
    let mut devs = Vec::new();
    let device_list = match ctx.devices() {
        Ok(list) => list,
        Err(_) => return devs,
    };
    for device in device_list.iter() {
        let desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue,
        };
        if desc.vendor_id() == WCH_VID && desc.product_id() == WCH_PID_BLE_MCU {
            devs.push(McuDevice::new(device.bus_number(), device.address()));
        }
    }
    devs
}

pub fn open_device(dev: &mut McuDevice, ctx: &Context) -> rusb::Result<()> {
    let device_list = ctx.devices()?;
    for device in device_list.iter() {
        if device.bus_number() == dev.bus && device.address() == dev.addr {
            let handle = device.open()?;
            // Try to detach kernel driver
            if handle.kernel_driver_active(0).unwrap_or(false) {
                let _ = handle.detach_kernel_driver(0);
            }
            handle.claim_interface(0)?;
            dev.handle = Some(handle);
            dev.is_open = true;
            dev.rx_count = 0;
            dev.err_count = 0;
            dev.ts_prev_us = 0;
            dev.ts_hi_us = 0;
            dev.pkt_seq = 0;
            return Ok(());
        }
    }
    Err(rusb::Error::NoDevice)
}

pub fn close_device(dev: &mut McuDevice) {
    if let Some(ref handle) = dev.handle {
        let _ = handle.release_interface(0);
    }
    dev.handle = None;
    dev.is_open = false;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PhyMode {
    Phy1M = 1,
    Phy2M = 2,
    CodedS8 = 3,
    CodedS2 = 4,
}

pub struct CaptureConfig {
    pub channel: u8,
    #[allow(dead_code)]
    pub phy: PhyMode,
}

fn send_identify(dev: &McuDevice) -> rusb::Result<()> {
    let mut frame = vec![0u8; 4 + 4 + 15];
    frame[0] = WCH_MAGIC;
    frame[1] = CMD_IDENTIFY;
    frame[2] = 0x13;
    frame[3] = 0x00;
    frame[8..23].copy_from_slice(&IAP_STR);
    dev.bulk_write(&frame)?;

    let mut resp = [0u8; 64];
    match dev.bulk_read(&mut resp) {
        Ok((n, _)) if n >= 1 && verbose() => {
            eprintln!(
                "[wch bus={} addr={}] AA84 response[0]=0x{:02X} ({})",
                dev.bus,
                dev.addr,
                resp[0],
                if resp[0] != 0 {
                    "firmware present"
                } else {
                    "no firmware?"
                }
            );
        }
        _ => {}
    }
    Ok(())
}

fn send_ble_config(dev: &McuDevice, cfg: &CaptureConfig) -> rusb::Result<()> {
    let mut frame = vec![0u8; 4 + 25];
    frame[0] = WCH_MAGIC;
    frame[1] = CMD_BLE_CONFIG;
    frame[2] = 0x19;
    frame[3] = 0x00;
    frame[4] = 0xFF;
    frame[5] = 0x01;
    frame[6] = cfg.channel;
    frame[15] = 0xd6;
    frame[16] = 0xbe;
    frame[17] = 0x89;
    frame[18] = 0x8e;
    frame[19] = 0x55;
    frame[20] = 0x55;
    frame[21] = 0x55;
    frame[22] = 0x10;
    dev.bulk_write(&frame)?;

    let mut resp = [0u8; 64];
    let _ = dev.bulk_read(&mut resp);
    Ok(())
}

fn send_start_scan(dev: &McuDevice) -> rusb::Result<()> {
    let frame = [WCH_MAGIC, CMD_SCAN_START, 0x00, 0x00];
    dev.bulk_write(&frame)?;

    let mut resp = [0u8; 64];
    match dev.bulk_read(&mut resp) {
        Ok((n, _)) if n >= 1 && verbose() => {
            eprintln!(
                "[wch bus={} addr={}] AA A1 response: {} bytes (magic=0x{:02X} type=0x{:02X})",
                dev.bus,
                dev.addr,
                n,
                resp[0],
                if n > 1 { resp[1] } else { 0 }
            );
        }
        _ => {}
    }
    Ok(())
}

pub fn start_capture(dev: &McuDevice, cfg: &CaptureConfig) -> rusb::Result<()> {
    send_identify(dev)?;
    send_ble_config(dev, cfg)?;
    send_start_scan(dev)?;
    Ok(())
}

pub fn stop_capture(dev: &McuDevice) -> rusb::Result<()> {
    let frame = [WCH_MAGIC, CMD_SCAN_START, 0x00, 0x00];
    match dev.bulk_write(&frame) {
        Ok(_) => Ok(()),
        Err(rusb::Error::Timeout) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn read_packets<F>(dev: &mut McuDevice, cb: &mut F) -> rusb::Result<usize>
where
    F: FnMut(&PktHeader, &[u8]),
{
    let mut buf = vec![0u8; BULK_TRANSFER_SIZE];
    let xfer = match dev.bulk_read(&mut buf) {
        Ok((n, _)) => n,
        Err(rusb::Error::Timeout) => return Ok(0),
        Err(e) => return Err(e),
    };

    if xfer < 4 {
        return Ok(0);
    }

    let mut offset = 0;
    let mut decoded = 0;

    while offset + 4 <= xfer {
        if buf[offset] != FRAME_MAGIC {
            offset += 1;
            continue;
        }

        let ftype = buf[offset + 1];
        let plen = buf[offset + 2] as usize | ((buf[offset + 3] as usize) << 8);
        let frame_size = 4 + plen;

        if offset + frame_size > xfer {
            break;
        }

        // Filter junk: non-zero byte at payload[5] (reserved field)
        if offset + 9 < xfer && buf[offset + 9] != 0 {
            offset += frame_size;
            continue;
        }

        // Status echo: skip
        if ftype == FRAME_TYPE_STS {
            offset += frame_size;
            dev.err_count += 1;
            continue;
        }

        // Unknown type: skip
        if ftype != FRAME_TYPE_DATA {
            offset += 1;
            continue;
        }

        // Need minimum payload
        if plen < MIN_DATA_PAYLOAD {
            offset += frame_size;
            continue;
        }

        let p = offset + 4; // payload start

        let channel = buf[p + 4];
        if channel > 39 {
            offset += frame_size;
            continue;
        }

        let ts32 = buf[p] as u32
            | ((buf[p + 1] as u32) << 8)
            | ((buf[p + 2] as u32) << 16)
            | ((buf[p + 3] as u32) << 24);
        let rssi = buf[p + 8] as i8;
        let pdu_hdr0 = buf[p + 10];
        let pdu_plen = buf[p + 11];
        let pkt_type = pdu_hdr0 & 0x0F;
        let flags = buf[p + 5];

        // Extend 32-bit timestamp to 64-bit
        if ts32 < dev.ts_prev_us {
            dev.ts_hi_us += 0x1_0000_0000u64;
        }
        let ts64 = dev.ts_hi_us | ts32 as u64;
        let dt = ts64 - (dev.ts_hi_us | dev.ts_prev_us as u64);
        dev.ts_prev_us = ts32;

        let mut src_addr = [0u8; 6];
        let mut dst_addr = [0u8; 6];
        if p + 18 <= xfer {
            src_addr.copy_from_slice(&buf[p + 12..p + 18]);
        }
        if (pkt_type == 0x03 || pkt_type == 0x05)
            && pdu_plen >= 12
            && plen >= 18 + 6
            && p + 24 <= xfer
        {
            dst_addr.copy_from_slice(&buf[p + 18..p + 24]);
        }

        let pdu_start = p + 10;
        let avail = plen.saturating_sub(10);
        let mut pdu_len = 2 + pdu_plen as usize;
        if pdu_len > avail {
            pdu_len = avail;
        }
        let pdu_end = std::cmp::min(pdu_start + pdu_len, xfer);
        let pdu = buf[pdu_start..pdu_end].to_vec();

        let hdr = PktHeader {
            rssi,
            pkt_type,
            direction: flags & 0x01,
            access_addr: BLE_ADV_AA,
            src_addr,
            dst_addr,
            timestamp_us: ts64,
            interval_us: dt,
            pkt_index: {
                let idx = dev.pkt_seq;
                dev.pkt_seq += 1;
                idx
            },
            channel_index: channel,
        };

        dev.rx_count += 1;
        cb(&hdr, &pdu);
        decoded += 1;

        offset += frame_size;
    }

    Ok(decoded)
}

pub fn pkt_type_name(t: u8) -> &'static str {
    match t {
        0x00 => "ADV_IND",
        0x01 => "ADV_DIRECT_IND",
        0x02 => "ADV_NONCONN_IND",
        0x03 => "SCAN_REQ",
        0x04 => "SCAN_RSP",
        0x05 => "CONNECT_REQ",
        0x06 => "ADV_SCAN_IND",
        0x07 => "AUX_SCAN_REQ",
        0x08 => "AUX_CONNECT_REQ",
        0x09 => "AUX_COMMON",
        0x0A => "AUX_ADV_IND",
        0x0B => "AUX_SCAN_RSP",
        0x0C => "AUX_SYNC_IND",
        0x0D => "AUX_CONNECT_RSP",
        0x0E => "AUX_CHAIN_IND",
        _ => "UNKNOWN",
    }
}
