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
#![warn(
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_results
)]
#![warn(clippy::unwrap_used)]

//! End-to-end tests for Aurae.
//!
//! These tests require a running Aurae daemon in the background.
//! `make e2e-test` will span an Aurae daemon in the bacground and run the
//! entire end-to-end test suite.

mod cells;
mod common;
mod observe;
