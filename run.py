#!/usr/bin/env python3

import asyncio
import socket
import os

this_dir = os.path.dirname(os.path.abspath(__file__))
qemu_system = os.path.join(this_dir, "build", "qemu-system-x86_64")
mock_dev = os.path.join(this_dir, "mock_dev", "target", "debug", "mock_dev")
nixos_iso = os.path.join(this_dir, "build", "latest-nixos-minimal-x86_64-linux.iso")


async def main() -> int:
    qemu_sock, dev_sock = socket.socketpair(socket.AF_UNIX, socket.SOCK_STREAM)
    os.set_inheritable(qemu_sock.fileno(), True)
    os.set_inheritable(dev_sock.fileno(), True)

    qemu_proc = await asyncio.create_subprocess_exec(
        qemu_system,
        # fmt: off
        "-machine", "pc",
        "-cpu", "host",
        "-accel", "kvm",
        "-object", "memory-backend-memfd,id=sysmem-file,size=2G",
        "--numa", "node,memdev=sysmem-file",
        "-m", "2048",
        "-display", "gtk",
        "-cdrom", nixos_iso,
        "--trace", "mpqemu*",
        "--trace", "pci*",
        "-D", "qemu.log",
        "-device", "x-pci-proxy-dev,id=what,fd=" + str(qemu_sock.fileno()),
        # fmt: on
        close_fds=False,
    )

    dev_proc = await asyncio.create_subprocess_exec(
        mock_dev, str(dev_sock.fileno()), close_fds=False
    )

    await asyncio.gather(qemu_proc.wait(), dev_proc.wait())

    return 0


if __name__ == "__main__":
    rc = asyncio.run(main())
    exit(rc)
