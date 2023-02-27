# Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause
"""Run the reference VMM and shut it down through a command on the serial."""
import fcntl
import json
import os
import subprocess
import tempfile
import platform

import time
from subprocess import PIPE

import pytest

from tools.s3 import s3_download

# No. of seconds after which to give up for the test
TEST_TIMEOUT = 30

arch = platform.machine()

def process_exists(pid):
    try:
        os.kill(pid, 0)
    except OSError:
        return False
    else:
        return True


def default_busybox_bzimage():
    return s3_download(
        resource_type="kernel",
        resource_name="bzimage-hello-busybox",
        first=True
    )


def default_busybox_elf():
    return s3_download(
        resource_type="kernel",
        resource_name="vmlinux-hello-busybox",
        first=True
    )


def default_ubuntu_bzimage():
    return s3_download(
        resource_type="kernel",
        resource_name="bzimage-focal",
        first=True
    )


def default_ubuntu_elf():
    return s3_download(
        resource_type="kernel",
        resource_name="vmlinux-focal",
        first=True
    )

def default_ubuntu_pe():
    return s3_download(
        resource_name="pe-focal",
        resource_type="kernel",
        first=True
    )

def default_busybox_pe():
    return s3_download(
        resource_name="pe-hello-busybox",
        resource_type="kernel",
        first=True
    )

def default_disk():
    return s3_download(
        resource_name=f"ubuntu-focal-rootfs-{arch}.ext4",
        resource_type="disk",
        first=True
    )

if arch == "aarch64":
    KERNELS_INITRAMFS = [default_busybox_pe()]

    UBUNTU_KERNEL_DISK_PAIRS = [
        (default_ubuntu_pe() , default_disk())
    ]
else:
    KERNELS_INITRAMFS = [
        default_busybox_elf(),
        default_busybox_bzimage()
    ]

    UBUNTU_KERNEL_DISK_PAIRS = [
        (default_ubuntu_elf(), default_disk()),
        (default_ubuntu_bzimage(), default_disk()),
    ]


"""
The following methods would be nice to have a part of class revolving
around the vmm process. Let's figure out how to proceed here as part
of the discussion around making the CI/testing easier to use, extend,
and run locally.
"""


def start_vmm_process(kernel_path, disk_path=None, num_vcpus=1, mem_size_mib=1024 ,default_cmdline=False):
    # Kernel config
    cmdline = "console=ttyS0 i8042.nokbd reboot=t panic=1 pci=off"

    kernel_load_addr = 1048576

    build_cmd = "cargo build --release"
    subprocess.run(build_cmd, shell=True, check=True)
    vmm_cmd = [
        "target/release/vmm-reference",
        "--memory", "size_mib={}".format(mem_size_mib),
        "--vcpu", "num={}".format(num_vcpus),
        "--kernel"
    ]
    if default_cmdline:
        kernel_config = "path={}".format(
            kernel_path
        )
    else:
        kernel_config = "cmdline={},path={}".format(
            cmdline, kernel_path
        )
    if arch == "x86_64":
        kernel_config += ",kernel_load_addr={}".format(kernel_load_addr)
    
    vmm_cmd.append(kernel_config)
    
    tmp_file_path = None

    if disk_path is not None:
        # Terrible hack to have a rootfs owned by the user.
        with tempfile.NamedTemporaryFile(dir='/tmp', delete=True) as tmpfile:
            tmp_file_path = tmpfile.name
        cp_cmd = "cp {} {}".format(disk_path, tmp_file_path)
        subprocess.run(cp_cmd, shell=True, check=True)
        vmm_cmd.append("--block")
        vmm_cmd.append("path={}".format(tmp_file_path))

    vmm_process = subprocess.Popen(vmm_cmd, stdout=PIPE, stdin=PIPE)

    # Let's quickly check if the process died (i.e. because of invalid vmm
    # configuration). We need to wait here because otherwise the returncode
    # will be None even if the `vmm_process` died.
    try:
        vmm_process.wait(timeout=2)
    except subprocess.TimeoutExpired:
        # The process is still alive.
        pass

    assert process_exists(vmm_process.pid)

    return vmm_process, tmp_file_path


def shutdown(vmm_process):
    vmm_process.stdin.write(b'reboot -f\n')
    vmm_process.stdin.flush()

    # If the process hasn't ended within 3 seconds, this will raise a
    # TimeoutExpired exception and fail the test.
    vmm_process.wait(timeout=3)


def setup_stdout_nonblocking(vmm_process):
    # We'll need to do non-blocking I/O with the underlying sub-process since
    # we cannot use `communicate`, because `communicate` would close the
    # `stdin` that we later want to use to `shutdown`, to do that by hand,
    # we set `vmm_process.stdout` to non-blocking
    # Then we can use `os.read` that would raise `BlockingIOError`

    # FIXME: This should NOT be required to be done on every call, do it when we
    #        'Class'ify the test case
    flags = fcntl.fcntl(vmm_process.stdout, fcntl.F_GETFL)
    fcntl.fcntl(vmm_process.stdout, fcntl.F_SETFL, flags | os.O_NONBLOCK)


def expect_string(vmm_process, expected_string, timeout=TEST_TIMEOUT):
    setup_stdout_nonblocking(vmm_process)

    # No. of seconds after which we'll give up
    giveup_after = timeout
    then = time.time()
    found = False
    # This is required because the pattern we are expecting might get split across two reads
    all_data = bytes()
    while not found:
        try:
            data = os.read(vmm_process.stdout.fileno(), 4096)
            all_data += data
            for line in all_data.split(b'\r\n'):
                if expected_string in line.decode():
                    found = True
                    return line.decode()
            # Whatever remains is collected in `all_data`.
            all_data = line
        except BlockingIOError as _:
            # Raised on `EWOULDBLOCK`, so it's better to wait for sometime before retrying
            time.sleep(1)
            now = time.time()
            if now - then > giveup_after:
                raise TimeoutError(
                    "Timed out {} waiting for {}".format(now - then, expected_string))
        except Exception as _:
            raise


def run_cmd_inside_vm(cmd, vmm_process, prompt, timeout=5):
    """Runs a command inside the VM process and returns the output.

       If the command runs successfully, output of the command is returned or else a
       suitable exception is raised. The `timeout` parameter is used to indicate how
       much time to wait for the command to complete.

       Note: `cmd` and `prompt` should be a `bytes` object and not `str`.
    """
    cmd = cmd.strip() + b'\r\n'

    vmm_process.stdin.write(cmd)
    vmm_process.stdin.flush()

    then = time.time()
    giveup_after = timeout
    all_output = []
    while True:
        try:
            data = os.read(vmm_process.stdout.fileno(), 4096)
            output_lines = data.split(b'\r\n')
            last = output_lines[-1].strip()
            if prompt in last:
                # FIXME: WE get the prompt twice in the output at the end,
                # So removing it. No idea why twice?
                # First one is 'cmd'
                
                all_output.extend(output_lines[1:-2])
                return all_output
            else:
                all_output.extend(output_lines)
        except BlockingIOError as _:
            time.sleep(1)
            now = time.time()
            if now - then > giveup_after:
                raise TimeoutError(
                        "Timed out {} waiting for {}".format(now - then, cmd.decode()))
        except Exception as e:
            raise


def expect_vcpus(vmm_process, expected_vcpus):
    # Actually following is not required because this function will be called after
    # `expect_string` is called once, which sets non-blocking, but let's not be
    # dependent on it, so it's just fine to call it again, less than ideal, but not
    # wrong.
    setup_stdout_nonblocking(vmm_process)
    # We check for the prompt at the start so that the inital stdout of the process
    # gets cleared up, which is printed while the machine is booted.
    prompt = '/ #'
    expect_string(vmm_process, prompt)
    
    # /proc/cpuinfo displays info about each vCPU
    cmd = 'cat /proc/cpuinfo'
    output = run_cmd_inside_vm(cmd.encode(), vmm_process, prompt.encode(), timeout=5)
    actual_vcpus = 0
    for line in output:
        if "processor" in line.decode():
            actual_vcpus +=1

    assert actual_vcpus == expected_vcpus, \
        "Expected {}, found {} vCPUs".format(expected_vcpus, actual_vcpus)


def expect_mem(vmm_process, expected_mem_mib):
    expected_mem_kib = expected_mem_mib << 10

    # Extract memory information from the bootlog.
    # Example:
    # [    0.000000] Memory: 496512K/523896K available (8204K kernel
    # code, 646K rwdata, 1480K rodata, 2884K init, 2792K bss, 27384K reserved,
    # 0K cma-reserved)
    # The second value (523896K) is the initial guest memory in KiB, which we
    # will compare against the expected memory specified during VM creation.
    memory_string = expect_string(vmm_process, "Memory:")
    actual_mem_kib = int(memory_string.split('/')[1].split('K')[0])

    # Expect the difference between the expected and actual memory
    # to be a few hundred KiB.  For the guest memory sizes being tested, this
    # should be under 0.1% of the expected memory size.
    normalized_diff = (expected_mem_kib - float(actual_mem_kib)) / expected_mem_kib
    assert normalized_diff < 0.001, \
        "Expected {} KiB, found {} KiB of guest" \
        " memory".format(expected_mem_kib, actual_mem_kib)


def test_expect_string_timeout():
    """ Verifies that a timeout error is raised when not finding the expected
    string in the VMM output."""

    kernel = KERNELS_INITRAMFS[0]
    vmm_process, _ = start_vmm_process(kernel)

    with pytest.raises(TimeoutError) as e:
        # Let's expect a string that cannot show up as part of booting the
        # vmm reference.
        expected_string = "Goodbye, world, from the rust-vmm reference VMM!"
        expect_string(vmm_process, expected_string, timeout=20)

    shutdown(vmm_process)

    assert e.type is TimeoutError


@pytest.mark.parametrize("kernel", KERNELS_INITRAMFS)
def test_reference_vmm(kernel):
    """Start the reference VMM and verify that it works."""

    vmm_process, _ = start_vmm_process(kernel)

    # Poll process for new output until we find the hello world message.
    # If we do not find the expected message, this loop will not break and the
    # test will fail when the timeout expires.
    expected_string = "Hello, world, from the rust-vmm reference VMM!"
    expect_string(vmm_process, expected_string)

    shutdown(vmm_process)

@pytest.mark.parametrize("kernel", KERNELS_INITRAMFS)
def test_reference_vmm_with_deault_cmdline(kernel):
    """Start the reference VMM with default cmdline and verify that it works."""

    vmm_process, _ = start_vmm_process(kernel,default_cmdline=True)

    # Poll process for new output until we find the hello world message.
    # If we do not find the expected message, this loop will not break and the
    # test will fail when the timeout expires.
    expected_string = "Hello, world, from the rust-vmm reference VMM!"
    expect_string(vmm_process, expected_string)

    shutdown(vmm_process)

@pytest.mark.parametrize("kernel,disk", UBUNTU_KERNEL_DISK_PAIRS)
def test_reference_vmm_with_disk(kernel, disk):
    """Start the reference VMM with a block device and verify that it works."""

    vmm_process, tmp_disk_path = start_vmm_process(kernel, disk_path=disk)

    prompt = 'root@ubuntu-rust-vmm:~#'
    expect_string(vmm_process, prompt)

    cmd = 'lsblk --json'
    output = run_cmd_inside_vm(cmd.encode(), vmm_process, prompt.encode(), timeout=5)

    shutdown(vmm_process)
    if tmp_disk_path is not None:
        os.remove(tmp_disk_path)

    output = b''.join(output).decode()
    blockdevs_dict = json.loads(output)
    assert blockdevs_dict["blockdevices"][0]["name"] == "vda"
    assert blockdevs_dict["blockdevices"][0]["ro"] == False


@pytest.mark.parametrize("kernel", KERNELS_INITRAMFS)
def test_reference_vmm_num_vcpus(kernel):
    """Start the reference VMM and verify the number of vCPUs."""

    num_vcpus = [1, 2, 4]

    for expected_vcpus in num_vcpus:
        # Start a VM with a specified number of vCPUs
        vmm_process, _ = start_vmm_process(kernel, num_vcpus=expected_vcpus)

        # Poll the output from /proc/cpuinfo for the field displaying the the
        # number of vCPUs.
        #
        expect_vcpus(vmm_process, expected_vcpus)

        shutdown(vmm_process)


@pytest.mark.parametrize("kernel", KERNELS_INITRAMFS)
def test_reference_vmm_mem(kernel):
    """Start the reference VMM and verify the amount of guest memory."""

    # Test small and large guest memory sizes, as well as sizes around the
    # beginning of the MMIO gap, which require a partition of guest memory.
    #
    # The MMIO gap sits in 768 MiB at the end of the first 4GiB of memory, and
    # we want to ensure memory is correctly partitioned; therefore, in addition
    # to memory sizes that fit well below the and extend well beyond the gap,
    # we will test the edge cases around the start of the gap.
    # See 'vmm/src/lib.rs:create_guest_memory()`
    mmio_gap_end = 1 << 32
    mmio_gap_size = 768 << 20
    mmio_gap_start = mmio_gap_end - mmio_gap_size
    mmio_gap_start_mib = mmio_gap_start >> 20

    mem_sizes_mib = [
        512,
        mmio_gap_start_mib - 1,
        mmio_gap_start_mib,
        mmio_gap_start_mib + 1,
        8192]

    for expected_mem_mib in mem_sizes_mib:
        # Start a VM with a specified amount of memory
        vmm_process, _ = start_vmm_process(kernel, mem_size_mib=expected_mem_mib)

        # Poll the output from /proc/meminfo for the field displaying the the
        # total amount of memory.
        #
        # If we do not find the field, this loop will not break and the
        # test will fail when the timeout expires.  If we find the field, but
        # the expected and actual guest memory sizes diverge by more than 0.1%,
        # the test will fail immediately with an explanation.
        expect_mem(vmm_process, expected_mem_mib)

        shutdown(vmm_process)
