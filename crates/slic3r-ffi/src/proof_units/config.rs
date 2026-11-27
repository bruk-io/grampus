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
    struct ProofIstreamExtractDoubleResult {
        fail: bool,
        value: f64,
    }
    struct ProofDeserializePointResult {
        ok: bool,
        x: f64,
        y: f64,
    }
    struct ProofDeserializePoint3Result {
        ok: bool,
        x: f64,
        y: f64,
        z: f64,
    }

    /// `ConfigBase::option<TYPE>`/`ConfigOptionResolver::option<TYPE>` (tag compare, then
    /// `static_cast`) measured against `DynamicConfig::opt<T>` (a real `dynamic_cast`) on one
    /// stored option -- see `cxx/proof_units/config.h`.
    struct ProofTaggedAccessResult {
        /// `opt->nullable()` on the stored option, so the Rust side can confirm it set up the
        /// same storage the oracle did.
        stored_is_nullable: bool,
        /// `cfg.option<ConfigOptionFloats>("k") != nullptr`.
        tag_checked_ok: bool,
        /// The const overload of the same template, on a `const ConfigOptionResolver&`.
        tag_checked_const_ok: bool,
        /// `cfg.opt<ConfigOptionFloats>("k") != nullptr`.
        dynamic_cast_ok: bool,
        /// `values.size()` through the tag-checked pointer, or 0 when it is null.
        tag_checked_size: usize,
        /// `get_at(0)` through the tag-checked pointer, or `-999.0` when it is null or empty.
        tag_checked_value: f64,
        /// `cfg.option<ConfigOptionInts>("k") != nullptr` -- the tag-mismatch path.
        wrong_tag_tag_checked_ok: bool,
        /// `cfg.opt<ConfigOptionInts>("k") != nullptr` -- the tag-mismatch path.
        wrong_tag_dynamic_cast_ok: bool,
    }

    /// `ConfigBase::get_abs_value_at` (`Config.cpp:715-739`) on a `coFloats` store, nullable or
    /// not -- a call site whose own body is a tag check and a `static_cast`.
    struct ProofAbsValueAtResult {
        /// False when the vendored body threw (`ConfigurationError`).
        ok: bool,
        /// The returned absolute value, or `-999.0` when it threw.
        value: f64,
    }

    /// `ConfigOptionVector<T>::overriden_by` and `::apply_override` (`Config.hpp:746-815`) with
    /// a non-nullable `lhs` and a nullable `rhs` -- the pair both methods exist for.
    struct ProofApplyOverrideResult {
        /// False when `overriden_by` threw.
        overridden_by_ok: bool,
        /// Its returned value.
        overridden_by: bool,
        /// False when `apply_override` threw.
        ok: bool,
        /// Its returned value.
        modified: bool,
        /// `lhs->serialize()` after `apply_override` ran -- the values it actually wrote.
        serialized: String,
    }

    /// `option<TYPE>` vs `opt<T>` across the `coEnum` pair -- the one tag whose two classes have
    /// no common base below `ConfigOption`. One twin is stored and the other is asked for.
    struct ProofEnumPairResult {
        /// `opt->type()` of what was stored, as its discriminant (always `coEnum` = 9).
        stored_tag: i32,
        /// `cfg.option<the other twin>("k") != nullptr`.
        tag_checked_ok: bool,
        /// `->value` read through that tag-checked pointer, or `-999` when it is null. This is the
        /// integer the cross-pair `static_cast` reinterprets.
        tag_checked_value: i32,
        /// `cfg.opt<the other twin>("k") != nullptr`.
        dynamic_cast_ok: bool,
        /// `cfg.option("k")->getInt()` -- the virtual route across the pair, which both twins
        /// implement (`Config.hpp:2088`, `:1027`).
        get_int: i32,
    }

    /// `ConfigBase::option_throw<TYPE>` (`Config.hpp:2790-2797`): which of the two exceptions
    /// comes out, or the size on success. The `const ConfigOptionResolver` overload at
    /// `:2716-2723` is absent on purpose -- it does not compile at the pin (see
    /// `cxx/proof_units/config.cpp` for the error and why nothing ever caught it), so the
    /// `const_*` fields below are left unset.
    struct ProofOptionThrowResult {
        /// `ConfigBase::option_throw<TYPE>` returned a pointer.
        ok: bool,
        /// It threw `UnknownOptionException`.
        unknown_option: bool,
        /// It threw `BadOptionTypeException`.
        bad_option_type: bool,
        /// `values.size()` through the returned pointer, when it returned one.
        size: usize,
        /// The same three, for the `const ConfigOptionResolver&` overload.
        const_ok: bool,
        const_unknown_option: bool,
        const_bad_option_type: bool,
    }

    /// `dynamic_cast<ConfigOptionVectorBase*>` over one constructed option type.
    struct ProofVectorBaseResult {
        /// False when `type_name` is not on the shim's roster.
        on_roster: bool,
        /// `dynamic_cast<ConfigOptionVectorBase*>(opt) != nullptr`.
        vector_base_ok: bool,
        /// `opt->is_vector()`, which `Config.cpp:1872` guards its own `static_cast` with.
        is_vector: bool,
        /// `size()` through the base pointer, or 0 when it is null.
        vector_base_size: usize,
    }

    /// `ConfigOptionDef::create_default_option()` on a def whose default value carries no keys
    /// map -- see `cxx/proof_units/config.h`.
    struct ProofCreateDefaultResult {
        ok: bool,
        /// `opt->type()` as its `ConfigOptionType` discriminant.
        option_type: i32,
        /// `getInt()` for a scalar; for a vector, its length (`getInt()` throws there), and -1
        /// when the result is neither.
        scalar_value: i32,
        /// `opt->nullable()`.
        nullable: bool,
        /// `opt->serialize()` -- the observable that shows whether `enum_keys_map` was attached:
        /// a name when it was, and an empty string when the map resolves no name for the value.
        serialized: String,
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

        // `std::istringstream >> double` by itself. `ConfigOptionFloat::deserialize` extracts
        // into a local and assigns the option only on success, so it cannot observe what a
        // *failing* extraction assigned -- and underflow (`ERANGE`) is exactly a failure that
        // carries a value. See `cxx/proof_units/config.h` for the full reason.
        fn path3_istream_extract_double(value: &str) -> ProofIstreamExtractDoubleResult;

        // `ConfigOptionPoint::deserialize`/`ConfigOptionPoint3::deserialize`, whose `sscanf`
        // `%lf` conversion is `strtod` -- a wider grammar than the `istream` extraction above.
        fn path3_deserialize_point(value: &str) -> ProofDeserializePointResult;
        fn path3_deserialize_point3(value: &str) -> ProofDeserializePoint3Result;

        // The three option-accessor gaps the amendment `docs/PORTING.md`, "Decisions recorded on
        // 2026-11-26", decision 11 records closes in `crates/slic3r-config`. See
        // `cxx/proof_units/config.h` for what each measures.
        fn path3_tagged_vs_dynamic_floats(
            serialized: &str,
            nullable: bool,
        ) -> ProofTaggedAccessResult;
        fn path3_get_abs_value_at(
            serialized: &str,
            nullable: bool,
            index: usize,
        ) -> ProofAbsValueAtResult;
        fn path3_apply_override_floats(
            lhs: &str,
            rhs: &str,
            rhs_nullable: bool,
            default_index: &[i32],
        ) -> ProofApplyOverrideResult;
        fn path3_tagged_vs_dynamic_enum(store_generic: bool, value: i32) -> ProofEnumPairResult;
        fn path3_option_throw_floats(
            serialized: &str,
            nullable: bool,
            ask_wrong_tag: bool,
            ask_missing_key: bool,
        ) -> ProofOptionThrowResult;
        fn path3_create_empty_option_enum(enums: bool) -> ProofCreateDefaultResult;
        fn path3_option_vector_base(type_name: &str) -> ProofVectorBaseResult;
        fn path3_create_default_option_enum(default_value: i32) -> ProofCreateDefaultResult;
        fn path3_create_default_option_enums(
            serialized_values: &str,
            nullable: bool,
        ) -> ProofCreateDefaultResult;
    }
}

pub use ffi::{
    path3_apply_override_floats, path3_create_default_option_enum,
    path3_create_default_option_enums, path3_create_empty_option_enum, path3_deserialize_float,
    path3_deserialize_int, path3_deserialize_point, path3_deserialize_point3,
    path3_escape_ampersand, path3_escape_string_cstyle, path3_escape_strings_cstyle,
    path3_get_abs_value_at, path3_is_end_of_gcode_line, path3_is_end_of_line, path3_is_end_of_word,
    path3_is_whitespace, path3_istream_extract_double, path3_normalize_cli_bool_value,
    path3_option_throw_floats, path3_option_vector_base, path3_serialize_float,
    path3_tagged_vs_dynamic_enum, path3_tagged_vs_dynamic_floats, path3_unescape_string_cstyle,
    path3_unescape_strings_cstyle, ProofAbsValueAtResult, ProofCreateDefaultResult,
    ProofDeserializeFloatResult, ProofDeserializeIntResult, ProofDeserializePoint3Result,
    ProofDeserializePointResult, ProofEnumPairResult, ProofIstreamExtractDoubleResult,
    ProofOptionThrowResult, ProofTaggedAccessResult, ProofUnescapeResult,
    ProofUnescapeStringsResult, ProofVectorBaseResult,
};
