---
kind: task
step: 24
delivers: [R20]
depends: [39-compare-stages, 38-phase-b-spike, 24-port-check-ci]
size: 1
---
# Task: Gate the per-stage comparison in the macOS job

## What to build

This is the second of the two halves plan step 24 orders separately. Task `39-compare-stages`
builds the command `just compare-stages`, during the Phase B fan-out and before it finishes,
because R20 and C3 make the per-stage comparison the *first* comparison against the oracle. This
task does the other half: it makes that command a gate, which it can only be once every stage is
ported. The step's `delivers` is R20 for both halves.

Add one step to the **macOS arm64 job** of `.github/workflows/ci.yml` that runs
`just compare-stages`. That is the whole change to the job's check lines.

**What that step reads, and where it sits.** `just compare-stages` compares against the committed
stage dumps under `port/dumps/`, which task `25-stage-dumper` commits, and its fallback for the two
seamless stages compares the port's G-code against the built oracle's exported G-code — which
`just oracle-build` does not produce, since that recipe builds the library and exports nothing. So
the step goes **after** the job's `just oracle-build` step, and this task adds one
`just oracle-dump 3DBenchy base-fff` step immediately before its own line, as part of its own
change, to produce that run's G-code and refresh the dumps on the runner. Those two lines are this
task's whole diff: without the dump step the gate goes red on a missing dump or a missing oracle
G-code and this task has no authority to fix it, which is why the step is named here rather than
left to be discovered.

Plan step 16 creates that job with two lines only, `just oracle-fetch` and
`cargo test --workspace`, and leaves every gate to the step that makes it pass. Eight later steps
each add one line to it and each names its own line; this task's line is `just compare-stages` and
no other. Add the step beside the job's existing steps, in the same shape they use, under the
path filter and the two caches that job already carries.

What this task does not do, because another task owns it:

- It does not create or restructure the macOS job, change its path filter, its caches, its
  checkout or its submodule step. Task `24-port-check-ci` owns those.
- It does not write, change or test the `just compare-stages` recipe, `crates/oracle-compare` or
  anything under `crates/`. Task `39-compare-stages` owns the command and its behaviour.
- It does not add the line to `.asdlc/checks` or to the ubuntu job. R23 keeps every check that
  needs the oracle binary, a local libslic3r build or a comparison against either out of both, so
  that a document-only pull request still passes with the document checks alone.
- It adds no other step's line to the workflow. `just port-status` without `--per-commit` is added
  by the last task of plan step 23; `just compare-benchy`, `just compare-fixtures` and
  `just compare-configurations` by the tasks of steps 25, 26 and 27.
- Nothing under `vendor/OrcaSlicer/` changes and the submodule pointer stays at
  `f92a7c065c5718b8ef1baac1cd704df2911238f4`.

**When it lands.** `just compare-stages` reports `not ported` rather than failing for a stage
whose manifest rows are not all `ported`, so it is useful from the day task `39-compare-stages`
merges, and it gets stronger with every Phase B batch. As a CI gate it only holds once the whole
pipeline is ported, so this task is built after the Phase B fan-out is complete. Besides
`39-compare-stages` and `24-port-check-ci` it depends on the spike `38-phase-b-spike`, and it also
waits for every task cut from that spike's `port/taskcuts/phase-b.tsv` in the later passes — those
tasks do not exist yet, and `just port-status` without `--per-commit` passing is the signal that
they are all merged.

## Done when

- The macOS arm64 job of `.github/workflows/ci.yml` has a step, added by this task, that runs
  `just compare-stages`.
- The branch changes `.github/workflows/ci.yml` and nothing else: no `justfile` recipe, no file
  under `crates/`, `oracle/`, `port/` or `.asdlc/`, and no line of the macOS job but its own two —
  the `just oracle-dump 3DBenchy base-fff` step and the `just compare-stages` step, in that order,
  both after `just oracle-build`.
- On this task's own pull request the macOS job is green and that step passes:
  `just compare-stages` exits 0, prints one line per stage with the level reached, prints
  `not ported` for no stage, and prints `fallback: exported G-code` for G-code emission and conflict
  check rather than naming `psSkirtBrim` as the first difference. That holds because every Phase B
  task is merged by then, which `just port-status` without `--per-commit` passing in the same job is
  the signal for.
- `just check` and `just port-check` pass, and `just guard` reports nothing.
