use std::{
    io::{Read, Write},
    mem::transmute,
    os::{
        fd::{FromRawFd, RawFd},
        unix::net::UnixStream,
    },
};

use anyhow::Context;
use mpqemu::{MsgHeader, HDR_SIZE};

use crate::mpqemu::{
    BarAccessMsg, MPQemuCmd, PciConfDataMsg, SyncSysmemMsg, PCI_CONF_DATA_MSG_SIZE,
    SYNC_SYSMEM_SIZE,
};

mod mpqemu;

fn read_pci_conf_msg(stream: &mut UnixStream) -> anyhow::Result<PciConfDataMsg> {
    let mut msg: PciConfDataMsg = PciConfDataMsg::default();
    stream
        .read_exact(unsafe {
            transmute::<&mut PciConfDataMsg, &mut [u8; PCI_CONF_DATA_MSG_SIZE]>(&mut msg)
        })
        .context("unable to read PciConfDataMsg")?;
    println!("msg={:X?}", msg);
    Ok(msg)
}

fn read_bar_access_msg(stream: &mut UnixStream) -> anyhow::Result<BarAccessMsg> {
    let mut msg: BarAccessMsg = BarAccessMsg::default();
    stream
        .read_exact(unsafe {
            transmute::<&mut BarAccessMsg, &mut [u8; PCI_CONF_DATA_MSG_SIZE]>(&mut msg)
        })
        .context("unable to read BarAccessMsg")?;
    println!("msg={:X?}", msg);
    Ok(msg)
}

fn main() -> anyhow::Result<()> {
    let mut args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        let usage: String = format!("Usage: {} fd", args[0]);
        anyhow::bail!(usage)
    }
    let fd: RawFd = args
        .pop()
        .unwrap()
        .parse()
        .context("Argument is not an FD")?;

    let mut stream: UnixStream = unsafe { UnixStream::from_raw_fd(fd) };

    let mut bar: [u32; 7] = [0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0];

    loop {
        let mut hdr_buf: [u8; HDR_SIZE] = [0; HDR_SIZE];
        stream.read_exact(&mut hdr_buf)?;

        let hdr: MsgHeader = unsafe { transmute::<[u8; HDR_SIZE], MsgHeader>(hdr_buf) };

        let cmd: MPQemuCmd = MPQemuCmd::try_from(hdr.cmd)
            .ok()
            .with_context(|| format!("invalid command {}", hdr.cmd))?;

        println!("cmd={:?} len={}", cmd, hdr.size);

        // FIXME: will need recv_vectored_with_ancillary to read fds

        match cmd {
            MPQemuCmd::SYNC_SYSMEM => {
                assert_eq!(hdr.size, SYNC_SYSMEM_SIZE);
                let mut msg: SyncSysmemMsg = SyncSysmemMsg::default();
                stream
                    .read_exact(unsafe {
                        transmute::<&mut SyncSysmemMsg, &mut [u8; SYNC_SYSMEM_SIZE]>(&mut msg)
                    })
                    .context("unable to read SYNC_SYSMEM")?;
                println!("msg={:X?}", msg);

                stream
                    .write_all(&mpqemu::ret_data())
                    .context("unable to write response")?;
            }
            MPQemuCmd::RET => todo!(),
            MPQemuCmd::PCI_CFGWRITE => {
                let msg: PciConfDataMsg = read_pci_conf_msg(&mut stream)?;

                match msg.addr {
                    0x10 => bar[0] = 0xffffff01,
                    0x14 => bar[1] = 0xfffffc00,
                    0x18 => bar[2] = 0,
                    0x1C => bar[3] = 0,
                    0x20 => bar[4] = 0,
                    0x24 => bar[5] = 0,
                    0x28 => bar[5] = 0,
                    0x30 => {}
                    _ => println!("TODO: handle msg"),
                };

                stream
                    .write_all(&mpqemu::ret_data())
                    .context("unable to write response")?;
            }
            MPQemuCmd::PCI_CFGREAD => {
                let msg: PciConfDataMsg = read_pci_conf_msg(&mut stream)?;

                let ret: u64 = match msg.addr {
                    0x00 => 0x1000, // VID
                    0x02 => 0x0001, // PID
                    0x04 => 0x0,
                    0x08 => 0x1000000,
                    0x0A => 0x0100, // SCSI class ID
                    0x0E => 0x0,
                    0x10 => bar[0].into(),
                    0x14 => bar[1].into(),
                    0x18 => bar[2].into(),
                    0x1C => bar[3].into(),
                    0x20 => bar[4].into(),
                    0x24 => bar[5].into(),
                    0x2E => 0x1000, // subsystem ID
                    0x28 => bar[6].into(),
                    0x30 => 0x0,
                    0x3C => 0xb,
                    0x3D => 0x1,
                    _ => {
                        println!("TODO: handle msg");
                        u64::MAX
                    }
                };

                stream
                    .write_all(&mpqemu::ret_u64(ret))
                    .context("unable to write response")?;
            }
            MPQemuCmd::BAR_WRITE => {
                let _msg: BarAccessMsg = read_bar_access_msg(&mut stream)?;

                println!("TODO: handle msg");

                stream
                    .write_all(&mpqemu::ret_data())
                    .context("unable to write response")?;
            }
            MPQemuCmd::BAR_READ => {
                let msg: BarAccessMsg = read_bar_access_msg(&mut stream)?;

                println!("TODO: handle msg");

                stream
                    .write_all(&mpqemu::ret_u64(u64::MAX))
                    .context("unable to write response")?;
            }
            MPQemuCmd::SET_IRQFD => {
                assert_eq!(hdr.size, 0);

                println!("TODO: discarding SET_IRQFD");
            }
            MPQemuCmd::DEVICE_RESET => {
                println!("TODO: device reset");
                bar = [0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0];

                stream
                    .write_all(&mpqemu::ret_data())
                    .context("unable to write response")?;
            }
        }
    }
}
