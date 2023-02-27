#!/bin/bash

# Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

# This script illustrates the steps for installing an OS image in a disk used
# by the reference VMM.
# It's called from inside a chroot in the disk image being provisioned.

# We expect .deb packages containing the Linux image to be present in
# /mnt/root. Install them from there.
DEBIAN_FRONTEND=noninteractive apt-get -y install /mnt/root/linux*.deb

# Delete root's password.
passwd -d root

exit 0
