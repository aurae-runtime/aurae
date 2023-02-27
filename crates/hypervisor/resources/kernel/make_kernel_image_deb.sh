#!/bin/bash

# Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

# This script illustrates the build steps for kernel images used with the
# reference VMM.

set -e

arch=$(uname -m)
SOURCE=$(readlink -f "$0")
TEST_RESOURCE_DIR="$(dirname "$SOURCE")"

trap "exit 1" TERM
export TOP_PID=$$

source "$TEST_RESOURCE_DIR/make_kernel.sh"

KERNEL_VERSION="5.4.81"

if [[ $arch = "x86_64" ]]; then
	KERNEL_CFG="microvm-kernel-5.4-x86_64.config"
elif [[ $arch = "aarch64" ]]; then
	KERNEL_CFG="microvm-kernel-5.4-aarch64.config"

fi

# Reset index for cmdline arguments for the following `getopts`.
OPTIND=1
# Flag for optionally building a guest that halts.
HALT=
# Number of CPUs to use during the kernel build process.
MAKEPROCS=1
# Flag for optionally cleaning the workdir and recompiling the kernel.
CLEAN=
# Working directory. Defaults to a unique tmpdir.
WORKDIR=$(mktemp -d)
# Kernel binary format.
KERNEL_FMT=
# Destination kernel binary name.
KERNEL_BINARY_NAME=

USAGE="
Usage: $(basename $SOURCE) -f (elf|bzimage) [-j nprocs] [-k kernel] [-w workdir] [-c] [-h]

Options:
  -f elf|bzimage    Kernel image format (either elf or bzimage).
  -j nprocs         Number of CPUs to use for the kernel build.
  -k kernel         Name of the resulting kernel image. Has the '-halt' suffix if '-h' is passed.
  -w workdir        Working directory for the kernel build.
  -c                Clean up the working directory after the build.
  -h                Create a kernel image that halts immediately after boot.
"
export USAGE

while getopts ":chf:j:k:w:" opt; do
    case "$opt" in
    c)  CLEAN=1
        ;;
    h)  HALT=1
        ;;
    f)  KERNEL_FMT=$(validate_kernel_format "$OPTARG")
        ;;
    j)  MAKEPROCS=$OPTARG
        ;;
    k)  KERNEL_BINARY_NAME=$OPTARG
        ;;
    w)  rm -rf "$WORKDIR"
        WORKDIR=$OPTARG
        ;;
    *)  echo "$USAGE"
        exit 1
    esac
done
shift $((OPTIND-1))

cleanup() {
    if [ -n "$CLEAN" ]; then
        echo "Cleaning $WORKDIR..."
        rm -rf "$WORKDIR"
    fi
}

# Step 0: clean up the workdir, if the user wants to.
cleanup

# Step 1: what are we building?
[ -z "$KERNEL_BINARY_NAME" ] && KERNEL_BINARY_NAME=$(kernel_binary "$KERNEL_FMT")
[ -n "$HALT" ] && KERNEL_BINARY_NAME="$KERNEL_BINARY_NAME-halt"

# Step 2: start from scratch.
mkdir -p "$WORKDIR" && cd "$WORKDIR"

# Step 3: acquire kernel sources & config.
extract_kernel_srcs "$KERNEL_VERSION"
kernel_dir="$WORKDIR/linux-$KERNEL_VERSION"
make_kernel_config "$TEST_RESOURCE_DIR/$KERNEL_CFG" "$kernel_dir"

# Step 4: build *.deb packages.
# We could build them from the kernel sources with `make deb-pkg`, but the
# `deb-pkg` target forces a `make clean`, so we'll just download them instead
# of building the entire kernel over and over again.
# If, however, you want to build them from scratch, set the `MAKEDEB` env var.
mkdir -p "$kernel_dir/deb"
if [ -n "$MAKEDEB" ]; then
    echo "Building deb packages..."
    make_kernel "$kernel_dir" "$KERNEL_FMT" "deb-pkg" "$MAKEPROCS"
    cp "$kernel_dir"/../*.deb "$kernel_dir/deb"
else
    echo "Downloading deb packages..."
    if [[ $arch = "x86_64" ]]; then
        DEB_URL="http://security.ubuntu.com/ubuntu/pool/main/l/linux-hwe-5.4"
        DEBS=(
            "linux-modules-5.4.0-42-generic_5.4.0-42.46~18.04.1_amd64.deb"
            "linux-image-unsigned-5.4.0-42-generic_5.4.0-42.46~18.04.1_amd64.deb"
        )
    elif [[ $arch = "aarch64" ]]; then
        DEB_URL="http://ports.ubuntu.com/pool/main/l/linux-hwe-5.4"
        DEBS=(
            "linux-modules-5.4.0-42-generic_5.4.0-42.46~18.04.1_arm64.deb"
            "linux-image-unsigned-5.4.0-42-generic_5.4.0-42.46~18.04.1_arm64.deb"
        )
    fi
    for deb in "${DEBS[@]}"; do
        wget -nc -P "$kernel_dir/deb" "$DEB_URL/$deb"
    done
fi

# Step 5: build kernel.
target=$(kernel_target "$KERNEL_FMT")
make_kernel "$kernel_dir" "$KERNEL_FMT" "$target" "$MAKEPROCS" "$KERNEL_BINARY_NAME"

# Final step: profit!
echo "Done!"
echo "Kernel binary placed in: $kernel_dir/$KERNEL_BINARY_NAME"
echo ".deb packages placed in:"
ls -1 "$kernel_dir"/deb/*.deb

cleanup
exit 0
