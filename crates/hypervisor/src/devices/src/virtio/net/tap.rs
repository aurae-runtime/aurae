// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
//
// Portions Copyright 2017 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the THIRD-PARTY file.

// We should add a tap abstraction to rust-vmm as well. Using this one, which is copied from
// Firecracker until then.

use std::fs::File;
use std::io::{Error as IoError, Read, Result as IoResult, Write};
use std::os::raw::{c_char, c_int, c_uint, c_ulong};
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};

use vmm_sys_util::ioctl::{ioctl_with_mut_ref, ioctl_with_ref, ioctl_with_val};
use vmm_sys_util::{ioctl_expr, ioctl_ioc_nr, ioctl_iow_nr};

use super::bindings::ifreq;

// As defined in the Linux UAPI:
// https://elixir.bootlin.com/linux/v4.17/source/include/uapi/linux/if.h#L33
const IFACE_NAME_MAX_LEN: usize = 16;

// Taken from firecracker net_gen/if_tun.rs ... we should see what to do about the net related
// bindings overall for rust-vmm.
const IFF_TAP: ::std::os::raw::c_uint = 2;
const IFF_NO_PI: ::std::os::raw::c_uint = 4096;
const IFF_VNET_HDR: ::std::os::raw::c_uint = 16384;

/// List of errors the tap implementation can throw.
#[derive(Debug)]
pub enum Error {
    /// Unable to create tap interface.
    CreateTap(IoError),
    /// Invalid interface name.
    InvalidIfname,
    /// ioctl failed.
    IoctlError(IoError),
    /// Couldn't open /dev/net/tun.
    OpenTun(IoError),
}

pub type Result<T> = ::std::result::Result<T, Error>;

const TUNTAP: ::std::os::raw::c_uint = 84;
ioctl_iow_nr!(TUNSETIFF, TUNTAP, 202, ::std::os::raw::c_int);
ioctl_iow_nr!(TUNSETOFFLOAD, TUNTAP, 208, ::std::os::raw::c_uint);
ioctl_iow_nr!(TUNSETVNETHDRSZ, TUNTAP, 216, ::std::os::raw::c_int);

/// Handle for a network tap interface.
///
/// For now, this simply wraps the file descriptor for the tap device so methods
/// can run ioctls on the interface. The tap interface fd will be closed when
/// Tap goes out of scope, and the kernel will clean up the interface automatically.
#[derive(Debug)]
pub struct Tap {
    tap_file: File,
    pub(crate) if_name: [u8; IFACE_NAME_MAX_LEN],
}

// Returns a byte vector representing the contents of a null terminated C string which
// contains if_name.
fn build_terminated_if_name(if_name: &str) -> Result<[u8; IFACE_NAME_MAX_LEN]> {
    // Convert the string slice to bytes, and shadow the variable,
    // since we no longer need the &str version.
    let if_name = if_name.as_bytes();

    if if_name.len() >= IFACE_NAME_MAX_LEN {
        return Err(Error::InvalidIfname);
    }

    let mut terminated_if_name = [b'\0'; IFACE_NAME_MAX_LEN];
    terminated_if_name[..if_name.len()].copy_from_slice(if_name);

    Ok(terminated_if_name)
}

pub struct IfReqBuilder(ifreq);

impl IfReqBuilder {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(Default::default())
    }

    pub fn if_name(mut self, if_name: &[u8; IFACE_NAME_MAX_LEN]) -> Self {
        // Since we don't call as_mut on the same union field more than once, this block is safe.
        let ifrn_name = unsafe { self.0.ifr_ifrn.ifrn_name.as_mut() };
        ifrn_name.copy_from_slice(if_name.as_ref());

        self
    }

    pub(crate) fn flags(mut self, flags: i16) -> Self {
        // Since we don't call as_mut on the same union field more than once, this block is safe.
        let ifru_flags = unsafe { self.0.ifr_ifru.ifru_flags.as_mut() };
        *ifru_flags = flags;

        self
    }

    pub(crate) fn execute<F: AsRawFd>(mut self, socket: &F, ioctl: u64) -> Result<ifreq> {
        // ioctl is safe. Called with a valid socket fd, and we check the return.
        let ret = unsafe { ioctl_with_mut_ref(socket, ioctl, &mut self.0) };
        if ret < 0 {
            return Err(Error::IoctlError(IoError::last_os_error()));
        }

        Ok(self.0)
    }
}

impl Tap {
    /// Create a TUN/TAP device given the interface name.
    /// # Arguments
    ///
    /// * `if_name` - the name of the interface.
    pub fn open_named(if_name: &str) -> Result<Tap> {
        let terminated_if_name = build_terminated_if_name(if_name)?;

        let fd = unsafe {
            // Open calls are safe because we give a constant null-terminated
            // string and verify the result.
            libc::open(
                b"/dev/net/tun\0".as_ptr() as *const c_char,
                libc::O_RDWR | libc::O_NONBLOCK | libc::O_CLOEXEC,
            )
        };
        if fd < 0 {
            return Err(Error::OpenTun(IoError::last_os_error()));
        }
        // We just checked that the fd is valid.
        let tuntap = unsafe { File::from_raw_fd(fd) };

        let ifreq = IfReqBuilder::new()
            .if_name(&terminated_if_name)
            .flags((IFF_TAP | IFF_NO_PI | IFF_VNET_HDR) as i16)
            .execute(&tuntap, TUNSETIFF())?;

        // Safe since only the name is accessed, and it's cloned out.
        Ok(Tap {
            tap_file: tuntap,
            if_name: unsafe { *ifreq.ifr_ifrn.ifrn_name.as_ref() },
        })
    }

    pub fn if_name_as_str(&self) -> &str {
        let len = self
            .if_name
            .iter()
            .position(|x| *x == 0)
            .unwrap_or(IFACE_NAME_MAX_LEN);
        std::str::from_utf8(&self.if_name[..len]).unwrap_or("")
    }

    /// Set the offload flags for the tap interface.
    pub fn set_offload(&self, flags: c_uint) -> Result<()> {
        // ioctl is safe. Called with a valid tap fd, and we check the return.
        let ret = unsafe { ioctl_with_val(&self.tap_file, TUNSETOFFLOAD(), c_ulong::from(flags)) };
        if ret < 0 {
            return Err(Error::IoctlError(IoError::last_os_error()));
        }

        Ok(())
    }

    /// Set the size of the vnet hdr.
    pub fn set_vnet_hdr_size(&self, size: c_int) -> Result<()> {
        // ioctl is safe. Called with a valid tap fd, and we check the return.
        let ret = unsafe { ioctl_with_ref(&self.tap_file, TUNSETVNETHDRSZ(), &size) };
        if ret < 0 {
            return Err(Error::IoctlError(IoError::last_os_error()));
        }

        Ok(())
    }
}

impl Read for Tap {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        self.tap_file.read(buf)
    }
}

impl Write for Tap {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.tap_file.write(buf)
    }

    fn flush(&mut self) -> IoResult<()> {
        Ok(())
    }
}

impl AsRawFd for Tap {
    fn as_raw_fd(&self) -> RawFd {
        self.tap_file.as_raw_fd()
    }
}

// TODO: If we don't end up using an external abstraction for `Tap` interfaces, add unit tests
// based on a mock framework that do not require elevated privileges to run.
