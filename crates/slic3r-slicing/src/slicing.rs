//! Port of `src/libslic3r/Slicing.hpp`/`Slicing.cpp` (task `61-foundation-line-slicing`, plan
//! step 20, `port/foundation.tsv` row `foundation-10`): the slicing parameters struct
//! (`SlicingParameters`) and the pure layer-height-profile arithmetic that operates on it and on
//! a plain `Vec<coordf_t>` profile -- no `Model`/`PrintConfig` type in sight.
//!
//! **Scope this task hand-designs, and the line it draws.** `Slicing.hpp`'s struct,
//! `HeightProfileSmoothingParams`, `LayerHeightEditActionType` and `equal_layering` are
//! hand-designed here (`template-heavy: yes`, the row this task owns). `Slicing.cpp`'s own
//! functions split cleanly into two groups by whether they take a `PrintConfig`/
//! `PrintObjectConfig`/`DynamicPrintConfig`/`ModelObject` parameter:
//!
//! - **Ported**: `smooth_height_profile`, `generate_object_layers` (and its own file-local
//!   helper `adjust_layer_series_to_align_object_height`), `check_object_layers_fixed` and
//!   `generate_layer_height_texture` take only `SlicingParameters`, a plain `Vec<coordf_t>`
//!   profile and (the last) a raw pixel buffer -- no fan-out type needed, so each is translated
//!   in full below.
//! - **Deferred -- needs a type from a crate this task does not own, and this task's own rule
//!   forbids inventing one:** `min_layer_height_from_nozzle`/`max_layer_height_from_nozzle`
//!   (`Slicing.cpp:38-52`, the two `inline`, non-`Slicing::`-namespaced overloads: need
//!   `PrintConfig`, which `slic3r-config` does not define -- that crate ports `Config.hpp`'s
//!   generic option machinery, not `PrintConfig.hpp`'s own strongly-typed config classes),
//!   `SlicingParameters::create_from_config` (`:71-240`, needs `PrintConfig`/
//!   `PrintObjectConfig`), `layer_height_profile_from_ranges` (`:244-315`, needs `ModelConfig`
//!   through `t_layer_config_ranges`), `layer_height_profile_adaptive` (`:319-417`, needs
//!   `ModelObject` and `SlicingAdaptive`, both fan-out) and `adjust_layer_height_profile`
//!   (`:528-721`, needs `ModelObject`). Each is named at its own `// TODO(port):` site below.
//!
//! **`Slicing::min_layer_height_from_nozzle`/`max_layer_height_from_nozzle`
//! (`Slicing.cpp:55-69`) are the one pair of the four nozzle-limit overloads this task *can*
//! port**: their own parameter is `DynamicPrintConfig`, and `opt_float(key, idx)`
//! (`Config.cpp:2097-2106`) -- the only method either one calls -- is inherited unchanged from
//! `ConfigBase`/`DynamicConfig`, which `slic3r-config` *does* define
//! ([`slic3r_config::config::DynamicConfig::opt_float_at`]). A real
//! `Slic3r::DynamicConfig` (not even `DynamicPrintConfig` itself) populated with the same two
//! keys is behaviourally identical for this one call, so the oracle shim needs no
//! `PrintConfig.hpp` include at all.
//!
//! **`t_layer_height_range` (`:158`) is ported** as a plain `(f64, f64)` tuple -- no `Model`
//! dependency. **`t_layer_config_ranges` (`:159`) is not**: it is `std::map<t_layer_height_range,
//! ModelConfig>`, and `ModelConfig` is `slic3r-model`'s, not this crate's dependency.

use slic3r_libslic3r::libslic3r::{is_approx_default, lerp, EPSILON};

// -------------------------------------------------------------------------------------------
// SlicingParameters (Slicing.hpp:30-124)
// -------------------------------------------------------------------------------------------

/// `struct SlicingParameters` (`:30-123`). Every field defaults to `0`/`false` except
/// `belt_floor_from_axis`, whose vendored in-class initialiser is `{1}` (`:121`) -- the one
/// field [`Default`] cannot derive, so this struct implements it by hand rather than mixing a
/// derive with a field the derive would get wrong.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SlicingParameters {
    pub valid: bool,
    pub base_raft_layers: usize,
    pub interface_raft_layers: usize,
    pub base_raft_layer_height: f64,
    pub interface_raft_layer_height: f64,
    pub contact_raft_layer_height: f64,
    pub layer_height: f64,
    pub min_layer_height: f64,
    pub max_layer_height: f64,
    pub max_suport_layer_height: f64,
    pub first_print_layer_height: f64,
    pub first_object_layer_height: f64,
    pub first_object_layer_bridging: bool,
    pub zero_gap_interface_top: bool,
    pub zero_gap_interface_bottom: bool,
    pub zero_gap_interface_raft: bool,
    pub gap_raft_object: f64,
    pub gap_object_support: f64,
    pub gap_support_object: f64,
    pub raft_base_top_z: f64,
    pub raft_interface_top_z: f64,
    pub raft_contact_top_z: f64,
    pub object_print_z_min: f64,
    pub object_print_z_max: f64,
    pub object_print_z_uncompensated_max: f64,
    pub object_shrinkage_compensation_z: f64,
    pub belt_floor_shear_factor: f64,
    /// `int belt_floor_from_axis { 1 };` (`:121`): which axis the shear is from (`0 = X, 1 =
    /// Y`).
    pub belt_floor_from_axis: i32,
    pub belt_floor_z_shift: f64,
}

impl Default for SlicingParameters {
    // PORT SRC: src/libslic3r/Slicing.hpp:121
    fn default() -> Self {
        SlicingParameters {
            valid: false,
            base_raft_layers: 0,
            interface_raft_layers: 0,
            base_raft_layer_height: 0f64,
            interface_raft_layer_height: 0f64,
            contact_raft_layer_height: 0f64,
            layer_height: 0f64,
            min_layer_height: 0f64,
            max_layer_height: 0f64,
            max_suport_layer_height: 0f64,
            first_print_layer_height: 0f64,
            first_object_layer_height: 0f64,
            first_object_layer_bridging: false,
            zero_gap_interface_top: false,
            zero_gap_interface_bottom: false,
            zero_gap_interface_raft: false,
            gap_raft_object: 0f64,
            gap_object_support: 0f64,
            gap_support_object: 0f64,
            raft_base_top_z: 0f64,
            raft_interface_top_z: 0f64,
            raft_contact_top_z: 0f64,
            object_print_z_min: 0f64,
            object_print_z_max: 0f64,
            object_print_z_uncompensated_max: 0f64,
            object_shrinkage_compensation_z: 0f64,
            belt_floor_shear_factor: 0f64,
            belt_floor_from_axis: 1,
            belt_floor_z_shift: 0f64,
        }
    }
}

impl SlicingParameters {
    /// `bool has_raft() const { return raft_layers() > 0; }` (`:43`).
    pub fn has_raft(&self) -> bool {
        self.raft_layers() > 0
    }

    /// `size_t raft_layers() const { return base_raft_layers + interface_raft_layers; }`
    /// (`:44`).
    pub fn raft_layers(&self) -> usize {
        self.base_raft_layers + self.interface_raft_layers
    }

    /// `bool first_object_layer_height_fixed() const` (`:47`).
    pub fn first_object_layer_height_fixed(&self) -> bool {
        !self.has_raft() || self.first_object_layer_bridging
    }

    /// `coordf_t object_print_z_height() const` (`:50`).
    pub fn object_print_z_height(&self) -> f64 {
        self.object_print_z_max - self.object_print_z_min
    }

    /// `coordf_t object_print_z_uncompensated_height() const` (`:54`).
    pub fn object_print_z_uncompensated_height(&self) -> f64 {
        self.object_print_z_uncompensated_max - self.object_print_z_min
    }
}

// TODO(port): src/libslic3r/Slicing.cpp:71-240 --
// `SlicingParameters::create_from_config`, needs `PrintConfig`/`PrintObjectConfig` (not a
// dependency of this crate; see the module doc).

/// `inline bool equal_layering(const SlicingParameters &sp1, const SlicingParameters &sp2)`
/// (`Slicing.hpp:127-156`). The `#if 0`'d block (`:144-151`, `zero_gap_interface_*`/`gap_*`) is
/// not ported (`docs/PORTING.md`'s don't-translate list: upstream code that is commented out).
pub fn equal_layering(sp1: &SlicingParameters, sp2: &SlicingParameters) -> bool {
    debug_assert!(sp1.valid);
    debug_assert!(sp2.valid);
    sp1.base_raft_layers == sp2.base_raft_layers
        && sp1.interface_raft_layers == sp2.interface_raft_layers
        && sp1.base_raft_layer_height == sp2.base_raft_layer_height
        && sp1.interface_raft_layer_height == sp2.interface_raft_layer_height
        && sp1.contact_raft_layer_height == sp2.contact_raft_layer_height
        && sp1.layer_height == sp2.layer_height
        && sp1.first_print_layer_height == sp2.first_print_layer_height
        && sp1.first_object_layer_height == sp2.first_object_layer_height
        && sp1.first_object_layer_bridging == sp2.first_object_layer_bridging
        && sp1.raft_base_top_z == sp2.raft_base_top_z
        && sp1.raft_interface_top_z == sp2.raft_interface_top_z
        && sp1.raft_contact_top_z == sp2.raft_contact_top_z
        && sp1.object_print_z_min == sp2.object_print_z_min
}

/// `typedef std::pair<coordf_t, coordf_t> t_layer_height_range;` (`:158`).
pub type TLayerHeightRange = (f64, f64);

// TODO(port): src/libslic3r/Slicing.hpp:159 -- `t_layer_config_ranges =
// std::map<t_layer_height_range, ModelConfig>`, needs `ModelConfig` (`slic3r-model`, not a
// dependency of this crate).
// TODO(port): src/libslic3r/Slicing.cpp:244-315 --
// `layer_height_profile_from_ranges`, needs `t_layer_config_ranges` above.
// TODO(port): src/libslic3r/Slicing.cpp:319-417 --
// `layer_height_profile_adaptive`, needs `ModelObject` and `SlicingAdaptive` (`slic3r-model`,
// `slic3r-slicingadaptive`, a fan-out module of a crate this crate depends on but whose module
// Phase A/B has not yet written).

/// `struct HeightProfileSmoothingParams` (`Slicing.hpp:169-176`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeightProfileSmoothingParams {
    pub radius: u32,
    pub keep_min: bool,
}

impl Default for HeightProfileSmoothingParams {
    /// `HeightProfileSmoothingParams() : radius(5), keep_min(false) {}` (`:174`).
    fn default() -> Self {
        HeightProfileSmoothingParams {
            // PORT SRC: src/libslic3r/Slicing.hpp:174
            radius: 5,
            keep_min: false,
        }
    }
}

impl HeightProfileSmoothingParams {
    /// `HeightProfileSmoothingParams(unsigned int radius, bool keep_min)` (`:175`).
    pub fn new(radius: u32, keep_min: bool) -> Self {
        HeightProfileSmoothingParams { radius, keep_min }
    }
}

/// `enum LayerHeightEditActionType : unsigned int` (`Slicing.hpp:182-187`); explicit
/// discriminants matching the vendored values, since `generate_layer_height_texture`'s own
/// palette-index arithmetic and any later FFI boundary read this enum by its numeric value, not
/// only by name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum LayerHeightEditActionType {
    Increase = 0,
    Decrease = 1,
    Reduce = 2,
    // PORT SRC: src/libslic3r/Slicing.hpp:186
    Smooth = 3,
}

// TODO(port): src/libslic3r/Slicing.cpp:528-721 --
// `adjust_layer_height_profile`, needs `ModelObject` (`slic3r-model`, not a dependency of this
// crate; see the module doc).

/// `bool adjust_layer_series_to_align_object_height(const SlicingParameters &slicing_params,
/// std::vector<coordf_t>& layer_series)` (`Slicing.cpp:723-817`). Not declared in `Slicing.hpp`
/// -- a file-local helper [`generate_object_layers`] alone calls, so `pub(crate)` rather than
/// `pub`, matching the idiom map's "a free function in an anonymous namespace" rule read against
/// this file's own linkage (external, but with no other in-scope caller).
// PORT SRC: src/libslic3r/Slicing.cpp:734
pub(crate) fn adjust_layer_series_to_align_object_height(
    slicing_params: &SlicingParameters,
    layer_series: &mut Vec<f64>,
) -> bool {
    if layer_series.is_empty() {
        return false;
    }
    let object_height = slicing_params.object_print_z_height();
    if is_approx_default(*layer_series.last().unwrap(), object_height) {
        return true;
    }

    let layer_size = layer_series.len();
    // PORT SRC: src/libslic3r/Slicing.cpp:734
    if layer_size < 12 {
        return false;
    }

    // PORT SRC: src/libslic3r/Slicing.cpp:738 (the `5` last-five-layers count)
    let mut last_5_layers_height = [0f64; 5];
    for (i, h) in last_5_layers_height.iter_mut().enumerate() {
        // PORT SRC: src/libslic3r/Slicing.cpp:739
        *h = layer_series[layer_size - 10 + 2 * i + 1] - layer_series[layer_size - 10 + 2 * i];
    }

    let mut gap = (layer_series[layer_size - 1] - object_height).abs();
    // PORT SRC: src/libslic3r/Slicing.cpp:743 (the `5` last-five-layers count)
    let mut can_adjust = [true; 5];
    let taller_than_object = layer_series[layer_size - 1] < object_height;

    let get_valid_size =
        // PORT SRC: src/libslic3r/Slicing.cpp:743 (the `5` last-five-layers count,
        // `can_adjust`'s own length)
        |can_adjust: &[bool; 5]| -> i32 { can_adjust.iter().filter(|b| **b).count() as i32 };

    loop {
        let valid_size = get_valid_size(&can_adjust);
        if valid_size == 0 {
            // 5 layers can not adjust z within valid layer height.
            return false;
        }

        let delta_gap = gap / valid_size as f64;
        let mut remain_gap = 0f64;
        for i in 0..5 {
            let l_height = last_5_layers_height[i];
            if taller_than_object {
                if can_adjust[i] && is_approx_default(l_height, slicing_params.max_layer_height) {
                    remain_gap += delta_gap;
                    can_adjust[i] = false;
                    continue;
                }
                if can_adjust[i] && l_height + delta_gap > slicing_params.max_layer_height {
                    remain_gap += l_height + delta_gap - slicing_params.max_layer_height;
                    last_5_layers_height[i] = slicing_params.max_layer_height;
                    can_adjust[i] = false;
                } else {
                    last_5_layers_height[i] += delta_gap;
                }
            } else {
                if can_adjust[i] && is_approx_default(l_height, slicing_params.min_layer_height) {
                    remain_gap += delta_gap;
                    can_adjust[i] = false;
                    continue;
                }
                if can_adjust[i] && l_height - delta_gap < slicing_params.min_layer_height {
                    remain_gap += slicing_params.min_layer_height + delta_gap - l_height;
                    last_5_layers_height[i] = slicing_params.min_layer_height;
                    can_adjust[i] = false;
                } else {
                    last_5_layers_height[i] -= delta_gap;
                }
            }
        }
        gap = remain_gap;

        if is_approx_default(gap, 0f64) {
            // Adjust succeeded.
            break;
        }
    }

    // PORT SRC: src/libslic3r/Slicing.cpp:738 (the `5` last-five-layers count)
    for i in 0..5 {
        if i > 0 {
            // PORT SRC: src/libslic3r/Slicing.cpp:811
            layer_series[layer_size - 10 + 2 * i] = layer_series[layer_size - 10 + 2 * i - 1];
        }
        // PORT SRC: src/libslic3r/Slicing.cpp:813
        layer_series[layer_size - 10 + 2 * i + 1] =
            // PORT SRC: src/libslic3r/Slicing.cpp:813
            layer_series[layer_size - 10 + 2 * i] + last_5_layers_height[i];
    }

    true
}

/// `std::vector<coordf_t> generate_object_layers(...)` (`Slicing.cpp:819-880`).
pub fn generate_object_layers(
    slicing_params: &SlicingParameters,
    layer_height_profile: &[f64],
    is_precise_z_height: bool,
) -> Vec<f64> {
    debug_assert!(!layer_height_profile.is_empty());

    let mut print_z = 0f64;

    let mut out: Vec<f64> = Vec::new();

    if slicing_params.first_object_layer_height_fixed() {
        out.push(0f64);
        print_z = slicing_params.first_object_layer_height;
        out.push(print_z);
    }

    let shrinkage_compensation_z = slicing_params.object_shrinkage_compensation_z;
    let mut idx_layer_height_profile: usize = 0;
    // PORT SRC: src/libslic3r/Slicing.cpp:842
    let mut slice_z = print_z + 0.5 * slicing_params.min_layer_height;
    while slice_z < slicing_params.object_print_z_height() {
        let mut height = slicing_params.min_layer_height;
        if idx_layer_height_profile < layer_height_profile.len() {
            let mut next = idx_layer_height_profile + 2;
            loop {
                if next >= layer_height_profile.len()
                    || slice_z < layer_height_profile[next] * shrinkage_compensation_z
                {
                    break;
                }
                idx_layer_height_profile = next;
                next += 2;
            }
            let z1 = layer_height_profile[idx_layer_height_profile] * shrinkage_compensation_z;
            let h1 = layer_height_profile[idx_layer_height_profile + 1];
            height = h1;
            if next < layer_height_profile.len() {
                let z2 = layer_height_profile[next] * shrinkage_compensation_z;
                let h2 = layer_height_profile[next + 1];
                height = lerp(h1, h2, (slice_z - z1) / (z2 - z1));
                debug_assert!(
                    height >= slicing_params.min_layer_height - EPSILON
                        && height <= slicing_params.max_layer_height + EPSILON
                );
            }
        }
        // PORT SRC: src/libslic3r/Slicing.cpp:866
        slice_z = print_z + 0.5 * height;
        if slice_z >= slicing_params.object_print_z_height() {
            break;
        }
        debug_assert!(height > slicing_params.min_layer_height - EPSILON);
        debug_assert!(height < slicing_params.max_layer_height + EPSILON);
        out.push(print_z);
        print_z += height;
        // PORT SRC: src/libslic3r/Slicing.cpp:873
        slice_z = print_z + 0.5 * slicing_params.min_layer_height;
        out.push(print_z);
    }

    if is_precise_z_height {
        adjust_layer_series_to_align_object_height(slicing_params, &mut out);
    }
    out
}

/// `bool check_object_layers_fixed(...)` (`Slicing.cpp:882-914`).
pub fn check_object_layers_fixed(
    slicing_params: &SlicingParameters,
    layer_height_profile: &[f64],
) -> bool {
    // PORT SRC: src/libslic3r/Slicing.cpp:887
    debug_assert!(layer_height_profile.len() >= 4);
    debug_assert!(layer_height_profile.len() % 2 == 0);
    debug_assert!(layer_height_profile[0] == 0f64);

    // PORT SRC: src/libslic3r/Slicing.cpp:891
    if layer_height_profile.len() != 4 && layer_height_profile.len() != 8 {
        return false;
    }

    // PORT SRC: src/libslic3r/Slicing.cpp:894
    let fixed_step1 = is_approx_default(layer_height_profile[1], layer_height_profile[3]);
    // PORT SRC: src/libslic3r/Slicing.cpp:895
    let fixed_step2 = layer_height_profile.len() == 4
        // PORT SRC: src/libslic3r/Slicing.cpp:896
        || (layer_height_profile[2] == layer_height_profile[4]
            // PORT SRC: src/libslic3r/Slicing.cpp:896
            && is_approx_default(layer_height_profile[5], layer_height_profile[7]));

    if !fixed_step1 || !fixed_step2 {
        return false;
    }

    // PORT SRC: src/libslic3r/Slicing.cpp:901
    if layer_height_profile[2] < 0.5 * slicing_params.first_object_layer_height + EPSILON
        || !is_approx_default(
            // PORT SRC: src/libslic3r/Slicing.cpp:902
            layer_height_profile[3],
            slicing_params.first_object_layer_height,
        )
    {
        return false;
    }

    let z_max = layer_height_profile[layer_height_profile.len() - 2];
    // PORT SRC: src/libslic3r/Slicing.cpp:906
    let z_2nd = slicing_params.first_object_layer_height + 0.5 * slicing_params.layer_height;
    if z_2nd > z_max {
        return true;
    }
    // PORT SRC: src/libslic3r/Slicing.cpp:909
    if z_2nd < layer_height_profile[layer_height_profile.len() - 4] + EPSILON
        || !is_approx_default(
            *layer_height_profile.last().unwrap(),
            slicing_params.layer_height,
        )
    {
        return false;
    }

    true
}

/// `std::vector<double> smooth_height_profile(...)` (`Slicing.cpp:419-526`). The vendored
/// `gauss_kernel`/`gauss_blur` lambdas become the private [`gauss_kernel`]/[`gauss_blur`]
/// functions below -- `has_steep_height_change` and its call site are commented out upstream
/// (`:500-518`) and are not ported (`docs/PORTING.md`'s don't-translate list).
pub fn smooth_height_profile(
    profile: &[f64],
    slicing_params: &SlicingParameters,
    smoothing_params: &HeightProfileSmoothingParams,
) -> Vec<f64> {
    let mut ret = profile.to_vec();
    let mut count = 0;
    // PORT SRC: src/libslic3r/Slicing.cpp:519
    while count < 6 {
        ret = gauss_blur(&ret, slicing_params, smoothing_params);
        count += 1;
    }
    ret
}

/// `gauss_kernel` (`Slicing.cpp:422-439`), the lambda inside `smooth_height_profile`'s own
/// `gauss_blur` lambda.
fn gauss_kernel(radius: u32) -> Vec<f64> {
    let size = 2 * radius + 1;
    let mut ret = Vec::with_capacity(size as usize);
    // Reworked from static inline int getGaussianKernelSize(float sigma) taken from
    // opencv-4.1.2\modules\features2d\src\kaze\AKAZEFeatures.cpp
    // PORT SRC: src/libslic3r/Slicing.cpp:428
    let sigma = 0.3 * (radius as f64 - 1f64) + 0.8;
    let two_sq_sigma = 2f64 * sigma * sigma;
    let inv_root_two_pi_sq_sigma = 1f64 / (std::f64::consts::PI * two_sq_sigma).sqrt();
    for i in 0..size {
        let x = i as f64 - radius as f64;
        ret.push(inv_root_two_pi_sq_sigma * (-x * x / two_sq_sigma).exp());
    }
    ret
}

/// `gauss_blur` (`Slicing.cpp:421-497`), the lambda `smooth_height_profile` defines and calls up
/// to six times.
fn gauss_blur(
    profile: &[f64],
    slicing_params: &SlicingParameters,
    smoothing_params: &HeightProfileSmoothingParams,
) -> Vec<f64> {
    // Skip first layer?
    let skip_count: usize = if slicing_params.first_object_layer_height_fixed() {
        // PORT SRC: src/libslic3r/Slicing.cpp:442
        4
    } else {
        0
    };

    // Not enough data to smooth.
    // PORT SRC: src/libslic3r/Slicing.cpp:445
    if (profile.len() as i64) - (skip_count as i64) < 6 {
        return profile.to_vec();
    }

    let radius = smoothing_params.radius.max(1);
    let kernel = gauss_kernel(radius);
    let two_radius = 2 * radius as i64;

    let size = profile.len();
    let mut ret: Vec<f64> = Vec::with_capacity(size);

    // Leave first layer untouched.
    ret.extend_from_slice(&profile[..skip_count]);

    // Smooth the rest of the profile by biasing a gaussian blur -- the bias moves the smoothed
    // profile closer to the min_layer_height.
    let delta_h = slicing_params.max_layer_height - slicing_params.min_layer_height;
    let inv_delta_h = if delta_h != 0f64 {
        1f64 / delta_h
    } else {
        1f64
    };

    let max_dz_band = radius as f64 * slicing_params.layer_height;
    let mut i = skip_count;
    while i < size {
        let zi = profile[i];
        let hi = profile[i + 1];
        ret.push(zi);
        ret.push(0f64);
        let height_idx = ret.len() - 1;
        let begin = ((i as i64) - two_radius).max(skip_count as i64);
        let end = ((i as i64) + two_radius).min(size as i64 - 2);
        let mut weight_total = 0f64;
        let mut height = 0f64;
        let mut j = begin;
        while j <= end {
            let kernel_id = (radius as i64 + (j - i as i64) / 2) as usize;
            let dz = (zi - profile[j as usize]).abs();
            if dz * slicing_params.layer_height <= max_dz_band {
                let dh = (slicing_params.max_layer_height - profile[j as usize + 1]).abs();
                let weight = kernel[kernel_id] * (dh * inv_delta_h).sqrt();
                height += weight * profile[j as usize + 1];
                weight_total += weight;
            }
            j += 2;
        }

        height = (if weight_total == 0f64 {
            hi
        } else {
            height / weight_total
        })
        .clamp(
            slicing_params.min_layer_height,
            slicing_params.max_layer_height,
        );
        if smoothing_params.keep_min {
            height = height.min(hi);
        }
        ret[height_idx] = height;
        i += 2;
    }

    ret
}

/// `int generate_layer_height_texture(...)` (`Slicing.cpp:916-1026`): the `void *data` buffer
/// becomes `data: &mut [u8]`, long enough to hold the main (0th LOD) texture
/// (`rows * cols * 4` bytes) and, when `level_of_detail_2nd_level` is set, the 2nd-LOD texture
/// immediately after it (`(rows/2) * (cols/2) * 4` bytes) -- exactly the single contiguous
/// allocation the vendored `unsigned char *data1 = reinterpret_cast<unsigned char*>(data) + rows
/// * cols * 4;` pointer arithmetic (`:935`) assumes.
// https://github.com/aschn/gnuplot-colorbrewer
pub fn generate_layer_height_texture(
    slicing_params: &SlicingParameters,
    layers: &[f64],
    data: &mut [u8],
    rows: i32,
    cols: i32,
    level_of_detail_2nd_level: bool,
) -> i32 {
    let palette_raw = [
        [0x01Ai64, 0x098, 0x050], // PORT SRC: src/libslic3r/Slicing.cpp:923
        [0x066, 0x0BD, 0x063],    // PORT SRC: src/libslic3r/Slicing.cpp:924
        [0x0A6, 0x0D9, 0x06A],    // PORT SRC: src/libslic3r/Slicing.cpp:925
        [0x0D9, 0x0F1, 0x0EB],    // PORT SRC: src/libslic3r/Slicing.cpp:926
        [0x0FE, 0x0E6, 0x0EB],    // PORT SRC: src/libslic3r/Slicing.cpp:927
        [0x0FD, 0x0AE, 0x061],    // PORT SRC: src/libslic3r/Slicing.cpp:928
        [0x0F4, 0x06D, 0x043],    // PORT SRC: src/libslic3r/Slicing.cpp:929
        [0x0D7, 0x030, 0x027],    // PORT SRC: src/libslic3r/Slicing.cpp:930
    ];

    // PORT SRC: src/libslic3r/Slicing.cpp:935
    let data1_offset = (rows * cols * 4) as usize;
    let ncells = ((cols - 1) * rows).min(
        // PORT SRC: src/libslic3r/Slicing.cpp:936
        (16. * (slicing_params.object_print_z_height() / slicing_params.min_layer_height)).ceil()
            as i32,
    );
    let ncells1 = ncells / 2;
    let cols1 = cols / 2;
    let z_to_cell = (ncells - 1) as f64 / slicing_params.object_print_z_height();
    let cell_to_z = slicing_params.object_print_z_height() / (ncells - 1) as f64;
    let z_to_cell1 = (ncells1 - 1) as f64 / slicing_params.object_print_z_height();
    let mut hscale = 2f64
        * (slicing_params.max_layer_height - slicing_params.layer_height)
            .max(slicing_params.layer_height - slicing_params.min_layer_height);
    if hscale == 0f64 {
        // All layers have the same height. Provide some height scale to avoid division by zero.
        hscale = slicing_params.layer_height;
    }

    let mut idx_layer = 0usize;
    while idx_layer < layers.len() {
        let lo = layers[idx_layer];
        let mut hi = layers[idx_layer + 1];
        // PORT SRC: src/libslic3r/Slicing.cpp:950
        let mid = 0.5 * (lo + hi);
        debug_assert!(mid <= slicing_params.object_print_z_height());
        let h = hi - lo;
        hi = hi.min(slicing_params.object_print_z_height());
        let cell_first = ((lo * z_to_cell).ceil() as i32).clamp(0, ncells - 1);
        let cell_last = ((hi * z_to_cell).floor() as i32).clamp(0, ncells - 1);
        for cell in cell_first..=cell_last {
            // PORT SRC: src/libslic3r/Slicing.cpp:957
            let idxf = (0.5 * hscale + (h - slicing_params.layer_height))
                * (palette_raw.len() - 1) as f64
                / hscale;
            let idx1 = (idxf.floor() as i32).clamp(0, palette_raw.len() as i32 - 1) as usize;
            let idx2 = (palette_raw.len() - 1).min(idx1 + 1);
            let t = idxf - idx1 as f64;
            let color1 = palette_raw[idx1];
            let color2 = palette_raw[idx2];
            let z = cell_to_z * cell as f64;
            debug_assert!(lo - EPSILON <= z && z <= hi + EPSILON);
            // Intensity profile to visualize the layers.
            // PORT SRC: src/libslic3r/Slicing.cpp:966
            let intensity = (std::f64::consts::PI * 0.7 * (mid - z) / h).cos();
            // Color mapping from layer height to RGB.
            let color = [
                intensity * lerp(color1[0] as f64, color2[0] as f64, t),
                intensity * lerp(color1[1] as f64, color2[1] as f64, t),
                intensity * lerp(color1[2] as f64, color2[2] as f64, t),
            ];
            let row = cell / (cols - 1);
            let col = cell - row * (cols - 1);
            debug_assert!(row >= 0 && row < rows);
            debug_assert!(col >= 0 && col < cols);
            // PORT SRC: src/libslic3r/Slicing.cpp:976
            let ptr = ((row * cols + col) * 4) as usize;
            // PORT SRC: src/libslic3r/Slicing.cpp:977
            data[ptr] = ((color[0] + 0.5).floor() as i32).clamp(0, 255) as u8;
            // PORT SRC: src/libslic3r/Slicing.cpp:978
            data[ptr + 1] = ((color[1] + 0.5).floor() as i32).clamp(0, 255) as u8;
            // PORT SRC: src/libslic3r/Slicing.cpp:979
            data[ptr + 2] = ((color[2] + 0.5).floor() as i32).clamp(0, 255) as u8;
            // PORT SRC: src/libslic3r/Slicing.cpp:980
            data[ptr + 3] = 255;
            if col == 0 && row > 0 {
                // Duplicate the first value in a row as a last value of the preceding row.
                // PORT SRC: src/libslic3r/Slicing.cpp:983
                data[ptr - 4] = data[ptr];
                // PORT SRC: src/libslic3r/Slicing.cpp:984
                data[ptr - 3] = data[ptr + 1];
                data[ptr - 2] = data[ptr + 2];
                // PORT SRC: src/libslic3r/Slicing.cpp:986
                data[ptr - 1] = data[ptr + 3];
            }
        }
        if level_of_detail_2nd_level {
            let cell_first1 = ((lo * z_to_cell1).ceil() as i32).clamp(0, ncells1 - 1);
            let cell_last1 = ((hi * z_to_cell1).floor() as i32).clamp(0, ncells1 - 1);
            for cell in cell_first1..=cell_last1 {
                // PORT SRC: src/libslic3r/Slicing.cpp:993
                let idxf = (0.5 * hscale + (h - slicing_params.layer_height))
                    * (palette_raw.len() - 1) as f64
                    / hscale;
                let idx1 = (idxf.floor() as i32).clamp(0, palette_raw.len() as i32 - 1) as usize;
                let idx2 = (palette_raw.len() - 1).min(idx1 + 1);
                let t = idxf - idx1 as f64;
                let color1 = palette_raw[idx1];
                let color2 = palette_raw[idx2];
                let color = [
                    lerp(color1[0] as f64, color2[0] as f64, t),
                    lerp(color1[1] as f64, color2[1] as f64, t),
                    lerp(color1[2] as f64, color2[2] as f64, t),
                ];
                let row = cell / (cols1 - 1);
                let col = cell - row * (cols1 - 1);
                debug_assert!(row >= 0 && row < rows / 2);
                debug_assert!(col >= 0 && col < cols / 2);
                // PORT SRC: src/libslic3r/Slicing.cpp:1008
                let ptr = data1_offset + ((row * cols1 + col) * 4) as usize;
                // PORT SRC: src/libslic3r/Slicing.cpp:1009
                data[ptr] = ((color[0] + 0.5).floor() as i32).clamp(0, 255) as u8;
                // PORT SRC: src/libslic3r/Slicing.cpp:1010
                data[ptr + 1] = ((color[1] + 0.5).floor() as i32).clamp(0, 255) as u8;
                // PORT SRC: src/libslic3r/Slicing.cpp:1011
                data[ptr + 2] = ((color[2] + 0.5).floor() as i32).clamp(0, 255) as u8;
                // PORT SRC: src/libslic3r/Slicing.cpp:1012
                data[ptr + 3] = 255;
                if col == 0 && row > 0 {
                    // PORT SRC: src/libslic3r/Slicing.cpp:1015
                    data[ptr - 4] = data[ptr];
                    // PORT SRC: src/libslic3r/Slicing.cpp:1016
                    data[ptr - 3] = data[ptr + 1];
                    data[ptr - 2] = data[ptr + 2];
                    // PORT SRC: src/libslic3r/Slicing.cpp:1018
                    data[ptr - 1] = data[ptr + 3];
                }
            }
        }
        idx_layer += 2;
    }

    // Returns number of cells of the 0th LOD level.
    ncells
}

// -------------------------------------------------------------------------------------------
// Slicing::min_layer_height_from_nozzle / max_layer_height_from_nozzle (Slicing.cpp:55-69)
// -------------------------------------------------------------------------------------------

/// Local, file-scoped `MIN_LAYER_HEIGHT`/`MIN_LAYER_HEIGHT_DEFAULT` (`Slicing.cpp:33-34`); not
/// `pub` -- no in-scope caller outside this file reaches either, exactly as their vendored
/// `static const` linkage keeps them private to `Slicing.cpp`.
// PORT SRC: src/libslic3r/Slicing.cpp:33
const MIN_LAYER_HEIGHT: f64 = 0.01;
// PORT SRC: src/libslic3r/Slicing.cpp:34
const MIN_LAYER_HEIGHT_DEFAULT: f64 = 0.07;

/// `Slicing::min_layer_height_from_nozzle(const DynamicPrintConfig&, int)` (`Slicing.cpp:55-59`).
/// See the module doc for why `DynamicConfig`, not `DynamicPrintConfig`, is this port's
/// parameter type.
pub fn min_layer_height_from_nozzle(
    print_config: &slic3r_config::config::DynamicConfig<'_>,
    idx_nozzle: i32,
) -> f64 {
    let min_layer_height = print_config
        .opt_float_at("min_layer_height", (idx_nozzle - 1) as usize)
        .unwrap_or(0f64);
    if min_layer_height == 0f64 {
        MIN_LAYER_HEIGHT_DEFAULT
    } else {
        MIN_LAYER_HEIGHT.max(min_layer_height)
    }
}

/// `Slicing::max_layer_height_from_nozzle(const DynamicPrintConfig&, int)` (`Slicing.cpp:63-69`).
pub fn max_layer_height_from_nozzle(
    print_config: &slic3r_config::config::DynamicConfig<'_>,
    idx_nozzle: i32,
) -> f64 {
    let min_layer_height = min_layer_height_from_nozzle(print_config, idx_nozzle);
    let max_layer_height = print_config
        .opt_float_at("max_layer_height", (idx_nozzle - 1) as usize)
        .unwrap_or(0f64);
    let nozzle_dmr = print_config
        .opt_float_at("nozzle_diameter", (idx_nozzle - 1) as usize)
        .unwrap_or(0f64);
    min_layer_height.max(if max_layer_height == 0f64 {
        // PORT SRC: src/libslic3r/Slicing.cpp:68
        0.75 * nozzle_dmr
    } else {
        max_layer_height
    })
}

// TODO(port): src/libslic3r/Slicing.cpp:38-52 -- the two `inline`,
// non-`Slicing::`-namespaced `min_layer_height_from_nozzle`/`max_layer_height_from_nozzle`
// overloads, need `PrintConfig` (not a dependency of this crate; see the module doc).

#[cfg(test)]
mod tests {
    use super::*;
    use slic3r_proof::tiers::{compare_unit, Oracle};

    fn params_with(layer_height: f64, first_object_layer_height: f64) -> SlicingParameters {
        SlicingParameters {
            layer_height,
            first_object_layer_height,
            min_layer_height: 0.1,
            max_layer_height: 0.3,
            object_print_z_min: 0f64,
            object_print_z_max: 10.0,
            object_print_z_uncompensated_max: 10.0,
            object_shrinkage_compensation_z: 1f64,
            valid: true,
            ..SlicingParameters::default()
        }
    }

    #[test]
    fn has_raft_is_false_with_no_raft_layers() {
        let params = SlicingParameters::default();
        assert!(!params.has_raft());
        assert!(params.first_object_layer_height_fixed());
    }

    #[test]
    fn has_raft_is_true_with_a_contact_layer() {
        let params = SlicingParameters {
            base_raft_layers: 1,
            ..SlicingParameters::default()
        };
        assert!(params.has_raft());
    }

    #[test]
    fn object_print_z_height_subtracts_min_from_max() {
        let params = SlicingParameters {
            object_print_z_min: 2f64,
            object_print_z_max: 7.5,
            ..SlicingParameters::default()
        };
        assert_eq!(params.object_print_z_height(), 5.5);
    }

    #[test]
    fn check_object_layers_fixed_accepts_a_four_entry_fixed_profile() {
        let params = params_with(0.2, 0.2);
        let profile = vec![0f64, 0.2, 10.0, 0.2];
        assert!(check_object_layers_fixed(&params, &profile));
    }

    #[test]
    fn smooth_height_profile_leaves_a_uniform_profile_unchanged() {
        let params = params_with(0.2, 0.2);
        let profile = vec![
            0f64, 0.2, 0.2, 0.2, 2f64, 0.2, 4.0, 0.2, 6.0, 0.2, 8.0, 0.2, 10.0, 0.2,
        ];
        let smoothing = HeightProfileSmoothingParams::default();
        let smoothed = smooth_height_profile(&profile, &params, &smoothing);
        for i in (5..smoothed.len()).step_by(2) {
            assert!(
                (smoothed[i] - 0.2).abs() < 1e-9,
                "a uniform-height profile stays uniform after smoothing, got {smoothed:?}"
            );
        }
    }

    /// `smooth_height_profile`/`gauss_kernel`/`gauss_blur` (`Slicing.cpp:419-497`), proven
    /// against the oracle through the per-unit bridge -- added on review: this unit's
    /// `// PROOF: 1` marker above proves `generate_object_layers`/`check_object_layers_fixed`,
    /// not this function, which had no oracle comparison at all until this test. Not itself a
    /// `// PROOF:` marker (the manifest row needs only one per proof-path cell, already
    /// satisfied), but a second, real differential comparison against the same proof path.
    #[test]
    fn smooth_height_profile_matches_the_oracle() {
        let params = params_with(0.2, 0.2);
        // A varying profile, so a real interpolation/weighting bug in `gauss_blur` would show.
        let profile = vec![
            0f64, 0.2, 0.2, 0.2, 2f64, 0.35, 4.0, 0.15, 6.0, 0.3, 8.0, 0.2, 10.0, 0.25,
        ];
        let smoothing = HeightProfileSmoothingParams::new(3, false);
        let actual = smooth_height_profile(&profile, &params, &smoothing);
        let oracle = slic3r_proof::proof_units::slicing::path1_smooth_height_profile(
            &profile,
            params.first_object_layer_height,
            params.layer_height,
            params.min_layer_height,
            params.max_layer_height,
            smoothing.radius,
            smoothing.keep_min,
        );
        compare_unit("Slicing.cpp", Oracle::Built, actual, oracle)
            .expect("smooth_height_profile must match the oracle exactly");
    }

    #[test]
    fn generate_layer_height_texture_paints_the_cell_at_the_first_layer_opaque() {
        let params = params_with(0.2, 0.2);
        let rows = 4;
        let cols = 8;
        let mut data = vec![0u8; (rows * cols * 4) as usize];
        // Layer heights near `params.layer_height` (0.2), the range this visualisation's own
        // palette-index arithmetic (`hscale`, scaled against `max_layer_height -
        // min_layer_height`) is meant to be read against -- unlike the z-spans
        // `check_object_layers_fixed`'s own fixture uses, which are deliberately far outside it.
        // `object_print_z_height()` is 10mm, so this one 0.2mm-high layer paints only the very
        // first cell of the grid -- a texture, like the real caller's, is always far wider in Z
        // than any one call's own `layers` argument covers.
        let layers = vec![0f64, 0.2];
        let ncells = generate_layer_height_texture(&params, &layers, &mut data, rows, cols, false);
        assert!(ncells > 0, "the cell count must be positive");
        let ptr = 3; // row 0, col 0, alpha byte.
        assert_eq!(data[ptr], 255, "the first cell must be painted opaque");
    }

    /// Proof path 1 (R19, `port/manifest.tsv`'s `Slicing.cpp` row): `generate_object_layers`
    /// and `check_object_layers_fixed` are externally linked, genuine `Slicing.cpp` code,
    /// called here through `crates/slic3r-ffi/cxx/proof_units/slicing.cpp`'s wrapper, reached
    /// through `slic3r-proof`'s re-export of the FFI crate's per-unit proof bridges
    /// (`docs/PORTING.md`, "Decisions recorded on 2026-11-25").
    // PROOF: 1
    #[test]
    fn slicing_generate_object_layers_matches_the_oracle() {
        let params = params_with(0.2, 0.2);
        // A varying (not flat) profile with `is_precise_z_height: true`, so this proof exercises
        // both `generate_object_layers`'s own `lerp` interpolation (a flat `h1 == h2` profile
        // would return the same height for every `t`, hiding an interpolation bug) and, through
        // it, `adjust_layer_series_to_align_object_height` (reached only when
        // `is_precise_z_height` is set) -- found on review: an earlier version of this test used
        // a flat profile and `false`, leaving both unproven.
        let profile = vec![0f64, 0.2, 5.0, 0.3, 10.0, 0.1];

        let actual = generate_object_layers(&params, &profile, true);
        let oracle = slic3r_proof::proof_units::slicing::path1_generate_object_layers(
            params.base_raft_layers as i64,
            params.interface_raft_layers as i64,
            params.first_object_layer_bridging,
            params.first_object_layer_height,
            params.min_layer_height,
            params.max_layer_height,
            params.object_print_z_min,
            params.object_print_z_max,
            params.object_shrinkage_compensation_z,
            &profile,
            true,
        );
        assert!(
            actual.len() >= 12,
            "the output must be long enough for adjust_layer_series_to_align_object_height's \
             own 12-entry floor to actually run, got {} entries",
            actual.len()
        );
        compare_unit("Slicing.cpp", Oracle::Built, actual, oracle)
            .expect("generate_object_layers (with adjust_layer_series_to_align_object_height) must match the oracle exactly");

        let fixed_profile = vec![0f64, 0.2, 10.0, 0.2];
        let actual_fixed = check_object_layers_fixed(&params, &fixed_profile);
        let oracle_fixed = slic3r_proof::proof_units::slicing::path1_check_object_layers_fixed(
            params.first_object_layer_height,
            params.layer_height,
            &fixed_profile,
        );
        assert_eq!(
            actual_fixed, oracle_fixed,
            "check_object_layers_fixed must match the oracle exactly"
        );
    }

    /// Proof path 2 (R19, `port/manifest.tsv`'s `Slicing.hpp` row): `has_raft`,
    /// `first_object_layer_height_fixed`, `object_print_z_height` and
    /// `object_print_z_uncompensated_height` are `Slicing.hpp`'s own header-only, inline-defined
    /// methods -- no stage dump field is any one of these calls' own output in isolation, so
    /// this proof reaches the oracle through the per-unit bridge instead, naming oracle `built`,
    /// the same `proof path: 2`-with-no-dump-coverage shape `docs/PORTING.md`'s "Decisions
    /// recorded on 2026-11-26", decision 2, accepts.
    // PROOF: 2
    #[test]
    fn slicing_params_header_only_methods_match_the_oracle() {
        let params = SlicingParameters {
            base_raft_layers: 2,
            interface_raft_layers: 1,
            first_object_layer_bridging: false,
            object_print_z_min: 1f64,
            object_print_z_max: 11.0,
            object_print_z_uncompensated_max: 10.5,
            ..SlicingParameters::default()
        };
        let actual = vec![
            params.has_raft() as i64 as f64,
            params.first_object_layer_height_fixed() as i64 as f64,
            params.object_print_z_height(),
            params.object_print_z_uncompensated_height(),
        ];
        let oracle = slic3r_proof::proof_units::slicing::path2_slicing_params_header_only(
            params.base_raft_layers as i64,
            params.interface_raft_layers as i64,
            params.first_object_layer_bridging,
            params.object_print_z_min,
            params.object_print_z_max,
            params.object_print_z_uncompensated_max,
        );
        assert_eq!(
            actual, oracle,
            "SlicingParameters's header-only accessors must match the oracle exactly"
        );
    }
}

// PORT STATUS
// source: src/libslic3r/Slicing.cpp
// source: src/libslic3r/Slicing.hpp
// confidence: medium
// tier: exact
// oracle: built
