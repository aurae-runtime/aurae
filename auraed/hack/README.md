auraed VM hacking tools
=======================

This directory includes scripts to run auraed as the pid 1 process within a VM.

    make build-container
    make kernel
    make initramfs

    # create `vm-br0` bridge on your machine:
    make network

    # run auraed in a VM as pid 1:
    make virsh-start virsh-console virsh-stop

    # exit VM console with Ctrl+]


As auraed is dynamically compiled, we need to copy all linked libraries into the initramfs. To get a consistent result across different build machines, a build container based on Debian is used (`make build-container`) to build auraed. Also the libraries will be copied from this container into the initramfs (`make initramfs`).

The Linux kernel is built from source (`make kernel`) with a custom kernel config. You can specify the used kernel version in `hack/kernel/config.sh` and modify the kernel config using `make menuconfig`.

The virtual machine has a virtio-net NIC attached which will be connected to a Linux bridge on the host system. You can create and configure this bridge using `make network`. The NIC will appear as `eth0` within the VM.

With the make target `virsh-start` the VM will be created and started. `virsh-console` brings you into the serial console of the VM (you can exit it with `Ctrl+]`). To stop and destroy the VM call `make virsh-stop` - you'll probably want to concatenate those commands to `make virsh-start virsh-console virsh-stop`. This will start the VM, opens the serial console and waits for you to hit `Ctrl+]` to exit the serial console and destroy the VM.