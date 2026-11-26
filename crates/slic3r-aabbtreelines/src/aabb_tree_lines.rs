//! Port of `src/libslic3r/AABBTreeLines.hpp` (task `61-foundation-line-slicing`, plan step 20,
//! `port/foundation.tsv` row `foundation-10`): an AABB tree over line segments, built on top of
//! `60-foundation-trees-grids`'s `AABBTreeIndirect::Tree`.
//!
//! **The one instantiation this task can hand-design, and why.** The vendored header is
//! generic over `LineType` (anything with `::Dim`, `::Scalar`, `a`, `b`); this port, like
//! `Point.rs`'s own `Vec2`/`Vec3`/`Vec4` and `line.rs`'s own `line_alg` functions, writes out
//! one concrete instantiation rather than a Rust generic over an arbitrary line shape. The
//! choice of *which* instantiation is forced, not a style preference:
//! `crates/slic3r-aabbtreeindirect/src/aabb_tree_indirect.rs`'s own `TreeCoord` trait --
//! `AlignedBox`/`Tree`'s coordinate bound -- is implemented for `f32` and `f64` only (that
//! task's own module doc: "the two types `Tree2f`/`Tree3f`/`Tree2d`/`Tree3d` actually use"), not
//! for `i64`. This unit's own two integer line types, [`slic3r_line::line::Line`] (2D) and
//! [`slic3r_line::line::Line3`] (3D), both have `Scalar = i64` (`coord_t`) and so cannot
//! instantiate `AABBTreeIndirect::Tree` at all through that crate's module -- a real gap in a
//! sibling foundation task's already-merged crate, which this task may neither edit (it is not
//! this task's own module) nor work around by inventing a competing integer tree type. Of this
//! unit's own two `f64` line types, [`slic3r_line::line::Linef`] (2D) is the one this task
//! hand-designs below; [`slic3r_line::line::Linef3`] (3D) would need `Tree<f64, 3>`, which
//! compiles the same way, but no in-scope caller in either this task's own files or
//! `Linef3`'s own module reaches a 3D line tree, so it is left unreached rather than guessed at
//! a second time with nothing to hold it to. **Named in this task's pull request as a finding**:
//! the production, integer-coordinate instantiation this header mostly exists for
//! (`Lines`-typed callers such as `MotionPlanner`/`SeamPlacer`/Arachne, all fan-out) cannot be
//! completed until `TreeCoord` is extended to `i64` -- either by the sibling task that would
//! need it, or by a regeneration pass, neither of which this task may do itself.
//!
//! **The generic recursive algorithms `detail::squared_distance_to_indexed_primitives_recursive`
//! and `detail::indexed_primitives_within_distance_squared_recurisve` are declared in
//! `AABBTreeIndirect.hpp`, not in this unit's own header** (`:557-637,640-668`) -- the same
//! templates `60-foundation-trees-grids`'s own `aabb_tree_indirect.rs` already instantiates for
//! triangles (as private, triangle-specific functions, not a reusable generic the sibling task
//! exposed). This task instantiates the identical vendored templates a second time, monomorphized
//! for [`LineDistancer`] below, exactly as a second C++ translation unit including the header
//! would -- [`squared_distance_to_indexed_primitives_recursive`]/
//! [`indexed_primitives_within_distance_squared_recursive`].

use slic3r_aabbtreeindirect::aabb_tree_indirect::{AlignedBox, SourceNode, Tree, NPOS};
use slic3r_line::line::{line_distance_to_squared_f64, line_intersection_f64, Linef};
use slic3r_point::point::Vec2d;

/// This unit's one hand-designed instantiation: `AABBTreeIndirect::Tree<2, double>` over
/// [`Linef`] (see the module doc for why `f64`, not the integer `Line`).
pub type LineTree = Tree<f64, 2>;

/// `InputType` (`AABBTreeLines.hpp:180-188`, the unnamed local struct
/// `build_aabb_tree_over_indexed_lines` builds): one [`Linef`]'s index, bounding box and
/// centroid, the `SourceNode` shape `Tree::build` needs.
#[derive(Debug, Clone, Copy)]
struct LineInput {
    idx: usize,
    bbox: AlignedBox<f64, 2>,
    centroid: [f64; 2],
}

impl SourceNode<f64, 2> for LineInput {
    fn idx(&self) -> usize {
        self.idx
    }
    fn centroid(&self) -> [f64; 2] {
        self.centroid
    }
    fn bbox(&self) -> AlignedBox<f64, 2> {
        self.bbox
    }
}

/// `AABBTreeIndirect::Tree<LineType::Dim, typename LineType::Scalar>
/// build_aabb_tree_over_indexed_lines(const std::vector<LineType>& lines)`
/// (`AABBTreeLines.hpp:171-205`).
// PORT SRC: src/libslic3r/AABBTreeLines.hpp:196
pub fn build_aabb_tree_over_indexed_lines(lines: &[Linef]) -> LineTree {
    let mut input: Vec<LineInput> = Vec::with_capacity(lines.len());
    for (i, line) in lines.iter().enumerate() {
        let mut bbox = AlignedBox::from_point([line.a.x(), line.a.y()]);
        bbox.extend_point([line.b.x(), line.b.y()]);
        input.push(LineInput {
            idx: i,
            bbox,
            centroid: [
                (line.a.x() + line.b.x()) * 0.5, // PORT SRC: src/libslic3r/AABBTreeLines.hpp:196
                (line.a.y() + line.b.y()) * 0.5, // PORT SRC: src/libslic3r/AABBTreeLines.hpp:196
            ],
        });
    }
    let mut tree = LineTree::new();
    tree.build(input);
    tree
}

/// `detail::IndexedLinesDistancer<LineType, TreeType, VectorType>::closest_point_to_origin`
/// (`AABBTreeLines.hpp:37-44`): the one `Floating`/`Scalar` cast the vendored body performs is a
/// no-op here (both are `f64`), so it is dropped rather than written out as an identity.
// PORT SRC: src/libslic3r/AABBTreeLines.hpp:42
fn closest_point_to_origin(
    lines: &[Linef],
    primitive_index: usize,
    origin: [f64; 2],
) -> ([f64; 2], f64) {
    let line = lines[primitive_index];
    let (sqr_dist, nearest) =
        line_distance_to_squared_f64(line.a, line.b, Vec2d::new(origin[0], origin[1]));
    ([nearest.x(), nearest.y()], sqr_dist)
}

/// `detail::squared_distance_to_indexed_primitives_recursive` (`AABBTreeIndirect.hpp:556-637`),
/// monomorphized for [`LineDistancer`]'s own `closest_point_to_origin` above -- see the module
/// doc for why this is a second instantiation of that template rather than a call into
/// `60-foundation-trees-grids`'s own (triangle-only, private) copy.
// PORT SRC: src/libslic3r/AABBTreeIndirect.hpp:622
//
// `looked_left`/`looked_right`'s final assignment inside `look_left!()`/`look_right!()` is never
// read back in the branch that runs last (the vendored body itself leaves both the same way,
// `AABBTreeIndirect.hpp:596-634`: each flag only ever gates whether the *other* branch still
// needs to look) -- a harmless dead store, not a bug, so the lint is silenced rather than
// restructured away from the vendored control flow it mirrors (the identical allowance
// `aabb_tree_indirect.rs`'s own triangle instantiation of this same template carries).
#[allow(clippy::too_many_arguments, unused_assignments)]
fn squared_distance_to_indexed_primitives_recursive(
    lines: &[Linef],
    tree: &LineTree,
    origin: [f64; 2],
    node_idx: usize,
    low_sqr_d: f64,
    up_sqr_d: f64,
    best_idx: &mut usize,
    best_point: &mut [f64; 2],
) -> f64 {
    if low_sqr_d > up_sqr_d {
        return low_sqr_d;
    }
    let mut up_sqr_d = up_sqr_d;
    let node = tree.node(node_idx);
    debug_assert!(node.is_valid());
    if node.is_leaf() {
        let (c_candidate, sqr_dist) = closest_point_to_origin(lines, node.idx, origin);
        if sqr_dist < up_sqr_d {
            *best_idx = node.idx;
            *best_point = c_candidate;
            up_sqr_d = sqr_dist;
        }
    } else {
        let left_idx = Tree::<f64, 2>::left_child_idx(node_idx);
        let right_idx = Tree::<f64, 2>::right_child_idx(node_idx);
        let node_left = tree.node(left_idx);
        let node_right = tree.node(right_idx);
        debug_assert!(node_left.is_valid());
        debug_assert!(node_right.is_valid());

        let mut looked_left = false;
        let mut looked_right = false;
        macro_rules! look_left {
            () => {{
                let mut i_left = *best_idx;
                let mut c_left = *best_point;
                let sqr_d_left = squared_distance_to_indexed_primitives_recursive(
                    lines,
                    tree,
                    origin,
                    left_idx,
                    low_sqr_d,
                    up_sqr_d,
                    &mut i_left,
                    &mut c_left,
                );
                if sqr_d_left < up_sqr_d {
                    *best_idx = i_left;
                    *best_point = c_left;
                    up_sqr_d = sqr_d_left;
                }
                looked_left = true;
            }};
        }
        macro_rules! look_right {
            () => {{
                let mut i_right = *best_idx;
                let mut c_right = *best_point;
                let sqr_d_right = squared_distance_to_indexed_primitives_recursive(
                    lines,
                    tree,
                    origin,
                    right_idx,
                    low_sqr_d,
                    up_sqr_d,
                    &mut i_right,
                    &mut c_right,
                );
                if sqr_d_right < up_sqr_d {
                    *best_idx = i_right;
                    *best_point = c_right;
                    up_sqr_d = sqr_d_right;
                }
                looked_right = true;
            }};
        }

        if node_left.bbox.contains_point(origin) {
            look_left!();
        }
        if node_right.bbox.contains_point(origin) {
            look_right!();
        }
        let left_up_sqr_d = node_left.bbox.squared_exterior_distance(origin);
        let right_up_sqr_d = node_right.bbox.squared_exterior_distance(origin);
        if left_up_sqr_d < right_up_sqr_d {
            if !looked_left && left_up_sqr_d < up_sqr_d {
                look_left!();
            }
            if !looked_right && right_up_sqr_d < up_sqr_d {
                look_right!();
            }
        } else {
            if !looked_right && right_up_sqr_d < up_sqr_d {
                look_right!();
            }
            if !looked_left && left_up_sqr_d < up_sqr_d {
                look_left!();
            }
        }
    }
    up_sqr_d
}

/// `detail::indexed_primitives_within_distance_squared_recurisve`
/// (`AABBTreeIndirect.hpp:639-668`); see
/// [`squared_distance_to_indexed_primitives_recursive`]'s doc for why this is a second
/// instantiation of the same shared template.
fn indexed_primitives_within_distance_squared_recursive(
    lines: &[Linef],
    tree: &LineTree,
    origin: [f64; 2],
    node_idx: usize,
    squared_distance_limit: f64,
    found_primitives_indices: &mut Vec<usize>,
) {
    let node = tree.node(node_idx);
    debug_assert!(node.is_valid());
    if node.is_leaf() {
        let (_, sqr_dist) = closest_point_to_origin(lines, node.idx, origin);
        if sqr_dist < squared_distance_limit {
            found_primitives_indices.push(node.idx);
        }
    } else {
        let left_idx = Tree::<f64, 2>::left_child_idx(node_idx);
        let right_idx = Tree::<f64, 2>::right_child_idx(node_idx);
        let node_left = tree.node(left_idx);
        let node_right = tree.node(right_idx);
        debug_assert!(node_left.is_valid());
        debug_assert!(node_right.is_valid());

        if node_left.bbox.squared_exterior_distance(origin) < squared_distance_limit {
            indexed_primitives_within_distance_squared_recursive(
                lines,
                tree,
                origin,
                left_idx,
                squared_distance_limit,
                found_primitives_indices,
            );
        }
        if node_right.bbox.squared_exterior_distance(origin) < squared_distance_limit {
            indexed_primitives_within_distance_squared_recursive(
                lines,
                tree,
                origin,
                right_idx,
                squared_distance_limit,
                found_primitives_indices,
            );
        }
    }
}

/// `squared_distance_to_indexed_lines(...)` (`AABBTreeLines.hpp:210-225`). Returns `(-1, NPOS,
/// [0, 0])` for an empty tree, else `(squared_distance, hit_idx, hit_point)` -- the idiom map's
/// "a non-const reference out-parameter" -> "a return value" rule, folded into one tuple.
///
/// Review finding: this used to initialise (and, on a miss, return) `hit_point` as the query
/// `point`. The vendored function (`:210-225`) never writes its `hit_idx_out`/`hit_point_out`
/// out-parameters unless a hit is recorded (`:220-221`'s empty-tree arm returns `Scalar(-1)`
/// without touching either); the only caller, `distance_from_lines_extra` (`:338-344`),
/// initialises them to `size_t(-1)` and `Vec<Dim,Floating>::Zero()` before the call, so a miss
/// leaves them at the origin, not at the query point.
pub fn squared_distance_to_indexed_lines(
    lines: &[Linef],
    tree: &LineTree,
    point: [f64; 2],
    max_sqr_dist: f64,
) -> (f64, usize, [f64; 2]) {
    if tree.empty() {
        return (-1f64, NPOS, [0f64, 0f64]);
    }
    let mut hit_idx = NPOS;
    let mut hit_point = [0f64, 0f64];
    let sq = squared_distance_to_indexed_primitives_recursive(
        lines,
        tree,
        point,
        0,
        0f64,
        max_sqr_dist,
        &mut hit_idx,
        &mut hit_point,
    );
    (sq, hit_idx, hit_point)
}

/// `all_lines_in_radius(...)` (`AABBTreeLines.hpp:228-243`).
pub fn all_lines_in_radius(
    lines: &[Linef],
    tree: &LineTree,
    point: [f64; 2],
    max_distance_squared: f64,
) -> Vec<usize> {
    if tree.empty() {
        return Vec::new();
    }
    let mut found = Vec::new();
    indexed_primitives_within_distance_squared_recursive(
        lines,
        tree,
        point,
        0,
        max_distance_squared,
        &mut found,
    );
    found
}

/// `detail::coordinate_aligned_ray_hit_count<LineType, TreeType, VectorType, coordinate>`
/// (`AABBTreeLines.hpp:50-123`), with the vendored `coordinate` compile-time template parameter
/// as a runtime `coordinate: usize` (`0` or `1`) -- a compile-time-vs-runtime idiom
/// simplification per `docs/WORKFLOW.md`'s "the simplest translation that keeps the C++
/// meaning", not a behavioural change: the vendored template is instantiated at exactly two
/// concrete values of `coordinate` in this unit (`0` and `1`, from [`point_outside_closed_contours`]),
/// and a runtime branch over the same two values computes the identical result. Returns
/// `(hits_above, hits_below)`, `(-1, -1)` where the vendored body returns `{-1, -1}` (the ray
/// origin on the boundary, or a boundary found in a child).
// PORT SRC: src/libslic3r/AABBTreeLines.hpp:56
fn coordinate_aligned_ray_hit_count(
    lines: &[Linef],
    tree: &LineTree,
    ray_origin: [f64; 2],
    coordinate: usize,
    node_idx: usize,
) -> (i32, i32) {
    let other_coordinate = (coordinate + 1) % 2;
    let node = tree.node(node_idx);
    debug_assert!(node.is_valid());
    if node.is_leaf() {
        let line = lines[node.idx];
        let a = [line.a.x(), line.a.y()];
        let b = [line.b.x(), line.b.y()];
        if ray_origin[other_coordinate] < a[other_coordinate].min(b[other_coordinate])
            || ray_origin[other_coordinate] >= a[other_coordinate].max(b[other_coordinate])
        {
            // The second inequality is non-sharp for a reason: without it, we may count a
            // contour border twice when the lines meet exactly at the spot of intersection.
            // This prevents it.
            return (0, 0);
        }

        let line_max = a[coordinate].max(b[coordinate]);
        let line_min = a[coordinate].min(b[coordinate]);
        if ray_origin[coordinate] > line_max {
            (1, 0)
        } else if ray_origin[coordinate] < line_min {
            (0, 1)
        } else {
            // Find the intersection of the ray with the line: that is when `(line.a + t *
            // (line.b - line.a))[other_coordinate] == ray_origin[other_coordinate]`, `t =
            // (ray_origin[oc] - line.a[oc]) / (line.b[oc] - line.a[oc])`; then we want
            // `val_c = line.a[c] + t * (line.b[c] - line.a[c])`. The ray and line may overlap,
            // when `(line.b[oc] - line.a[oc])` is zero; in that case this returns a negative
            // number.
            let distance_oc = b[other_coordinate] - a[other_coordinate];
            let t = (ray_origin[other_coordinate] - a[other_coordinate]) / distance_oc;
            let val_c = a[coordinate] + t * (b[coordinate] - a[coordinate]);
            if ray_origin[coordinate] > val_c {
                (1, 0)
            } else if ray_origin[coordinate] < val_c {
                (0, 1)
            } else {
                // Ray origin is on the boundary.
                (-1, -1)
            }
        }
    } else {
        let mut intersections_above = 0;
        let mut intersections_below = 0;
        let left_node_idx = Tree::<f64, 2>::left_child_idx(node_idx);
        let right_node_idx = Tree::<f64, 2>::right_child_idx(node_idx);
        let node_left = tree.node(left_node_idx);
        let node_right = tree.node(right_node_idx);
        debug_assert!(node_left.is_valid());
        debug_assert!(node_right.is_valid());

        let left_min = node_left.bbox.min();
        let left_max = node_left.bbox.max();
        if left_min[other_coordinate] <= ray_origin[other_coordinate]
            && left_max[other_coordinate] >= ray_origin[other_coordinate]
        {
            let (above, below) = coordinate_aligned_ray_hit_count(
                lines,
                tree,
                ray_origin,
                coordinate,
                left_node_idx,
            );
            if above < 0 || below < 0 {
                return (-1, -1);
            }
            intersections_above += above;
            intersections_below += below;
        }

        let right_min = node_right.bbox.min();
        let right_max = node_right.bbox.max();
        if right_min[other_coordinate] <= ray_origin[other_coordinate]
            && right_max[other_coordinate] >= ray_origin[other_coordinate]
        {
            let (above, below) = coordinate_aligned_ray_hit_count(
                lines,
                tree,
                ray_origin,
                coordinate,
                right_node_idx,
            );
            if above < 0 || below < 0 {
                return (-1, -1);
            }
            intersections_above += above;
            intersections_below += below;
        }
        (intersections_above, intersections_below)
    }
}

/// `point_outside_closed_contours(...)` (`AABBTreeLines.hpp:248-273`): `1` outside, `-1` inside,
/// `0` on the contour or undeterminable.
pub fn point_outside_closed_contours(lines: &[Linef], tree: &LineTree, point: [f64; 2]) -> i32 {
    if tree.empty() {
        return 1;
    }
    let (hits_above, hits_below) = coordinate_aligned_ray_hit_count(lines, tree, point, 0, 0);
    if hits_above < 0 || hits_below < 0 {
        return 0;
    }
    if hits_above % 2 == 1 && hits_below % 2 == 1 {
        return -1;
    }
    if hits_above % 2 == 0 && hits_below % 2 == 0 {
        return 1;
    }
    // This should not happen with closed contours; check it in the Y direction.
    let (hits_above, hits_below) = coordinate_aligned_ray_hit_count(lines, tree, point, 1, 0);
    if hits_above < 0 || hits_below < 0 {
        return 0;
    }
    if hits_above % 2 == 1 && hits_below % 2 == 1 {
        return -1;
    }
    if hits_above % 2 == 0 && hits_below % 2 == 0 {
        return 1;
    }
    // Both results were unclear.
    0
}

/// `detail::get_intersections_with_line<LineType, TreeType, VectorType>`
/// (`AABBTreeLines.hpp:125-163`).
fn get_intersections_with_line_recursive(
    lines: &[Linef],
    tree: &LineTree,
    line: Linef,
    line_bb: AlignedBox<f64, 2>,
    node_idx: usize,
) -> Vec<([f64; 2], usize)> {
    let node = tree.node(node_idx);
    debug_assert!(node.is_valid());
    if node.is_leaf() {
        match line_intersection_f64(line.a, line.b, lines[node.idx].a, lines[node.idx].b) {
            Some(p) => vec![([p.x(), p.y()], node.idx)],
            None => Vec::new(),
        }
    } else {
        let left_node_idx = Tree::<f64, 2>::left_child_idx(node_idx);
        let right_node_idx = Tree::<f64, 2>::right_child_idx(node_idx);
        let node_left = tree.node(left_node_idx);
        let node_right = tree.node(right_node_idx);
        debug_assert!(node_left.is_valid());
        debug_assert!(node_right.is_valid());

        let mut result = Vec::new();
        if node_left.bbox.intersects(&line_bb) {
            result.extend(get_intersections_with_line_recursive(
                lines,
                tree,
                line,
                line_bb,
                left_node_idx,
            ));
        }
        if node_right.bbox.intersects(&line_bb) {
            result.extend(get_intersections_with_line_recursive(
                lines,
                tree,
                line,
                line_bb,
                right_node_idx,
            ));
        }
        result
    }
}

/// `get_intersections_with_line<sorted, VectorType, LineType, TreeType>(...)`
/// (`AABBTreeLines.hpp:275-304`), with the vendored `sorted` compile-time template parameter as
/// a runtime `sorted: bool` -- see [`coordinate_aligned_ray_hit_count`]'s doc for why this is the
/// same idiom simplification, not a behavioural change.
pub fn get_intersections_with_line(
    lines: &[Linef],
    tree: &LineTree,
    line: Linef,
    sorted: bool,
) -> Vec<([f64; 2], usize)> {
    if tree.empty() {
        return Vec::new();
    }
    let mut line_bb = AlignedBox::from_point([line.a.x(), line.a.y()]);
    line_bb.extend_point([line.b.x(), line.b.y()]);

    let mut intersections = get_intersections_with_line_recursive(lines, tree, line, line_bb, 0);
    if sorted {
        // FIDELITY(port): src/libslic3r/AABBTreeLines.hpp:295-297 -- `std::sort` (no stability
        // guarantee between equal elements) is `slice::sort_unstable_by`, per
        // `docs/PORTING.md`'s idiom map (`std::sort` -> `sort_unstable`, reserving plain `sort`
        // for `std::stable_sort`). The comparator itself (`left.first < right.first`) is not a
        // total order on `f64` (NaN is incomparable). `AABBTreeLines.hpp:295` has no row at all
        // in `port/reports/order-comparators.md` -- that report classifies no site in this
        // header -- so per `docs/PORTING.md`'s banned-substitution row for comparators, where
        // 85 classified sites "are still `undecided` and are judged per-site during porting",
        // this site is the porter's own per-site judgement, not a reading of the report.
        // The deliberate choice made here is `partial_cmp(..).unwrap_or(Ordering::Equal)`: a
        // total-order comparator substituted for the vendored non-total `left.first <
        // right.first`, which is the substitution this marker exists to record.
        intersections.sort_unstable_by(|left, right| {
            let dl = [left.0[0] - line.a.x(), left.0[1] - line.a.y()];
            let dr = [right.0[0] - line.a.x(), right.0[1] - line.a.y()];
            let sq_l = dl[0] * dl[0] + dl[1] * dl[1];
            let sq_r = dr[0] * dr[0] + dr[1] * dr[1];
            sq_l.partial_cmp(&sq_r).unwrap_or(std::cmp::Ordering::Equal)
        });
    }
    intersections
}

/// `class LinesDistancer<LineType>` (`AABBTreeLines.hpp:306-374`).
pub struct LinesDistancer {
    lines: Vec<Linef>,
    tree: LineTree,
}

impl LinesDistancer {
    /// `explicit LinesDistancer(const std::vector<LineType>& lines)`/`explicit
    /// LinesDistancer(std::vector<LineType>&& lines)` (`:317-327`): Rust has no by-value/by-ref
    /// overload pair, so this takes ownership either way, the same shape `cxx`-free Rust code
    /// uses throughout this port.
    pub fn new(lines: Vec<Linef>) -> Self {
        let tree = build_aabb_tree_over_indexed_lines(&lines);
        LinesDistancer { lines, tree }
    }

    // TODO(port): src/libslic3r/AABBTreeLines.hpp:329 -- `LinesDistancer() = default`, an
    // empty-lines/empty-tree default constructor. No in-scope caller default-constructs a
    // `LinesDistancer` rather than building one from a `Vec<Linef>` (an empty `Vec` through
    // [`LinesDistancer::new`] above already yields the same empty tree a default construction
    // would), so this gets no separate `fn` of its own.

    /// `int outside(const Vec<Dim,Scalar>& point) const` (`:332`).
    pub fn outside(&self, point: [f64; 2]) -> i32 {
        point_outside_closed_contours(&self.lines, &self.tree, point)
    }

    /// `std::tuple<Floating, size_t, Vec<Dim,Floating>>
    /// distance_from_lines_extra<SIGNED_DISTANCE>(...) const` (`:335-353`), with the vendored
    /// `SIGNED_DISTANCE` compile-time template parameter as a runtime `signed_distance: bool` --
    /// see [`coordinate_aligned_ray_hit_count`]'s doc for why this is the same idiom
    /// simplification, not a behavioural change.
    pub fn distance_from_lines_extra(
        &self,
        point: [f64; 2],
        signed_distance: bool,
    ) -> (f64, usize, [f64; 2]) {
        let (sqr_distance, idx, nearest) =
            squared_distance_to_indexed_lines(&self.lines, &self.tree, point, f64::INFINITY);
        if sqr_distance < 0f64 {
            return (f64::INFINITY, idx, nearest);
        }
        let mut distance = sqr_distance.sqrt();
        if signed_distance {
            distance *= self.outside(point) as f64;
        }
        (distance, idx, nearest)
    }

    /// `Floating distance_from_lines<SIGNED_DISTANCE>(...) const` (`:355-359`).
    pub fn distance_from_lines(&self, point: [f64; 2], signed_distance: bool) -> f64 {
        self.distance_from_lines_extra(point, signed_distance).0
    }

    /// `std::vector<size_t> all_lines_in_radius(...) const` (`:361-364`): the vendored body
    /// squares `radius` itself (`radius * radius`), so this takes the same un-squared `radius`.
    pub fn all_lines_in_radius(&self, point: [f64; 2], radius: f64) -> Vec<usize> {
        all_lines_in_radius(&self.lines, &self.tree, point, radius * radius)
    }

    /// `std::vector<std::pair<Vec<Dim,Scalar>, size_t>> intersections_with_line<sorted>(...)
    /// const` (`:366-369`).
    pub fn intersections_with_line(&self, line: Linef, sorted: bool) -> Vec<([f64; 2], usize)> {
        get_intersections_with_line(&self.lines, &self.tree, line, sorted)
    }

    /// `const LineType& get_line(size_t line_idx) const` (`:371`).
    pub fn get_line(&self, line_idx: usize) -> Linef {
        self.lines[line_idx]
    }

    /// `const std::vector<LineType>& get_lines() const` (`:373`).
    pub fn get_lines(&self) -> &[Linef] {
        &self.lines
    }
}

// TODO(port): src/libslic3r/AABBTreeLines.hpp -- the integer-coordinate
// (`Line`/`Line3`) instantiation of every function above; needs `AABBTreeIndirect::Tree`'s
// `TreeCoord` bound extended to `i64`, a gap in a sibling foundation task's already-merged
// module this task may not edit (see the module doc).

#[cfg(test)]
mod tests {
    use super::*;
    use slic3r_point::point::Vec2d;

    fn square_lines() -> Vec<Linef> {
        vec![
            Linef::new(Vec2d::new(0f64, 0f64), Vec2d::new(10.0, 0f64)),
            Linef::new(Vec2d::new(10.0, 0f64), Vec2d::new(10.0, 10.0)),
            Linef::new(Vec2d::new(10.0, 10.0), Vec2d::new(0f64, 10.0)),
            Linef::new(Vec2d::new(0f64, 10.0), Vec2d::new(0f64, 0f64)),
        ]
    }

    #[test]
    fn point_outside_closed_contours_is_negative_one_for_a_point_inside_a_square() {
        let lines = square_lines();
        let tree = build_aabb_tree_over_indexed_lines(&lines);
        assert_eq!(point_outside_closed_contours(&lines, &tree, [5.0, 5.0]), -1);
    }

    #[test]
    fn point_outside_closed_contours_is_one_for_a_point_outside_a_square() {
        let lines = square_lines();
        let tree = build_aabb_tree_over_indexed_lines(&lines);
        assert_eq!(
            point_outside_closed_contours(&lines, &tree, [20.0, 20.0]),
            1
        );
    }

    #[test]
    fn squared_distance_to_indexed_lines_finds_the_nearest_edge() {
        let lines = square_lines();
        let tree = build_aabb_tree_over_indexed_lines(&lines);
        let (sq, idx, nearest) =
            squared_distance_to_indexed_lines(&lines, &tree, [5.0, -3.0], f64::INFINITY);
        assert_eq!(sq, 9.0);
        assert_eq!(idx, 0);
        assert_eq!(nearest, [5.0, 0f64]);
    }

    /// Review finding: an empty tree's miss used to leave `hit_point` at the query point
    /// instead of the oracle's own `Vec::Zero()` (see [`squared_distance_to_indexed_lines`]'s
    /// doc). Proves both the free function and `distance_from_lines_extra`, which propagates
    /// the same miss out of its own `pub` surface.
    #[test]
    fn squared_distance_to_indexed_lines_on_an_empty_tree_yields_the_oracles_zero_not_the_query_point(
    ) {
        let lines: Vec<Linef> = Vec::new();
        let tree = build_aabb_tree_over_indexed_lines(&lines);
        assert!(tree.empty());
        let (sq, idx, nearest) =
            squared_distance_to_indexed_lines(&lines, &tree, [5.0, -3.0], f64::INFINITY);
        assert_eq!(sq, -1f64);
        assert_eq!(idx, NPOS);
        assert_eq!(nearest, [0f64, 0f64]);

        let distancer = LinesDistancer::new(lines);
        let (distance, idx, nearest) = distancer.distance_from_lines_extra([5.0, -3.0], true);
        assert_eq!(distance, f64::INFINITY);
        assert_eq!(idx, NPOS);
        assert_eq!(nearest, [0f64, 0f64]);
    }

    #[test]
    fn lines_distancer_signed_distance_is_negative_inside() {
        let distancer = LinesDistancer::new(square_lines());
        let (distance, _, _) = distancer.distance_from_lines_extra([5.0, 5.0], true);
        assert!(
            distance < 0f64,
            "a point well inside a closed contour has a negative signed distance"
        );
    }

    /// Proof path 2 (R19, `port/manifest.tsv`'s `AABBTreeLines.hpp` row): no committed stage
    /// dump carries this header's own output (its callers are all fan-out modules not yet
    /// written), so -- `docs/PORTING.md`'s "Decisions recorded on 2026-11-26", decision 2 --
    /// this is proven through its own per-unit bridge instead, the same route
    /// `AABBTreeIndirect.hpp`'s own precedent (`60-foundation-trees-grids`'s
    /// `aabb_tree_indirect_matches_the_oracle`) uses: `crates/slic3r-ffi/cxx/proof_units/
    /// aabb_tree_lines.cpp` builds the real vendored `Slic3r::AABBTreeLines::LinesDistancer<
    /// Slic3r::Linef>` over the same lines this test builds its own tree from, and calls its
    /// real vendored `distance_from_lines_extra<true>` and `outside`.
    // PROOF: 2
    #[test]
    fn aabb_tree_lines_matches_the_oracle() {
        let lines = square_lines();
        let ax: Vec<f64> = lines.iter().map(|l| l.a.x()).collect();
        let ay: Vec<f64> = lines.iter().map(|l| l.a.y()).collect();
        let bx: Vec<f64> = lines.iter().map(|l| l.b.x()).collect();
        let by: Vec<f64> = lines.iter().map(|l| l.b.y()).collect();

        let distancer = LinesDistancer::new(lines);
        let query = [3.0, -4.0];
        let (actual_distance, actual_idx, actual_point) =
            distancer.distance_from_lines_extra(query, true);
        let actual_outside = distancer.outside(query);

        let oracle = slic3r_proof::proof_units::aabb_tree_lines::path2_distance_and_outside(
            &ax, &ay, &bx, &by, query[0], query[1],
        );
        slic3r_proof::tiers::compare_unit(
            "AABBTreeLines.hpp",
            slic3r_proof::tiers::Oracle::Built,
            actual_distance,
            oracle[0],
        )
        .expect("LinesDistancer::distance_from_lines_extra<true>'s distance must match the oracle exactly");
        assert_eq!(
            actual_idx as f64, oracle[1],
            "the nearest-line index must match the oracle"
        );
        assert_eq!(
            actual_point[0], oracle[2],
            "the nearest point's x must match the oracle"
        );
        assert_eq!(
            actual_point[1], oracle[3],
            "the nearest point's y must match the oracle"
        );
        assert_eq!(
            actual_outside as f64, oracle[4],
            "LinesDistancer::outside must match the oracle"
        );
    }

    /// `LinesDistancer::all_lines_in_radius`/`intersections_with_line<true>` -- added on
    /// review: the first version of this test proved only `distance_from_lines_extra`/
    /// `outside`, leaving these two ported-but-unproven.
    #[test]
    fn aabb_tree_lines_radius_and_intersections_match_the_oracle() {
        let lines = square_lines();
        let ax: Vec<f64> = lines.iter().map(|l| l.a.x()).collect();
        let ay: Vec<f64> = lines.iter().map(|l| l.a.y()).collect();
        let bx: Vec<f64> = lines.iter().map(|l| l.b.x()).collect();
        let by: Vec<f64> = lines.iter().map(|l| l.b.y()).collect();

        let distancer = LinesDistancer::new(lines);

        // Review finding: this used to sort both sides before comparing. The vendored
        // `all_lines_in_radius` has no sort anywhere in its own call chain (see the proof
        // shim's own updated doc), so its returned order is part of the `exact`-tier output;
        // comparing unsorted is what the tier asks for.
        let actual_radius = distancer.all_lines_in_radius([5.0, -1.0], 2.0);
        let oracle_radius = slic3r_proof::proof_units::aabb_tree_lines::path2_all_lines_in_radius(
            &ax, &ay, &bx, &by, 5.0, -1.0, 2.0,
        );
        assert_eq!(
            actual_radius.iter().map(|&i| i as i64).collect::<Vec<_>>(),
            oracle_radius,
            "LinesDistancer::all_lines_in_radius must match the oracle exactly, in traversal order"
        );

        // A probe line crossing two sides of the square at different distances from its own
        // `a` endpoint, so the sorted order is unambiguous (no tie for `std::sort`/`sort_by` to
        // break differently).
        let probe = Linef::new(Vec2d::new(-5.0, 3.0), Vec2d::new(15.0, 3.0));
        let actual_hits = distancer.intersections_with_line(probe, true);
        let oracle_hits = slic3r_proof::proof_units::aabb_tree_lines::path2_intersections_with_line(
            &ax,
            &ay,
            &bx,
            &by,
            probe.a.x(),
            probe.a.y(),
            probe.b.x(),
            probe.b.y(),
        );
        let actual_flat: Vec<f64> = actual_hits
            .iter()
            .flat_map(|(p, idx)| [p[0], p[1], *idx as f64])
            .collect();
        assert_eq!(
            actual_flat, oracle_hits,
            "LinesDistancer::intersections_with_line<true> must match the oracle exactly"
        );
    }
}

// PORT STATUS
// source: src/libslic3r/AABBTreeLines.hpp
// confidence: low
// tier: exact
// oracle: built
