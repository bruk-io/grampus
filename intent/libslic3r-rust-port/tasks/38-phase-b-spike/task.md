---
kind: task
step: 23
delivers: [R26]
depends: [15-task-cuts, 37-phase-a-spike]
size: 2
---
# Task: Spike: batch every crate into Phase B tasks

## What to build

This is the spike of plan step 23, "Phase B — compile bottom-up, crate by crate". It is the first
and only task cut from that step so far. The compile tasks of the step are cut from the file this
task commits, `port/taskcuts/phase-b.tsv`, in later passes — **one pass per crate-level level**,
for the same reason step 22 gives: one draft message has to carry every task's file in full.

The question the spike answers: **how do the crates the cut defines split into Phase B batches, and
in what order do those batches depend on each other?** The crate list comes from `port/cut.tsv` and
the per-crate unit lists and line sums from `port/manifest.tsv`, so neither the batch count nor the
dependency chain can be known from the plan. The batch index this file assigns is what the later
tasks' `depends` is read from, which is why it has to exist before any of them is cut.

Answer it by running the batcher, not by writing a new one. `port/cut-tasks` and
`just task-cuts <phase>` come from step 11, cut as task `15-task-cuts`, and the whole of this task
is:

1. Run `just task-cuts phase-b` on a `main` that holds `port/manifest.tsv` and `port/cut.tsv`,
   which are the two files that phase reads. `port/taskcuts/phase-a.tsv` is on `main` too by then —
   that is what the dependency on `37-phase-a-spike` is for — but it is what the committed result is
   **checked against**, not an input to the batcher: task `15-task-cuts` states that all four phases
   read the manifest and the cut, and that `ownership` is the one phase reading a second file.
2. Read `port/taskcuts/phase-b.tsv` and check it against the rules below. Where a row is wrong
   because the batcher is wrong, fix `port/cut-tasks` and regenerate; never hand-edit a row of the
   generated file.
3. Commit `port/taskcuts/phase-b.tsv`, and `port/cut-tasks` if it needed a fix.

The batching rules the file has to satisfy, all from step 23:

- Crates are ordered by `crate-level level` ascending. Within a crate, units are ordered by
  file-level include-DAG level ascending, then path.
- One task per crate, except that a crate whose in-scope line sum exceeds 2,500 lines is split
  along that order into consecutive batches of at most 6 units or 2,500 lines — the same batch size
  as Phase A, because the work per unit here is strictly harder: making it compile, adding its
  `mod` line, giving it its proof at its manifest tier and its differential fuzz target where its
  row says `required`. A unit longer than that is its own batch; `PrintConfig.cpp` at 14,438 lines
  and `GCode.cpp` at 11,805 lines are the extreme cases. Most crates are split, and the two mutual
  clusters the cut cannot break — `GCode.cpp` with `GCode/*`, and `Model` with `Format` — are each
  one crate and are split into many batches.
- Each row carries the task's name, its crate, its **batch index**, its unit count, its line sum,
  its unit paths and the crates it depends on.
- **Batch N of a crate depends on batch N-1 of the same crate**, by that batch index, as well as on
  the tasks of every crate below it in the cut graph. That serialisation is what makes a partial
  crate compile at all, and it works because the include graph is acyclic at *file* granularity
  even though the module graph is one 39-module cycle: ordering a crate's batches by file-level
  include-DAG level means every unit in batch N includes only units in batches 1 to N-1 of its own
  crate, or units in crates below it. It is not a guarantee — Rust's `use` graph is not the
  `#include` graph — and a batch that therefore does not `cargo check` is evidence the crate cut or
  the batch boundary is wrong, to be re-cut with `just task-cuts phase-b` after the cut is
  corrected, never stubbed around.
- The unit set is the same as `port/taskcuts/phase-a.tsv`'s: Phase B's work is to make the drafted
  modules compile, so the rows step 20 already ported are not in it and neither is any out-of-scope
  row. **That exclusion is by the same derived-status rule `phase-a` uses**, and task
  `15-task-cuts` states it only under `phase-a`, so if `port/cut-tasks`'s `phase-b` does not apply
  it the committed file will hold the foundation units and adding the exclusion to that script is
  part of this task — the one fix of the batcher this task expects to need, which is why the
  instruction above says to fix the script and regenerate rather than hand-edit a row.

**What this task waits for.** It depends on `15-task-cuts` for the batcher and on
`37-phase-a-spike` for the unit set it has to agree with. It does not wait for any draft to exist:
the batcher reads `port/manifest.tsv` and `port/cut.tsv` and nothing in the Rust
tree, so this file can be committed while Phase A is still being drafted. The compile tasks cut
from it later are the ones that wait — each on the Phase A task that drafted its units, on batch
N-1 of its own crate, and on the last batch of every crate below it in the cut graph.

**What this task must not do.** It makes no module compile, adds no `mod` line, and writes no Rust.
It edits no cell of `port/manifest.tsv`, no generated report under `port/reports/`, no root
`Cargo.toml`, no `Cargo.lock`, and nothing under `vendor/OrcaSlicer/`. It adds no line to
`.github/workflows/ci.yml` — `just port-status`, the completion gate R26 is worded as, is added to
the macOS job by the **last** task of step 23, which is the moment that gate starts holding, and
not by this spike.

Why this is small: it runs a batcher that already exists, reviews the batching and the dependency
chain it produces, and commits one generated file.

## Done when

- `port/taskcuts/phase-b.tsv` is committed, and `just task-cuts phase-b` run again leaves it
  unchanged.
- Every unit that has a row in `port/taskcuts/phase-a.tsv` appears in exactly one row of
  `port/taskcuts/phase-b.tsv`, and no other unit appears in it.
- Every row's units belong to one crate; the rows of a crate are numbered by batch index from 1 in
  file-level include-DAG then path order; and each row holds at most 6 units or at most 2,500
  lines, except a row holding a single unit longer than 2,500 lines.
- Each row names the crates it depends on, and every row with batch index N>1 names batch N-1 of
  its own crate among them.
- Crates appear in `crate-level level` ascending order, and `GCode.cpp` with `GCode/*` and `Model`
  with `Format` are each one crate.
- `just check` and `just port-check` pass, and `git status` shows no change to
  `port/manifest.tsv`, `port/reports/`, `Cargo.toml`, `Cargo.lock`, `.github/workflows/ci.yml` or
  `vendor/OrcaSlicer/`.
- The pull request body states the crate count and the row count the file came out at, and which
  crate-level levels exist, so the passes that cut the compile tasks know how many passes they
  need.
