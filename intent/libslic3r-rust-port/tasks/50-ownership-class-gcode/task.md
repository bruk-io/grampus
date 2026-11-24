---
kind: task
step: 12
delivers: [R30]
depends: [16-ownership-spike]
size: 5
---
# Task: Classify GCode's forty-seven raw-pointer fields

## What to build

This is one of the eight tasks cut from step 12 of `intent/libslic3r-rust-port/plan.md`, "The
ownership manifest", and a piece of a split step: its `delivers: [R30]` is the whole step's, not
this task's alone. Step 12's first task, cut in an earlier pass and not one of these eight, was the
spike `16-ownership-spike`, which committed `port/ownership.tsv` — 208 rows, one per raw-pointer
field and per container-of-raw-pointers field declared in a header `port/manifest.tsv` marks in
scope, every `class` cell blank — together with `port/ownership-batches.tsv`, which batches those
fields by subsystem into rows of at most 120 fields each in the manifest's crate-level order. The
plan cuts **one task per row of that file**, plus a last task that removes the blank-class
allowance.

**This task is the `ownership-gcode-01` row of that file**: the `GCode` subsystem, 47 fields.

The work is filling in three cells — `class`, `evidence` and `rust type` — on those 47 rows of
`port/ownership.tsv`, with the class decided from the vendored headers at the pin and the evidence
cited. Nothing else in the repository changes.

### The rows you classify

They are every row of `port/ownership.tsv` whose `header path` cell is one of these, and no others.
Match a row by its `header path` and `line`, never by the field name alone: the names in the
`fields` cell of `port/ownership-batches.tsv` repeat both inside a batch and across batches (`m_tm`
and `m_mesh` are in this file twice, once under `AABBMesh.hpp` and once under
`SLA/IndexedMesh.hpp`), and `port/ownership` keys a row on `(header path, line, declaring type,
field)`.

| header | fields |
|---|---|
| `GCode/ToolOrdering.hpp` | 13 — `:107` `entity_map`, `:109` `support_map`, `:110` `support_intf_map` on `WipingExtrusions`, `:113` `m_layer_tools`, `:198` `custom_gcode` on `LayerTools`, `:224` `per_object_gradient` on `MixedSubLayerGroup`, `:233` `obj` on `VolumeKey`, `:405` `m_print_full_config`, `:406` `m_print_config_ptr`, `:410` `m_mixed_object_layers`, `:415` `m_object_all_layer_indices`, `:426` `m_print_object_ptr`, `:427` `m_print` on `ToolOrdering` |
| `GCode/ExtrusionProcessor.hpp` | 7 — `:449` `object` and `:450` `layer` on `PrecomputedOverhangLayer`, `:494` `prev_layer_boundaries`, `:495` `prev_curled_extrusions`, `:497` `prev_layer_sources`, `:498` `last_prepared_layers`, `:500` `current_object` on `ExtrusionQualityEstimator` |
| `GCode/ConflictChecker.hpp` | 6 — `:27` `_id` on `LineWithID`, `:38` `layer` on `ExtrusionLayer`, `:70` `_id` on `LinesBucket`, `:123` `line_bucket_ptr_queue` on `LinesBucketQueue`, `:142` `_obj1` and `:143` `_obj2` on `ConflictComputeResult` |
| `GCode/PreciseSeam.hpp` | 5 — `:67` `modifier_usage` on `PreciseSeamWarnings`, `:91` `layer`, `:92` `modifier` and `:93` `warnings` on `ExtractionContext`, `:152` `modifier` on `WeakModifierSegment` |
| `GCode/TimelapsePosPicker.hpp` | 4 — `:27` `curr_layer` and `:30` `printed_objects` on `PosPickCtx`, `:73` `print` and `:87` `bbox_cache` on `TimelapsePosPicker` |
| `GCode/GCodeProcessor.hpp` | 3 — `:164` `_obj1` and `:165` `_obj2` on `ConflictResult`, `:1402` `m_print` on `GCodeProcessor` |
| `GCode/RetractWhenCrossingPerimeters.hpp` | 2 — `:23` `m_layer` and `:25` `m_internal_islands` |
| `GCode/WipeTower.hpp` | 2 — `:415` `m_multi_nozzle_group_result` and `:444` `m_cur_block` |
| `GCode/PreciseSeamInternal.hpp` | 1 — `:23` `reason` on `FragmentBindingFailure` |
| `GCode/PressureEqualizer.hpp` | 1 — `:211` `m_layer_results` on `PressureEqualizer` |
| `GCode/SeamPlacer.hpp` | 1 — `:163` `m_seam_per_object` on `SeamPlacer` |
| `GCode/SpiralVase.hpp` | 1 — `:51` `m_previous_layer` on `SpiralVase` |
| `GCode/Thumbnails.hpp` | 1 — `:30` `data` on `CompressedImageBuffer` |

That is 47 rows, which is the `field count` cell of the `ownership-gcode-01` row of
`port/ownership-batches.tsv`. `port/cut-tasks` derived this batch by calling `port/inventory`'s
`subsystem_of()` on each row's `header path`: a row's subsystem is the top-level directory of that
path under `src/libslic3r`, and every field of this subsystem fits in one batch, because the step's
cap is 120 fields a row.

Those line numbers are the `line` column of `port/ownership.tsv`, and each field is listed under
the header that actually declares it. Do not re-derive the grouping from the flat `fields` cell of
`port/ownership-batches.tsv`: that cell is one comma-separated list in file-sorted row order with
no header boundaries in it, and slicing it by eye is how a field ends up attributed to the wrong
header. The table above is the authority for which header to open; the rule in the paragraph above
it — every row whose `header path` is one of these thirteen — is the authority for which rows you
own.

### What is in this batch

Forty-seven fields in thirteen headers — every in-scope header under `src/libslic3r/GCode`. They are
several independent pointer graphs rather than one, which is what makes the batch big:

- **Tool ordering, thirteen fields in one header.** `ToolOrdering.hpp:405`
  `m_print_full_config`, `:406` `m_print_config_ptr`, `:426` `m_print_object_ptr` and `:427`
  `m_print` are the tool ordering's pointers at the print and its config, which it does not own;
  `:113` `m_layer_tools` and `:198` `custom_gcode` point at per-layer data held elsewhere. The
  other seven rows are containers **keyed** on a raw pointer rather than holding one as a value —
  `:107` `entity_map`, `:109` `support_map`, `:110` `support_intf_map`, `:224`
  `per_object_gradient`, `:410` `m_mixed_object_layers`, `:415` `m_object_all_layer_indices`, and
  `:233` `obj` on the `VolumeKey` those maps are keyed with — so class each of them on who owns
  the pointee the key names, not on the map. These are the same borrow and back-reference shapes
  the `root` batch meets in `Print.hpp`, so cite evidence in `GCode/ToolOrdering.cpp` rather than
  reasoning from the other batch.
- **Per-object caches carried between layers, seven fields in `ExtrusionProcessor.hpp`.** `:494`
  `prev_layer_boundaries`, `:495` `prev_curled_extrusions`, `:497` `prev_layer_sources` and `:498`
  `last_prepared_layers` are `std::unordered_map`s keyed on `const PrintObject*`, `:500`
  `current_object` is the `const PrintObject *` the estimator is working on, and `:449` `object`
  with `:450` `layer` are the same pair held per layer on `PrecomputedOverhangLayer`. One reading
  settles all seven: does the estimator keep the object alive, or only point at what the slicing
  loop owns? Follow it into that loop and cite the line.
- **Type-erased handles, seven rows that name no pointee.** Six are `const void *` —
  `ConflictChecker.hpp:27` `_id` on `LineWithID`, `:70` `_id` on `LinesBucket`, `:142` `_obj1` and
  `:143` `_obj2` on `ConflictComputeResult`, and `GCodeProcessor.hpp:164` `_obj1` with `:165`
  `_obj2` on `ConflictResult` — and `Thumbnails.hpp:30` `data` is a `void *` buffer. A `void *`
  carries no type, so the class turns on what is actually stored through it: find the assignment,
  cite that `path:line`, and name the real pointee in the `rust type` cell rather than writing a
  pointer at nothing. These are the rows most likely to end `unknown`, which is an acceptable
  answer here when the cited use does not decide.
- **The conflict checker's own state.** `ConflictChecker.hpp:38` `layer` (`const Layer *` on
  `ExtrusionLayer`) and `:123` `line_bucket_ptr_queue`, a
  `std::priority_queue<LinesBucket *, std::vector<LinesBucket *>, LinesBucketPtrComp>` — the one
  queue of raw pointers in this batch, so the question is whether the queue owns the buckets or the
  checker holds them elsewhere and only orders them here.
- **Seam work, in three headers that are easy to mix up.** `PreciseSeam.hpp` holds five rows:
  `:67` `modifier_usage` (a map keyed on `const ModelVolume*`), `:91` `layer`, `:92` `modifier`,
  `:93` `warnings` (`PreciseSeamWarnings *`, the only non-`const` pointer of the five) and `:152`
  `modifier`. `PreciseSeamInternal.hpp:23` `reason` is a `const char *` — a static message rather
  than a pointer at an object, the shape `root`'s `nocli` also has. `SeamPlacer.hpp:163`
  `m_seam_per_object` is separate work again: a map keyed on `const PrintObject*`.
- **The remaining five headers, ten fields with nothing in common.** `GCodeProcessor.hpp:1402`
  `m_print` is a `Print *` the processor is handed and does not own — the last of that header's
  three rows, and the only one of them that is not type-erased. `PressureEqualizer.hpp:211`
  `m_layer_results` is a `std::queue<LayerResult*>` the equaliser pushes and pops;
  `RetractWhenCrossingPerimeters.hpp:23` `m_layer` and `:25` `m_internal_islands` are a layer
  pointer and a `std::vector<const ExPolygon*>` into that layer's islands; `SpiralVase.hpp:51`
  `m_previous_layer` is a `std::vector<SpiralPoint> *` buffer the vase keeps across layers;
  `TimelapsePosPicker.hpp:27` `curr_layer`, `:30` `printed_objects`, `:73` `print` and `:87`
  `bbox_cache` point at the print and the layer being picked over; and `WipeTower.hpp:415`
  `m_multi_nozzle_group_result` with `:444` `m_cur_block` point into the tower's own per-layer
  blocks, so find where a block is created.

Work header by header, and within a header let one reading of who allocates the pointee settle every
field that shares it. The alias trap the `root` and `Support` batches meet does **not** arise here:
every one of these 47 rows spells its pointer literally in the `C++ type` column, so no row of this
batch hides a raw pointer behind a typedef. Read that column as written.

### What makes this a 5, and why it is not split

Forty-seven fields across thirteen headers and four unrelated pointer graphs, each needing its own
reading of the slicing loop that owns the pointees, plus the per-field evidence and Rust type the
whole step requires.

It is not split, because the spike's committed `port/ownership-batches.tsv` makes one row one task
and the plan's cut rule is one task per row: the step's own batch cap is 120 fields a row, and
`GCode`'s 47 fields are a single row under it. Splitting would mean re-cutting the spike's answer.
Inside the task the thirteen headers are independent of each other, which is how the work is kept
in order — but that is independent work inside one step, not a reason to cut two tasks.

### What the three cells mean

`class` is one of six values and nothing else: `owned`, `shared`, `borrowed`, `back-reference`,
`ffi`, `unknown`. `VALID_CLASSES` in `port/ownership` is what accepts them, and it also still
accepts a blank cell — the allowance the spike put there so a row could be committed before it was
classified. Task `51-ownership-blank-class-gate` removes that allowance once every batch is in, and
**step 12 is done when no in-scope row's class is blank**, so leave none of your rows blank.

`evidence` holds one or more `path:line` entries separated by `, `, the decisive one first. A path
is spelled exactly as the `header path` column spells one — relative to
`vendor/OrcaSlicer/src/libslic3r/` — and may name a `.cpp` in that same tree when the proof is
there: a `delete` that proves `owned`, an assignment from a longer-lived object that proves a
borrow. The evidence this codebase offers is comments over raw pointers rather than types, which is
why the column exists at all. The plan's two worked examples:

- `ExtrusionEntityCollection.hpp:39` `ExtrusionEntitiesPtr entities;     // we own these entities`
  against `:19` `// Removed elements are NOT being deleted.` — an owning container.
- `Model.hpp:189` `Model *m_model;` under `// Parent, owning this material.` — a back-reference:
  the parent owns this object, not the other way round. **This is a deliberate departure from the
  plan's wording**, which calls the same line "a borrow": the comment at `Model.hpp:188` is what
  makes it a `back-reference` under this step's own vocabulary, and the eight tasks of this step
  all take the back-reference reading so that seven branches do not spell one shape two ways. The
  two readings give different cells under the table below — `&'a Model` against `Weak<Model>` or
  `usize` — so where a row of yours is a child's pointer back at its parent, follow the
  back-reference reading and name the departure in your pull request.

`just ownership-check` fails any row whose class is filled in, is not `unknown`, and whose evidence
cell is empty, so every row you classify carries evidence.

### The Rust type cell

`docs/PORTING.md` is not on `main` yet — task `19-porting-contract`, step 15, writes it, and
`docs/` holds only `ORACLE.md` and `WORKFLOW.md` today. It **will** tell porters to take this
cell verbatim, by header path and line, and will forbid a module inventing a Rust type for a
raw-pointer field. The seven
batch tasks of this step are built side by side by builders who never see each other, so they all
fill the column from one default per class. No accepted document fixes these defaults — they are
this step's own, written so that seven branches do not spell the same situation three ways:

| class | the `rust type` cell, by default |
|---|---|
| `owned` | the owning Rust type: `Box<Pointee>` for a single owned pointee, `Vec<Pointee>` for a container that owns its elements, or `Pointee` where the value is stored inline |
| `shared` | `Arc<Pointee>`, or `Arc<Mutex<Pointee>>` when the C++ mutates the pointee through the pointer from more than one owner |
| `borrowed` | `&'a Pointee`, or `&'a mut Pointee` for a non-`const` pointee |
| `back-reference` | `Weak<Pointee>` where the owner holds an `Arc`, otherwise `usize` — an index into the owning collection the `declaring type` and `C++ type` columns already name — because a plain reference back at the parent cannot be written in a cycle |
| `ffi` | the raw pointer the bridge carries, `*const Pointee` or `*mut Pointee`, as it crosses `crates/slic3r-ffi` |
| `unknown` | the conservative type a porter must use until the class is resolved: `Arc<Mutex<Pointee>>` for a pointee the C++ mutates, `Arc<Pointee>` for one it does not |

Write the concrete type, never a placeholder: `Box<ExtrusionEntity>`, not `Box<Pointee>`. Where a
C++ container of raw pointers is `owned`, the cell names the Rust container and its element type —
`Vec<ExtrusionEntity>`, not a container of pointers. A `const` C++ pointee never takes `&mut` or
`Arc<Mutex<..>>`. A function-pointer field carries no ownership at all: its cell is the equivalent
Rust function-pointer type, such as `fn(f32, f32, f32) -> f32`.

These are **faithfulness defaults, not idiom upgrades**. R15 has the port's banned-substitution
list reject idiom "improvements", and the intent leaves Rust idioms and optimisation to the
following intent, so the cell names the simplest Rust type that keeps the C++ meaning rather than
the nicest one. A row may depart from its class's default where the faithful translation needs it —
a pointer kept raw behind a `// SAFETY:` comment, which R14 anticipates, is the usual case — and
then the pull request names that row and why. What is not allowed is a seventh vocabulary: a
departure is one row, not a new rule for the column.

### When a row is genuinely unclear

`unknown` is allowed, and is not a way to leave work undone: it is for a field whose use at the pin
does not settle the class. An `unknown` row carries the `path:line` of the use that leaves the class
unclear **and** the conservative Rust type above, and `just ownership-check` fails it when either is
missing. Prefer a decided class with cited evidence; use `unknown` only where the evidence does not
decide, and name every `unknown` row, with why, in the pull request — which is the only place an
`unknown` row is visible today. `just port-status` **will** print the count of `unknown` rows once
task `22-port-status-gates` is in: the recipe exists at `justfile:66`, but `port/port-status` does
not read `port/ownership.tsv` yet, so do not go looking for a number nothing prints.

### Built side by side with the other batches

Every batch task of step 12 edits `port/ownership.tsv` and nothing else, so these are the rules that
keep them from colliding:

- Edit only the `class`, `evidence` and `rust type` cells of **your own rows**, the ones named
  above. Leave every other row of the file byte-identical, its blank cells included — another
  batch's builder is filling those in a branch of their own at the same time. The one exception is
  resolving a rebase conflict, below: there you keep the other branch's filled-in cells rather than
  restoring the blanks they replaced.
- Change no cell of the first five columns (`header path`, `line`, `declaring type`, `field`,
  `C++ type`). `just ownership` recomputes those from the vendored headers, so a hand edit there is
  exactly the drift `just ownership-check` fails on.
- Do not reorder rows, add a row, delete a row, change the header line or reformat the file. The
  check compares the committed rows against regeneration as an ordered, duplicate-sensitive list, so
  a reorder fails even when the set of rows is unchanged.
- Do not edit `port/ownership`, the script. The only task of this step that touches it is
  `51-ownership-blank-class-gate`, which removes the blank-class allowance after every batch is in.
- Do not edit `port/ownership-batches.tsv`, `port/manifest.tsv`, `port/cut-tasks`,
  `port/port-status`, `port/inventory`, the `justfile`, `.asdlc/checks`, anything under `docs/`, or
  anything under `vendor/OrcaSlicer/` — the submodule pointer stays at
  `f92a7c065c5718b8ef1baac1cd704df2911238f4`.
- Add no file under `intent/` and change none.
- This task is not blocked by, and does not wait for, any other batch task; it depends only on
  `16-ownership-spike`, which is on `main`. Where your reading of a field disagrees with a
  neighbouring batch's reading of a similar one, the rows are still separate rows: classify yours
  from the headers and say so in the pull request.
- **Rebase on `main` before you open the pull request, and again whenever `main` moves.**
  `port/ownership.tsv` is sorted by `header path`, so the bare root headers interleave
  alphabetically with the subsystem directories and this batch's rows are not an island in the file:
  its first row, `GCode/ConflictChecker.hpp:27`, sits directly below `44-ownership-class-root`'s
  `GCode.hpp:975`, and its last row, `GCode/WipeTower.hpp:444`, sits directly above that batch's
  `GCodeWriter.hpp:262`. Git's three-way merge conflicts on edits to adjacent lines, so if the root
  batch merges first you can get a conflict at exactly those two rows through no fault of your own.
  Resolve such a conflict by **keeping both sides' cells**, yours and the other branch's, and never
  by reverting a sibling's filled-in row: a discarded classification is invisible, because `just
  ownership-check` still passes on a blank class until `51-ownership-blank-class-gate` lands. Then
  run `just ownership` and `just ownership-check` again to prove the file is still byte-identical to
  regeneration.

### How to check it

- **Rebase on `main` first**, so every check below and the `git diff main` in "Done when" are
  measured against current `main` rather than against the commit this branch started from. A
  sibling batch may have merged while you were working.
- `just ownership-check` exits 0 and prints `ownership-check: port/ownership.tsv holds 208 row(s)`.
  This is the recipe that matters: it regenerates the field list into
  `port/.cache/ownership-check/`, fails on any drift from the headers at the pin, and separately
  validates every row's class and evidence rule across the whole file, not only your rows. It does
  not fail a blank class yet, so it passing is not on its own proof that your rows are filled in.
- `just ownership` rewrites the committed file and must leave it byte-identical after your edit,
  because `class`, `evidence` and `rust type` are carried forward verbatim by key. Run it once to
  prove that.
- `just check` (`asdlc document check`) and `just guard` (`asdlc branch check`) both pass. There is
  no `just port-check` recipe yet — task `24-port-check-ci` adds it, along with the
  `just ownership-check` line in `.asdlc/checks` — so add neither here.

## Done when

- Every one of the 47 rows named above — the rows of `port/ownership.tsv` whose `header path`
  is one of this batch's headers, which is the `field count` of the `ownership-gcode-01` row of
  `port/ownership-batches.tsv` — has a `class` cell holding one of `owned`, `shared`, `borrowed`,
  `back-reference`, `ffi` or `unknown`, and none of the 47 is blank.
- Each of those 47 rows has a non-empty `evidence` cell holding at least one `path:line`,
  spelled relative to `vendor/OrcaSlicer/src/libslic3r/` as the `header path` column spells a path,
  and a non-empty `rust type` cell naming a concrete Rust type that follows this task's table of
  defaults, or departs from it for a reason the pull request names row by row.
- Every one of those rows whose class is `unknown` carries both the `path:line` of the use that
  leaves the class unclear and the conservative Rust type, and the pull request names each
  `unknown` row and why it is one.
- The branch is rebased on `main`, and `git diff main` then touches no line of `port/ownership.tsv`
  other than those 47, and changes no cell of `header path`, `line`, `declaring type`, `field` or
  `C++ type` on any row.
- `port/ownership.tsv` still has 209 lines, its header row unchanged and its rows in the same
  order — which a rebase resolution that keeps both sides' cells preserves, since neither side
  adds, removes or moves a row.
- `just ownership-check` exits 0 and prints that `port/ownership.tsv` holds 208 rows.
- Running `just ownership` leaves `port/ownership.tsv` byte-identical.
- `port/ownership`, `port/ownership-batches.tsv`, `port/manifest.tsv`, `port/cut-tasks`,
  `port/port-status`, `port/inventory`, the `justfile`, `.asdlc/checks` and everything under `docs/`
  are unchanged.
- Nothing under `vendor/OrcaSlicer/` changed and the submodule pointer is still
  `f92a7c065c5718b8ef1baac1cd704df2911238f4`.
- The branch changes no file under `intent/`, so `just guard` passes, and `just check`
  passes.
- Each of the thirteen headers' rows is classed from evidence inside `src/libslic3r/GCode` or the
  slicing loop that owns the pointee, not by analogy with another batch's rows.
- The pull request lists the thirteen headers with the class counts it gave, so a reviewer can see
  no header was skipped.
- Each of the seven type-erased rows — `ConflictChecker.hpp:27`, `:70`, `:142`, `:143`,
  `GCodeProcessor.hpp:164`, `:165` and `Thumbnails.hpp:30` — either cites the `path:line` of the
  assignment that shows what is stored through the `void *` and names that pointee in its
  `rust type` cell, or is `unknown` with the `path:line` of the use that leaves it unclear.
