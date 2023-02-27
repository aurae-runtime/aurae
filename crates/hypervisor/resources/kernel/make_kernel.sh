#!/bin/bash

# Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

# This script contains functions for compiling a Linux kernel and initramfs.

KERNEL_URL_BASE="https://cdn.kernel.org/pub/linux/kernel"

die() {
    echo "[ERROR] $1" >&2
    echo "$USAGE" >&2 # To be filled by the caller.
    # Kill the caller.
    if [ -n "$TOP_PID" ]; then kill -s TERM "$TOP_PID"; else exit 1; fi
}

pushd_quiet() {
    pushd "$1" &>/dev/null || die "Failed to enter $1."
}

popd_quiet() {
    popd &>/dev/null || die "Failed to return to previous directory."
}

# Usage:
#   extract_kernel_srcs kernel_version
extract_kernel_srcs() {
    kernel_version="$1"

    [ -z "$kernel_version" ] && die "Kernel version not specified."

    # This magic trick gets the major component of the version number.
    kernel_major="${kernel_version%%.*}"
    kernel_archive="linux-$kernel_version.tar.xz"
    kernel_url="$KERNEL_URL_BASE/v$kernel_major.x/$kernel_archive"

    echo "Starting kernel build."
    # Download kernel sources.
    echo "Downloading kernel from $kernel_url"
    [ -f "$kernel_archive" ] || curl "$kernel_url" > "$kernel_archive"
    echo "Extracting kernel sources..."
    tar --skip-old-files -xf "$kernel_archive"
}

# Usage:
#   make_kernel_config /path/to/source/config /path/to/kernel
make_kernel_config() {
    # Copy base kernel config.
    # Add any custom config options, if necessary (currently N/A).
    kernel_config="$1"
    kernel_dir="$2"

    [ -z "$kernel_config" ] && die "Kernel config file not specified."
    [ ! -f "$kernel_config" ] && die "Kernel config file not found."
    [ -z "$kernel_dir" ] && die "Kernel directory not specified."
    [ ! -d "$kernel_dir" ] && die "Kernel directory not found."

    echo "Copying kernel config..."
    cp "$kernel_config" "$kernel_dir/.config"
}

# Usage:
#   make_initramfs              \
#       /path/to/kernel/dir     \
#       /path/to/busybox/rootfs \
#       [halt_value]
make_initramfs() {
    kernel_dir="$1"
    busybox_rootfs="$2"
    halt="$3"

    [ -z "$kernel_dir" ] && die "Kernel directory not specified."
    [ ! -d "$kernel_dir" ] && die "Kernel directory not found."
    [ -z "$busybox_rootfs" ] && die "Busybox rootfs not specified."
    [ ! -d "$busybox_rootfs" ] && die "Busybox rootfs directory not found."

    # Move to the directory with the kernel sources.
    pushd_quiet "$kernel_dir"

    # Prepare initramfs directory.
    mkdir -p initramfs/{bin,dev,etc,home,mnt,proc,sys,usr}
    # Copy busybox.
    echo "Copying busybox to the initramfs directory..."
    cp -r "$busybox_rootfs"/* initramfs/

    # Make a block device and a console.
    pushd_quiet initramfs/dev
    echo "Creating device nodes..."
    rm -f sda && mknod sda b 8 0
    rm -f console && mknod console c 5 1
    rm -f ttyS0 && mknod ttyS0 c 4 64

    make_init "$halt"

    chmod +x init
    fakeroot chown root init

    # Pack it up...
    echo "Packing initramfs.cpio..."
    find . | cpio -H newc -o > ../initramfs.cpio
    fakeroot chown root ../initramfs.cpio

    # Return to kernel srcdir.
    popd_quiet
    # Return to previous directory.
    popd_quiet
}

# Usage: validate_kernel_format format
# Prints the lowercase format name, if one of "elf" or "bzimage"
# for x86 or "pe" for aarch64.
# Exits with error if any other format is specified.
validate_kernel_format() {
    format="$1"
    arch=$(uname -m)

    kernel_fmt=$(echo "$format" | tr '[:upper:]' '[:lower:]')
    if [ $arch = "x86_64" ]; then
        if [ "$kernel_fmt" != "elf" ] && [ "$kernel_fmt" != "bzimage" ]; then
            die "Invalid kernel binary format: $kernel_fmt for this type of architecture."
        fi
    elif [ $arch = "aarch64" ]; then
        if [ "$kernel_fmt" != "pe" ]; then
            die "Invalid kernel binary format: $kernel_fmt for this type of architecture."
        fi
    else
        die "Unsupported architecture!"
    fi
    echo "$kernel_fmt"
}

# Usage: kernel_target format
# Prints the `make` target that builds a kernel of the specified format.
kernel_target() {
    format=$(validate_kernel_format "$1")

    case "$format" in
    elf)        echo "vmlinux"
                ;;
    bzimage)    # This is the default target.
                ;;
    pe)    echo "Image"
                ;;
    esac
}

# Usage: kernel_binary format
# Prints the name of the generated kernel binary.
kernel_binary() {
    format=$(validate_kernel_format "$1")
    arch=$(uname -m)

    if [ $arch = "x86_64" ]; then
        case "$format" in
        elf)        echo "vmlinux"
                    ;;
        bzimage)    echo "arch/x86/boot/bzImage"
                    ;;
        esac
    elif [ $arch = "aarch64" ]; then
        case "$format" in
            pe)        echo "arch/arm64/boot/Image"
                        ;;
        esac
    else
        die "Unsupported architecture!"
    fi
}

# Usage:
#   make_kernel
#       /path/to/kernel/dir \
#       format              \
#       [target]            \
#       [num_cpus_build]    \
#       [/path/to/kernel/destination]
make_kernel() {
    kernel_dir="$1"
    format="$2"
    target="$3"
    nprocs="$4"
    dst="$5"

    [ -z "$kernel_dir" ] && die "Kernel directory not specified."
    [ ! -d "$kernel_dir" ] && die "Kernel directory not found."
    [ -z "$format" ] && die "Kernel format not specified."
    [ -z "$nprocs" ] && nprocs=1

    kernel_binary=$(kernel_binary "$format")

    # Move to the directory with the kernel sources.
    pushd_quiet "$kernel_dir"

    # Build kernel.
    echo "Building kernel..."
    # Missing the quotes for $target is intentional: if the target is empty,
    # quotes will cause it to be passed as an empty string, which `make`
    # doesn't understand. No quotes => no empty string.
    make -j "$nprocs" $target

    if [ -n "$dst" ] && [ "$kernel_binary" != "$dst" ]; then
        # Copy to destination.
        cp "$kernel_binary" "$dst"
    fi

    # Return to previous directory.
    popd_quiet
}
