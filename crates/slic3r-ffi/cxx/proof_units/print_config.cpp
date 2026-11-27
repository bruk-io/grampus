// Proof shim for task `64-foundation-print-config` (`port/manifest.tsv`'s `PrintConfig.cpp`/
// `PrintConfig.hpp` rows). Compiled onto `crates/slic3r-ffi`'s shared `cxx_build::Build` by this
// crate's `build.rs` -- the one sanctioned route a ported unit reaches the oracle by
// (`docs/PORTING.md`, "Decisions recorded on 2026-11-25"). `crates/slic3r-printconfig`'s proof
// calls this through `slic3r_proof::proof_units::print_config`.
//
// **Route note, recorded here and in this task's pull request body
// (`docs/WORKFLOW.md`, "Decisions that govern over a task's file list", corollary 1; and a
// second divergence this comment also covers, the same shape `cxx/proof_units/utils.cpp`'s own
// header comment records for its unit): the manifest row's `proof path` cell is `3`, but this
// shim is shaped like proof path 1, not proof path 3.** Path 3 means `#include`ing the vendored
// `.cpp` whole to reach an internal-linkage helper. Measured before this shim was written:
//
//   nm -m vendor/OrcaSlicer/build/arm64/src/libslic3r/CMakeFiles/libslic3r.dir/Release/PrintConfig.cpp.o \
//     | grep -E '^\S+ \(.*\) external ' | grep -v weak | c++filt | sort -u | wc -l
//   -> 340
//
// 340 strong, externally-linked `Slic3r::` symbols -- far past the 1-4 range
// `cxx/proof_units/clipper_z_utils.cpp`'s own sweep found viable for the same `#include` trick,
// and well past the 99 `cxx/proof_units/utils.cpp` already refused for the same reason.
// `crates/slic3r-ffi/build.rs`'s derived link line reproduces the oracle's test-binary link, i.e.
// the whole `liblibslic3r.a` archive, so every one of these 340 symbols is already provided by
// that archive on every build of this crate; `#include`ing `PrintConfig.cpp` here a second time
// would redefine all 340 of them in this translation unit, and `ld` would extract both archives'
// copies into the same link the instant anything needed even one -- a duplicate-symbol collision
// across most of the file, not a few-symbol risk. Path 3 as a literal `#include` is not attempted.
//
// `extern const PrintConfigDef print_config_def` (`PrintConfig.hpp:880`) is externally linked, so
// this shim calls it directly instead (the path-1 mechanism), the same shape
// `cxx/proof_units/material_type.cpp`'s and `cxx/proof_units/utils.cpp`'s shims use. The marker on
// the unit's `// PROOF: 3` test still reads `3`, because `port/port-status`'s `proof_present`
// matches the manifest row's own cell verbatim (`docs/WORKFLOW.md`, "How a proof is recognised");
// the route actually taken is recorded here and in the pull request body rather than silently
// reconciled with the cell.
#include "print_config.h"

#include "PrintConfig.hpp"

#include <string>

namespace slic3r_ffi {
namespace proof_units {
namespace print_config {

// One `std::vector<std::string>` as a `rust::Vec<rust::String>`, order preserved.
static rust::Vec<rust::String> to_rust(const std::vector<std::string> &keys) {
    rust::Vec<rust::String> out;
    for (const auto &key : keys) {
        out.push_back(rust::String(key));
    }
    return out;
}

rust::Vec<rust::String> oracle_option_keys() {
    rust::Vec<rust::String> out;
    // `ConfigDef::options` is a `std::map<t_config_option_key, ConfigOptionDef>`
    // (`Config.hpp:2658`), so this iteration is already in the map's own alphabetical-by-key
    // order -- the order the port's own table has to match.
    for (const auto &kvp : Slic3r::print_config_def.options) {
        out.push_back(rust::String(kvp.first));
    }
    return out;
}

// One `ConfigOptionDef` as an `OracleOptionInfo`, field for field. Factored out of
// `oracle_option_info` so the other 13 `ConfigDef` subclasses are read through exactly the same
// field list the registry is, rather than through a second copy of it that could drift.
static OracleOptionInfo fill_option_info(const Slic3r::ConfigOptionDef *def) {
    OracleOptionInfo info{};
    info.found = false;
    info.option_type = 0;
    info.nullable = false;
    info.has_default = false;

    if (def == nullptr) {
        return info;
    }

    info.found = true;
    info.option_type = static_cast<int32_t>(def->type);
    info.nullable = def->nullable;

    if (def->default_value) {
        info.has_default = true;
        info.default_serialized = rust::String(def->default_value->serialize());
    }

    // The GUI-presentation half of `ConfigOptionDef`. These compare bytewise because `L()` is
    // the identity extraction marker inside libslic3r (`I18N.hpp:31`), not a catalog lookup --
    // see the bridge module's doc comment.
    info.gui_type = static_cast<int32_t>(def->gui_type);
    info.gui_flags = rust::String(def->gui_flags);
    info.plugin_type = rust::String(def->plugin_type);
    info.label = rust::String(def->label);
    info.full_label = rust::String(def->full_label);
    info.printer_technology = static_cast<int32_t>(def->printer_technology);
    info.category = rust::String(def->category);
    info.tooltip = rust::String(def->tooltip);
    info.sidetext = rust::String(def->sidetext);
    info.cli = rust::String(def->cli);
    info.cli_params = rust::String(def->cli_params);
    info.ratio_over = rust::String(def->ratio_over);
    info.multiline = def->multiline;
    info.full_width = def->full_width;
    info.is_code = def->is_code;
    info.readonly = def->readonly;
    info.height = static_cast<int32_t>(def->height);
    info.width = static_cast<int32_t>(def->width);
    info.min = def->min;
    info.max = def->max;
    info.max_literal = def->max_literal;
    info.mode = static_cast<int32_t>(def->mode);

    for (const auto &alias : def->aliases) {
        info.aliases.push_back(rust::String(alias));
    }
    for (const auto &shortcut : def->shortcut) {
        info.shortcut.push_back(rust::String(shortcut));
    }
    for (const auto &value : def->enum_values) {
        info.enum_values.push_back(rust::String(value));
    }
    for (const auto &label : def->enum_labels) {
        info.enum_labels.push_back(rust::String(label));
    }
    // `def->enum_keys_map` is `const t_config_enum_values*` (`std::map<std::string, int>`),
    // never null for an in-scope enum option at the pin (every `coEnum`/`coEnums` row in
    // `PrintConfig.cpp` sets it); a null map here is reported as an empty list rather than
    // dereferenced.
    if (def->enum_keys_map != nullptr) {
        for (const auto &kvp : *def->enum_keys_map) {
            OracleEnumKeyValue entry;
            entry.name = rust::String(kvp.first);
            entry.value = static_cast<int32_t>(kvp.second);
            info.enum_keys_map.push_back(entry);
        }
    }

    return info;
}

OracleOptionInfo oracle_option_info(rust::Str key) {
    return fill_option_info(Slic3r::print_config_def.get(std::string(key)));
}

OracleLegacyResult oracle_handle_legacy(rust::Str opt_key, rust::Str value) {
    std::string k(opt_key);
    std::string v(value);
    Slic3r::PrintConfigDef::handle_legacy(k, v);
    OracleLegacyResult out;
    out.opt_key = rust::String(k);
    out.value = rust::String(v);
    return out;
}

// The four key lists are each read straight off their own public const accessor, in the
// `std::vector`'s own order. See this unit's header for why the two filament lists come back
// empty at the pin.

rust::Vec<rust::String> oracle_extruder_option_keys() {
    return to_rust(Slic3r::print_config_def.extruder_option_keys());
}

rust::Vec<rust::String> oracle_extruder_retract_keys() {
    return to_rust(Slic3r::print_config_def.extruder_retract_keys());
}

rust::Vec<rust::String> oracle_filament_option_keys() {
    return to_rust(Slic3r::print_config_def.filament_option_keys());
}

rust::Vec<rust::String> oracle_filament_retract_keys() {
    return to_rust(Slic3r::print_config_def.filament_retract_keys());
}

// One `ConfigDef` of the other 13 subclasses, by class name. The four with an `extern const`
// global are read off it; the nine without one are default-constructed here, which runs the
// same externally-linked constructor the GUI runs.
//
// The nine are returned by value into a `static` of this function, so the pointer stays valid
// after the call: each is built once, on first use, and never mutated afterwards -- the same
// lifetime the four globals have.
static const Slic3r::ConfigDef *other_def(rust::Str which) {
    const std::string name(which);
    if (name == "CLIActionsConfigDef") {
        return &Slic3r::cli_actions_config_def;
    }
    if (name == "CLITransformConfigDef") {
        return &Slic3r::cli_transform_config_def;
    }
    if (name == "CLIMiscConfigDef") {
        return &Slic3r::cli_misc_config_def;
    }
    if (name == "CustomGcodeSpecificConfigDef") {
        return &Slic3r::custom_gcode_specific_config_def;
    }
    if (name == "ReadOnlySlicingStatesConfigDef") {
        static const Slic3r::ReadOnlySlicingStatesConfigDef def;
        return &def;
    }
    if (name == "ReadWriteSlicingStatesConfigDef") {
        static const Slic3r::ReadWriteSlicingStatesConfigDef def;
        return &def;
    }
    if (name == "OtherSlicingStatesConfigDef") {
        static const Slic3r::OtherSlicingStatesConfigDef def;
        return &def;
    }
    if (name == "PrintStatisticsConfigDef") {
        static const Slic3r::PrintStatisticsConfigDef def;
        return &def;
    }
    if (name == "ObjectsInfoConfigDef") {
        static const Slic3r::ObjectsInfoConfigDef def;
        return &def;
    }
    if (name == "DimensionsConfigDef") {
        static const Slic3r::DimensionsConfigDef def;
        return &def;
    }
    if (name == "TemperaturesConfigDef") {
        static const Slic3r::TemperaturesConfigDef def;
        return &def;
    }
    if (name == "TimestampsConfigDef") {
        static const Slic3r::TimestampsConfigDef def;
        return &def;
    }
    if (name == "OtherPresetsConfigDef") {
        static const Slic3r::OtherPresetsConfigDef def;
        return &def;
    }
    return nullptr;
}

rust::Vec<rust::String> oracle_other_def_keys(rust::Str which) {
    rust::Vec<rust::String> out;
    const Slic3r::ConfigDef *def = other_def(which);
    if (def == nullptr) {
        return out;
    }
    // `ConfigDef::options` is a `std::map`, so this is its own alphabetical-by-key order.
    for (const auto &kvp : def->options) {
        out.push_back(rust::String(kvp.first));
    }
    return out;
}

OracleOptionInfo oracle_other_def_option_info(rust::Str which, rust::Str key) {
    const Slic3r::ConfigDef *table = other_def(which);
    if (table == nullptr) {
        OracleOptionInfo info{};
        info.found = false;
        return info;
    }
    return fill_option_info(table->get(std::string(key)));
}

rust::Vec<rust::String> oracle_custom_gcode_placeholder_keys() {
    rust::Vec<rust::String> out;
    for (const auto &kvp : Slic3r::custom_gcode_specific_placeholders()) {
        out.push_back(rust::String(kvp.first));
    }
    return out;
}

rust::Vec<rust::String> oracle_custom_gcode_placeholders(rust::Str key) {
    const auto &map = Slic3r::custom_gcode_specific_placeholders();
    auto it = map.find(std::string(key));
    if (it == map.end()) {
        return rust::Vec<rust::String>();
    }
    // The vector's own braced-initialiser order, not sorted here.
    return to_rust(it->second);
}

} // namespace print_config
} // namespace proof_units
} // namespace slic3r_ffi
