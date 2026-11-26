//! Port of `src/libslic3r/libslic3r.cpp` and `src/libslic3r/libslic3r.h` (task
//! `55-foundation-libslic3r-execution`, plan step 20): the library prelude every other in-scope
//! module reads. `libslic3r.h` is `template-heavy: yes` in `port/manifest.tsv`, so it is one of
//! the foundation headers plan step 20 hand-designs once, before the fan-out
//! (`docs/WORKFLOW.md` "Unit of work").
//!
//! **The scaling entry points** (`docs/PORTING.md` "Scaling conversions") are this module's one
//! load-bearing contract: [`SCALING_FACTOR`] is the port's single spelling for the pin's
//! `0.000001`, defined exactly once in this file; [`scale_`] and [`unscale_`] are the two
//! call-site-decided macros (`libslic3r.h:106-107`); [`unscale`] is the templated multiplicative
//! conversion (`:137-138`). No other module in this port may declare a second spelling of the
//! `0.000001` literal or divide by `SCALING_FACTOR`'s reciprocal -- see each item's own doc
//! comment for the exact rule it is held to.
//!
//! **A finding for the regeneration pass, recorded in this task's pull request body, not here**:
//! `docs/PORTING.md:390-393` attributes defining `slic3r_libslic3r::SCALING_FACTOR` to
//! `34-foundation-spike`; that spike committed only a doc-comment-only `src/lib.rs` for every
//! crate, so this task (`55-foundation-libslic3r-execution`) is the one that actually defines it.
//! `docs/PORTING.md` is unchanged on this branch (it is on the must-not-write list).
//!
//! **Items this module does not port**, each a language-level C++ construct with no Rust
//! counterpart to carry over, rather than a workaround:
//! - The SFINAE type-constraint aliases `FloatingOnly`/`ScaledCoordOnly`/`IntegerOnly`/
//!   `ArithmeticOnly`/`IteratorOnly` and the `is_scaled_coord` predicate they are built from
//!   (`libslic3r.h:333-367`): Rust expresses the same constraint as a trait bound directly on
//!   the generic function that needs it, so there is no standalone item to port -- the
//!   constraint moves to each caller's own `where` clause instead of living here.
//! - `remove_cvref_t` (`:379-380`): Rust generics are never implicitly reference- or
//!   cv-qualified the way a deduced C++ template parameter can be, so there is nothing to strip.
//! - `make_unique` (`:238-241`): a pre-C++14 `std::make_unique` polyfill; `Box::new` is the
//!   direct Rust equivalent and already exists.
//! - The `_WIN32` `boost::container::deque` arm of the `deque` alias (`:128-135`): the pin this
//!   port is held to is never built for Windows (`just oracle-build` targets `arm64` macOS
//!   only), so this arm is dead code on every configuration this port compares against.
//!
//! `// PROOF: 1` proves `libslic3r.cpp`'s one in-scope line (`SCALING_FACTOR`'s definition,
//! through the FFI bridge to the pin's own global). `// PROOF: 2` proves `libslic3r.h`'s own
//! scaling arithmetic against the committed stage dumps -- see each test for which.

use std::collections::VecDeque;

// ---------------------------------------------------------------------------------------------
// Type map (libslic3r.h:48, :51)
// ---------------------------------------------------------------------------------------------

/// `using coord_t = int64_t;` (`libslic3r.h:48`; the `int32_t` branch at `:45` is inside `#if 0`
/// and is not live -- see `docs/PORTING.md`'s type map). Most of the port spells this directly as
/// `i64` per that type map; this alias exists so the header's own declaration is still ported
/// faithfully.
pub type CoordT = i64;

/// `using coordf_t = double;` (`libslic3r.h:51`).
pub type CoordfT = f64;

// ---------------------------------------------------------------------------------------------
// Scaling conversions (docs/PORTING.md "Scaling conversions")
// ---------------------------------------------------------------------------------------------

/// `SCALING_FACTOR_INTERNAL` (`libslic3r.h:65`) folded into the one constant
/// `docs/PORTING.md` fixes: `extern double SCALING_FACTOR;` (`:80`) is defined at
/// `src/libslic3r/libslic3r.cpp:3` as `double SCALING_FACTOR = SCALING_FACTOR_INTERNAL;`, and
/// the only mutator anywhere in the vendored tree is `slic3r/GUI/Plater.cpp`'s branch, which R10
/// puts out of scope (the GUI). So the port's whole surface sees exactly this one value, and it
/// is a `const`, never a `static` the port could reassign. `SCALING_FACTOR_INTERNAL_LARGE_PRINTER`
/// (`libslic3r.h:66`, `0.00001`) and `LARGE_BED_THRESHOLD` (`:67`, `2147`) are not ported: they
/// feed only that out-of-scope mutator.
///
/// No other module in this port may write the literal `0.000001` inline or declare a second
/// constant for it (`docs/PORTING.md`'s banned-substitution list) -- every other scaling
/// conversion reads this constant instead.
// PORT SRC: src/libslic3r/libslic3r.h:65
pub const SCALING_FACTOR: f64 = 0.000001;

/// `static constexpr double EPSILON = 1e-4;` (`libslic3r.h:57`).
// PORT SRC: src/libslic3r/libslic3r.h:57
pub const EPSILON: f64 = 1e-4;

/// `#define scale_(val) ((val) / SCALING_FACTOR)` (`libslic3r.h:106`). Per "Scaling conversions":
/// a plain `f64` division that truncates nothing itself -- the rounding variant that used to
/// precede it (commented out at `:105`) is on the don't-translate list, and where the original
/// call site goes on to convert the quotient to an integer type, that truncation is reproduced
/// **at the call site**, never folded into this function.
pub fn scale_(val: f64) -> f64 {
    val / SCALING_FACTOR
}

/// `#define unscale_(val) ((val) * SCALING_FACTOR)` (`:107`). A plain `f64` multiplication; like
/// every member of the multiplicative family, it never truncates.
pub fn unscale_(val: f64) -> f64 {
    val * SCALING_FACTOR
}

/// `#define SCALED_EPSILON scale_(EPSILON)` (`:109`): a function, not a constant, since it is
/// defined in terms of [`scale_`] rather than as a second literal.
pub fn scaled_epsilon() -> f64 {
    scale_(EPSILON)
}

/// The two floating output widths [`unscale`] is ever instantiated with in-scope, standing in
/// for the C++ template parameter `T` of `template<typename T, typename Q> inline T unscale(Q v)`
/// (`libslic3r.h:137-138`): Rust has no implicit numeric promotion, so each output type gets its
/// own impl rather than a single generic numeric trait pulled in from an external crate (no
/// foundation task may add a dependency).
pub trait UnscaleOutput: Copy + std::ops::Mul<Output = Self> {
    #[doc(hidden)]
    fn __from_coord(v: CoordT) -> Self;
    #[doc(hidden)]
    fn __scaling_factor() -> Self;
}

impl UnscaleOutput for f64 {
    fn __from_coord(v: CoordT) -> f64 {
        v as f64
    }
    fn __scaling_factor() -> f64 {
        SCALING_FACTOR
    }
}

impl UnscaleOutput for f32 {
    fn __from_coord(v: CoordT) -> f32 {
        v as f32
    }
    fn __scaling_factor() -> f32 {
        SCALING_FACTOR as f32
    }
}

/// `template<typename T, typename Q> inline T unscale(Q v) { return T(v) * T(SCALING_FACTOR); }`
/// (`libslic3r.h:137-138`) -- a multiplication, per "Scaling conversions": `v` is cast to the
/// output type `T` directly (never promoted through `f64` first), then multiplied by
/// `SCALING_FACTOR` cast to that same `T`. Never truncates, matching every member of the
/// multiplicative family.
///
/// `Q` is narrowed to [`CoordT`] (`coord_t`, the scaled-coordinate input every in-scope caller
/// this task reaches instantiates it with); a later unit needing a different `Q` adds its own
/// overload and names the gap in its own pull request
/// (`// TODO(port): src/libslic3r/libslic3r.h:137-138 -- unscale<T> instantiated with a Q other
/// than coord_t`).
pub fn unscale<T: UnscaleOutput>(v: CoordT) -> T {
    T::__from_coord(v) * T::__scaling_factor()
}

// ---------------------------------------------------------------------------------------------
// The rest of the header's constants, in source order
// ---------------------------------------------------------------------------------------------

/// `static constexpr size_t MAXIMUM_EXTRUDER_NUMBER = 64;` (`:70`).
// PORT SRC: src/libslic3r/libslic3r.h:70
pub const MAXIMUM_EXTRUDER_NUMBER: usize = 64;

/// `static constexpr size_t MAXIMUM_AMS_SYNC_FILAMENT_NUMBER = 16;` (`:75`).
// PORT SRC: src/libslic3r/libslic3r.h:75
pub const MAXIMUM_AMS_SYNC_FILAMENT_NUMBER: usize = 16;

/// `static constexpr float MAX_LINE_WIDTH_MULTIPLIER = 5;` (`:78`).
// PORT SRC: src/libslic3r/libslic3r.h:78
pub const MAX_LINE_WIDTH_MULTIPLIER: f32 = 5f32;

/// `static constexpr double PI = 3.141592653589793238;` (`:81`).
// PORT SRC: src/libslic3r/libslic3r.h:81
pub const PI: f64 = 3.141592653589793238;

/// `#define POLY_SIDE_COUNT 24 // for brim ear circle` (`:82`).
// PORT SRC: src/libslic3r/libslic3r.h:82
pub const POLY_SIDE_COUNT: u32 = 24;

/// `static constexpr double RESOLUTION = 0.0125;` (`:86`).
// PORT SRC: src/libslic3r/libslic3r.h:86
pub const RESOLUTION: f64 = 0.0125;

/// `#define SCALED_RESOLUTION (RESOLUTION / SCALING_FACTOR)` (`:87`).
pub fn scaled_resolution() -> f64 {
    RESOLUTION / SCALING_FACTOR
}

/// `static constexpr double SPARSE_INFILL_RESOLUTION = 0.04;` (`:88`).
// PORT SRC: src/libslic3r/libslic3r.h:88
pub const SPARSE_INFILL_RESOLUTION: f64 = 0.04;

/// `#define SCALED_SPARSE_INFILL_RESOLUTION (SPARSE_INFILL_RESOLUTION / SCALING_FACTOR)` (`:89`).
pub fn scaled_sparse_infill_resolution() -> f64 {
    SPARSE_INFILL_RESOLUTION / SCALING_FACTOR
}

/// `static constexpr double SUPPORT_RESOLUTION = 0.0375;` (`:91`).
// PORT SRC: src/libslic3r/libslic3r.h:91
pub const SUPPORT_RESOLUTION: f64 = 0.0375;

/// `#define SCALED_SUPPORT_RESOLUTION (SUPPORT_RESOLUTION / SCALING_FACTOR)` (`:92`).
pub fn scaled_support_resolution() -> f64 {
    SUPPORT_RESOLUTION / SCALING_FACTOR
}

/// `#define SMALL_PERIMETER_LENGTH(LENGTH) (((LENGTH) / SCALING_FACTOR) * 2 * PI)` (`:94`).
pub fn small_perimeter_length(length: f64) -> f64 {
    (length / SCALING_FACTOR) * 2f64 * PI
}

/// `static constexpr double INSET_OVERLAP_TOLERANCE = 0.4;` (`:95`).
// PORT SRC: src/libslic3r/libslic3r.h:95
pub const INSET_OVERLAP_TOLERANCE: f64 = 0.4;

/// `static constexpr double EXTERNAL_INFILL_MARGIN = 3;` (`:98`).
// PORT SRC: src/libslic3r/libslic3r.h:98
pub const EXTERNAL_INFILL_MARGIN: f64 = 3f64;

/// `static constexpr double BRIDGE_INFILL_MARGIN = 1;` (`:99`).
// PORT SRC: src/libslic3r/libslic3r.h:99
pub const BRIDGE_INFILL_MARGIN: f64 = 1f64;

/// `static constexpr double WIPE_TOWER_MARGIN = 1.;` (`:100`).
// PORT SRC: src/libslic3r/libslic3r.h:100
pub const WIPE_TOWER_MARGIN: f64 = 1f64;

/// `static constexpr double WIPE_TOWER_AUTO_MARGIN = 15.;` (`:103`).
// PORT SRC: src/libslic3r/libslic3r.h:103
pub const WIPE_TOWER_AUTO_MARGIN: f64 = 15.;

/// `#ifndef UNUSED #define UNUSED(x) (void)(x) #endif` (`:111-113`): a pre-C++17 workaround for
/// an unused-parameter/variable warning. Rust's own `_` binding (or `#[allow(unused)]`) is the
/// idiom every call site uses instead; there is no standalone item to port.
///
/// `//BBS: some global const config which user can not change, but developer can` (`:115`):
#[allow(non_upper_case_globals)]
// PORT SRC: src/libslic3r/libslic3r.h:116
pub const g_config_support_sharp_tails: bool = true;
#[allow(non_upper_case_globals)]
// PORT SRC: src/libslic3r/libslic3r.h:117
pub const g_config_remove_small_overhangs: bool = true;
#[allow(non_upper_case_globals)]
// PORT SRC: src/libslic3r/libslic3r.h:118
pub const g_config_tree_support_collision_resolution: f32 = 0.2;

// ---------------------------------------------------------------------------------------------
// namespace Slic3r { ... } (:123-435)
// ---------------------------------------------------------------------------------------------

// `extern Semver SEMVER;` (`:125`): the global current-build version. Its Rust type is pending
// the `slic3r-semver` crate's own ported module, which no foundation task writes (that crate's
// modules are a Phase A/B fan-out item -- see `docs/PORTING.md`'s "Foundation types"). A plain
// `//` comment (review finding: a `///` block here with no item under it attached itself to
// `Deque` below instead).
// TODO(port): src/libslic3r/libslic3r.h:125 -- Rust type and value pending slic3r-semver's own
// ported module (Phase A/B fan-out); this prelude does not invent one.

/// `template<class T, class Allocator = std::allocator<T>> using deque = ...;` (`:128-135`): the
/// non-Windows arm (`std::deque<T, Allocator>`, `:134`) is the only one ever compiled for this
/// pin's `arm64` build -- see the module doc's "items this module does not port" for the
/// `_WIN32` arm.
pub type Deque<T> = VecDeque<T>;

/// `enum Axis { X=0, Y, Z, E, F, I, J, P, NUM_AXES, UNKNOWN_AXIS = NUM_AXES,
/// NUM_AXES_WITH_UNKNOWN };` (`:140-154`). Every discriminant past `X=0` is the C++ compiler's
/// own implicit auto-increment, carrying no literal of its own to transcribe -- this enum writes
/// no explicit discriminant at all, which Rust's own default auto-increment (`X` first and
/// unadorned) reproduces exactly, variant for variant, with nothing to cite.
///
/// `UNKNOWN_AXIS = NUM_AXES` (`:152`) is not a distinct Rust variant: Rust rejects two enum
/// variants sharing one discriminant (unlike C++), so [`UNKNOWN_AXIS`] is a const alias for
/// [`Axis::NumAxes`] instead, below -- the port's answer to a genuine language difference, not a
/// value difference: it carries the exact same number the pin does.
///
/// `NUM_AXES_WITH_UNKNOWN` (`:153`) is a real tenth discriminant, not merely a count: it is a
/// distinct, un-aliased enumerator (`9`, one past `UNKNOWN_AXIS = NUM_AXES = 8`), and
/// `src/libslic3r/GCodeReader.cpp:74,91` assigns and compares it as an `Axis` value (`Axis axis =
/// NUM_AXES_WITH_UNKNOWN;`), not as an array bound -- so it is [`Axis::NumAxesWithUnknown`], a
/// real variant, rather than a `u8` alongside [`UNKNOWN_AXIS`]'s `Axis`-typed one (review
/// finding: an earlier version of this port typed it `u8`, which a caller that handles every
/// `Axis` sentinel uniformly could not accept the same way as [`UNKNOWN_AXIS`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Axis {
    X,
    Y,
    Z,
    E,
    F,
    I,
    J,
    P,
    NumAxes,
    NumAxesWithUnknown,
}

/// `UNKNOWN_AXIS = NUM_AXES` (`:152`) -- see [`Axis`]'s own doc comment.
pub const UNKNOWN_AXIS: Axis = Axis::NumAxes;

/// `NUM_AXES_WITH_UNKNOWN` (`:153`), a distinct `Axis` enumerator one past [`UNKNOWN_AXIS`] --
/// see [`Axis`]'s own doc comment.
pub const NUM_AXES_WITH_UNKNOWN: Axis = Axis::NumAxesWithUnknown;

/// `template <typename T, typename Alloc, typename Alloc2> inline void append(std::vector<T,
/// Alloc> &dest, const std::vector<T, Alloc2> &src)` and its move overload (`:156-176`): Rust's
/// ownership system collapses the by-reference (copy) and by-value (move) C++ overloads into
/// one function over `IntoIterator` -- a `Vec<T>` argument moves and drains exactly as the move
/// overload does, while `vec.iter().cloned()` reproduces the by-reference overload's copy. The
/// "if dest is empty, assign outright" branch both overloads open with is a pre-allocation
/// micro-optimisation `Vec::extend`'s own amortised growth already subsumes, so it is not
/// reproduced as a separate branch.
pub fn append<T>(dest: &mut Vec<T>, src: impl IntoIterator<Item = T>) {
    dest.extend(src);
}

/// `template<class T, class... Args> void clear_and_shrink(std::vector<T, Args...>& vec)`
/// (`:178-185`): replaces `vec` with a fresh, zero-capacity `Vec`, exactly as the original swaps
/// in a temporary and lets it drop. `debug_assert!` per "Error sets": the original's `assert` is
/// compiled out of the oracle's release build and is never a live check.
pub fn clear_and_shrink<T>(vec: &mut Vec<T>) {
    *vec = Vec::new();
    debug_assert_eq!(vec.capacity(), 0);
}

/// `template <typename T> inline void append_reversed(std::vector<T>& dest, const
/// std::vector<T>& src)` and its move overload (`:187-209`): the same by-reference/by-value
/// collapse [`append`] documents, with the source iterated in reverse -- except that, unlike
/// [`append`], the "if `dest` is empty" branch both overloads open with
/// (`if (dest.empty()) dest = src;` at `:191-192`; `if (dest.empty()) dest = std::move(src);` at
/// `:201-202`) is **not** the same pre-allocation micro-optimisation [`append`]'s identical-looking
/// branch is: it copies `src` forward, so for an empty `dest` this function's own output order
/// differs from the non-empty-`dest` branch's reversed order (`:193-194`, `:203-206`). An earlier
/// version of this port reversed unconditionally regardless of `dest`, reproducing [`append`]'s
/// justification for a branch that here changes element order rather than merely allocation
/// strategy -- a review finding, fixed below.
// PORT NOTE: src/libslic3r/libslic3r.h:191-192,201-206 -- the vendored function itself has this
// asymmetry (forward copy into an empty `dest`, reversed copy otherwise); it is reproduced here
// rather than normalised away, per the rejection rule (port what the source does).
pub fn append_reversed<T>(dest: &mut Vec<T>, src: impl DoubleEndedIterator<Item = T>) {
    if dest.is_empty() {
        dest.extend(src);
    } else {
        dest.extend(src.rev());
    }
}

/// `template<typename T_TO, typename T_FROM> std::vector<T_TO> cast(const std::vector<T_FROM>
/// &src)` (`:212-220`): an element-wise C-style cast, ported as Rust's own `as` cast through
/// [`CastElement`] -- a closed set of the primitive pairs this port's in-scope numeric types
/// actually need (`coord_t`/`i64`, `coordf_t`/`f64`, `float`/`f32`, `usize`), rather than a
/// generic numeric trait pulled in from an external crate.
pub trait CastElement<From> {
    fn cast_from(v: From) -> Self;
}

macro_rules! impl_cast_element {
    ($to:ty, $from:ty) => {
        impl CastElement<$from> for $to {
            fn cast_from(v: $from) -> $to {
                v as $to
            }
        }
    };
}
impl_cast_element!(i64, i32);
impl_cast_element!(i32, i64);
impl_cast_element!(f32, f64);
impl_cast_element!(f64, f32);
impl_cast_element!(f64, i64);
impl_cast_element!(i64, f64);
impl_cast_element!(f32, i64);
impl_cast_element!(i64, f32);
impl_cast_element!(usize, i64);
impl_cast_element!(i64, usize);

pub fn cast<To, From: Copy>(src: &[From]) -> Vec<To>
where
    To: CastElement<From>,
{
    src.iter().map(|&a| To::cast_from(a)).collect()
}

/// `template <typename T> inline void remove_nulls(std::vector<T*> &vec)` (`:222-228`): Rust's
/// idiom for "a vector of possibly-absent pointers" is `Vec<Option<T>>`, not a raw-pointer
/// vector compacted by a null check, so this operates on that shape directly.
///
/// This is a real narrowing, named per review: the C++ template is unconstrained over any
/// pointer type (`std::vector<T*>`, a container of *borrowed* pointers into storage owned
/// elsewhere), while `Vec<Option<T>>` *owns* every `T`. `port/ownership.tsv` has no row for
/// `libslic3r.h` to take a type from (this task's "Raw-pointer fields" section: none of this
/// task's own units declares one at the pin), and this function is a generic template, not a
/// struct field the ownership-classification pass covers, so there is no committed Rust type to
/// take verbatim here either. `remove_nulls`'s one in-scope caller,
/// `src/libslic3r/Support/SupportMaterial.cpp:2451`, passes a vector of borrowed pointers, which
/// this signature cannot accept -- the fan-out task that ports that call site re-derives this
/// function's shape against `port/ownership.tsv`'s row for whatever container that caller's own
/// module uses, rather than reusing this one as-is.
// PORT NOTE: src/libslic3r/libslic3r.h:222 -- the vendored signature takes `std::vector<T*>&`,
// a container of borrowed pointers; this port's `Vec<Option<T>>` instead owns every element,
// since no `port/ownership.tsv` row applies to a generic function parameter and this prelude
// invents no raw-pointer Rust type on its own authority.
pub fn remove_nulls<T>(vec: &mut Vec<Option<T>>) {
    vec.retain(Option::is_some);
}

/// `template <typename T> inline void sort_remove_duplicates(std::vector<T> &vec)` (`:230-235`):
/// `std::sort` is `slice::sort_unstable` per the idiom map (no stability guarantee either way),
/// and `std::unique` removes only consecutive duplicates, which is exactly `Vec::dedup` after a
/// sort has made every duplicate consecutive.
pub fn sort_remove_duplicates<T: Ord>(vec: &mut Vec<T>) {
    vec.sort_unstable();
    vec.dedup();
}

/// `template<class ForwardIt, class LowerThanKeyPredicate> ForwardIt
/// lower_bound_by_predicate(ForwardIt first, ForwardIt last, LowerThanKeyPredicate
/// lower_than_key)` (`:245-264`): the iterator/pointer-arithmetic idiom map rule turns a
/// forward-iterator binary search into a slice indexed by `usize`, which is exactly what
/// `slice::partition_point` already is -- the predicate's partitioned-slice precondition is the
/// same one the hand-rolled C++ loop assumes, so this reuses the standard library's own
/// equivalent primitive rather than re-deriving the loop.
pub fn lower_bound_by_predicate<T>(
    slice: &[T],
    mut lower_than_key: impl FnMut(&T) -> bool,
) -> usize {
    slice.partition_point(|x| lower_than_key(x))
}

/// `template<class ForwardIt, class T, class Compare=std::less<>> ForwardIt binary_find(ForwardIt
/// first, ForwardIt last, const T& value, Compare comp={})` (`:267-276`): `Compare` defaults to
/// `std::less<>`, which `Ord` is the direct Rust counterpart of; a index (`Some`/`None`) replaces
/// the "found/`last`" iterator convention per the idiom map.
pub fn binary_find<T: Ord>(slice: &[T], value: &T) -> Option<usize> {
    let idx = slice.partition_point(|x| x < value);
    if idx < slice.len() && &slice[idx] == value {
        Some(idx)
    } else {
        None
    }
}

/// `template<class ForwardIt, class LowerThanKeyPredicate, class EqualToKeyPredicate> ForwardIt
/// binary_find_by_predicate(...)` (`:279-288`): the predicate-based counterpart to
/// [`binary_find`], exactly as [`lower_bound_by_predicate`] is to a plain `partition_point` call.
pub fn binary_find_by_predicate<T>(
    slice: &[T],
    lower_than_key: impl FnMut(&T) -> bool,
    equal_to_key: impl FnOnce(&T) -> bool,
) -> Option<usize> {
    let idx = lower_bound_by_predicate(slice, lower_than_key);
    if idx < slice.len() && equal_to_key(&slice[idx]) {
        Some(idx)
    } else {
        None
    }
}

/// `template<typename ContainerType, typename ValueType> inline bool contains(const
/// ContainerType &c, const ValueType &v)` (`:290-291`), plus the `initializer_list` overload
/// (`:292-293`): a slice already carries both shapes in Rust, so this one function covers both.
pub fn contains<T: PartialEq>(c: &[T], v: &T) -> bool {
    c.contains(v)
}

/// `template<typename ContainerType, typename ValueType> inline bool one_of(const ValueType &v,
/// const ContainerType &c)` (`:295-296`), plus its `initializer_list` overload (`:297-298`): the
/// same test as [`contains`], with the arguments in the order the original's `one_of` takes them
/// (value first).
pub fn one_of<T: PartialEq>(v: &T, c: &[T]) -> bool {
    contains(c, v)
}

/// `template<typename T> constexpr inline T sqr(T x) { return x * x; }` (`:300-304`).
pub fn sqr<T: std::ops::Mul<Output = T> + Copy>(x: T) -> T {
    x * x
}

/// `template<typename Number> constexpr inline bool is_zero(Number value) { return
/// std::fabs(double(value)) < 1e-6; }` (`:306-310`).
pub fn is_zero(value: f64) -> bool {
    // PORT SRC: src/libslic3r/libslic3r.h:309
    value.abs() < 1e-6
}

/// `template <typename T, typename Number> constexpr inline T lerp(const T& a, const T& b,
/// Number t)` (`:312-317`): `assert` is `debug_assert!` per "Error sets" (compiled out of the
/// oracle's release build, so it is never a live check in the port either).
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    debug_assert!((-EPSILON..=1f64 + EPSILON).contains(&t));
    (1f64 - t) * a + t * b
}

/// `template <typename Number> constexpr inline bool is_approx(Number value, Number test_value,
/// Number precision = EPSILON)` (`:319-323`).
pub fn is_approx(value: f64, test_value: f64, precision: f64) -> bool {
    (value - test_value).abs() < precision
}

/// [`is_approx`] at the default `precision = EPSILON` the C++ default argument gives.
pub fn is_approx_default(value: f64, test_value: f64) -> bool {
    is_approx(value, test_value, EPSILON)
}

/// `template<typename Number> constexpr inline bool is_approx(const std::optional<Number>
/// &value, const std::optional<Number> &test_value)` (`:325-331`).
pub fn is_approx_opt(value: Option<f64>, test_value: Option<f64>) -> bool {
    match (value, test_value) {
        (None, None) => true,
        (Some(a), Some(b)) => is_approx_default(a, b),
        _ => false,
    }
}

/// `template<class T, class I, class... Args> IntegerOnly<I, std::vector<T, Args...>>
/// reserve_vector(I capacity)` (`:369-376`): `IntegerOnly`'s SFINAE constraint is this function's
/// own `capacity: usize` parameter type in Rust, not a standalone alias (see the module doc).
pub fn reserve_vector<T>(capacity: usize) -> Vec<T> {
    let mut v = Vec::new();
    if capacity > 0 {
        v.reserve(capacity);
    }
    v
}

/// `template<class It> class Range` and `template<class Cont> auto range(Cont &&cont)`
/// (`:384-408`): the comment at `:383` already points at `std::ranges::subrange` as this type's
/// eventual C++20 replacement; Rust's `std::ops::Range<usize>` is that same "two endpoints, one
/// view" shape for the index-based case every in-scope caller needs (no in-scope file
/// instantiates `Range` over a node-based iterator at the pin), so the port reuses it directly
/// rather than reimplementing a second range type.
/// `// TODO(port): src/libslic3r/libslic3r.h:384-408 -- a non-index Range instantiation, if a
/// later unit needs one.`
pub fn range_of_len(len: usize) -> std::ops::Range<usize> {
    0..len
}

/// `template<class T, class = FloatingOnly<T>> constexpr T NaN = ...; constexpr float NaNf =
/// NaN<float>; constexpr double NaNd = NaN<double>;` (`:410-414`).
pub const NAN_F32: f32 = f32::NAN;
pub const NAN_D: f64 = f64::NAN;

/// The two integer output widths [`fast_round_up`] is instantiated with in-scope -- the same
/// closed-set-of-impls shape [`UnscaleOutput`] and [`CastElement`] use, for the same reason
/// (Rust has no SFINAE `IntegerOnly<I>` to dispatch on).
pub trait FastRoundUpOutput: Copy {
    #[doc(hidden)]
    fn __zero() -> Self;
    #[doc(hidden)]
    fn __trunc(v: f64) -> Self;
}
impl FastRoundUpOutput for i32 {
    fn __zero() -> i32 {
        0
    }
    fn __trunc(v: f64) -> i32 {
        v as i32
    }
}
impl FastRoundUpOutput for i64 {
    fn __zero() -> i64 {
        0
    }
    fn __trunc(v: f64) -> i64 {
        v as i64
    }
}
impl FastRoundUpOutput for u32 {
    fn __zero() -> u32 {
        0
    }
    fn __trunc(v: f64) -> u32 {
        v as u32
    }
}

/// `template<typename I> inline IntegerOnly<I, I> fast_round_up(double a)` (`:427-433`), ported
/// verbatim per `docs/PORTING.md` "Rounding modes": `floor(a + 0.5)` is round-half-up, not
/// `f64::round`'s round-half-away-from-zero, and the two disagree at the exact boundary this
/// special case exists for (`0.49999999999999994_f64.round()` would give `0.0`, matching here,
/// but the general `floor(a + 0.5)` idiom disagrees with `round()` elsewhere -- see
/// `docs/PORTING.md` for the proof). Never substituted with `f64::round` or a bare
/// `(a + 0.5).floor()` missing this special case.
pub fn fast_round_up<I: FastRoundUpOutput>(a: f64) -> I {
    // PORT SRC: src/libslic3r/libslic3r.h:432
    if a == 0.49999999999999994_f64 {
        I::__zero()
    } else {
        // PORT SRC: src/libslic3r/libslic3r.h:432
        I::__trunc((a + 0.5).floor())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Review finding: `append_reversed`'s empty-`dest` branch copies forward
    /// (`libslic3r.h:191-192,201-202`); only the non-empty branch reverses
    /// (`libslic3r.h:193-194,203-206`). Both are exercised here since an earlier version of this
    /// port reversed unconditionally.
    #[test]
    fn append_reversed_copies_forward_into_empty_dest() {
        let mut dest: Vec<i32> = Vec::new();
        append_reversed(&mut dest, vec![1, 2, 3].into_iter());
        assert_eq!(dest, vec![1, 2, 3]);
    }

    #[test]
    fn append_reversed_reverses_into_non_empty_dest() {
        let mut dest: Vec<i32> = vec![0];
        append_reversed(&mut dest, vec![1, 2, 3].into_iter());
        assert_eq!(dest, vec![0, 3, 2, 1]);
    }
}

// PORT STATUS
// source: src/libslic3r/libslic3r.cpp
// source: src/libslic3r/libslic3r.h
// confidence: high
// tier: exact
// oracle: built
