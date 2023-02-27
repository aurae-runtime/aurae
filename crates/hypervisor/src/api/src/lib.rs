// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause
//! CLI for the Reference VMM.

#![deny(missing_docs)]
use std::result;

use clap::{App, Arg};
use vmm::VMMConfig;

/// Command line parser.
pub struct Cli;

impl Cli {
    /// Parses the command line options into VMM configurations.
    ///
    /// # Arguments
    ///
    /// * `cmdline_args` - command line arguments passed to the application.
    pub fn launch(cmdline_args: Vec<&str>) -> result::Result<VMMConfig, String> {
        let mut app = App::new(cmdline_args[0].to_string())
            .arg(
                Arg::with_name("memory")
                    .long("memory")
                    .takes_value(true)
                    .help("Guest memory configuration.\n\tFormat: \"size_mib=<u32>\""),
            )
            .arg(
                Arg::with_name("vcpu")
                    .long("vcpu")
                    .takes_value(true)
                    .help("vCPU configuration.\n\tFormat: \"num=<u8>\""),
            )
            .arg(
                Arg::with_name("kernel")
                    .long("kernel")
                    .required(true)
                    .takes_value(true)
                    .help("Kernel configuration.\n\tFormat: \"path=<string>[,cmdline=<string>,kernel_load_addr=<u64>]\""),
            )
            .arg(
                Arg::with_name("net")
                    .long("net")
                    .takes_value(true)
                    .help("Network device configuration. \n\tFormat: \"tap=<string>\"")
            )
            .arg(
                Arg::with_name("block")
                    .long("block")
                    .required(false)
                    .takes_value(true)
                    .help("Block device configuration. \n\tFormat: \"path=<string>\"")
            );

        // Save the usage beforehand as a string, because `get_matches` consumes the `App`.
        let mut help_msg_buf: Vec<u8> = vec![];
        // If the write fails, we'll just have an empty help message.
        let _ = app.write_long_help(&mut help_msg_buf);
        let help_msg = String::from_utf8_lossy(&help_msg_buf);

        let matches = app.get_matches_from_safe(cmdline_args).map_err(|e| {
            eprintln!("{}", help_msg);
            format!("Invalid command line arguments: {}", e)
        })?;

        VMMConfig::builder()
            .memory_config(matches.value_of("memory"))
            .kernel_config(matches.value_of("kernel"))
            .vcpu_config(matches.value_of("vcpu"))
            .net_config(matches.value_of("net"))
            .block_config(matches.value_of("block"))
            .build()
            .map_err(|e| format!("{:?}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use linux_loader::cmdline::Cmdline;

    use vmm::{KernelConfig, MemoryConfig, VcpuConfig, DEFAULT_KERNEL_LOAD_ADDR};

    #[test]
    fn test_launch() {
        // Missing command line arguments.
        assert!(Cli::launch(vec!["foobar"]).is_err());

        // Invalid extra command line parameter.
        assert!(Cli::launch(vec![
            "foobar",
            "--memory",
            "size_mib=128",
            "--vcpu",
            "num=1",
            "--kernel",
            "path=/foo/bar,cmdline=\"foo=bar\",kernel_load_addr=42",
            "foobar",
        ])
        .is_err());

        // Invalid memory config: invalid value for `size_mib`.
        assert!(Cli::launch(vec![
            "foobar",
            "--memory",
            "size_mib=foobar",
            "--vcpu",
            "num=1",
            "--kernel",
            "path=/foo/bar,cmdline=\"foo=bar\",kernel_load_addr=42",
        ])
        .is_err());

        // Memory config: missing value for `size_mib`, use the default
        assert!(Cli::launch(vec![
            "foobar",
            "--memory",
            "size_mib=",
            "--vcpu",
            "num=1",
            "--kernel",
            "path=/foo/bar,cmdline=\"foo=bar\",kernel_load_addr=42",
        ])
        .is_ok());

        // Invalid memory config: unexpected parameter `foobar`.
        assert!(Cli::launch(vec![
            "foobar",
            "--memory",
            "foobar=1024",
            "--vcpu",
            "num=1",
            "--kernel",
            "path=/foo/bar,cmdline=\"foo=bar\",kernel_load_addr=42",
        ])
        .is_err());

        // Invalid kernel config: invalid value for `kernel_load_addr`.
        // TODO: harden cmdline check.
        assert!(Cli::launch(vec![
            "foobar",
            "--memory",
            "size_mib=128",
            "--vcpu",
            "num=1",
            "--kernel",
            "path=/foo/bar,cmdline=\"foo=bar\",kernel_load_addr=foobar",
        ])
        .is_err());

        // Kernel config: missing value for `kernel_load_addr`, use default
        assert!(Cli::launch(vec![
            "foobar",
            "--memory",
            "size_mib=128",
            "--vcpu",
            "num=1",
            "--kernel",
            "path=/foo/bar,cmdline=\"foo=bar\",kernel_load_addr=",
        ])
        .is_ok());

        // Invalid kernel config: unexpected parameter `foobar`.
        assert!(Cli::launch(vec![
            "foobar",
            "--memory",
            "size_mib=128",
            "--vcpu",
            "num=1",
            "--kernel",
            "path=/foo/bar,cmdline=\"foo=bar\",kernel_load_addr=42,foobar=42",
        ])
        .is_err());

        // Invalid vCPU config: invalid value for `num_vcpus`.
        assert!(Cli::launch(vec![
            "foobar",
            "--memory",
            "size_mib=128",
            "--vcpu",
            "num=foobar",
            "--kernel",
            "path=/foo/bar,cmdline=\"foo=bar\",kernel_load_addr=42",
        ])
        .is_err());

        // vCPU config: missing value for `num_vcpus`, use default
        assert!(Cli::launch(vec![
            "foobar",
            "--memory",
            "size_mib=128",
            "--vcpu",
            "num=",
            "--kernel",
            "path=/foo/bar,cmdline=\"foo=bar\",kernel_load_addr=42",
        ])
        .is_ok());

        // Invalid vCPU config: unexpected parameter `foobar`.
        assert!(Cli::launch(vec![
            "foobar",
            "--memory",
            "size_mib=128",
            "--vcpu",
            "foobar=1",
            "--kernel",
            "path=/foo/bar,cmdline=\"foo=bar\",kernel_load_addr=42",
        ])
        .is_err());

        let mut foo_cmdline = Cmdline::new(4096);
        foo_cmdline.insert_str("\"foo=bar bar=foo\"").unwrap();

        // OK.
        assert_eq!(
            Cli::launch(vec![
                "foobar",
                "--memory",
                "size_mib=128",
                "--vcpu",
                "num=1",
                "--kernel",
                "path=/foo/bar,cmdline=\"foo=bar bar=foo\",kernel_load_addr=42",
            ])
            .unwrap(),
            VMMConfig {
                kernel_config: KernelConfig {
                    path: PathBuf::from("/foo/bar"),
                    cmdline: foo_cmdline,
                    load_addr: 42,
                },
                memory_config: MemoryConfig { size_mib: 128 },
                vcpu_config: VcpuConfig { num: 1 },
                block_config: None,
                net_config: None,
            }
        );

        // Test default values.
        assert_eq!(
            Cli::launch(vec!["foobar", "--kernel", "path=/foo/bar",]).unwrap(),
            VMMConfig {
                kernel_config: KernelConfig {
                    path: PathBuf::from("/foo/bar"),
                    cmdline: KernelConfig::default_cmdline(),
                    load_addr: DEFAULT_KERNEL_LOAD_ADDR,
                },
                memory_config: MemoryConfig { size_mib: 256 },
                vcpu_config: VcpuConfig { num: 1 },
                block_config: None,
                net_config: None,
            }
        );
    }
}
