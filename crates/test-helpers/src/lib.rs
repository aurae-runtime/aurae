/* -------------------------------------------------------------------------- *\
 *                |   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ |              *
 *                |  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ |              *
 *                |  ███████║██║   ██║██████╔╝███████║█████╗   |              *
 *                |  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   |              *
 *                |  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ |              *
 *                |  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ |              *
 *                +--------------------------------------------+              *
 *                                                                            *
 *                         Distributed Systems Runtime                        *
 * -------------------------------------------------------------------------- *
 * Copyright 2022 - 2024, the aurae contributors                              *
 * SPDX-License-Identifier: Apache-2.0                                        *
\* -------------------------------------------------------------------------- */

// Lint groups: https://doc.rust-lang.org/rustc/lints/groups.html
#![warn(future_incompatible, nonstandard_style, unused)]
#![warn(
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    unconditional_recursion,
    unused_comparisons,
    while_true
)]
#![warn(missing_debug_implementations,
// TODO: missing_docs,
trivial_casts,
trivial_numeric_casts,
unused_extern_crates,
unused_import_braces,
unused_qualifications,
unused_results
)]
#![warn(clippy::unwrap_used)]
#![allow(unused_qualifications)]

// Nix has a collection of test helpers that are not exposed publicly by their crate
// The below skip helpers are here: https://github.com/nix-rust/nix/blob/master/test/common/mod.rs

#[macro_export]
macro_rules! skip {
    ($($reason: expr),+) => {
        use ::std::io::{self, Write};

        let stderr = io::stderr();
        let mut handle = stderr.lock();
        writeln!(handle, $($reason),+).unwrap();
        return;
    }
}

#[macro_export]
macro_rules! skip_if_not_root {
    ($name:expr) => {
        use nix::unistd::Uid;

        if !Uid::current().is_root() {
            skip!("{} requires root privileges. Skipping test.", $name);
        }
    };
}

#[macro_export]
macro_rules! skip_if_seccomp {
    ($name:expr) => {
        if let Ok(s) = std::fs::read_to_string("/proc/self/status") {
            for l in s.lines() {
                let mut fields = l.split_whitespace();
                if fields.next() == Some("Seccomp:")
                    && fields.next() != Some("0")
                {
                    skip!(
                        "{} cannot be run in Seccomp mode.  Skipping test.",
                        stringify!($name)
                    );
                }
            }
        }
    };
}

#[macro_export]
macro_rules! assert_eventually_eq {
    ($left: expr, $right: expr $(,)?) => {
        assert_eventually_eq!($left, $right, Duration::from_millis(200), Duration::from_millis(10));
    };
    ($left: expr, $right: expr, $timeout: expr $(,)?) => {
        assert_eventually_eq!($left, $right, $timeout, Duration::from_millis(10));
    };
    ($left: expr, $right: expr, $timeout: expr, $poll_interval: expr $(,)?) => {
        let start = ::std::time::Instant::now();
        let timeout = $timeout;
        let poll_interval = $poll_interval;
        while !($left == $right) {
            ::tokio::time::sleep(poll_interval).await;
            let now = ::std::time::Instant::now();
            if now.duration_since(start) > timeout {
                ::core::panic!("assertion failed: `(left == right)`\nleft: {:#?}\nright: {:#?}", $left, $right);
            }
        }
    };
}

pub mod mock_time {
    use once_cell::sync::OnceCell;
    use std::sync::Mutex;
    use std::time::{Duration, SystemTime};

    pub static TIME: OnceCell<Mutex<SystemTime>> = OnceCell::new();

    pub fn now() -> SystemTime {
        *TIME
            .get_or_init(|| Mutex::new(SystemTime::UNIX_EPOCH))
            .lock()
            .expect("mock_time failed to initialize the system time")
    }

    pub fn advance_time(d: Duration) {
        let mut guard = TIME
            .get_or_init(|| Mutex::new(SystemTime::UNIX_EPOCH))
            .lock()
            .expect("mock_time failed to get the system time");
        *guard = guard
            .checked_add(d)
            .expect("mock_time failed to advance the system time");
    }

    pub fn reset() {
        let mut guard = TIME
            .get_or_init(|| Mutex::new(SystemTime::UNIX_EPOCH))
            .lock()
            .expect("mock_time failed to reset the system time");
        *guard = SystemTime::UNIX_EPOCH;
    }
}