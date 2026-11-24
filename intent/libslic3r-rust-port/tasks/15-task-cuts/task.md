---
kind: task
step: 11
delivers: [R8, R9]
depends: [14-manifest-columns]
size: 3
---
# Task: Batch a plan step's units into its own cut file

## What to build

This is **part 5 of step 11** of `intent/libslic3r-rust-port/plan.md` ("The scope manifest and the
crate cut"): `port/cut-tasks` and `just task-cuts <phase>`. Parts 1 to 4 are
`03-manifest-file-set`, `06-manifest-scope`, `07-crate-cut` and `14-manifest-columns`, which build
`port/manifest.tsv` and `port/cut.tsv`. This script lives in step 11 because the manifest and the
cut are what it batches, and no later step creates it.

Add the standalone executable `port/cut-tasks` and the recipe `just task-cuts <phase>`. Four plan
steps cannot be sized until the manifest is on `main`, so each is cut spike-first: the step's first
task runs this script for its phase and commits the cut file, and the rest of that step's tasks are
cut from that file in a later pass. The four phases, each reading `port/manifest.tsv` and
`port/cut.tsv` and writing exactly the path named here:

**`ownership`** → `port/ownership-batches.tsv`. It takes its field list from
`port/ownership.tsv`, batches the fields **by subsystem into rows of at most 120 fields each**, in
the manifest's crate-level order, and each row names the task, its subsystem, its field count and
its fields. This is the one phase that reads a second file, so it is runnable only once
`16-ownership-spike` has written `port/ownership.tsv`: while that file is absent the script
**exits non-zero with a message naming it**, rather than batching nothing and writing an empty
file.

**`foundation`** → `port/foundation.tsv`. The in-scope rows whose `template-heavy` column is `yes`,
ordered by crate-level level ascending then path, batched to at most **4 units or 2,500 lines**,
whichever comes first — a smaller batch than `phase-a`'s because each unit there is hand-designed,
compiles, and carries a proof. Each row names the task, its unit count, its line sum and its unit
paths.

**`phase-a`** → `port/taskcuts/phase-a.tsv`. The in-scope rows ordered by crate-level level
ascending, then file-level include-DAG level ascending, then path — Phase A needs no order for
correctness, so this one is chosen because it is deterministic and groups a crate's units together
— batched into consecutive units **within one target crate** until either **6 units or 2,500
source lines** is reached, whichever comes first. A unit of more than 2,500 lines is a task on its
own, and a unit over 5,000 lines is **marked as needing its own plan** in its row;
`PrintConfig.cpp` at 14,438 lines and `GCode.cpp` at 11,805 are the extreme cases. Rows the
manifest marks out of scope get no row, and the units `34-foundation-spike`'s step has already
ported are skipped — decided from the derived status, not from a cell, so a run today correctly
skips nothing. **The script does not re-implement that derivation**: `14-manifest-columns` writes
the text-visible status rule into `port/inventory`, and `port/cut-tasks` calls that same function
rather than writing a third copy of it — a target file present at the manifest row's path carrying a
`PORT STATUS` trailer that names that row's source path, an uncommented `mod` line reaching it, and
no `todo!()` or `unimplemented!()`. A reviewer finds one implementation of the rule, in
`port/inventory`, and a call to it here. Each row names the task, its target crate, its
crate-level level, its unit count, its line sum and its unit paths.

**`phase-b`** → `port/taskcuts/phase-b.tsv`. Crates ordered by crate-level level ascending, and
within a crate units ordered by file-level include-DAG level ascending then path. One row per
crate, except that a crate whose in-scope line sum exceeds 2,500 lines is split along that order
into consecutive batches of at most **6 units or 2,500 lines** — the same batch as `phase-a`
because the work per unit here is strictly harder — and a unit longer than that is its own batch.
Each row names the task, its crate, its **batch index**, its unit count, its line sum, its unit
paths and **the crates it depends on**, read from the `dep` rows of `port/cut.tsv` —
`07-crate-cut` writes one `dep` row per retained crate-to-crate edge, naming the source crate, the
target crate and the references it carries, which is the crate dependency graph; the `edge` rows of
that file are the **cut** edges and are not it, and the `crate-level level` is an ordering and not a
dependency list. Batch N of a crate depends on
batch N-1 of the same crate by that batch index, as well as on every crate below it in the cut
graph; that serialisation is what makes a partial crate compile at all. The two mutual clusters
the cut cannot break, `GCode.cpp` with `GCode/*` and `Model` with `Format`, are each one crate and
are therefore each split into many batches.

A unit is one libslic3r `.cpp` plus its same-stem `.hpp` — one manifest row each, one unit between
them — so a batch's unit count counts units and its line sum adds both rows' lines. Both `.mm`
files are deducted in `06-manifest-scope`, so no unit is Objective-C++.

Keep the output paths exactly as written above, including the asymmetry: `ownership-batches.tsv`
and `foundation.tsv` sit at `port/`, while the two phase files sit under `port/taskcuts/`. Do not
normalise them into one directory; later tasks name them as they are.

**This task commits no cut file.** Each of the four phases is run and committed by the first task
of the step it belongs to — `16-ownership-spike`, `34-foundation-spike`, `37-phase-a-spike` and
`38-phase-b-spike` — so this task adds the script and the recipe and leaves `port/taskcuts/`
without a committed `.tsv`. It does not create `port/ownership.tsv` either; that is
`16-ownership-spike`'s. Every phase's output is deterministic: the sorting is total and the
batching is a single pass over it, so two runs of the same phase against the same manifest produce
identical bytes. The script writes to the path named above, or, given a scratch directory, into
that, which is how the determinism is checked here without committing anything. **The script creates
its output file's parent directory when it is absent**: git tracks no empty directory, so on a fresh
clone `port/taskcuts/` does not exist and a `phase-a` or `phase-b` run would otherwise fail on a
missing directory.

## Done when

- `port/cut-tasks` exists as a standalone executable with PEP 723 inline metadata and no file
  extension, run through `uv`, and `just task-cuts <phase>` forwards to it.
- `just task-cuts foundation`, `just task-cuts phase-a` and `just task-cuts phase-b`, each run
  against the committed `port/manifest.tsv` and `port/cut.tsv` **with a scratch output directory**,
  write their named file there — creating the `taskcuts/` parent when it is absent — and running the
  same phase twice that way produces byte-identical output. No run of this task's own proof writes
  into the committed tree, which is what keeps the last line below true.
- `just task-cuts ownership` exits non-zero and names `port/ownership.tsv` while that file is
  absent, and writes no file.
- `just task-cuts <anything else>` exits non-zero and names the four phases.
- `foundation`'s rows hold at most 4 units or 2,500 lines each, and cover exactly the in-scope rows
  whose `template-heavy` column is `yes`.
- `phase-a`'s rows hold at most 6 units or 2,500 lines each, never mix two target crates in one
  row, cover exactly the in-scope rows that are not already ported, give a unit over 2,500 lines a
  row of its own, and mark a unit over 5,000 lines as needing its own plan.
- `phase-b`'s rows carry a batch index, and a crate's batch N names batch N-1 of the same crate
  among its dependencies along with every crate its `dep` rows in `port/cut.tsv` name.
- `phase-a`'s already-ported skip is the status rule implemented in `port/inventory`:
  `grep -n 'PORT STATUS' port/cut-tasks` finds no second copy of the rule, and a reviewer finds the
  call into `port/inventory` instead.
- Every row of every phase carries its task name, unit count, line sum and unit paths, and
  `phase-a` additionally its target crate and crate-level level.
- No `.tsv` under `port/taskcuts/`, no `port/foundation.tsv`, no `port/ownership-batches.tsv` and
  no `port/ownership.tsv` is committed by this task, and `port/manifest.tsv` and `port/cut.tsv` are
  unchanged by it.
