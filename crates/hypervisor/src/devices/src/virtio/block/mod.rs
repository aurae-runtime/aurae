// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

mod device;
mod inorder_handler;
mod queue_handler;

use std::fs::File;
use std::io::{self, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use virtio_blk::stdio_executor;

use crate::virtio::features::{VIRTIO_F_IN_ORDER, VIRTIO_F_RING_EVENT_IDX, VIRTIO_F_VERSION_1};

pub use device::Block;

// TODO: Move relevant defines to vm-virtio crate.

// Block device ID as defined by the standard.
pub const BLOCK_DEVICE_ID: u32 = 2;

// Block device read-only feature.
pub const VIRTIO_BLK_F_RO: u64 = 5;
// Block device FLUSH feature.
pub const VIRTIO_BLK_F_FLUSH: u64 = 9;

// The sector size is 512 bytes (1 << 9).
const SECTOR_SHIFT: u8 = 9;

#[derive(Debug)]
pub enum Error {
    Backend(stdio_executor::Error),
    Virtio(crate::virtio::Error),
    OpenFile(io::Error),
    Seek(io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

// TODO: Add a helper abstraction to rust-vmm for building the device configuration space.
// The one we build below for the block device contains the minimally required `capacity` member,
// but other fields can be present as well depending on the negotiated features.
fn build_config_space<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    // TODO: right now, the file size is computed by the StdioBackend as well. Maybe we should
    // create the backend as early as possible, and get the size information from there.
    let file_size = File::open(path)
        .map_err(Error::OpenFile)?
        .seek(SeekFrom::End(0))
        .map_err(Error::Seek)?;
    // If the file size is actually not a multiple of sector size, then data at the very end
    // will be ignored.
    let num_sectors = file_size >> SECTOR_SHIFT;
    // This has to be in little endian btw.
    Ok(num_sectors.to_le_bytes().to_vec())
}

// Arguments required when building a block device.
pub struct BlockArgs {
    pub file_path: PathBuf,
    pub read_only: bool,
    pub root_device: bool,
    pub advertise_flush: bool,
}

impl BlockArgs {
    // Generate device features based on the configuration options.
    pub fn device_features(&self) -> u64 {
        // The queue handling logic for the device uses the buffers in order, so we enable the
        // corresponding feature as well.
        let mut features =
            1 << VIRTIO_F_VERSION_1 | 1 << VIRTIO_F_IN_ORDER | 1 << VIRTIO_F_RING_EVENT_IDX;

        if self.read_only {
            features |= 1 << VIRTIO_BLK_F_RO;
        }

        if self.advertise_flush {
            features |= 1 << VIRTIO_BLK_F_FLUSH;
        }

        features
    }

    // Generate additional info that needs to be appended to the kernel command line based
    // on the current arg configuration.
    pub fn cmdline_config_substring(&self) -> String {
        let mut s = String::new();
        if self.root_device {
            s.push_str("root=/dev/vda");

            if self.read_only {
                s.push_str(" ro");
            } else {
                s.push_str(" rw");
            }
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::mem::size_of;

    use vmm_sys_util::tempfile::TempFile;

    use super::*;

    impl Default for BlockArgs {
        fn default() -> Self {
            BlockArgs {
                file_path: "".into(),
                read_only: false,
                root_device: false,
                advertise_flush: false,
            }
        }
    }

    #[test]
    fn test_build_config_space() {
        let tmp = TempFile::new().unwrap();

        let sector = [1u8; 512];
        let num_sectors = 1024u64;

        for _ in 0..num_sectors {
            tmp.as_file().write_all(&sector).unwrap();
        }

        {
            let config_space = build_config_space(tmp.as_path()).unwrap();

            // The config space is only populated with the `capacity` field for now.
            assert_eq!(config_space.len(), size_of::<u64>());
            assert_eq!(config_space[..8], num_sectors.to_le_bytes());
        }

        // Let's write some more bytes to the file, such that the size is no longer a multiple
        // of the sector size.
        tmp.as_file().write_all(&[1u8, 2, 3]).unwrap();

        {
            let config_space = build_config_space(tmp.as_path()).unwrap();
            // We should get the same value of capacity, as the extra bytes are ignored.
            assert_eq!(config_space[..8], num_sectors.to_le_bytes());
        }
    }

    #[test]
    fn test_device_features() {
        let mut args = BlockArgs::default();

        let base =
            1u64 << VIRTIO_F_VERSION_1 | 1 << VIRTIO_F_IN_ORDER | 1 << VIRTIO_F_RING_EVENT_IDX;

        assert_eq!(args.device_features(), base);

        args.read_only = true;
        assert_eq!(args.device_features(), base | 1 << VIRTIO_BLK_F_RO);

        args.read_only = false;
        args.advertise_flush = true;
        assert_eq!(args.device_features(), base | 1 << VIRTIO_BLK_F_FLUSH);
    }

    #[test]
    fn test_cmdline_string() {
        let mut args = BlockArgs::default();

        assert_eq!(args.cmdline_config_substring(), "");

        args.read_only = true;
        // There's no effect unless `root_device` is also `true`.
        assert_eq!(args.cmdline_config_substring(), "");

        args.root_device = true;
        assert_eq!(args.cmdline_config_substring(), "root=/dev/vda ro");

        args.read_only = false;
        assert_eq!(args.cmdline_config_substring(), "root=/dev/vda rw");
    }
}
