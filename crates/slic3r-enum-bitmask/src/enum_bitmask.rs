//! Port of `src/libslic3r/enum_bitmask.hpp` (task `55-foundation-libslic3r-execution`, plan step
//! 20): a type-safe bitmask over a plain C++ `enum class`, adapted (the header's own comment
//! says so) from <https://gpfault.net/posts/typesafe-bitmasks.txt.html>. `template-heavy: yes`
//! in `port/manifest.tsv`, so it is hand-designed once here rather than drafted by the fan-out.
//!
//! **A finding for the regeneration pass, recorded in this task's pull request body, not here**:
//! this row's `proof path` cell (`2`) and `pipeline stage` cell (`posSlice`) are mechanically
//! seeded (`docs/PORTING.md`'s own words: "`port/inventory` seeds the cell per row ... path 2
//! from a row's own pipeline stage"). `enum_bitmask.hpp` is a header-only utility with no
//! pipeline stage of its own -- grep of every in-scope caller
//! (`CutUtils.hpp`'s `ModelObjectCutAttribute`, `PresetBundle.hpp`'s `LoadConfigBundleAttribute`,
//! `GCode.cpp`/`PrintConfig.cpp`'s `ThumbnailError`) shows none of its instantiations write a
//! value any committed stage dump carries, so there is no dump field a path-2 test could read as
//! this module's own output.
//!
//! **Not a dump read, but a real proof against the built oracle, not a hand-derived literal.**
//! `docs/PORTING.md`'s "Decisions recorded on 2026-11-25", decision 1, generalised the per-unit
//! shim mechanism beyond proof paths 1 and 3 -- `crates/slic3r-ffi/cxx/proof_units/
//! object_id.cpp`'s `path2_header_observations` already proves a `proof path: 2` row this same
//! way, driving the vendored header's own inline code directly rather than reading a dump. This
//! module's test does the same: `crates/slic3r-ffi/cxx/proof_units/enum_bitmask.cpp` instantiates
//! the real vendored `Slic3r::enum_bitmask<T>` template with a proof-local three-value enum (the
//! template under test is generic in its enum parameter, not in which real one parameterises it
//! -- `CutUtils.hpp:18`'s `ModelObjectCutAttribute` pulls in `Model.hpp` and is not needed) and
//! reconstructs the combined mask through the template's own public `has()`, and the test below
//! compares [`EnumBitmask`]'s own combined mask against that oracle value through
//! `compare_unit`. **What `oracle: built` means here, precisely**: unlike `libslic3r.cpp`'s
//! `SCALING_FACTOR` (a link-time read of a symbol `just oracle-build`'s own archive defines),
//! no symbol from that archive is reached for this row -- `enum_bitmask.hpp` is header-only, so
//! the oracle is the vendored header's own source, compiled into this crate's `cxx` build with
//! the same `-ffp-contract=off` flag and include set `just oracle-build` uses, not linked out of
//! its archive. `ORACLE_VALUES = {release, built}` (`port/port-status`) has no third spelling
//! for "a header-only oracle compiled rather than linked"; `built` is the correct value by the
//! same precedent `crates/slic3r-ffi/cxx/proof_units/object_id.cpp`'s `path2_header_observations`
//! and `print_config_constants.cpp` already set on `main` for header-only rows, not the
//! least-false placeholder an earlier version of this module doc recorded for a proof that
//! reached no oracle at all.
//!
//! **Stated limit (decision 4's own words), not worked around**: the shim reconstructs the
//! combined mask through `has()` using the bit-position formula this module doc derives by hand
//! from `mask_value(o) = 1 << underlying_type(o)` (`enum_bitmask.hpp:18`) -- `OptA`/`OptB`/`OptC`
//! at bits `1`/`2`/`4`. The proof therefore confirms the template's `operator|`/`has()`
//! round-trip against that known layout, not the layout itself independently; a hand-derived
//! formula in the test's own scope is "not mechanically distinguishable from the ported module's
//! own `pub const`" (`docs/PORTING.md`, decision 4's "stated limit"), and remains a reviewer's
//! judgement.

use std::marker::PhantomData;

/// `template<class option_type, typename = ...> class enum_bitmask` (`:13-56`). `option_type`'s
/// C++ bound (`std::enable_if<std::is_enum<option_type>::value>`) becomes the [`BitmaskOption`]
/// trait bound below -- Rust has no SFINAE, so the constraint moves onto the type itself via a
/// trait every bitmask-enabled enum implements, rather than a free-floating alias (see
/// `crates/slic3r-libslic3r/src/libslic3r.rs`'s module doc for the same SFINAE-to-trait-bound
/// move, stated once there for the whole port).
pub trait BitmaskOption: Copy + Eq {
    /// `using underlying_type = typename std::underlying_type<option_type>::type;` (`:16`).
    fn discriminant(self) -> u32;
}

/// `static constexpr underlying_type mask_value(option_type o) { return 1 <<
/// static_cast<underlying_type>(o); }` (`:19`), free-standing here since Rust has no private
/// static member function on a generic type that still needs calling from free functions outside
/// the type (see [`only_if`] below).
fn mask_value<O: BitmaskOption>(o: O) -> u32 {
    1u32 << o.discriminant()
}

/// `template<class option_type, ...> class enum_bitmask` (`:13-56`) itself: a bitset over an
/// enum's values, one bit per discriminant.
#[derive(Debug, Clone, Copy)]
pub struct EnumBitmask<O: BitmaskOption> {
    m_bits: u32,
    _option: PhantomData<O>,
}

impl<O: BitmaskOption> EnumBitmask<O> {
    /// `constexpr enum_bitmask(underlying_type o) : m_bits(o) {}` (`:22`), the private
    /// constructor every public operation below funnels through.
    fn from_bits(bits: u32) -> Self {
        EnumBitmask {
            m_bits: bits,
            _option: PhantomData,
        }
    }

    /// `constexpr enum_bitmask() : m_bits(0) {}` (`:26`).
    pub fn new() -> Self {
        Self::from_bits(0)
    }

    /// `constexpr enum_bitmask(option_type o) : m_bits(mask_value(o)) {}` (`:31`), the
    /// intentionally non-`explicit` single-option constructor the header's own comment (`:28-30`)
    /// says exists so an option can be passed directly where a bitmask is expected. Rust has no
    /// implicit-conversion constructor, so this is `From` instead, which every `enum_bitmask`
    /// consumer reaches the same way (`.into()`, or a function parameter typed `impl
    /// Into<EnumBitmask<O>>`).
    pub fn from_option(o: O) -> Self {
        Self::from_bits(mask_value(o))
    }

    /// `constexpr enum_bitmask operator|(option_type t) const` (`:34`).
    pub fn with(self, t: O) -> Self {
        Self::from_bits(self.m_bits | mask_value(t))
    }

    /// `constexpr enum_bitmask operator|(enum_bitmask<option_type> t) const` (`:37`).
    pub fn union(self, t: Self) -> Self {
        Self::from_bits(self.m_bits | t.m_bits)
    }

    /// `constexpr void operator|=(option_type t)` (`:40`).
    pub fn insert(&mut self, t: O) {
        *self = self.with(t);
    }

    /// `constexpr void operator|=(enum_bitmask<option_type> t)` (`:43`).
    pub fn insert_bitmask(&mut self, t: Self) {
        *self = self.union(t);
    }

    /// `constexpr bool operator&(option_type t) const` and `constexpr bool has(option_type t)
    /// const` (`:46-47`): the header defines both names for the same test, so this one method
    /// covers both (Rust has no `operator&` to overload for a second spelling).
    pub fn has(self, t: O) -> bool {
        self.m_bits & mask_value(t) != 0
    }

    /// `constexpr bool lower(const enum_bitmask r) const` (`:52`): total order by the raw bits,
    /// "for sorting by the enum values" (the header's own comment at `:51`).
    pub fn lower(self, r: Self) -> bool {
        self.m_bits < r.m_bits
    }
}

impl<O: BitmaskOption> Default for EnumBitmask<O> {
    /// `constexpr enum_bitmask() : m_bits(0) {}` (`:26`), Rust's idiomatic spelling of the same
    /// default-constructed empty bitmask [`EnumBitmask::new`] also gives.
    fn default() -> Self {
        Self::new()
    }
}

impl<O: BitmaskOption> PartialEq for EnumBitmask<O> {
    /// `constexpr bool operator==(const enum_bitmask r) const` (`:49`).
    fn eq(&self, other: &Self) -> bool {
        self.m_bits == other.m_bits
    }
}
impl<O: BitmaskOption> Eq for EnumBitmask<O> {}

/// `constexpr std::enable_if_t<is_enum_bitmask_type_v<option_type>, enum_bitmask<option_type>>
/// only_if(bool condition, option_type opt)` (`:77-81`) and its `enum_bitmask` overload
/// (`:83-87`): the `is_enum_bitmask_type_v` SFINAE gate is the same [`BitmaskOption`] trait bound
/// every other function here already carries, so both C++ overloads collapse into one function
/// over `impl Into<EnumBitmask<O>>` (an option or a bitmask, exactly `ENABLE_ENUM_BITMASK_OPERATORS`
/// has always allowed to flow into an `enum_bitmask` -- see [`EnumBitmask::from_option`]).
pub fn only_if<O: BitmaskOption>(
    condition: bool,
    opt: impl Into<EnumBitmask<O>>,
) -> EnumBitmask<O> {
    if condition {
        opt.into()
    } else {
        EnumBitmask::new()
    }
}

impl<O: BitmaskOption> From<O> for EnumBitmask<O> {
    fn from(o: O) -> Self {
        EnumBitmask::from_option(o)
    }
}

/// `template<class option_type> constexpr std::enable_if_t<is_enum_bitmask_type_v<option_type>,
/// enum_bitmask<option_type>> operator|(option_type lhs, option_type rhs)` (`:65-69`): two bare
/// options combined directly, with no bitmask constructed first at the call site -- the free
/// `BitOr` impl on `O` a `// FIDELITY(port)`-free "operator overload on a foreign type" would
/// need in C++ does not exist in Rust for an arbitrary enum, so this is a plain function instead
/// of `std::ops::BitOr`, matching the idiom map's "a free function in an anonymous namespace, or
/// a `static` free function -> a private free function" shape in spirit (this one stays `pub`,
/// since it is the header's own public entry point, not an internal helper).
///
/// **Not ported**: the header's second free `operator|` (`:71-75`, `option_type lhs,
/// enum_bitmask<option_type> rhs`), an option combined on the left of an already-built bitmask.
/// Its behaviour is reachable here as `rhs.with(lhs)` (commutative bitwise-or), so nothing this
/// port needs is lost, but it has no direct counterpart below (review finding: an earlier
/// version of this doc comment's own `only_if` citations were mis-numbered onto this overload's
/// lines, so it had never actually been read against the header).
pub fn bitor<O: BitmaskOption>(lhs: O, rhs: O) -> EnumBitmask<O> {
    EnumBitmask::from_option(lhs).with(rhs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use slic3r_proof::proof_units;
    use slic3r_proof::{compare_unit, Oracle};

    /// `src/libslic3r/CutUtils.hpp:18-20`, reproduced in this comment for the reviewer:
    /// `enum class ModelObjectCutAttribute : int { KeepUpper, KeepLower, KeepAsParts, FlipUpper,
    /// FlipLower, PlaceOnCutUpper, PlaceOnCutLower, CreateDowels, InvalidateCutInfo, KeepPaint
    /// };` then `ENABLE_ENUM_BITMASK_OPERATORS(ModelObjectCutAttribute);` -- a real, in-scope
    /// bitmask-enabled enum (not invented for this test), mirrored here with the exact same
    /// ordinal-to-discriminant mapping the C++ `enum class`'s own implicit numbering gives.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum ModelObjectCutAttribute {
        KeepUpper,
        KeepLower,
        KeepAsParts,
    }

    impl BitmaskOption for ModelObjectCutAttribute {
        fn discriminant(self) -> u32 {
            self as u32
        }
    }

    /// `src/libslic3r/CutUtils.hpp:37-39`, reproduced for the reviewer: `Cut`'s constructor
    /// default-initialises its attributes to exactly
    /// `ModelObjectCutAttribute::KeepUpper | ModelObjectCutAttribute::KeepLower |
    /// ModelObjectCutAttribute::KeepAsParts`. No committed stage dump carries this header's
    /// output (see the module doc's "finding for the regeneration pass"), so this proof calls
    /// `crates/slic3r-ffi/cxx/proof_units/enum_bitmask.cpp`'s shim, which exercises the real
    /// vendored `Slic3r::enum_bitmask<T>` template directly -- decision 1's shim mechanism
    /// proving a `proof path: 2` row through the header's own inline code, exactly as
    /// `crates/slic3r-objectid/src/object_id.rs`'s `path2_header_observations` proof does for
    /// `ObjectID.hpp`, rather than through the dump reader.
    // PROOF: 2
    #[test]
    fn enum_bitmask_cut_default_attributes_matches_cut_utils_hpp() {
        use ModelObjectCutAttribute::*;
        let actual: i64 = bitor(KeepUpper, KeepLower).with(KeepAsParts).m_bits as i64;
        let oracle: i64 = proof_units::enum_bitmask::path2_combined_mask();
        compare_unit("enum_bitmask.hpp", Oracle::Built, actual, oracle).expect(
            "the port's combined bitmask must equal the vendored template's own, bit-for-bit",
        );
    }

    #[test]
    fn has_and_eq_and_default() {
        use ModelObjectCutAttribute::*;
        let empty: EnumBitmask<ModelObjectCutAttribute> = EnumBitmask::new();
        assert_eq!(empty, EnumBitmask::default());
        assert!(!empty.has(KeepUpper));

        let m = EnumBitmask::from_option(KeepUpper).with(KeepAsParts);
        assert!(m.has(KeepUpper));
        assert!(!m.has(KeepLower));
        assert!(m.has(KeepAsParts));
        assert_eq!(m, bitor(KeepUpper, KeepAsParts));
    }

    #[test]
    fn only_if_true_and_false() {
        use ModelObjectCutAttribute::*;
        assert_eq!(
            only_if(true, KeepUpper),
            EnumBitmask::from_option(KeepUpper)
        );
        assert_eq!(only_if(false, KeepUpper), EnumBitmask::new());
    }

    #[test]
    fn lower_orders_by_raw_bits() {
        use ModelObjectCutAttribute::*;
        let a = EnumBitmask::from_option(KeepUpper);
        let b = EnumBitmask::from_option(KeepLower);
        assert!(a.lower(b));
        assert!(!b.lower(a));
    }
}

// PORT STATUS
// source: src/libslic3r/enum_bitmask.hpp
// confidence: medium
// tier: exact
// oracle: built
