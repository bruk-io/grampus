//! The `cxx` bridge over the vendored admesh (task 30-ffi-bridge-libraries, plan step 17, R17).
//!
//! It exposes the `stl_*` surface in `deps_src/admesh/stl.h` that the in-scope callers reach --
//! `src/libslic3r/TriangleMesh.cpp` is the main one -- starting with `stl_calculate_volume`
//! (`deps_src/admesh/util.cpp`), which this crate's own `stl_file` builder feeds, and which
//! grows per ported unit in later tasks the same way `clipper2.rs` does.

#[cxx::bridge(namespace = "slic3r_ffi::admesh")]
mod ffi {
    /// One vertex or normal: admesh's `stl_vertex`/`stl_normal`, an `Eigen::Matrix<float, 3, 1>`
    /// (`stl.h:39-40`), laid out here as three plain floats.
    #[derive(Clone, Copy, Debug, PartialEq)]
    struct CVertex {
        x: f32,
        y: f32,
        z: f32,
    }

    /// One `stl_facet` (`stl.h:55-65`): an outward unit normal and three vertices. The normal is
    /// supplied, not computed by this bridge, but `stl_calculate_volume`
    /// (`deps_src/admesh/util.cpp`) does not simply trust it for the whole computation: `get_volume`'s
    /// height term reads the *stored* normal (`util.cpp:282`), while its area term (`get_area`,
    /// `util.cpp:271-273`) derives its own normal from the vertices. A facet whose stored normal
    /// disagrees with its vertices changes only the height factor, not the area one.
    #[derive(Clone, Copy, Debug, PartialEq)]
    struct CFacet {
        normal: CVertex,
        v0: CVertex,
        v1: CVertex,
        v2: CVertex,
    }

    // SAFETY: cxx's own unsafe-extern declaration block (see clipper2.rs's module doc for why
    // this is not a hand-written unsafe expression) -- the only unsafety is in the bindings cxx
    // generates from it.
    unsafe extern "C++" {
        include!("cxx/admesh_shim.h");

        /// Builds an in-memory `stl_file` from `facets`, calls `stl_calculate_volume`
        /// (`deps_src/admesh/util.cpp:290`), and returns `stl->stats.volume` -- the value the
        /// vendored C++ computed via its own area/height summation, not one restated here.
        fn admesh_compute_volume(facets: &Vec<CFacet>) -> f32;
    }
}

pub use ffi::{CFacet, CVertex};

/// The volume `stl_calculate_volume` computes for the given closed set of outward-facing
/// facets.
pub fn compute_volume(facets: &[CFacet]) -> f32 {
    ffi::admesh_compute_volume(&facets.to_vec())
}
