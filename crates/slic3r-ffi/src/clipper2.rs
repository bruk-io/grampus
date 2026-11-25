//! The `cxx` bridge over the vendored Clipper2 (task 29-ffi-bridge-core, plan step 17, R17).
//!
//! It binds the entry points `src/libslic3r/ClipperUtils.{hpp,cpp}` and
//! `src/libslic3r/ClipperZUtils.{hpp,cpp}` call: `Clipper2Lib::Clipper64` /
//! `Clipper2Lib::ClipperOffset` for the plain 2D surface, and `Clipper2Lib_Z::Clipper64` for the
//! `USINGZ` surface `clipper2_z.cpp` builds. This starts with one boolean operation, one offset,
//! and one Z-namespace call -- what `tests/ffi_smoke.rs` needs -- and grows per ported unit in
//! later tasks (this crate reimplements nothing: every call here passes paths straight through to
//! the vendored library and reads back what it returns).
//!
//! `Clipper2Lib` and `Clipper2Lib_Z` are two distinct namespaces with same-named, differently
//! laid out types (`Point64` carries no `z` field in one and does in the other), produced by
//! `#define USINGZ` wrapping the *same* vendored headers twice. One translation unit cannot see
//! both: `clipper2/clipper2_z.hpp` leaves its own include guard (`CLIPPER_H`) undone on exit so a
//! *second*, separate inclusion of plain `clipper.h` in another TU still works, but the vendored
//! headers below `clipper.h` (`clipper.core.h`, `clipper.engine.h`, ...) keep their own include
//! guards up, so a second include in the *same* TU is silently skipped and the plain namespace's
//! aliases (`Paths64`, `ClipType`, ...) never get defined (`clang++ -E` on a two-include probe
//! confirms this -- see this task's pull request body). So the glue is two translation units,
//! `cxx/clipper2_plain.cc` (plain Clipper2Lib only) and `cxx/clipper2_z.cc` (Clipper2Lib_Z only),
//! sharing one declaration header, `cxx/clipper2_shim.h`, that never includes a Clipper2 header
//! itself -- only the types this bridge generates.

#[cxx::bridge(namespace = "slic3r_ffi::clipper2")]
mod ffi {
    /// A single Clipper2 `Point64`: `{x, y}`, no Z.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct CPoint64 {
        x: i64,
        y: i64,
    }

    /// A single Clipper2 `Path64`, i.e. one closed or open polygon contour.
    #[derive(Clone, Debug, PartialEq, Eq, Default)]
    struct CPath64 {
        points: Vec<CPoint64>,
    }

    /// A single `Clipper2Lib_Z` point: `{x, y, z}`.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct CPointZ {
        x: i64,
        y: i64,
        z: i64,
    }

    /// A single `Clipper2Lib_Z` path.
    #[derive(Clone, Debug, PartialEq, Eq, Default)]
    struct CPathZ {
        points: Vec<CPointZ>,
    }

    unsafe extern "C++" {
        include!("cxx/clipper2_shim.h");

        /// `Clipper2Lib::Clipper64::Execute(ClipType::Union, FillRule::NonZero, ...)` over the
        /// given subject paths and no clip paths -- the boolean operation
        /// `src/libslic3r/ClipperUtils.cpp`'s `c2_clip(C2::ClipType::Union, ..., {}, ...)` calls
        /// at (for example) `:808`.
        fn clipper2_union(subject: &Vec<CPath64>) -> Vec<CPath64>;

        /// `Clipper2Lib::ClipperOffset::Execute(delta, ...)` with `JoinType::Miter` and
        /// `EndType::Polygon` -- `DefaultJoinType`/`DefaultEndType` at
        /// `src/libslic3r/ClipperUtils.hpp:31,33`.
        fn clipper2_offset(subject: &Vec<CPath64>, delta: f64) -> Vec<CPath64>;

        /// `Clipper2Lib_Z::Clipper64::Execute(ClipType::Union, FillRule::NonZero, ...)`, the Z
        /// surface `src/libslic3r/ClipperZUtils.cpp`'s `clip_zpaths` reaches through
        /// `<clipper2/clipper2_z.hpp>` (`ClipperZUtils.cpp:4`). No Z-fill callback is wired yet --
        /// the smoke test exercises Z paths with no intersection, where Clipper2 carries each
        /// input vertex's Z through unchanged.
        fn clipper2_z_union(subject: &Vec<CPathZ>) -> Vec<CPathZ>;
    }
}

pub use ffi::{CPath64, CPathZ, CPoint64, CPointZ};

/// Union a set of integer-coordinate paths through `Clipper2Lib::Clipper64`.
pub fn union_paths(subject: &[CPath64]) -> Vec<CPath64> {
    ffi::clipper2_union(&subject.to_vec())
}

/// Offset a set of integer-coordinate paths by `delta` through `Clipper2Lib::ClipperOffset`,
/// with the library's own default join (miter) and end (polygon) types.
pub fn offset_paths(subject: &[CPath64], delta: f64) -> Vec<CPath64> {
    ffi::clipper2_offset(&subject.to_vec(), delta)
}

/// Union a set of `(x, y, z)` paths through `Clipper2Lib_Z::Clipper64`.
pub fn union_paths_z(subject: &[CPathZ]) -> Vec<CPathZ> {
    ffi::clipper2_z_union(&subject.to_vec())
}
