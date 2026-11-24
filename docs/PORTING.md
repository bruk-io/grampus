# The per-file translation contract

`just port-status` reads the six headings below to check that this document says what it must
(R29); renaming one breaks that check. This document is committed before any unit is ported.
Every unit of Phase A and Phase B — port/manifest.tsv's roughly 420 in-scope files — is held to
it, and a porter is given their task, this repository and nothing else: this document is written
so a porter never has to guess and never has to ask. `docs/WORKFLOW.md` is the process this
document's rules are used inside; it states the phases, the review loop and the conflict
discipline, and is not restated here.

## Crate map

`port/cut.tsv` and `port/manifest.tsv`'s `target crate` column are the authority on which crate
a file belongs to; this section is derived from them, never authored, and lists no libslic3r
source file individually. Re-derive it with:

```
awk -F'\t' 'NR==1{for(i=1;i<=NF;i++)c[$i]=i} NR>1 {print $c["target crate"]}' port/manifest.tsv \
  | sort | uniq -c
```

against the `crate` rows of `port/cut.tsv` (`awk -F'\t' '$1=="crate"{print $2, $5}' port/cut.tsv`).
As this document is written, the two agree exactly: the same 126 crate names, and the same
in-scope file count per crate, summing to the 409 in-scope rows `port/manifest.tsv` carries. Plan
step 20's prose estimate of "roughly 43 crates" is stale against the committed cut; `port/cut.tsv`
is the authority, not the plan's prose.

**An agent may not invent a crate, add one, or move a file between crates.** The crate map changes
only through a pull request that regenerates `port/cut.tsv` and `port/manifest.tsv` — C10's
recorded decision, that the crate-level cut is a design choice owned by that regeneration, not by
a porting task. A porter who finds a file apparently misplaced does not move it; they say so in
their unit's pull request and leave the manifest row as it stands.

The table states the crate, its crate-level level (the cut's feedback-arc order; level 0 has no
in-cut dependency on another crate, and a crate at level *n* may depend only on crates at a lower
level), and the subsystem(s) (`port/cut.tsv`'s `modules` column) it holds. It does not restate the
cut's edges or any file's per-file scope decision — those are `port/cut.tsv`'s and
`port/manifest.tsv`'s rows, read directly.

| crate | level | subsystems | in-scope files |
|---|---|---|---|
| `slic3r-clonable-ptr` | 0 | clonable_ptr | 1 |
| `slic3r-commondefs` | 0 | CommonDefs | 1 |
| `slic3r-customgcode` | 0 | CustomGCode | 2 |
| `slic3r-enum-bitmask` | 0 | enum_bitmask | 1 |
| `slic3r-exception` | 0 | Exception | 1 |
| `slic3r-filamentmixermodel` | 0 | FilamentMixerModel | 1 |
| `slic3r-format` | 0 | format | 1 |
| `slic3r-i18n` | 0 | I18N | 1 |
| `slic3r-instancelock` | 0 | InstanceLock | 2 |
| `slic3r-int128` | 0 | Int128 | 1 |
| `slic3r-lifecycleevents` | 0 | LifecycleEvents | 1 |
| `slic3r-localesutils` | 0 | LocalesUtils | 2 |
| `slic3r-macutils` | 0 | MacUtils | 1 |
| `slic3r-materialtype` | 0 | MaterialType | 2 |
| `slic3r-mutablepriorityqueue` | 0 | MutablePriorityQueue | 1 |
| `slic3r-objectid` | 0 | ObjectID | 2 |
| `slic3r-platform` | 0 | Platform | 2 |
| `slic3r-pngreadwrite` | 0 | PNGReadWrite | 2 |
| `slic3r-printconfigconstants` | 0 | PrintConfigConstants | 1 |
| `slic3r-projecttask` | 0 | ProjectTask | 2 |
| `slic3r-technologies` | 0 | Technologies | 1 |
| `slic3r-thread` | 0 | Thread | 2 |
| `slic3r-time` | 0 | Time | 2 |
| `slic3r-timer` | 0 | Timer | 2 |
| `slic3r-semver` | 1 | Semver | 2 |
| `slic3r-libslic3r` | 2 | libslic3r | 2 |
| `slic3r-color` | 3 | Color | 2 |
| `slic3r-execution` | 3 | Execution/ | 3 |
| `slic3r-mtutils` | 3 | MTUtils | 1 |
| `slic3r-point` | 3 | Point | 2 |
| `slic3r-slicingadaptive` | 3 | SlicingAdaptive | 2 |
| `slic3r-utils` | 3 | utils | 2 |
| `slic3r-arachne-beadingstrategy` | 4 | Arachne/BeadingStrategy/ | 14 |
| `slic3r-boundingbox` | 4 | BoundingBox | 2 |
| `slic3r-brimearspoint` | 4 | BrimEarsPoint | 1 |
| `slic3r-config` | 4 | Config | 2 |
| `slic3r-flushvolpredictor` | 4 | FlushVolPredictor | 2 |
| `slic3r-kdtreeindirect` | 4 | KDTreeIndirect | 1 |
| `slic3r-marchingsquares` | 4 | MarchingSquares | 1 |
| `slic3r-meshsplitimpl` | 4 | MeshSplitImpl | 1 |
| `slic3r-minimumspanningtree` | 4 | MinimumSpanningTree | 2 |
| `slic3r-trianglesetsampling` | 4 | TriangleSetSampling | 2 |
| `slic3r-aabbtreeindirect` | 5 | AABBTreeIndirect | 1 |
| `slic3r-filamentmixer` | 5 | FilamentMixer | 2 |
| `slic3r-line` | 5 | Line | 2 |
| `slic3r-slicing` | 5 | Slicing | 2 |
| `slic3r-aabbtreelines` | 6 | AABBTreeLines | 1 |
| `slic3r-circle` | 6 | Circle | 2 |
| `slic3r-colordecomposerecipe` | 6 | ColorDecomposeRecipe | 2 |
| `slic3r-multipoint` | 6 | MultiPoint | 2 |
| `slic3r-arcfitter` | 7 | ArcFitter | 2 |
| `slic3r-polygon` | 7 | Polygon | 2 |
| `slic3r-polyline` | 8 | Polyline | 2 |
| `slic3r-principalcomponents2d` | 8 | PrincipalComponents2D | 2 |
| `slic3r-printconfig` | 8 | PrintConfig | 2 |
| `slic3r-expolygon` | 9 | ExPolygon | 2 |
| `slic3r-extruder` | 9 | Extruder | 2 |
| `slic3r-extrusionentitycollection` | 9 | ExtrusionEntityCollection | 2 |
| `slic3r-flow` | 9 | Flow | 2 |
| `slic3r-gcodereader` | 9 | GCodeReader | 2 |
| `slic3r-geometry` | 9 | Geometry | 2 |
| `slic3r-multinozzleutils` | 9 | MultiNozzleUtils | 2 |
| `slic3r-parameterutils` | 9 | ParameterUtils | 2 |
| `slic3r-belttransform` | 10 | BeltTransform | 2 |
| `slic3r-expolygoncollection` | 10 | ExPolygonCollection | 2 |
| `slic3r-extrusionentity` | 10 | ExtrusionEntity | 2 |
| `slic3r-filamentgrouputils` | 10 | FilamentGroupUtils | 2 |
| `slic3r-intersectionpoints` | 10 | IntersectionPoints | 2 |
| `slic3r-mutablepolygon` | 10 | MutablePolygon | 2 |
| `slic3r-placeholderparser` | 10 | PlaceholderParser | 2 |
| `slic3r-surface` | 10 | Surface | 2 |
| `slic3r-tesselate` | 10 | Tesselate | 2 |
| `slic3r-filamentgroup` | 11 | FilamentGroup | 2 |
| `slic3r-shortestpath` | 11 | ShortestPath | 2 |
| `slic3r-trianglemeshslicer` | 11 | TriangleMeshSlicer | 2 |
| `slic3r-triangulation` | 11 | Triangulation | 2 |
| `slic3r-variablewidth` | 11 | VariableWidth | 2 |
| `slic3r-clipperutils` | 12 | ClipperUtils | 2 |
| `slic3r-trianglemesh` | 12 | TriangleMesh | 2 |
| `slic3r-bridgedetector` | 13 | BridgeDetector | 2 |
| `slic3r-buildvolume` | 13 | BuildVolume | 2 |
| `slic3r-clipperzutils` | 13 | ClipperZUtils | 2 |
| `slic3r-imexhelpers` | 13 | IMEXHelpers | 2 |
| `slic3r-meshrepair` | 13 | MeshRepair | 2 |
| `slic3r-sla` | 13 | SLA/ | 6 |
| `slic3r-svg` | 13 | SVG | 2 |
| `slic3r-triangleselector` | 13 | TriangleSelector | 2 |
| `slic3r-aabbmesh` | 14 | AABBMesh | 2 |
| `slic3r-algorithm` | 14 | Algorithm/ | 4 |
| `slic3r-arachne-utils` | 14 | Arachne/utils/ | 16 |
| `slic3r-edgegrid` | 14 | EdgeGrid | 2 |
| `slic3r-gcodewriter` | 14 | GCodeWriter | 2 |
| `slic3r-imexzones` | 14 | IMEXZones | 2 |
| `slic3r-surfacecollection` | 14 | SurfaceCollection | 2 |
| `slic3r-beltgcode` | 15 | BeltGCode | 2 |
| `slic3r-calib` | 15 | calib | 2 |
| `slic3r-elephantfootcompensation` | 15 | ElephantFootCompensation | 2 |
| `slic3r-multimaterialsegmentation` | 15 | MultiMaterialSegmentation | 2 |
| `slic3r-appconfig` | 16 | AppConfig | 2 |
| `slic3r-geometry-dir` | 16 | Geometry/ | 19 |
| `slic3r-layer` | 16 | Layer | 2 |
| `slic3r-arachne` | 17 | Arachne/ | 8 |
| `slic3r-beltbrim` | 17 | BeltBrim | 2 |
| `slic3r-connectedbodies` | 17 | ConnectedBodies | 2 |
| `slic3r-exclusionvolumegeometry` | 17 | ExclusionVolumeGeometry | 2 |
| `slic3r-feature-fuzzyskin` | 17 | Feature/FuzzySkin/ | 2 |
| `slic3r-model` | 17 | Format/, Model | 4 |
| `slic3r-beltslicestrategy` | 18 | BeltSliceStrategy | 2 |
| `slic3r-normalutils` | 18 | NormalUtils | 2 |
| `slic3r-perimetergenerator` | 18 | PerimeterGenerator | 2 |
| `slic3r-printbase` | 18 | PrintBase | 2 |
| `slic3r-brim` | 19 | Brim | 2 |
| `slic3r-shortedgecollapse` | 19 | ShortEdgeCollapse | 2 |
| `slic3r-print` | 20 | Print | 2 |
| `slic3r-contourz` | 21 | ContourZ | 1 |
| `slic3r-fill-lightning` | 21 | Fill/Lightning/ | 8 |
| `slic3r-layerregion` | 21 | LayerRegion | 1 |
| `slic3r-printapply` | 21 | PrintApply | 1 |
| `slic3r-printregion` | 21 | PrintRegion | 1 |
| `slic3r-fill` | 22 | Fill/ | 36 |
| `slic3r-support` | 22 | Support/ | 17 |
| `slic3r-feature-interlocking` | 23 | Feature/Interlocking/ | 4 |
| `slic3r-gcode` | 23 | GCode, GCode/ | 71 |
| `slic3r-printobject` | 23 | PrintObject | 1 |
| `slic3r-beltpurge` | 24 | BeltPurge | 1 |
| `slic3r-printobjectslice` | 24 | PrintObjectSlice | 1 |

## Type map

One row per C++ type a porter meets, with the Rust type and the authority for it.

| C++ type | Rust type | Authority |
|---|---|---|
| `coord_t` | `i64` | `src/libslic3r/libslic3r.h:48` is the live definition; the `int32_t` branch at `:45` is inside an `#if 0` and is not live. Nothing in the port makes `coord_t` `i32`. |
| `coordf_t` | `f64` | `src/libslic3r/libslic3r.h:51`. |
| `float` | `f32` | See `### Float widths` below. |
| `double` | `f64` | See `### Float widths` below. |
| `size_t` | `usize` | — |
| `std::string` | `String` | — |
| `std::optional` | `Option` | — |
| `std::variant` | an enum | — |
| `std::map` (key-ordered) | a key-ordered map | — |
| `std::vector` | `Vec` | — |
| `enum class` | an enum keeping the original's discriminant values | They reach serialised output. |
| a raw-pointer field, or a container of raw pointers | the Rust type `port/ownership.tsv` gives, **taken verbatim** | See "Raw-pointer fields" below; this is R30's artifact and this instruction is what makes it binding. |
| `Point`, the Eigen vector aliases (`src/libslic3r/Point.hpp:52-69`), `Polygon`, `ExPolygon`, the configuration types | the spelling the crate map's crate and module export | See "Foundation types" below. |
| `std::unordered_map`, `std::unordered_set`, `ankerl::unordered_dense::map`, `tbb::concurrent_unordered_map`, `tbb::concurrent_unordered_set` — wherever iteration order reaches output | one insertion-ordered map spelling, one insertion-ordered set spelling, for the whole port | See "Insertion-ordered containers" below. |

**Raw-pointer fields.** For every raw-pointer field and every field that is a container of raw
pointers, the porter opens `port/ownership.tsv`, finds the row by header path and line, and takes
its `rust type` cell verbatim. A porter never chooses a pointer's Rust type independently of that
file, because `port/ownership.tsv` is R30's committed classification and this document only points
at it.

The column is filled in two different ways, and the two cases are not interchangeable, because
only one of them has a Rust type to take yet:

- **A row whose `class` cell is `unknown`:** the row names a conservative Rust type, and
  `just ownership-check` fails an `unknown` row that carries no conservative Rust type — so this
  cell is guaranteed present. Take it.
- **A row whose `class` cell is still blank:** `16-ownership-spike` deliberately leaves no `class`
  cell filled, and no check requires a blank-class row's `rust type` cell to be filled, so that
  cell is guaranteed *empty* for the whole window the fan-out drafts are written in (the window
  `12`'s classification tasks, cut from `port/ownership-batches.tsv`, are still open). **No module
  may invent a Rust type for a blank-class row.** Instead, leave the field with a
  `// TODO(port): <header path>:<line> — Rust type pending the classification task for this row's
  batch (port/ownership-batches.tsv)` marker, and move on; the step-12 classification task cut for
  that row's batch fills the cell in, later, on `main`.

As this document is written, every row of `port/ownership.tsv` has a blank `class` cell (there are
no `unknown` rows yet), so the `// TODO(port):` case is, today, every row. That is expected: it is
the state the window before fan-out is in, not an error in the ownership manifest.

**Foundation types.** `Point`, the Eigen vector aliases at `src/libslic3r/Point.hpp:52-69`
(`Vec2crd`, `Vec3crd`, `Vec2i32`, `Vec2i64`, `Vec3i32`, `Vec3i64`, `Vec4i32`, `Vec2f`, `Vec3f`,
`Vec4f`, `Vec2d`, `Vec3d`, `Vec4d`), `Polygon`, `ExPolygon` and the configuration types are
hand-designed once, before fan-out, by plan step 20 (cut as `34-foundation-spike` and the
foundation tasks it batches) — not by this document. A porter does not invent a second spelling:
they find the type's crate from the crate map above (`slic3r-point` for `Point` and the Eigen
aliases, `slic3r-polygon` for `Polygon`, `slic3r-expolygon` for `ExPolygon`, `slic3r-config` and
`slic3r-printconfig` for the configuration types) and use the spelling that crate's module exports.

**Insertion-ordered containers.** `17-order-audits` found five hash-keyed container shapes whose
iteration order may reach compared output: `std::unordered_map`, `std::unordered_set`,
`ankerl::unordered_dense::map`, `tbb::concurrent_unordered_map` and `tbb::concurrent_unordered_set`
(`port/reports/order-containers.md`). The port gives these **one** insertion-ordered map spelling
and **one** insertion-ordered set spelling for the whole port, regardless of which of the five
shapes the original site used: `indexmap::IndexMap` and `indexmap::IndexSet`, from the `indexmap`
crate. A port task may not add a dependency (`Cargo.lock` is one of the seven paths
`docs/WORKFLOW.md` says a port task does not write), so `indexmap` is added once, as a workspace
dependency, by `34-foundation-spike` — the crate-skeleton task of plan step 20 — not by a fan-out
task that first needs it.

The two `tbb::concurrent_*` shapes are concurrent containers in the original, built for parallel
insertion from a `tbb::parallel_for` or `tbb::parallel_reduce` body; `port/reports/order-containers.md`
already resolves them to the same insertion-ordered spelling as the non-concurrent shapes, because
the port's corresponding loop is sequential (see `### Linked libraries` under `## Rules` and the
TBB row of the idiom map below) and so needs no concurrent container to begin with — a single
`IndexMap`/`IndexSet` built up in the fixed order the port chooses is both correct and simpler than
a concurrent container the port has no parallel writer for.

`port/reports/order-containers.md` is the per-site authority for which existing sites need this:
read it for the declaration, the declaring scope and whether the port's choice there diverges from
the oracle (every row in the committed report currently does, and is marked `// FIDELITY(port):`
accordingly — see `## Banned-substitution list`).

## Idiom map

One row per C++ construct, with the single rendering the whole port uses — a porter who chooses
differently changes every call site.

| Construct | Rendering |
|---|---|
| A method on a class | A method on the corresponding Rust type (an inherent method, or a trait method where the crate map's module defines a trait for it). |
| A free function in an anonymous namespace, or a `static` free function | A private (non-`pub`) free function in the module. It is **not** made `pub` to ease testing: proof path 3 (R19) — a test translation unit inside the FFI crate that `#include`s the vendored `.cpp` — is how such a function is proven equal, not visibility. |
| A non-const reference out-parameter | A return value, or a tuple of return values, from the Rust function — never a `&mut` parameter adopted just to mirror the C++ signature. |
| A destructor | `Drop`, implemented only where the destructor does more than drop its fields (an RAII release, a log, a cache invalidation); a destructor that only frees owned sub-objects needs no explicit `Drop` impl, because Rust already drops them. |
| Virtual dispatch over a class hierarchy (e.g. `ExtrusionEntity`) | A Rust trait with one implementing type per concrete C++ subclass, dispatched through `&dyn Trait` or an enum, whichever the foundation design for that hierarchy (`slic3r-extrusionentity`, hand-designed by plan step 20) chooses; a port task does not introduce a second dispatch mechanism for the same hierarchy. |
| A template | A generic function or type with the same bound, monomorphised the same way the C++ template's instantiations are used in in-scope code; a template is not force-collapsed to one concrete type unless the manifest row's unit only ever instantiates one. |
| A `#define` numeric constant | A `const` item, carrying a `// PORT SRC: <vendored path>:<line>` marker beside it naming the macro's definition. |
| `std::sort` | `slice::sort_unstable` (no ordering guarantee between equal elements, matching `std::sort`'s own lack of a stability guarantee). |
| `std::stable_sort` | `slice::sort` (stable, matching `std::stable_sort`). |
| `tbb::parallel_for`, `tbb::parallel_reduce` | See `### Linked libraries` below: TBB itself is not reimplemented. Where the original's reduction order is load-bearing (R18), the port sums in a fixed order and is serial, and the module carries a `// PERF(port):` marker and nothing else — that marker is the only place a deferred optimization may live (R15). Where the manifest's divergence audit (`port/reports/order-containers.md`, `port/reports/order-comparators.md`) finds no load-bearing order at a given site, the port may still parallelise, through the same linked TBB, provided the result is order-independent; it is not ported as a sequential Rust loop "to be safe" where the audit found no hazard, because that is an unrecorded, unaudited divergence of its own. |
| Iterator and pointer arithmetic | Slices and indices — a C++ `T*` walked with `++`/`--`/offset arithmetic over a contiguous buffer becomes a `&[T]`/`&mut [T]` walked by index or by a `std::slice` iterator; a raw pointer that is not walking a contiguous Rust-owned buffer is instead a `port/ownership.tsv` row (see the type map). |
| `operator<<` (or other serialisation) | A `Display` or `Debug` impl where the output is for humans, or an explicit `write_...`/`format_...` function where the output is G-code or another emitted artifact whose exact bytes are compared — never `Display`/`{}` for G-code or other compared output, because `### Number formatting` below is specific about digit counts and padding that a derived or generic `Display` impl does not reproduce. |

## Don't-translate list

What a porter stops at rather than ports:

- **Out-of-scope code reached from an in-scope file.** The manifest row's reason (`port/manifest.tsv`,
  "in scope or out with a reason" column) is the authority for why; the porter adds a
  `// PORT NOTE:` marker at the boundary (the call or reference into the out-of-scope code) rather
  than following it in.
- **The libraries R17 binds**: Clipper2, admesh, libnest2d, qhull, Boost.Polygon's Voronoi builder,
  and NLopt only where the manifest marks an in-scope caller. They are called through
  `crates/slic3r-ffi` (see `### Linked libraries` below), not reimplemented.
- **Upstream code that is commented out.** Two sites this contract itself depends on:
  `src/libslic3r/Point.hpp:676` (`//return static_cast<Tout>(std::round(v / SCALING_FACTOR));`) and
  `src/libslic3r/libslic3r.h:105` (`//inline coord_t scale_(coordf_t v) { return coord_t(floor(v /
  SCALING_FACTOR + 0.5f)); }`). Neither is ported and neither is revived — see
  `### Scaling conversions` below for why the first matters to every scaled integer coordinate the
  port emits.
- **Debug-only blocks and SVG dumps behind `SLIC3R_DEBUG*` guards.**
- **The GUI and the plate layer** (R10): nothing under `src/slic3r/GUI/PartPlate`, and no arranger —
  R24 fixes placement in the fixture instead.
- **The preset layer and the project-format readers** (R11, and the manifest's deductions): preset
  files, the `inherits` chain, preset-bundle resolution, and `Format/` readers other than `STL`.
- **The SLA implementations**, except `SLA/IndexedMesh`, which **is** in scope (R9) because
  Z-contouring calls it (`ContourZ.cpp`, `AABBMesh.cpp`, `PrintObject.cpp`, `Layer.hpp`). The SLA
  fields carried on the model type are ported as data only; hollowing, pad and the concave-hull
  implementations are out.
- **Arrange and orient** (C14's decision): the application does call this code, but R24 fixes
  placement in the fixture and R2 passes `--arrange 0`, so no arranger runs in any comparison.
- **The two Objective-C++ (`.mm`) files in the file set**, each deducted in `port/manifest.tsv`
  with its own reason — so no porting rule in this document has to cover Objective-C++ at all.
- **An `assert` body.** The oracle's release build compiles assertions out (see
  `### Error sets` below), so an `assert` is not ported as a live check of any kind.

## Banned-substitution list

Each entry names what is banned and the check that catches it, where one exists.

| Banned | Check |
|---|---|
| Rounding a scaling conversion (`scaled<coord_t>()`, the Eigen `scaled()` overload, or any truncating entry point) | None automated yet; `### Scaling conversions` below is the rule a reviewer holds a unit to. |
| A hash-ordered map or set where iteration order reaches output, in place of the one insertion-ordered spelling this document fixes | `just ownership-check`/`just inventory-check` do not check this directly; `port/reports/order-containers.md`'s regeneration is where a reintroduced hash-ordered container at an audited site would be caught. |
| A parallel reduction where the original's order is load-bearing | `port/reports/order-containers.md` and `port/reports/order-comparators.md`'s regeneration (R18). |
| Replacing one of R17's six libraries (Clipper2, admesh, libnest2d, qhull, Boost.Polygon's Voronoi builder, NLopt) with a Rust crate | `just port-check`'s substitute check fails a crate manifest that depends on a named substitute. |
| Widening a `float` field to `f64`, or narrowing an `f64` field to `float`/`f32` | None automated yet; `### Float widths` below. |
| Any fused multiply-add, including `f64::mul_add` | None automated yet; the normative oracle is built with `-ffp-contract=off` and never fuses, so a port that fuses diverges from it. |
| Multiplying by the reciprocal of `SCALING_FACTOR` instead of dividing by it | None automated yet; `### Scaling conversions` below. |
| Locale-dependent or `format!`-based number formatting in emitted G-code | None automated yet; `### Number formatting` below. |
| Widening or narrowing an error set — adding a variant libslic3r's exception hierarchy has no sibling for, or collapsing two of its variants into one | None automated yet; `### Error sets` below. |
| Returning `Result` from a function that cannot fail, or a live `panic!` where the oracle compiles the assertion out | None automated yet; `### Error sets` below. |
| A comparator that is not a total order, substituted for one of the port's deliberate total-order comparators | None automated yet. This substitution is itself the port's **deliberate divergence** where the oracle's comparator is not a total order: the port's choice at such a site carries a `// FIDELITY(port):` marker, and that marker is what puts the site in `port/reports/divergences.md`. `17-order-audits` has already classified the comparator shapes found in in-scope files (`port/reports/order-comparators.md`): 85 sites are still `undecided` and are judged per-site during porting, and 15 already have a resolved shape — `tolerance`/EPSILON comparisons (4 sites: the port's choice is the same comparison with the tolerance removed, with the row's equality tier deciding equality instead), a non-strict relation (6 sites: the port's choice is the strict form of the same relation), and a comparator with disjunctive branches (5 sites: the port's choice is a nested lexicographic chain over the same fields in the same order). That report, not this document, is the per-site authority; a porter reads the row for their unit's comparator rather than re-deriving its shape. |

## Rules

### Scaling conversions

Stated per API, because the five entry points do not behave alike.

**Integral output — truncating, rounding banned.** `scaled<coord_t>()`, the integral overload
(`src/libslic3r/Point.hpp:669-678`, body at `:677`), is `Tout(v / Tin(SCALING_FACTOR))` with an
integral `Tout`: a division whose result a C-style cast converts to an integer, so it **truncates
toward zero**. Port it as truncating division. Adding rounding is banned, and the reason is in the
original: the `std::round` at `Point.hpp:676` is commented out, under the `TODO: is the rounding
necessary?` comment at `:670-672` that reads as an invitation — it is not one the port accepts. The
Eigen overload (`Point.hpp:680-690`, body at `:689`) is `(v / SCALING_FACTOR).template
cast<Tout>()`, which truncates per element the same way.

**Floating output — does not truncate.** The floating-output `scaled<Tout>()`
(`Point.hpp:660-667`, body at `:666`) is `Tout(v / Tin(SCALING_FACTOR))` with a floating-point
`Tout`, so the cast to `Tout` does not truncate.

**Each of these divides at the input's precision, and the port never widens the input to `f64`
first** — a porter who widens first diverges in the low bits at exactly the sites this rule exists
to protect. The global is cast to `Tin`, the *input's* type, at both `Point.hpp:666` and `:677`, so
`scaled<coord_t>(0.1f)` divides in `f32` and only then converts to the integral `Tout`; the Eigen
overload at `:689` divides by a bare `SCALING_FACTOR`, which Eigen's scalar `operator/` coerces to
the matrix's own `Scalar`, so a `Vec2f` likewise divides in `f32`. So the port divides a `f32`
input in `f32` and a `f64` input in `f64`.

**The two macros carry no cast, so the call site decides.** `scale_()` (`libslic3r.h:106`,
`#define scale_(val) ((val) / SCALING_FACTOR)`) and `unscale_()` (`:107`,
`#define unscale_(val) ((val) * SCALING_FACTOR)`) carry no cast at all, so their precision is
whatever the usual arithmetic conversions give at the call site — this document does not state one
precision for them; a porter reads the call site. `scale_()` is a plain `double` division and
truncates nothing (the rounding variant that precedes it, commented out at `:105`, is not ported —
see `## Don't-translate list`); where the original then converts the quotient to an integer type,
the port reproduces that truncation **at the call site**, not inside a shared API function. The
same holds for `unscale_()`, which never truncates (it is a multiplication).

**The multiplicative family never truncates.** `unscale_()` (`libslic3r.h:107`), `unscale<T>()`
(`libslic3r.h:137-138`), `unscaled<T>()` (`Point.hpp:692-700`, body at `:699`) and its Eigen
overload (`Point.hpp:702-714`, body at `:713`) are all multiplications by the global and never
truncate.

**In every case the port divides by the global and never multiplies by its reciprocal.**
Multiplying by `1.0 / SCALING_FACTOR` is a different floating-point operation from dividing by
`SCALING_FACTOR` and is banned (see `## Banned-substitution list`).

**The global itself is not a constant.** `SCALING_FACTOR` is `extern double` (`libslic3r.h:80`),
defined at `src/libslic3r/libslic3r.cpp:3` as `double SCALING_FACTOR = SCALING_FACTOR_INTERNAL;`,
where `SCALING_FACTOR_INTERNAL` is `0.000001` (`libslic3r.h:65`). The only assignments anywhere in
the vendored tree are `src/slic3r/GUI/Plater.cpp:15015` and `:15017`, which set the large-printer
value `SCALING_FACTOR_INTERNAL_LARGE_PRINTER`, `0.00001` (`libslic3r.h:66`), above
`LARGE_BED_THRESHOLD`, `2147` (`:67`) — code R10 puts out of scope (the GUI's `Plater`). So the
port's surface sees exactly `0.000001`. The port fixes one spelling for this value — a module does
not write the literal `0.000001` inline — and R7 already records `SCALING_FACTOR`'s value with
every oracle output, so a comparison is never run against an oracle output whose scale disagrees.

### Float widths

Mirror `f32` and `f64` exactly where the original uses `float` and `double`. Widening a `float`
field to `f64` (or narrowing an `f64` field) changes every emitted `E` value and every other value
derived from it; see `## Banned-substitution list`.

### Rounding modes

There is no universal rounding rule, and this document states none. The spec's own scoping is
exactly two places and no further: **ties away from zero in the G-code formatter and in Clipper2
point construction, ties to even in Clipper2 intersections, and every remaining site ported
individually**, each to the idiom that site's citation shows, not to a single rule applied
everywhere.

**Ties-away-from-zero sites, inside libslic3r, that reach the compared G-code:**

- `std::round` is ties away from zero, which is exactly what Rust's `f64::round` does. The two
  sites that reach the compared artifact are `int64_t(std::round(v * pow_10[digits]))` in
  `GCodeFormatter::emit_axis` (`src/libslic3r/GCodeWriter.cpp:1696`) and in
  `GCodeFormatter::quantize` (`src/libslic3r/GCodeWriter.hpp:341`). `GCodeWriter.cpp:782`'s
  `std::round` is **not** cited here because it is flavour-gated to MakerWare and Sailfish output,
  which is out of the port's scope; the `TextureDisplacement` and `SLA/` rounding sites are
  likewise out of scope.
- **`floor(x + 0.5)` is round-half-up, not round-half-away, and is not `f64::round`.** They differ
  at an exact negative half (`floor(-0.5 + 0.5)` is `0`, while `(-0.5f64).round()` is `-1.0`) and at
  `0.49999999999999994`. Transcribe it as `(x + 0.5).floor()`, never as `.round()`. Sites that reach
  the emitted G-code: `src/libslic3r/GCode.cpp:9467`, `(unsigned int)floor(acceleration + 0.5)` (the
  M204/M201 acceleration value); `src/libslic3r/GCode/CoolingBuffer.cpp:995`,
  `int(floor(60. * line->feedrate + 0.5))` (a rewritten `F` word); `src/libslic3r/GCode/WipeTower.cpp:1616`,
  `int(floor(f + 0.5f))` (another `F` word — note the `0.5f`: this one rounds in `f32`, not `f64`);
  `src/libslic3r/Config.hpp:1023`, `int(floor(_value + 0.5))` in `ConfigOptionInt`; and
  `src/libslic3r/Fill/FillLine.cpp:78`, `coord_t(floor(this->_min_spacing * INFILL_OVERLAP_OVER_SPACING + 0.5f))`
  (also an `f32`-precision round, from the `0.5f` literal).
- **`lround`/`llround` are ties away from zero**, like `std::round`, and appear where a cast would
  truncate: `src/libslic3r/BeltBrim.cpp:39-40` uses `std::llround` for exactly that reason.
- **`fast_round_up<I>()` (`src/libslic3r/libslic3r.h:427-433`) is its own thing**, and its body is
  the proof that `floor(a + 0.5)` is not round-half-away:
  `a == 0.49999999999999994 ? I(0) : I(floor(a + 0.5))`. Port it as that expression, special case
  included, and never as `f64::round` or as a bare `(a + 0.5).floor()` missing the special case.

**Ties-to-even sites, behind the FFI bridge, not ported directly:** Clipper2's point construction
uses `std::round` (`vendor/OrcaSlicer/deps_src/clipper2/Clipper2Lib/include/clipper2/clipper.core.h:132-133`
and `:186-187`), while its intersection arithmetic uses `nearbyint`
(`clipper.core.h:930-931`, `:1066-1067`, `vendor/OrcaSlicer/deps_src/clipper2/Clipper2Lib/src/clipper.engine.cpp:157`),
which in the default IEEE-754 rounding mode is ties to even. Both are behind `crates/slic3r-ffi`
and are not ported as Rust source (see `## Don't-translate list` and `### Linked libraries`). Where
a port site does its own ties-to-even conversion, it writes it out explicitly, because **Rust has
no ties-to-even rounding method** in `std`. A C++ `double`-to-integer conversion by a plain cast
(not through `round`, `floor`, `lround` or `nearbyint`) is a truncation, which Rust's `as i64` (and
similar `as` casts) matches.

### Error sets

Never widen or narrow an error set: a `throw` site and its `catch` keep the same set across the
port. A function that cannot fail does not return `Result`. An `assert` is compiled out of the
oracle's release build (standard `NDEBUG` behaviour), so it becomes `debug_assert!` and **never** a
live `panic!` — a live panic at a site the oracle has compiled the check out of is itself a
divergence (see `## Banned-substitution list`).

libslic3r's own exception hierarchy (`src/libslic3r/Exception.hpp:9-49`) is: `Exception` (the
base, deriving `std::runtime_error`), and under it `CriticalException`, under which
`RuntimeError`, `LogicError` (under which `InvalidArgument` and `OutOfRange`), `HardCrash`,
`IOError` (under which `FileIOError` and `HostNetworkError`), `ExportError`, and
`PlaceholderParserError` (under `RuntimeError`); and, separately under `Exception`,
`SlicingError`, which carries an `objectId` (`:31-40`), and `SlicingErrors`, which wraps a
`std::vector<SlicingError>` (`:42-49`). The port's mapping is one Rust error enum (or a small family
of them, one per crate that needs one) whose variants mirror this hierarchy one-for-one, so a
`throw SlicingError(...)` site and the code that would have caught it keep the same set of cases
across the port; a port task does not collapse two of these into one variant or add a variant the
hierarchy has no sibling for.

### Number formatting

Format numbers locale-independently (no `setlocale`/current-locale-dependent formatting, ever, in
emitted output). Transcribe `GCodeFormatter::emit_axis` (`src/libslic3r/GCodeWriter.cpp:1690-1747`;
its live body ends at `:1731`, and its `#if 0` debug block at `:1733-1746` is dead code the port
does not carry) verbatim:

1. Round `v * pow_10[digits]` to the nearest `int64_t` with ties away from zero (`:1696`,
   `std::round` — see `### Rounding modes`).
2. Render that integer's decimal digits. The `__APPLE__` branch at `:1698-1704` chooses
   `boost::spirit::karma::generate` over `std::to_chars` for the same `int64_t`, for an older
   macOS stdlib's missing `std::from_chars` support; it produces the same decimal digits either
   way and changes nothing the port has to reproduce.
3. Zero-pad on the left when the rendered integer has fewer digits than the decimal-digit count
   the row below gives.
4. Insert the decimal point at that digit count from the right.
5. Strip trailing zero digits after the decimal point.
6. Fix up: if stripping left an empty integer part, or a bare `-` with no digit after it, insert a
   leading `0`.

The digit counts (`src/libslic3r/GCodeWriter.hpp:334-336`): `xyz_digits` is 3, or 4 with high
precision; `e_digits` is 5, or 6 with high precision; `F_EXPORT_DIGITS` is 3.

### Linked libraries

Never reimplement one of these in this intent (R17): Clipper2 (the vendored copy is patched, so no
published crate matches it), admesh, libnest2d, qhull, Boost.Polygon's Voronoi builder (used at
`src/libslic3r/Geometry/Voronoi.hpp` and `Arachne/SkeletalTrapezoidation.hpp`; bound against the
Boost superbuild under `vendor/OrcaSlicer/deps/Boost/`, since it has no vendored source of its
own), and NLopt only where the manifest marks an in-scope caller (at the pin, NLopt's only
libslic3r includer is `SLA/SupportTreeBuildsteps.cpp`, which R9 puts out of scope, so it is absent
from every in-scope caller today). They are called through `crates/slic3r-ffi`, exactly as they
are at the pin; `just port-check` fails a crate manifest that depends on a named substitute for one
of them. Replacing any of them with a Rust implementation is out of scope for this intent (C4's
decision leaves that to a later one).
