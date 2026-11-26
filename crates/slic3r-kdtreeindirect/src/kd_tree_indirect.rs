//! Port of `src/libslic3r/KDTreeIndirect.hpp` (task `60-foundation-trees-grids`, plan step 20,
//! `foundation-09` row of `port/foundation.tsv`): a KD-tree over an externally owned data set,
//! referenced by index rather than by value. `template-heavy: yes` in `port/manifest.tsv`, so it
//! is hand-designed once here rather than drafted by the fan-out.
//!
//! **Generic over the coordinate type, not collapsed to one.** The idiom map's template row
//! force-collapses a template to one concrete type only when its unit's in-scope callers all
//! instantiate it the same way; this one does not hold here. The three in-scope callers each
//! instantiate differently: `GCode/SeamPlacer.hpp:110` and `Support/SupportSpotsGenerator.cpp:1245`
//! both use `KDTreeIndirect<3, float, ...>`, while `ShortestPath.cpp:202,613` uses
//! `KDTreeIndirect<2, double, ...>`. So [`KDTreeIndirect`] below stays generic over the dimension
//! count (a `const D: usize` generic, the direct counterpart of `ANumDimensions`) and the
//! coordinate type (bounded by [`KdCoord`] below, implemented for `f32` and `f64` -- the two
//! types those three call sites actually use).
//!
//! **`KdCoord` is this module's own bound, not a borrow of a type map row.** The vendored
//! template's `CoordType` is unconstrained except by the operations `KDTreeIndirect.hpp` itself
//! performs on it (`-`, `*`, `<`, `>`, construction from `0` and from `EPSILON`); no row of the
//! type map names a trait for an arbitrary scalar coordinate, so this module defines the minimal
//! one its own body needs, mirroring the vendored header's own implicit requirements rather than
//! inventing a wider numeric abstraction.
//!
//! **`PointType` is `std::ops::Index<usize, Output = C>` rather than a crate type.** The vendored
//! template's `find_closest_points`/`find_nearby_points` are generic over `PointType` and access
//! it with `point[i]` (`find_closest_points`'s `Visitor`) or `bb_min(i)`/`bb_max(i)` (the
//! bounding-box overload of `find_nearby_points`) -- the same "access coordinate `i`" operation
//! under two different C++ call syntaxes. Rust's `Index` trait is the one bound that covers both:
//! it is implemented for `[C; D]` by the standard library already (the natural stand-in for
//! whatever concrete point type -- `Vec2f`/`Vec3f`/a raw array -- a Phase A/B caller supplies), so
//! this module invents no second spelling for a point type the hand-designed `slic3r-point` crate
//! already owns; it only states the bound a caller's point type must satisfy. This crate carries
//! no dependency on `slic3r-point` (`port/cut.tsv`'s generated `Cargo.toml`), so a caller's own
//! concrete point type is supplied by that caller, not by this module.
//!
//! **`CoordinateFn` stays a bare `Fn(usize, usize) -> C` generic, not a trait**, the direct
//! counterpart of the vendored template's own unconstrained `ACoordinateFn` parameter -- every
//! in-scope instantiation passes a capturing lambda, which a Rust closure represents exactly.
//!
//! **The `VisitorReturnMask` enum class is ported as plain `u32` flag constants, not a Rust
//! `enum`.** Every call site returns zero or more of its three values OR'd together
//! (`descent_mask`, `find_closest_points`'s `Visitor::operator()`, ...) and never compares the
//! bitmask against a single named value; a Rust `enum` cannot be bitwise-OR'd, so the type map's
//! "enum class -> an enum keeping the original's discriminant values" is honoured on the three
//! discriminants themselves (`1`, `2`, `4`, unchanged below) rather than on an enum type that
//! would not support this header's own use of it.

/// `enum : size_t { npos = size_t(-1) };` (`KDTreeIndirect.hpp:34-36`): the sentinel for "no
/// node"/"not found". `size_t(-1)` on a 64-bit `size_t` is `usize::MAX`; no digit literal is
/// transcribed here, so this needs no `// PORT SRC:` marker (R16's constant rule is about a
/// transcribed numeric literal, and `usize::MAX` writes none).
pub const NPOS: usize = usize::MAX;

/// `CONTINUE_LEFT = 1` (`KDTreeIndirect.hpp:20`).
pub const VISITOR_CONTINUE_LEFT: u32 = 1;
/// `CONTINUE_RIGHT = 2` (`KDTreeIndirect.hpp:21`).
pub const VISITOR_CONTINUE_RIGHT: u32 = 2;
/// `STOP = 4` (`KDTreeIndirect.hpp:22`).
// PORT SRC: src/libslic3r/KDTreeIndirect.hpp:22
pub const VISITOR_STOP: u32 = 4;

/// The bound [`KDTreeIndirect`]'s coordinate type needs -- see the module doc's "this module's
/// own bound" note. Implemented for `f32` and `f64`, the two types every in-scope caller
/// instantiates the template with.
pub trait KdCoord:
    Copy
    + PartialOrd
    + std::ops::Sub<Output = Self>
    + std::ops::Mul<Output = Self>
    + std::ops::Add<Output = Self>
{
    /// `CoordType(0)` (`KDTreeIndirect.hpp:74`).
    fn zero() -> Self;
    /// `CoordType(EPSILON)` (`:70`). Reuses `slic3r_libslic3r::libslic3r::EPSILON`
    /// (`55-foundation-libslic3r-execution`'s own committed definition of
    /// `static constexpr double EPSILON = 1e-4;`, `libslic3r.h:57`) rather than writing a second
    /// spelling of the literal, the same "one spelling for the whole port" rule
    /// `docs/PORTING.md`'s "Scaling conversions" section states for `SCALING_FACTOR`.
    fn epsilon() -> Self;
    /// `std::numeric_limits<CoordT>::max()` (`:224`), used by [`find_closest_points`] to seed its
    /// top-`K` result slots with "no candidate yet".
    fn max_value() -> Self;
}

impl KdCoord for f32 {
    fn zero() -> Self {
        // PORT SRC: src/libslic3r/KDTreeIndirect.hpp:74
        0f32
    }
    fn epsilon() -> Self {
        slic3r_libslic3r::libslic3r::EPSILON as f32
    }
    fn max_value() -> Self {
        f32::MAX
    }
}

impl KdCoord for f64 {
    fn zero() -> Self {
        // PORT SRC: src/libslic3r/KDTreeIndirect.hpp:74
        0f64
    }
    fn epsilon() -> Self {
        slic3r_libslic3r::libslic3r::EPSILON
    }
    fn max_value() -> Self {
        f64::MAX
    }
}

/// `template<size_t ANumDimensions, typename ACoordType, typename ACoordinateFn> class
/// KDTreeIndirect` (`KDTreeIndirect.hpp:26-28`). `D` is `ANumDimensions`, `C` is `ACoordType`
/// (bounded by [`KdCoord`], see the module doc), `F` is `ACoordinateFn`.
pub struct KDTreeIndirect<const D: usize, C, F> {
    /// `CoordinateFn coordinate;` (`:84`), public in the vendored class too. Called as
    /// `(self.coordinate)(idx, dimension)` at every use below, Rust's own syntax for invoking a
    /// field of a `Fn`-bounded generic type.
    pub coordinate: F,
    m_nodes: Vec<usize>,
    _coord: std::marker::PhantomData<C>,
}

impl<const D: usize, C, F> KDTreeIndirect<D, C, F>
where
    C: KdCoord,
    F: Fn(usize, usize) -> C,
{
    /// `KDTreeIndirect(CoordinateFn coordinate)` (`:38`).
    pub fn new(coordinate: F) -> Self {
        KDTreeIndirect {
            coordinate,
            m_nodes: Vec::new(),
            _coord: std::marker::PhantomData,
        }
    }

    /// `KDTreeIndirect(CoordinateFn coordinate, std::vector<size_t> indices)` (`:39`).
    pub fn with_indices(coordinate: F, mut indices: Vec<usize>) -> Self {
        let mut tree = Self::new(coordinate);
        tree.build(&mut indices);
        tree
    }

    /// `KDTreeIndirect(CoordinateFn coordinate, size_t num_indices)` (`:40`).
    pub fn with_num_indices(coordinate: F, num_indices: usize) -> Self {
        let mut tree = Self::new(coordinate);
        tree.build_n(num_indices);
        tree
    }

    /// `void clear()` (`:43`).
    pub fn clear(&mut self) {
        self.m_nodes.clear();
    }

    /// `void build(size_t num_indices)` (`:45-52`).
    pub fn build_n(&mut self, num_indices: usize) {
        let mut indices: Vec<usize> = Vec::with_capacity(num_indices);
        for i in 0..num_indices {
            indices.push(i);
        }
        self.build(&mut indices);
    }

    /// `void build(std::vector<size_t> &indices)` (`:54-64`). `indices` is cleared on return,
    /// matching the vendored signature's own out-parameter-by-side-effect (`:63`).
    pub fn build(&mut self, indices: &mut Vec<usize>) {
        if indices.is_empty() {
            self.clear();
        } else {
            // `m_nodes.assign(next_highest_power_of_2(indices.size() + 1), npos);` (`:60`).
            let len = slic3r_utils::utils::next_highest_power_of_2_u64((indices.len() + 1) as u64)
                as usize;
            self.m_nodes = vec![NPOS; len];
            let last = indices.len() - 1;
            self.build_recursive(indices, 0, 0, 0, last);
        }
        indices.clear();
    }

    /// `void build_recursive(...)` (`:88-113`).
    fn build_recursive(
        &mut self,
        input: &mut [usize],
        node: usize,
        dimension: usize,
        left: usize,
        right: usize,
    ) {
        if left > right {
            return;
        }
        debug_assert!(node < self.m_nodes.len());

        if left == right {
            self.m_nodes[node] = input[left];
            return;
        }

        let center = (left + right) / 2;
        self.partition_input(input, dimension, left, right, center);
        self.m_nodes[node] = input[center];

        let mut next_dimension = dimension + 1;
        if next_dimension == D {
            next_dimension = 0;
        }
        if center > left {
            self.build_recursive(input, node * 2 + 1, next_dimension, left, center - 1);
        }
        self.build_recursive(input, node * 2 + 2, next_dimension, center + 1, right);
    }

    /// `void partition_input(...)` (`:119-172`): QuickSelect, translated statement for statement
    /// (including its pre-increment/pre-decrement index walk) since this is exactly the
    /// algorithm whose comparisons must match the oracle's bit for bit, not merely its result.
    fn partition_input(
        &self,
        input: &mut [usize],
        dimension: usize,
        mut left: usize,
        mut right: usize,
        k: usize,
    ) {
        while left < right {
            let center = (left + right) / 2;
            let pivot;
            {
                let mut left_value = (self.coordinate)(input[left], dimension);
                let mut center_value = (self.coordinate)(input[center], dimension);
                let mut right_value = (self.coordinate)(input[right], dimension);
                if left_value > center_value {
                    input.swap(left, center);
                    std::mem::swap(&mut left_value, &mut center_value);
                }
                if left_value > right_value {
                    input.swap(left, right);
                    right_value = left_value;
                }
                if center_value > right_value {
                    input.swap(center, right);
                    center_value = right_value;
                }
                pivot = center_value;
            }
            if right <= left + 2 {
                break;
            }
            let mut i = left;
            let mut j = right - 1;
            input.swap(center, j);
            loop {
                i += 1;
                if !((self.coordinate)(input[i], dimension) < pivot) {
                    break;
                }
            }
            loop {
                j -= 1;
                if !((self.coordinate)(input[j], dimension) > pivot && i < j) {
                    break;
                }
            }
            while i < j {
                input.swap(i, j);
                loop {
                    i += 1;
                    if !((self.coordinate)(input[i], dimension) < pivot) {
                        break;
                    }
                }
                loop {
                    j -= 1;
                    if !((self.coordinate)(input[j], dimension) > pivot && i < j) {
                        break;
                    }
                }
            }
            input.swap(i, right - 1);
            if k < i {
                right = i - 1;
            } else if k == i {
                break;
            } else {
                left = i + 1;
            }
        }
    }

    /// `unsigned int descent_mask(...)` (`:66-75`).
    pub fn descent_mask(
        &self,
        point_coord: C,
        search_radius: C,
        idx: usize,
        dimension: usize,
    ) -> u32 {
        let dist = point_coord - (self.coordinate)(idx, dimension);
        if dist * dist < search_radius + C::epsilon() {
            VISITOR_CONTINUE_LEFT | VISITOR_CONTINUE_RIGHT
        } else if dist > C::zero() {
            VISITOR_CONTINUE_RIGHT
        } else {
            VISITOR_CONTINUE_LEFT
        }
    }

    /// `void visit(Visitor &visitor) const` (`:78-82`).
    pub fn visit(&self, visitor: &mut impl FnMut(usize, usize) -> u32) {
        self.visit_recursive(0, 0, visitor);
    }

    /// `void visit_recursive(...)` (`:174-192`).
    fn visit_recursive(
        &self,
        node: usize,
        dimension: usize,
        visitor: &mut impl FnMut(usize, usize) -> u32,
    ) {
        debug_assert!(!self.m_nodes.is_empty());
        if node >= self.m_nodes.len() || self.m_nodes[node] == NPOS {
            return;
        }

        let left = node * 2 + 1;
        let right = left + 1;
        let mask = visitor(self.m_nodes[node], dimension);
        if mask & VISITOR_STOP == 0 {
            let next_dimension = if dimension + 1 == D { 0 } else { dimension + 1 };
            if mask & VISITOR_CONTINUE_LEFT != 0 {
                self.visit_recursive(left, next_dimension, visitor);
            }
            if mask & VISITOR_CONTINUE_RIGHT != 0 {
                self.visit_recursive(right, next_dimension, visitor);
            }
        }
    }
}

/// `template<size_t K, ...> std::array<size_t, K> find_closest_points(..., FilterFn filter)`
/// (`KDTreeIndirect.hpp:199-257`): the `K` nearest indices under `filter`, nearest first.
/// `PointType` is bounded by `Index<usize, Output = C>` -- see the module doc.
pub fn find_closest_points<const K: usize, P, Filter, const D: usize, C, F>(
    kdtree: &KDTreeIndirect<D, C, F>,
    point: &P,
    mut filter: Filter,
) -> [usize; K]
where
    P: std::ops::Index<usize, Output = C>,
    C: KdCoord,
    F: Fn(usize, usize) -> C,
    Filter: FnMut(usize) -> bool,
{
    let mut results: [(usize, C); K] = [(NPOS, C::max_value()); K];

    {
        let mut visitor = |idx: usize, dimension: usize| -> u32 {
            if filter(idx) {
                let mut dist = C::zero();
                for i in 0..D {
                    let d = point[i] - (kdtree.coordinate)(idx, i);
                    dist = dist + d * d;
                }

                // `std::lower_bound(results.begin(), results.end(), res, cmp)` then
                // `std::rotate(it, std::prev(results.end()), results.end()); *it = res;`
                // (`:235-244`): find the first slot whose distance is not less than `dist`, and
                // insert there, shifting the slots after it one place to the right and dropping
                // the last one. Written as an explicit shift rather than `Vec::rotate_*` because
                // the shift's own direction and the drop of the last element are exactly what
                // must match the oracle's `std::rotate` call, not merely the end result.
                let mut pos = K;
                for (ii, r) in results.iter().enumerate() {
                    if !(r.1 < dist) {
                        pos = ii;
                        break;
                    }
                }
                if pos < K {
                    for ii in (pos + 1..K).rev() {
                        results[ii] = results[ii - 1];
                    }
                    results[pos] = (idx, dist);
                }
            }
            kdtree.descent_mask(point[dimension], results[0].1, idx, dimension)
        };
        kdtree.visit(&mut visitor);
    }

    let mut ret = [NPOS; K];
    for i in 0..K {
        ret[i] = results[i].0;
    }
    ret
}

/// `find_closest_points<K>(kdtree, point)` with no filter (`:259-264`).
pub fn find_closest_points_unfiltered<const K: usize, P, const D: usize, C, F>(
    kdtree: &KDTreeIndirect<D, C, F>,
    point: &P,
) -> [usize; K]
where
    P: std::ops::Index<usize, Output = C>,
    C: KdCoord,
    F: Fn(usize, usize) -> C,
{
    find_closest_points::<K, _, _, D, C, F>(kdtree, point, |_| true)
}

/// `find_closest_point(kdtree, point, filter)` (`:266-276`): `find_closest_points<1>(...)[0]`.
pub fn find_closest_point_filtered<P, Filter, const D: usize, C, F>(
    kdtree: &KDTreeIndirect<D, C, F>,
    point: &P,
    filter: Filter,
) -> usize
where
    P: std::ops::Index<usize, Output = C>,
    C: KdCoord,
    F: Fn(usize, usize) -> C,
    Filter: FnMut(usize) -> bool,
{
    find_closest_points::<1, _, _, D, C, F>(kdtree, point, filter)[0]
}

/// `find_closest_point(kdtree, point)` with no filter (`:278-282`).
pub fn find_closest_point<P, const D: usize, C, F>(
    kdtree: &KDTreeIndirect<D, C, F>,
    point: &P,
) -> usize
where
    P: std::ops::Index<usize, Output = C>,
    C: KdCoord,
    F: Fn(usize, usize) -> C,
{
    find_closest_point_filtered(kdtree, point, |_| true)
}

/// `find_nearby_points(kdtree, center, max_distance, filter)` (`:286-319`): every index within
/// `max_distance` of `center`, in visiting order.
pub fn find_nearby_points_filtered<P, Filter, const D: usize, C, F>(
    kdtree: &KDTreeIndirect<D, C, F>,
    center: &P,
    max_distance: C,
    mut filter: Filter,
) -> Vec<usize>
where
    P: std::ops::Index<usize, Output = C>,
    C: KdCoord,
    F: Fn(usize, usize) -> C,
    Filter: FnMut(usize) -> bool,
{
    let max_distance_squared = max_distance * max_distance;
    let mut result = Vec::new();
    {
        let mut visitor = |idx: usize, dimension: usize| -> u32 {
            if filter(idx) {
                let mut dist = C::zero();
                for i in 0..D {
                    let d = center[i] - (kdtree.coordinate)(idx, i);
                    dist = dist + d * d;
                }
                if dist < max_distance_squared {
                    result.push(idx);
                }
            }
            kdtree.descent_mask(center[dimension], max_distance_squared, idx, dimension)
        };
        kdtree.visit(&mut visitor);
    }
    result
}

/// `find_nearby_points(kdtree, center, max_distance)` with no filter (`:351-358`).
pub fn find_nearby_points<P, const D: usize, C, F>(
    kdtree: &KDTreeIndirect<D, C, F>,
    center: &P,
    max_distance: C,
) -> Vec<usize>
where
    P: std::ops::Index<usize, Output = C>,
    C: KdCoord,
    F: Fn(usize, usize) -> C,
{
    find_nearby_points_filtered(kdtree, center, max_distance, |_| true)
}

/// `visit_nearby_points(kdtree, center, max_distance, visitor_fn)` (`:324-349`): the same search
/// as [`find_nearby_points_filtered`] (with the "always true" filter the vendored overload at
/// `:352-358` also uses, since every in-scope caller of the two-filter form happens to pass
/// none), but handing each index to `visitor_fn` as it is found instead of collecting a `Vec` --
/// the module doc's own words at `:321-323` are why this second entry point exists at all
/// (avoiding a collection allocation the caller usually discards most of).
pub fn visit_nearby_points<P, VisitorFn, const D: usize, C, F>(
    kdtree: &KDTreeIndirect<D, C, F>,
    center: &P,
    max_distance: C,
    mut visitor_fn: VisitorFn,
) where
    P: std::ops::Index<usize, Output = C>,
    C: KdCoord,
    F: Fn(usize, usize) -> C,
    VisitorFn: FnMut(usize),
{
    let max_distance_squared = max_distance * max_distance;
    let mut visitor = |idx: usize, dimension: usize| -> u32 {
        let mut dist = C::zero();
        for i in 0..D {
            let d = center[i] - (kdtree.coordinate)(idx, i);
            dist = dist + d * d;
        }
        if dist < max_distance_squared {
            visitor_fn(idx);
        }
        kdtree.descent_mask(center[dimension], max_distance_squared, idx, dimension)
    };
    kdtree.visit(&mut visitor);
}

/// `find_nearby_points(kdtree, bb_min, bb_max, filter)` (`:361-405`): every index whose point
/// (per `kdtree.coordinate`) lies within the axis-aligned box `[bb_min, bb_max]`. `bb_min`/
/// `bb_max` use C++'s `operator()` (`bb_min(i)`) where the other overloads above use `operator[]`
/// (`point[i]`) -- both collapse to the one `Index` bound the module doc explains.
pub fn find_nearby_points_in_box<P, Filter, const D: usize, C, F>(
    kdtree: &KDTreeIndirect<D, C, F>,
    bb_min: &P,
    bb_max: &P,
    mut filter: Filter,
) -> Vec<usize>
where
    P: std::ops::Index<usize, Output = C>,
    C: KdCoord,
    F: Fn(usize, usize) -> C,
    Filter: FnMut(usize) -> bool,
{
    let mut result = Vec::new();
    let mut visitor = |idx: usize, dimension: usize| -> u32 {
        let mut ret = VISITOR_CONTINUE_LEFT | VISITOR_CONTINUE_RIGHT;

        if filter(idx) {
            let mut contains = true;
            let mut p_dimension = C::zero();
            for i in 0..D {
                let pi = (kdtree.coordinate)(idx, i);
                if i == dimension {
                    p_dimension = pi;
                }
                contains = contains && bb_min[i] <= pi && pi <= bb_max[i];
            }

            if p_dimension < bb_min[dimension] {
                ret = VISITOR_CONTINUE_RIGHT;
            }
            if p_dimension > bb_max[dimension] {
                ret = VISITOR_CONTINUE_LEFT;
            }

            if contains {
                result.push(idx);
            }
        }

        ret
    };
    kdtree.visit(&mut visitor);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use slic3r_proof::{compare_unit, proof_units, Oracle};

    #[test]
    fn build_then_visit_reaches_every_node_exactly_once() {
        let pts: Vec<[f64; 2]> = vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0], [0.5, 0.5]];
        let coordinate = |idx: usize, dim: usize| pts[idx][dim];
        let tree = KDTreeIndirect::<2, f64, _>::with_num_indices(coordinate, pts.len());

        let mut seen = Vec::new();
        let mut visitor = |idx: usize, _dim: usize| -> u32 {
            seen.push(idx);
            VISITOR_CONTINUE_LEFT | VISITOR_CONTINUE_RIGHT
        };
        tree.visit(&mut visitor);
        seen.sort_unstable();
        assert_eq!(seen, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn find_closest_point_returns_the_nearest_unfiltered_index() {
        let pts: Vec<[f64; 2]> = vec![[0.0, 0.0], [10.0, 0.0], [0.0, 10.0], [3.0, 4.0]];
        let coordinate = |idx: usize, dim: usize| pts[idx][dim];
        let tree = KDTreeIndirect::<2, f64, _>::with_num_indices(coordinate, pts.len());
        let actual = find_closest_point(&tree, &[3.5f64, 3.9]);
        assert_eq!(actual, 3);
    }

    /// Proof path 2 (R19, `port/manifest.tsv`'s `KDTreeIndirect.hpp` row). This header-only
    /// template has no committed stage dump that carries its output (it is reached only through
    /// `GCode/SeamPlacer.hpp`/`ShortestPath.cpp`/`Support/SupportSpotsGenerator.cpp`, none of
    /// which is itself in scope today), so -- `docs/PORTING.md`'s "Decisions recorded on
    /// 2026-11-26", decision 2 -- it is proven through its own per-unit bridge instead, the same
    /// route `object_id.hpp`'s `path2_header_observations` and `Execution/Execution.hpp`'s own
    /// `proof path: 2` row already use on `main`. `crates/slic3r-ffi/cxx/proof_units/
    /// kd_tree_indirect.cpp` instantiates the real vendored `Slic3r::KDTreeIndirect<2, double,
    /// ...>` and calls the real vendored `Slic3r::find_closest_point` over the same point set,
    /// compiled by this crate's own `cxx` build with `-ffp-contract=off` (`Oracle::Built`, the
    /// same spelling those two precedents use for a header-only row with nothing linked out of
    /// `just oracle-build`'s own archive).
    // PROOF: 2
    #[test]
    fn kd_tree_indirect_find_closest_point_matches_the_oracle() {
        let pts: Vec<[f64; 2]> = vec![
            [0.0, 0.0],
            [10.0, 0.0],
            [0.0, 10.0],
            [10.0, 10.0],
            [5.0, 5.0],
            [3.0, 4.0],
            [-2.0, 7.0],
        ];
        let coords_flat: Vec<f64> = pts.iter().flat_map(|p| p.iter().copied()).collect();
        let coordinate = |idx: usize, dim: usize| pts[idx][dim];
        let tree = KDTreeIndirect::<2, f64, _>::with_num_indices(coordinate, pts.len());

        let query = [3.5f64, 3.9];
        let actual = find_closest_point(&tree, &query) as u64;
        let oracle = proof_units::kd_tree_indirect::path2_find_closest_point(
            &coords_flat,
            query[0],
            query[1],
        );
        compare_unit("KDTreeIndirect.hpp", Oracle::Built, actual, oracle)
            .expect("find_closest_point must match the vendored template exactly");
    }

    /// Same oracle route as the test above, for [`find_nearby_points`] rather than
    /// [`find_closest_point`] -- not the row's `// PROOF:` marker (one per row is all
    /// `port/port-status` requires), but exercised for real rather than left unverified.
    #[test]
    fn kd_tree_indirect_find_nearby_points_matches_the_oracle() {
        let pts: Vec<[f64; 2]> = vec![
            [0.0, 0.0],
            [10.0, 0.0],
            [0.0, 10.0],
            [10.0, 10.0],
            [5.0, 5.0],
            [3.0, 4.0],
            [-2.0, 7.0],
        ];
        let coords_flat: Vec<f64> = pts.iter().flat_map(|p| p.iter().copied()).collect();
        let coordinate = |idx: usize, dim: usize| pts[idx][dim];
        let tree = KDTreeIndirect::<2, f64, _>::with_num_indices(coordinate, pts.len());

        let center = [1.0f64, 1.0];
        let max_distance = 6.0f64;
        // Not sorted before comparing: `KDTreeIndirect.hpp:310`'s `result.push_back(idx)` inside
        // `visit_recursive` appends in a traversal order the tree layout fully determines
        // (`:184-190`), so the *order* `find_nearby_points` returns is itself part of what this
        // differential holds to exact equality, not only the set of indices.
        let actual: Vec<u64> = find_nearby_points(&tree, &center, max_distance)
            .into_iter()
            .map(|i| i as u64)
            .collect();
        let oracle = proof_units::kd_tree_indirect::path2_find_nearby_points(
            &coords_flat,
            center[0],
            center[1],
            max_distance,
        );
        assert_eq!(
            actual, oracle,
            "find_nearby_points must match the vendored template exactly, including order"
        );
    }
}

// PORT STATUS
// source: src/libslic3r/KDTreeIndirect.hpp
// confidence: medium
// tier: exact
// oracle: built
