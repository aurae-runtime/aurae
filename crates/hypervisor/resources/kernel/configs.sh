#!/bin/bash

# Copyright 2021 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

# The busybox version, kernel version and config parameters are exported
# from this script. The current busybox version is compatible with glibc
# version > 2.31.

arch=$(uname -m)
KERNEL_VERSION="5.4.81"

if [[ $arch = "x86_64" ]]; then
	KERNEL_CFG="microvm-kernel-initramfs-hello-x86_64.config"
elif [[ $arch = "aarch64" ]]; then
	KERNEL_CFG="microvm-kernel-initramfs-hello-aarch64.config"
fi

BUSYBOX_CFG="busybox_1_32_1_static_config"
BUSYBOX_VERSION="1.32.1"

echo "Busybox Version: $BUSYBOX_VERSION"
echo "Config: $BUSYBOX_CFG"
