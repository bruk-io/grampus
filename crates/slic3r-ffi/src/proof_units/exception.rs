//! Proof path 2 (R19) for the `Exception.hpp` unit (task 52-foundation-common-defs,
//! `port/foundation.tsv` row `foundation-01`).
//!
//! One `what()` round-trip per concrete type in the vendored hierarchy, plus `SlicingError`'s
//! `objectId()` and `SlicingErrors`' always-`"Errors"` message and element count -- see
//! `cxx/proof_units/exception.cpp` for why this shim never throws or catches across the bridge.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::exception")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/exception.h");

        fn path2_exception_what(msg: &str) -> String;
        fn path2_critical_exception_what(msg: &str) -> String;
        fn path2_runtime_error_what(msg: &str) -> String;
        fn path2_logic_error_what(msg: &str) -> String;
        fn path2_hard_crash_what(msg: &str) -> String;
        fn path2_invalid_argument_what(msg: &str) -> String;
        fn path2_out_of_range_what(msg: &str) -> String;
        fn path2_io_error_what(msg: &str) -> String;
        fn path2_file_io_error_what(msg: &str) -> String;
        fn path2_host_network_error_what(msg: &str) -> String;
        fn path2_export_error_what(msg: &str) -> String;
        fn path2_placeholder_parser_error_what(msg: &str) -> String;

        fn path2_slicing_error_what(msg: &str) -> String;
        fn path2_slicing_error_default_object_id(msg: &str) -> u64;
        fn path2_slicing_error_object_id(msg: &str, object_id: u64) -> u64;

        fn path2_slicing_errors_from_message_what(msg: &str) -> String;
        fn path2_slicing_errors_from_vector_what(count: u32) -> String;
        fn path2_slicing_errors_from_vector_count(count: u32) -> i64;
    }
}

pub use ffi::{
    path2_critical_exception_what, path2_exception_what, path2_export_error_what,
    path2_file_io_error_what, path2_hard_crash_what, path2_host_network_error_what,
    path2_invalid_argument_what, path2_io_error_what, path2_logic_error_what,
    path2_out_of_range_what, path2_placeholder_parser_error_what, path2_runtime_error_what,
    path2_slicing_error_default_object_id, path2_slicing_error_object_id, path2_slicing_error_what,
    path2_slicing_errors_from_message_what, path2_slicing_errors_from_vector_count,
    path2_slicing_errors_from_vector_what,
};
