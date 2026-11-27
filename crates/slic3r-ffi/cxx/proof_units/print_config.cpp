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

#include <limits>
#include <memory>
#include <set>
#include <string>
#include <vector>

// `get_shaper_type_values_for_flavor` (`PrintConfig.cpp:159-173`) is defined with external
// linkage and declared in no header -- measured: `grep -rn get_shaper_type_values_for_flavor
// vendor/OrcaSlicer/src/` finds only its definition, so it has no caller at the pin and nothing
// declares it. It is declared here so this shim can call it; declaring an
// externally-linked symbol in this file changes nothing under `vendor/OrcaSlicer/`.
namespace Slic3r {
std::vector<std::string> get_shaper_type_values_for_flavor(GCodeFlavor flavor);

} // namespace Slic3r

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

// One `StaticPrintConfig`-derived class's key list and its defaults' serialized values, read
// off the `CLASS_NAME::defaults()` object. Templates rather than 22 functions, because every
// one of the 11 classes is read the same way; `KEYS_OR_DEFAULTS` dispatches the class name once
// for both.
template <class T> static rust::Vec<rust::String> class_keys() {
    return to_rust(T::defaults().keys_ref());
}

template <class T> static rust::Vec<rust::String> class_defaults() {
    const T &defaults = T::defaults();
    rust::Vec<rust::String> out;
    for (const std::string &key : defaults.keys_ref()) {
        const Slic3r::ConfigOption *opt = defaults.optptr(key);
        // `keys_ref()` only ever holds a key `optptr` resolves, by construction in
        // `StaticCache::finalize`, so a null here would be a broken cache rather than a missing
        // option; it is reported as a value rather than dereferenced.
        out.push_back(rust::String(opt == nullptr ? std::string("<no option>") : opt->serialize()));
    }
    return out;
}

#define SLIC3R_FFI_STATIC_CLASS_DISPATCH(FN)                                                       \
    if (name == "PrintObjectConfig") {                                                             \
        return FN<Slic3r::PrintObjectConfig>();                                                    \
    }                                                                                              \
    if (name == "PrintRegionConfig") {                                                             \
        return FN<Slic3r::PrintRegionConfig>();                                                    \
    }                                                                                              \
    if (name == "MachineEnvelopeConfig") {                                                         \
        return FN<Slic3r::MachineEnvelopeConfig>();                                                \
    }                                                                                              \
    if (name == "GCodeConfig") {                                                                   \
        return FN<Slic3r::GCodeConfig>();                                                          \
    }                                                                                              \
    if (name == "PrintConfig") {                                                                   \
        return FN<Slic3r::PrintConfig>();                                                          \
    }                                                                                              \
    if (name == "FullPrintConfig") {                                                               \
        return FN<Slic3r::FullPrintConfig>();                                                      \
    }                                                                                              \
    if (name == "SLAPrintConfig") {                                                                \
        return FN<Slic3r::SLAPrintConfig>();                                                       \
    }                                                                                              \
    if (name == "SLAPrintObjectConfig") {                                                          \
        return FN<Slic3r::SLAPrintObjectConfig>();                                                 \
    }                                                                                              \
    if (name == "SLAMaterialConfig") {                                                             \
        return FN<Slic3r::SLAMaterialConfig>();                                                    \
    }                                                                                              \
    if (name == "SLAPrinterConfig") {                                                              \
        return FN<Slic3r::SLAPrinterConfig>();                                                     \
    }                                                                                              \
    if (name == "SLAFullPrintConfig") {                                                            \
        return FN<Slic3r::SLAFullPrintConfig>();                                                   \
    }                                                                                              \
    return rust::Vec<rust::String>();

rust::Vec<rust::String> oracle_static_class_keys(rust::Str which) {
    const std::string name(which);
    SLIC3R_FFI_STATIC_CLASS_DISPATCH(class_keys)
}

rust::Vec<rust::String> oracle_static_class_defaults(rust::Str which) {
    const std::string name(which);
    SLIC3R_FFI_STATIC_CLASS_DISPATCH(class_defaults)
}

#undef SLIC3R_FFI_STATIC_CLASS_DISPATCH

rust::Vec<rust::String> oracle_custom_gcode_placeholders(rust::Str key) {
    const auto &map = Slic3r::custom_gcode_specific_placeholders();
    auto it = map.find(std::string(key));
    if (it == map.end()) {
        return rust::Vec<rust::String>();
    }
    // The vector's own braced-initialiser order, not sorted here.
    return to_rust(it->second);
}

// One of the seven option-set globals, by variable name.
rust::Vec<rust::String> oracle_option_set(rust::Str which) {
    const std::string name(which);
    const std::set<std::string> *set = nullptr;
    if (name == "print_options_with_variant") {
        set = &Slic3r::print_options_with_variant;
    } else if (name == "filament_options_with_variant") {
        set = &Slic3r::filament_options_with_variant;
    } else if (name == "printer_extruder_options") {
        set = &Slic3r::printer_extruder_options;
    } else if (name == "printer_options_with_variant_1") {
        set = &Slic3r::printer_options_with_variant_1;
    } else if (name == "printer_options_with_variant_2") {
        set = &Slic3r::printer_options_with_variant_2;
    } else if (name == "empty_options") {
        set = &Slic3r::empty_options;
    } else if (name == "filament_dev_options") {
        set = &Slic3r::filament_dev_options;
    }
    rust::Vec<rust::String> out;
    if (set == nullptr) {
        return out;
    }
    // A `std::set<std::string>`, so this is its own bytewise-ascending order.
    for (const std::string &key : *set) {
        out.push_back(rust::String(key));
    }
    return out;
}

rust::Vec<rust::String> oracle_shaper_type_values_for_flavor(int32_t flavor) {
    return to_rust(Slic3r::get_shaper_type_values_for_flavor(Slic3r::GCodeFlavor(flavor)));
}

rust::String oracle_extruder_variant_string(int32_t extruder_type, int32_t nozzle_volume_type) {
    return rust::String(Slic3r::get_extruder_variant_string(
        Slic3r::ExtruderType(extruder_type), Slic3r::NozzleVolumeType(nozzle_volume_type)));
}

rust::String oracle_nozzle_volume_type_string(int32_t nozzle_volume_type) {
    return rust::String(
        Slic3r::get_nozzle_volume_type_string(Slic3r::NozzleVolumeType(nozzle_volume_type)));
}

int32_t oracle_convert_to_nvt_type(rust::Str variant_str) {
    return static_cast<int32_t>(Slic3r::convert_to_nvt_type(std::string(variant_str)));
}

static std::vector<std::string> from_rust(const rust::Vec<rust::String> &values) {
    std::vector<std::string> out;
    out.reserve(values.size());
    for (const rust::String &value : values) {
        out.push_back(std::string(value));
    }
    return out;
}

static std::vector<int> from_rust_ints(const rust::Vec<int32_t> &values) {
    std::vector<int> out;
    out.reserve(values.size());
    for (int32_t value : values) {
        out.push_back(static_cast<int>(value));
    }
    return out;
}

int32_t oracle_find_variant_index(rust::Str variant, int32_t variant_id_1based,
                                  const rust::Vec<rust::String> &variant_list,
                                  const rust::Vec<int32_t> &variant_ids_1based) {
    return static_cast<int32_t>(Slic3r::find_variant_index(
        std::string(variant), static_cast<int>(variant_id_1based), from_rust(variant_list),
        from_rust_ints(variant_ids_1based)));
}

rust::Vec<int32_t> oracle_map_variant_indices(const rust::Vec<rust::String> &variants,
                                              const rust::Vec<int32_t> &ids,
                                              const rust::Vec<rust::String> &from_variants,
                                              const rust::Vec<int32_t> &from_ids) {
    const std::vector<int> mapped =
        Slic3r::map_variant_indices(from_rust(variants), from_rust_ints(ids),
                                    from_rust(from_variants), from_rust_ints(from_ids));
    rust::Vec<int32_t> out;
    for (int value : mapped) {
        out.push_back(static_cast<int32_t>(value));
    }
    return out;
}

int32_t oracle_config_index_base(int32_t volume_type, int32_t extruder_type,
                                 int32_t variant_id_1based,
                                 const rust::Vec<rust::String> &variant_list,
                                 const rust::Vec<int32_t> &variant_ids_1based) {
    return static_cast<int32_t>(Slic3r::get_config_index_base(
        Slic3r::NozzleVolumeType(volume_type), Slic3r::ExtruderType(extruder_type),
        static_cast<int>(variant_id_1based), from_rust(variant_list),
        from_rust_ints(variant_ids_1based)));
}

// The flat encoding described in the header: entry 0 is the map count, then one entry per map.
template <typename K>
static rust::Vec<rust::String> encode_maps(const std::vector<std::map<K, int>> &maps) {
    rust::Vec<rust::String> out;
    out.push_back(rust::String(std::to_string(maps.size())));
    for (const std::map<K, int> &m : maps) {
        std::string entry;
        for (const std::pair<const K, int> &kv : m) {
            if (!entry.empty())
                entry += ";";
            entry += std::to_string(static_cast<int>(kv.first));
            entry += "=";
            entry += std::to_string(kv.second);
        }
        out.push_back(rust::String(entry));
    }
    return out;
}

template <typename K>
static std::vector<std::map<K, int>> decode_maps(const rust::Vec<rust::String> &flat) {
    std::vector<std::map<K, int>> maps;
    if (flat.size() == 0)
        return maps;
    // Entry 0 is the count; the remaining entries are the maps, so a truncated list yields
    // fewer maps rather than reading past the end.
    for (size_t i = 1; i < flat.size(); ++i) {
        std::map<K, int> m;
        const std::string entry(flat[i]);
        size_t pos = 0;
        while (pos < entry.size()) {
            const size_t semi = entry.find(';', pos);
            const std::string pair =
                entry.substr(pos, semi == std::string::npos ? std::string::npos : semi - pos);
            const size_t eq = pair.find('=');
            if (eq != std::string::npos) {
                m[static_cast<K>(std::stoi(pair.substr(0, eq)))] = std::stoi(pair.substr(eq + 1));
            }
            if (semi == std::string::npos)
                break;
            pos = semi + 1;
        }
        maps.push_back(std::move(m));
    }
    return maps;
}

rust::Vec<rust::String> oracle_get_extruder_ams_count(const rust::Vec<rust::String> &strs) {
    // `get_extruder_ams_count` (`PrintConfig.cpp:884`) calls `stoi` unguarded under an
    // `assert(numbers.size() == 2)` that the oracle's Release build compiles out, so a
    // non-numeric token throws `std::invalid_argument` -- and an exception escaping a cxx
    // bridge aborts the test binary. Reported as a refusal instead. Its nozzle-stats twin
    // (`:957-964`) was explicitly hardened against exactly these shapes; this one was not.
    try {
        return encode_maps<int>(Slic3r::get_extruder_ams_count(from_rust(strs)));
    } catch (const std::exception &) {
        rust::Vec<rust::String> out;
        out.push_back(rust::String("<threw>"));
        return out;
    }
}

rust::Vec<rust::String> oracle_save_extruder_ams_count_to_string(
    const rust::Vec<rust::String> &flat) {
    return to_rust(Slic3r::save_extruder_ams_count_to_string(decode_maps<int>(flat)));
}

rust::Vec<rust::String> oracle_get_extruder_nozzle_stats(const rust::Vec<rust::String> &strs) {
    // Guarded the same way as its sibling above, although this body's own guards
    // (`PrintConfig.cpp:957-964`) mean nothing here is expected to throw -- the shim is not the
    // place to assume that.
    try {
        return encode_maps<Slic3r::NozzleVolumeType>(
            Slic3r::get_extruder_nozzle_stats(from_rust(strs)));
    } catch (const std::exception &) {
        rust::Vec<rust::String> out;
        out.push_back(rust::String("<threw>"));
        return out;
    }
}

rust::Vec<rust::String> oracle_save_extruder_nozzle_stats_to_string(
    const rust::Vec<rust::String> &flat) {
    return to_rust(Slic3r::save_extruder_nozzle_stats_to_string(
        decode_maps<Slic3r::NozzleVolumeType>(flat)));
}

uint64_t oracle_extruder_index_of_defaults(uint32_t filament_id) {
    return static_cast<uint64_t>(
        Slic3r::get_extruder_index(Slic3r::GCodeConfig::defaults(), filament_id));
}

double oracle_nozzle_diameter_for_filament_of_defaults(int32_t filament_id, bool is_bbl_printer) {
    return Slic3r::nozzle_diameter_for_filament(Slic3r::PrintConfig::defaults(),
                                                static_cast<int>(filament_id), is_bbl_printer);
}

// One `ConfigBase` serialized whole: every key in the `std::map`'s own order, one
// `"<key>=<serialized>"` entry each. The comparison form for every `DynamicPrintConfig` call
// below -- a config's observable content is its keys and their serialized values.
static rust::Vec<rust::String> serialize_config(const Slic3r::ConfigBase &config) {
    rust::Vec<rust::String> out;
    for (const std::string &key : config.keys()) {
        const Slic3r::ConfigOption *opt = config.option(key);
        out.push_back(rust::String(key + "=" +
                                   (opt == nullptr ? std::string("<absent>") : opt->serialize())));
    }
    return out;
}

rust::Vec<rust::String> oracle_full_print_config_serialized() {
    Slic3r::DynamicPrintConfig config = Slic3r::DynamicPrintConfig::full_print_config();
    return serialize_config(config);
}

rust::Vec<rust::String>
oracle_new_from_defaults_keys_serialized(const rust::Vec<rust::String> &keys) {
    std::vector<std::string> key_vec;
    for (const rust::String &key : keys) {
        key_vec.push_back(std::string(key));
    }
    std::unique_ptr<Slic3r::DynamicPrintConfig> config(
        Slic3r::DynamicPrintConfig::new_from_defaults_keys(key_vec));
    return serialize_config(*config);
}

template <class T> static rust::Vec<rust::String> dynamic_from_static() {
    Slic3r::DynamicPrintConfig config(static_cast<const Slic3r::StaticPrintConfig &>(T::defaults()));
    return serialize_config(config);
}

// Only the eight non-derived classes: the three derived ones inherit `StaticPrintConfig`
// through each of their parents, so a conversion to it is ambiguous and the vendored
// `DynamicPrintConfig(const StaticPrintConfig&)` constructor cannot be called with one at all.
// That is exactly why `full_print_config()` casts its `FullPrintConfig` to `PrintRegionConfig`
// first (`PrintConfig.cpp:10309`), and why this shim offers the three through
// `oracle_full_print_config_serialized` rather than here. An unknown or derived name yields an
// empty list, which the port's own proof asserts against.
rust::Vec<rust::String> oracle_dynamic_from_static_serialized(rust::Str which) {
    const std::string name(which);
    if (name == "PrintObjectConfig") {
        return dynamic_from_static<Slic3r::PrintObjectConfig>();
    }
    if (name == "PrintRegionConfig") {
        return dynamic_from_static<Slic3r::PrintRegionConfig>();
    }
    if (name == "MachineEnvelopeConfig") {
        return dynamic_from_static<Slic3r::MachineEnvelopeConfig>();
    }
    if (name == "GCodeConfig") {
        return dynamic_from_static<Slic3r::GCodeConfig>();
    }
    if (name == "SLAPrintConfig") {
        return dynamic_from_static<Slic3r::SLAPrintConfig>();
    }
    if (name == "SLAPrintObjectConfig") {
        return dynamic_from_static<Slic3r::SLAPrintObjectConfig>();
    }
    if (name == "SLAMaterialConfig") {
        return dynamic_from_static<Slic3r::SLAMaterialConfig>();
    }
    if (name == "SLAPrinterConfig") {
        return dynamic_from_static<Slic3r::SLAPrinterConfig>();
    }
    return rust::Vec<rust::String>();
}

uint64_t oracle_parameter_size_of_full(rust::Str param_name, uint64_t extruder_nums) {
    Slic3r::DynamicPrintConfig config = Slic3r::DynamicPrintConfig::full_print_config();
    return static_cast<uint64_t>(
        config.get_parameter_size(std::string(param_name), static_cast<size_t>(extruder_nums)));
}

double oracle_min_object_distance(rust::Str print_sequence, rust::Str clearance_radius,
                                  rust::Str printer_technology) {
    Slic3r::DynamicPrintConfig config;
    Slic3r::ConfigSubstitutionContext ctx(Slic3r::ForwardCompatibilitySubstitutionRule::Disable);
    // An empty override leaves the key absent, which is what the function's two null checks and
    // its `ptUnknown` fallback turn on.
    // As in `oracle_filament_type_probe`: a refused value must not let an exception escape the
    // bridge. A refusal yields NaN, which no real result is, so the Rust side sees a mismatch
    // rather than the binary aborting.
    try {
        if (!print_sequence.empty()) {
            config.set_deserialize("print_sequence", std::string(print_sequence), ctx);
        }
        if (!clearance_radius.empty()) {
            config.set_deserialize("extruder_clearance_radius", std::string(clearance_radius),
                                   ctx);
        }
        if (!printer_technology.empty()) {
            config.set_deserialize("printer_technology", std::string(printer_technology), ctx);
        }
    } catch (const std::exception &) {
        return std::numeric_limits<double>::quiet_NaN();
    }
    return Slic3r::min_object_distance(config);
}

rust::Vec<rust::String> oracle_filament_type_probe(rust::Str filament_type, rust::Str filament_id,
                                                   rust::Str filament_is_support,
                                                   rust::Str filament_vendor, uint64_t id) {
    Slic3r::DynamicPrintConfig config;
    Slic3r::ConfigSubstitutionContext ctx(Slic3r::ForwardCompatibilitySubstitutionRule::Disable);
    // A `?` marks the key as absent; an empty string sets it to an empty vector, which is the
    // other case the body's "treat an empty option the same as a missing one" guard covers.
    // `set_deserialize` throws `BadOptionValueException` for a value the option cannot parse,
    // and an exception escaping a `cxx` bridge function declared to return a value aborts the
    // whole test binary rather than failing one case. A refused value is reported as the marker
    // below instead, so a probe the oracle rejects shows up as a mismatch the Rust side can
    // print.
    bool refused = false;
    std::string refused_key;
    auto set = [&](const char *key, rust::Str value) {
        const std::string v(value);
        if (v == "?") {
            return;
        }
        try {
            config.set_deserialize(key, v, ctx);
        } catch (const std::exception &) {
            refused = true;
            if (refused_key.empty()) {
                refused_key = key;
            }
        }
    };
    set("filament_type", filament_type);
    set("filament_id", filament_id);
    set("filament_is_support", filament_is_support);
    set("filament_vendor", filament_vendor);

    rust::Vec<rust::String> out;
    if (refused) {
        out.push_back(rust::String("<refused:" + refused_key + ">"));
        return out;
    }
    std::string displayed;
    const std::string returned = config.get_filament_type(displayed, static_cast<int>(id));
    out.push_back(rust::String(returned));
    out.push_back(rust::String(displayed));
    out.push_back(rust::String(config.get_filament_vendor()));
    out.push_back(rust::String(config.get_filament_type()));
    return out;
}

// A `DynamicPrintConfig` carrying exactly the given `"<key>=<value>"` assignments, deserialized
// in the order given. `ok` is false when any assignment was refused, which the callers report
// rather than letting an exception escape the bridge.
static Slic3r::DynamicPrintConfig config_from(const rust::Vec<rust::String> &keys_and_values,
                                              bool &ok) {
    Slic3r::DynamicPrintConfig config;
    Slic3r::ConfigSubstitutionContext ctx(Slic3r::ForwardCompatibilitySubstitutionRule::Disable);
    ok = true;
    for (const rust::String &entry : keys_and_values) {
        const std::string text(entry);
        const size_t eq = text.find('=');
        if (eq == std::string::npos) {
            ok = false;
            continue;
        }
        try {
            config.set_deserialize(text.substr(0, eq), text.substr(eq + 1), ctx);
        } catch (const std::exception &) {
            ok = false;
        }
    }
    return config;
}

rust::Vec<rust::String> oracle_config_from(const rust::Vec<rust::String> &keys_and_values) {
    bool ok = false;
    Slic3r::DynamicPrintConfig config = config_from(keys_and_values, ok);
    if (!ok) {
        rust::Vec<rust::String> out;
        out.push_back(rust::String("<refused>"));
        return out;
    }
    return serialize_config(config);
}

rust::Vec<rust::String> oracle_normalize_fdm_1(const rust::Vec<rust::String> &keys_and_values) {
    bool ok = false;
    Slic3r::DynamicPrintConfig config = config_from(keys_and_values, ok);
    if (!ok) {
        rust::Vec<rust::String> out;
        out.push_back(rust::String("<refused>"));
        return out;
    }
    config.normalize_fdm_1();
    return serialize_config(config);
}

rust::Vec<rust::String> oracle_handle_legacy_sla(const rust::Vec<rust::String> &keys_and_values) {
    bool ok = false;
    Slic3r::DynamicPrintConfig config = config_from(keys_and_values, ok);
    if (!ok) {
        rust::Vec<rust::String> out;
        out.push_back(rust::String("<refused>"));
        return out;
    }
    Slic3r::handle_legacy_sla(config);
    return serialize_config(config);
}

int32_t oracle_is_using_different_extruders(const rust::Vec<rust::String> &keys_and_values) {
    bool ok = false;
    Slic3r::DynamicPrintConfig config = config_from(keys_and_values, ok);
    if (!ok) {
        return -1;
    }
    return config.is_using_different_extruders() ? 1 : 0;
}

int32_t oracle_has_multi_variant_filament(const rust::Vec<rust::String> &keys_and_values) {
    bool ok = false;
    Slic3r::DynamicPrintConfig config = config_from(keys_and_values, ok);
    if (!ok) {
        return -1;
    }
    return config.has_multi_variant_filament() ? 1 : 0;
}

rust::Vec<int32_t>
oracle_support_different_extruders(const rust::Vec<rust::String> &keys_and_values) {
    rust::Vec<int32_t> out;
    bool ok = false;
    Slic3r::DynamicPrintConfig config = config_from(keys_and_values, ok);
    if (!ok) {
        out.push_back(-1);
        return out;
    }
    // A sentinel the method never writes, so "left untouched" is distinguishable from any value
    // it could write -- the vendored body writes the out-parameter only inside its one branch.
    int extruder_count = -12345;
    const bool result = config.support_different_extruders(extruder_count);
    out.push_back(result ? 1 : 0);
    out.push_back(extruder_count == -12345 ? 0 : 1);
    out.push_back(static_cast<int32_t>(extruder_count));
    return out;
}

rust::Vec<int32_t>
oracle_extruder_nozzle_volume_count(const rust::Vec<rust::String> &keys_and_values,
                                    int32_t extruder_count) {
    rust::Vec<int32_t> out;
    bool ok = false;
    Slic3r::DynamicPrintConfig config = config_from(keys_and_values, ok);
    if (!ok) {
        out.push_back(-1);
        return out;
    }
    std::vector<std::vector<Slic3r::NozzleVolumeType>> nozzle_volume_types;
    const int count =
        config.get_extruder_nozzle_volume_count(static_cast<int>(extruder_count),
                                                nozzle_volume_types);
    out.push_back(static_cast<int32_t>(count));
    out.push_back(static_cast<int32_t>(nozzle_volume_types.size()));
    for (const auto &per_extruder : nozzle_volume_types) {
        out.push_back(static_cast<int32_t>(per_extruder.size()));
        for (const Slic3r::NozzleVolumeType type : per_extruder) {
            out.push_back(static_cast<int32_t>(type));
        }
    }
    return out;
}

int32_t oracle_index_for_extruder(const rust::Vec<rust::String> &keys_and_values,
                                  int32_t extruder_or_filament_id, rust::Str id_name,
                                  int32_t extruder_type, int32_t nozzle_volume_type,
                                  rust::Str variant_name, uint32_t stride) {
    bool ok = false;
    Slic3r::DynamicPrintConfig config = config_from(keys_and_values, ok);
    if (!ok) {
        return -12345;
    }
    return static_cast<int32_t>(config.get_index_for_extruder(
        static_cast<int>(extruder_or_filament_id), std::string(id_name),
        Slic3r::ExtruderType(extruder_type), Slic3r::NozzleVolumeType(nozzle_volume_type),
        std::string(variant_name), static_cast<unsigned int>(stride)));
}

// As `config_from`, but starting from the named base. See the header for why "full" exists.
static Slic3r::DynamicPrintConfig config_from_base(rust::Str base,
                                                   const rust::Vec<rust::String> &keys_and_values,
                                                   bool &ok) {
    ok = true;
    Slic3r::DynamicPrintConfig config;
    if (std::string(base) == "full") {
        config = Slic3r::DynamicPrintConfig::full_print_config();
    } else if (std::string(base) != "empty") {
        ok = false;
        return config;
    }
    Slic3r::ConfigSubstitutionContext ctx(Slic3r::ForwardCompatibilitySubstitutionRule::Disable);
    for (const rust::String &entry : keys_and_values) {
        const std::string text(entry);
        const size_t eq = text.find('=');
        if (eq == std::string::npos) {
            ok = false;
            continue;
        }
        try {
            config.set_deserialize(text.substr(0, eq), text.substr(eq + 1), ctx);
        } catch (const std::exception &) {
            ok = false;
        }
    }
    return config;
}

rust::Vec<rust::String> oracle_config_from_base(rust::Str base,
                                                const rust::Vec<rust::String> &keys_and_values) {
    bool ok = false;
    Slic3r::DynamicPrintConfig config = config_from_base(base, keys_and_values, ok);
    if (!ok) {
        rust::Vec<rust::String> out;
        out.push_back(rust::String("<refused>"));
        return out;
    }
    return serialize_config(config);
}

rust::Vec<rust::String> oracle_normalize_fdm(rust::Str base,
                                             const rust::Vec<rust::String> &keys_and_values,
                                             int32_t used_filaments) {
    bool ok = false;
    Slic3r::DynamicPrintConfig config = config_from_base(base, keys_and_values, ok);
    if (!ok) {
        rust::Vec<rust::String> out;
        out.push_back(rust::String("<refused>"));
        return out;
    }
    config.normalize_fdm(static_cast<int>(used_filaments));
    return serialize_config(config);
}

rust::Vec<rust::String> oracle_set_num_extruders(rust::Str base,
                                                 const rust::Vec<rust::String> &keys_and_values,
                                                 uint32_t num_extruders) {
    bool ok = false;
    Slic3r::DynamicPrintConfig config = config_from_base(base, keys_and_values, ok);
    if (!ok) {
        rust::Vec<rust::String> out;
        out.push_back(rust::String("<refused>"));
        return out;
    }
    config.set_num_extruders(static_cast<unsigned int>(num_extruders));
    return serialize_config(config);
}

rust::Vec<rust::String> oracle_set_num_filaments(rust::Str base,
                                                 const rust::Vec<rust::String> &keys_and_values,
                                                 uint32_t num_filaments) {
    bool ok = false;
    Slic3r::DynamicPrintConfig config = config_from_base(base, keys_and_values, ok);
    if (!ok) {
        rust::Vec<rust::String> out;
        out.push_back(rust::String("<refused>"));
        return out;
    }
    config.set_num_filaments(static_cast<unsigned int>(num_filaments));
    return serialize_config(config);
}

rust::Vec<rust::String>
oracle_normalize_filament_values_to_variants(const rust::Vec<rust::String> &keys_and_values) {
    bool ok = false;
    Slic3r::DynamicPrintConfig config = config_from(keys_and_values, ok);
    if (!ok) {
        rust::Vec<rust::String> out;
        out.push_back(rust::String("<refused>"));
        return out;
    }
    Slic3r::normalize_filament_values_to_variants(config);
    return serialize_config(config);
}

rust::Vec<rust::String>
oracle_set_filament_dev_options(const rust::Vec<rust::String> &keys_and_values,
                                const rust::Vec<rust::String> &filament_a,
                                const rust::Vec<rust::String> &filament_b) {
    bool ok = false, ok_a = false, ok_b = false;
    Slic3r::DynamicPrintConfig config = config_from(keys_and_values, ok);
    Slic3r::DynamicPrintConfig a = config_from(filament_a, ok_a);
    Slic3r::DynamicPrintConfig b = config_from(filament_b, ok_b);
    if (!ok || !ok_a || !ok_b) {
        rust::Vec<rust::String> out;
        out.push_back(rust::String("<refused>"));
        return out;
    }
    std::vector<const Slic3r::DynamicPrintConfig *> filaments;
    if (!filament_a.empty()) {
        filaments.push_back(&a);
    }
    if (!filament_b.empty()) {
        filaments.push_back(&b);
    }
    Slic3r::set_filament_dev_options(config, filaments);
    return serialize_config(config);
}

int32_t oracle_has_skirt(rust::Str base, const rust::Vec<rust::String> &keys_and_values) {
    bool ok = false;
    Slic3r::DynamicPrintConfig config = config_from_base(base, keys_and_values, ok);
    if (!ok) {
        return -1;
    }
    return Slic3r::has_skirt(config) ? 1 : 0;
}

double oracle_real_skirt_dist(rust::Str base,
                              const rust::Vec<rust::String> &keys_and_values) {
    bool ok = false;
    Slic3r::DynamicPrintConfig config = config_from_base(base, keys_and_values, ok);
    if (!ok) {
        return std::numeric_limits<double>::quiet_NaN();
    }
    // Returned as a double so the bridge carries the float exactly; the port compares the same
    // widening of its own `f32`.
    return static_cast<double>(Slic3r::get_real_skirt_dist(config));
}

int32_t oracle_is_xl_printer_dynamic(rust::Str base,
                                     const rust::Vec<rust::String> &keys_and_values) {
    bool ok = false;
    Slic3r::DynamicPrintConfig config = config_from_base(base, keys_and_values, ok);
    if (!ok) {
        return -1;
    }
    return Slic3r::is_XL_printer(config) ? 1 : 0;
}

rust::Vec<rust::String> oracle_print_and_cli_def_keys() {
    rust::Vec<rust::String> out;
    const Slic3r::DynamicPrintAndCLIConfig config;
    const Slic3r::ConfigDef *def = config.def();
    if (def == nullptr) {
        return out;
    }
    for (const auto &kvp : def->options) {
        out.push_back(rust::String(kvp.first));
    }
    return out;
}

OracleLegacyResult oracle_print_and_cli_handle_legacy(rust::Str opt_key, rust::Str value) {
    const Slic3r::DynamicPrintAndCLIConfig config;
    std::string k(opt_key);
    std::string v(value);
    // `handle_legacy` is a protected `ConfigBase` override, so it is reached through the public
    // `set_deserialize_nothrow` path the comment above its declaration names... except that
    // would also write the option. `DynamicPrintAndCLIConfig::handle_legacy` is declared
    // `public` in the vendored class (`PrintConfig.hpp:2525`, under the `public:` section), so
    // it is called directly.
    config.handle_legacy(k, v);
    OracleLegacyResult out;
    out.opt_key = rust::String(k);
    out.value = rust::String(v);
    return out;
}

int32_t oracle_is_xl_printer_of_print_defaults() {
    return Slic3r::is_XL_printer(Slic3r::PrintConfig::defaults()) ? 1 : 0;
}

rust::Vec<int64_t> oracle_bed_shape_of_sla_defaults() {
    rust::Vec<int64_t> out;
    for (const Slic3r::Point &p : Slic3r::get_bed_shape(Slic3r::SLAPrinterConfig::defaults())) {
        out.push_back(static_cast<int64_t>(p.x()));
        out.push_back(static_cast<int64_t>(p.y()));
    }
    return out;
}

rust::Vec<int32_t>
oracle_extruder_supported_nozzle_volume_types(const rust::Vec<rust::String> &keys_and_values,
                                              int32_t extruder_id) {
    rust::Vec<int32_t> out;
    bool ok = false;
    Slic3r::DynamicPrintConfig config = config_from(keys_and_values, ok);
    if (!ok) {
        out.push_back(-1);
        return out;
    }
    // A `std::set`, so this is its own ascending order.
    for (const Slic3r::NozzleVolumeType type :
         Slic3r::get_extruder_supported_nozzle_volume_types(config,
                                                            static_cast<int>(extruder_id))) {
        out.push_back(static_cast<int32_t>(type));
    }
    return out;
}

rust::Vec<rust::String>
oracle_update_static_print_config_from_dynamic(rust::Str base,
                                               const rust::Vec<rust::String> &target_kv,
                                               const rust::Vec<rust::String> &dest_kv,
                                               const rust::Vec<int32_t> &variant_index,
                                               const rust::Vec<rust::String> &key_set1,
                                               int32_t stride) {
    bool ok_a = false, ok_b = false;
    Slic3r::DynamicPrintConfig target = config_from_base(base, target_kv, ok_a);
    Slic3r::DynamicPrintConfig dest = config_from(dest_kv, ok_b);
    if (!ok_a || !ok_b) {
        rust::Vec<rust::String> out;
        out.push_back(rust::String("<refused>"));
        return out;
    }
    std::vector<int> indices;
    for (int32_t i : variant_index) {
        indices.push_back(static_cast<int>(i));
    }
    std::set<std::string> keys;
    for (const rust::String &k : key_set1) {
        keys.insert(std::string(k));
    }
    Slic3r::update_static_print_config_from_dynamic(target, dest, indices, keys,
                                                    static_cast<int>(stride));
    return serialize_config(target);
}

rust::String oracle_set_variant_override(rust::Str target_value, rust::Str source_value,
                                         const rust::Vec<int32_t> &variant_index,
                                         int32_t stride) {
    Slic3r::ConfigOptionFloats target, source;
    // `ConfigOptionFloatsTempl<false>::deserialize` **throws** `ConfigurationError`
    // ("Deserializing nil into a non-nullable object") for a `nil` element rather than
    // returning false, and an exception escaping a `cxx` bridge function declared to return a
    // value aborts the whole test binary. Reported as a refusal instead -- observed: the first
    // run of this comparison died with `SIGABRT` on a `nil` probe.
    try {
        if (!target.deserialize(std::string(target_value)) ||
            !source.deserialize(std::string(source_value))) {
            return rust::String("<refused>");
        }
    } catch (const std::exception &) {
        return rust::String("<refused>");
    }
    std::vector<int> indices;
    for (int32_t i : variant_index) {
        indices.push_back(static_cast<int>(i));
    }
    Slic3r::set_variant_override(target, source, indices, static_cast<int>(stride));
    return rust::String(target.serialize());
}

static std::set<std::string> to_set(const rust::Vec<rust::String> &values) {
    std::set<std::string> out;
    for (const rust::String &v : values) {
        out.insert(std::string(v));
    }
    return out;
}

rust::Vec<rust::String>
oracle_update_diff_values_to_child_config(rust::Str base, const rust::Vec<rust::String> &self_kv,
                                          const rust::Vec<rust::String> &new_kv,
                                          rust::Str extruder_id_name,
                                          rust::Str extruder_variant_name,
                                          const rust::Vec<rust::String> &key_set1,
                                          const rust::Vec<rust::String> &key_set2) {
    bool ok_a = false, ok_b = false;
    Slic3r::DynamicPrintConfig self = config_from_base(base, self_kv, ok_a);
    Slic3r::DynamicPrintConfig other = config_from(new_kv, ok_b);
    if (!ok_a || !ok_b) {
        rust::Vec<rust::String> out;
        out.push_back(rust::String("<refused>"));
        return out;
    }
    std::set<std::string> set1 = to_set(key_set1), set2 = to_set(key_set2);
    try {
        self.update_diff_values_to_child_config(other, std::string(extruder_id_name),
                                                std::string(extruder_variant_name), set1, set2);
    } catch (const std::exception &) {
        rust::Vec<rust::String> out;
        out.push_back(rust::String("<threw>"));
        return out;
    }
    return serialize_config(self);
}

rust::Vec<rust::String>
oracle_update_non_diff_values_to_base_config(rust::Str base,
                                             const rust::Vec<rust::String> &self_kv,
                                             const rust::Vec<rust::String> &new_kv,
                                             const rust::Vec<rust::String> &keys,
                                             const rust::Vec<rust::String> &different_keys,
                                             rust::Str extruder_id_name,
                                             rust::Str extruder_variant_name,
                                             const rust::Vec<rust::String> &key_set1,
                                             const rust::Vec<rust::String> &key_set2) {
    bool ok_a = false, ok_b = false;
    Slic3r::DynamicPrintConfig self = config_from_base(base, self_kv, ok_a);
    Slic3r::DynamicPrintConfig other = config_from(new_kv, ok_b);
    if (!ok_a || !ok_b) {
        rust::Vec<rust::String> out;
        out.push_back(rust::String("<refused>"));
        return out;
    }
    std::vector<std::string> key_vec;
    for (const rust::String &k : keys) {
        key_vec.push_back(std::string(k));
    }
    std::set<std::string> diff = to_set(different_keys), set1 = to_set(key_set1),
                          set2 = to_set(key_set2);
    try {
        self.update_non_diff_values_to_base_config(other, key_vec, diff,
                                                   std::string(extruder_id_name),
                                                   std::string(extruder_variant_name), set1, set2);
    } catch (const std::exception &) {
        rust::Vec<rust::String> out;
        out.push_back(rust::String("<threw>"));
        return out;
    }
    return serialize_config(self);
}

rust::Vec<rust::String>
oracle_update_values_from_single_to_multi(rust::Str base, const rust::Vec<rust::String> &self_kv,
                                          const rust::Vec<rust::String> &multi_kv,
                                          const rust::Vec<rust::String> &key_set,
                                          rust::Str id_name, rust::Str variant_name) {
    bool ok_a = false, ok_b = false;
    Slic3r::DynamicPrintConfig self = config_from_base(base, self_kv, ok_a);
    Slic3r::DynamicPrintConfig multi = config_from(multi_kv, ok_b);
    rust::Vec<rust::String> out;
    if (!ok_a || !ok_b) {
        out.push_back(rust::String("<refused>"));
        return out;
    }
    std::set<std::string> keys = to_set(key_set);
    int result = 0;
    try {
        result = self.update_values_from_single_to_multi(multi, keys, std::string(id_name),
                                                        std::string(variant_name));
    } catch (const std::exception &) {
        out.push_back(rust::String("<threw>"));
        return out;
    }
    out.push_back(rust::String(std::to_string(result)));
    for (rust::String &entry : serialize_config(self)) {
        out.push_back(std::move(entry));
    }
    return out;
}

rust::Vec<rust::String> oracle_compute_filament_override_value(
    const rust::Vec<rust::String> &full_kv, const rust::Vec<rust::String> &old_kv,
    const rust::Vec<int32_t> &f_map_indices, rust::Str opt_key) {
    rust::Vec<rust::String> out;
    bool ok_a = false, ok_b = false;
    // Both configs start from a full print config, as `Print.cpp:4790-4792` has them: the
    // "machine" option comes off `m_config` and the two new ones off `m_full_print_config`.
    Slic3r::DynamicPrintConfig full = config_from_base(rust::Str("full"), full_kv, ok_a);
    Slic3r::DynamicPrintConfig old = config_from_base(rust::Str("full"), old_kv, ok_b);
    if (!ok_a || !ok_b) {
        out.push_back(rust::String("<refused>"));
        return out;
    }

    const std::string key(opt_key);
    Slic3r::t_config_option_keys diff_keys;
    Slic3r::DynamicPrintConfig filament_overrides;
    std::vector<int> indices(f_map_indices.begin(), f_map_indices.end());

    try {
        const Slic3r::ConfigOption *opt_new_filament = full.option("filament_" + key);
        const Slic3r::ConfigOption *opt_new_machine = full.option(key);
        const Slic3r::ConfigOption *opt_old_machine = old.option(key);
        // `Print.cpp:4794` guards only on `opt_new_filament`; the other two are guarded here
        // as well, because this shim can be handed a key the caller's loop never would and an
        // unguarded null would reach `opt_new_machine->clone()` at `:12380`.
        if (opt_new_filament == nullptr || opt_new_machine == nullptr ||
            opt_old_machine == nullptr) {
            out.push_back(rust::String("<absent>"));
            return out;
        }
        Slic3r::compute_filament_override_value(key, opt_old_machine, opt_new_machine,
                                                opt_new_filament, full, diff_keys,
                                                filament_overrides, indices);
    } catch (const std::exception &) {
        out.push_back(rust::String("<threw>"));
        return out;
    }

    out.push_back(rust::String("<ok>"));
    std::string joined;
    for (const std::string &k : diff_keys) {
        if (!joined.empty())
            joined += ",";
        joined += k;
    }
    out.push_back(rust::String(joined));
    for (rust::String &entry : serialize_config(filament_overrides)) {
        out.push_back(std::move(entry));
    }
    return out;
}

} // namespace print_config
} // namespace proof_units
} // namespace slic3r_ffi

