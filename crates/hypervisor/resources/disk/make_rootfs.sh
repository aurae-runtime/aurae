#!/bin/bash

# Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

# This script illustrates the build steps for disk images used with the
# reference VMM.

set -e

SOURCE=$(readlink -f "$0")
TEST_RESOURCE_DIR="$(dirname "$SOURCE")"

# Reset index for cmdline arguments for the following `getopts`.
OPTIND=1
# Flag for optionally cleaning the workdir and recompiling the kernel.
CLEAN=
# Working directory. Defaults to a unique tmpdir.
WORKDIR=$(mktemp -d)
# Name of the resulting disk file. Defaults to "rootfs.ext4".
DISKFILE="rootfs.ext4"
# Directory containing .deb packages for the Linux image.
DEBDIR=
# Disk size. Currently hardcoded to 1 GiB.
DISKSIZE="1G"
# Disk mountpoint. The disk file will be mounted here and filled with data.
DISKMNT="mnt/rootfs"
# The Ubuntu release we'll use to build the rootfs. Hardcoded to focal (fossa, 20.04).
UBUNTUVER="focal"
# Hostname for the guest image we're building.
HOSTNAME="ubuntu-rust-vmm"
# Installation script.
INSTALL_SCRIPT="$TEST_RESOURCE_DIR/install_system.sh"

USAGE="
Usage: $(basename $SOURCE) -d debdir [-w workdir] [-o diskfile] [-v version] [-s size] [-c] [-n]

Options:
  -d debdir         Directory containing .deb packages for the Linux image.
  -w workdir        Working directory for the kernel build.
  -o diskfile       Name of the resulting disk file.
  -v version        The Ubuntu version desired. Defaults to focal.
  -s disk size      The size of the resulting rootfs. This needs to be
                    an integer followed by an unit (10K, 500M, 1G).
  -c                Clean up the working directory after the build.
  -n                Do not perform default system installation.
"
export USAGE

while getopts ":cd:w:o:v:s:n" opt; do
    case "$opt" in
    c)  CLEAN=1
        ;;
    d)  DEBDIR="$OPTARG"
        ;;
    w)  rm -rf "$WORKDIR"
        WORKDIR="$OPTARG"
        ;;
    o)  DISKFILE="$OPTARG"
        ;;
    v)  UBUNTUVER="$OPTARG"
        ;;
    s)  DISKSIZE="$OPTARG"
        ;;
    n)  NOINSTALL=1
        ;;
    *)  echo "$USAGE"
        exit 1
    esac
done
shift $((OPTIND-1))

die() {
    echo "[ERROR] $1"
    echo "$USAGE"
    exit 1
}

cleanup() {
    if [ -n "$CLEAN" ]; then
        echo "Cleaning $WORKDIR..."
        rm -rf "$WORKDIR"
    fi
}

cleanup

# Create an empty file for the disk.
mkdir -p "$WORKDIR"
truncate -s "$DISKSIZE" "$WORKDIR/$DISKFILE"
mkfs.ext4 -F "$WORKDIR/$DISKFILE"

# Create a mountpoint for the disk.
mkdir -p "$WORKDIR/$DISKMNT"

# Mount.
mount "$WORKDIR/$DISKFILE" "$WORKDIR/$DISKMNT" # Needs to be root.

# Download Ubuntu packages inside the mountpoint. We'll use the focal fossa (20.04) release.
# Needs to be root.
debootstrap --include openssh-server "$UBUNTUVER" "$WORKDIR/$DISKMNT"

# Set a hostname.
echo "$HOSTNAME" > "$WORKDIR/$DISKMNT/etc/hostname"

# The serial getty service hooks up the login prompt to the kernel console at
# ttyS0 (where the reference VMM connects its serial console).
# We'll set it up for autologin to avoid the login prompt.
mkdir "$WORKDIR/$DISKMNT/etc/systemd/system/serial-getty@ttyS0.service.d/"
cat <<EOF > "$WORKDIR/$DISKMNT/etc/systemd/system/serial-getty@ttyS0.service.d/autologin.conf"
[Service]
ExecStart=
ExecStart=-/sbin/agetty --autologin root -o '-p -- \\u' --keep-baud 115200,38400,9600 %I $TERM
EOF

if [ -z "$NOINSTALL" ]; then
    [ ! -d "$DEBDIR" ] && die "$DEBDIR does not exist."

    # OS is bootstrapped now, time to install the kernel packages.
    # This is done from inside a chroot, to trick dpkg.
    # First, copy the .deb packages inside the chroot folder, in /mnt/root.
    mkdir -p "$WORKDIR/$DISKMNT/mnt/root/"
    cp "$DEBDIR"/*.deb "$WORKDIR/$DISKMNT/mnt/root/"

    # Copy the script that calls dpkg (and some other things) inside the chroot.
    cp "$INSTALL_SCRIPT" "$WORKDIR/$DISKMNT/install_system.sh"

    # Chroot.
    chroot "$WORKDIR/$DISKMNT" /bin/bash "/install_system.sh"
fi

# Unmount.
umount "$WORKDIR/$DISKMNT"

echo "Done!"
echo "Disk placed in $WORKDIR/$DISKFILE."

cleanup
exit 0
