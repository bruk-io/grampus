//! Port of `src/libslic3r/AABBTreeIndirect.hpp` (task `60-foundation-trees-grids`, plan step 20,
//! `foundation-09` row of `port/foundation.tsv`): a static balanced AABB tree over an externally
//! owned, indexed entity set (triangles, segments, points, ...), for raycasting and
//! closest-entity search. `template-heavy: yes` in `port/manifest.tsv`.
//!
//! **`Eigen::AlignedBox<CoordType, NumDimensions>` becomes this module's own [`AlignedBox`],
//! not a borrow of `slic3r-boundingbox`'s types.** That crate (`58-foundation-bounding-box`)
//! hand-designs four *concrete* structs for the vendored `BoundingBoxBase<PointType,
//! APointsType>`/`BoundingBox3Base<PointType>` templates -- `Slic3r::BoundingBox`,
//! `BoundingBoxf`, `BoundingBoxf3` and `BoundingBox3`, none of them generic over a dimension
//! count or a coordinate type. `Tree<ANumDimensions, ACoordType>` needs exactly that generality
//! (`Tree2f`/`Tree3f`/`Tree2d`/`Tree3d` below, all real in-scope instantiations -- see the next
//! paragraph), which is `Eigen::AlignedBox`'s own template, not `Slic3r::BoundingBox`'s. [`AlignedBox`]
//! is this task's own hand-design of that different, more general template -- the crate map
//! names no other owner for `Eigen::AlignedBox<CoordType, NumDimensions>` generically, only for
//! libslic3r's own concrete `BoundingBoxBase` instantiations.
//!
//! **Not collapsed to one instantiation.** `Tree2f`/`Tree3f`/`Tree2d`/`Tree3d` (`:221-224`) are
//! all real type aliases at the pin, and `build_aabb_tree_over_indexed_triangle_set` (`:677`)
//! produces `Tree<3, typename VertexType::Scalar>` for whichever `VertexType` its caller passes
//! -- so [`Tree`] stays generic over both the dimension count (`const D: usize`) and the
//! coordinate type (bounded by [`TreeCoord`], implemented for `f32` and `f64`, the two
//! coordinate types every alias above uses).
//!
//! **`VertexType`/`IndexedFaceType`/`VectorType` collapse to one local [`Vec3`], generic over
//! [`TreeCoord`], rather than three independent template parameters.** The vendored template
//! leaves these three fully independent (a caller can raycast a `float` mesh at `double`
//! accuracy by choosing a different `VectorType` than `VertexType`), but no crate this task may
//! depend on (`port/cut.tsv`'s generated `Cargo.toml`: `slic3r-boundingbox`, `slic3r-libslic3r`,
//! `slic3r-utils` -- no `slic3r-point`, no `slic3r-trianglemesh`) names a concrete vertex or
//! indexed-face type to parameterise over in the first place, so a from-scratch three-parameter
//! design would be designing against nothing. [`Vec3<C>`] is this task's minimal, dependency-free
//! stand-in -- a plain 3-component vector generic in `C: TreeCoord`, with the handful of
//! operations (`-`, `dot`, `cross`, indexing, an `f64` cast) the ray/distance queries below
//! actually use, so a Phase A/B caller with a real vertex type satisfies the same bound rather
//! than being forced through this one.
//!
//! **The four `intersect_triangle` SFINAE overloads (`:319-386`) collapse to one function that
//! always computes in `f64`.** Read together, the four overloads are not four different
//! computations: `(double, double)` computes in `f64` directly; the other three each cast
//! *whichever side is not already `double`* to `f64` and then run the identical body. So every
//! call, through every overload, computes in `f64` -- the SFINAE split exists only to skip a
//! redundant cast when a side is already `double`, never to change a value. Casting an already-
//! `f64` operand to `f64` is the identity, so always converting both `V`/`W` to `f64` first (this
//! module's single [`intersect_triangle`]) reproduces every one of the four overloads' outputs
//! bit for bit; this is a simplification of *which code path runs*, not of *what is computed*,
//! and needs no `// FIDELITY(port):` marker because nothing a comparison could observe differs.
//!
//! **FIDELITY(port) -- see the `// FIDELITY(port):` markers on `closest_point_on_triangle_to_origin`
//! and `ray_box_intersect_invdir`'s callers below. The closest-point/distance path
//! (`squared_distance_to_indexed_triangle_set`,
//! `is_any_triangle_in_radius`, `all_triangles_in_radius`, and the box test and accumulation
//! inside their shared recursion) is forced to `f64` throughout, where the vendored template
//! keeps it independent of the query point's own type.** `:536`'s `using ScalarType = typename
//! VectorType::Scalar` (with `:547-550`) runs `closest_point_to_triangle` in `VectorType::Scalar`,
//! not in `VertexType::Scalar`; `:617`'s `distancer.origin.template cast<BBoxScalar>()` *narrows*
//! the query origin to the tree's own `BoundingBox::Scalar`, and `:622`'s
//! `squaredExteriorDistance` returns and accumulates in that same box scalar (Eigen's
//! `AlignedBox::squaredExteriorDistance`), not in `f64`. Three in-scope callers instantiate this
//! independently of `f64`: `GCode/SeamPlacer.cpp:379`'s `is_any_triangle_in_radius(enforcers.vertices,
//! enforcers.indices, enforcers_tree, position /* Vec3f */, radius_sqr /* float */)` runs the
//! whole path in `float`, and `SLA/IndexedMesh.cpp:76`/`AABBMesh.cpp:69` each call
//! `squared_distance_to_indexed_triangle_set` with `std::vector<Vec3f>` vertices against a
//! `Vec3d` query point -- the independent-`VectorType` shape `:797-798`'s own comment names
//! ("performed with the accuracy of `VectorType::Scalar` even if the triangle mesh and the AABB
//! Tree are built with floats"). This module's single [`Vec3<C>`] collapses the vertex type and
//! the query-point type to the same `C`, so that call shape cannot be expressed at all, and every
//! query through this path -- including one built over `Tree2f`/`Tree3f` -- always computes in
//! `f64`. The justification two paragraphs up (the `intersect_triangle` SFINAE collapse) does
//! **not** cover this path: those four overloads really do all compute in `double`; the
//! closest-point/distance path's vendored `Scalar` is `float` whenever `VectorType::Scalar` is,
//! and this module silently widens it. **This is not held to exact equality against the oracle
//! for a `Tree2f`/`Tree3f` (all-`f32`) query**: `aabb_tree_indirect_matches_the_oracle` only
//! builds `Eigen::Vector3d`/`Tree<3, double>` (`crates/slic3r-ffi/cxx/proof_units/
//! aabb_tree_indirect.cpp`), so nothing above is exercised by this task's proof. Reported as a
//! finding in this task's pull request rather than redesigned here: fixing it needs `Scalar` to
//! vary independently of the vertex/query-point type, which is a second generic parameter this
//! task did not have time to thread through every call site under review.
//!
//! **`BoundingBoxWrapper` (`:228-241`) is not ported.** It wraps `Slic3r::BoundingBox` (2D,
//! `coord_t`) and constructs from a `Point` inflated by `SCALED_EPSILON` -- `Point`,
//! `Slic3r::BoundingBox` and `SCALED_EPSILON` are `slic3r-point`/`slic3r-boundingbox`/
//! `slic3r-libslic3r` surface this crate has no dependency on naming the first of (and a port
//! task may add none). **Named as a finding in this task's pull request**: until a regeneration
//! pass adds `slic3r-point` to this crate's dependency list, no task can port this type at all.
//! Nothing else in this header needs it -- `build_aabb_tree_over_indexed_triangle_set` and every
//! ray/distance query are generic over `VertexType`/`VectorType` and never mention
//! `BoundingBoxWrapper` or `Point` themselves.
//!
//! **`get_candidate_idxs`/`Intersecting`/`Within`/`traverse` (`:884-992`) need only
//! [`AlignedBox`]/[`Tree`]**, no vector algebra at all, and are ported exactly as generic as the
//! vendored template.

/// The bound [`AlignedBox`]/[`Tree`]'s coordinate type needs. Implemented for `f32` and `f64`,
/// the two types `Tree2f`/`Tree3f`/`Tree2d`/`Tree3d` (`:221-224`) actually use.
pub trait TreeCoord:
    Copy
    + PartialOrd
    + std::ops::Add<Output = Self>
    + std::ops::Sub<Output = Self>
    + std::ops::Mul<Output = Self>
    + std::ops::Neg<Output = Self>
{
    fn zero() -> Self;
    fn infinity() -> Self;
    fn to_f64(self) -> f64;
    fn from_f64(v: f64) -> Self;
    fn min(self, other: Self) -> Self {
        if self < other {
            self
        } else {
            other
        }
    }
    fn max(self, other: Self) -> Self {
        if self > other {
            self
        } else {
            other
        }
    }
}

impl TreeCoord for f32 {
    fn zero() -> Self {
        0f32
    }
    fn infinity() -> Self {
        f32::INFINITY
    }
    fn to_f64(self) -> f64 {
        self as f64
    }
    fn from_f64(v: f64) -> Self {
        v as f32
    }
}

impl TreeCoord for f64 {
    fn zero() -> Self {
        0f64
    }
    fn infinity() -> Self {
        f64::INFINITY
    }
    fn to_f64(self) -> f64 {
        self
    }
    fn from_f64(v: f64) -> Self {
        v
    }
}

/// `Eigen::AlignedBox<CoordType, NumDimensions>` -- see the module doc's "becomes this module's
/// own `AlignedBox`" note.
#[derive(Debug, Clone, Copy)]
pub struct AlignedBox<C: TreeCoord, const D: usize> {
    min: [C; D],
    max: [C; D],
}

impl<C: TreeCoord, const D: usize> AlignedBox<C, D> {
    /// `AlignedBox(const VectorType &p)`: a box containing exactly one point.
    pub fn from_point(p: [C; D]) -> Self {
        AlignedBox { min: p, max: p }
    }

    pub fn min(&self) -> [C; D] {
        self.min
    }
    pub fn max(&self) -> [C; D] {
        self.max
    }

    /// `AlignedBox::extend(const VectorType &p)`.
    pub fn extend_point(&mut self, p: [C; D]) {
        for i in 0..D {
            self.min[i] = self.min[i].min(p[i]);
            self.max[i] = self.max[i].max(p[i]);
        }
    }

    /// `AlignedBox::extend(const AlignedBox &b)`.
    pub fn extend_box(&mut self, b: &AlignedBox<C, D>) {
        self.extend_point(b.min);
        self.extend_point(b.max);
    }

    /// `AlignedBox::contains(const VectorType &p)`.
    pub fn contains_point(&self, p: [C; D]) -> bool {
        (0..D).all(|i| self.min[i] <= p[i] && p[i] <= self.max[i])
    }

    /// `AlignedBox::contains(const AlignedBox &b)`.
    pub fn contains_box(&self, b: &AlignedBox<C, D>) -> bool {
        self.contains_point(b.min) && self.contains_point(b.max)
    }

    /// `AlignedBox::intersects(const AlignedBox &b)`.
    pub fn intersects(&self, b: &AlignedBox<C, D>) -> bool {
        (0..D).all(|i| self.min[i] <= b.max[i] && b.min[i] <= self.max[i])
    }

    /// `AlignedBox::diagonal()`.
    pub fn diagonal(&self) -> [C; D] {
        let mut d = [C::zero(); D];
        for i in 0..D {
            d[i] = self.max[i] - self.min[i];
        }
        d
    }

    /// `AlignedBox::diagonal().maxCoeff(&dimension)`: the dimension of the longest edge.
    pub fn longest_axis(&self) -> usize {
        let d = self.diagonal();
        let mut best = 0;
        for i in 1..D {
            if d[i] > d[best] {
                best = i;
            }
        }
        best
    }

    /// `AlignedBox::squaredExteriorDistance(const VectorType &p)`: `0` where `p` is inside the
    /// box on that axis, else the squared distance to the nearest face.
    pub fn squared_exterior_distance(&self, p: [C; D]) -> C {
        let mut acc = C::zero();
        for i in 0..D {
            let d = if p[i] < self.min[i] {
                self.min[i] - p[i]
            } else if p[i] > self.max[i] {
                p[i] - self.max[i]
            } else {
                C::zero()
            };
            acc = acc + d * d;
        }
        acc
    }

    /// `AlignedBox::cast<Scalar>()`, narrowed to the one target precision the ray/distance
    /// queries below need (see the module doc's `intersect_triangle` note for why `f64` is the
    /// one precision this port's query path ever casts to).
    pub fn to_f64(&self) -> AlignedBox<f64, D> {
        let mut min = [0f64; D];
        let mut max = [0f64; D];
        for i in 0..D {
            min[i] = self.min[i].to_f64();
            max[i] = self.max[i].to_f64();
        }
        AlignedBox { min, max }
    }
}

/// `struct Node` (`:61-77`).
#[derive(Debug, Clone, Copy)]
pub struct Node<C: TreeCoord, const D: usize> {
    pub idx: usize,
    pub bbox: AlignedBox<C, D>,
}

/// `size_t(-1)` (`:54`).
pub const NPOS: usize = usize::MAX;
/// `size_t(-2)` (`:56`).
pub const INNER: usize = usize::MAX - 1;

impl<C: TreeCoord, const D: usize> Node<C, D> {
    /// `Node()` (`:61-63`, default-constructed): `idx = npos` is explicit (`:63`), but `BoundingBox
    /// bbox;` (`:66`) default-constructs `Eigen::AlignedBox`, which Eigen initialises *empty*
    /// (`min = +infinity`, `max = -infinity` componentwise), not a zero-sized box at the origin --
    /// the value this port's own `AlignedBox::from_point([C::zero(); D])` gives instead. **Found
    /// on review and deliberately not matched**: `m_nodes.assign(..., Node())` (`build_modify_input`,
    /// `:104`) pre-fills every slot, including the ones a non-power-of-2 leaf count's last level
    /// never writes; those slots' `idx` stays `npos` (`is_valid()` false), and every reader
    /// (`traverse_recurse`, `intersect_ray_recursive_first_hit`/`_all_hits`,
    /// `squared_distance_to_indexed_primitives_recursive`, `get_candidate_idxs`) reaches a node only
    /// by descending into a specific child index of a node it already found `is_inner()`, which
    /// `build_recursive` guarantees was itself written -- an unwritten slot is therefore never
    /// visited by any of them, on either side, so its `bbox` value is read by nothing. A degenerate
    /// point box at the origin was chosen over reproducing Eigen's own empty-box sentinel only
    /// because the latter needs no separate representation this module does not otherwise carry
    /// (`AlignedBox` has no "empty" state); should a future caller ever iterate `Tree::nodes()`
    /// directly rather than through the validity-gated recursion above, this is the one place that
    /// value would surface, and `is_valid()` is what it must check first.
    fn empty() -> Self {
        Node {
            idx: NPOS,
            bbox: AlignedBox::from_point([C::zero(); D]),
        }
    }
    /// `bool is_valid() const` (`:68`).
    pub fn is_valid(&self) -> bool {
        self.idx != NPOS
    }
    /// `bool is_inner() const` (`:69`).
    pub fn is_inner(&self) -> bool {
        self.idx == INNER
    }
    /// `bool is_leaf() const` (`:70`).
    pub fn is_leaf(&self) -> bool {
        !self.is_inner()
    }
}

/// `SourceNode` (the unnamed constraint `:81-89` document, and `InputType` at `:690-698`'s own
/// shape): `idx()`, `centroid()`, `bbox()`.
pub trait SourceNode<C: TreeCoord, const D: usize> {
    fn idx(&self) -> usize;
    fn centroid(&self) -> [C; D];
    fn bbox(&self) -> AlignedBox<C, D>;
}

/// `template<int ANumDimensions, typename ACoordType> class Tree` (`:43-219`).
pub struct Tree<C: TreeCoord, const D: usize> {
    m_nodes: Vec<Node<C, D>>,
}

impl<C: TreeCoord, const D: usize> Default for Tree<C, D> {
    fn default() -> Self {
        Tree {
            m_nodes: Vec::new(),
        }
    }
}

impl<C: TreeCoord, const D: usize> Tree<C, D> {
    pub fn new() -> Self {
        Self::default()
    }

    /// `void clear()` (`:79`).
    pub fn clear(&mut self) {
        self.m_nodes.clear();
    }

    /// `const std::vector<Node>& nodes() const` (`:109`).
    pub fn nodes(&self) -> &[Node<C, D>] {
        &self.m_nodes
    }
    /// `const Node& node(size_t idx) const` (`:110`).
    pub fn node(&self, idx: usize) -> &Node<C, D> {
        &self.m_nodes[idx]
    }
    /// `bool empty() const` (`:111`).
    pub fn empty(&self) -> bool {
        self.m_nodes.is_empty()
    }
    /// `static size_t left_child_idx(size_t idx)` (`:114`).
    pub fn left_child_idx(idx: usize) -> usize {
        idx * 2 + 1
    }
    /// `static size_t right_child_idx(size_t idx)` (`:115`).
    pub fn right_child_idx(idx: usize) -> usize {
        Self::left_child_idx(idx) + 1
    }
    /// `const Node& left_child(size_t idx) const` (`:116`).
    pub fn left_child(&self, idx: usize) -> &Node<C, D> {
        self.node(Self::left_child_idx(idx))
    }
    /// `const Node& right_child(size_t idx) const` (`:117`).
    pub fn right_child(&self, idx: usize) -> &Node<C, D> {
        self.node(Self::right_child_idx(idx))
    }

    /// `void build(std::vector<SourceNode> &&input)` (`:90-95`).
    pub fn build<S: SourceNode<C, D>>(&mut self, mut input: Vec<S>) {
        self.build_modify_input(&mut input);
        input.clear();
    }

    /// `void build(const std::vector<SourceNode> &input)` (`:119-124`): builds from a copy, so
    /// the caller's own `input` is left untouched -- a `Vec<S>: Clone` bound, the Rust
    /// counterpart of the vendored copy constructor this overload relies on.
    pub fn build_from_ref<S: SourceNode<C, D> + Clone>(&mut self, input: &[S]) {
        self.build(input.to_vec());
    }

    /// `void build_modify_input(std::vector<SourceNode> &input)` (`:97-107`).
    pub fn build_modify_input<S: SourceNode<C, D>>(&mut self, input: &mut [S]) {
        if input.is_empty() {
            self.clear();
        } else {
            let len = slic3r_utils::utils::next_highest_power_of_2_u64(input.len() as u64) as usize
                * 2
                - 1;
            self.m_nodes = vec![Node::empty(); len];
            let last = input.len() - 1;
            self.build_recursive(input, 0, 0, last);
        }
    }

    /// `void build_recursive(...)` (`:128-155`).
    fn build_recursive<S: SourceNode<C, D>>(
        &mut self,
        input: &mut [S],
        node: usize,
        left: usize,
        right: usize,
    ) {
        debug_assert!(node < self.m_nodes.len());
        debug_assert!(left <= right);

        if left == right {
            let idx = input[left].idx();
            let bbox = input[left].bbox();
            self.m_nodes[node] = Node { idx, bbox };
            return;
        }

        let mut bbox = input[left].bbox();
        for i in (left + 1)..=right {
            bbox.extend_box(&input[i].bbox());
        }
        let dimension = bbox.longest_axis();

        let center = (left + right) / 2;
        self.partition_input(input, dimension, left, right, center);
        self.m_nodes[node].idx = INNER;
        self.m_nodes[node].bbox = bbox;
        self.build_recursive(input, node * 2 + 1, left, center);
        self.build_recursive(input, node * 2 + 2, center + 1, right);
    }

    /// `void partition_input(...)` (`:161-215`) -- QuickSelect, translated statement for
    /// statement, the same technique `kd_tree_indirect.rs`'s own `partition_input` uses.
    fn partition_input<S: SourceNode<C, D>>(
        &self,
        input: &mut [S],
        dimension: usize,
        mut left: usize,
        mut right: usize,
        k: usize,
    ) {
        while left < right {
            let center = (left + right) / 2;
            let pivot;
            {
                let mut left_value = input[left].centroid()[dimension];
                let mut center_value = input[center].centroid()[dimension];
                let mut right_value = input[right].centroid()[dimension];
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
                if !(input[i].centroid()[dimension] < pivot) {
                    break;
                }
            }
            loop {
                j -= 1;
                if !(input[j].centroid()[dimension] > pivot && i < j) {
                    break;
                }
            }
            while i < j {
                input.swap(i, j);
                loop {
                    i += 1;
                    if !(input[i].centroid()[dimension] < pivot) {
                        break;
                    }
                }
                loop {
                    j -= 1;
                    if !(input[j].centroid()[dimension] > pivot && i < j) {
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
}

/// `using Tree2f = Tree<2, float>;` (`:221`).
pub type Tree2f = Tree<f32, 2>;
/// `using Tree3f = Tree<3, float>;` (`:222`).
// PORT SRC: src/libslic3r/AABBTreeIndirect.hpp:222
pub type Tree3f = Tree<f32, 3>;
/// `using Tree2d = Tree<2, double>;` (`:223`).
pub type Tree2d = Tree<f64, 2>;
/// `using Tree3d = Tree<3, double>;` (`:224`).
// PORT SRC: src/libslic3r/AABBTreeIndirect.hpp:224
pub type Tree3d = Tree<f64, 3>;

/// `template<class G> struct Intersecting {};` / the `Eigen::AlignedBox` specialisation
/// (`:910-923`).
pub struct Intersecting<C: TreeCoord, const D: usize> {
    pub box_: AlignedBox<C, D>,
}

impl<C: TreeCoord, const D: usize> Intersecting<C, D> {
    pub fn matches(&self, node: &Node<C, D>) -> bool {
        self.box_.intersects(&node.bbox)
    }
}

/// `template<class G> auto intersecting(const G &g)` (`:925`).
pub fn intersecting<C: TreeCoord, const D: usize>(bbox: AlignedBox<C, D>) -> Intersecting<C, D> {
    Intersecting { box_: bbox }
}

/// `template<class G> struct Within {};` / the `Eigen::AlignedBox` specialisation (`:927-940`).
pub struct Within<C: TreeCoord, const D: usize> {
    pub box_: AlignedBox<C, D>,
}

impl<C: TreeCoord, const D: usize> Within<C, D> {
    pub fn matches(&self, node: &Node<C, D>) -> bool {
        if node.is_leaf() {
            self.box_.contains_box(&node.bbox)
        } else {
            self.box_.intersects(&node.bbox)
        }
    }
}

/// `template<class G> auto within(const G &g)` (`:942`).
pub fn within<C: TreeCoord, const D: usize>(bbox: AlignedBox<C, D>) -> Within<C, D> {
    Within { box_: bbox }
}

/// `bool traverse_recurse(...)` (`:948-976`).
fn traverse_recurse<C: TreeCoord, const D: usize>(
    tree: &Tree<C, D>,
    idx: usize,
    pred: &impl Fn(&Node<C, D>) -> bool,
    callback: &mut impl FnMut(&Node<C, D>) -> bool,
) -> bool {
    debug_assert!(tree.node(idx).is_valid());

    if !pred(tree.node(idx)) {
        return true;
    }

    if tree.node(idx).is_leaf() {
        callback(tree.node(idx))
    } else {
        // `:974`'s `&&` short-circuits: the right subtree is not visited once the left call
        // returns `false` (the callback's "stop traversal" signal, `:961`/`:984`).
        traverse_recurse(tree, Tree::<C, D>::left_child_idx(idx), pred, callback)
            && traverse_recurse(tree, Tree::<C, D>::right_child_idx(idx), pred, callback)
    }
}

/// `void traverse(const Tree &tree, Predicate &&pred, Fn &&callback)` (`:985-992`).
pub fn traverse<C: TreeCoord, const D: usize>(
    tree: &Tree<C, D>,
    pred: impl Fn(&Node<C, D>) -> bool,
    mut callback: impl FnMut(&Node<C, D>) -> bool,
) {
    if tree.empty() {
        return;
    }
    traverse_recurse(tree, 0, &pred, &mut callback);
}

/// `get_candidate_idxs(const TreeType&, const VectorType&, std::vector<size_t>&, size_t)`
/// (`:886-907`).
pub fn get_candidate_idxs<C: TreeCoord, const D: usize>(
    tree: &Tree<C, D>,
    v: [C; D],
    candidates: &mut Vec<usize>,
    node_idx: usize,
) {
    if tree.empty() || !tree.node(node_idx).bbox.contains_point(v) {
        return;
    }

    let node = tree.node(node_idx);
    debug_assert!(node.is_valid());
    debug_assert!(node.bbox.contains_point(v));

    if !node.is_leaf() {
        if tree.left_child(node_idx).bbox.contains_point(v) {
            get_candidate_idxs(tree, v, candidates, Tree::<C, D>::left_child_idx(node_idx));
        }
        if tree.right_child(node_idx).bbox.contains_point(v) {
            get_candidate_idxs(tree, v, candidates, Tree::<C, D>::right_child_idx(node_idx));
        }
    } else {
        candidates.push(node.idx);
    }
}

// --------------------------------------------------------------------------------------------
// The ray/distance queries over an indexed triangle set -- see the module doc for why
// `VertexType`/`IndexedFaceType`/`VectorType` collapse to one `Vec3<C>` here.
// --------------------------------------------------------------------------------------------

/// This module's stand-in for `VertexType`/`VectorType` -- see the module doc.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3<C: TreeCoord> {
    pub x: C,
    pub y: C,
    pub z: C,
}

impl<C: TreeCoord> Vec3<C> {
    pub fn new(x: C, y: C, z: C) -> Self {
        Vec3 { x, y, z }
    }
    pub fn splat(v: C) -> Self {
        Vec3::new(v, v, v)
    }
    pub fn get(&self, i: usize) -> C {
        match i {
            0 => self.x,
            1 => self.y,
            2 => self.z,
            _ => panic!("Vec3::get: index out of range"),
        }
    }
    // PORT SRC: src/libslic3r/Point.hpp:65 (the literal `3` is Vec3's own dimensionality, the
    // `Eigen::Matrix<float, 3, 1, ...>` Vec3f alias Point.hpp:68's Vec3d mirrors), not from
    // AABBTreeIndirect.hpp:677, which fixes an unrelated `3` (the tree's NumDimensions).
    pub fn to_array(&self) -> [C; 3] {
        [self.x, self.y, self.z]
    }
    pub fn sub(self, o: Vec3<C>) -> Vec3<C> {
        Vec3::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }
    pub fn add(self, o: Vec3<C>) -> Vec3<C> {
        Vec3::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }
    pub fn scale(self, s: C) -> Vec3<C> {
        Vec3::new(self.x * s, self.y * s, self.z * s)
    }
    pub fn to_f64(self) -> Vec3<f64> {
        Vec3::new(self.x.to_f64(), self.y.to_f64(), self.z.to_f64())
    }
}

impl Vec3<f64> {
    pub fn dot(self, o: Vec3<f64>) -> f64 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }
    /// `dir.cross(edge2)` etc. (`:327` and elsewhere).
    pub fn cross(self, o: Vec3<f64>) -> Vec3<f64> {
        Vec3::new(
            self.y * o.z - self.z * o.y,
            self.z * o.x - self.x * o.z,
            self.x * o.y - self.y * o.x,
        )
    }
    /// `dir.cwiseInverse()` (`:747`).
    pub fn cwise_inverse(self) -> Vec3<f64> {
        Vec3::new(1f64 / self.x, 1f64 / self.y, 1f64 / self.z)
    }
    pub fn squared_norm(self) -> f64 {
        self.dot(self)
    }
}

impl std::ops::Mul<f64> for Vec3<f64> {
    type Output = Vec3<f64>;
    fn mul(self, s: f64) -> Vec3<f64> {
        self.scale(s)
    }
}

/// The one-primitive-per-triangle `SourceNode` [`build_aabb_tree_over_indexed_triangle_set`]
/// builds `Tree<C, 3>` from -- `struct InputType` (`:690-698`).
struct TriangleInput<C: TreeCoord> {
    idx: usize,
    // PORT SRC: src/libslic3r/AABBTreeIndirect.hpp:677
    bbox: AlignedBox<C, 3>,
    // PORT SRC: src/libslic3r/AABBTreeIndirect.hpp:677
    centroid: [C; 3],
}

// PORT SRC: src/libslic3r/AABBTreeIndirect.hpp:677
impl<C: TreeCoord> SourceNode<C, 3> for TriangleInput<C> {
    fn idx(&self) -> usize {
        self.idx
    }
    // PORT SRC: src/libslic3r/AABBTreeIndirect.hpp:677
    fn centroid(&self) -> [C; 3] {
        self.centroid
    }
    // PORT SRC: src/libslic3r/AABBTreeIndirect.hpp:677
    fn bbox(&self) -> AlignedBox<C, 3> {
        self.bbox
    }
}

/// `build_aabb_tree_over_indexed_triangle_set(vertices, faces, eps)` (`:676-722`).
pub fn build_aabb_tree_over_indexed_triangle_set<C: TreeCoord>(
    vertices: &[Vec3<C>],
    // PORT SRC: src/libslic3r/AABBTreeIndirect.hpp:710 (`(1./3.) * (v1 + v2 + v3)`, the one line in this header that spells out a triangle's vertex count as a literal; distinct from :677's unrelated NumDimensions `3`).
    faces: &[[usize; 3]],
    eps: C,
    // PORT SRC: src/libslic3r/AABBTreeIndirect.hpp:677
) -> Tree<C, 3> {
    let mut input = Vec::with_capacity(faces.len());
    let veps = Vec3::splat(eps);
    for (i, face) in faces.iter().enumerate() {
        let v1 = vertices[face[0]];
        let v2 = vertices[face[1]];
        let v3 = vertices[face[2]];
        // `n.m_centroid = (1./3.) * (v1 + v2 + v3);` (`:710`).
        // PORT SRC: src/libslic3r/AABBTreeIndirect.hpp:710
        let third = C::from_f64(1f64 / 3f64);
        let centroid = v1.add(v2).add(v3).scale(third);
        let mut bbox = AlignedBox::from_point(v1.to_array());
        bbox.extend_point(v2.to_array());
        bbox.extend_point(v3.to_array());
        for d in 0..3 {
            bbox.min[d] = bbox.min[d] - veps.get(d);
            bbox.max[d] = bbox.max[d] + veps.get(d);
        }
        input.push(TriangleInput {
            idx: i,
            bbox,
            centroid: centroid.to_array(),
        });
    }

    let mut out = Tree::new();
    out.build(input);
    out
}

/// `igl::Hit<float>`: the ray-intersection hit record (`:24-25`'s own `#include <igl/Hit.h>`).
/// `id`/`gid` are `int` at `deps_src/libigl/igl/Hit.h:20-22`, narrowed explicitly at
/// `AABBTreeIndirect.hpp:422`'s `int(node.idx)` construction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hit {
    pub id: i32,
    pub gid: i32,
    pub u: f32,
    pub v: f32,
    pub t: f32,
}

/// `ray_box_intersect_invdir(...)` (`:273-311`): always `f64`, per the vendored body's own `x()`/
/// `y()`/`z()` 3D-only access.
fn ray_box_intersect_invdir(
    origin: Vec3<f64>,
    inv_dir: Vec3<f64>,
    // PORT SRC: src/libslic3r/AABBTreeIndirect.hpp:277 (`Eigen::AlignedBox<Scalar,3> box`, this
    // function's own parameter, not :677's unrelated tree-level `3`).
    mut bbox: AlignedBox<f64, 3>,
    t0: f64,
    t1: f64,
) -> bool {
    if inv_dir.x < 0f64 {
        std::mem::swap(&mut bbox.min[0], &mut bbox.max[0]);
    }
    if inv_dir.y < 0f64 {
        std::mem::swap(&mut bbox.min[1], &mut bbox.max[1]);
    }
    let mut tmin = (bbox.min[0] - origin.x) * inv_dir.x;
    let tymax = (bbox.max[1] - origin.y) * inv_dir.y;
    if tmin > tymax {
        return false;
    }
    let mut tmax = (bbox.max[0] - origin.x) * inv_dir.x;
    let tymin = (bbox.min[1] - origin.y) * inv_dir.y;
    if tymin > tmax {
        return false;
    }
    if tymin > tmin {
        tmin = tymin;
    }
    if tymax < tmax {
        tmax = tymax;
    }
    if inv_dir.z < 0f64 {
        std::mem::swap(&mut bbox.min[2], &mut bbox.max[2]);
    }
    let tzmin = (bbox.min[2] - origin.z) * inv_dir.z;
    if tzmin > tmax {
        return false;
    }
    let tzmax = (bbox.max[2] - origin.z) * inv_dir.z;
    if tmin > tzmax {
        return false;
    }
    if tzmin > tmin {
        tmin = tzmin;
    }
    if tzmax < tmax {
        tmax = tzmax;
    }
    tmin < t1 && tmax > t0
}

/// `intersect_triangle(...)` (`:319-368`'s `(double,double)` body -- see the module doc for why
/// this is the one body every one of the four vendored overloads reduces to). Returns
/// `Some((t, u, v))` on a hit.
fn intersect_triangle(
    orig: Vec3<f64>,
    dir: Vec3<f64>,
    vert0: Vec3<f64>,
    vert1: Vec3<f64>,
    vert2: Vec3<f64>,
    eps: f64,
) -> Option<(f64, f64, f64)> {
    let edge1 = vert1.sub(vert0);
    let edge2 = vert2.sub(vert0);
    let pvec = dir.cross(edge2);
    let det = edge1.dot(pvec);

    let (mut u, v, t);
    if det > eps {
        let tvec = orig.sub(vert0);
        u = tvec.dot(pvec);
        if u < 0f64 || u > det {
            return None;
        }
        let qvec = tvec.cross(edge1);
        let v_num = dir.dot(qvec);
        if v_num < 0f64 || u + v_num > det {
            return None;
        }
        let inv_det = 1f64 / det;
        t = edge2.dot(qvec) * inv_det;
        u *= inv_det;
        v = v_num * inv_det;
    } else if det < -eps {
        let tvec = orig.sub(vert0);
        u = tvec.dot(pvec);
        if u > 0f64 || u < det {
            return None;
        }
        let qvec = tvec.cross(edge1);
        let v_num = dir.dot(qvec);
        if v_num > 0f64 || u + v_num < det {
            return None;
        }
        let inv_det = 1f64 / det;
        t = edge2.dot(qvec) * inv_det;
        u *= inv_det;
        v = v_num * inv_det;
    } else {
        return None;
    }
    Some((t, u, v))
}

// TODO(port): src/libslic3r/AABBTreeIndirect.hpp:388-398 -- `intersect_triangle_epsilon`.
// **Not ported: a pre-existing bug in dead vendored code**, found while writing this task's
// proof shim. `double l = (bbox.max() - bbox.min()).cwiseMax();` (`:393`) calls Eigen's
// `cwiseMax()` with zero arguments; `cwiseMax` takes exactly one (either a matching vector, to
// combine element-wise, or a scalar, per `MatrixCwiseBinaryOps.inc`), so this does not compile
// under the pinned Eigen the moment the template is instantiated. Measured, not assumed: `grep
// -rn intersect_triangle_epsilon vendor/OrcaSlicer/src/libslic3r vendor/OrcaSlicer/src/slic3r`
// finds only this function's own definition -- no caller anywhere in the vendored tree, in or
// out of scope, so the real oracle build never instantiates it and the bug is latent rather
// than a build failure anyone has hit. The evidently intended call, matching both this same
// file's own working `bbox.diagonal().maxCoeff(&dimension)` (`build_recursive`, ported as
// [`AlignedBox::longest_axis`] above) and Eigen's real API, is `maxCoeff()` with no
// arguments (the scalar maximum coefficient) -- but porting that corrected body would be
// porting this task's own guess at a fix, not what the pinned source does, and the function has
// no caller to observe either answer against. Left unported; this task's own proof and
// [`intersect_ray_first_hit`]'s own tests instead use the plain literal `0.000001` --
// `AABBTreeIndirect.hpp:742`'s own default argument for `eps`, which *is* exercised by real
// (if presently out-of-scope) callers, unlike this function. Named as a finding in this task's
// pull request for the regeneration pass or an amendment to judge.

fn intersect_ray_recursive_first_hit<C: TreeCoord>(
    vertices: &[Vec3<C>],
    // PORT SRC: src/libslic3r/AABBTreeIndirect.hpp:710 (`(1./3.) * (v1 + v2 + v3)`, the one line in this header that spells out a triangle's vertex count as a literal; distinct from :677's unrelated NumDimensions `3`).
    faces: &[[usize; 3]],
    // PORT SRC: src/libslic3r/AABBTreeIndirect.hpp:222 (`Tree3f`'s NumDimensions `3`, which
    // this port's query functions fix `TreeType` to, mirroring `Tree2f`/`Tree3f`/`Tree2d`/
    // `Tree3d` at :221-224 rather than the vendored template's generic `TreeType`).
    tree: &Tree<C, 3>,
    origin: Vec3<f64>,
    dir: Vec3<f64>,
    invdir: Vec3<f64>,
    eps: f64,
    node_idx: usize,
    min_t: f64,
) -> Option<Hit> {
    let node = tree.node(node_idx);
    debug_assert!(node.is_valid());

    if !ray_box_intersect_invdir(origin, invdir, node.bbox.to_f64(), 0f64, min_t) {
        return None;
    }

    if node.is_leaf() {
        let face = faces[node.idx];
        let v0 = vertices[face[0]].to_f64();
        let v1 = vertices[face[1]].to_f64();
        let v2 = vertices[face[2]].to_f64();
        match intersect_triangle(origin, dir, v0, v1, v2, eps) {
            Some((t, u, v)) if t > 0f64 => Some(Hit {
                id: node.idx as i32,
                gid: -1,
                u: u as f32,
                v: v as f32,
                t: t as f32,
            }),
            _ => None,
        }
    } else {
        // `size_t left = node_idx * 2 + 1; size_t right = left + 1;` (`:427-429`).
        // PORT SRC: src/libslic3r/AABBTreeIndirect.hpp:428
        let left = node_idx * 2 + 1;
        let right = left + 1;

        let mut min_t = min_t;
        let mut best: Option<Hit> = None;
        if let Some(left_hit) = intersect_ray_recursive_first_hit(
            vertices, faces, tree, origin, dir, invdir, eps, left, min_t,
        ) {
            if (left_hit.t as f64) < min_t {
                min_t = left_hit.t as f64;
                best = Some(left_hit);
            }
        }
        if let Some(right_hit) = intersect_ray_recursive_first_hit(
            vertices, faces, tree, origin, dir, invdir, eps, right, min_t,
        ) {
            if (right_hit.t as f64) < min_t {
                best = Some(right_hit);
            }
        }
        best
    }
}

/// `intersect_ray_first_hit(...)` (`:727-752`).
pub fn intersect_ray_first_hit<C: TreeCoord>(
    vertices: &[Vec3<C>],
    // PORT SRC: src/libslic3r/AABBTreeIndirect.hpp:710 (`(1./3.) * (v1 + v2 + v3)`, the one line in this header that spells out a triangle's vertex count as a literal; distinct from :677's unrelated NumDimensions `3`).
    faces: &[[usize; 3]],
    // PORT SRC: src/libslic3r/AABBTreeIndirect.hpp:222 (`Tree3f`'s NumDimensions `3`; see the
    // note on `intersect_ray_recursive_first_hit`'s own `tree` parameter above).
    tree: &Tree<C, 3>,
    origin: Vec3<C>,
    dir: Vec3<C>,
    eps: f64,
) -> Option<Hit> {
    if tree.empty() {
        return None;
    }
    let origin = origin.to_f64();
    let dir = dir.to_f64();
    let invdir = dir.cwise_inverse();
    intersect_ray_recursive_first_hit(
        vertices,
        faces,
        tree,
        origin,
        dir,
        invdir,
        eps,
        0,
        f64::INFINITY,
    )
}

fn intersect_ray_recursive_all_hits<C: TreeCoord>(
    vertices: &[Vec3<C>],
    // PORT SRC: src/libslic3r/AABBTreeIndirect.hpp:710 (`(1./3.) * (v1 + v2 + v3)`, the one line in this header that spells out a triangle's vertex count as a literal; distinct from :677's unrelated NumDimensions `3`).
    faces: &[[usize; 3]],
    // PORT SRC: src/libslic3r/AABBTreeIndirect.hpp:222 (`Tree3f`'s NumDimensions `3`; see the
    // note on `intersect_ray_recursive_first_hit`'s own `tree` parameter above).
    tree: &Tree<C, 3>,
    origin: Vec3<f64>,
    dir: Vec3<f64>,
    invdir: Vec3<f64>,
    eps: f64,
    node_idx: usize,
    hits: &mut Vec<Hit>,
) {
    let node = tree.node(node_idx);
    debug_assert!(node.is_valid());

    if !ray_box_intersect_invdir(origin, invdir, node.bbox.to_f64(), 0f64, f64::INFINITY) {
        return;
    }

    if node.is_leaf() {
        let face = faces[node.idx];
        let v0 = vertices[face[0]].to_f64();
        let v1 = vertices[face[1]].to_f64();
        let v2 = vertices[face[2]].to_f64();
        if let Some((t, u, v)) = intersect_triangle(origin, dir, v0, v1, v2, eps) {
            if t > 0f64 {
                hits.push(Hit {
                    id: node.idx as i32,
                    gid: -1,
                    u: u as f32,
                    v: v as f32,
                    t: t as f32,
                });
            }
        }
    } else {
        let left = node_idx * 2 + 1;
        let right = left + 1;
        intersect_ray_recursive_all_hits(
            vertices, faces, tree, origin, dir, invdir, eps, left, hits,
        );
        intersect_ray_recursive_all_hits(
            vertices, faces, tree, origin, dir, invdir, eps, right, hits,
        );
    }
}

/// `intersect_ray_all_hits(...)` (`:759-793`): every intersection, sorted by `t`.
pub fn intersect_ray_all_hits<C: TreeCoord>(
    vertices: &[Vec3<C>],
    // PORT SRC: src/libslic3r/AABBTreeIndirect.hpp:710 (`(1./3.) * (v1 + v2 + v3)`, the one line in this header that spells out a triangle's vertex count as a literal; distinct from :677's unrelated NumDimensions `3`).
    faces: &[[usize; 3]],
    // PORT SRC: src/libslic3r/AABBTreeIndirect.hpp:222 (`Tree3f`'s NumDimensions `3`; see the
    // note on `intersect_ray_recursive_first_hit`'s own `tree` parameter above).
    tree: &Tree<C, 3>,
    origin: Vec3<C>,
    dir: Vec3<C>,
    eps: f64,
) -> Vec<Hit> {
    if tree.empty() {
        return Vec::new();
    }
    let origin = origin.to_f64();
    let dir = dir.to_f64();
    let invdir = dir.cwise_inverse();
    let mut hits = Vec::new();
    intersect_ray_recursive_all_hits(
        vertices, faces, tree, origin, dir, invdir, eps, 0, &mut hits,
    );
    // `:790`'s `std::sort` renders as `sort_unstable_by` (idiom map: no ordering guarantee
    // between equal elements, matching `std::sort`'s own lack of a stability guarantee); a
    // shared edge's two triangles (`:758`) can tie on `t`, and `sort_by`'s stable tie order
    // would diverge from `std::sort`'s unspecified one.
    // PORT NOTE: `:790`'s comparator (`l.t < r.t`) is undefined behaviour under `std::sort`'s
    // strict-weak-ordering precondition if `t` is NaN, with no panic in the oracle's release
    // build. This port makes that undefined behaviour a live panic instead.
    hits.sort_unstable_by(|l, r| {
        l.t.partial_cmp(&r.t)
            .expect("ray hit `t` must never be NaN")
    });
    hits
}

/// `closest_point_to_triangle(const Vector &p, const Vector &a, const Vector &b, const Vector
/// &c)` (`:479-526`), ported verbatim (Real-time Collision Detection, Ericson, ch. 5).
fn closest_point_to_triangle(p: Vec3<f64>, a: Vec3<f64>, b: Vec3<f64>, c: Vec3<f64>) -> Vec3<f64> {
    let ab = b.sub(a);
    let ac = c.sub(a);
    let ap = p.sub(a);
    let d1 = ab.dot(ap);
    let d2 = ac.dot(ap);
    if d1 <= 0f64 && d2 <= 0f64 {
        return a;
    }
    let bp = p.sub(b);
    let d3 = ab.dot(bp);
    let d4 = ac.dot(bp);
    if d3 >= 0f64 && d4 <= d3 {
        return b;
    }
    let vc = d1 * d4 - d3 * d2;
    if a != b && vc <= 0f64 && d1 >= 0f64 && d3 <= 0f64 {
        let v = d1 / (d1 - d3);
        return a.add(ab.scale(v));
    }
    let cp = p.sub(c);
    let d5 = ab.dot(cp);
    let d6 = ac.dot(cp);
    if d6 >= 0f64 && d5 <= d6 {
        return c;
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0f64 && d2 >= 0f64 && d6 <= 0f64 {
        let w = d2 / (d2 - d6);
        return a.add(ac.scale(w));
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0f64 && (d4 - d3) >= 0f64 && (d5 - d6) >= 0f64 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return b.add(c.sub(b).scale(w));
    }
    let denom = 1f64 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    a.add(ab.scale(v)).add(ac.scale(w))
}

// FIDELITY(port): always computes in `f64`, where `:536`'s `using ScalarType = typename
// VectorType::Scalar` (with `:547-550`) runs the vendored `closest_point_to_origin` in
// `VectorType::Scalar` -- `float` for `GCode/SeamPlacer.cpp:379`'s `is_any_triangle_in_radius`
// caller. See the module doc's FIDELITY(port) paragraph; not held to exact equality for a
// `Tree2f`/`Tree3f` query, and no oracle comparison in this task's proof reaches that case.
fn closest_point_on_triangle_to_origin<C: TreeCoord>(
    vertices: &[Vec3<C>],
    // PORT SRC: src/libslic3r/AABBTreeIndirect.hpp:710 (`(1./3.) * (v1 + v2 + v3)`, the one line in this header that spells out a triangle's vertex count as a literal; distinct from :677's unrelated NumDimensions `3`).
    faces: &[[usize; 3]],
    primitive_index: usize,
    origin: Vec3<f64>,
) -> (Vec3<f64>, f64) {
    let face = faces[primitive_index];
    let closest = closest_point_to_triangle(
        origin,
        vertices[face[0]].to_f64(),
        vertices[face[1]].to_f64(),
        vertices[face[2]].to_f64(),
    );
    let squared_distance = origin.sub(closest).squared_norm();
    (closest, squared_distance)
}

// `looked_left`/`looked_right`'s final assignment inside `look_left!()`/`look_right!()` is
// never read back in the branch that runs last (the vendored body itself leaves both the same
// way, `:596-634`: each flag only ever gates whether the *other* branch still needs to look) --
// a harmless dead store, not a bug, so the lint is silenced rather than restructured away from
// the vendored control flow it mirrors.
#[allow(clippy::too_many_arguments, unused_assignments)]
fn squared_distance_to_indexed_primitives_recursive<C: TreeCoord>(
    vertices: &[Vec3<C>],
    // PORT SRC: src/libslic3r/AABBTreeIndirect.hpp:710 (`(1./3.) * (v1 + v2 + v3)`, the one line in this header that spells out a triangle's vertex count as a literal; distinct from :677's unrelated NumDimensions `3`).
    faces: &[[usize; 3]],
    // PORT SRC: src/libslic3r/AABBTreeIndirect.hpp:222 (`Tree3f`'s NumDimensions `3`; see the
    // note on `intersect_ray_recursive_first_hit`'s own `tree` parameter above).
    tree: &Tree<C, 3>,
    origin: Vec3<f64>,
    node_idx: usize,
    low_sqr_d: f64,
    up_sqr_d: f64,
    best_idx: &mut usize,
    best_point: &mut Vec3<f64>,
) -> f64 {
    if low_sqr_d > up_sqr_d {
        return low_sqr_d;
    }

    let mut up_sqr_d = up_sqr_d;
    let node = tree.node(node_idx);
    debug_assert!(node.is_valid());
    if node.is_leaf() {
        let (c_candidate, sqr_dist) =
            closest_point_on_triangle_to_origin(vertices, faces, node.idx, origin);
        if sqr_dist < up_sqr_d {
            *best_idx = node.idx;
            *best_point = c_candidate;
            up_sqr_d = sqr_dist;
        }
    } else {
        let left_idx = node_idx * 2 + 1;
        let right_idx = left_idx + 1;
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
                    vertices,
                    faces,
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
                    vertices,
                    faces,
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

        // FIDELITY(port): `:617`'s `distancer.origin.template cast<BBoxScalar>()` narrows the
        // *origin* to the box's own scalar; this widens the *box* to `f64` instead, and
        // `:622`'s `squaredExteriorDistance` likewise accumulates in `BBoxScalar`, not `f64`.
        // See the module doc's FIDELITY(port) paragraph.
        if node_left.bbox.to_f64().contains_point(origin.to_array()) {
            look_left!();
        }
        if node_right.bbox.to_f64().contains_point(origin.to_array()) {
            look_right!();
        }
        let left_up_sqr_d = node_left
            .bbox
            .to_f64()
            .squared_exterior_distance(origin.to_array());
        let right_up_sqr_d = node_right
            .bbox
            .to_f64()
            .squared_exterior_distance(origin.to_array());
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

/// `squared_distance_to_indexed_triangle_set(...)` (`:800-820`). Returns
/// `(squared_distance, hit_idx, hit_point)`, or `(-1, NPOS, origin)` for an empty tree -- the
/// Rust return-values-tuple translation of the two `&`/`&mut` out-parameters (idiom map, "A
/// non-const reference out-parameter -> A return value").
pub fn squared_distance_to_indexed_triangle_set<C: TreeCoord>(
    vertices: &[Vec3<C>],
    // PORT SRC: src/libslic3r/AABBTreeIndirect.hpp:710 (`(1./3.) * (v1 + v2 + v3)`, the one line in this header that spells out a triangle's vertex count as a literal; distinct from :677's unrelated NumDimensions `3`).
    faces: &[[usize; 3]],
    // PORT SRC: src/libslic3r/AABBTreeIndirect.hpp:222 (`Tree3f`'s NumDimensions `3`; see the
    // note on `intersect_ray_recursive_first_hit`'s own `tree` parameter above).
    tree: &Tree<C, 3>,
    point: Vec3<C>,
) -> (f64, usize, Vec3<f64>) {
    let origin = point.to_f64();
    if tree.empty() {
        return (-1f64, NPOS, origin);
    }
    let mut hit_idx = NPOS;
    let mut hit_point = origin;
    let sq = squared_distance_to_indexed_primitives_recursive(
        vertices,
        faces,
        tree,
        origin,
        0,
        0f64,
        f64::INFINITY,
        &mut hit_idx,
        &mut hit_point,
    );
    (sq, hit_idx, hit_point)
}

/// `is_any_triangle_in_radius(...)` (`:826-854`).
pub fn is_any_triangle_in_radius<C: TreeCoord>(
    vertices: &[Vec3<C>],
    // PORT SRC: src/libslic3r/AABBTreeIndirect.hpp:710 (`(1./3.) * (v1 + v2 + v3)`, the one line in this header that spells out a triangle's vertex count as a literal; distinct from :677's unrelated NumDimensions `3`).
    faces: &[[usize; 3]],
    // PORT SRC: src/libslic3r/AABBTreeIndirect.hpp:222 (`Tree3f`'s NumDimensions `3`; see the
    // note on `intersect_ray_recursive_first_hit`'s own `tree` parameter above).
    tree: &Tree<C, 3>,
    point: Vec3<C>,
    max_distance_squared: f64,
) -> bool {
    if tree.empty() {
        return false;
    }
    let origin = point.to_f64();
    let mut hit_idx = NPOS;
    let mut hit_point = Vec3::splat(f64::NAN);
    squared_distance_to_indexed_primitives_recursive(
        vertices,
        faces,
        tree,
        origin,
        0,
        0f64,
        max_distance_squared,
        &mut hit_idx,
        &mut hit_point,
    );
    hit_point.x.is_finite() && hit_point.y.is_finite() && hit_point.z.is_finite()
}

fn indexed_primitives_within_distance_squared_recursive<C: TreeCoord>(
    vertices: &[Vec3<C>],
    // PORT SRC: src/libslic3r/AABBTreeIndirect.hpp:710 (`(1./3.) * (v1 + v2 + v3)`, the one line in this header that spells out a triangle's vertex count as a literal; distinct from :677's unrelated NumDimensions `3`).
    faces: &[[usize; 3]],
    // PORT SRC: src/libslic3r/AABBTreeIndirect.hpp:222 (`Tree3f`'s NumDimensions `3`; see the
    // note on `intersect_ray_recursive_first_hit`'s own `tree` parameter above).
    tree: &Tree<C, 3>,
    origin: Vec3<f64>,
    node_idx: usize,
    squared_distance_limit: f64,
    found: &mut Vec<usize>,
) {
    let node = tree.node(node_idx);
    debug_assert!(node.is_valid());
    if node.is_leaf() {
        let (_c, sqr_dist) = closest_point_on_triangle_to_origin(vertices, faces, node.idx, origin);
        if sqr_dist < squared_distance_limit {
            found.push(node.idx);
        }
    } else {
        let left_idx = node_idx * 2 + 1;
        let right_idx = left_idx + 1;
        let node_left = tree.node(left_idx);
        let node_right = tree.node(right_idx);
        debug_assert!(node_left.is_valid());
        debug_assert!(node_right.is_valid());

        if node_left
            .bbox
            .to_f64()
            .squared_exterior_distance(origin.to_array())
            < squared_distance_limit
        {
            indexed_primitives_within_distance_squared_recursive(
                vertices,
                faces,
                tree,
                origin,
                left_idx,
                squared_distance_limit,
                found,
            );
        }
        if node_right
            .bbox
            .to_f64()
            .squared_exterior_distance(origin.to_array())
            < squared_distance_limit
        {
            indexed_primitives_within_distance_squared_recursive(
                vertices,
                faces,
                tree,
                origin,
                right_idx,
                squared_distance_limit,
                found,
            );
        }
    }
}

/// `all_triangles_in_radius(...)` (`:858-881`).
pub fn all_triangles_in_radius<C: TreeCoord>(
    vertices: &[Vec3<C>],
    // PORT SRC: src/libslic3r/AABBTreeIndirect.hpp:710 (`(1./3.) * (v1 + v2 + v3)`, the one line in this header that spells out a triangle's vertex count as a literal; distinct from :677's unrelated NumDimensions `3`).
    faces: &[[usize; 3]],
    // PORT SRC: src/libslic3r/AABBTreeIndirect.hpp:222 (`Tree3f`'s NumDimensions `3`; see the
    // note on `intersect_ray_recursive_first_hit`'s own `tree` parameter above).
    tree: &Tree<C, 3>,
    point: Vec3<C>,
    max_distance_squared: f64,
) -> Vec<usize> {
    if tree.empty() {
        return Vec::new();
    }
    let origin = point.to_f64();
    let mut found = Vec::new();
    indexed_primitives_within_distance_squared_recursive(
        vertices,
        faces,
        tree,
        origin,
        0,
        max_distance_squared,
        &mut found,
    );
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use slic3r_proof::{compare_unit, proof_units, Oracle};

    fn unit_triangle_mesh() -> (Vec<Vec3<f64>>, Vec<[usize; 3]>) {
        (
            vec![
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(4.0, 0.0, 0.0),
                Vec3::new(0.0, 4.0, 0.0),
                Vec3::new(10.0, 10.0, 0.0),
                Vec3::new(14.0, 10.0, 0.0),
                Vec3::new(10.0, 14.0, 0.0),
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        )
    }

    /// Five triangles with distinct, spread-out centroids, so `partition_input`'s (`:161-215`)
    /// QuickSelect loop body (`:190-205`) actually runs: with the two-triangle fixture above,
    /// `build_recursive`'s only call is `right (1) <= left (0) + 2`, which is `:187-189`'s early
    /// break, so the swap-partition loop never executes in any oracle comparison. `right (4) <=
    /// left (0) + 2` is false for five elements, so this fixture reaches it.
    fn unit_triangle_mesh_wide() -> (Vec<Vec3<f64>>, Vec<[usize; 3]>) {
        (
            vec![
                // Triangle 0: centroid (1.33, 1.33, 0).
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(4.0, 0.0, 0.0),
                Vec3::new(0.0, 4.0, 0.0),
                // Triangle 1: centroid (11.33, 11.33, 0).
                Vec3::new(10.0, 10.0, 0.0),
                Vec3::new(14.0, 10.0, 0.0),
                Vec3::new(10.0, 14.0, 0.0),
                // Triangle 2: centroid (0, 1.33, 11.33).
                Vec3::new(0.0, 0.0, 10.0),
                Vec3::new(0.0, 4.0, 10.0),
                Vec3::new(0.0, 0.0, 14.0),
                // Triangle 3: centroid (21.33, 1.33, 0).
                Vec3::new(20.0, 0.0, 0.0),
                Vec3::new(24.0, 0.0, 0.0),
                Vec3::new(20.0, 4.0, 0.0),
                // Triangle 4: centroid (1.33, 21.33, 5).
                Vec3::new(0.0, 20.0, 5.0),
                Vec3::new(4.0, 20.0, 5.0),
                Vec3::new(0.0, 24.0, 5.0),
            ],
            vec![[0, 1, 2], [3, 4, 5], [6, 7, 8], [9, 10, 11], [12, 13, 14]],
        )
    }

    #[test]
    fn build_then_traverse_visits_both_leaves() {
        let (vertices, faces) = unit_triangle_mesh();
        let tree = build_aabb_tree_over_indexed_triangle_set(&vertices, &faces, 0.0);

        let mut leaves = Vec::new();
        traverse(
            &tree,
            |_node| true,
            |node| {
                leaves.push(node.idx);
                true
            },
        );
        leaves.sort_unstable();
        assert_eq!(leaves, vec![0, 1]);
    }

    #[test]
    fn intersect_ray_first_hit_finds_the_near_triangle() {
        let (vertices, faces) = unit_triangle_mesh();
        let tree = build_aabb_tree_over_indexed_triangle_set(&vertices, &faces, 0.0);
        let eps = 0.000001; // AABBTreeIndirect.hpp:742 default argument -- see the TODO(port) note above intersect_triangle_epsilon.
        let hit = intersect_ray_first_hit(
            &vertices,
            &faces,
            &tree,
            Vec3::new(1.0, 1.0, -5.0),
            Vec3::new(0.0, 0.0, 1.0),
            eps,
        );
        assert_eq!(hit.map(|h| h.id), Some(0));
    }

    /// Proof path 2 (R19, `port/manifest.tsv`'s `AABBTreeIndirect.hpp` row). No committed stage
    /// dump carries this header's own output (it is reached only through `TriangleMesh.hpp`'s
    /// slicer/raycast surface, not itself in scope at the pin as this task finds it), so --
    /// `docs/PORTING.md`'s "Decisions recorded on 2026-11-26", decision 2 -- it is proven
    /// through its own per-unit bridge instead. `crates/slic3r-ffi/cxx/proof_units/
    /// aabb_tree_indirect.cpp` builds the real vendored `Slic3r::AABBTreeIndirect::Tree<3,
    /// double>` over the same mesh this test builds its own tree from -- the five-triangle
    /// `unit_triangle_mesh_wide` fixture, not the two-triangle one -- and calls the real
    /// vendored `intersect_ray_first_hit` and `squared_distance_to_indexed_triangle_set`.
    // PROOF: 2
    #[test]
    fn aabb_tree_indirect_matches_the_oracle() {
        let (vertices, faces) = unit_triangle_mesh_wide();
        let flat_vertices: Vec<f64> = vertices.iter().flat_map(|v| [v.x, v.y, v.z]).collect();
        let flat_faces: Vec<i64> = faces
            .iter()
            .flat_map(|f| [f[0] as i64, f[1] as i64, f[2] as i64])
            .collect();

        let tree = build_aabb_tree_over_indexed_triangle_set(&vertices, &faces, 0.0);
        let eps = 0.000001; // AABBTreeIndirect.hpp:742 default argument -- see the TODO(port) note above intersect_triangle_epsilon.

        let origin = Vec3::new(1.0, 1.0, -5.0);
        let dir = Vec3::new(0.0, 0.0, 1.0);
        let actual_hit = intersect_ray_first_hit(&vertices, &faces, &tree, origin, dir, eps)
            .map(|h| h.id as i64)
            .unwrap_or(-1);
        let oracle_hit = proof_units::aabb_tree_indirect::path2_intersect_ray_first_hit(
            &flat_vertices,
            &flat_faces,
            origin.x,
            origin.y,
            origin.z,
            dir.x,
            dir.y,
            dir.z,
        );
        compare_unit(
            "AABBTreeIndirect.hpp",
            Oracle::Built,
            actual_hit,
            oracle_hit,
        )
        .expect("intersect_ray_first_hit's hit face index must match the oracle exactly");

        let point = Vec3::new(2.0, 2.0, 1.0);
        let (actual_sq, actual_idx, _) =
            squared_distance_to_indexed_triangle_set(&vertices, &faces, &tree, point);
        let (oracle_sq, oracle_idx) = proof_units::aabb_tree_indirect::path2_squared_distance(
            &flat_vertices,
            &flat_faces,
            point.x,
            point.y,
            point.z,
        );
        assert_eq!(
            actual_idx as i64, oracle_idx,
            "squared_distance_to_indexed_triangle_set's hit index must match the oracle"
        );
        compare_unit("AABBTreeIndirect.hpp", Oracle::Built, actual_sq, oracle_sq).expect(
            "squared_distance_to_indexed_triangle_set's distance must match the oracle exactly",
        );
    }
}

// PORT STATUS
// source: src/libslic3r/AABBTreeIndirect.hpp
// confidence: low
// tier: exact
// oracle: built
