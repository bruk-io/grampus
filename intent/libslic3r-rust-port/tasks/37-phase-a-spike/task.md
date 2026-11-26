---
kind: task
step: 22
delivers: []
depends: [15-task-cuts, 34-foundation-spike, 35-trial-run, 52-foundation-common-defs, 53-foundation-int128-queue, 54-foundation-objectid-clonable, 55-foundation-libslic3r-execution, 56-foundation-point, 57-foundation-utils, 58-foundation-bounding-box, 59-foundation-config, 60-foundation-trees-grids, 61-foundation-line-slicing, 62-foundation-polygon, 63-foundation-polyline, 64-foundation-print-config, 65-foundation-expolygon-flow, 66-foundation-geometry, 67-foundation-extrusion-entity, 68-foundation-surface, 69-foundation-clipper-utils, 70-foundation-triangle-mesh, 71-foundation-sla-indexed-mesh, 72-foundation-arachne-grids, 73-foundation-arachne-indices, 74-foundation-arachne-half-edge, 75-foundation-edge-grid, 76-foundation-voronoi-curves, 77-foundation-voronoi-offset, 78-foundation-layer-skeletal, 79-foundation-model, 80-foundation-print-base, 81-foundation-print, 82-foundation-fill-base, 83-foundation-support-params, 84-foundation-precise-seam]
size: 2
---
# Task: Spike: batch every in-scope unit into Phase A tasks

## What to build

This is the spike of plan step 22, "Phase A — draft every in-scope unit". It is the first and only
task cut from that step so far. The draft tasks of the step are cut from the file this task
commits, `port/taskcuts/phase-a.tsv`, in later passes — **one pass per crate-level level**, because
one draft message has to carry every task's file in full and roughly a hundred rows do not fit in
one message.

The question the spike answers: **how many Phase A units are there, and how do they batch into
tasks?** Neither the unit count nor the per-crate batching can be known from the plan, because the
units are the in-scope rows of `port/manifest.tsv` and the batching is per target crate. That is
what makes the rest of step 22 unsizable until this file exists.

Answer it by running the batcher, not by writing a new one. `port/cut-tasks` and
`just task-cuts <phase>` already exist — they come from step 11, cut as task
`15-task-cuts` — and the whole of this task is:

1. Run `just task-cuts phase-a` on a `main` that already holds `port/manifest.tsv`, `port/cut.tsv`
   and step 20's ported foundation modules.
2. Read the file it writes, `port/taskcuts/phase-a.tsv`, and check the batching against the rules
   below. Where a row is wrong because the batcher is wrong, fix `port/cut-tasks` and regenerate;
   never hand-edit a row of the generated file.
3. Commit `port/taskcuts/phase-a.tsv`, and `port/cut-tasks` if it needed a fix.

The batching rules the file has to satisfy, all of them from step 22, so a reviewer checks the file
against this list and not against the script:

- The rows are the in-scope rows of `port/manifest.tsv`, as units: one libslic3r `.cpp` plus its
  same-stem `.hpp` is one unit, with one manifest row each and one unit between them. Rows the
  manifest marks out of scope get no task, and so do the rows step 20 already ported — the
  `template-heavy` rows that the foundation tasks hand-designed, which are modules and not drafts.
  **A unit whose header step 20 ported and whose `.cpp` it did not is still batched**, as a one-row
  unit naming only that `.cpp` path. Task `34-foundation-spike` is written so this should not arise
  — a `template-heavy: yes` header whose same-stem `.cpp` is in scope brings that `.cpp` into the
  same foundation unit — but the rule is stated because the alternative reading, skipping the whole
  unit, would leave that `.cpp` with no Phase A task and no Phase B task and so unportable, which
  R26 forbids.
- Order: `crate-level level` ascending, then file-level include-DAG level ascending, then path.
  Phase A needs no order for correctness; this one is chosen because it is deterministic and groups
  a crate's units together.
- Batches are **within one target crate**: consecutive units of the same crate are batched until
  either 6 units or 2,500 source lines is reached, whichever comes first.
- A unit of more than 2,500 lines is a task on its own. `PrintConfig.cpp` at 14,438 lines and
  `GCode.cpp` at 11,805 lines are the extreme cases.
- A unit over 5,000 lines is marked in its row as needing its own plan, so the pass that cuts it
  cuts it as a size-8 task.
- Each row carries the task's name, its target crate, its crate-level level, its unit count, its
  line sum and its unit paths.

**What this task waits for.** It depends on `15-task-cuts` for the batcher, on
`34-foundation-spike` for step 20's set of hand-designed foundation units, and on `35-trial-run`
because the trial corrects `docs/PORTING.md` before any fan-out pull request is opened. It also
waits for the **foundation tasks cut after `34-foundation-spike`** to be merged: the rows this file
skips are the ones those tasks ported, so running the batcher before they land puts foundation
units into Phase A batches. **Check that with an executable check rather than with
`just port-status`**, which prints counts and not per-row status and whose remaining count is
legitimately large at this point: for every unit path in `port/foundation.tsv`, confirm that a file
exists at that row's `target module` path, that it carries a `PORT STATUS` trailer naming that row's
source path, and that an uncommented `mod` line reaches it. Where any of them fails, the foundation
tasks have not all landed and the batcher is not run yet.

**What this task must not do.** It writes no Rust module and drafts nothing — the drafts are the
later tasks' work. It edits no cell of `port/manifest.tsv` (the derived columns are regenerated and
the judgement columns are carried forward, step 11), no generated report under `port/reports/`, no
root `Cargo.toml`, no `Cargo.lock`, and nothing under `vendor/OrcaSlicer/`. It adds no line to
`.github/workflows/ci.yml`.

Why this is small: it runs a batcher that already exists, reviews the batching it produces, and
commits one generated file.

## Done when

- `port/taskcuts/phase-a.tsv` is committed, and `just task-cuts phase-a` run again leaves it
  unchanged.
- Every in-scope row of `port/manifest.tsv` whose own derived status is not `ported` appears in
  exactly one row of `port/taskcuts/phase-a.tsv`, and no out-of-scope row and no row whose own
  derived status is already `ported` appears in it at all.
- Every row's units share one target crate, and every row holds at most 6 units or at most 2,500
  source lines, except a row holding a single unit longer than 2,500 lines.
- Every row whose single unit is longer than 5,000 lines is marked as needing its own plan.
- The rows are in `crate-level level`, then file-level include-DAG level, then path order, and each
  carries a task name, a target crate, a crate-level level, a unit count, a line sum and its unit
  paths.
- `just check` and `just port-check` pass, and `git status` shows no change to
  `port/manifest.tsv`, `port/reports/`, `Cargo.toml`, `Cargo.lock`, `.github/workflows/ci.yml` or
  `vendor/OrcaSlicer/`.
- The pull request body states the unit count and the row count the file came out at, so the pass
  that cuts the draft tasks knows how many passes it needs.
