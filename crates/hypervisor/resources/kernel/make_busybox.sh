#!/bin/bash

# Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

# This script contains functions for compiling a Busybox initramfs.

die() {
    echo "[ERROR] $1"
    echo "$USAGE" # To be filled by the caller.
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
#   make_busybox                    \
#       /path/to/busybox/workdir    \
#       /path/to/busybox/config     \
#       busybox_version             \
#       [num_cpus_build]
make_busybox() {
    workdir="$1"
    config="$2"
    busybox_version="$3"
    nprocs="$4"

    [ -z "$workdir" ] && die "Workdir for busybox build not specified."
    [ -z "$config" ] && die "Busybox config file not specified."
    [ ! -f "$config" ] && die "Busybox config file not found."
    [ -z "$busybox_version" ] && die "Busybox version not specified."
    [ -z "$nprocs" ] && nprocs=1

    busybox="busybox-$busybox_version"
    busybox_archive="$busybox.tar.bz2"
    busybox_url="https://busybox.net/downloads/$busybox_archive"

    # Move to the work directory.
    mkdir -p "$workdir"
    echo "Changing working directory to $workdir..."
    pushd_quiet "$workdir"

    # Prepare busybox.
    echo "Downloading busybox..."
    mkdir -p busybox_rootfs
    [ -f "$busybox_archive" ] || curl "$busybox_url" > "$busybox_archive"

    echo "Extracting busybox..."
    tar --skip-old-files -xf "$busybox_archive"
    # Move to the busybox sources directory.
    pushd_quiet "$busybox"

    # Build statically linked busybox.
    cp "$config" .config
    echo "Building busybox..."
    make -j "$nprocs"
    # Package all artefacts somewhere else.

    echo "Packaging busybox..."
    make CONFIG_PREFIX=../busybox_rootfs install

    # Back to workdir.
    popd_quiet

    # Back to wherever we were before.
    popd_quiet
}

# Usage:
#   make_init [halt_value]
make_init() {
    halt="$1"
    # Make an init script.
    cd ..
    echo "Creating init script..."
    cat >init <<EOF
#!/bin/sh
mount -t proc none /proc
mount -t sysfs none /sys
/bin/echo "                                                   "
/bin/echo "                 _                                 "
/bin/echo "  _ __ _   _ ___| |_    __   ___ __ ___  _ __ ___  "
/bin/echo " | '__| | | / __| __|___\ \ / / '_ \ _ \| '_ \ _ \ "
/bin/echo " | |  | |_| \__ \ ||_____\ V /| | | | | | | | | | |"
/bin/echo " |_|   \__,_|___/\__|     \_/ |_| |_| |_|_| |_| |_|"
/bin/echo "                                                   "
/bin/echo "                                                   "
/bin/echo "Hello, world, from the rust-vmm reference VMM!"
EOF

    if [ -z "$halt" ]; then
        echo "setsid /bin/sh -c 'exec /bin/sh </dev/ttyS0 >/dev/ttyS0 2>&1'" >>init
    else
        echo "exec /sbin/halt" >>init
    fi
}
