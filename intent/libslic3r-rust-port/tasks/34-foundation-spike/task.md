---
kind: task
step: 20
delivers: []
depends: [08-comparator-workspace, 14-manifest-columns, 15-task-cuts]
size: 5
---
# Task: Spike: create the crates and batch the foundation types

## What to build

This is the spike of plan step 20, "Create the crates and hand-design the foundation types".
That step has two halves: the crates, which are created here in one commit, and the foundation
types, which are not ported here. `delivers` is empty because the whole step is groundwork and
delivers no requirement by itself; steps 21, 22 and 23 need it.

The foundation units themselves are cut as their own tasks from `port/foundation.tsv`, the file
this spike commits, in a later pass once this task is on `main`. Port no foundation module here.

### The question this spike answers

**Which in-scope rows of `port/manifest.tsv` are really template-heavy — type-and-trait design
rather than a file-unit — and how do those rows batch into tasks?**

`port/manifest.tsv`'s `template-heavy` column is seeded by a rule written into `port/inventory`
(task `14-manifest-columns`): an in-scope header with no same-stem `.cpp` sibling, or an in-scope
header where `template<`, `constexpr` and `inline` definitions make up more than a quarter of its
non-comment lines. That rule will read some rows wrongly. At the pin
`vendor/OrcaSlicer/src/libslic3r/Point.hpp`, `Config.hpp`, `Model.hpp` and `libslic3r.h` all have
a same-stem `.cpp`, so the first half of the rule misses all four of the heaviest foundation
headers and they can be reached only by the second half, a line-ratio heuristic. The first half is
not narrow in general — it reaches every in-scope header-only row, 77 headers with no same-stem
`.cpp` at the pin, of which plan step 20 counts 76 header-only files holding 14,840 lines — so the
reading work is roughly those marked rows plus the four named candidates. `Model.hpp` is the
likeliest of the four to be read wrongly. Until that set is corrected, no
foundation task can be sized, because the set is what the tasks are made of: the unit of work in
`docs/WORKFLOW.md` is one `.cpp` plus its same-stem `.hpp` **except** these files, which are
designed once, by hand, before fan-out.

Answer the question by reading each candidate header in the vendored tree and deciding, per row,
whether it is type-and-trait design that every other unit depends on the shape of (`yes`) or a
file-unit the fan-out can draft (`no`). Read at least: every in-scope row the committed manifest
already marks `yes`, which is on the order of 76 rows at the pin; and every in-scope header whose
same-stem `.cpp` exists but which the second
half of the rule marked `no` and which the pipeline's entry points reach widely —
`Point.hpp`, `Config.hpp`, `Model.hpp` and `libslic3r.h` among them. Write the decision and its
one-line reason for every row you changed into this task's pull request body, since the cell
itself carries no reason column.

Header-only files are 76 files and 14,840 of the library's 340,112 lines, 4.36 per cent, so the
corrected set is expected to stay small; a set that comes out far larger than that is evidence
the correction went the wrong way and is worth saying so in the pull request rather than
batching.

### The crates, in one commit

Nothing before this task creates the crates `port/cut.tsv` names, and a hundred Phase
A tasks each creating a crate skeleton on its own branch is a hundred-way conflict. The count is
whatever that file's `crate` rows hold and is not asserted here: the plan speaks of "roughly 43
crates", which came from a module measurement whose derivation neither the spec nor the plan states,
while task `07-crate-cut` derives the module itself and prints the count it produced. Read the count
from `port/cut.tsv`, and where it is far from 43 say so in the pull request rather than working
around it. So this task adds:

- `port/crate-skeletons`, a standalone executable with PEP 723 inline metadata run through `uv`,
  like every other script under `port/` and `oracle/`;
- `just crate-skeletons`, the recipe that runs it.

It reads `port/cut.tsv` and writes, for every crate the cut names, `crates/<crate>/Cargo.toml`
and `crates/<crate>/src/lib.rs`. The `lib.rs` holds only the crate's doc comment and the `mod`
lines Phase B will fill — none at first. Each `Cargo.toml` names the crate, inherits the
workspace's `[workspace.package]` fields where the root `Cargo.toml` defines them and otherwise
repeats what `crates/oracle-compare/Cargo.toml` uses.

**What a generated `Cargo.toml` declares, which is the part two builders would read differently.**
No third-party dependency, and **the path dependencies the cut names**: for each crate, one
`[dependencies]` path entry per crate its `dep` rows in `port/cut.tsv` name, plus one
`[dev-dependencies]` path entry on `slic3r-proof`, which every ported unit's proof calls. They are
generated from `port/cut.tsv` rather than added later because no port task may add a dependency and
no other task is named as the one that would: task `35-trial-run` needs the `slic3r-proof`
dev-dependency and the cut's edges for its three units, and every Phase B unit needs its crate's
edges to compile at all. The plan lists "Step 20's skeleton task" among the steps that may change
`Cargo.lock`, which is what this makes use of. Where `19-porting-contract` names a crate for the
insertion-ordered map, that one third-party dependency is added by the task that first needs it and
not generated here.

Re-running the script adds a crate that is missing and changes no file that exists, so it stays
safe to run after the cut is regenerated. The root `Cargo.toml` needs no edit and must not get
one: task `08-comparator-workspace` made it `members = ["crates/*"]` precisely so a directory
appearing is enough. `Cargo.lock` does change here, because the workspace gains members; this is
one of the few tasks allowed to commit it.

### The batch file

Run `just task-cuts foundation`, the recipe task `15-task-cuts` adds, and commit the
`port/foundation.tsv` it writes. Its rows are the in-scope `template-heavy` rows in crate-level
order then path, batched to **at most 4 units or 2,500 source lines**, whichever comes first — a
smaller batch than Phase A's because each unit here is hand-designed, compiles and carries a
proof. Each row carries the task's name, its unit count, its line sum and its unit paths. If
`just task-cuts foundation` disagrees with those batch limits or those columns, fix the script in
this task rather than hand-editing the file it writes.

Correct the `template-heavy` cells **before** running it, so the file is cut from the corrected
set.

**A `template-heavy: yes` header whose same-stem `.cpp` is in scope brings that `.cpp` row into the
same foundation unit.** The unit count counts the pair as one and the line sum adds both rows. That
is stated because the four likeliest candidates — `Point.hpp`, `Config.hpp`, `Model.hpp` and
`libslic3r.h` — all have one, and task `07-crate-cut` gives a `.cpp` and its same-stem `.hpp` **one**
`target module`: were the header designed here and the `.cpp` left to Phase A, the Phase A draft
would be written over the hand-designed module at that same path, and nothing would skip it —
`15-task-cuts`'s phase-a skip is by derived status, and a status is derived per row, so the
unported `.cpp` row would still be batched. So no Phase A row ever shares a `target module` with a
foundation unit.

### Editing the manifest here, and why that is allowed

`template-heavy` is one of the columns `port/inventory` seeds and then carries forward verbatim,
and `just inventory-check` diffs the derived columns only, so correcting a cell by hand passes it.
That is deliberate and it is this task's job. It is also the exception: a port task never edits
`port/manifest.tsv`, and a derived column — path, subsystem, lines, either level, scope and its
reason, target crate and module, pipeline stage — is never edited by hand in any task. Run
`just inventory` once after the correction to confirm it leaves the corrected cells exactly as it
found them.

### What it waits for and what it must not touch

This spike needs only the workspace with its `crates/*` glob, the committed `port/manifest.tsv`
and `port/cut.tsv`, and `just task-cuts` — which is what its `depends` names. It creates no
module, so it needs neither the FFI bridge nor the proof harness nor the port's surface, although
the foundation tasks cut from `port/foundation.tsv` later do wait on those, which is the rest of
step 20's "waits on" line.

It must not: port or draft any module; commit a fuzz target; edit the root `Cargo.toml`; edit a
derived manifest column; change anything under `vendor/OrcaSlicer/` or move the submodule
pointer, which stays at `f92a7c065c5718b8ef1baac1cd704df2911238f4`; or write any of the generated
reports under `port/reports/`. It also must not create `just port-check` or add a line to it:
that recipe is task `24-port-check-ci`'s, it is not among this task's dependencies, and writing it
here would collide with that task.

### Why this is a 5

The unknown is which rows are really template-heavy. The seed rule will read some wrongly,
`Model.hpp` likeliest, and until that set is corrected no foundation task can be sized at all, so
the answer has to be reached by reading the vendored headers one at a time. It also creates the
roughly 43 crates in one commit, because a hundred tasks each creating one is a hundred-way
conflict, and that commit has to leave the workspace building.

## Done when

- `port/crate-skeletons` and `just crate-skeletons` exist, and the recipe runs it through `uv`.
- The committed tree holds a `crates/<crate>/` with a `Cargo.toml` and a `src/lib.rs` — doc comment
  only, no `mod` line — for every `crate` row of `port/cut.tsv`.
- With `crates/` removed except `crates/oracle-compare/`, `just crate-skeletons` recreates exactly
  those directories and `git status` is clean afterwards; and run again on the committed tree it
  changes no file that already exists.
- The committed crate skeletons are in the tree and `cargo build --workspace` and
  `cargo test --workspace` pass with them.
- `cargo fmt --check` exits 0 with the generated skeletons committed. It is run directly rather than
  through `just port-check`, which this task does not depend on, and it matters because
  `cargo fmt --check` is one of the five lines of that recipe and every later port task's pull
  request has to pass it.
- Each generated `Cargo.toml` declares no third-party dependency, one path dependency per crate its
  `dep` rows in `port/cut.tsv` name, and a path dev-dependency on `slic3r-proof`; a reviewer picks
  one crate and checks its dependencies against its `dep` rows.
- `git diff` for this task shows no change to the root `Cargo.toml`.
- `just check` and `just guard` pass.
- `just inventory-check` passes with the corrected `template-heavy` cells committed, and running
  `just inventory` afterwards leaves those cells unchanged.
- **The answer is committed in `port/foundation.tsv`**: one row per foundation task, batched to at
  most 4 units or 2,500 lines, in crate-level order then path, each row carrying the task's name,
  its unit count, its line sum and its unit paths, and every in-scope row the manifest marks
  `template-heavy: yes` appears in exactly one row of it.
- No row of `port/foundation.tsv` names a unit whose `target module` is also the target of a row
  left to Phase A: where a `template-heavy: yes` header has an in-scope same-stem `.cpp`, that
  `.cpp`'s row is in the same foundation unit.
- The pull request body lists every `template-heavy` cell this task changed, with its one-line
  reason, and states whether `Model.hpp` was one of them.
- No file under `crates/*/src/` other than a generated `lib.rs` is added, and no file under
  `crates/slic3r-ffi/fuzz/fuzz_targets/` is added.
- Nothing under `vendor/OrcaSlicer/` changed and `git submodule status` still shows
  `f92a7c065c5718b8ef1baac1cd704df2911238f4`.
- The rest of step 20 — one task per row of `port/foundation.tsv`, each porting its units — is cut
  in a later pass, once this task is on `main`.
