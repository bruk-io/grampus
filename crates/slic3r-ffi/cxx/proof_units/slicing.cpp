// Proof shims for the `Slicing.cpp`/`Slicing.hpp` unit (R19, proof paths 1 and 2, task
// `61-foundation-line-slicing`), compiled onto `crates/slic3r-ffi`'s shared `cxx_build::Build`
// by this crate's `build.rs` -- the one sanctioned route a ported unit reaches the oracle by
// (`docs/PORTING.md`, "Decisions recorded on 2026-11-25").
//
// `path1_generate_object_layers`/`path1_check_object_layers_fixed` call
// `Slic3r::generate_object_layers`/`Slic3r::check_object_layers_fixed` (`Slicing.cpp:819-914`),
// genuinely linked `Slicing.cpp` code, proof path 1. Each constructs a real
// `Slic3r::SlicingParameters` from plain scalars -- the only shape a `cxx` bridge may carry
// (`docs/PORTING.md`, "Decisions recorded on 2026-11-25", decision 1) -- setting only the
// fields the function under test actually reads, the rest left at their in-class default.
//
// `path2_slicing_params_header_only` drives `Slicing.hpp`'s own header-only, inline `has_raft()`,
// `first_object_layer_height_fixed()`, `object_print_z_height()` and
// `object_print_z_uncompensated_height()` (`Slicing.hpp:43-54`) directly through a real
// `Slic3r::SlicingParameters` -- `Slicing.hpp`'s own row is `proof path: 2` with no stage-dump
// field that is any one of these calls' own output in isolation, so this proof reaches the
// oracle through the per-unit bridge instead, the same `proof path: 2`-with-no-dump-coverage
// shape `docs/PORTING.md`'s "Decisions recorded on 2026-11-26", decision 2, accepts.
//
// `path1_smooth_height_profile` calls `Slic3r::smooth_height_profile` (`Slicing.cpp:419-526`),
// genuinely linked `Slicing.cpp` code, proof path 1 -- added on review: the first version of
// this shim file proved `generate_object_layers`/`check_object_layers_fixed` but left
// `smooth_height_profile` (and the `gauss_kernel`/`gauss_blur` lambdas it alone calls) with no
// oracle comparison at all.
#include "slicing.h"

#include "Slicing.hpp"

namespace slic3r_ffi {
namespace proof_units {
namespace slicing {

rust::Vec<double> path1_generate_object_layers(
    int64_t base_raft_layers, int64_t interface_raft_layers, bool first_object_layer_bridging,
    double first_object_layer_height, double min_layer_height, double max_layer_height,
    double object_print_z_min, double object_print_z_max, double object_shrinkage_compensation_z,
    rust::Slice<const double> profile, bool is_precise_z_height) {
    Slic3r::SlicingParameters params;
    params.base_raft_layers = static_cast<size_t>(base_raft_layers);
    params.interface_raft_layers = static_cast<size_t>(interface_raft_layers);
    params.first_object_layer_bridging = first_object_layer_bridging;
    params.first_object_layer_height = first_object_layer_height;
    params.min_layer_height = min_layer_height;
    params.max_layer_height = max_layer_height;
    params.object_print_z_min = object_print_z_min;
    params.object_print_z_max = object_print_z_max;
    params.object_shrinkage_compensation_z = object_shrinkage_compensation_z;

    std::vector<coordf_t> layer_height_profile(profile.begin(), profile.end());
    std::vector<coordf_t> out =
        Slic3r::generate_object_layers(params, layer_height_profile, is_precise_z_height);

    rust::Vec<double> result;
    result.reserve(out.size());
    for (coordf_t v : out) {
        result.push_back(v);
    }
    return result;
}

bool path1_check_object_layers_fixed(double first_object_layer_height, double layer_height,
                                      rust::Slice<const double> profile) {
    Slic3r::SlicingParameters params;
    params.first_object_layer_height = first_object_layer_height;
    params.layer_height = layer_height;
    std::vector<coordf_t> layer_height_profile(profile.begin(), profile.end());
    return Slic3r::check_object_layers_fixed(params, layer_height_profile);
}

rust::Vec<double> path2_slicing_params_header_only(int64_t base_raft_layers,
                                                    int64_t interface_raft_layers,
                                                    bool first_object_layer_bridging,
                                                    double object_print_z_min,
                                                    double object_print_z_max,
                                                    double object_print_z_uncompensated_max) {
    Slic3r::SlicingParameters params;
    params.base_raft_layers = static_cast<size_t>(base_raft_layers);
    params.interface_raft_layers = static_cast<size_t>(interface_raft_layers);
    params.first_object_layer_bridging = first_object_layer_bridging;
    params.object_print_z_min = object_print_z_min;
    params.object_print_z_max = object_print_z_max;
    params.object_print_z_uncompensated_max = object_print_z_uncompensated_max;

    rust::Vec<double> out;
    out.push_back(params.has_raft() ? 1.0 : 0.0);
    out.push_back(params.first_object_layer_height_fixed() ? 1.0 : 0.0);
    out.push_back(params.object_print_z_height());
    out.push_back(params.object_print_z_uncompensated_height());
    return out;
}

rust::Vec<double> path1_smooth_height_profile(rust::Slice<const double> profile,
                                               double first_object_layer_height,
                                               double layer_height, double min_layer_height,
                                               double max_layer_height, uint32_t radius,
                                               bool keep_min) {
    Slic3r::SlicingParameters params;
    params.first_object_layer_height = first_object_layer_height;
    params.layer_height = layer_height;
    params.min_layer_height = min_layer_height;
    params.max_layer_height = max_layer_height;

    std::vector<double> layer_height_profile(profile.begin(), profile.end());
    Slic3r::HeightProfileSmoothingParams smoothing_params(radius, keep_min);
    std::vector<double> out =
        Slic3r::smooth_height_profile(layer_height_profile, params, smoothing_params);

    rust::Vec<double> result;
    result.reserve(out.size());
    for (double v : out) {
        result.push_back(v);
    }
    return result;
}

} // namespace slicing
} // namespace proof_units
} // namespace slic3r_ffi
