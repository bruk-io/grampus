#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart at <crate-name>/src/proof_units/slicing.rs.h on the include path it
// sets up, and `cxx/proof_units/slicing.cpp` is the one translation unit that defines the
// functions below.
#include "slic3r-ffi/src/proof_units/slicing.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace slicing {

rust::Vec<double> path1_generate_object_layers(
    int64_t base_raft_layers, int64_t interface_raft_layers, bool first_object_layer_bridging,
    double first_object_layer_height, double min_layer_height, double max_layer_height,
    double object_print_z_min, double object_print_z_max, double object_shrinkage_compensation_z,
    rust::Slice<const double> profile, bool is_precise_z_height);

bool path1_check_object_layers_fixed(double first_object_layer_height, double layer_height,
                                      rust::Slice<const double> profile);

rust::Vec<double> path2_slicing_params_header_only(int64_t base_raft_layers,
                                                    int64_t interface_raft_layers,
                                                    bool first_object_layer_bridging,
                                                    double object_print_z_min,
                                                    double object_print_z_max,
                                                    double object_print_z_uncompensated_max);

rust::Vec<double> path1_smooth_height_profile(rust::Slice<const double> profile,
                                               double first_object_layer_height,
                                               double layer_height, double min_layer_height,
                                               double max_layer_height, uint32_t radius,
                                               bool keep_min);

} // namespace slicing
} // namespace proof_units
} // namespace slic3r_ffi
