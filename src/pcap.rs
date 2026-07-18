use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};

pub const DLT_BLUETOOTH_LE_LL_WITH_PHDR: u32 = 256;

const PCAP_MAGIC: u32 = 0xA1B2C3D4;
const PCAP_VERSION_MAJ: u16 = 2;
const PCAP_VERSION_MIN: u16 = 4;
const PCAP_SNAPLEN: u32 = 65535;

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
struct PcapFileHdr {
    magic: u32,
    version_major: u16,
    version_minor: u16,
    thiszone: i32,
    sigfigs: u32,
    snaplen: u32,
    network: u32,
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
struct PcapRecHdr {
    ts_sec: u32,
    ts_usec: u32,
    incl_len: u32,
    orig_len: u32,
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
struct BlePhdr {
    rf_channel: u8,
    signal_power: i8,
    noise_power: i8,
    access_address_offenses: u8,
    reference_access_address: u32,
    flags: u16,
}

pub fn write_pcap_header<W: Write>(out: &mut W) -> io::Result<()> {
    let hdr = PcapFileHdr {
        magic: PCAP_MAGIC,
        version_major: PCAP_VERSION_MAJ,
        version_minor: PCAP_VERSION_MIN,
        thiszone: 0,
        sigfigs: 0,
        snaplen: PCAP_SNAPLEN,
        network: DLT_BLUETOOTH_LE_LL_WITH_PHDR,
    };
    let bytes = unsafe { std::slice::from_raw_parts(&hdr as *const _ as *const u8, std::mem::size_of::<PcapFileHdr>()) };
    out.write_all(bytes)?;
    out.flush()?;
    Ok(())
}

pub fn write_pcap_packet<W: Write>(
    out: &mut W,
    rf_channel: u8,
    rssi: i8,
    access_addr: u32,
    pdu: &[u8],
) -> io::Result<()> {
    // DEWHITENED | SIGPOWER_VALID | REF_AA_VALID | CHECKSUM_INSPECTED | CHECKSUM_VALID
    let flags: u16 = 0x0001 | 0x0002 | 0x0010 | 0x0400 | 0x0800;

    let ph = BlePhdr {
        rf_channel,
        signal_power: rssi,
        noise_power: 0x80u8 as i8,
        access_address_offenses: 0,
        reference_access_address: access_addr,
        flags,
    };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let ts_sec = now.as_secs() as u32;
    let ts_usec = now.subsec_micros();

    let aa_le = access_addr;
    let crc = [0u8; 3];

    let data_len = std::mem::size_of::<BlePhdr>() as u32 + 4 + pdu.len() as u32 + 3;

    let rh = PcapRecHdr {
        ts_sec,
        ts_usec,
        incl_len: data_len,
        orig_len: data_len,
    };

    // Write record header
    let bytes = unsafe {
        std::slice::from_raw_parts(&rh as *const _ as *const u8, std::mem::size_of::<PcapRecHdr>())
    };
    out.write_all(bytes)?;

    // Write pseudo-header
    let bytes = unsafe {
        std::slice::from_raw_parts(&ph as *const _ as *const u8, std::mem::size_of::<BlePhdr>())
    };
    out.write_all(bytes)?;

    // Access address (LE)
    out.write_all(&aa_le.to_le_bytes())?;

    // BLE LL PDU
    if !pdu.is_empty() {
        out.write_all(pdu)?;
    }

    // CRC placeholder
    out.write_all(&crc)?;

    out.flush()?;
    Ok(())
}
