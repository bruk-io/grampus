# The per-file translation contract

`just port-status` reads the twelve headings below — the six `##` sections and the six `###`
rules under `## Rules` — to check that this document says what it must (R29); renaming one
breaks that check. `## Decisions recorded on 2026-11-25` and `## Decisions recorded on
2026-11-26` are later additions and are not among the twelve; adding a heading does not break
the check, renaming one of the twelve does.
This document is committed before any unit is ported.
Every unit of Phase A and Phase B — `port/manifest.tsv`'s 408 in-scope files — is held to
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

(an empty-string line in that output counts the 199 out-of-scope rows, which carry no crate —
expected, not an error) against the `crate` rows of `port/cut.tsv`
(`awk -F'\t' '$1=="crate"{print $2, $3, $4, $5}' port/cut.tsv`, which prints the table's four
columns — crate, level, subsystems, in-scope files — in order).
As this document is written, the two agree exactly: the same 125 crate names, and the same
in-scope file count per crate, summing to the 408 in-scope rows `port/manifest.tsv` carries. Plan
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

Every row has a cell to take, and `just ownership-check` is what guarantees it:

- **Every row's `class` cell is one of `owned`, `shared`, `borrowed`, `back-reference`, `ffi` or
  `unknown`.** A blank class is not among them (`port/ownership`'s `VALID_CLASSES`, `:95`) and
  `validate_rows` (`:747-749`) fails any row whose class is not, saying so in as many words — "a
  blank class is no longer accepted". When this document was first written every row's class was
  blank and the classification tasks cut from `port/ownership-batches.tsv` were still open; they
  have since filled all 208, so a blank-class row can no longer exist on `main` at all.
- **An `unknown`-class row names a conservative Rust type**, and `validate_rows` (`:752-755`)
  fails an `unknown` row carrying no Rust type and no evidence — so that cell is guaranteed
  present. Take it. `unknown` does not mean "no type to take": it means the type is the
  conservative one.
- **Every other class's `rust type` cell is the field's type, taken as the cell reads it.** See
  `## Decisions recorded on 2026-11-26`, decision 6, for a nullable `owned` field. Measured on the
  committed file: 143 `borrowed`, 31 `back-reference`, 20 `owned`, 7 `unknown`, 6 `ffi`, 1
  `shared`, no blank-class row — and all 208 rows carry a filled `rust type` cell.

**The one case with no type to take, and the only one left.** `validate_rows` requires a `rust
type` cell only on an `unknown` row, so a row of some *other* class with an empty cell would pass
every check. No row is in that state today (measured above), and if a porter ever meets one it is
a finding to report, not a type to invent: leave the field with a
`// TODO(port): <header path>:<line> — Rust type pending the classification task for this row's
batch (port/ownership-batches.tsv)` marker, say so in the pull request body, and move on; the cell
is corrected on `main` by an amendment pull request, as decision 6's was. **No module invents a
Rust type for a row whose cell is empty**, and no module writes a type that differs from a cell
that is filled — #60 reported the mismatch that produced decision 6 rather than silently picking
one side, which is the behaviour this rule wants.

**Foundation types.** `Point`, the Eigen vector aliases at `src/libslic3r/Point.hpp:52-69`
(`Vec2crd`, `Vec3crd`, `Vec2i32`, `Vec2i64`, `Vec3i32`, `Vec3i64`, `Vec4i32`, `Vec2f`, `Vec3f`,
`Vec4f`, `Vec2d`, `Vec3d`, `Vec4d`), `Polygon`, `ExPolygon` and the configuration headers whose
manifest row marks `template-heavy=yes` are hand-designed once, before fan-out, by plan step 20
(cut as `34-foundation-spike` and the foundation tasks it batches, which also corrects the seeded
`template-heavy` column where it is wrong) — not by this document. A porter does not invent a
second spelling for one of these: they find the type's crate from the crate map above
(`slic3r-point` for `Point` and the Eigen aliases, `slic3r-polygon` for `Polygon`,
`slic3r-expolygon` for `ExPolygon`, `slic3r-config` for the `template-heavy=yes` configuration
headers) and use the spelling that crate's module exports. A configuration header whose row is
`template-heavy=no` — `PrintConfig.hpp` is one, at the pin — is not part of this hand-designed
set: its crate (`slic3r-printconfig`) is an ordinary fan-out unit, so a porter who reaches it
before its own unit is drafted reads that crate's module as it then stands, rather than expecting
a pre-fan-out design to already be there.

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
| A free function in an anonymous namespace, or a `static` free function | A private (non-`pub`) free function in the module. It is **not** made `pub` to ease testing: proof path 3 (R19) — a test translation unit inside the FFI crate that `#include`s the vendored `.cpp` — is how such a function is proven equal, not visibility. `## Decisions recorded on 2026-11-25`, decision 1, states the three files that translation unit and its bridge are, and that they are the only route. |
| A non-const reference out-parameter | A return value, or a tuple of return values, from the Rust function — never a `&mut` parameter adopted just to mirror the C++ signature. |
| A destructor | `Drop`, implemented only where the destructor does more than drop its fields (an RAII release, a log, a cache invalidation); a destructor that only frees owned sub-objects needs no explicit `Drop` impl, because Rust already drops them. |
| Virtual dispatch over a class hierarchy (e.g. `ExtrusionEntity`) | A Rust trait with one implementing type per concrete C++ subclass, dispatched through `&dyn Trait` or an enum, whichever the foundation design for that hierarchy (`slic3r-extrusionentity`, hand-designed by plan step 20) chooses; a port task does not introduce a second dispatch mechanism for the same hierarchy. |
| A template | A generic function or type with the same bound, monomorphised the same way the C++ template's instantiations are used in in-scope code; a template is not force-collapsed to one concrete type unless the manifest row's unit only ever instantiates one. |
| A `#define` numeric constant | A `const` item, carrying a `// PORT SRC: <vendored path>:<line>` marker beside it naming the macro's definition. Where the value's origin is outside the vendored tree — a C standard clause, or a linked library — the marker takes the `// PORT SRC: external: <what defines it>` form instead; see `## Decisions recorded on 2026-11-25`, decision 2, which also states what "beside" means. |
| `std::sort` | `slice::sort_unstable` (no ordering guarantee between equal elements, matching `std::sort`'s own lack of a stability guarantee). |
| `std::stable_sort` | `slice::sort` (stable, matching `std::stable_sort`). |
| `tbb::parallel_for`, `tbb::parallel_reduce` | See `### Linked libraries` below: TBB itself is not reimplemented. R18 states this without qualification: the port sums in a fixed order and is serial, and the module carries a `// PERF(port):` marker and nothing else — that marker is the only place a deferred optimization may live (R15). This is the unconditional default at every such site; parallelising a reduction requires a committed audit that establishes the site's order-independence, and no such audit exists today — `port/reports/order-containers.md` and `port/reports/order-comparators.md` audit container shapes and comparators, not parallel sites, so neither grants that licence — so no port task may parallelise a reduction yet. |
| Iterator and pointer arithmetic | Slices and indices — a C++ `T*` walked with `++`/`--`/offset arithmetic over a contiguous buffer becomes a `&[T]`/`&mut [T]` walked by index or by a `std::slice` iterator; a raw pointer that is not walking a contiguous Rust-owned buffer is instead a `port/ownership.tsv` row (see the type map). |
| `operator<<` (or other serialisation) | A `Display` or `Debug` impl where the output is for humans, or an explicit `write_...`/`format_...` function where the output is G-code or another emitted artifact whose exact bytes are compared — never `Display`/`{}` for G-code or other compared output, because `### Number formatting` below is specific about digit counts and padding that a derived or generic `Display` impl does not reproduce. |

## Don't-translate list

What a porter stops at rather than ports:

- **Out-of-scope code reached from an in-scope file.** The manifest row's reason (`port/manifest.tsv`,
  "in scope or out with a reason" column) is the authority for why; the porter adds a
  `// PORT NOTE:` marker at the boundary (the call or reference into the out-of-scope code) rather
  than following it in. A **declaration-only header whose every definition is in a file the
  manifest marks out of scope** is one of these: `port/inventory` deducts it, with a reason naming
  that file, so it has no row to port at all — see `## Decisions recorded on 2026-11-25`,
  decision 3.
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
  files, the `inherits` chain, preset-bundle resolution, and the `Format/` readers
  `port/manifest.tsv` deducts out — read the manifest's rows for which that is, rather than this
  list.
- **The SLA implementations**, except `SLA/IndexedMesh` and the handful of `SLA/` headers the
  manifest marks in as carried data — `port/manifest.tsv` is the authority for which, and the
  `## Crate map`'s `slic3r-sla` row gives the count. `SLA/IndexedMesh` is in scope because
  Z-contouring calls it (R9); the carried headers hold fields the model type stores as data only.
  Hollowing, pad and the concave-hull implementations are out.
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
| A hash-ordered map or set where iteration order reaches output, in place of the one insertion-ordered spelling this document fixes | None automated yet; `port/reports/order-containers.md` is the per-site authority a reviewer holds the unit to. `just audit-check` (`port/audit-order --check`) only verifies that report has not drifted from the vendored tree — it never reads a port module, so it cannot catch this substitution. |
| A parallel reduction where the original's order is load-bearing | None automated yet; `port/reports/order-comparators.md` and `port/reports/divergences.md` (R18) are the per-site authority a reviewer holds the unit to. `just audit-check` only verifies those reports have not drifted from the vendored tree — it never reads a port module, so it cannot catch this substitution. |
| Replacing one of R17's six libraries (Clipper2, admesh, libnest2d, qhull, Boost.Polygon's Voronoi builder, NLopt) with a Rust crate | `just port-check`'s substitute check (added by `24-port-check-ci`) fails a crate manifest that depends on a named substitute. |
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
the vendored tree are both in `src/slic3r/GUI/Plater.cpp`, the two arms of one branch
(`:15014-15017`): `:15015` restores the default `SCALING_FACTOR_INTERNAL`, and `:15017` sets the
large-printer value `SCALING_FACTOR_INTERNAL_LARGE_PRINTER`, `0.00001` (`libslic3r.h:66`), above
`LARGE_BED_THRESHOLD`, `2147` (`:67`) — code R10 puts out of scope (the GUI's `Plater`). So the
port's surface sees exactly `0.000001`.

The port fixes one spelling for this value: `slic3r_libslic3r::SCALING_FACTOR`, a `pub const f64`
carrying `// PORT SRC: src/libslic3r/libslic3r.h:65`, in the `slic3r-libslic3r` crate (crate map
above, level 2). `libslic3r.h` is one of the headers whose manifest row is `template-heavy=yes`,
so it is part of the set plan step 20 hand-designs before fan-out (see "Foundation types" above);
`55-foundation-libslic3r-execution` defines this constant once, in that crate's module —
`34-foundation-spike` committed only the crate skeletons, each with no `mod` line
(`git show b5b404b:crates/slic3r-libslic3r/src/lib.rs`), so it never defined this or any other
value; see `## Decisions recorded on 2026-11-26`, decision 3. No fan-out module
may declare a second spelling or write the literal `0.000001` inline. It is a `const`, not a
`static` the port can reassign: the only mutator found above is `Plater.cpp`'s branch, and R10
puts the whole GUI out of scope, so nothing in the port ever needs the large-printer value and the
port's surface — everything this contract binds — sees only the one value. R7 already records
`SCALING_FACTOR`'s value with every oracle output, so a comparison is never run against an oracle
output whose scale disagrees.

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
  formatter sites that reach the compared artifact are `int64_t(std::round(v * pow_10[digits]))`
  in `GCodeFormatter::emit_axis` (`src/libslic3r/GCodeWriter.cpp:1696`), and
  `std::round(v * pow_10[ndigits]) * pow_10_inv[ndigits]` in `GCodeFormatter::quantize`
  (`src/libslic3r/GCodeWriter.hpp:341`) — `quantize` returns a `double` and scales back by the
  reciprocal table at `:338`, not by dividing, so its body is not `emit_axis`'s and its result is
  never cast to `int64_t`. The `TextureDisplacement` and `SLA/` rounding sites are out of scope.
- **`floor(x + 0.5)` is round-half-up, not round-half-away, and is not `f64::round`.** They differ
  at an exact negative half (`floor(-0.5 + 0.5)` is `0`, while `(-0.5f64).round()` is `-1.0`) and at
  `0.49999999999999994`. Transcribe it as `(x + 0.5).floor()`, never as `.round()`. Sites that reach
  the emitted G-code include: `src/libslic3r/GCode.cpp:9467`, `(unsigned int)floor(acceleration + 0.5)`
  (the M204/M201 acceleration value — the same idiom recurs in the same acceleration path at
  `GCode.cpp:4612`, `:6881`, `:6919`, `:10547`, `:10560`, `:10563` and `:10566`);
  `src/libslic3r/GCode/CoolingBuffer.cpp:995`, `int(floor(60. * line->feedrate + 0.5))` (a rewritten
  `F` word); `src/libslic3r/GCode/WipeTower.cpp:1616`, `int(floor(f + 0.5f))` (another `F` word —
  `f` is a `float` parameter of `set_format_F(float f)` at `:1614`, so the add and the floor are in
  `f32`); `src/libslic3r/Config.hpp:1023`, `int(floor(_value + 0.5))` in `ConfigOptionInt`; and
  `src/libslic3r/Fill/FillLine.cpp:78`, `coord_t(floor(this->_min_spacing * INFILL_OVERLAP_OVER_SPACING + 0.5f))`
  (`INFILL_OVERLAP_OVER_SPACING` is a `const float` at `:76`, and `_min_spacing` is a `coord_t` at
  `src/libslic3r/Fill/FillLine.hpp:32`, so the `i64` converts to `f32` first and the multiply, add
  and floor are all in `f32`). This list is the sites the rule was written from, not the whole set
  the idiom occurs at across in-scope files — the rule applies at every occurrence, so a porter
  greps their own unit for `floor(` and `0.5` rather than relying on this list alone.
  `GCodeWriter.cpp:782`'s `floor(100.0 * num / tot + 0.5)` in `GCodeWriter::update_progress` is
  excluded from this list: the function returns early unless the flavour is MakerWare or Sailfish
  (`GCodeWriter.cpp:775-776`), which no compared configuration selects. The site is inside this
  in-scope file (`slic3r-gcodewriter`); it is the comparison that excludes it, not the file's scope.
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
6. If the strip left the decimal point as the last character, drop it (`GCodeWriter.cpp:1727-1728`).
7. Fix up: if stripping left an empty integer part, or a bare `-` with no digit after it, insert a
   leading `0`.

The digit counts (`src/libslic3r/GCodeWriter.hpp:334-336`): `xyz_digits` is 3, or 4 with high
precision; `e_digits` is 5, or 6 with high precision; `F_EXPORT_DIGITS` is 3.

### Linked libraries

Never reimplement one of these in this intent (R17): Clipper2 (the vendored copy is patched, so no
published crate matches it), admesh, libnest2d, qhull, Boost.Polygon's Voronoi builder (bound
against the Boost superbuild under `vendor/OrcaSlicer/deps/Boost/`, since it has no vendored
source of its own), and NLopt only where `port/manifest.tsv` marks an in-scope caller — read the
manifest, not this document, for whether one exists today. They are called through
`crates/slic3r-ffi`, exactly as they are at the pin — and so is every unit's own proof shim, by
the one route `## Decisions recorded on 2026-11-25`, decision 1, fixes; `just port-check`'s substitute check (added
by `24-port-check-ci`) fails a crate manifest that depends on a named substitute for one of them.
Replacing any of them with a Rust implementation is out of scope for this intent (C4's decision
leaves that to a later one).

## Decisions recorded on 2026-11-25

The first five port attempts (pull requests #53 to #57) each hit the same handful of gaps and
each invented a different way around them. These five decisions close those gaps. Every one was
made by Claude acting for Bruk Habtu, under the delegation for this intent, on 2026-11-25; the
owner was away and had handed over this intent's decisions. This section is a record, not a
summary of a rule stated elsewhere: where a decision changes a rule above, the rule's own text
says so and points here.

### 1. One route from a ported crate to the oracle

**The gap.** Port crates are generated with only a dev-dependency on `slic3r-proof`, and a port
task may add no dependency and may not write `Cargo.lock`. A unit whose manifest row declares
proof path 1 or 3 therefore had no sanctioned way to reach the built library at all, and the
first three attempts each invented one: a per-crate `build.rs` with its own C++ shim (#54, and
#53 as merged), a Cargo dependency on `slic3r-ffi` (#57, later reverted), and appends to the
shared `crates/slic3r-ffi/cxx/proof_shims.cpp`, `proof_shims.h` and `src/proof.rs` (#53 and
#56). The appends also do not work: `cxx` compiles its generated translation unit against the
bridge's own `include!`ed header, so a declaration appended to a *shared* header is not visible
to it (#54 measured the undefined-symbol error), and a bridge in `slic3r-ffi` is unreachable
from a crate that may not depend on it (#56 measured the `E0433`).

**The decision.** Every port crate reaches the oracle through `slic3r-proof`, which re-exports
the FFI crate's per-unit proof bridges (`slic3r_proof::proof_units`). No port crate gets its own
`build.rs`, its own C++ shim, or a new Cargo dependency. A port task adds its shim by **adding
new files only**, touching no file another port task also edits, because hundreds of port
branches are built side by side and every shared append point is a merge conflict.

**What a port task creates — three new files, plus its own test file:**

| file | what it holds |
|---|---|
| `crates/slic3r-ffi/cxx/proof_units/<unit>.h` | the shim's C++ declarations, inside `namespace slic3r_ffi::proof_units::<unit>`; `#include`s its own `cxx`-generated header, `"slic3r-ffi/src/proof_units/<unit>.rs.h"` |
| `crates/slic3r-ffi/cxx/proof_units/<unit>.cpp` | the definitions: for proof path 1, `#include`s the vendored header and calls the externally linked symbol; for proof path 3, `#include`s the vendored `.cpp` whole and forwards to its internal-linkage helper |
| `crates/slic3r-ffi/src/proof_units/<unit>.rs` | one `#[cxx::bridge(namespace = "slic3r_ffi::proof_units::<unit>")]`, `include!`ing `"cxx/proof_units/<unit>.h"`, and the `pub use` of its own names |
| the unit's own proof | `crates/<crate>/tests/<something>.rs`, or a `#[cfg(test)]` module in the unit's module file; calls `slic3r_proof::proof_units::<unit>::<fn>` |

`<unit>` is the unit's `port/manifest.tsv` `target module` file stem (`object_id` for
`crates/slic3r-objectid/src/object_id.rs`), so the shim's name is derivable from the manifest row
and two units can never pick the same file name.

**What a shim may carry across the bridge:** a plain integer or a plain floating-point value, a
`rust::Vec` of those, and a `cxx` bridge `struct` whose every field is one of those — never a
libslic3r type. The third is not an exception grudgingly allowed: the one proof-path-3 unit in
the tree is built that way (`crates/slic3r-ffi/src/proof_units/clipper_z_utils.rs`'s
`ProofZPoint`, three `i64` fields standing for one `ClipperZUtils::ZPoint`), and the first
version of this rule, written as "plain integers and plain floating-point values" alone, forbade
the shape of its own worked example. A coordinate travels as a 64-bit integer and never through
an `f64` (`coord_t` is `int64_t` at the pin, `src/libslic3r/libslic3r.h:45-48`).

`crates/slic3r-ffi/build.rs` globs both directories: every `src/proof_units/*.rs` goes to
`cxx_build::bridges`, every `cxx/proof_units/*.cpp` onto the one shared `cxx_build::Build` that
carries `-ffp-contract=off` and the whole include set, each shim's vendored
`cargo:rerun-if-changed` set is read from its own `#include "..."` lines, and the `#[path]`-ed
`pub mod` list is generated into `OUT_DIR` for `src/proof_units.rs` to `include!`. There is no
shared list, no shared C++ file and no shared `pub use` block to append to. A bridge with no
matching `.h` and `.cpp` is a named build failure, not a silent skip.

`cargo fmt` cannot follow an `include!` and so never reaches the bridge files; `just port-check`
runs `rustfmt --check` over them directly instead, so they are not a gap in its coverage.

The shims that existed before this decision — the harness's own two and the four pull request #53
added — are migrated into this layout, so there is exactly one pattern in the tree. Of the
migrated functions, `object_id`'s `path1_wipe_tower_object_id` and `clipper_z_utils`'s
`path3_to_paths64` had live callers; `print_config_constants`' and `technologies`' did not, and
now do.

### 2. A constant whose origin is outside the vendored tree

**The gap.** R16 requires every numeric literal other than `0`, `1` and `2` to carry a
`// PORT SRC: <vendored path>:<line>` marker naming a line that contains the literal. Pull
request #57 has a correct literal that cannot comply: the `4` in C's `%g`
fixed-versus-scientific threshold comes from the C standard library's specification, not from any
OrcaSlicer line, and citing an unrelated vendored `-4` would satisfy the scanner with a false
statement.

**The decision.** One sanctioned second form, and the exempt literal set is **not** widened:

```
// PORT SRC: external: <what defines it, specifically>
```

for example `// PORT SRC: external: ISO/IEC 9899:2018 7.21.6.1 p8 -- %g renders in scientific
notation when the converted value's exponent is less than -4`, or a named linked library and
version with its file and line under the superbuild. `port/port-status` accepts it without a
vendored-line lookup, requires the text after `external:` to be non-empty, and **counts and
prints every external marker separately, with its text**, so a reviewer can audit each one.
`docs/WORKFLOW.md` makes checking every external marker against the source it cites a reviewer's
job.

Three things it deliberately does not do, each of which the form's first implementation did and
a reviewer measured:

- It does not exempt the file, the module or any other line. A literal with no marker of either
  form still fails.
- It does not cover **more than one literal on its own line**. There is no cited line to match a
  literal against, so one marker on a line carrying two or three would legitimise all of them —
  a per-line exemption in all but name. Such a line is its own finding; the externally defined
  constant goes on a line of its own.
- It does not suppress a vendored marker sitting in the same comment block. A
  `// PORT SRC: <path>:<line>` naming a line that does not exist at the pin is still R16's own
  placeholder-constant failure, with or without an external marker beside it.

"Beside" means the same for both forms: on the literal's own line, or in the contiguous run of
`//` comment lines ending on the line directly above it. **A blank line between the marker and
the literal breaks the run**, which is the other half of #57's red gate and is now stated in
`port/port-status --help`.

### 3. A declaration-only header whose definitions are all out of scope

**The gap.** `port/manifest.tsv` marked `MacUtils.hpp` in scope with proof path 1, but it is a
ten-line declaration-only header whose only definitions are in `MacUtils.mm`, which the same
manifest deducts. The unit had no in-scope body to port and no in-scope symbol to prove against,
so it could never be ported or proven (#54).

**The decision.** A declaration-only header whose every definition is in a file the manifest
marks out of scope is itself out of scope, with a reason naming that file. It is a rule in
`port/inventory`'s scope computation, not a hand edit of the manifest, and it is conservative in
the direction of keeping a row in scope: a header declaring a class, struct, enum, typedef,
`using`, template, variable, a `#define` with a replacement list, or carrying any function body
is not declaration-only; every declared name must have a definition somewhere in the tracked
tree; one in-scope definer leaves the header in scope; and the search runs over every tracked
file but the header itself, headers included. At the pin it deducts exactly one row,
`MacUtils.hpp`, and `port/inventory` fails with a named error if it ever deducts a different set,
so the rule cannot widen without someone judging the new row against the vendored source.

A caller that reaches such a header adds a `// PORT NOTE:` marker at the boundary, exactly as
`## Don't-translate list` already requires for out-of-scope code reached from an in-scope file.

### 4. A unit with nothing executable to prove, and what counts as a proof

**The gap.** Some units are declarations, constants or type definitions with no behaviour the
oracle can be asked about; #53 and #54 both had some, and one proof compared a module's output
against literals the test itself had transcribed, which is not a proof against the oracle. R19's
answer — "Where even that is not possible the row says so and the helper is proven through its
enclosing file's exported surface" — was neither concrete nor checkable.

**The decision, first half.** A manifest row may declare that its unit has no independently
provable behaviour, in its existing `proof path` column:

```
enclosing: <the enclosing unit's own `path` cell>
```

The named unit is the in-scope unit whose proof exercises this one. Such a row carries no
`// PROOF:` marker of its own, and `port/port-status` derives it as `ported` only when its module
exists, a `mod` line reaches it, it holds no `todo!()`/`unimplemented!()`, and the named row is
itself `ported`. A cell naming no manifest row, an out-of-scope row, itself, or a row whose own
proof path is another `enclosing:` cell, or a row in a different `target crate`, is a finding and
derives `drafted` — a chain is refused outright, which is also what makes a cycle impossible, and
the same-crate requirement is the widest check a text rule can make of R19's "its **enclosing
file's** exported surface" (the manifest has no "this file includes that one" column), so it is a
floor and not the whole of that wording: a reviewer still reads whether the named unit's proof
really exercises this one. The cell is set by the regeneration or amendment pull request that
writes `port/manifest.tsv`, never by a port task.

**The decision, second half.** A `// PROOF:` marker counts only over a test that compares a value
the **oracle** produced against a value this port computed.

The oracle's own entry points are the per-unit proof bridges `slic3r-proof` re-exports and the
committed stage dumps (decision 1's route), plus an `extern "C"` declaration. That list is not
narrowed by the row's own `proof path` cell (`## Decisions recorded on 2026-11-26`, decision 2).
A name holds an
oracle value when it is bound — by a `let`, a tuple pattern or a `for` pattern — off an
expression that reaches one of those, whether by an imported name or by a fully-qualified
`proof_units::`/`dumps::` path; a `let`'s type annotation is not part of its pattern and binds
no name (`## Decisions recorded on 2026-11-26`, decision 1). A comparison counts when it names an oracle value and a value of
this port's own, **in two different arguments**, or in one argument that itself makes a call (so
`assert!(agrees(ported, oracle))` counts and `assert_eq!(oracle[index], 5)`, where the loop
variable merely shares an argument with the oracle value, does not). The port's side must be a
plain value identifier: a macro name (`vec!`, `format!`), either end of a path
(`String::from`, `Oracle::Built`), a field or method name, a turbofish's type argument and the
literal and type words are none of them values this port computed. A marker over a test that
reaches no oracle, or that compares an oracle value only against a literal, is a
`proof-marker-not-differential` finding and its row stays `drafted`.

**Its stated limit.** A hand-transcribed named constant in the test's own scope is not
mechanically distinguishable from the ported module's own `pub const`: comparing an oracle value
against such a constant still counts, and remains a reviewer's judgement. What the rule does
catch is a test that reaches no oracle at all, and one that compares an oracle value only against
a literal — the two shapes the first port attempts actually produced. Every accept and reject the
rule is held to is listed in `port/port-status --self-test` and in
`crates/oracle-compare/src/stages.rs`'s own tests, in the same order in both, so a reader can see
exactly where the line falls.

Both halves are mirrored in `crates/oracle-compare/src/stages.rs`, which carries the Rust copy of
this derivation that `just compare-stages` reads, so the two authorities cannot disagree about
whether a stage is fully ported.

### 5. Where a task's file list and this document disagree

Foundation task bodies enumerate exactly which files `git diff main` may touch, and task files on
`main` are not edited after acceptance; decisions 1 to 4 move where shims and markers go.
`docs/WORKFLOW.md`'s "Decisions that govern over a task's file list" states the rule: this
document governs, the builder says so in the pull request body, and a reviewer does not count it
as a divergence.

## Decisions recorded on 2026-11-26

The three port tasks merged as pull requests #54, #56 and #57 each carried forward a finding they
were not allowed to fix: every file that would have to change was on their own must-not-write
list. Decisions 1 to 4 close those findings; decision 5 closes a gap found while fixing decision 1;
decision 6 closes one `57-foundation-utils` (#60) carried forward for the same reason; and
decision 7 reconciles the section decision 6 corrected, whose older text contradicted it; and
decision 8 is task `35-trial-run`'s own trial-run finding, recorded here by that task; and
decision 9 accepts `60-foundation-trees-grids` with one unit left drafted.
Every one was made by Claude acting for Bruk Habtu, under the delegation for this intent, on
2026-11-26; the owner was away and had handed over this intent's decisions. Like the section
above, this is a record: where a decision changes a rule above, the rule's own text says so and
points here.

### 1. A `let`'s type annotation is not part of its binding

**The gap.** `## Decisions recorded on 2026-11-25`, decision 4, says a name holds an oracle value
when it is bound "by a `let`, a tuple pattern or a `for` pattern" off an expression reaching an
oracle entry point. `port/port-status` read the whole of a `let`'s pattern as the names it bound,
type annotation included, so `let oracle: Vec<u32> = path1_x();` made `Vec` and `u32` oracle
names as well as `oracle`. The taint then spread through the fixed point to any later binding
whose right-hand side spelled one of those words, and a genuinely differential proof read as
having no ported side at all: its marker became a `proof-marker-not-differential` finding and its
row stayed `drafted`. #56 measured it (`port/port-status:511` as that branch read it) and worked
around it by dropping its proof's type annotations rather than by fixing the scanner, which it
could not write.

**The decision.** A type annotation names types and binds nothing, so it is cut off before the
pattern's names are taken: `let_binding_pattern` keeps the pattern's text up to its first `:` at
bracket depth 0 that is not part of a `::` path. A `for` pattern is passed through untouched,
since only a `let` carries an annotation. The depth counter is what keeps a struct pattern's own
`:` (`let Foo { a: x }: Foo`) and an array length (`let oracle: [i64; 3]`) from cutting in the wrong
place, and the `::` test is what keeps a path pattern (`let Enum::V(x)`) from reading as an
annotation.

This widens nothing else. A proof still needs an oracle value and a value of its own in two
different arguments (or in one that makes a call), and the ported side is still read with the
narrower view that refuses a macro name, either end of a path, a field or method name and a
turbofish's type argument — so `let oracle: Mask = path1_x(); assert_eq!(oracle, Mask::ALL);` is
still not a differential comparison.

**Proof.** Four accept/reject shapes are pinned in `port/port-status --self-test` and in
`crates/oracle-compare/src/stages.rs`'s own test, in the same order in both (16 shapes to 20),
and the `proof-differential-annotated-binding` fixture holds the derivation itself. Both bite:
with the cut removed, that fixture derives `drafted` with a `proof-marker-not-differential`
finding and the Rust test panics on the first of the four shapes.

**And it moves no row that is already ported**, measured rather than inferred: `--per-commit`'s
whole output over the real tree is byte-identical run with `main`'s `port/port-status` and with
this one, and `cargo test -p oracle-compare`'s
`row_is_ported_agrees_with_port_status_on_every_ported_row_of_the_real_manifest` still passes. The
cut only clears false negatives.

### 2. What a `proof path: 2` cell commits a task to

**The gap.** #54 and #57 each reported that a row's `proof path` cell of `2` — R19's "the
committed stage dumps (R5), for whole pipeline steps" — names a mechanism that cannot reach their
unit, and that the cell should arguably read `1`. Four merged rows carry the finding in their
module doc comments: `LifecycleEvents.hpp`
(`crates/slic3r-lifecycleevents/src/lifecycle_events.rs`), `enum_bitmask.hpp`, `format.hpp` and
`Execution/Execution.hpp`. The finding's premise is correct and was measured again here:
`port/inventory`'s `seed_judgement_cells` seeds `2` from the row's own
`pipeline stage or none` cell, and that cell means only "some stage's `#include` closure
reaches this file" (`compute_pipeline_stages`' own docstring says header rows are dominated by the
earliest stage that reaches the hub `Print.hpp`) — not "this file is that pipeline step". 85
in-scope rows carry `proof path: 2`. Seven of them name a stage the committed corpus holds no dump
for at all: `posContouring` fires only for a belt printer and does not fire for either committed
fixture (`oracle/stage-dump/README.md:145`), and `psGCodeExport` and `psConflictCheck` are not on
`dump_seams.cpp`'s `kSeamRoster` at all.

**The decision — the cell is not amended, and a bridge proof under a `2` cell is conforming.**
The oracle's entry points are the ones decision 4 of 2026-11-25 names — the per-unit proof bridges
`slic3r-proof` re-exports, the committed stage dumps, and an `extern "C"` declaration — and that
list is **not restricted by the row's `proof path` cell**. A `// PROOF:` marker must match its
row's cell verbatim, because that is how `port/port-status` pairs a marker with a row; which
oracle route the proof body then takes is governed by the entry-point list, not by the cell's
number. So a `proof path: 2` row whose unit writes nothing any dump records is proven through its
per-unit bridge, keeps its `// PROOF: 2` marker, and is conforming. `ObjectID.hpp` is the merged
precedent, and the four rows above follow it.

**Why the cells are not corrected here, with the measurement.** Editing a judgement cell is an
amendment pull request's prerogative (decision 4 of 2026-11-25 says so for the `enclosing:` form,
and `port/inventory` carries all six judgement columns forward verbatim and `just inventory-check`
does not diff them), so this pull request could have done it. It is wrong to do it in this one:

- Changing a *ported* row's cell to `1` without editing its merged module in the same commit
  regresses it. Measured through `derive_row_status` itself, with only the cell changed: all four
  of `Execution/Execution.hpp`, `LifecycleEvents.hpp`, `enum_bitmask.hpp` and `format.hpp` go from
  `ported` to `drafted`, because each module's marker still reads `// PROOF: 2`.
- Changing a *not-yet-ported* row's cell breaks whichever in-flight port branch holds that unit:
  its builder reads the cell from `main` as it stood when the task was cut, writes that marker, and
  its row derives `drafted` once merged. This is not hypothetical. `Execution/ExecutionSeq.hpp`
  and `Execution/ExecutionTBB.hpp` are two of the seven dumpless-stage rows, and both are units
  of `56-foundation-point`, which is being built now.
- Correcting 4 rows of the 85 would leave the column less consistent than it is now, and deciding
  the other 81 is a per-row reading of whether a unit's output reaches a dump field, which no text
  rule derives.

Re-deriving the whole `proof path` column against R19's "for whole pipeline steps" is therefore
regeneration-pass work, to be done in one pass with the markers of every ported row it moves. The
four module doc comments that call this an open finding are left as they are, deliberately: they
are in port units' own files, which in-flight branches are editing, and this decision is where the
finding is answered.

### 3. `34-foundation-spike` did not define `SCALING_FACTOR`

**The gap.** The "Scaling and coordinates" section above attributed defining
`slic3r_libslic3r::SCALING_FACTOR` to `34-foundation-spike`. #57 reported that this is false and
could not fix it: that spike committed only the crate skeletons, each carrying a doc comment and
no `mod` line (`git show b5b404b:crates/slic3r-libslic3r/src/lib.rs`), so it defined no value at
all.

**The decision.** The sentence now names `55-foundation-libslic3r-execution`, which defines the
constant in `crates/slic3r-libslic3r/src/libslic3r.rs` (commit `3255b4a`). The section's two other
references to the spike — the hand-designed foundation types and the one-time `indexmap`
workspace dependency — are correct and are unchanged: both are the crate-skeleton task's own work.

### 4. `53-foundation-int128-queue` delivered three of its four units

**The gap.** That task's `task.md` names four units, the fourth being `MacUtils.hpp`, and three of
its Done-when lines ask for `crates/slic3r-macutils/src/mac_utils.rs`. Decision 3 of 2026-11-25
then deducted `MacUtils.hpp` out of scope — a declaration-only header whose every definition is in
the out-of-scope `MacUtils.mm` — and the regeneration that followed removed `slic3r-macutils` from
`port/cut.tsv` and the crate from the tree. The task file on `main` is not edited after
acceptance, so those three lines cannot be struck where they are written.

**The decision.** The task is complete at three units. `port/manifest.tsv:365` reads
`MacUtils.hpp root 10 0 out: ...` and no `slic3r-macutils` crate exists, so there is no in-scope
body to port and no in-scope symbol to prove against; building the unit would have meant writing
`port/manifest.tsv`, which that task may not write, or inventing a crate the generated cut does
not hold. This is decision 5 of 2026-11-25 applied to a unit list rather than a file list, and
`docs/WORKFLOW.md`'s "Decisions that govern over a task's file list" already states its second
corollary — "A task whose unit list names a file `port/manifest.tsv` now marks out of scope drops
that unit, says so in the pull request body, and does not count the task's `ported` line against
itself" (`docs/WORKFLOW.md:155-156`). #54 said so in its own pull request body; this is the
record it asked for.

### 5. `port/port-status --self-test` runs in `just port-check`

**The gap.** Every rule `port/port-status` enforces is pinned by its own `--self-test`: all 50
fixture trees, the differential rule's 20 accept/reject shapes, the `enclosing:` cases and the
external-marker cases. It ran in no `justfile` recipe and no CI job — measured by grep over the
`justfile` and `.github/workflows/` — so nothing in the repository ever asked whether the checker
still held its own rules, and decision 1's new fixture would have bitten only for whoever ran it by
hand. This was found while fixing decision 1, not reported by a task.

**The decision.** `just port-check` runs it, as its second line. That recipe is the right home and
not a new one: its stated entry condition is "the code checks that need neither the oracle binary
nor a built libslic3r", and it takes about a tenth of a second. The recipe's own header comment
accounts for every line that has joined it, including this one, and points here.

**Measured, because this line is added to a gate three branches are building against.** A port
branch's `crates/` and `port/manifest.tsv` differ from `main`'s, so "the self-test reads only its
own fixtures" has to be a measurement and not a reading of the code. `run_selftest()` was called
with `Path.read_text`, `Path.read_bytes`, `Path.is_file`, `Path.iterdir`, `Path.glob`,
`Path.rglob`, `builtins.open` and `subprocess` wrapped to record every path: **348 distinct paths
touched, and exactly two of them outside `port/tests/port-status/` and the pinned
`vendor/OrcaSlicer/`** — a `port/manifest.tsv` in each of two scratch trees the self-test itself
creates under the system temporary directory. It touches no file of the real `crates/` tree and
not the real `port/manifest.tsv`. (`vendor/OrcaSlicer/` is in the set because a fixture's
`source:` lines resolve against the pin, which the recipe's first line already reads.)

So this is a strengthening and not a narrowing: no existing check's rule moved, and a port task
that passed `just port-check` before passes it now unless `port/port-status` itself has stopped
holding one of its own rules. `git grep -- --self-test` also confirms nothing else in the
repository — no other recipe, workflow, hook or script — invoked it before this.

### 6. An `owned` field that the C++ can null is an `Option`

**The gap.** #60 (`57-foundation-utils`) found that `port/ownership.tsv`'s row for `FilePtr::f`
(`Utils.hpp:544`, `FILE *`, class `owned`) gave the Rust type as `std::fs::File`. The vendored
field is nullable and the port depends on it: it is initialised to `nullptr`, `close()` at
`Utils.hpp:538-543` nulls it while the object lives on, and the in-scope construction sites pass it
the result of an `fopen` that may have failed. A reviewer rejected the unit for being unable to
represent that state, and the task could not write the manifest.

**The decision.** The cell now reads `Option<std::fs::File>`. As a rule for the rows not yet
ported: where an `owned` row's C++ field can hold null during the object's life, the `rust type`
cell wraps the owned type in `Option`. A port task that finds such a row without the `Option`
reports it and stops on that field, as #60 did; the cell is corrected on `main` by a pull request
like this one, not inside the task. Made by Claude acting for Bruk Habtu under the delegation for
this intent, on 2026-11-26.

### 7. What the ownership column's cases are, now that every row has a class

**The gap.** Decision 6 corrected the stale sentence that closed "Raw-pointer fields" above, but
the two bullets over it were written for the pre-classification window and were left in place, so
the section then said three things that cannot all be true: that a blank class is "guaranteed" and
is the expected state "for the whole window the fan-out drafts are written in"; that a blank class
"is now a failure of `just ownership-check`"; and that "only the `unknown` rows still take the
`// TODO(port):` marker", when the `unknown` bullet directly above says an `unknown` row's cell is
guaranteed present and is to be taken.

**What the code says, which settles all three.** `port/ownership`'s `VALID_CLASSES` (`:95`) does
not contain the empty string, and `validate_rows` (`:747-749`) fails any row whose class is not in
it — with the literal detail "a blank class is no longer accepted". So a blank-class row cannot
exist on `main`: decision 6's sentence is right, and the blank-class bullet described a state the
checks now forbid. And `validate_rows` (`:752-755`) fails an `unknown` row with no evidence or no
Rust type, so an `unknown` row always has a cell to take: the "only the `unknown` rows still take
the `// TODO(port):` marker" clause had it backwards.

**The decision.** The section is rewritten around what is true: every row carries a class from
that set, every row's `rust type` cell is taken verbatim, and `unknown` means the type is the
conservative one rather than that there is no type. The `// TODO(port):` route is kept with its
one real trigger — `validate_rows` requires a `rust type` cell only on an `unknown` row, so a row
of another class with an empty cell would pass every check — and is stated as a finding to report
rather than a type to invent. Measured on the committed file: 143 `borrowed`, 31 `back-reference`,
20 `owned`, 7 `unknown`, 6 `ffi`, 1 `shared`, no blank-class row, and all 208 rows carrying a
filled `rust type` cell, so that route has no row today.

**Recorded against myself.** A closed pull request of mine (#63) proposed the same correction and
asserted the opposite of the first paragraph above — that "nothing stops a future row of the
vendored tree arriving unclassified" and that a porter should fall back to the blank-class route.
That was false, from a reading of `validate_rows` that grepped only its `rust type` lines and
missed the `VALID_CLASSES` gate. #62's author got it right; this decision follows the code, not
that pull request. Made by Claude acting for Bruk Habtu under the delegation for this intent, on
2026-11-26.

### 8. A transcribed literal must be spelled so its own text, not only its value, matches the cited line

**The gap.** Task `35-trial-run` found a transcribed numeric constant that carries a real
`// PORT SRC: <vendored path>:<line>` marker and still gets no mechanical benefit from it, because
`port/port-status`'s `check_numeric_constants` matches a cited line by the literal's *normalised
text*, not its numeric value: `_vendored_line_holds` is a text search for the normalised digits,
not a numeric-equality check. Two concrete ways a porter can get this wrong while believing a
marker is honest and sufficient:

- Writing a whole-number float with a type suffix instead of a decimal point (`1f64` rather than
  `1.0`) when the vendored line spells it `1.0`. `normalize_literal` strips the suffix, so the
  check reads the bare digits `1` — which lands in R16's exempt `0`/`1`/`2` set and needs no marker
  at all, silently. The constant is correctly transcribed and the marker, if written, would even
  be correctly placed; the literal's *spelling* is what defeats the check, not its value.
- Writing an octal literal in Rust's `0o` syntax (`0o644`) to transcribe a C octal literal spelled
  without the `o` (`0644`). The two denote the same value, but `0o644`'s normalised text is not a
  substring of a line that writes `0644`, so a marker citing that exact line fails
  `_vendored_line_holds` and the porter is left with no vendored-line marker that can pass — the
  external form is the only one `port-status` currently accepts, even though the constant's origin
  is not outside the vendored tree (decision 2 of 2026-11-25's criterion), only differently
  spelled.

**The decision.** A transcribed numeric constant's Rust spelling must be chosen, among the
spellings that type-check and have the correct value, so its normalised text (after
`normalize_literal` strips digit separators and a type suffix) occurs verbatim on the cited
vendored line — never relying on landing in the `0`/`1`/`2` exempt set as a way to skip writing a
marker for a value that *is* transcribed from a vendored line. The 75-row `MaterialType` table
(`crates/slic3r-materialtype/src/material_type.rs`) is the sanctioned instance of the suffix
technique: every whole-number float field there is written with an explicit `f64` suffix (`100f64`,
not `100.0`) specifically because the *vendored* line spells the value as a bare integer, so the
suffix form is what makes the normalised text match — every one of those cells also carries its
own `// PORT SRC:` marker, so the suffix is chosen for a textual match, not to reach the exempt
set un-marked.

Where no Rust spelling of a value can be made to match the cited line's text (the octal case
above, until `port-status` gains a numeric-equality fallback), the external marker form is used as
a stated workaround — not under decision 2 of 2026-11-25's "origin outside the vendored tree"
criterion, which does not apply — and the module's own comment says so plainly, naming the vendored line the
constant actually comes from and the reviewer's job of checking that disclosure rather than taking
the external form's usual meaning on faith. This document's idiom map (the `#define` numeric
constant row) and decision 2 of 2026-11-25 are read together with this rule, not in place of it.

### 9. `60-foundation-trees-grids` is accepted with three of its four units ported

**The gap.** #70 ports `KDTreeIndirect.hpp`, `MarchingSquares.hpp` and `AABBTreeIndirect.hpp`, each
proven against the oracle through its own proof unit. Three things in it are outside what the task
text allows, and the task could settle none of them itself:

- `MeshSplitImpl.hpp` is translated in full but left `drafted`, with no `mod` line. Every function
  it exports takes the concrete `indexed_triangle_set` of `TriangleMesh.hpp`, and
  `slic3r-meshsplitimpl` has no dependency on `slic3r-trianglemesh` in `port/cut.tsv`; a port task
  may add none. Four of the task's "Done when" lines name this unit and are therefore not met.
- `AABBTreeIndirect.hpp`'s `BoundingBoxWrapper` is not ported, for the same kind of reason: it needs
  `Point`, `BoundingBox` and `SCALED_EPSILON`, and its crate has no dependency on `slic3r-point`.
  `intersect_triangle_epsilon` is not ported either: it has no caller anywhere in the vendored tree
  and does not compile when instantiated.
- `crates/slic3r-ffi/build.rs` gains one include path, `deps_src/libigl`, because
  `AABBTreeIndirect.hpp:25` includes `<igl/Hit.h>` unconditionally. That file is not on the task's
  list.

**The decision.** #70 is accepted as it stands. `mesh_split_impl.rs` stays `drafted`, and its row
stays in `port-status`'s remaining count, until `70-foundation-triangle-mesh` has landed; whichever
of that task or the regeneration pass first has `indexed_triangle_set` in reach finishes the unit,
either by giving the crate the dependency in `port/cut.tsv` or by making the module generic over
the mesh type, and adds its `mod` line and `// PROOF:` test. `BoundingBoxWrapper` is finished the
same way once its crate can see `slic3r-point`. `intersect_triangle_epsilon` is not ported at all:
dead code that cannot be instantiated has no behaviour to preserve. The `build.rs` include path is
accepted; an include path a vendored header needs unconditionally is part of reaching the oracle,
not a change to the harness. Made by Claude acting for Bruk Habtu under the delegation for this
intent, on 2026-11-26.
