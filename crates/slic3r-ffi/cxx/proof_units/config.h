#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart at <crate-name>/src/proof_units/config.rs.h.
#include "slic3r-ffi/src/proof_units/config.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace config {

// The four character-class helpers (internal linkage: `static` free functions, Config.cpp:1250-1253).
bool path3_is_whitespace(uint8_t c);
bool path3_is_end_of_line(uint8_t c);
bool path3_is_end_of_gcode_line(uint8_t c);
bool path3_is_end_of_word(uint8_t c);

// `normalize_cli_bool_value` (internal linkage: `static`, Config.cpp:1758-1782).
rust::String path3_normalize_cli_bool_value(rust::Str value);

// The externally-linked string-escaping free functions (Config.cpp:76-259), reached through the
// same `#include` as the internal-linkage helpers above.
rust::String path3_escape_string_cstyle(rust::Str value);
rust::String path3_escape_ampersand(rust::Str value);

// `ProofUnescapeResult`/`ProofUnescapeStringsResult` are `cxx`-generated shared structs (declared
// in the bridge module, `src/proof_units/config.rs`); this header's own include of the
// `cxx`-generated header above already defines them, and this file does not redefine them.
ProofUnescapeResult path3_unescape_string_cstyle(rust::Str value);

rust::String path3_escape_strings_cstyle(rust::Vec<rust::String> values);

ProofUnescapeStringsResult path3_unescape_strings_cstyle(rust::Str value);

// `ConfigOptionFloat::serialize`/`::deserialize` (`Config.hpp:840-854`) -- this unit's own
// primary surface, proving the float serialisation/deserialisation format directly rather than
// only the string-escaping fan-in.
rust::String path3_serialize_float(double value);
ProofDeserializeFloatResult path3_deserialize_float(rust::Str value);
ProofDeserializeIntResult path3_deserialize_int(rust::Str value);

// `std::istringstream >> double` on its own, seeded with a sentinel the caller can recognise.
// `ConfigOptionFloat::deserialize` (`Config.hpp:844-854`) extracts into a *local* `double` and
// assigns the option only when `!iss.fail()`, so `path3_deserialize_float` above cannot observe
// what a failing extraction assigned -- and the underflow (`ERANGE`) cases are exactly the ones
// that fail *with* a value. This exposes the extraction itself, which is what the port's
// `cpp_istream_extract_f64` claims to reproduce, in the same translation unit and locale as the
// vendored code above.
ProofIstreamExtractDoubleResult path3_istream_extract_double(rust::Str value);

// `ConfigOptionPoint::deserialize`/`ConfigOptionPoint3::deserialize` (`Config.hpp:1557-1567`
// and `:1686-1696`), whose `sscanf(" %lf , %lf %c", ...)` is `strtod` and not the `istream`
// extraction above -- a different acceptance grammar, measured rather than assumed.
ProofDeserializePointResult path3_deserialize_point(rust::Str value);
ProofDeserializePoint3Result path3_deserialize_point3(rust::Str value);

// -------------------------------------------------------------------------------------------------
// Added by the amendment `docs/PORTING.md`, "Decisions recorded on 2026-11-26", decision 11
// records: the three option-accessor gaps task `64-foundation-print-config` found in
// `crates/slic3r-config` and could not fix from its own branch.
// -------------------------------------------------------------------------------------------------

// Gap 1. `ConfigBase::option<TYPE>` (`Config.hpp:2771-2780`) and
// `ConfigOptionResolver::option<TYPE>` (`:2701-2706`) compare the stored option's
// `ConfigOptionType` tag against `TYPE::static_type()` and then `static_cast`;
// `DynamicConfig::opt<T>` (`:3007-3010`) is a real `dynamic_cast`. On a stored
// `ConfigOptionFloatsNullable` the two therefore DISAGREE -- both are `coFloats`, so the tag check
// passes and the `static_cast` to `ConfigOptionFloats*` succeeds, while the `dynamic_cast` fails.
// This shim measures that disagreement in the oracle's own translation unit rather than asserting
// it from reading the header, and also measures the tag-MISMATCH path (asking for
// `ConfigOptionInts` on a `coFloats` store), where both accessors agree on null.
ProofTaggedAccessResult path3_tagged_vs_dynamic_floats(rust::Str serialized, bool nullable);

// Gap 1, at the call site that measurably diverged. `ConfigBase::get_abs_value_at`
// (`Config.cpp:715-739`) is one of the places whose body is a tag check and a `static_cast`
// (`:719-720`), so it reads a nullable store; the port's own translation used a type-exact
// downcast with `.expect("coFloats")` behind it, which PANICS there. Only the `coFloats` branch
// is reachable from this shim: the `coFloatsOrPercents` branch needs `this->def()`, and
// `DynamicConfig::def()` returns `nullptr` by definition (`Config.hpp:3006`).
ProofAbsValueAtResult path3_get_abs_value_at(rust::Str serialized, bool nullable, size_t index);

// Gap 1, at the call site whose whole nullable branch the port had made unreachable.
// `ConfigOptionVector<T>::apply_override` (`Config.hpp:767-815`) and `::overriden_by`
// (`:746-764`) are a tag check and then `static_cast<const ConfigOptionVector<T>*>(rhs)`
// (`:751`, `:772`) -- and both exist to read a NULLABLE `rhs` into a non-nullable `this`
// (`:752-762`, `:773-813`), which is exactly the pair a type-exact downcast rejects. `lhs` is
// always the non-nullable twin here, `rhs` always the nullable one, which is the only
// combination the vendored body does anything in. `default_index` must be `rhs->size()` long:
// `:795` asserts that, and this shim is compiled with assertions live.
ProofApplyOverrideResult path3_apply_override_floats(rust::Str lhs, rust::Str rhs,
                                                     rust::Slice<const int32_t> default_index);

// Gap 2. `dynamic_cast<ConfigOptionVectorBase*>` (`Config.cpp:497`) and the `is_vector()`-guarded
// `static_cast<ConfigOptionVectorBase*>` (`:1872`), over every option type this unit can
// construct. `ConfigOptionEnum<T>` is absent from the roster on purpose: it needs a `T` with a
// `CONFIG_OPTION_ENUM` specialization, and every one of those lives in `PrintConfig.hpp`, outside
// this unit's `#include`. `type_name` is the vendored class name; `ok` is false when it is not on
// the roster.
ProofVectorBaseResult path3_option_vector_base(rust::Str type_name);

// Gap 3. `ConfigOptionDef::create_default_option` (`Config.cpp:322-342`), on a def whose
// `default_value` carries NO keys map and whose `enum_keys_map` does -- the exact shape the
// vendored comment at `:330` describes, and the only shape in which the `coEnum`/`coEnums` arms
// are observable at all: a plain `clone()` of such a default has a null `keys_map`, so its
// `serialize()` (`Config.hpp:2181-2187`) would dereference null rather than emit a name.
// `kProofEnumValues` below is the keys map both sides use.
ProofCreateDefaultResult path3_create_default_option_enum(int default_value);
ProofCreateDefaultResult path3_create_default_option_enums(rust::Str serialized_values,
                                                           bool nullable);

} // namespace config
} // namespace proof_units
} // namespace slic3r_ffi
