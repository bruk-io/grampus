---
kind: task
step: 8
delivers: [R5]
depends: [25-stage-dumper]
size: 5
---
# Task: Serialise every fired seam's layer state

## What to build

This is part 2 of plan step 8, "Dump the pipeline's intermediate state", in
`intent/libslic3r-rust-port/plan.md`: `dump_seams.cpp` and its serialisation. It delivers R5
together with part 1, `25-stage-dumper`, which has already built the program, registered and armed
and reset the hook, added the L1 identity check and written the configuration driver. This task
fills in the state each seam leaves behind and changes nothing about `dump_main.cpp`'s registration,
arming or reset.

Files this task adds or changes:

- `oracle/stage-dump/dump_seams.cpp` — the body: one serialiser per seam.
- `oracle/stage-dump/dump_seams.hpp` — extended only where a new declaration is needed.
- `oracle/stage-dump/README.md` — gains the dump format section: every field written, how it is
  written, and the order it is written in.
- `vendor/README.md` — its "Pipeline steps" table is corrected to say which steps have a hook seam
  and which do not. This is the only change this task makes outside `oracle/stage-dump/`.

Which seams fire, and therefore which serialisers exist, is R5 in
`intent/libslic3r-rust-port/spec.md`. Read it there. R5 stays the one place the seams are enumerated
with their reasons: `dump_seams.cpp`'s comments name only the seam each serialiser handles, and
`README.md` points at R5 rather than listing them. The `vendor/README.md` table plan step 8
commissions is per step and says only whether that step has a hook seam — no reason text and no
commentary on the steps that have none.

Nothing under `vendor/OrcaSlicer/` changes; the submodule pointer stays at
`f92a7c065c5718b8ef1baac1cd704df2911238f4`. Step 8 adds no line to
`.github/workflows/ci.yml`.

### the state each seam reads

The hook's signature carries no layer (`vendor/OrcaSlicer/src/libslic3r/Print.hpp:1061`), so for a
per-object seam the dumper walks the object itself: `PrintObject::layers()` (`Print.hpp:374`) and,
for the support seam, `PrintObject::support_layers()` (`Print.hpp:375`). The fields, at the pin:

- per layer, `Layer::lslices` (`src/libslic3r/Layer.hpp:170`) and its `print_z`, `slice_z` and
  `height`;
- per region of `Layer::regions()`, `LayerRegion::slices` (`Layer.hpp:58`), `raw_slices` (`:62`),
  `thin_fills` (`:67`), `fill_surfaces` (`:73`), `perimeters` (`:86`) and `fills` (`:90`);
- per support layer, `SupportLayer::support_islands` (`Layer.hpp:307`) and `support_fills`
  (`:309`);
- the two print-wide seams pass a null object, so they read the print: the skirt-and-brim seam takes
  `Print::skirt()` (`Print.hpp:1170`), `Print::skirt_brim_groups()` (`:1171`) **and
  `PrintObject::object_skirt()` (`Print.hpp:446`) for every object of the print**, and the wipe-tower
  seam takes `Print::wipe_tower_data()` (`:1196`).

  `object_skirt()` is separate state and not a view of `Print::skirt()`: it returns
  `PrintObject::m_skirt` (declared `Print.hpp:653`), which is the per-object collection skirt
  generation fills and clears per object (`Print.cpp:3704`, `Print.cpp:4222`) when `skirt_type` is
  `stPerObject` (`Brim.cpp:63`). Without it, a per-object-skirt configuration dumps an empty
  `Print::skirt()` and no skirt extrusions at all, and `crates/slic3r-proof/src/dumps.rs` and
  `just compare-stages` then compare nothing for that stage while reporting a match. Plan step 8
  names it among the state this file reads, so it is in the list.

A seam reads only what the step it follows has produced, so each serialiser names the fields it
reads and writes no field the step has not filled. `posEstimateCurledExtrusions` is not relied on:
its `hook_after` requires `is_step_done(posEstimateCurledExtrusions)` (`Print.cpp:3504-3506`), and
the only `set_done` that would satisfy it for a sliced object is commented out at
`PrintObject.cpp:1054`, so this task writes no serialiser that depends on it firing and the run
reports it as not fired rather than as missing.

### how a dump is written

These dumps are the only per-stage proof the port gets: `crates/slic3r-proof/src/dumps.rs` reads
them for R19's second proof path (task `32-proof-dumps-fuzz`) and `just compare-stages` compares the
port's stage output against them (task `39-compare-stages`). So the format rules are the substance
of this task, not a detail of it.

- Integer `coord_t` coordinates are written as integers, in decimal, exactly as held. Nothing
  round-trips through `float` or `double`: no `unscale`, no `unscale_`, no `unscaled`, no scaling,
  and no formatting through a floating-point conversion. A reviewer must be able to read a dump and
  find integer coordinates with no decimal point.
- A field that is genuinely floating-point in the library — a layer's `print_z`, `slice_z` or
  `height`, the wipe tower's depth — is written in a form that is exact for the value held, either a
  hexadecimal float or the shortest round-tripping decimal, and `README.md` says which. A `double`
  is never narrowed to a `float` on the way out.
- One dump per seam that fires, written where part 1's hook body puts it, each carrying part 1's
  provenance header from `oracle/provenance` unchanged.
- The order is the library's own and is never sorted or deduplicated: layers in `layers()` order,
  regions in `Layer::regions()` order, entities in collection order, points in their stored order. An
  order the library did not produce would make a faithful port look wrong.
- Each record names the field it came from and the layer and region index it belongs to, so a reader
  and `dumps.rs` can find a field without knowing the writing order.

The L1 identity check part 1 added still has to pass after this task: serialising state must read the
pipeline and never mutate it. Take nothing by non-const reference out of the `Print` or the
`PrintObject` the hook hands over.

### what makes this a 5

Eleven seams, each reading different state the step leaves behind, with the hook carrying no layer
so the dumper walks the layers itself, and integer `coord_t` coordinates that must never round-trip
through float. These dumps are the only per-stage proof the port gets.

It is not split further. The serialisers share one format, one provenance header and one ordering
rule, which is exactly what a consumer of the dumps depends on; half the seams would give the port a
dump set it cannot prove a stage against, and the format would then be settled twice.

## Done when

- `just oracle-dump 20mm_cube base-fff` exits 0, writes one dump per fired seam, prints their count,
  and names every seam that did not fire for that configuration. There is no fallback to a vendored
  path: task `25-stage-dumper`, which this task depends on, depends on `11-fixture-set`, so the
  frozen fixture and its `vendor/README.md` record both exist — and a run over a model with no
  record would get no provenance header at all, which would fail the provenance line below.
- Every seam R5 enumerates has a serialiser in `dump_seams.cpp`: a reviewer greps the file for each
  seam's enumerator name and finds one.
- A reviewer opens one dump and finds integer coordinates, no decimal point in any `coord_t` field,
  and each record naming the field, the layer index and the region index it came from.
- Every floating-point value in a dump is a field `README.md`'s format section names as
  floating-point in the library, written in the exact form that section states; no `coord_t` value
  reaches the output through a `float` or a `double`, and `dump_seams.cpp` calls no `unscale`,
  `unscale_` or `unscaled`.
- `oracle/provenance --check 'port/dumps/*/*.dump'` exits 0 over the dumps this run committed, and
  `grep -c '^; '` is non-zero on each.
- The psSkirtBrim dump carries `Print::skirt()`, `Print::skirt_brim_groups()` and one
  `object_skirt()` record per print object, which a reviewer can point to in the dump by name.
- The L1 identity check part 1 added still prints `L1` and the recipe still exits 0.
- `oracle/stage-dump/README.md` has a dump format section listing every field written, how each is
  written, and the deterministic order, and points at R5 for the seams rather than listing them.
- `vendor/README.md`'s "Pipeline steps" table says, for every step, whether it has a hook seam, and
  carries no reason text.
- `oracle/stage-dump/dump_main.cpp` and `oracle/stage-dump/CMakeLists.txt` are unchanged by this
  branch, and `git status --porcelain vendor/OrcaSlicer` prints nothing.
