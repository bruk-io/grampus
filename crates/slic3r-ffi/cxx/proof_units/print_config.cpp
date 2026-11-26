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

OracleOptionInfo oracle_option_info(rust::Str key) {
    OracleOptionInfo info{};
    info.found = false;
    info.option_type = 0;
    info.nullable = false;
    info.has_default = false;

    const Slic3r::ConfigOptionDef *def = Slic3r::print_config_def.get(std::string(key));
    if (def == nullptr) {
        return info;
    }

    info.found = true;
    info.option_type = static_cast<int32_t>(def->type);
    info.nullable = def->nullable;
    info.cli = rust::String(def->cli);

    if (def->default_value) {
        info.has_default = true;
        info.default_serialized = rust::String(def->default_value->serialize());
    }

    for (const auto &value : def->enum_values) {
        info.enum_values.push_back(rust::String(value));
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

OracleLegacyResult oracle_handle_legacy(rust::Str opt_key, rust::Str value) {
    std::string k(opt_key);
    std::string v(value);
    Slic3r::PrintConfigDef::handle_legacy(k, v);
    OracleLegacyResult out;
    out.opt_key = rust::String(k);
    out.value = rust::String(v);
    return out;
}

} // namespace print_config
} // namespace proof_units
} // namespace slic3r_ffi
