# The porting workflow

`just port-status` reads the six headings below to check that this document says what it must
(R29); renaming one breaks that check. This document is committed before any unit is ported, and
every Phase A and Phase B task is built against it: a builder is given one task, this repository
and nothing else. `docs/PORTING.md` is the per-file translation contract this document refers to
by path and by section name; it is not quoted here.

## Unit of work

One libslic3r `.cpp` plus its same-stem `.hpp` becomes one Rust module — one manifest row each,
one unit between them — at the path `port/manifest.tsv`'s `target crate` and `target module`
columns name.

The exception is the template-heavy and header-only files, which are type-and-trait design rather
than file-units and are hand-designed once, before fan-out, by plan step 20. The set is the
in-scope rows whose `template-heavy` column is `yes`, taken from `port/manifest.tsv`; it is not
listed in this document, because a list here would drift from the column that is its source of
truth.

A porter reads the manifest row for a unit before starting it: the paths, the target crate, the
equality tier, the tolerance and comparison function, the proof path, and whether a fuzz target is
required. `docs/PORTING.md` is where the translation rules — the crate map, the type map, the
idiom map, the don't-translate list, the banned-substitution list and the six load-bearing rules —
live; this document states the process around them, not the rules themselves.

## The two phases

**Phase A** drafts every in-scope unit and **is not required to compile**. A draft is written at
its final path, `crates/<crate>/src/<module>.rs`, and **no `mod` declaration names it**. That is
what keeps `main` building: `cargo build` never reads it, so CI stays green with hundreds of
non-compiling drafts in the tree, and `cargo fmt --check` never sees one either.

**Phase B** makes the drafts compile, bottom-up by the crate-level level the cut defines, with
compiler errors as the work queue. Phase B's work on a module is to make it compile, add its `mod`
line to the crate's `lib.rs`, give it its proof at the tier its manifest row declares, and give it
its differential fuzz target where its row says `required`. `just port-status` counts a
trailer-carrying file that no `mod` line reaches as a module gated out of the build, which is the
burn-down that has to reach zero.

A draft is not exempt from the marker checks, and does not need to be: `just port-status
--per-commit` is text-based and reads a file that need not be valid Rust, and its constant rule
fires only on a constant the draft actually writes. Transcribe a numeric constant and it carries a
`// PORT SRC: <vendored path>:<line>` marker whose line really holds that literal; do not
transcribe it yet and leave `// TODO(port):` in its place instead. Neither choice fails the check.
A draft that invents a constant with no marker does — that is the case the check exists for. The
same holds for `// SAFETY:`, which applies to the `unsafe` blocks a draft writes: write the
comment on the block that exists, do not add an `unsafe` block with none.

A module may carry exactly six markers, and the list is closed:

- `// TODO(port):`
- `// PERF(port):`
- `// PORT NOTE:`
- `// SAFETY:`
- `// PORT SRC:`
- `// FIDELITY(port):`

Every module that ports a libslic3r file ends with a `PORT STATUS` trailer naming four things: its
source path, `confidence: high|medium|low`, its equality tier, and the oracle that proves it.

## The loop

One implementer, then two adversarial reviewers in separate contexts, then one fixer. Each reviewer
reads the diff **and** the vendored source file the module came from, is told to assume the module
is wrong, cites every divergence as `source:line` against `module:line`, and **may not implement**.

Both verdicts go in the unit's pull request body in that form, together with the module's manifest
row. `just pr-body-check <file>` fails a pull request without them, and the check fails closed — an
empty, absent or non-conforming body is a failure — and is skipped only for a pull request that
adds no module.

## The rejection rule

A workaround that needs a paragraph of justification is wrong code. A reviewer does not accept the
justification: instead, they re-read the vendored source and port what it does.

Where a Phase B draft turns out to be wrong, the source is re-read, not patched around. A batch
that does not `cargo check` is evidence that the crate cut or the batch boundary is wrong, not that
the module needs a stub — the answer is to re-cut it with `just task-cuts phase-b` after the cut is
corrected, never to stub the missing module out.

## Conflict discipline

Written for both forms, because the fan-out is a hundred-plus task branches built at once and not
one shared worktree.

**Agents sharing a worktree:** commit named files only, no `git stash` and no `git reset`, no build
commands, no slow commands.

**Concurrent branches:** a rule about files rather than commands. A port task writes only its own
modules — `crates/<crate>/src/<module>.rs` at the paths its rows name — plus the crate's `lib.rs`
`mod` line in Phase B, and its own test files. A port task may not write these seven paths; each is
produced by the repository instead, and writing it by hand is what the rule forbids:

| Path | Why a port task does not write it |
|---|---|
| `port/manifest.tsv` | Status is derived from the tree, so a row advances because a file was added, never because a cell was edited. |
| `port/reports/perf-backlog.md` | Written only by the regeneration pass that runs `just regenerate` on `main`. |
| `port/reports/divergences.md` | Written only by the regeneration pass that runs `just regenerate` on `main`. |
| `port/reports/order-containers.md` | Written only by the regeneration pass that runs `just regenerate` on `main`. |
| `port/reports/order-comparators.md` | Written only by the regeneration pass that runs `just regenerate` on `main`. |
| The root `Cargo.toml` | It is a `crates/*` glob, so a new crate needs no edit. |
| `Cargo.lock` | No port task adds a dependency. |

A task's branch changes nothing under `intent/`, and therefore passes `just guard` as it stands.

The `mod` line in `lib.rs` is the one shared file a port task does write. It is a one-line append
that git merges, and it goes in the crate's documented alphabetical position, so two branches
appending to the same `lib.rs` conflict textually rather than reordering silently.

A task passes `just port-check`, which includes `just port-status --per-commit` and its
per-marker assertion: every `// PERF(port):` and `// FIDELITY(port):` marker the branch adds is
well formed, and every marker already in the tree still has its row. A task does **not** pass
`just audit-check` or the perf-backlog whole-file diff — those belong to the regeneration pass,
which a hundred concurrent branches could not each pass.

The port's own order choices, which `docs/PORTING.md` carries as rules, are the reason the
`// FIDELITY(port):` marker exists at all: sum in a fixed order, use total-order comparators, and
use insertion-ordered maps wherever iteration order reaches output, so the port is deterministic
where the oracle is not. Each such site carries the marker that puts it in
`port/reports/divergences.md`.

## Trial-run record

Before any fan-out pull request is opened, `35-trial-run` runs a three-unit trial and records it
here: one unit of each equality tier — one fixed-point unit held to exact integer-coordinate
equality, one float unit held to its row's declared tolerance and comparison function, and one unit
with a non-const reference out-parameter and a `throw`. The record takes the same form a unit's
pull request uses: both review verdicts, each citing `source:line` against `module:line`, plus the
module's manifest row. The trial's output corrects `docs/PORTING.md` before fan-out.

This section is filled by `35-trial-run`.
