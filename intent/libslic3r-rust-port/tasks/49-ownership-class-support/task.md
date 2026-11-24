---
kind: task
step: 12
delivers: [R30]
depends: [16-ownership-spike]
size: 3
---
# Task: Classify Support's fifteen raw-pointer fields

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

**This task is the `ownership-support-01` row of that file**: the `Support` subsystem, 15 fields.

The work is filling in three cells — `class`, `evidence` and `rust type` — on those 15 rows of
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
| `Support/TreeSupport.hpp` | 10 — `:65` `NO_PARENT`, `:177` `parent`, `:178` `parents`, `:179` `child`, `:188` `merged_neighbours`, `:418` `m_print_config`, `:426` `overhang_types`, `:436` `contact_nodes`, `:439` `m_object`, `:440` `m_object_config` |
| `Support/TreeSupportCommon.hpp` | 3 — `:787` `top_contacts`, `:788` `top_interfaces`, `:789` `top_base_interfaces`, all `SupportGeneratorLayersPtr &` on `InterfacePlacer` |
| `Support/SupportMaterial.hpp` | 2 — `:91` `m_print_config` and `:92` `m_object_config` on `PrintObjectSupportMaterial` |

That is 15 rows, which is the `field count` cell of the `ownership-support-01` row of
`port/ownership-batches.tsv`. `port/cut-tasks` derived this batch by calling `port/inventory`'s
`subsystem_of()` on each row's `header path`: a row's subsystem is the top-level directory of that
path under `src/libslic3r`, and every field of this subsystem fits in one batch, because the step's
cap is 120 fields a row.

### What is in this batch

Four shapes, and the node graph is the one that needs care.

- **The tree-support node graph.** `SupportNode::parent` and `child` (`SupportNode *`), `parents`
  (`std::vector<SupportNode*>`), `merged_neighbours` (`std::list<SupportNode*>`) and
  `TreeSupport::contact_nodes` (`std::vector<std::vector<SupportNode*>>`). The nodes themselves are
  allocated and freed by `TreeSupport` — find that allocation and that free in
  `Support/TreeSupport.cpp` and cite it, because it is what separates the one owning container from
  the links between nodes. The links are a cyclic graph, so a child's pointer at its parent is a
  `back-reference` under this task's table, taking `Weak<..>` or an index, never a reference.
- **A sentinel, not a pointer to anything.** `Support/TreeSupport.hpp:65` `NO_PARENT` is
  `static constexpr SupportNode *` — the "no parent" marker, not a pointer at a live node. The
  vocabulary has no sentinel class, so decide one with evidence (the declaration and the comparisons
  against it) or make it `unknown` with the `path:line` of the use that leaves it unclear and a
  conservative Rust type. In Rust a sentinel like this usually disappears into `Option`'s `None`:
  say in the pull request what the cell names and why. It may not be left blank.
- **Config borrows.** `TreeSupport::m_print_config` and `m_object_config`, and
  `SupportMaterial.hpp:91`–`:92`'s pair on `PrintObjectSupportMaterial`, all `const`, point at
  configuration owned by the print. `TreeSupport::m_object` (`PrintObject *`, non-`const`) points
  back at the object being supported — the same back-reference shape the `root` batch meets in
  `Layer.hpp`.
- **Two rows whose declared type is not a plain pointer.** `TreeSupport::overhang_types` is
  `std::map<const ExPolygon*, OverhangType>` — pointers as *keys*, into polygons owned elsewhere —
  and `TreeSupportCommon.hpp`'s three `SupportGeneratorLayersPtr &` fields are *references to*
  containers of raw pointers held by the caller. Class the reference rows on who owns the container
  they refer to, and give them a borrow of that container, not of its elements.

### What makes this a 3

Fifteen fields in three headers with a clear approach, and real work in following the support-node
allocation out of the header, in keeping the node graph's links consistent with one another, and in
the two rows whose declared types are a pointer-keyed map and a reference to a pointer container.

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
  its first row, `Support/SupportMaterial.hpp:91`, sits directly below `44-ownership-class-root`'s
  `SVG.hpp:95`, and its last row, `Support/TreeSupportCommon.hpp:789`, sits directly above that
  batch's `Utils.hpp:544`. Git's three-way merge conflicts on edits to adjacent lines, so if the
  root batch merges first you can get a conflict at exactly those two rows through no fault of your
  own. Resolve such a conflict by **keeping both sides' cells**, yours and the other branch's, and
  never by reverting a sibling's filled-in row: a discarded classification is invisible, because
  `just ownership-check` still passes on a blank class until `51-ownership-blank-class-gate` lands.
  Then run `just ownership` and `just ownership-check` again to prove the file is still
  byte-identical to regeneration.

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

- Every one of the 15 rows named above — the rows of `port/ownership.tsv` whose `header path`
  is one of this batch's headers, which is the `field count` of the `ownership-support-01` row of
  `port/ownership-batches.tsv` — has a `class` cell holding one of `owned`, `shared`, `borrowed`,
  `back-reference`, `ffi` or `unknown`, and none of the 15 is blank.
- Each of those 15 rows has a non-empty `evidence` cell holding at least one `path:line`,
  spelled relative to `vendor/OrcaSlicer/src/libslic3r/` as the `header path` column spells a path,
  and a non-empty `rust type` cell naming a concrete Rust type that follows this task's table of
  defaults, or departs from it for a reason the pull request names row by row.
- Every one of those rows whose class is `unknown` carries both the `path:line` of the use that
  leaves the class unclear and the conservative Rust type, and the pull request names each
  `unknown` row and why it is one.
- The branch is rebased on `main`, and `git diff main` then touches no line of `port/ownership.tsv`
  other than those 15, and changes no cell of `header path`, `line`, `declaring type`, `field` or
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
- The evidence for `parent`, `parents`, `child`, `merged_neighbours` and `contact_nodes` cites the
  `path:line` in `Support/TreeSupport.cpp` where support nodes are allocated or freed, and the link
  rows carry a class and Rust shape consistent with one another.
- `Support/TreeSupport.hpp:65` `NO_PARENT` has a non-blank class, evidence and a Rust type, and the
  pull request says what was done with it as a sentinel.
- `overhang_types` and the three `SupportGeneratorLayersPtr &` rows are classed on who owns what
  they point at or refer to, and the pull request names that owner for each.
