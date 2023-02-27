// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
//
// Portions Copyright 2017 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the THIRD-PARTY file.

// The following are manually copied from crosvm/firecracker. In the latter, they can be found as
// part of the `net_gen` local crate. We should figure out how to proceed going forward (i.e.
// create some bindings of our own, put them in a common crate, etc).

#![allow(clippy::all)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub const TUN_F_CSUM: ::std::os::raw::c_uint = 1;
pub const TUN_F_TSO4: ::std::os::raw::c_uint = 2;
pub const TUN_F_TSO6: ::std::os::raw::c_uint = 4;
pub const TUN_F_UFO: ::std::os::raw::c_uint = 16;

#[repr(C)]
pub struct __BindgenUnionField<T>(::std::marker::PhantomData<T>);
impl<T> __BindgenUnionField<T> {
    #[inline]
    pub fn new() -> Self {
        __BindgenUnionField(::std::marker::PhantomData)
    }
    #[inline]
    pub unsafe fn as_ref(&self) -> &T {
        ::std::mem::transmute(self)
    }
    #[inline]
    pub unsafe fn as_mut(&mut self) -> &mut T {
        ::std::mem::transmute(self)
    }
}
impl<T> ::std::default::Default for __BindgenUnionField<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
impl<T> ::std::clone::Clone for __BindgenUnionField<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self::new()
    }
}
impl<T> ::std::marker::Copy for __BindgenUnionField<T> {}
impl<T> ::std::fmt::Debug for __BindgenUnionField<T> {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        fmt.write_str("__BindgenUnionField")
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy)]
pub struct ifreq {
    pub ifr_ifrn: ifreq__bindgen_ty_1,
    pub ifr_ifru: ifreq__bindgen_ty_2,
}

impl Clone for ifreq {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy)]
pub struct ifreq__bindgen_ty_1 {
    pub ifrn_name: __BindgenUnionField<[::std::os::raw::c_uchar; 16usize]>,
    pub bindgen_union_field: [u8; 16usize],
}

impl Clone for ifreq__bindgen_ty_1 {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy)]
pub struct ifreq__bindgen_ty_2 {
    pub ifru_addr: __BindgenUnionField<sockaddr>,
    pub ifru_dstaddr: __BindgenUnionField<sockaddr>,
    pub ifru_broadaddr: __BindgenUnionField<sockaddr>,
    pub ifru_netmask: __BindgenUnionField<sockaddr>,
    pub ifru_hwaddr: __BindgenUnionField<sockaddr>,
    pub ifru_flags: __BindgenUnionField<::std::os::raw::c_short>,
    pub ifru_ivalue: __BindgenUnionField<::std::os::raw::c_int>,
    pub ifru_mtu: __BindgenUnionField<::std::os::raw::c_int>,
    pub ifru_map: __BindgenUnionField<ifmap>,
    pub ifru_slave: __BindgenUnionField<[::std::os::raw::c_char; 16usize]>,
    pub ifru_newname: __BindgenUnionField<[::std::os::raw::c_char; 16usize]>,
    pub ifru_data: __BindgenUnionField<*mut ::std::os::raw::c_void>,
    pub ifru_settings: __BindgenUnionField<if_settings>,
    pub bindgen_union_field: [u64; 3usize],
}

impl Clone for ifreq__bindgen_ty_2 {
    fn clone(&self) -> Self {
        *self
    }
}

pub type sa_family_t = ::std::os::raw::c_ushort;

#[repr(C)]
#[derive(Debug, Default, Copy)]
pub struct sockaddr {
    pub sa_family: sa_family_t,
    pub sa_data: [::std::os::raw::c_char; 14usize],
}

impl Clone for sockaddr {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy)]
pub struct if_settings {
    pub type_: ::std::os::raw::c_uint,
    pub size: ::std::os::raw::c_uint,
    pub ifs_ifsu: if_settings__bindgen_ty_1,
}

impl Clone for if_settings {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy)]
pub struct if_settings__bindgen_ty_1 {
    pub raw_hdlc: __BindgenUnionField<*mut raw_hdlc_proto>,
    pub cisco: __BindgenUnionField<*mut cisco_proto>,
    pub fr: __BindgenUnionField<*mut fr_proto>,
    pub fr_pvc: __BindgenUnionField<*mut fr_proto_pvc>,
    pub fr_pvc_info: __BindgenUnionField<*mut fr_proto_pvc_info>,
    pub sync: __BindgenUnionField<*mut sync_serial_settings>,
    pub te1: __BindgenUnionField<*mut te1_settings>,
    pub bindgen_union_field: u64,
}

impl Clone for if_settings__bindgen_ty_1 {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy)]
pub struct ifmap {
    pub mem_start: ::std::os::raw::c_ulong,
    pub mem_end: ::std::os::raw::c_ulong,
    pub base_addr: ::std::os::raw::c_ushort,
    pub irq: ::std::os::raw::c_uchar,
    pub dma: ::std::os::raw::c_uchar,
    pub port: ::std::os::raw::c_uchar,
}

impl Clone for ifmap {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy)]
pub struct raw_hdlc_proto {
    pub encoding: ::std::os::raw::c_ushort,
    pub parity: ::std::os::raw::c_ushort,
}

impl Clone for raw_hdlc_proto {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy)]
pub struct cisco_proto {
    pub interval: ::std::os::raw::c_uint,
    pub timeout: ::std::os::raw::c_uint,
}

impl Clone for cisco_proto {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy)]
pub struct fr_proto {
    pub t391: ::std::os::raw::c_uint,
    pub t392: ::std::os::raw::c_uint,
    pub n391: ::std::os::raw::c_uint,
    pub n392: ::std::os::raw::c_uint,
    pub n393: ::std::os::raw::c_uint,
    pub lmi: ::std::os::raw::c_ushort,
    pub dce: ::std::os::raw::c_ushort,
}

impl Clone for fr_proto {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy)]
pub struct fr_proto_pvc {
    pub dlci: ::std::os::raw::c_uint,
}

impl Clone for fr_proto_pvc {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy)]
pub struct fr_proto_pvc_info {
    pub dlci: ::std::os::raw::c_uint,
    pub master: [::std::os::raw::c_char; 16usize],
}

impl Clone for fr_proto_pvc_info {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy)]
pub struct sync_serial_settings {
    pub clock_rate: ::std::os::raw::c_uint,
    pub clock_type: ::std::os::raw::c_uint,
    pub loopback: ::std::os::raw::c_ushort,
}

impl Clone for sync_serial_settings {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy)]
pub struct te1_settings {
    pub clock_rate: ::std::os::raw::c_uint,
    pub clock_type: ::std::os::raw::c_uint,
    pub loopback: ::std::os::raw::c_ushort,
    pub slot_map: ::std::os::raw::c_uint,
}

impl Clone for te1_settings {
    fn clone(&self) -> Self {
        *self
    }
}
