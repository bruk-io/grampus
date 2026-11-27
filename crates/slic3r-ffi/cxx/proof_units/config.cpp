// Proof path 3 (R19, plan step 18, task 31-proof-harness): `Config.cpp` has `static` free
// functions (internal linkage) this crate can only reach by `#include`ing the vendored file
// whole, exactly as `clipper_z_utils.cpp`'s own header comment states for the same mechanism --
// see this task's pull request body for the general rule this file's own history is an instance
// of, and the measurement (461 strong defined external `Slic3r::` symbols in this file's own
// object, per the `nm -m` count `clipper_z_utils.cpp` prescribes) that preceded writing it.
//
// ONE GENERAL FINDING THIS FILE'S OWN NAME PRODUCES, RECORDED HERE BECAUSE IT IS A RULE ABOUT THE
// SHIM LAYOUT ITSELF, NOT JUST THIS UNIT'S HISTORY: `docs/PORTING.md`'s naming rule makes this
// shim's own file stem the unit's manifest module stem, verbatim -- "config" for this unit. On a
// case-insensitive filesystem (the default for macOS APFS, which this pin's own CI and every
// local worktree run on), a quote-include (`#include "Config.cpp"`) written inside a file named
// `config.cpp` resolves *first* against the including file's own directory before ever reaching
// `-I`/`-isystem` -- and `"Config.cpp"` case-insensitively matches this file's own name. The
// first version of this file hit exactly that: `#include nested too deeply` and clang's own
// `-Wnonportable-include-path`, pointing at `"config.cpp"` on disk, not the vendored file.
// `clipper_z_utils` (underscored, lower-case) never collides with `ClipperZUtils` this way;
// `config`/`Config` is the first unit whose manifest stem and vendored stem are the identical
// word, differing only in case. THE FIX, BELOW: an angle-bracket `#include <Config.cpp>`, which
// skips the including file's own directory entirely and resolves only against the compiler's
// include search path -- which already carries `vendor/OrcaSlicer/src/libslic3r` as a plain `-I`
// entry (`crates/slic3r-ffi/build.rs`). A future unit whose manifest stem collides with its own
// vendored file's stem this same way needs the same fix.

#include "config.h"

#include <Config.cpp>

namespace slic3r_ffi {
namespace proof_units {
namespace config {

bool path3_is_whitespace(uint8_t c) {
    return Slic3r::is_whitespace(static_cast<char>(c));
}
bool path3_is_end_of_line(uint8_t c) {
    return Slic3r::is_end_of_line(static_cast<char>(c));
}
bool path3_is_end_of_gcode_line(uint8_t c) {
    return Slic3r::is_end_of_gcode_line(static_cast<char>(c));
}
bool path3_is_end_of_word(uint8_t c) {
    return Slic3r::is_end_of_word(static_cast<char>(c));
}

rust::String path3_normalize_cli_bool_value(rust::Str value) {
    std::string cpp_value(value);
    return rust::String(Slic3r::normalize_cli_bool_value(cpp_value));
}

rust::String path3_escape_string_cstyle(rust::Str value) {
    std::string cpp_value(value);
    return rust::String(Slic3r::escape_string_cstyle(cpp_value));
}

rust::String path3_escape_ampersand(rust::Str value) {
    std::string cpp_value(value);
    return rust::String(Slic3r::escape_ampersand(cpp_value));
}

ProofUnescapeResult path3_unescape_string_cstyle(rust::Str value) {
    std::string cpp_value(value);
    std::string out;
    bool ok = Slic3r::unescape_string_cstyle(cpp_value, out);
    return ProofUnescapeResult{ok, rust::String(out)};
}

rust::String path3_escape_strings_cstyle(rust::Vec<rust::String> values) {
    std::vector<std::string> cpp_values;
    cpp_values.reserve(values.size());
    for (const rust::String &v : values)
        cpp_values.emplace_back(std::string(v));
    return rust::String(Slic3r::escape_strings_cstyle(cpp_values));
}

ProofUnescapeStringsResult path3_unescape_strings_cstyle(rust::Str value) {
    std::string cpp_value(value);
    std::vector<std::string> out;
    bool ok = Slic3r::unescape_strings_cstyle(cpp_value, out);
    rust::Vec<rust::String> rust_out;
    for (const std::string &v : out)
        rust_out.push_back(rust::String(v));
    return ProofUnescapeStringsResult{ok, std::move(rust_out)};
}

rust::String path3_serialize_float(double value) {
    Slic3r::ConfigOptionFloat opt(value);
    return rust::String(opt.serialize());
}

ProofDeserializeFloatResult path3_deserialize_float(rust::Str value) {
    std::string cpp_value(value);
    Slic3r::ConfigOptionFloat opt(-999.0);
    bool ok = opt.deserialize(cpp_value);
    return ProofDeserializeFloatResult{ok, opt.value};
}

ProofDeserializeIntResult path3_deserialize_int(rust::Str value) {
    std::string cpp_value(value);
    Slic3r::ConfigOptionInt opt(-999);
    bool ok = opt.deserialize(cpp_value);
    return ProofDeserializeIntResult{ok, opt.value};
}

ProofIstreamExtractDoubleResult path3_istream_extract_double(rust::Str value) {
    std::string cpp_value(value);
    std::istringstream iss(cpp_value);
    double extracted = -999.0;
    iss >> extracted;
    return ProofIstreamExtractDoubleResult{iss.fail(), extracted};
}

ProofDeserializePointResult path3_deserialize_point(rust::Str value) {
    std::string cpp_value(value);
    Slic3r::ConfigOptionPoint opt(Slic3r::Vec2d(-999.0, -999.0));
    bool ok = opt.deserialize(cpp_value);
    return ProofDeserializePointResult{ok, opt.value.x(), opt.value.y()};
}

ProofDeserializePoint3Result path3_deserialize_point3(rust::Str value) {
    std::string cpp_value(value);
    Slic3r::ConfigOptionPoint3 opt(Slic3r::Vec3d(-999.0, -999.0, -999.0));
    bool ok = opt.deserialize(cpp_value);
    return ProofDeserializePoint3Result{ok, opt.value.x(), opt.value.y(), opt.value.z()};
}

// -------------------------------------------------------------------------------------------------
// The three option-accessor gaps -- see `config.h` for what each one measures and why.
// -------------------------------------------------------------------------------------------------

// The keys map both sides of the gap-3 proof use. Deliberately not in alphabetical order by value:
// `ConfigOptionEnumGeneric::serialize` (`Config.hpp:2182-2186`) walks `*keys_map` in the map's own
// key order and returns the FIRST key whose value matches, so a value reachable by two names would
// hide which order the walk takes. Every value here has exactly one name, and the Rust side builds
// a `BTreeMap<String, i32>` with these same pairs -- `std::map<std::string, int>` and `BTreeMap`
// order keys identically (byte-wise ascending), which is what makes the two walks comparable.
// This shim's own enum, so this translation unit can instantiate `ConfigOptionEnum<T>` at all:
// `ConfigOptionEnum<T>::get_enum_values`/`get_enum_names` (`Config.hpp:2128-2130`) are declared
// but not defined, and every specialization the vendored tree writes is in `PrintConfig.hpp`,
// outside this unit's `#include`. The values match `kProofEnumValues` below.
enum ProofEnum { peClassic = 0, peZigZag = 1, peArachne = 2 };

static const Slic3r::t_config_enum_values kProofEnumValues{
    {"arachne", 2},
    {"classic", 0},
    {"zig-zag", 1},
};

static const Slic3r::t_config_enum_names kProofEnumNames{"classic", "zig-zag", "arachne"};

} // namespace config
} // namespace proof_units
} // namespace slic3r_ffi

// The two specializations, which must be in `Slic3r` rather than in this shim's namespace.
namespace Slic3r {
template <>
const t_config_enum_values &ConfigOptionEnum<slic3r_ffi::proof_units::config::ProofEnum>::get_enum_values() {
    return slic3r_ffi::proof_units::config::kProofEnumValues;
}
template <>
const t_config_enum_names &ConfigOptionEnum<slic3r_ffi::proof_units::config::ProofEnum>::get_enum_names() {
    return slic3r_ffi::proof_units::config::kProofEnumNames;
}
} // namespace Slic3r

namespace slic3r_ffi {
namespace proof_units {
namespace config {

ProofTaggedAccessResult path3_tagged_vs_dynamic_floats(rust::Str serialized, bool nullable) {
    std::string cpp_value(serialized);

    Slic3r::DynamicConfig cfg;
    if (nullable) {
        cfg.set_key_value("k", new Slic3r::ConfigOptionFloatsNullable());
    } else {
        cfg.set_key_value("k", new Slic3r::ConfigOptionFloats());
    }
    cfg.option("k")->deserialize(cpp_value, false);

    ProofTaggedAccessResult out{};
    out.stored_is_nullable = cfg.option("k")->nullable();

    // `ConfigBase::option<TYPE>` (`:2771-2780`): tag compare, then `static_cast`.
    Slic3r::ConfigOptionFloats *tag_checked = cfg.option<Slic3r::ConfigOptionFloats>("k");
    out.tag_checked_ok = tag_checked != nullptr;
    out.tag_checked_size = tag_checked == nullptr ? 0 : tag_checked->values.size();
    out.tag_checked_value =
        (tag_checked == nullptr || tag_checked->values.empty()) ? -999.0 : tag_checked->get_at(0);

    // `ConfigOptionResolver::option<TYPE>` (`:2701-2706`), the const overload: the same body.
    const Slic3r::ConfigOptionResolver &resolver = cfg;
    out.tag_checked_const_ok = resolver.option<Slic3r::ConfigOptionFloats>("k") != nullptr;

    // `DynamicConfig::opt<T>` (`:3007-3008`): a real `dynamic_cast`.
    out.dynamic_cast_ok = cfg.opt<Slic3r::ConfigOptionFloats>("k") != nullptr;

    // The tag-MISMATCH path: `coInts` asked of a `coFloats` store. Both accessors return null,
    // and `ConfigBase::option<TYPE>` also logs a warning (`:2776`) the port does not reproduce.
    out.wrong_tag_tag_checked_ok = cfg.option<Slic3r::ConfigOptionInts>("k") != nullptr;
    out.wrong_tag_dynamic_cast_ok = cfg.opt<Slic3r::ConfigOptionInts>("k") != nullptr;

    return out;
}

ProofAbsValueAtResult path3_get_abs_value_at(rust::Str serialized, bool nullable, size_t index) {
    std::string cpp_value(serialized);

    Slic3r::DynamicConfig cfg;
    if (nullable) {
        cfg.set_key_value("k", new Slic3r::ConfigOptionFloatsNullable());
    } else {
        cfg.set_key_value("k", new Slic3r::ConfigOptionFloats());
    }
    cfg.option("k")->deserialize(cpp_value, false);

    try {
        return ProofAbsValueAtResult{true, cfg.get_abs_value_at("k", index)};
    } catch (const std::exception &) {
        return ProofAbsValueAtResult{false, -999.0};
    }
}

ProofApplyOverrideResult path3_apply_override_floats(rust::Str lhs, rust::Str rhs,
                                                     bool rhs_nullable,
                                                     rust::Slice<const int32_t> default_index) {
    Slic3r::ConfigOptionFloats lhs_opt;
    lhs_opt.deserialize(std::string(lhs), false);
    // `lhs` is always the non-nullable twin: `:747-748`/`:768-769` throw outright for a nullable
    // `this`, so that is the one combination with nothing to compare. `rhs` is either, because
    // `:773-779` (a non-nullable `rhs`) and `:782-813` (a nullable one) are two different bodies
    // and both need covering.
    std::unique_ptr<Slic3r::ConfigOption> rhs_opt;
    if (rhs_nullable) {
        rhs_opt.reset(new Slic3r::ConfigOptionFloatsNullable());
    } else {
        rhs_opt.reset(new Slic3r::ConfigOptionFloats());
    }
    rhs_opt->deserialize(std::string(rhs), false);

    std::vector<int> di(default_index.begin(), default_index.end());

    ProofApplyOverrideResult out{};
    // `overriden_by` first, on untouched values, since `apply_override` mutates `lhs_opt`.
    try {
        out.overridden_by = lhs_opt.overriden_by(rhs_opt.get());
        out.overridden_by_ok = true;
    } catch (const std::exception &) {
        out.overridden_by_ok = false;
    }
    try {
        out.modified = lhs_opt.apply_override(rhs_opt.get(), di);
        out.ok = true;
    } catch (const std::exception &) {
        out.ok = false;
    }
    out.serialized = rust::String(lhs_opt.serialize());
    return out;
}

ProofEnumPairResult path3_tagged_vs_dynamic_enum(bool store_generic, int value) {
    Slic3r::DynamicConfig cfg;
    if (store_generic) {
        cfg.set_key_value("k", new Slic3r::ConfigOptionEnumGeneric(&kProofEnumValues, value));
    } else {
        cfg.set_key_value("k", new Slic3r::ConfigOptionEnum<ProofEnum>(static_cast<ProofEnum>(value)));
    }

    ProofEnumPairResult out{};
    out.stored_tag = static_cast<int>(cfg.option("k")->type());

    // Ask for the OTHER twin than the one stored, through both accessors.
    if (store_generic) {
        Slic3r::ConfigOptionEnum<ProofEnum> *tag_checked =
            cfg.option<Slic3r::ConfigOptionEnum<ProofEnum>>("k");
        out.tag_checked_ok = tag_checked != nullptr;
        out.tag_checked_value = tag_checked == nullptr ? -999 : static_cast<int>(tag_checked->value);
        out.dynamic_cast_ok = cfg.opt<Slic3r::ConfigOptionEnum<ProofEnum>>("k") != nullptr;
    } else {
        Slic3r::ConfigOptionEnumGeneric *tag_checked = cfg.option<Slic3r::ConfigOptionEnumGeneric>("k");
        out.tag_checked_ok = tag_checked != nullptr;
        out.tag_checked_value = tag_checked == nullptr ? -999 : tag_checked->value;
        out.dynamic_cast_ok = cfg.opt<Slic3r::ConfigOptionEnumGeneric>("k") != nullptr;
    }
    // The virtual route across the pair, which is what the port's own accessor offers.
    out.get_int = cfg.option("k")->getInt();
    return out;
}

ProofOptionThrowResult path3_option_throw_floats(rust::Str serialized, bool nullable,
                                                 bool ask_wrong_tag, bool ask_missing_key) {
    Slic3r::DynamicConfig cfg;
    if (nullable) {
        cfg.set_key_value("k", new Slic3r::ConfigOptionFloatsNullable());
    } else {
        cfg.set_key_value("k", new Slic3r::ConfigOptionFloats());
    }
    cfg.option("k")->deserialize(std::string(serialized), false);

    const char *key = ask_missing_key ? "absent" : "k";
    ProofOptionThrowResult out{};

    // `ConfigBase::option_throw<TYPE>` (`:2790-2797`), non-const.
    try {
        if (ask_wrong_tag) {
            out.size = cfg.option_throw<Slic3r::ConfigOptionInts>(key)->values.size();
        } else {
            out.size = cfg.option_throw<Slic3r::ConfigOptionFloats>(key)->values.size();
        }
        out.ok = true;
    } catch (const Slic3r::UnknownOptionException &) {
        out.unknown_option = true;
    } catch (const Slic3r::BadOptionTypeException &) {
        out.bad_option_type = true;
    }

    // `ConfigOptionResolver::option_throw<TYPE>`, the const overload at `:2716-2723`, is NOT
    // measured, and cannot be: **it does not compile.** Its body is `return
    // static_cast<TYPE*>(opt);` with `opt` a `const ConfigOption*` (`:2719`, `:2722`), which casts
    // away `const` -- a hard error, not a warning. Nothing in the vendored tree instantiates it:
    // the only three `option_throw<...>` uses (`:2823`, `:2827`, `:2829`) all pass `create` and so
    // resolve to `ConfigBase::option_throw<TYPE>(key, create)` at `:2790`, the non-const one
    // measured above. Instantiating it here is what surfaced that; the error, verbatim:
    //   Config.hpp:2722:16: error: static_cast from 'const ConfigOption *' to
    //   'Slic3r::ConfigOptionFloatsTempl<false> *' casts away qualifiers
    // So the const overload has no observable behaviour at the pin to compare against, and the
    // port's immutable `option_as_tagged_throw` is measured against the non-const template's
    // semantics instead, which are identical but for the `const`.
    return out;
}

ProofCreateDefaultResult path3_create_empty_option_enum(bool enums) {
    Slic3r::ConfigOptionDef def;
    def.type = enums ? Slic3r::coEnums : Slic3r::coEnum;
    def.label = "proof-empty-enum";
    def.enum_keys_map = &kProofEnumValues;
    // No default value at all, which is what sends `create_default_option` down to
    // `create_empty_option` (`Config.cpp:341`).

    std::unique_ptr<Slic3r::ConfigOption> opt(def.create_default_option());
    Slic3r::ConfigOptionVectorBase *as_vector =
        dynamic_cast<Slic3r::ConfigOptionVectorBase *>(opt.get());
    return ProofCreateDefaultResult{
        true, static_cast<int>(opt->type()),
        as_vector == nullptr ? opt->getInt() : static_cast<int>(as_vector->size()),
        opt->nullable(), rust::String(opt->serialize())};
}

ProofVectorCopyResult path3_set_only_diff_floats(rust::Str lhs, rust::Str rhs,
                                                 rust::Slice<const int32_t> diff_index,
                                                 int32_t stride, bool keep_nil) {
    Slic3r::ConfigOptionFloats lhs_opt;
    lhs_opt.deserialize(std::string(lhs), false);
    Slic3r::ConfigOptionFloats rhs_opt;
    rhs_opt.deserialize(std::string(rhs), false);
    std::vector<int> di(diff_index.begin(), diff_index.end());

    ProofVectorCopyResult out{};
    try {
        lhs_opt.set_only_diff(&rhs_opt, di, stride, keep_nil);
        out.ok = true;
    } catch (const std::exception &) {
        out.ok = false;
    }
    out.serialized = rust::String(lhs_opt.serialize());
    return out;
}

ProofVectorCopyResult path3_set_to_index_floats(rust::Str lhs, rust::Str rhs,
                                                rust::Slice<const int32_t> dest_index,
                                                int32_t stride) {
    Slic3r::ConfigOptionFloats lhs_opt;
    lhs_opt.deserialize(std::string(lhs), false);
    Slic3r::ConfigOptionFloats rhs_opt;
    rhs_opt.deserialize(std::string(rhs), false);
    std::vector<int> di(dest_index.begin(), dest_index.end());

    ProofVectorCopyResult out{};
    try {
        lhs_opt.set_to_index(&rhs_opt, di, stride);
        out.ok = true;
    } catch (const std::exception &) {
        out.ok = false;
    }
    out.serialized = rust::String(lhs_opt.serialize());
    return out;
}

ProofVectorBaseResult path3_option_vector_base(rust::Str type_name) {
    std::string name(type_name);
    std::unique_ptr<Slic3r::ConfigOption> opt;

    if (name == "ConfigOptionFloat") {
        opt.reset(new Slic3r::ConfigOptionFloat(1.5));
    } else if (name == "ConfigOptionFloats") {
        opt.reset(new Slic3r::ConfigOptionFloats{1.5, 2.5});
    } else if (name == "ConfigOptionFloatsNullable") {
        opt.reset(new Slic3r::ConfigOptionFloatsNullable{1.5, 2.5});
    } else if (name == "ConfigOptionInt") {
        opt.reset(new Slic3r::ConfigOptionInt(7));
    } else if (name == "ConfigOptionInts") {
        opt.reset(new Slic3r::ConfigOptionInts{7, 8});
    } else if (name == "ConfigOptionIntsNullable") {
        opt.reset(new Slic3r::ConfigOptionIntsNullable{7, 8});
    } else if (name == "ConfigOptionString") {
        opt.reset(new Slic3r::ConfigOptionString("s"));
    } else if (name == "ConfigOptionStrings") {
        opt.reset(new Slic3r::ConfigOptionStrings{"s", "t"});
    } else if (name == "ConfigOptionPercent") {
        opt.reset(new Slic3r::ConfigOptionPercent(50.0));
    } else if (name == "ConfigOptionPercents") {
        opt.reset(new Slic3r::ConfigOptionPercents{50.0, 75.0});
    } else if (name == "ConfigOptionPercentsNullable") {
        opt.reset(new Slic3r::ConfigOptionPercentsNullable{50.0, 75.0});
    } else if (name == "ConfigOptionFloatOrPercent") {
        opt.reset(new Slic3r::ConfigOptionFloatOrPercent(50.0, true));
    } else if (name == "ConfigOptionFloatsOrPercents") {
        opt.reset(new Slic3r::ConfigOptionFloatsOrPercents{Slic3r::FloatOrPercent{50.0, true}});
    } else if (name == "ConfigOptionFloatsOrPercentsNullable") {
        opt.reset(
            new Slic3r::ConfigOptionFloatsOrPercentsNullable{Slic3r::FloatOrPercent{50.0, true}});
    } else if (name == "ConfigOptionPoint") {
        opt.reset(new Slic3r::ConfigOptionPoint(Slic3r::Vec2d(1.0, 2.0)));
    } else if (name == "ConfigOptionPoints") {
        opt.reset(new Slic3r::ConfigOptionPoints{Slic3r::Vec2d(1.0, 2.0)});
    } else if (name == "ConfigOptionPoint3") {
        opt.reset(new Slic3r::ConfigOptionPoint3(Slic3r::Vec3d(1.0, 2.0, 3.0)));
    } else if (name == "ConfigOptionPointsGroups") {
        opt.reset(new Slic3r::ConfigOptionPointsGroups());
    } else if (name == "ConfigOptionIntsGroups") {
        opt.reset(new Slic3r::ConfigOptionIntsGroups());
    } else if (name == "ConfigOptionBool") {
        opt.reset(new Slic3r::ConfigOptionBool(true));
    } else if (name == "ConfigOptionBools") {
        opt.reset(new Slic3r::ConfigOptionBools{true, false});
    } else if (name == "ConfigOptionBoolsNullable") {
        opt.reset(new Slic3r::ConfigOptionBoolsNullable{true, false});
    } else if (name == "ConfigOptionEnumGeneric") {
        opt.reset(new Slic3r::ConfigOptionEnumGeneric(&kProofEnumValues, 1));
    } else if (name == "ConfigOptionEnumsGeneric") {
        opt.reset(new Slic3r::ConfigOptionEnumsGeneric(std::vector<int>{0, 1}));
    } else if (name == "ConfigOptionEnumsGenericNullable") {
        opt.reset(new Slic3r::ConfigOptionEnumsGenericNullable(std::vector<int>{0, 1}));
    } else {
        return ProofVectorBaseResult{false, false, false, 0};
    }

    // `Config.cpp:497`'s own cast, unguarded -- the shape that decides whether the base is
    // reachable at all. `:1872`'s `is_vector()`-guarded `static_cast` is the same question asked
    // of a type the caller already knows is a vector, which is why `is_vector` comes back too.
    Slic3r::ConfigOptionVectorBase *as_vector =
        dynamic_cast<Slic3r::ConfigOptionVectorBase *>(opt.get());
    return ProofVectorBaseResult{true, as_vector != nullptr, opt->is_vector(),
                                 as_vector == nullptr ? 0 : as_vector->size()};
}

ProofCreateDefaultResult path3_create_default_option_enum(int default_value) {
    Slic3r::ConfigOptionDef def;
    def.type = Slic3r::coEnum;
    def.label = "proof-enum";
    def.enum_keys_map = &kProofEnumValues;
    // The default carries NO keys map -- `nullptr`, the vendored default argument (`:2149`).
    def.set_default_value(new Slic3r::ConfigOptionEnumGeneric(nullptr, default_value));

    std::unique_ptr<Slic3r::ConfigOption> opt(def.create_default_option());
    return ProofCreateDefaultResult{true, static_cast<int>(opt->type()), opt->getInt(),
                                    opt->nullable(), rust::String(opt->serialize())};
}

ProofCreateDefaultResult path3_create_default_option_enums(rust::Str serialized_values,
                                                           bool nullable) {
    std::string cpp_values(serialized_values);

    Slic3r::ConfigOptionDef def;
    def.type = Slic3r::coEnums;
    def.label = "proof-enums";
    def.nullable = nullable;
    def.enum_keys_map = &kProofEnumValues;
    // Again with no keys map of its own, so the values are set numerically rather than by name
    // (`deserialize` would need the map this default deliberately lacks).
    std::vector<int> values;
    {
        // The serialised form is read on the Rust side too; here it is just a comma-separated
        // list of integers, so that both sides start from the same literal text.
        std::istringstream is(cpp_values);
        std::string item;
        while (std::getline(is, item, ',')) {
            values.push_back(std::stoi(item));
        }
    }
    if (nullable) {
        def.set_default_value(new Slic3r::ConfigOptionEnumsGenericNullable(values));
    } else {
        def.set_default_value(new Slic3r::ConfigOptionEnumsGeneric(values));
    }

    // `getInt()` is `ConfigOption`'s throwing base (`Config.hpp:294`) for a vector option, so the
    // scalar field carries the vector's length instead -- the one `int` worth reporting here.
    std::unique_ptr<Slic3r::ConfigOption> opt(def.create_default_option());
    Slic3r::ConfigOptionVectorBase *as_vector =
        dynamic_cast<Slic3r::ConfigOptionVectorBase *>(opt.get());
    return ProofCreateDefaultResult{
        true, static_cast<int>(opt->type()),
        as_vector == nullptr ? -1 : static_cast<int>(as_vector->size()), opt->nullable(),
        rust::String(opt->serialize())};
}

} // namespace config
} // namespace proof_units
} // namespace slic3r_ffi
