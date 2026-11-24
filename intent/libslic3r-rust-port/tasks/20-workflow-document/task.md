---
kind: task
step: 15
delivers: [R12, R13]
depends: [14-manifest-columns, 17-order-audits]
size: 3
---
# Task: Write the porting workflow document

## What to build

Plan step 15 writes two documents and is split in two. This is **part 2: `docs/WORKFLOW.md`**,
the porting workflow (R13). Part 1 is `19-porting-contract`, which writes `docs/PORTING.md`; the
frontmatter's `delivers` is the whole step's, so do not write `docs/PORTING.md` here and do not
edit it. The two tasks may run at the same time, and neither is a dependency of the other: this
document references `docs/PORTING.md` by path and by section name, never by quoting it.

Create `docs/WORKFLOW.md` and nothing else. Write no code: the command that checks this
document's sections (`just port-status`, R29) is written by `22-port-status-gates`, the
pull-request-body check is `36-pr-body-check`, and the three-unit trial that fills this document's
trial-run section is `35-trial-run`.

This document is what every Phase A and Phase B task is built against, and a builder is given one
task, this repository and nothing else. Its sections are enumerated in plan step 15, so the work
is transcription and precision rather than discovery.

### What it must contain, and under which headings

`22-port-status-gates` writes a check that reads this document's headings, so the headings are a
contract between the two tasks. Each one below is spelled as R13 and R29 name that section, so a
check written from the requirement's own words finds it. That task's builder reads the headings
from `docs/WORKFLOW.md` as it stands on `main`, not from a list typed into its own body, so if a
heading here and the check ever disagree, this document is the authority. Use exactly these, and
say in the opening paragraph that `just port-status` reads them and that renaming one breaks the
check:

- `## Unit of work`
- `## The two phases`
- `## The loop`
- `## The rejection rule`
- `## Conflict discipline`
- `## Trial-run record`

**Unit of work.** One libslic3r `.cpp` plus its same-stem `.hpp` becomes one Rust module — one
manifest row each, one unit between them — at the path the manifest's target crate and module
columns name. The exception is the template-heavy and header-only files, which are
type-and-trait design rather than file-units and are hand-designed once, before fan-out, by plan
step 20; the set is the in-scope rows whose `template-heavy` column is `yes`, taken from
`port/manifest.tsv` and not listed in this document. Say that a porter reads the row for the
paths, the crate, the equality tier, the tolerance and comparison function, the proof path and
whether a fuzz target is required, and that `docs/PORTING.md` is where the translation rules live.

**The two phases.** Phase A drafts every in-scope unit and **is not required to compile**; Phase B
makes the drafts compile, bottom-up by the crate-level level the cut defines, with compiler errors
as the work queue. State where a draft lives and why, because that is what keeps `main` building:
a draft is written at its final path, `crates/<crate>/src/<module>.rs`, and **no `mod`
declaration names it**, so `cargo build` never reads it, CI stays green with hundreds of
non-compiling drafts in the tree, and `cargo fmt --check` never sees one. Phase B's work on a
module is to make it compile, add its `mod` line to the crate's `lib.rs`, give it its proof at the
tier its row declares and its differential fuzz target where its row says `required`.
`just port-status` counts a trailer-carrying file that no `mod` line reaches as a module gated out
of the build, which is the burn-down that has to reach zero.

This section also carries, as its own paragraph, the rule that **a draft is not exempt from the
marker checks and does not need to be**: `just port-status --per-commit` is text-based and reads a
file that need not be valid Rust, and its constant rule fires only on a constant the draft
actually writes. So transcribe a numeric constant and it carries a
`// PORT SRC: <vendored path>:<line>` marker whose line really holds that literal; do not
transcribe it and leave `// TODO(port):` in its place. Neither choice fails the check; a draft
that invents a constant with no marker does, which is the case the check exists for. `// SAFETY:`
applies the same way, to the `unsafe` blocks the draft writes. Name the six markers a module may
carry — `// TODO(port):`, `// PERF(port):`, `// PORT NOTE:`, `// SAFETY:`, `// PORT SRC:` and
`// FIDELITY(port):` — and say the list is closed, and name the `PORT STATUS` trailer every
module ends with: its source path, `confidence: high|medium|low`, its equality tier and the oracle
that proves it.

**The loop.** One implementer, then two adversarial reviewers in separate contexts, then one
fixer. Each reviewer reads the diff **and** the vendored source file the module came from, is told
to assume the module is wrong, cites every divergence as `source:line` against `module:line`, and
**may not implement**. Say that both verdicts go in the unit's pull request body in that form,
together with the module's manifest row, that `just pr-body-check <file>` fails a pull request
without them, and that the check fails closed — an empty, absent or non-conforming body is a
failure — and is skipped only for a pull request that adds no module.

**The rejection rule.** A workaround that needs a paragraph of justification is wrong code. Say
what a reviewer does instead: re-read the vendored source and port what it does. Where a Phase B
draft turns out to be wrong, the source is re-read, not patched around, and a batch that does not
`cargo check` is evidence that the crate cut or the batch boundary is wrong — the answer is to
re-cut it with `just task-cuts phase-b` after the cut is corrected, never to stub the missing
module out.

**Conflict discipline, in both forms**, because the fan-out is a hundred-plus task branches built
at once and not one shared worktree. Write both:

- *Agents sharing a worktree:* commit named files only, no `git stash` and no `git reset`, no
  build commands, no slow commands.
- *Concurrent branches:* a rule about files rather than commands. A port task writes only its own
  modules — `crates/<crate>/src/<module>.rs` at the paths its rows name — plus the crate's
  `lib.rs` `mod` line in Phase B and its own test files. List the **seven paths it may not
  write**, each with the reason the repository produces it instead: `port/manifest.tsv` (status is
  derived from the tree, so a row advances because a file was added and not because a cell was
  edited), `port/reports/perf-backlog.md`, `port/reports/divergences.md`,
  `port/reports/order-containers.md` and `port/reports/order-comparators.md` (all four written
  only by the regeneration pass that runs `just regenerate` on `main`), the root `Cargo.toml` (a
  `crates/*` glob, so a new crate needs no edit) and `Cargo.lock` (no port task adds a
  dependency). Say that a task's branch changes nothing under `intent/` and therefore passes
  `just guard` as it stands.

Add that the `mod` line in `lib.rs` is the one shared file a port task does write, that it is a
one-line append git merges, and that it goes in the crate's documented alphabetical position so
two branches appending to the same `lib.rs` conflict textually rather than reordering silently.

Say in the same section what a task passes and what it does not: `just port-check`, which
includes `just port-status --per-commit` and its per-marker assertion — every `// PERF(port):`
and `// FIDELITY(port):` marker the branch adds is well formed and every marker already in the
tree still has its row — and **not** `just audit-check` or the perf-backlog whole-file diff,
which are the regeneration pass's and which a hundred concurrent branches could not each pass. The
port's own order choices, which `docs/PORTING.md` carries as rules, are stated here as the reason
the `// FIDELITY(port):` marker exists at all: sum in a fixed order, use total-order comparators,
and use insertion-ordered maps wherever iteration order reaches output, so the port is
deterministic where the oracle is not, and each such site carries the marker that puts it in
`port/reports/divergences.md`.

**Trial-run record.** The section the three-unit trial is recorded in, before any fan-out pull request is
opened. Write the heading, say that plan step 21 covers one unit of each equality tier — one
fixed-point unit held to exact integer-coordinate equality, one float unit held to its row's
declared tolerance and comparison function, and one unit with a non-const reference out-parameter
and a `throw` — that the record takes the same form a unit's pull request uses, both review
verdicts citing `source:line` against `module:line` plus the module's manifest row, and that the
trial's output corrects `docs/PORTING.md` before fan-out. Leave the record itself to
`35-trial-run`: the heading and that statement are what this task delivers, and the section says
in one line that it is filled by that task.

### What this task waits on, and what it must not touch

It waits on `14-manifest-columns` for the manifest's column semantics — which columns are derived,
which are judgements carried forward, and that status is derived from the tree and never edited,
which is what the conflict discipline rests on — and on `17-order-audits` for
`port/reports/order-containers.md` and `port/reports/order-comparators.md`, which are two of the
four paths the seven-path list names and which the `// FIDELITY(port):` rule is written against.

Touch no file but `docs/WORKFLOW.md`. Write nothing in `port/`, nothing in `crates/`, no
`justfile` recipe, no line in `.asdlc/checks` and no line in `.github/workflows/ci.yml`:
`24-port-check-ci` owns the checks and the CI jobs, `21-port-status-markers` and
`22-port-status-gates` own `port/port-status`, and `36-pr-body-check` owns the pull-request-body
mode. Name a recipe in the document only by the spelling the plan fixes —
`just port-check`, `just port-status`, `just port-status --per-commit`,
`just pr-body-check <file>`, `just regenerate`, `just audit-check`, `just task-cuts phase-b` —
and do not invent another.

## Done when

- `docs/WORKFLOW.md` exists and holds the headings `## Unit of work`, `## The two phases`,
  `## The loop`, `## The rejection rule`, `## Conflict discipline` and `## Trial-run record`.
- The opening paragraph says that `just port-status` reads those headings and that renaming one
  breaks the check.
- The unit-of-work section states the one `.cpp` plus same-stem `.hpp` to one module rule, names
  the manifest's target crate and module columns as where the path comes from, and names the
  `template-heavy` column as the exception hand-designed before fan-out, without listing the
  files.
- The two-phases section states that a Phase A draft need not compile, that it lives at its final
  path with no `mod` declaration naming it, why that keeps `main` building and `cargo fmt` away
  from it, and what Phase B does to a module.
- It carries the paragraph on drafts and the marker checks, including the two allowed choices for
  a numeric constant and the case that fails, and it names all six markers and the `PORT STATUS`
  trailer's four items.
- The loop section states one implementer, two adversarial reviewers in separate contexts who each
  read the diff and the vendored source, that they assume the module is wrong, cite `source:line`
  against `module:line` and may not implement, and one fixer; and it states that both verdicts and
  the manifest row go in the pull request body and that the check fails closed.
- The rejection rule is stated, with what is done instead of a workaround, including re-cutting a
  Phase B batch that does not `cargo check`.
- The conflict-discipline section states both forms, and lists exactly seven paths a port task may
  not write — `port/manifest.tsv`, `port/reports/perf-backlog.md`,
  `port/reports/divergences.md`, `port/reports/order-containers.md`,
  `port/reports/order-comparators.md`, the root `Cargo.toml` and `Cargo.lock` — each with the
  reason the repository produces it instead.
- It states the `mod`-line rule, including the alphabetical position, and which checks a task
  passes and which belong to the regeneration pass.
- The trial-run section exists, says what the three units are chosen for, in what form the trial is
  recorded, that the trial corrects `docs/PORTING.md` before fan-out, and that `35-trial-run`
  fills the record.
- Every recipe the document names is one the plan fixes, spelled as the plan spells it.
- The pull request changes `docs/WORKFLOW.md` and no other file.
