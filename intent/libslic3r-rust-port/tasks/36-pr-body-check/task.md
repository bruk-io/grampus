---
kind: task
step: 21
delivers: [R28]
depends: [20-workflow-document, 21-port-status-markers, 24-port-check-ci]
size: 3
---
# Task: Check that a port unit's pull request records both verdicts

## What to build

This is part 2 of plan step 21, "The three-unit trial run": the `--pr-body` mode and the CI step
that runs it. Part 1 is task `35-trial-run`, which runs the trial and corrects the documents;
**this task ports no unit and edits neither `docs/PORTING.md` nor `docs/WORKFLOW.md`**.
`delivers: [R28]` is the whole step's.

### The third mode of `port/port-status`

Add a third mode to `port/port-status`, the script task `21-port-status-markers` creates:

```
port/port-status --pr-body <file> [--base <ref>]
```

and wire it to its own recipe, `just pr-body-check <file>`. **That one spelling is the only one**:
it is what this task's CI step runs and what the plan's proof for R28 names. `just port-status`
forwards nothing to it — that recipe runs the per-commit subset and the completion gate and takes
no file.

`<file>` holds a pull request body. The mode fails unless the body:

- records **both** review verdicts from the loop `docs/WORKFLOW.md` states — the two adversarial
  reviewers in separate contexts — each citing at least one divergence or finding as
  `source:line` against `module:line`; and
- names a manifest row that exists, meaning a path in the body's recorded row matches the `path`
  column of a row of `port/manifest.tsv`.

On success it prints both recorded verdicts and the manifest row it found, and exits 0.

**The form is read from `docs/WORKFLOW.md`, not invented here.** That document states the loop
and carries the trial-run record in the same form a unit's pull request uses, so take the shape
the check matches from it. Task `35-trial-run` may correct that form; this task does not wait on
it, so whichever of the two lands second reconciles the pattern with the recorded form, and
neither is written in terms of the other's text.

Every check in this script is **text-based**: it reads the body and the manifest as lines and
parses no Rust, for the same reason the rest of `port/port-status` does — it has to run over
Phase A drafts that are not valid Rust.

### It fails closed

An empty body, an absent file, a body with one verdict, a body with no manifest row, and a body
that does not conform are each a failure. That is the one case the check exists to catch, so
there is no benefit-of-the-doubt path and no allowlist.

### It is skipped only from the diff, never from the body

The check is skipped for a pull request that adds no module, and **it decides that from the diff**
— a body can never switch the check off. `--base <ref>` names the base to diff against, defaulting
to `origin/main`; the script takes the changed path list from
`git diff --name-only <base>...HEAD` and treats the pull request as adding a module when that list
holds a file under `crates/*/src/` that carries a `PORT STATUS` trailer, or a `crates/*/src/lib.rs`
whose diff adds a `mod` line. Both forms matter: a Phase A pull request adds the module file, and a
Phase B pull request changes an existing draft and adds its `mod` line. When the diff holds
neither, the mode prints that it is skipped and exits 0.

That rule is the one judgement in this task, so state it in the pull request body for a reviewer
to check: a port unit's pull request must not be able to reach the skip path.

### The CI step

It reads the pull request, so it cannot live in `.asdlc/checks`, which a task also runs locally —
leave that file alone. It needs no oracle binary and no libslic3r build, so it runs in the
**ubuntu job** of `.github/workflows/ci.yml`, not the macOS arm64 job. Add, to that job:

- a step that runs only on a pull request, writes the body to a file with
  `gh pr view "$PR" --json body --jq .body`, and then runs `just pr-body-check <that file>`, with
  `PR` taken from `${{ github.event.pull_request.number }}` in that step's environment;
- `GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}` in that step's environment, since `gh` reads it;
- `pull-requests: read` in the workflow's `permissions`, which today holds `contents: read` only.

The ubuntu job already checks out with `fetch-depth: 0`, so the base ref the script diffs against
is present. Add one step and one permission line and nothing else to the workflow; the toolchain
step, the shallow submodule initialisation and the `just port-check` step belong to task
`24-port-check-ci`, and that task is the one whose shape this step is added beside.

### Why this is a 3

A third mode on a script that already exists, plus one CI step and one permissions line. The care
is in the two rules above: it fails closed, and it is skipped only from the diff and never from
the body.

## Done when

- `port/port-status --pr-body <file>` exists and `just pr-body-check <file>` is the recipe that
  runs it; `grep -n 'pr-body' justfile .github/workflows/ci.yml` shows that one spelling in both.
- For a branch whose diff **adds a module** — a `crates/*/src/<module>.rs` with a `PORT STATUS`
  trailer, added as a scratch commit the builder makes for the demonstration and does not keep —
  `just pr-body-check <file>` on a body recording both verdicts with `source:line` against
  `module:line` citations and an existing manifest row exits 0 and prints both verdicts and the
  row it found. The qualifier matters: this branch's own diff adds no module, so without it the run
  prints that it is skipped and prints neither verdict.
- It exits non-zero, for a pull request whose diff adds a module, on each of: a body with one
  verdict, a body with no manifest row, a body naming a manifest row that does not exist in
  `port/manifest.tsv`, an empty body, and a `<file>` that does not exist.
- It prints that it is skipped and exits 0 for a pull request whose diff adds no module, and the
  decision comes from `git diff --name-only <base>...HEAD` — demonstrated by a body that would
  fail the check passing while the diff adds no module, and failing once the diff adds one.
- `just port-status` with no argument neither reads a body file nor forwards to this mode.
- `.asdlc/checks` is unchanged by this task.
- `.github/workflows/ci.yml`'s ubuntu job holds a step that runs `gh pr view "$PR" --json body
  --jq .body` and then `just pr-body-check`, gated on `github.event_name == 'pull_request'`, with
  `GH_TOKEN` set from `secrets.GITHUB_TOKEN` and `PR` from
  `github.event.pull_request.number`; the workflow's `permissions` holds
  `pull-requests: read` beside `contents: read`; and the macOS arm64 job gained no line.
- `just check`, `just guard` and `just port-check` pass.
- `git diff` shows no change to `docs/PORTING.md`, `docs/WORKFLOW.md`, `port/manifest.tsv`, any
  file under `port/reports/`, or any file under `crates/`.
- Nothing under `vendor/OrcaSlicer/` changed and `git submodule status` still shows
  `f92a7c065c5718b8ef1baac1cd704df2911238f4`.
