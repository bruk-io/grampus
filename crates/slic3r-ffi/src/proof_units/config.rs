//! Proof path 3 bridge for task `59-foundation-config` (`port/manifest.tsv`'s `Config.cpp`/
//! `Config.hpp` rows). See `crates/slic3r-ffi/cxx/proof_units/config.cpp`'s own header comment
//! for the case-insensitive self-include finding this unit's shim surfaced.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::config")]
mod ffi {
    struct ProofUnescapeResult {
        ok: bool,
        value: String,
    }
    struct ProofUnescapeStringsResult {
        ok: bool,
        values: Vec<String>,
    }
    struct ProofDeserializeFloatResult {
        ok: bool,
        value: f64,
    }
    struct ProofDeserializeIntResult {
        ok: bool,
        value: i32,
    }

    unsafe extern "C++" {
        include!("slic3r-ffi/cxx/proof_units/config.h");

        fn path3_is_whitespace(c: u8) -> bool;
        fn path3_is_end_of_line(c: u8) -> bool;
        fn path3_is_end_of_gcode_line(c: u8) -> bool;
        fn path3_is_end_of_word(c: u8) -> bool;

        fn path3_normalize_cli_bool_value(value: &str) -> String;

        fn path3_escape_string_cstyle(value: &str) -> String;
        fn path3_escape_ampersand(value: &str) -> String;
        fn path3_unescape_string_cstyle(value: &str) -> ProofUnescapeResult;

        fn path3_escape_strings_cstyle(values: Vec<String>) -> String;
        fn path3_unescape_strings_cstyle(value: &str) -> ProofUnescapeStringsResult;

        // `ConfigOptionFloat::serialize`/`::deserialize` (`Config.hpp:840-854`), reached through
        // the same whole-file `#include` as the internal-linkage helpers above -- this unit's
        // own primary surface, not just its string-escaping fan-in. See this task's pull request
        // body for why these two were added after the three blockers both reviewers measured
        // (float serialisation/deserialisation) turned out to sit entirely outside what this
        // proof originally covered.
        fn path3_serialize_float(value: f64) -> String;
        fn path3_deserialize_float(value: &str) -> ProofDeserializeFloatResult;
        fn path3_deserialize_int(value: &str) -> ProofDeserializeIntResult;
    }
}

pub use ffi::{
    path3_deserialize_float, path3_deserialize_int, path3_escape_ampersand,
    path3_escape_string_cstyle, path3_escape_strings_cstyle, path3_is_end_of_gcode_line,
    path3_is_end_of_line, path3_is_end_of_word, path3_is_whitespace,
    path3_normalize_cli_bool_value, path3_serialize_float, path3_unescape_string_cstyle,
    path3_unescape_strings_cstyle, ProofDeserializeFloatResult, ProofDeserializeIntResult,
    ProofUnescapeResult, ProofUnescapeStringsResult,
};
