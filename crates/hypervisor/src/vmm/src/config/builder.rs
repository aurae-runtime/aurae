// Copyright 2021 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

//! Config builder
use std::convert::TryFrom;

use super::{
    BlockConfig, ConversionError, KernelConfig, MemoryConfig, NetConfig, VMMConfig, VcpuConfig,
};

/// Builder structure for VMMConfig
#[derive(Debug)]
pub struct Builder {
    inner: Result<VMMConfig, ConversionError>,
}

impl Default for Builder {
    fn default() -> Self {
        Builder {
            inner: Ok(VMMConfig::default()),
        }
    }
}

impl Builder {
    /// Creates a Builder Object
    pub fn new() -> Self {
        Builder::default()
    }

    /// Builds `VMMConfig`.
    ///
    /// This function should be called after all the configurations are setup using `*_config`
    /// functions. If any of the required config is missing, this function returns appropriate
    /// error.
    ///
    /// # Example
    ///
    /// ```
    ///  # use vmm::VMMConfig;
    ///
    /// let vmmconfig = VMMConfig::builder()
    ///     .memory_config(Some("size_mib=1024"))
    ///     .vcpu_config(Some("num=1"))
    ///     .kernel_config(Some("path=/path/to/bzImage"))
    ///     .net_config(Some("tap=tap0"))
    ///     .block_config(Some("path=/dev/loop0"))
    ///     .build();
    ///
    /// assert!(vmmconfig.is_ok());
    /// ```
    pub fn build(&self) -> Result<VMMConfig, ConversionError> {
        // Check if there are any errors
        match &self.inner {
            Ok(vc) => {
                // Empty kernel image path.
                if vc.kernel_config.path.to_str().unwrap().is_empty() {
                    return Err(ConversionError::ParseKernel(
                        "Kernel Image Path is Empty.".to_string(),
                    ));
                }
            }
            Err(_) => {}
        }

        self.inner.clone()
    }

    /// Configure Builder with Memory Configuration for the VMM.
    ///
    /// # Example
    ///
    /// You can see example of how to use this function in [`Example` section from
    /// `build`](#method.build)
    pub fn memory_config<T>(self, memory: Option<T>) -> Self
    where
        MemoryConfig: TryFrom<T>,
        <MemoryConfig as TryFrom<T>>::Error: Into<ConversionError>,
    {
        match memory {
            Some(m) => self.and_then(|mut config| {
                config.memory_config = TryFrom::try_from(m).map_err(Into::into)?;
                Ok(config)
            }),
            None => self,
        }
    }

    /// Configure Builder with VCPU Configuration for the VMM.
    ///
    /// # Example
    ///
    /// You can see example of how to use this function in [`Example` section from
    /// `build`](#method.build)
    pub fn vcpu_config<T>(self, vcpu: Option<T>) -> Self
    where
        VcpuConfig: TryFrom<T>,
        <VcpuConfig as TryFrom<T>>::Error: Into<ConversionError>,
    {
        match vcpu {
            Some(v) => self.and_then(|mut config| {
                config.vcpu_config = TryFrom::try_from(v).map_err(Into::into)?;
                Ok(config)
            }),
            None => self,
        }
    }

    /// Configure Builder with Kernel Configuration for the VMM.
    ///
    /// Note: Path argument of the Kernel Configuration is a required argument.
    ///
    /// # Example
    ///
    /// You can see example of how to use this function in [`Example` section from
    /// `build`](#method.build)
    pub fn kernel_config<T>(self, kernel: Option<T>) -> Self
    where
        KernelConfig: TryFrom<T>,
        <KernelConfig as TryFrom<T>>::Error: Into<ConversionError>,
    {
        match kernel {
            Some(k) => self.and_then(|mut config| {
                config.kernel_config = TryFrom::try_from(k).map_err(Into::into)?;
                Ok(config)
            }),
            None => self,
        }
    }

    /// Configure Builder with Network Configuration for the VMM.
    ///
    /// # Example
    ///
    /// You can see example of how to use this function in [`Example` section from
    /// `build`](#method.build)
    pub fn net_config<T>(self, net: Option<T>) -> Self
    where
        NetConfig: TryFrom<T>,
        <NetConfig as TryFrom<T>>::Error: Into<ConversionError>,
    {
        match net {
            Some(n) => self.and_then(|mut config| {
                config.net_config = Some(TryFrom::try_from(n).map_err(Into::into)?);
                Ok(config)
            }),
            None => self,
        }
    }

    /// Configure Builder with Block Device Configuration for the VMM.
    ///
    /// # Example
    ///
    /// You can see example of how to use this function in [`Example` section from
    /// `build`](#method.build)
    pub fn block_config<T>(self, block: Option<T>) -> Self
    where
        BlockConfig: TryFrom<T>,
        <BlockConfig as TryFrom<T>>::Error: Into<ConversionError>,
    {
        match block {
            Some(b) => self.and_then(|mut config| {
                config.block_config = Some(TryFrom::try_from(b).map_err(Into::into)?);
                Ok(config)
            }),
            None => self,
        }
    }

    fn and_then<F>(self, func: F) -> Self
    where
        F: FnOnce(VMMConfig) -> Result<VMMConfig, ConversionError>,
    {
        Builder {
            inner: self.inner.and_then(func),
        }
    }
}

#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use super::*;
    use crate::DEFAULT_KERNEL_LOAD_ADDR;

    #[test]
    fn test_builder_default_err() {
        let vmm_config = Builder::default().build();
        assert!(vmm_config.is_err());
    }

    #[test]
    fn test_builder_memory_config_success() {
        let vmm_config = Builder::default()
            .memory_config(Some("size_mib=1024"))
            .kernel_config(Some("path=bzImage"))
            .build();
        assert!(vmm_config.is_ok());
        assert_eq!(
            vmm_config.unwrap().memory_config,
            MemoryConfig { size_mib: 1024 }
        );
    }

    #[test]
    fn test_builder_memory_config_none_default() {
        let vmm_config = Builder::default()
            .memory_config(None as Option<&str>)
            .kernel_config(Some("path=bzImage"))
            .build();
        assert!(vmm_config.is_ok());
        assert_eq!(
            vmm_config.unwrap().memory_config,
            MemoryConfig { size_mib: 256 }
        );
    }

    #[test]
    fn test_builder_vcpu_config_success() {
        let vmm_config = Builder::default()
            .vcpu_config(Some("num=2"))
            .kernel_config(Some("path=bzImage"))
            .build();
        assert!(vmm_config.is_ok());
        assert_eq!(vmm_config.unwrap().vcpu_config, VcpuConfig { num: 2 });
    }

    #[test]
    fn test_builder_vcpu_config_none_default() {
        let vmm_config = Builder::default()
            .memory_config(None as Option<&str>)
            .kernel_config(Some("path=bzImage"))
            .build();
        assert!(vmm_config.is_ok());
        assert_eq!(vmm_config.unwrap().vcpu_config, VcpuConfig { num: 1 });
    }

    #[test]
    fn test_builder_kernel_config_success_default() {
        let vmm_config = Builder::default()
            .kernel_config(Some("path=bzImage"))
            .build();
        assert!(vmm_config.is_ok());
        assert_eq!(
            vmm_config.unwrap().kernel_config,
            KernelConfig {
                cmdline: KernelConfig::default_cmdline(),
                load_addr: DEFAULT_KERNEL_LOAD_ADDR,
                path: PathBuf::from("bzImage")
            }
        );
    }

    #[test]
    fn test_builder_kernel_config_none_error() {
        let vmm_config = Builder::default()
            .kernel_config(None as Option<&str>)
            .build();

        assert!(vmm_config.is_err());
    }

    #[test]
    fn test_builder_net_config_none_default() {
        let vmm_config = Builder::default()
            .net_config(None as Option<&str>)
            .kernel_config(Some("path=bzImage"))
            .build();
        assert!(vmm_config.is_ok());
        assert!(vmm_config.unwrap().net_config.is_none());
    }

    #[test]
    fn test_builder_net_config_success() {
        let vmm_config = Builder::default()
            .net_config(Some("tap=tap0"))
            .kernel_config(Some("path=bzImage"))
            .build();
        assert!(vmm_config.is_ok());
        assert_eq!(
            vmm_config.unwrap().net_config,
            Some(NetConfig {
                tap_name: "tap0".to_string()
            })
        );
    }

    #[test]
    fn test_builder_block_config_none_default() {
        let vmm_config = Builder::default()
            .block_config(None as Option<&str>)
            .kernel_config(Some("path=bzImage"))
            .build();
        assert!(vmm_config.is_ok());
        assert!(vmm_config.unwrap().block_config.is_none());
    }

    #[test]
    fn test_builder_block_config_success() {
        let vmm_config = Builder::default()
            .block_config(Some("path=/dev/loop0"))
            .kernel_config(Some("path=bzImage"))
            .build();
        assert!(vmm_config.is_ok());
        assert_eq!(
            vmm_config.unwrap().block_config,
            Some(BlockConfig {
                path: PathBuf::from("/dev/loop0")
            })
        );
    }

    #[test]
    fn test_builder_vmm_config_success() {
        let vmm_config = Builder::default()
            .memory_config(Some("size_mib=1024"))
            .vcpu_config(Some("num=2"))
            .net_config(Some("tap=tap0"))
            .kernel_config(Some("path=bzImage"))
            .block_config(Some("path=/dev/loop0"))
            .build();
        assert!(vmm_config.is_ok());
        assert_eq!(
            vmm_config.unwrap(),
            VMMConfig {
                memory_config: MemoryConfig { size_mib: 1024 },
                vcpu_config: VcpuConfig { num: 2 },
                kernel_config: KernelConfig {
                    cmdline: KernelConfig::default_cmdline(),
                    load_addr: DEFAULT_KERNEL_LOAD_ADDR,
                    path: PathBuf::from("bzImage")
                },
                net_config: Some(NetConfig {
                    tap_name: "tap0".to_string()
                }),
                block_config: Some(BlockConfig {
                    path: PathBuf::from("/dev/loop0")
                })
            }
        );
    }
}
