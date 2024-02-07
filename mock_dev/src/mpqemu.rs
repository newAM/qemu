//! Types in this module are from include/hw/remote/mpqemu-link.h

use std::{
    mem::{size_of, transmute},
    os::raw::{c_int as int, c_uint as unsigned},
};

const REMOTE_MAX_FDS: usize = 8;

#[repr(C)]
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum MPQemuCmd {
    SYNC_SYSMEM,
    RET,
    PCI_CFGWRITE,
    PCI_CFGREAD,
    BAR_WRITE,
    BAR_READ,
    SET_IRQFD,
    DEVICE_RESET,
    // MAX,
}

impl TryFrom<int> for MPQemuCmd {
    type Error = int;

    fn try_from(val: int) -> Result<Self, Self::Error> {
        match val {
            x if x == (Self::SYNC_SYSMEM as int) => Ok(Self::SYNC_SYSMEM),
            x if x == (Self::RET as int) => Ok(Self::RET),
            x if x == (Self::PCI_CFGWRITE as int) => Ok(Self::PCI_CFGWRITE),
            x if x == (Self::PCI_CFGREAD as int) => Ok(Self::PCI_CFGREAD),
            x if x == (Self::BAR_WRITE as int) => Ok(Self::BAR_WRITE),
            x if x == (Self::BAR_READ as int) => Ok(Self::BAR_READ),
            x if x == (Self::SET_IRQFD as int) => Ok(Self::SET_IRQFD),
            x if x == (Self::DEVICE_RESET as int) => Ok(Self::DEVICE_RESET),
            // x if x == (Self::MAX as int) => Ok(Self::MAX),
            _ => Err(val),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SyncSysmemMsg {
    gpas: [u64; REMOTE_MAX_FDS],
    sizes: [u64; REMOTE_MAX_FDS],
    offsets: [i64; REMOTE_MAX_FDS],
}

pub const SYNC_SYSMEM_SIZE: usize = size_of::<SyncSysmemMsg>();

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct PciConfDataMsg {
    pub addr: u32,
    pub val: u32,
    pub len: int,
}

pub const PCI_CONF_DATA_MSG_SIZE: usize = size_of::<PciConfDataMsg>();

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct BarAccessMsg {
    addr: u64,
    val: u64,
    size: u32,
    memory: u32,
}

#[repr(C)]
pub union Data {
    _u64: u64,
    pci_conf_data: PciConfDataMsg,
    sync_sysmem: SyncSysmemMsg,
    bar_access: BarAccessMsg,
}

#[repr(C)]
#[derive(Debug)]
pub struct MsgHeader {
    pub cmd: int,
    pub size: usize,
}

impl MsgHeader {
    pub const RET_NO_DATA: Self = Self {
        cmd: MPQemuCmd::RET as int,
        size: 0,
    };

    pub const RET_U64: Self = Self {
        cmd: MPQemuCmd::RET as int,
        size: size_of::<u64>(),
    };
}

pub const HDR_SIZE: usize = size_of::<MsgHeader>();

#[repr(C)]
#[allow(dead_code)]
pub struct MPQemuMsg {
    pub hdr: MsgHeader,
    pub data: Data,
    pub fds: [int; REMOTE_MAX_FDS],
    pub num_fds: int,
}

pub fn ret_data() -> Vec<u8> {
    let mut buf: Vec<u8> = Default::default();
    buf.extend_from_slice(unsafe {
        transmute::<&MsgHeader, &[u8; HDR_SIZE]>(&MsgHeader::RET_NO_DATA)
    });
    buf
}

pub fn ret_u64(val: u64) -> Vec<u8> {
    let mut buf: Vec<u8> = Default::default();
    buf.extend_from_slice(unsafe { transmute::<&MsgHeader, &[u8; HDR_SIZE]>(&MsgHeader::RET_U64) });
    buf.extend_from_slice(&val.to_ne_bytes());
    buf
}
