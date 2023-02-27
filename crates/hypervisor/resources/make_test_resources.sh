#!/bin/bash

# Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

# This script illustrates the build steps for all the kernel and disk images
# used by tests in vmm-reference.

set -e

SOURCE=$(readlink -f "$0")
TEST_RESOURCE_DIR="$(dirname "$SOURCE")"
cd $TEST_RESOURCE_DIR

# If the user doesn't provide a value for a variable, use the default one.
# We build the images in /tmp, so they don't pollute the vmm-reference
# repository, and then move them at the locations expected by the tests.
TMP_KERNEL_DIR=${TMP_KERNEL_DIR:="/tmp/vmlinux_busybox"}
TMP_DEB_DIR=${TMP_DEB_DIR:="/tmp/ubuntu-focal"}
DEST_KERNEL_DIR=${DEST_KERNEL_DIR:="$TEST_RESOURCE_DIR/kernel"}
DEST_DISK_DIR=${DEST_DISK_DIR:="$TEST_RESOURCE_DIR/disk"}
DEST_INITRD_DIR=${DEST_INITRD_DIR:="$TEST_RESOURCE_DIR/initrd"}

# By default use 2 CPUs. User can change this to speed up the builds. 
NPROC=${NPROC:=2}

arch=$(uname -m)
if [[ $arch = "x86_64" ]]; then
    ./kernel/make_kernel_busybox_image.sh -f elf -k vmlinux-hello-busybox -w $TMP_KERNEL_DIR -j $NPROC
    ./kernel/make_kernel_busybox_image.sh -f elf -k vmlinux-hello-busybox -w $TMP_KERNEL_DIR -j $NPROC -h
    ./kernel/make_kernel_busybox_image.sh -f bzimage -k bzimage-hello-busybox -w $TMP_KERNEL_DIR -j $NPROC
    ./kernel/make_kernel_busybox_image.sh -f bzimage -k bzimage-hello-busybox -w $TMP_KERNEL_DIR -j $NPROC -h

    # We are moving the built busybox images to where they are expected to be
    # found by the vmm-reference tests. This is also making it easier to upload
    # the whole `resources` folder to S3.
    mv $TMP_KERNEL_DIR/linux-5.4.81/vmlinux-hello-busybox $TMP_KERNEL_DIR/linux-5.4.81/vmlinux-hello-busybox-halt \
    $TMP_KERNEL_DIR/linux-5.4.81/bzimage-hello-busybox $TMP_KERNEL_DIR/linux-5.4.81/bzimage-hello-busybox-halt \
    $DEST_KERNEL_DIR

    ./kernel/make_kernel_image_deb.sh -f elf -k vmlinux-focal -w $TMP_DEB_DIR -j $NPROC
    ./kernel/make_kernel_image_deb.sh -f bzimage -k bzimage-focal -w $TMP_DEB_DIR -j $NPROC

    # Same applies to the Ubuntu images.
    mv $TMP_DEB_DIR/linux-5.4.81/vmlinux-focal $TMP_DEB_DIR/linux-5.4.81/bzimage-focal $DEST_KERNEL_DIR

    ./disk/make_rootfs.sh -d $TMP_DEB_DIR/linux-5.4.81/deb/ -w $DEST_DISK_DIR -o ubuntu-focal-rootfs-x86_64.ext4
elif [[ $arch = "aarch64" ]]; then
    ./kernel/make_kernel_busybox_image.sh -f pe -k pe-hello-busybox -w $TMP_KERNEL_DIR -j $NPROC
    ./kernel/make_kernel_busybox_image.sh -f pe -k pe-hello-busybox -w $TMP_KERNEL_DIR -j $NPROC -h

    # And same to the aarch64 images.
    mv $TMP_KERNEL_DIR/linux-5.4.81/pe-hello-busybox $TMP_KERNEL_DIR/linux-5.4.81/pe-hello-busybox-halt $DEST_KERNEL_DIR
    mkdir -p $DEST_INITRD_DIR
    mv $TMP_KERNEL_DIR/linux-5.4.81/initramfs.cpio $DEST_INITRD_DIR/aarch64-initramfs.cpio

    # making debian based image
    ./kernel/make_kernel_image_deb.sh -f pe -k pe-focal -w $TMP_DEB_DIR -j $NPROC

    #  move them to correct folder
    mv $TMP_DEB_DIR/linux-5.4.81/pe-focal $DEST_KERNEL_DIR

    ./disk/make_rootfs.sh -d $TMP_DEB_DIR/linux-5.4.81/deb/ -w $DEST_DISK_DIR -o ubuntu-focal-rootfs-aarch64.ext4

fi
