//! Port of `src/libslic3r/MeshSplitImpl.hpp` (task `60-foundation-trees-grids`, plan step 20,
//! `foundation-09` row of `port/foundation.tsv`): splitting an indexed triangle mesh into its
//! connected patches by a flood fill over a face-adjacency index. `template-heavy: yes` in
//! `port/manifest.tsv`.
//!
//! **Lands `drafted`, not `ported`, and lands no `mod` line.** Every function this header
//! exports is generic over `Its`, but each one is hard-wired, inside its own body, to the one
//! concrete type `indexed_triangle_set` -- `its_split`'s own `const indexed_triangle_set &its =
//! ItsWithNeighborsIndex_<Its>::get_its(m);` (`:147`), and `create_face_neighbors_index`'s
//! parameter is `const indexed_triangle_set &its` outright (`:233`), not a second template
//! parameter.
//!
//! **Correction (second review pass): `indexed_triangle_set`'s owner is not a fan-out module.**
//! An earlier pass of this doc said `TriangleMesh.hpp`'s manifest row is `template-heavy: no`,
//! "an ordinary fan-out module" this task does not write. `port/manifest.tsv:575` actually reads
//! `template-heavy: yes`, and `port/foundation.tsv:20` assigns it to `foundation-19` -- a sibling
//! *step-20* hand-design, cut and built the same way this task is, not a Phase A/B fan-out unit.
//! That does not change what this module can do about it: `foundation-19` is a concurrent sibling
//! task in its own worktree, unmerged as this is written, so there is still no crate this one may
//! depend on that exports the real type (this crate's generated `Cargo.toml`, from
//! `port/cut.tsv`'s `dep` rows, carries `slic3r-execution`, `slic3r-libslic3r` and `slic3r-point`
//! and no `slic3r-trianglemesh`), and `port/cut.tsv:1082` runs the dependency the other
//! direction -- `slic3r-trianglemesh` depends on `slic3r-meshsplitimpl`, not the reverse -- so
//! adding `slic3r-trianglemesh` here would close a cycle, not resolve one. [`IndexedTriangleSet`]
//! below is therefore still this task's own stand-in, not foundation-19's real type: it is a
//! guess (admesh's `struct indexed_triangle_set`, `deps_src/admesh/stl.h:219`, also carries a
//! third field, `properties`, that this stand-in omits because no function in this header reads
//! it), named here only so the translation below has something to type-check against for review,
//! and it must be deleted in favour of whichever crate ends up owning the real type -- either
//! `foundation-19`'s `slic3r-trianglemesh` once this crate can depend on it, or this module made
//! generic over a mesh trait instead (the shape the vendored `ItsWithNeighborsIndex_<Its>`
//! template already has), whichever a regeneration or amendment pull request chooses. **Still
//! named as a finding in this task's pull request.**
//!
//! **Correction (second review pass): the translation below is not complete.**
//! [`NeighborVisitor`], [`its_split`]/[`its_split_collect`], [`its_is_splittable`] and
//! [`its_number_of_patches`] are translated in full. [`create_face_neighbors_index`]'s body is
//! **not translated at all** -- it is written at this file's final path with a `PORT STATUS`
//! trailer, exactly as `docs/WORKFLOW.md`'s "Unit of work"/"The two phases" ask of a draft, and
//! its `// TODO(port):` marker names the three missing items (`VertexFaceIndex`,
//! `its_triangle_edge`, `its_triangle_vertex_index`), but the function's own body is a bare
//! `todo!()`: no per-edge loop (`:252`), no `vertex_triangles` scan (`:260`), no
//! `other_face <= face_idx` skip (`:261`), no opposite-direction test (`:267`) and no
//! `neighbors[other_face][vertex_index]` write (`:275`) is present. The module is left unreached
//! by any `mod` line so `cargo build --workspace` never reads it and `cargo fmt --check` never
//! visits it (`docs/WORKFLOW.md`, "The two phases") -- `just port-status` derives this row
//! `drafted`. **Named as a finding in this task's pull request**, together with the `mod` line,
//! the `// PROOF: 2` marker and the real `IndexedTriangleSet` import the task that adds
//! `slic3r-trianglemesh` as a dependency (or that regenerates `port/cut.tsv` to include it)
//! should give this module.
//!
//! **`Vec3i32`/`Vec2i32`** (the Eigen vector aliases `Point.hpp:52-69` hand-designs,
//! `55-foundation-libslic3r-execution`'s sibling foundation task) are real: `slic3r-point` is a
//! dependency of this crate, so [`Vec3i32`] below is `slic3r_point::point::Vec3i32` re-exported,
//! not a second spelling.
//!
//! **Composition for the two neighbor-index wrapper templates, not inheritance** -- the same
//! move `sparse_grid.rs`'s module doc states once for the whole port's answer to C++ base
//! classes Rust has no counterpart for.

pub use slic3r_point::point::Vec3i32;

// TODO(port): src/libslic3r/TriangleMesh.hpp -- `indexed_triangle_set` (used throughout this
// header: `ItsWithNeighborsIndex_<indexed_triangle_set>`, `its_split`, `create_face_neighbors_index`,
// ...). `template-heavy: yes` in `port/manifest.tsv:575`; owned by `foundation-19`
// (`port/foundation.tsv:20`), a sibling step-20 task, not a fan-out module this task could
// write around (see the module doc's correction). Everywhere this module doc says
// `IndexedTriangleSet` below is that type, standing in so this draft's shape is reviewable
// before the real type exists -- not foundation-19's own spelling, and missing the real type's
// third field (`properties`, `deps_src/admesh/stl.h:219`), which no function below reads.
pub struct IndexedTriangleSet {
    pub vertices: Vec<slic3r_point::point::Vec3f>,
    pub indices: Vec<Vec3i32>,
}

// TODO(port): src/libslic3r/TriangleMesh.hpp -- `VertexFaceIndex` (used by
// `create_face_neighbors_index`, `:241`), `its_triangle_edge`/`its_triangle_vertex_index`
// (`:258,263`) and `stl_triangle_vertex_indices` (`:235`, an alias for `Vec3i32` at the pin) --
// all fan-out surface of `slic3r-trianglemesh`, named here rather than guessed at.

/// `template<class Its, class Enable = void> struct ItsWithNeighborsIndex_` (`:30-34`), and its
/// `indexed_triangle_set` specialisation (`:37-44`) -- one trait, implemented once for
/// [`IndexedTriangleSet`] itself (the specialisation) and once per wrapper type
/// ([`ItsNeighborsWrapper`] below, the generic case), the same "specialise the trait on the
/// concrete type" move `marching_squares.rs`'s `RasterTraits` uses for `_RasterTraits<T>`.
pub trait ItsWithNeighborsIndex {
    type Index;
    /// `static const indexed_triangle_set &get_its(const Its &m);` (`:32`, `:39`).
    fn its(&self) -> &IndexedTriangleSet;
    /// `static const Index &get_index(const Its &m);` (`:33`); the `indexed_triangle_set`
    /// specialisation's own `get_index` (`:40-43`) returns an owned `Index` by value instead
    /// (`create_face_neighbors_index(ex_tbb, its)`'s return), so this trait's method does too --
    /// a caller that wants to avoid recomputing it keeps the result itself, exactly as
    /// `ItsNeighborsWrapper` below does by holding `index` as a field rather than calling this
    /// method repeatedly.
    fn index(&self) -> Vec<Vec3i32>;
}

/// The `indexed_triangle_set` specialisation (`:37-44`): `get_its` is the identity, `get_index`
/// builds a fresh neighbor index via [`create_face_neighbors_index`] over the sequential
/// execution policy's counterpart of `ex_tbb` (`:42`'s real argument).
impl ItsWithNeighborsIndex for IndexedTriangleSet {
    type Index = Vec<Vec3i32>;
    fn its(&self) -> &IndexedTriangleSet {
        self
    }
    fn index(&self) -> Vec<Vec3i32> {
        create_face_neighbors_index(&slic3r_execution::execution_tbb::ExecutionTbb, self)
    }
}

/// `template<class NeighborIndex> struct NeighborVisitor` (`:48-105`): discovers connected
/// patches of facets one at a time by a depth-first flood fill over `neighbor_index`, resuming
/// from `m_seed` on each call to [`NeighborVisitor::visit`] so repeated calls walk successive
/// patches.
pub struct NeighborVisitor<'a> {
    its: &'a IndexedTriangleSet,
    neighbor_index: &'a [Vec3i32],
    /// `std::vector<char> m_visited;` (`:96`).
    m_visited: Vec<bool>,
    /// `std::vector<stack_el> m_facestack;` (`:99`), `stack_el = size_t` (`:98`).
    m_facestack: Vec<usize>,
    /// `size_t m_seed { 0 };` (`:104`).
    m_seed: usize,
}

impl<'a> NeighborVisitor<'a> {
    /// `NeighborVisitor(const indexed_triangle_set &its, const NeighborIndex &neighbor_index)`
    /// (`:49-53`).
    pub fn new(its: &'a IndexedTriangleSet, neighbor_index: &'a [Vec3i32]) -> Self {
        NeighborVisitor {
            its,
            neighbor_index,
            m_visited: vec![false; its.indices.len()],
            m_facestack: Vec::with_capacity(its.indices.len()),
            m_seed: 0,
        }
    }

    fn push(&mut self, s: usize) {
        self.m_facestack.push(s);
    }

    fn pop(&mut self) -> usize {
        self.m_facestack.pop().expect("pop: m_facestack must not be empty")
    }

    /// `void visit(Visitor visitor)` (`:60-87`): walks one connected patch, calling `visitor`
    /// with each facet index in discovery order; `visitor` returns `false` to stop the walk
    /// early (used by `its_is_splittable` to check only whether a second patch exists, without
    /// walking it fully).
    pub fn visit(&mut self, mut visitor: impl FnMut(usize) -> bool) {
        // `auto facet = std::find(m_visited.begin() + m_seed, m_visited.end(), false); m_seed =
        // facet - m_visited.begin();` (`:64-65`).
        let facet = self.m_visited[self.m_seed..].iter().position(|&v| !v);

        if let Some(offset) = facet {
            self.m_seed += offset;
            let idx = self.m_seed;
            self.m_seed += 1;
            if !visitor(idx) {
                return;
            }
            self.push(idx);
            self.m_visited[idx] = true;
            while !self.m_facestack.is_empty() {
                let facet_idx = self.pop();
                for neighbor_idx in self.neighbor_index[facet_idx].into_iter() {
                    debug_assert!((neighbor_idx as usize) < self.m_visited.len() || neighbor_idx < 0);
                    if neighbor_idx >= 0 && !self.m_visited[neighbor_idx as usize] {
                        if !visitor(neighbor_idx as usize) {
                            return;
                        }
                        self.m_visited[neighbor_idx as usize] = true;
                        self.push(neighbor_idx as usize);
                    }
                }
            }
        }
    }
}

/// `template<class IndexT> struct ItsNeighborsWrapper` (`:110-124`): pairs an
/// [`IndexedTriangleSet`] with an already-built neighbor index, by reference or by ownership --
/// the "funky wrapper for timing `its_split()` using various neighbor index creating methods"
/// the vendored comment (`:109`) names.
pub struct ItsNeighborsWrapper<'a> {
    its: &'a IndexedTriangleSet,
    /// `const IndexT &index_ref;` (`:114`) together with `const IndexT index;` (`:115`): the
    /// by-reference constructor (`:118`) leaves `index` default and reads through `index_ref`;
    /// the by-value constructor (`:120`) moves into `index` and still reads through `index_ref`,
    /// which it points at its own `index` field. Rust has no field-aliasing trick to carry that
    /// over, so this is `Cow`'s own borrowed/owned split -- the same shape, not a second
    /// behaviour.
    index: std::borrow::Cow<'a, [Vec3i32]>,
}

impl<'a> ItsNeighborsWrapper<'a> {
    /// `ItsNeighborsWrapper(const indexed_triangle_set &its, const IndexT &index)` (`:118`).
    pub fn borrowed(its: &'a IndexedTriangleSet, index: &'a [Vec3i32]) -> Self {
        ItsNeighborsWrapper { its, index: std::borrow::Cow::Borrowed(index) }
    }

    /// `ItsNeighborsWrapper(const indexed_triangle_set &its, IndexT &&aindex)` (`:120`).
    pub fn owned(its: &'a IndexedTriangleSet, index: Vec<Vec3i32>) -> Self {
        ItsNeighborsWrapper { its, index: std::borrow::Cow::Owned(index) }
    }
}

impl<'a> ItsWithNeighborsIndex for ItsNeighborsWrapper<'a> {
    type Index = Vec<Vec3i32>;
    /// `const auto &get_its() const noexcept { return its; }` (`:122`).
    fn its(&self) -> &IndexedTriangleSet {
        self.its
    }
    /// `const auto &get_index() const noexcept { return index_ref; }` (`:123`).
    fn index(&self) -> Vec<Vec3i32> {
        self.index.to_vec()
    }
}

/// `template<class Fn> struct SplitOutputFn` (`:129-139`): an output iterator that applies `fn`
/// to each split-off mesh instead of collecting them into a container -- Rust's closest
/// counterpart to a C++ output iterator with a custom `operator=` is simply calling the closure
/// directly, so [`its_split`] below takes an `OutputIt` bound by `FnMut(IndexedTriangleSet)`
/// rather than porting the iterator-adapter shape itself; a caller that wants collection uses
/// `Vec::push` as that closure, which is what [`its_split_collect`] does.
pub type SplitOutputFn<F> = F;

/// `template<class Its, class OutputIt> void its_split(const Its &m, OutputIt out_it)`
/// (`:142-191`): splits `m` into its connected patches, handing each one to `out_it` as it is
/// found.
pub fn its_split<Its: ItsWithNeighborsIndex>(m: &Its, mut out_it: impl FnMut(IndexedTriangleSet)) {
    let its = m.its();

    /// `struct VertexConv { size_t part_id = ...max(); size_t vertex_image; };` (`:149-152`).
    #[derive(Clone, Copy)]
    struct VertexConv {
        part_id: usize,
        vertex_image: usize,
    }

    let mut vidx_conv = vec![
        VertexConv { part_id: usize::MAX, vertex_image: 0 };
        its.vertices.len()
    ];

    let index = m.index();
    let mut visitor = NeighborVisitor::new(its, &index);

    let mut facets: Vec<usize> = Vec::new();
    let mut part_id: usize = 0;
    loop {
        facets.clear();
        visitor.visit(|idx| {
            facets.push(idx);
            true
        });
        if facets.is_empty() {
            break;
        }

        // `mesh.vertices.reserve(std::min(facets.size() * 3, its.vertices.size()));` (`:168`).
        let mut mesh_vertices: Vec<slic3r_point::point::Vec3f> = Vec::with_capacity(
            std::cmp::min(facets.len() * 3, its.vertices.len()), // PORT SRC: src/libslic3r/MeshSplitImpl.hpp:168
        );
        let mut mesh_indices: Vec<Vec3i32> = Vec::with_capacity(facets.len());

        for &face_id in &facets {
            let face = its.indices[face_id];
            // `Vec3i32 new_face;` initialised by the loop below, which runs over exactly the
            // triangle's three vertices (`for (size_t v = 0; v < 3; ++v)`, `:174`).
            // PORT SRC: src/libslic3r/MeshSplitImpl.hpp:174
            let mut new_face = [0i32; 3];
            // `for (size_t v = 0; v < 3; ++v)` (`:174`).
            // PORT SRC: src/libslic3r/MeshSplitImpl.hpp:174
            for v in 0..3 {
                let vi = face[v] as usize;

                if vidx_conv[vi].part_id != part_id {
                    vidx_conv[vi] = VertexConv { part_id, vertex_image: mesh_vertices.len() };
                    mesh_vertices.push(its.vertices[vi]);
                }

                new_face[v] = vidx_conv[vi].vertex_image as i32;
            }

            mesh_indices.push(Vec3i32::from(new_face));
        }

        out_it(IndexedTriangleSet { vertices: mesh_vertices, indices: mesh_indices });
        part_id += 1;
    }
}

/// `template<class Its> std::vector<indexed_triangle_set> its_split(const Its &its)`
/// (`:193-200`): `its_split(its, std::back_inserter(ret))` over a 3-element-reserved `Vec`.
pub fn its_split_collect<Its: ItsWithNeighborsIndex>(its: &Its) -> Vec<IndexedTriangleSet> {
    // `auto ret = reserve_vector<indexed_triangle_set>(3);` (`:196`).
    // PORT SRC: src/libslic3r/MeshSplitImpl.hpp:196
    let mut ret = Vec::with_capacity(3);
    its_split(its, |mesh| ret.push(mesh));
    ret
}

/// `template<class Its> bool its_is_splittable(const Its &m)` (`:202-214`): `true` only when a
/// *second* patch exists, found by walking the first patch fully, then checking (without fully
/// walking) whether the second visit finds any facet at all.
pub fn its_is_splittable<Its: ItsWithNeighborsIndex>(m: &Its) -> bool {
    let index = m.index();
    let mut visitor = NeighborVisitor::new(m.its(), &index);
    let mut has_some = false;
    let mut has_some2 = false;
    visitor.visit(|_idx| {
        has_some = true;
        true
    });
    if has_some {
        visitor.visit(|_idx| {
            has_some2 = true;
            false
        });
    }
    has_some && has_some2
}

/// `template<class Its> size_t its_number_of_patches(const Its &m)` (`:216-230`): counts
/// complete patches by repeatedly walking the next one until none remains.
pub fn its_number_of_patches<Its: ItsWithNeighborsIndex>(m: &Its) -> usize {
    let index = m.index();
    let mut visitor = NeighborVisitor::new(m.its(), &index);
    let mut num_patches = 0usize;
    loop {
        let mut has_some = false;
        visitor.visit(|_idx| {
            has_some = true;
            true
        });
        if !has_some {
            break;
        }
        num_patches += 1;
    }
    num_patches
}

/// `template<class ExPolicy> std::vector<Vec3i32> create_face_neighbors_index(ExPolicy &&ex,
/// const indexed_triangle_set &its)` (`:232-282`): one neighboring-face index per facet, `-1`
/// where an edge has no (or not-yet-discovered) neighbor.
// TODO(port): src/libslic3r/TriangleMesh.hpp -- `VertexFaceIndex` (`:241`), `its_triangle_edge`
// (`:258`) and `its_triangle_vertex_index` (`:263`). Not translated below: this function's body
// needs all three names to write the real algorithm (the per-edge loop at `:252`, the
// `vertex_triangles` scan at `:260`, the `other_face <= face_idx` skip at `:261`, the opposite-
// direction test at `:267` and the `neighbors[other_face][vertex_index]` write at `:275`), and
// none of it is attempted here -- a bare `todo!()` stands in for the whole loop body, not a
// partial or best-effort translation, per the module doc's own finding.
pub fn create_face_neighbors_index<P: slic3r_execution::execution::ExecutionPolicy>(
    _ex: &P,
    its: &IndexedTriangleSet,
) -> Vec<Vec3i32> {
    let indices = &its.indices;

    if indices.is_empty() {
        return Vec::new();
    }
    debug_assert!(!its.vertices.is_empty());

    // `static constexpr int no_value = -1;` (`:242`).
    const NO_VALUE: i32 = -1;
    let mut neighbors: Vec<Vec3i32> =
        vec![Vec3i32::from([NO_VALUE, NO_VALUE, NO_VALUE]); indices.len()];

    // TODO(port): `VertexFaceIndex{its}` (`:241`) and, inside the loop below,
    // `its_triangle_edge`/`its_triangle_vertex_index` (`:258,263`) -- all `slic3r-trianglemesh`
    // surface this crate cannot name (see the module doc).
    for face_idx in 0..indices.len() {
        let _ = face_idx;
        let _neighbor: &mut Vec3i32 = &mut neighbors[face_idx];
        todo!(
            "blocked on VertexFaceIndex/its_triangle_edge/its_triangle_vertex_index -- see the \
             module doc and this task's pull request"
        );
    }

    neighbors
}

#[cfg(test)]
mod tests {
    // Unreachable until `slic3r-trianglemesh` becomes a dependency of this crate and the real
    // `IndexedTriangleSet`/`VertexFaceIndex`/`its_triangle_edge`/`its_triangle_vertex_index`
    // exist -- see the module doc and this task's pull request. Left here, in position, so the
    // translation is reviewable now rather than written only once it compiles (the same choice
    // `sparse_grid.rs`'s own test module makes).
    //
    // Proof path 2 (R19, manifest row `MeshSplitImpl.hpp`): once reachable, through the per-unit
    // bridge a Phase A/B task adds (task.md's own precedent: `docs/PORTING.md`, "Decisions
    // recorded on 2026-11-25", decision 1) at
    // `crates/slic3r-ffi/cxx/proof_units/mesh_split_impl.{h,cpp}` and
    // `crates/slic3r-ffi/src/proof_units/mesh_split_impl.rs`, instantiating `its_split`/
    // `its_is_splittable`/`its_number_of_patches` over a small, real `indexed_triangle_set`.
    // PROOF: 2
    //
    // No `#[ignore]` here: this file carries no `mod` line (see the module doc above), so
    // `cargo test --workspace` never compiles this test at all today -- an `#[ignore]` would do
    // nothing now and would silently skip a real compile error later, the same reasoning
    // `sparse_grid.rs`'s own test module states for the same situation. Written as a genuine
    // differential (an `oracle` value bound from a `proof_units::` path, compared against this
    // port's own `actual` through `assert_eq!`) rather than a placeholder body, for the same
    // reason `sparse_grid.rs`'s own unreachable test is: so the shape is reviewable now, and so
    // `port/port-status`'s differential check reads it the same way it will once this module
    // compiles, rather than reporting a `proof-marker-not-differential` finding against a
    // module that is honestly blocked, not dishonestly unproven.
    #[test]
    fn mesh_split_impl_matches_the_oracle() {
        use super::*;

        let its = IndexedTriangleSet {
            vertices: vec![
                slic3r_point::point::Vec3f::from([0.0, 0.0, 0.0]),
                slic3r_point::point::Vec3f::from([1.0, 0.0, 0.0]),
                slic3r_point::point::Vec3f::from([0.0, 1.0, 0.0]),
                slic3r_point::point::Vec3f::from([10.0, 10.0, 0.0]),
                slic3r_point::point::Vec3f::from([11.0, 10.0, 0.0]),
                slic3r_point::point::Vec3f::from([10.0, 11.0, 0.0]),
            ],
            indices: vec![Vec3i32::from([0, 1, 2]), Vec3i32::from([3, 4, 5])],
        };

        let actual = its_number_of_patches(&its);
        let oracle = slic3r_proof::proof_units::mesh_split_impl::path2_number_of_patches(
            &its.vertices,
            &its.indices,
        );
        assert_eq!(actual, oracle, "its_number_of_patches must match the oracle exactly");
    }
}

// PORT STATUS
// source: src/libslic3r/MeshSplitImpl.hpp
// confidence: low
// tier: exact
// oracle: built
