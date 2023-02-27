// Copyright 2021 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

pub mod resource_download;

/// A macro for printing errors only in debug mode.
#[macro_export]
#[cfg(debug_assertions)]
macro_rules! debug {
    ($( $args:expr ),*) => { eprintln!( $( $args ),* ); }
}

/// A macro that allows printing to be ignored in release mode.
#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! debug {
    ($( $args:expr ),*) => {
        ()
    };
}
