---
kind: task
step: 16
delivers: [R23]
depends: [05-oracle-provenance, 08-comparator-workspace, 14-manifest-columns, 16-ownership-spike, 22-port-status-gates]
size: 5
---
# Task: Split the checks between the document job and the macOS oracle job

## What to build

The whole of plan step 16: one new `justfile` recipe that composes the checks needing neither the
oracle binary nor a built libslic3r, a line for it in `.asdlc/checks`, three new steps in the
existing ubuntu job of `.github/workflows/ci.yml`, a new macOS arm64 job in the same file, and the
`README.md` corrections that go with them. The point of the split is R23: a task that touches no
code still passes with the document checks alone, and the document checks never wait on a build.

**The recipe.** Add `just port-check` to the `justfile`, beside `just check`, which keeps its
present meaning of linting the asdlc documents. `port-check` runs, in this order:

- `just port-status --per-commit`
- `just inventory-check`
- `just ownership-check`
- `oracle/provenance --check` over the paths plan step 3 names, which task `05-oracle-provenance`
  resolves to four literal arguments — `port/goldens/`, `port/dumps/*/*.dump` (the committed stage
  dumps, whose path that task fixes and which `25-stage-dumper` writes),
  `port/reports/determinism.md` and `port/reports/oracle-parity.md`
- `cargo fmt --check`

Every one of those reads only the vendored source and the repository's own files. Two further
lines join this recipe later, each added by the task that makes it pass, and this task adds
neither: the substitute and `-ffp-contract=off` drift checks of plan step 17
(task `29-ffi-bridge-core`) and the mock check of plan step 18 (task `31-proof-harness`).

`just audit-check` and the perf-backlog whole-file diff are **not** in it. Plan steps 13 and 14 put
those on the regeneration pass, because a whole-file diff of a generated report cannot be a check
that a hundred concurrent branches each pass.

The recipe has to be green on the tree as it stands when this task lands, which is a tree with no
oracle binary, no libslic3r build, no committed goldens and no stage dumps — all four covered groups
absent, which is the normal case and not an edge case. That works because
`05-oracle-provenance` already specifies it: a covered path, or glob, that matches nothing counts
zero files and exits 0, and a count of zero is a pass. So this task relies on that behaviour and
**changes no check's script**, which is what the claim below means.

**The line in `.asdlc/checks`.** Add `just port-check` as a further line beside `just check`. That
file is what a task is held to.

**A line in `.asdlc/checks` does not make CI run it.** The ubuntu job of
`.github/workflows/ci.yml` runs `just check` as a literal step and reads no line of that file, and
`asdlc` has no command that does. So add to that job, by name:

- a step that runs `just port-check`, beside the existing `Checks` step that runs `just check`;
- a Rust toolchain step that installs the version `rust-toolchain.toml` pins —
  `dtolnay/rust-toolchain` given that version, or any action that honours the file — because
  `cargo fmt --check` is in `port-check` and that job sets up only `astral-sh/setup-uv@v7` and
  `extractions/setup-just@v3` today, so without it a task that touches no code fails the checks it
  is held to;
- a shallow submodule initialisation, `git submodule update --init --depth 1 vendor/OrcaSlicer`,
  because the new checks read the vendored tree, placed before the checks that read it.

The checkout step's comment in that job today says the submodule is not checked out because
nothing here reads it. Correct it.

**The macOS arm64 job.** Create it in the same file for everything that needs the oracle binary, a
local libslic3r build, or a comparison against either, and create it with **two lines only**:
`just oracle-fetch` and `cargo test --workspace`. It runs on a pinned arm64 image, not
`macos-latest`, because C2's decision makes macOS arm64 the one platform the goldens are valid
for. It initialises the submodule shallowly as well.

Every other line is added by the task that makes it pass, because a job that fails on every pull
request until the port is finished gates nothing. Name them in a comment in the job, so each later
task adds its own line and no builder writes the job twice: `just oracle-build` by plan step 7
(task `13-build-libslic3r`), `just oracle-parity` by plan step 10 (task `28-oracle-parity`),
`just port-status` without `--per-commit` by plan step 23's last task, and `just compare-stages`,
`just compare-benchy`, `just compare-fixtures` and `just compare-configurations` by plan steps 24
to 27 (tasks `40-compare-stages-ci`, `41-compare-benchy`, `42-compare-fixtures`,
`43-compare-configurations`). `just pr-body-check` is the exception that goes in the ubuntu job,
added by plan step 21 (task `36-pr-body-check`), because it reads the pull request and needs no
build.

**The path filter, and the trap in it.** The macOS job does not run on every pull request:
`just oracle-build` is a dependency superbuild plus the main project from zero downloads with no
compiler cache, and the fan-out is a hundred-plus pull requests. The job runs when the pull request
touches `crates/**`, `oracle/**`, `port/**`, `fixtures/**`, `justfile`, `Cargo.toml`, `Cargo.lock`
or the workflow itself, and is skipped otherwise. **On a push to `main` it runs unconditionally**:
that workflow also triggers on `push: branches: [main]`, where there is no pull request to compute a
diff against, and the fallback this task records — `just oracle-build` and `just oracle-parity`
moving to `main` only — depends on the `main` case running. A filter that evaluated to skipped, or
errored, on a push would silently take that fallback away. The filter has to be **per job** — a filter job
that computes the paths and a `needs`/`if` on the macOS job, or an equivalent — and **not**
`on.pull_request.paths` at the top of the workflow, because a workflow-level filter would also
skip the ubuntu job, and a document-only pull request has to run the document checks and the
guard. That the ubuntu job stays unfiltered and the macOS job is skipped is exactly R23's promise
holding for the whole workflow and not only for `.asdlc/checks`.

**One cache here, and one that arrives with `just oracle-build`.** This job needs two cached paths in
the end, but they have different owners, because a cache block written twice is a duplicated step or
a deleted one:

- **This task's**: `vendor/OrcaSlicer/deps/build/arm64/OrcaSlicer_dep`, the dependency superbuild's
  install prefix and the only part upstream's own workflow keeps, keyed on the OrcaSlicer submodule
  pin (`f92a7c065c5718b8ef1baac1cd704df2911238f4`).
- **Not this task's**: `vendor/OrcaSlicer/build/arm64`, the main project's build tree, without which
  every pull request rebuilds libslic3r from scratch. It arrives with the `just oracle-build` step in
  task `13-build-libslic3r`, keyed on the pin and on the CMake flags that task records — which is
  what plan step 7 says and what that task's own body claims. This task adds no cache for it, and
  cannot key one sensibly, since it does not depend on 13 and adds no step that fills that tree.

Record in the same comment the fallback plan step 16 states, so the choice is made on a
measurement rather than discovered: if the first build's wall-clock or disk cost, which plan step 7
measures and records in `vendor/README.md`, exceeds the runner's limit or the cache's, then
`just oracle-build` and `just oracle-parity` run on `main` only and the per-pull-request macOS job
keeps `just oracle-fetch` and `cargo test --workspace`.

**README corrections.** `README.md:34` says today that code checks get added to the `check` recipe
as code arrives. Correct the Checks bullet to describe two recipes — `just check` for the asdlc
documents and `just port-check` for the code checks a task passes — and the CI bullet to describe
the ubuntu job's steps including the toolchain and submodule steps, and the macOS arm64 job with
its path filter. Correct the `justfile` row of the layout table, which names only `just check` and
`just guard`.

**The branch guard needs nothing from this task**, and that is worth knowing rather than
discovering: `just guard` is `asdlc branch check`, which judges a branch against its documents on
`main`. A branch may change files outside `intent/` freely, and the only file it may add under
`intent/` is its own plan. A port task writes only `crates/**` and its tests, so it passes the
guard as it stands, and `port/manifest.tsv` does not have to be an asdlc document for that to
hold.

**What this task must not do.** It adds none of the gates the eight later tasks add, including
`just port-status` without `--per-commit`, which is the completion gate and belongs in the macOS
job by plan step 23's last task. It does not add `just audit-check`, `just perf-check` or any
whole-file diff of a generated report to `port-check` or to `.asdlc/checks`. It does not touch
`.github/workflows/regenerate.yml`, which task `23-perf-backlog` adds. It adds no cache of
`vendor/OrcaSlicer/build/arm64`. It changes no check's behaviour and no check's own script: every
check it composes, including `oracle/provenance --check`'s tolerance of an absent covered path, is
already written by the task it waits on.

**What it waits on.** Task `08-comparator-workspace` for the workspace, `rust-toolchain.toml` and
`Cargo.lock`, without which `cargo fmt --check` and `cargo test --workspace` have nothing to run
on; task `22-port-status-gates` for `just port-status --per-commit`; task `14-manifest-columns` for
`just inventory-check`; task `16-ownership-spike` for `just ownership-check`; and task
`05-oracle-provenance` for `oracle/provenance --check`. The ownership spike is what makes
`ownership-check` exist and pass over a complete field list; the classification tasks cut from
`port/ownership-batches.tsv` in a later pass fill the rows' classes, and this task does not wait
for them — a blank class is allowed while those tasks are open and plan step 12's last task
removes that allowance. `just oracle-fetch` comes from task `01-oracle-fetch`, which is on `main`
well before this; if it is not, the macOS job is still created with that line, since this task
writes no recipe of its own for it.

**Why it is a 5, and why it is not split.** One recipe composing five checks, three new steps in
the ubuntu job, a new macOS arm64 job with a per-job path filter and two caches keyed on the pin
and the flags, and the README corrections. It is not split, because both halves edit
`.github/workflows/ci.yml`, the file the plan's Risks section names as the shared conflict surface
and that eight later tasks each add a line to; splitting it would turn one edit into two conflicts
for no gain.

## Done when

- `cat .asdlc/checks` shows `just check` and `just port-check`, one to a line.
- `just port-check` exits 0 on a machine with no oracle binary and no libslic3r build, and on a
  tree where every in-scope manifest row's derived status is `not-started`.
- `just port-check` runs `just port-status --per-commit`, `just inventory-check`,
  `just ownership-check`, `oracle/provenance --check port/goldens/ 'port/dumps/*/*.dump' port/reports/determinism.md port/reports/oracle-parity.md`,
  and `cargo fmt --check`, and nothing else.
- `just port-check` contains no `just audit-check`, no perf-backlog diff and no command that needs
  the oracle binary or a libslic3r build.
- The ubuntu job of `.github/workflows/ci.yml` has a step running `just check`, a step running
  `just port-check`, a Rust toolchain step installing the version `rust-toolchain.toml` pins, and
  a shallow `git submodule update --init --depth 1 vendor/OrcaSlicer` step before the checks that
  read the vendored tree; the checkout step's comment no longer says the submodule is unread.
- `.github/workflows/ci.yml` has a macOS arm64 job, on a pinned arm64 image, whose only check
  lines are `just oracle-fetch` and `cargo test --workspace`, and whose comment names the **seven**
  lines later tasks add to this job — `just oracle-build` (task `13-build-libslic3r`),
  `just oracle-parity` (task `28-oracle-parity`), `just port-status` without `--per-commit` (plan
  step 23's last task, which is not cut yet and is referred to by its step), and
  `just compare-stages`, `just compare-benchy`, `just compare-fixtures` and
  `just compare-configurations` (tasks `40-compare-stages-ci`, `41-compare-benchy`,
  `42-compare-fixtures`, `43-compare-configurations`) — notes that `just pr-body-check` is the
  eighth later line and belongs to the ubuntu job, and records the fallback that moves
  `just oracle-build` and `just oracle-parity` to `main` only.
- The macOS job also initialises the submodule shallowly with
  `git submodule update --init --depth 1 vendor/OrcaSlicer`, before `cargo test --workspace`, which
  runs `oracle-compare`'s tests and those read vendored fragments.
- That job is skipped for a pull request that changes only a document, and runs for one that
  touches `crates/**`, `oracle/**`, `port/**`, `fixtures/**`, `justfile`, `Cargo.toml`,
  `Cargo.lock` or the workflow itself, and runs unconditionally on a push to `main`; the filter is
  per job and the workflow has no `on.pull_request.paths` key, so the ubuntu job runs on every pull
  request.
- The macOS job caches `vendor/OrcaSlicer/deps/build/arm64/OrcaSlicer_dep` with the submodule pin
  in the cache key, and **adds no cache for `vendor/OrcaSlicer/build/arm64`**, which arrives with
  `just oracle-build` in task `13-build-libslic3r`.
- A pull request that changes only a document is green with the document checks alone and shows no
  macOS job.
- `README.md` describes two recipes, the ubuntu job's toolchain and submodule steps, and the macOS
  arm64 job with its path filter, and its layout table row for `justfile` is current.
- `git diff` shows no `just port-status` without `--per-commit` added anywhere, no new
  `.github/workflows/regenerate.yml`, and no change to any check's own script.
