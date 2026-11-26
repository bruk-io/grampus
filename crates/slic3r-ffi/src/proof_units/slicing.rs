//! Proof paths 1 and 2 (R19, task `61-foundation-line-slicing`) for the
//! `Slicing.cpp`/`Slicing.hpp` unit.
//!
//! `path1_generate_object_layers`/`path1_check_object_layers_fixed` call the externally linked
//! `Slic3r::generate_object_layers`/`Slic3r::check_object_layers_fixed` (`Slicing.cpp:819-914`);
//! `path2_slicing_params_header_only` drives `Slicing.hpp`'s own header-only accessors directly
//! through a real vendored `SlicingParameters`. See `cxx/proof_units/slicing.cpp` for both.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::slicing")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/slicing.h");

        /// `Slic3r::generate_object_layers(params, profile, is_precise_z_height)`
        /// (`Slicing.cpp:819-880`), with `params` constructed from the given scalar fields.
        #[allow(clippy::too_many_arguments)]
        fn path1_generate_object_layers(
            base_raft_layers: i64,
            interface_raft_layers: i64,
            first_object_layer_bridging: bool,
            first_object_layer_height: f64,
            min_layer_height: f64,
            max_layer_height: f64,
            object_print_z_min: f64,
            object_print_z_max: f64,
            object_shrinkage_compensation_z: f64,
            profile: &[f64],
            is_precise_z_height: bool,
        ) -> Vec<f64>;

        /// `Slic3r::check_object_layers_fixed(params, profile)` (`Slicing.cpp:882-914`), with
        /// `params` constructed from the given scalar fields.
        fn path1_check_object_layers_fixed(
            first_object_layer_height: f64,
            layer_height: f64,
            profile: &[f64],
        ) -> bool;

        /// `params.has_raft()`/`.first_object_layer_height_fixed()`/`.object_print_z_height()`/
        /// `.object_print_z_uncompensated_height()` (`Slicing.hpp:43-54`), returning `[has_raft,
        /// first_object_layer_height_fixed, object_print_z_height,
        /// object_print_z_uncompensated_height]` (the two booleans as `0.0`/`1.0`).
        fn path2_slicing_params_header_only(
            base_raft_layers: i64,
            interface_raft_layers: i64,
            first_object_layer_bridging: bool,
            object_print_z_min: f64,
            object_print_z_max: f64,
            object_print_z_uncompensated_max: f64,
        ) -> Vec<f64>;

        /// `Slic3r::smooth_height_profile(profile, params, smoothing_params)`
        /// (`Slicing.cpp:419-526`), with `params`/`smoothing_params` constructed from the given
        /// scalar fields.
        #[allow(clippy::too_many_arguments)]
        fn path1_smooth_height_profile(
            profile: &[f64],
            first_object_layer_height: f64,
            layer_height: f64,
            min_layer_height: f64,
            max_layer_height: f64,
            radius: u32,
            keep_min: bool,
        ) -> Vec<f64>;

        /// `Slic3r::generate_layer_height_texture(params, layers, data, rows, cols,
        /// level_of_detail_2nd_level)` (`Slicing.cpp:916-1026`), with `params` constructed from
        /// the given scalar fields and `data` a real byte buffer this shim allocates. Returns
        /// `[ncells, byte0, byte1, ...]` -- added on review: this function had no proof of any
        /// kind before.
        #[allow(clippy::too_many_arguments)]
        fn path1_generate_layer_height_texture(
            min_layer_height: f64,
            max_layer_height: f64,
            layer_height: f64,
            object_print_z_min: f64,
            object_print_z_max: f64,
            layers: &[f64],
            rows: i32,
            cols: i32,
            level_of_detail_2nd_level: bool,
        ) -> Vec<i64>;
    }
}

pub use ffi::{
    path1_check_object_layers_fixed, path1_generate_layer_height_texture,
    path1_generate_object_layers, path1_smooth_height_profile, path2_slicing_params_header_only,
};
