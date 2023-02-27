# Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause
"""Test the scripts that build kernel images for the reference VMM."""

import os, subprocess, sys
import pytest
from tempfile import TemporaryDirectory


# Accepted combinations for:
# (kernel_format, kernel_filename, halt_flag)
OK_COMBOS = [
    ("eLf", None, None),
    ("elf", "foo", None),
    ("elf", "foo", True),
    ("bZiMaGe", None, None),
    ("bzimage", "bar", None),
    ("bzimage", "bar", True),
]


# Malformed combinations for:
# (kernel_format, kernel_filename, halt_flag, expected_error_message)
BAD_COMBOS = [
    ("foo", None, None, '[ERROR] Invalid kernel binary format: foo.'),
]


@pytest.mark.parametrize("fmt,img,hlt", OK_COMBOS)
def test_build_kernel_image(fmt, img, hlt):
    """Build kernel images using the provided scripts."""
    workdir = TemporaryDirectory()
    build_cmd, expected_kernel = _make_script_cmd(workdir.name, fmt, img, hlt)  
    try:
        subprocess.run(build_cmd, check=True)
        assert os.path.isfile(expected_kernel)
    finally:
        workdir.cleanup()


@pytest.mark.parametrize("fmt,img,hlt,errmsg", BAD_COMBOS)
def test_build_kernel_image_err(fmt, img, hlt, errmsg):
    """Attempt to build kernel images with invalid parameters."""
    workdir = TemporaryDirectory()
    build_cmd, _ = _make_script_cmd(workdir.name, fmt, img, hlt)  
    try:
        subprocess.check_output(build_cmd, stderr=subprocess.PIPE)
    except subprocess.CalledProcessError as cpe:
        assert errmsg in cpe.stderr.decode(sys.getfilesystemencoding())
    finally:
        workdir.cleanup()


def _make_script_cmd(workdir, fmt, img, hlt):
    """Compose the command line invocation for the kernel build script."""
    kernel_dir = "linux-4.14.176"
    expected_image_path = os.path.join(workdir, kernel_dir)

    script_path = os.path.abspath(os.path.join(
        os.path.dirname(os.path.realpath(__file__)),
        "..",
        "resources/kernel/make_kernel_busybox_image.sh"
    ))
    
    # Add format. This argument is mandatory.
    script_cmd = [script_path, "-f", fmt, "-w", workdir]
    
    # Add number of CPUs to use for compilation. Not mandatory, but also not
    # easy to verify, so let's always use 2.
    script_cmd.extend(["-j", "2"])

    # Add resulting kernel image name, if specified.
    if img:
        script_cmd.extend(["-k", img])
        expected_image_path = os.path.join(expected_image_path, img)
    else:
        expected_image_path = os.path.join(
            expected_image_path,
            "vmlinux" if fmt.lower() == "elf" else "arch/x86/boot/bzImage"
        )
    
    # Generate a kernel that halts, if specified. The script will append the
    # "-halt" suffix.
    if hlt:
        script_cmd.extend(["-h"])
        expected_image_path = "{}-halt".format(expected_image_path)
    
    return script_cmd, expected_image_path
