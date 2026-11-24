---
kind: task
step: 14
delivers: [R14, R15, R16, R29]
depends: [21-port-status-markers, 18-divergences-regeneration]
size: 5
---
# Task: Regenerate the perf backlog from the PERF markers

## What to build

This is **part 3 of plan step 14**, "port-status", so its `delivers` is the whole step's. Part 1
(task `21-port-status-markers`) added `port/port-status`, `just port-status` and the text-based
per-module checks. Part 2 (task `22-port-status-gates`) adds the derived status, the burn-down
gate and the document-section checks. This task adds the last third: `port/reports/perf-backlog.md`
and how it is generated, the per-marker assertion a port task passes, and the regeneration pass on
`main` that is the only writer of the four generated reports.

**The backlog.** `port/reports/perf-backlog.md` is the only place a knowingly slower translation
may live, and it is the handover to the following intent, which is the one that does idioms and
optimization. It is generated, never hand-written: one row per `// PERF(port):` marker in the port
tree, each row naming the module the marker is in, the vendored source path the marker names, the
marker's line and its reason. Plan step 13's task `18-divergences-regeneration` added
`port/regenerate` and `just regenerate`, which runs `just audit-order` and the perf-backlog
regeneration together so that one command rewrites every generated report; this task writes the
perf-backlog half of it. `just regenerate` prints the marker count it found.

Add the drift-detecting variant beside it: `port/regenerate --check`, wired to the recipe
`just perf-check`, which regenerates the backlog into a scratch directory and fails when the
committed file differs. That spelling is this task's to fix — nothing else in the plan names this
check — and it is the spelling the regeneration pass below uses. It is the sibling of plan step
13's `just audit-check`, and like that one it is **not a check a port task passes**: a whole-file
diff of a generated report cannot be, because a hundred task branches would each commit a
regeneration that the next merge makes stale.

**The per-marker assertion, which is what a port task passes instead.** Add to
`port/port-status`'s `--per-commit` subset an assertion over markers, with no regeneration and no
whole-file diff anywhere in it:

- Every `// PERF(port):` and `// FIDELITY(port):` marker in the tree is well formed: on its own
  line, naming the module's vendored source path, and carrying the reason in the shape the
  report's row needs. A malformed marker fails.
- Every row of the committed `port/reports/perf-backlog.md` still has its marker in the tree, with
  the same source path and reason. A row whose marker was deleted or edited fails, which is how a
  branch that quietly drops a declared slower translation is caught.
- The same holds for the rows of `port/reports/divergences.md` **whose source cell is the
  `// FIDELITY(port):` markers, and for no other row of that file**. Task
  `18-divergences-regeneration` regenerates that file from four sources, only one of which is the
  markers, and rows from the other three — the two order audits, `oracle-parity.tsv` and
  `determinism.tsv` — have no marker in the tree by design, so an assertion over every row would
  fail on each of them. **That requires a per-row source cell in `divergences.md`**: every row of
  that file carries, as its first cell, which of the four sources contributed it, and this task's
  assertion selects the marker rows by that cell. Task 18 states the file's row contents without
  naming columns and carries the source attribution only in a whole-file summary section, so the
  per-row cell is declared here and `18-divergences-regeneration` is where the generator writes it;
  flag it to the reviewer of this pull request, since the two tasks have to agree on that cell.
- A marker the branch adds that no row covers yet is **not** a failure. The regeneration pass is
  the only writer of the reports, so a new marker has no row until that pass runs, and failing it
  would make every port task that records a slower translation unpassable. This is why the row
  direction is checked report-to-marker and not marker-to-report.

**This task deliberately inverts the direction the plan's Proof section states, for both markers,
and here is the statement of it** — `plan.md` is on `main` and is not edited from this branch, so
this is where a reviewer reads it. The plan's proof line for R15 says
`just port-status --per-commit` "exits non-zero on a `// PERF(port):` marker that is malformed or
has no row in that report", and its proof line for R18 says the same of a `// FIDELITY(port):`
marker with "no row in `divergences.md`". This task implements the malformed half of both and
**not** the no-row half of either: a well-formed marker with no row yet is a pass. The reason is the
one above — the regeneration pass on `main` is the only writer of those reports, so a branch's new
marker necessarily has no row at the moment the branch is checked, and failing it would stall the
fan-out on every pull request that records a slower translation or a declared divergence. The
report-to-marker direction this task does implement is what keeps the reports honest in the
direction that matters, since it is the deletion of a declared divergence that would go unnoticed.
This task's `delivers` does not list R18 — that is step 13's — so the statement lives here and the
reviewer of plan step 13's tasks is pointed at it by `18-divergences-regeneration`.

**The regeneration pass.** Add a scheduled workflow of its own,
`.github/workflows/regenerate.yml`, triggered by `schedule` with the cron expression
`17 4 * * 1` — once a week, Monday 04:17 UTC, which is often enough for a handover report and rare
enough that it never races the fan-out — and by `workflow_dispatch`. It declares **both**
`permissions: contents: write` **and** `permissions: pull-requests: write`: the second is what the
pull-request fallback below needs, and with `contents: write` alone that fallback fails with a 403,
which is exactly the case it exists to cover.

The job checks out `main`, then, because GitHub runners carry neither and every script involved is a
`uv`-run PEP 723 executable, installs the toolchain by name the way the existing ubuntu job of
`.github/workflows/ci.yml` does — `astral-sh/setup-uv@v7` and `extractions/setup-just@v3` — and
initialises the OrcaSlicer submodule shallowly with
`git submodule update --init --depth 1 vendor/OrcaSlicer`, because the audits read the vendored
tree. Then it runs `just regenerate`, commits whatever moved, and runs `just audit-check` and
`just perf-check`, failing if either still reports a difference. Where branch protection refuses a
direct push it opens a pull request instead, so the pass never silently does nothing. It is the only
writer of `port/reports/perf-backlog.md`, `divergences.md`, `order-containers.md` and
`order-comparators.md`, and a port task's branch leaves all four alone.

The plan's "Files that change" list names `.github/workflows/ci.yml` and not this file, although
plan step 14 describes the pass it holds; `plan.md` is on `main` and is not edited from this branch,
so this task creates the file and says so here, and the reviewer of this pull request is not
surprised by a workflow the list does not carry.

It is its own workflow file and not a job in `.github/workflows/ci.yml` on purpose: that file is
the shared conflict surface the plan's Risks section names, edited by the tasks of plan steps 7,
10, 21, 23, 24, 25, 26 and 27, and step 14 is not one of them.

**What this task must not do.** It adds nothing that gates on throughput, instruction count or
allocation: a knowingly slower translation is a `// PERF(port):` marker and a backlog row and
nothing else, which is R15. It does not put `just audit-check` or `just perf-check` in
`just port-check` or in `.asdlc/checks`. It does not add `port/reports/perf-backlog.md` to the
paths `oracle/provenance --check` covers: the four static-analysis reports are grepped out of the
source with no oracle and no fixture, and task `05-oracle-provenance` excludes them by name. It
does not edit `docs/WORKFLOW.md`, which task `20-workflow-document` writes and which is where the
rule that a port task leaves those four reports alone is stated. It writes no cell of
`port/manifest.tsv`.

**What it waits on.** Task `21-port-status-markers` for the script and the marker whitelist its
assertion extends, and task `18-divergences-regeneration` for `port/regenerate`, `just regenerate`,
`just audit-check` and `port/reports/divergences.md`, whose `// FIDELITY(port):` rows the same
assertion covers. It does not wait on task 22: the two add to different parts of the same script.
No ported module exists yet when this lands, so the backlog regenerates to an empty report with
zero markers, and that is the correct result rather than a failure.

**Why this is a 5, and why it is not split.** The backlog generator and the per-marker assertion
alone would be a 3, but this task also adds a whole new scheduled workflow that commits to `main`
and falls back to opening a pull request when branch protection refuses the push — token
permissions, branch protection and `GITHUB_TOKEN` push behaviour are all dependencies outside this
repository, and none of them can be exercised until the workflow runs on `main`. The task's own
body says why that matters: the per-marker assertion and the regeneration pass together are the
mechanism that keeps a hundred-plus concurrent port branches from going red after every merge, so
the two directions above have to be implemented exactly as stated rather than as a convenient diff.
It is not split from the workflow because the workflow is what makes the report-to-marker direction
safe — without a pass that writes the rows, the assertion's "a marker with no row is not a failure"
clause would be permanent rather than transient — so the two halves have to be reviewed as one
mechanism.

## Done when

- `just regenerate` rewrites `port/reports/perf-backlog.md` from the `// PERF(port):` markers in
  the port tree and prints the marker count; running it twice leaves the committed file unchanged.
- Each row of the generated backlog names the module, the vendored source path the marker names,
  the marker's line and its reason.
- `just perf-check` exits 0 and prints nothing on a clean tree, and exits non-zero with a diff
  when `port/reports/perf-backlog.md` is edited by hand.
- `just port-status --per-commit` exits non-zero on a malformed `// PERF(port):` or
  `// FIDELITY(port):` marker, and names the marker's file and line.
- `just port-status --per-commit` exits non-zero when a row of the committed
  `port/reports/perf-backlog.md` or of `port/reports/divergences.md` names a marker that is no
  longer in the tree or whose text changed.
- `just port-status --per-commit` exits 0 on a branch that adds a well-formed `// PERF(port):`
  marker with no row in the committed backlog, and the branch's diff shows no change to any file
  under `port/reports/`.
- `just port-status --per-commit` runs no regeneration and no whole-file comparison of any report.
- `.github/workflows/regenerate.yml` runs on `schedule` with the cron `17 4 * * 1` and on
  `workflow_dispatch`, runs `just regenerate` on `main`, commits whatever moved, and then runs
  `just audit-check` and `just perf-check`.
- That workflow declares both `contents: write` and `pull-requests: write` in its `permissions`,
  which is what the pull-request fallback needs.
- That workflow installs `just` and `uv` — `extractions/setup-just@v3` and `astral-sh/setup-uv@v7` —
  and initialises the submodule shallowly, all before it runs any recipe.
- `just port-status --per-commit` asserts the marker rows of `port/reports/divergences.md` and no
  other row of it: a divergences row whose source cell is `oracle-parity` or `determinism`, with no
  marker anywhere in the tree, does not fail the command.
- `git diff` on the task's branch shows no change to `.github/workflows/ci.yml`, `.asdlc/checks`,
  `docs/WORKFLOW.md` or `port/manifest.tsv`.
- `grep -rn 'throughput\|instructions\|allocation' justfile .asdlc/checks .github/workflows/` finds
  no check that gates on any of them.
