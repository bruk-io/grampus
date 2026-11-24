---
kind: task
step: 12
delivers: [R30]
depends: [16-ownership-spike]
size: 8
---
# Task: Classify the root subsystem's 120 raw-pointer fields

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

**This task is the `ownership-root-01` row of that file**: the `root` subsystem, 120 fields.

The work is filling in three cells — `class`, `evidence` and `rust type` — on those 120 rows of
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
| `GCode.hpp` | 30 |
| `Print.hpp` | 20 |
| `PerimeterGenerator.hpp` | 12 |
| `ProjectTask.hpp` | 9 |
| `Model.hpp` | 8 |
| `Layer.hpp` | 7 |
| `Config.hpp` | 6 |
| `PrintBase.hpp` | 4 |
| `AABBMesh.hpp`, `ClipperUtils.hpp`, `EdgeGrid.hpp`, `GCodeWriter.hpp`, `MarchingSquares.hpp`, `MutablePolygon.hpp` | 2 each |
| `calib.hpp`, `clonable_ptr.hpp`, `ConnectedBodies.hpp`, `Extruder.hpp`, `ExtrusionEntityCollection.hpp`, `FlushVolPredictor.hpp`, `InstanceLock.hpp`, `PlaceholderParser.hpp`, `PNGReadWrite.hpp`, `PrintConfig.hpp`, `SVG.hpp`, `Utils.hpp` | 1 each |

That is 120 rows, which is the `field count` cell of the `ownership-root-01` row of
`port/ownership-batches.tsv`. `port/cut-tasks` derived this batch by calling `port/inventory`'s
`subsystem_of()` on each row's `header path`: a row's subsystem is the top-level directory of that
path under `src/libslic3r`, and every field of this subsystem fits in one batch, because the step's
cap is 120 fields a row.

### What is in this batch, and the rows to think hardest about

`root` is every in-scope header directly under `src/libslic3r`, so this batch is the port's core
data model and the classes decided here fix the Rust shape the other crates are written against.

- **The owning spine.** `Print.hpp:1513` `m_objects` (`PrintObjectPtrs`) and `:1514`
  `m_print_regions` (`PrintRegionPtrs`), `:681` `m_layers` and `:682` `m_belt_truncated_layers`
  (`LayerPtrs`), `:693` `m_support_layers` (`SupportLayerPtrs`); `Layer.hpp:294` `m_regions`
  (`LayerRegionPtrs`); `Model.hpp:1702` `objects` (`ModelObjectPtrs`), `:390` `instances`, `:393`
  `volumes`, `:1700` `materials`; `ExtrusionEntityCollection.hpp:39` `entities`
  (`ExtrusionEntitiesPtr`). These are the alias-typed containers of raw pointers the spike's alias
  set found — they show no `*` at their declaration, which is why the `C++ type` column names the
  alias — and the plan's first worked example is one of them.
- **The back-references, which is why that class exists.** `Layer.hpp:133` `m_layer` (`Layer *` on
  `LayerRegion`), `:293` `m_object` (`PrintObject *` on `Layer`), `Model.hpp:189` and `:666`
  `m_model`, `:1182` and `:1559` `object` (`ModelObject *` on `ModelVolume` and on
  `ModelInstance`), `PrintBase.hpp:376` and `:691` `m_print`, `PrintBase.hpp:362` `m_model_object`.
  The `Print`→`PrintObject`→`Layer`→`LayerRegion` and `Model`→`ModelObject`→`ModelVolume` graphs
  are cyclic in C++, and a child's pointer back at its parent is a `back-reference`, not a
  `borrowed`. `Layer.hpp:146` and `:147`, `upper_layer` and `lower_layer`, are sibling links inside
  the same owning collection, and `Print.hpp:718` `m_shared_object` (`PrintObject *`) points at
  another object the print owns.
- **The config and option pointers.** `Config.hpp:264` `opt_def`, `:2152` and `:2215` `keys_map`,
  `:2619` `enum_keys_map`, `:2659` `by_serialization_key_ordinal`, and `GCode.hpp`'s `opt_position`,
  `opt_zhop`, `opt_e_position`, `opt_e_retracted`, `opt_e_restart_extra`, `opt_extruded_volume`,
  `opt_extruded_weight`, `opt_extruded_volume_total`, `opt_extruded_weight_total` — pointers into
  option tables and into the placeholder parser's own config, which the holder does not own.
- **Three rows the vocabulary does not name directly.** `Config.hpp:2636` `nocli` is
  `static const constexpr char *`, a static string constant rather than a pointer at an object;
  `clonable_ptr.hpp:141` `px` is the pointer inside a smart-pointer template, so its class follows
  from what that template does with it in its own destructor and `clone`; and `Print.hpp:218`
  `m_data` on `ConstVectorOfPtrsAdaptor` is a pointer *to* a vector of pointers, held by a view that
  owns neither. None of the three may be left blank: decide a class and cite it, or make it
  `unknown` with the use that leaves it unclear and a conservative type.

### What makes this an 8, and why it is not split

120 fields across 26 headers, and they are the fields whose classes the whole port is written
against: the cyclic `Print`/`PrintObject`/`Layer`/`LayerRegion` and `Model`/`ModelObject`/
`ModelVolume` graphs, the config option tables, and the G-code generator's state. A wrong
`borrowed`-versus-`back-reference` call here does not stay local, because `docs/PORTING.md` — which
task `19-porting-contract` writes in step 15, and which is not on `main` yet — will make every
porter take the Rust type from this row verbatim.

It is not split, because the spike's committed `port/ownership-batches.tsv` makes one row one task
and the plan's cut rule is one task per row; 120 is the step's own batch cap, already applied, and
`root` fills exactly one batch. Splitting it would mean re-cutting the spike's answer.

### Its first commit is a plan of its own

Start with `asdlc plan new libslic3r-rust-port/44-ownership-class-root` and commit that plan before
any cell is filled in. That is the one file this branch may add under `intent/`, and the guard
allows it.

The risk that plan talks through: **this batch's classes are what the whole port is written against,
and the cycles in the core data model are where a wrong call spreads furthest.** The defaults in
this task's table are not up for revision there — six sibling batches are being filled in against
that same table at the same time, and a seventh vocabulary invented in this plan would never reach
them. What the plan decides is how the table is *applied*: which of the three graphs
(`Print`/`PrintObject`/`Layer`/`LayerRegion`, `Model`/`ModelObject`/`ModelVolume`, and
`PrintBase.hpp`'s `m_print` pair) resolve to `Weak` and which to an index, so that a reader of the
file sees one answer per graph rather than one per row; which fields cannot be decided from the
headers at the pin, and so become `unknown` with a conservative type; and which rows, if any, depart
from a default because the faithful translation needs it, each named in the pull request.

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
- Add no file under `intent/` other than this task's own plan, described below, and
  change none.
- This task is not blocked by, and does not wait for, any other batch task; it depends only on
  `16-ownership-spike`, which is on `main`. Where your reading of a field disagrees with a
  neighbouring batch's reading of a similar one, the rows are still separate rows: classify yours
  from the headers and say so in the pull request.
- **Rebase on `main` before you open the pull request, and again whenever `main` moves.**
  `port/ownership.tsv` is sorted by `header path`, so a bare root header sits alphabetically
  between the subsystem directories and this batch's rows are *directly adjacent* to another
  batch's at eleven places: `AABBMesh.hpp:73`, `ClipperUtils.hpp:99`,
  `ExtrusionEntityCollection.hpp:39`, `FlushVolPredictor.hpp:48`, `GCode.hpp:975`,
  `GCodeWriter.hpp:262`, `GCodeWriter.hpp:402`, `InstanceLock.hpp:55`, `ProjectTask.hpp:340`,
  `SVG.hpp:95` (which borders a sibling's row on both sides) and `Utils.hpp:544`. Git's
  three-way merge conflicts on edits to adjacent lines, so each of the six sibling batches can
  hand you a conflict at one of those rows once it merges first — this batch is the one that
  borders all six. Resolve such a conflict by **keeping both sides' cells**, yours and the
  other branch's, and never by reverting a sibling's filled-in row: a discarded classification is
  invisible, because `just ownership-check` still passes on a blank class until
  `51-ownership-blank-class-gate` lands. Then run `just ownership` and `just ownership-check` again
  to prove the file is still byte-identical to regeneration.

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

- Every one of the 120 rows named above — the rows of `port/ownership.tsv` whose `header path`
  is one of this batch's headers, which is the `field count` of the `ownership-root-01` row of
  `port/ownership-batches.tsv` — has a `class` cell holding one of `owned`, `shared`, `borrowed`,
  `back-reference`, `ffi` or `unknown`, and none of the 120 is blank.
- Each of those 120 rows has a non-empty `evidence` cell holding at least one `path:line`,
  spelled relative to `vendor/OrcaSlicer/src/libslic3r/` as the `header path` column spells a path,
  and a non-empty `rust type` cell naming a concrete Rust type that follows this task's table of
  defaults, or departs from it for a reason the pull request names row by row.
- Every one of those rows whose class is `unknown` carries both the `path:line` of the use that
  leaves the class unclear and the conservative Rust type, and the pull request names each
  `unknown` row and why it is one.
- The branch is rebased on `main`, and `git diff main` then touches no line of `port/ownership.tsv`
  other than those 120, and changes no cell of `header path`, `line`, `declaring type`, `field` or
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
- The branch adds no file under `intent/` other than this task's own plan and changes
  none, so `just guard` passes, and `just check` passes.
- `intent/libslic3r-rust-port/tasks/44-ownership-class-root/plan.md` is committed as this branch's
  first commit, and says for each of the three graphs which `back-reference` default it applies —
  `Weak` or an index — and which rows it leaves `unknown`.
- The pull request states, per graph, the `back-reference` Rust shape the file now carries, and
  names every row that departs from this task's default with the reason.
