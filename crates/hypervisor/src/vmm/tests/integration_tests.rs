// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause
use std::convert::TryFrom;
use std::path::PathBuf;

use utils::resource_download::s3_download;
use vmm::{KernelConfig, MemoryConfig, VMMConfig, VcpuConfig, Vmm, DEFAULT_KERNEL_LOAD_ADDR};

fn default_memory_config() -> MemoryConfig {
    MemoryConfig { size_mib: 1024 }
}

fn default_kernel_config(path: PathBuf) -> KernelConfig {
    KernelConfig {
        path,
        load_addr: DEFAULT_KERNEL_LOAD_ADDR, // 1 MB
        cmdline: KernelConfig::default_cmdline(),
    }
}

fn default_vcpu_config() -> VcpuConfig {
    VcpuConfig { num: 1 }
}

fn run_vmm(kernel_path: PathBuf) {
    let vmm_config = VMMConfig {
        kernel_config: default_kernel_config(kernel_path),
        memory_config: default_memory_config(),
        vcpu_config: default_vcpu_config(),
        block_config: None,
        net_config: None,
    };

    let mut vmm = Vmm::try_from(vmm_config).unwrap();
    vmm.run().unwrap();
}

#[test]
#[cfg(target_arch = "x86_64")]
fn test_dummy_vmm_elf() {
    let tags = r#"
    {
        "halt_after_boot": true,
        "image_format": "elf",
        "with_disk": false
    }
    "#;

    let elf_halt = s3_download("kernel", Some(tags)).unwrap();
    run_vmm(elf_halt);
}

#[test]
#[cfg(target_arch = "x86_64")]
fn test_dummy_vmm_bzimage() {
    let tags = r#"
    {
        "halt_after_boot": true,
        "image_format": "bzimage",
        "with_disk": false
    }
    "#;
    let bzimage_halt = s3_download("kernel", Some(tags)).unwrap();
    run_vmm(bzimage_halt);
}

#[test]
#[cfg(target_arch = "aarch64")]
fn test_dummy_vmm_pe() {
    let tags = r#"
    {
        "halt_after_boot": true,
        "image_format": "pe",
        "with_disk": false
    }
    "#;
    let pe_halt = s3_download("kernel", Some(tags)).unwrap();
    run_vmm(pe_halt);
}
