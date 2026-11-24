---
kind: task
step: 12
delivers: [R30]
depends: [44-ownership-class-root, 45-ownership-class-arachne, 46-ownership-class-sla, 47-ownership-class-geometry, 48-ownership-class-fill, 49-ownership-class-support, 50-ownership-class-gcode]
size: 2
---
# Task: Make a blank ownership class a failure

## What to build

This is the **last** of the eight tasks cut from step 12 of `intent/libslic3r-rust-port/plan.md`,
"The ownership manifest", and a piece of a split step: its `delivers: [R30]` is the whole step's,
not this task's alone. The step's first task, cut in an earlier pass and not one of these eight, was
the spike `16-ownership-spike`, which committed `port/ownership.tsv` with a row per raw-pointer
field and every `class` cell blank; the seven tasks between it and this one
(`44-ownership-class-root` through `50-ownership-class-gcode`, one per row of
`port/ownership-batches.tsv`) fill those cells in. The plan names this task directly:

> A row whose `class` cell is still blank is allowed while this step's own batch tasks are open —
> the spike commits a row per field before any of them is classified — and **this step is done when
> no in-scope row's class is blank**, which is the one line a reviewer answers yes or no to. This
> step's last task removes that allowance from `port/ownership`, so a blank class is a failure from
> then on and no later work can reopen one.

**What to change, in `port/ownership` and nowhere else.** Two places hold the allowance:

- `VALID_CLASSES`, which today is `{"", "owned", "shared", "borrowed", "back-reference", "ffi",
  "unknown"}`. Drop the empty string, so `validate_rows` reports a blank `class` under its first
  rule — the one that prints `class <value> is not one of [...]`. Note that the message already
  computes its list as `sorted(VALID_CLASSES - {""})`, so that subtraction becomes redundant once
  the empty string is gone; leave the printed list the same six values either way.
- `validate_rows`'s last branch, `elif cls != "" and not evidence`, whose `cls != ""` guard exists
  only to let a blank row through without evidence. With a blank class now invalid, that guard goes,
  and a row's missing evidence is reported for every non-`unknown` class.

Make the failure message say what a reviewer needs: which row it is (the function already builds a
`header path:line declaring type::field` label) and that a blank class is no longer accepted.

**Update the two places that document the allowance**, so the file does not describe a rule it no
longer has: `port/ownership`'s module docstring, which today says a field seen for the first time
"gets a blank `class`, which is accepted while step 12's classification tasks are open", and the
comment on `VALID_CLASSES` if it carries one. Say instead that `just ownership` still writes a blank
`class` for a field it has not seen before — which is what `generate()` does, and must keep doing —
but that `just ownership-check` now fails such a row, so whoever moves the submodule pin and picks
up a new raw-pointer field classifies it in the same pull request.

**Do not change `generate()`.** It recomputes the first five columns from the vendored headers and
carries `class`, `evidence` and `rust type` forward by key, writing blanks for a field seen for the
first time. That behaviour stays: the gate belongs in the check, not in the writer, or
`just ownership` could no longer regenerate the file after a pin bump at all.

### What it waits for

All seven batch tasks, which is what its `depends` says, and for a real reason rather than a shared
file: `just ownership-check` validates every row in the file, so the moment the blank class is
invalid the recipe fails until all 208 rows carry one. This task cannot pass its own checks before
the seven are on `main`. Rebase on `main` and confirm `just ownership-check` exits 0 **before**
making the change, so a failure afterwards is the gate and not a missing batch.

### What this task does not do

- It fills in no `class`, `evidence` or `rust type` cell, and edits no row of `port/ownership.tsv`.
  If a row is still blank when this task is built, the fix is in that row's own batch task, not
  here.
- It adds no line to `.asdlc/checks` and none to `just port-check`; task `24-port-check-ci` is what
  puts `just ownership-check` in the checks a task passes, and there is no `just port-check` recipe
  yet.
- It does not touch `port/port-status`, which prints the count of `unknown` rows (tasks
  `21-port-status-markers` and `22-port-status-gates`). `unknown` stays a valid class: this task
  removes the blank-class allowance only, not the `unknown` one.
- It does not change the `justfile`, `port/manifest.tsv`, `port/ownership-batches.tsv`,
  `port/cut-tasks`, `port/inventory`, anything under `docs/`, anything under `intent/`, or anything
  under `vendor/OrcaSlicer/` — the submodule pointer stays at
  `f92a7c065c5718b8ef1baac1cd704df2911238f4`.

### What makes this a 2

A small, well-understood change in one file with no open questions: one value out of a set, one
guard out of a branch, and the docstring that described them. The work beyond the edit is the
proof — showing the recipe fails on a blanked class and still passes on the committed tree.

## Done when

- `port/ownership`'s `VALID_CLASSES` no longer holds the empty string, and `validate_rows` no longer
  has a branch that exempts a blank `class` from needing evidence.
- `just ownership-check` exits 0 against the committed tree and prints that `port/ownership.tsv`
  holds 208 rows.
- Blanking the `class` cell of any one row of `port/ownership.tsv` makes `just ownership-check` exit
  non-zero and name that row by header path, line, declaring type and field; the pull request shows
  that failure, and the blanked cell is not committed.
- No in-scope row of `port/ownership.tsv` has a blank `class` cell, and the pull request states that
  all 208 rows carry one of `owned`, `shared`, `borrowed`, `back-reference`, `ffi` or `unknown` —
  the one line that makes step 12 done.
- `unknown` is still a valid class, and `just ownership-check` still fails an `unknown` row that is
  missing either its evidence `path:line` or its conservative Rust type; the pull request shows both
  of those failures still happening.
- `just ownership` still rewrites `port/ownership.tsv` and leaves it byte-identical, and
  `generate()` still writes a blank `class` for a field it has not seen before.
- `port/ownership`'s module docstring no longer says a blank class is accepted, and says instead
  that `just ownership` writes one for a newly seen field while `just ownership-check` fails it.
- `port/ownership.tsv` is unchanged by this task's commits: `git diff main` on the branch touches
  `port/ownership` only.
- `.asdlc/checks`, the `justfile`, `port/port-status`, `port/manifest.tsv`,
  `port/ownership-batches.tsv`, `port/cut-tasks`, `port/inventory` and everything under `docs/` are
  unchanged.
- Nothing under `vendor/OrcaSlicer/` changed and the submodule pointer is still
  `f92a7c065c5718b8ef1baac1cd704df2911238f4`.
- The branch changes no file under `intent/`, so `just guard` passes, and `just check` passes.
