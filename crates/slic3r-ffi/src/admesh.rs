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
    /// supplied, not computed on either side of the bridge -- `stl_calculate_volume`
    /// (`deps_src/admesh/util.cpp`'s `get_volume`) reads the *stored* normal to get a face's
    /// signed distance from the reference point, it does not recompute one from the vertices.
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
