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
    MPQemuCmd, PciConfDataMsg, SyncSysmemMsg, PCI_CONF_DATA_MSG_SIZE, SYNC_SYSMEM_SIZE,
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
                let _msg: PciConfDataMsg = read_pci_conf_msg(&mut stream)?;

                println!("TODO: handle msg");

                stream
                    .write_all(&mpqemu::ret_data())
                    .context("unable to write response")?;
            }
            MPQemuCmd::PCI_CFGREAD => {
                let _msg: PciConfDataMsg = read_pci_conf_msg(&mut stream)?;

                println!("TODO: handle msg");

                stream
                    .write_all(&mpqemu::ret_u64(u64::MAX))
                    .context("unable to write response")?;
            }
            MPQemuCmd::BAR_WRITE => todo!(),
            MPQemuCmd::BAR_READ => todo!(),
            MPQemuCmd::SET_IRQFD => {
                assert_eq!(hdr.size, 0);

                println!("discarding SET_IRQFD");
            }
            MPQemuCmd::DEVICE_RESET => todo!(),
        }
    }
}
