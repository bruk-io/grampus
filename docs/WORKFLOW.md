# The porting workflow

`just port-status` reads the six headings below to check that this document says what it must
(R29); renaming one breaks that check. `## Decisions that govern over a task's file list` is a
later addition and is not one of the six; adding a heading does not break the check, renaming one
of the six does. This document is committed before any unit is ported, and
every Phase A and Phase B task is built against it: a builder is given one task, this repository
and nothing else. `docs/PORTING.md` is the per-file translation contract this document refers to
by path and by section name; it is not quoted here.

## Unit of work

One libslic3r `.cpp` plus its same-stem `.hpp` becomes one Rust module — one manifest row each,
one unit between them — at the path `port/manifest.tsv`'s `target crate` and `target module`
columns name.

The exception is the template-heavy and header-only files, which are type-and-trait design rather
than file-units and are hand-designed once, before fan-out, by plan step 20, cut as
`34-foundation-spike`. The set is the in-scope rows whose `template-heavy` column is `yes`, taken
from `port/manifest.tsv`; it is not listed in this document, because a list here would drift from
the column that is its source of truth.

A porter reads the manifest row for a unit before starting it: the paths, the target crate, the
equality tier, the tolerance and comparison function, the proof path, and whether a fuzz target is
required. `docs/PORTING.md` is where the translation rules — the crate map, the type map, the
idiom map, the don't-translate list, the banned-substitution list and the six load-bearing rules —
live; this document states the process around them, not the rules themselves.

## The two phases

**Phase A** drafts every in-scope unit and **is not required to compile**. A draft is written at
its final path, `crates/<crate>/src/<module>.rs`, and **no `mod` declaration names it**. That is
what keeps `main` building: `cargo build` never reads it, so CI stays green with hundreds of
non-compiling drafts in the tree, and `cargo fmt --check` never sees one, because no `mod` line
reaches it.

**Phase B** makes the drafts compile, bottom-up by the crate-level level the cut defines, with
compiler errors as the work queue. Phase B's work on a module is to make it compile, add its `mod`
line to the crate's `lib.rs`, give it its proof at the tier its manifest row declares, and give it
its differential fuzz target where its row says `required`. `just port-status` counts a
trailer-carrying file that no `mod` line reaches as a module gated out of the build, which is the
burn-down that has to reach zero.

A draft is not exempt from the marker checks, and does not need to be: `just port-status --per-commit` is
text-based and reads a file that need not be valid Rust, and its constant rule fires only on a
constant the draft actually writes. Transcribe a numeric constant and it carries a
`// PORT SRC: <vendored path>:<line>` marker whose line really holds that literal; do not
transcribe it yet and leave `// TODO(port):` in its place instead. Neither choice fails the check.
A draft that invents a constant with no marker does — that is the case the check exists for. The
same holds for `// SAFETY:`, which applies to the `unsafe` blocks a draft writes.

Every marker a module carries is one of these six, and the list is closed:

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

A reviewer also checks, by hand, **every `// PORT SRC: external:` marker the unit adds**, against
the source it cites. `port/port-status` accepts that form without a vendored-line lookup —
there is no vendored line to look up — and counts and prints each one, with its text, for exactly
this purpose (`docs/PORTING.md`, "Decisions recorded on 2026-11-25", decision 2). Nothing else
checks it: an external marker a reviewer did not read is an unchecked claim about a standard or a
library, and a reviewer who cannot find what it cites rejects the unit rather than accepting the
marker's word for it.

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
that git merges: the crate's `mod` lines are kept in one block, in alphabetical order by module
name, and a new `mod <module>;` line is inserted at its alphabetical position in that block, so
two branches appending to the same `lib.rs` conflict textually rather than reordering silently.

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

## Decisions that govern over a task's file list

A port task's body enumerates exactly which files `git diff main` may touch, and a task file on
`main` is not edited after acceptance. `docs/PORTING.md`'s "Decisions recorded on 2026-11-25"
moved where a proof shim lives, what form a numeric constant's marker may take, and which units
are in scope at all, so some task bodies now enumerate files that no longer exist and omit files
a unit now needs.

**Where a port task's enumerated file list disagrees with the proof route, the marker rules or
the scope rule in `docs/PORTING.md` as they stand on `main`, `docs/PORTING.md` governs.** The
builder says so in the pull request body, naming the task's line and the rule that overrides it,
and a reviewer does not count that as a divergence. The task's own list still governs everything
the rules do not reach: a builder does not take the override as licence to write a file neither
the task nor `docs/PORTING.md` puts in its path.

Two corollaries, both measured on the first port attempts:

- A task whose body says to append to `crates/slic3r-ffi/cxx/proof_shims.cpp`, `proof_shims.h` or
  `src/proof.rs` creates `cxx/proof_units/<unit>.{h,cpp}` and `src/proof_units/<unit>.rs`
  instead. Those three shared files no longer exist.
- A task whose unit list names a file `port/manifest.tsv` now marks out of scope drops that unit,
  says so in the pull request body, and does not count the task's `ported` line against itself.
  `port/foundation.tsv` is regenerated from the manifest and so can also re-batch: where it and
  an accepted task body disagree about which units a task holds, **the task body governs its own
  scope** and the manifest governs what is in scope at all. No check reads
  `port/foundation.tsv`; re-cutting the tasks to match it is a separate pull request's work.

**Shared append points that remain.** After decision 1 a port task appends to exactly one shared
file: its crate's `src/lib.rs` `mod` line. That one is deliberately **not** given a
`merge=union` rule in `.gitattributes`, unlike the `justfile` and `.gitignore` entries there. A
union merge would keep both branches' lines but not at their alphabetical positions, and would
duplicate a line two branches added identically — producing a `lib.rs` that does not compile,
silently, in place of the textual conflict "Conflict discipline" above deliberately wants. No
other file a port task writes is shared, so no other union rule is needed.

## Trial-run record

Before any fan-out pull request is opened, plan step 21, cut as `35-trial-run`, runs a three-unit
trial and records it here: one unit of each equality tier — one fixed-point unit held to exact
integer-coordinate equality, one float unit held to its row's declared tolerance and comparison
function, and one unit with a non-const reference out-parameter and a `throw`. The record takes
the same form a unit's pull request uses: both review verdicts, each citing `source:line` against
`module:line`, plus the module's manifest row. The trial's output corrects `docs/PORTING.md`
before fan-out.

This section is filled by `35-trial-run`.

Dated 2026-11-26, before any fan-out pull request is opened.

**On the review loop, stated plainly rather than overclaimed.** This trial ran with one builder
agent and no tool to spawn a second, separate-context subagent for a reviewer role (recorded in
this task's own plan, "Not a risk, withdrawn" — later corrected again when the trial was actually
built: that withdrawal reasoned the orchestration around a task supplies the two reviewers, which
holds for an ordinary fan-out task but not for this trial, which had no such orchestration to
delegate to). Each unit's "both review verdicts" below are therefore two **sequential** adversarial
passes by that one agent — each a fresh re-read of the vendored source against the diff, from an
"assume the module is wrong" stance, before looking at the other pass's findings — not two
independently-run contexts. One genuine divergence was found and fixed this way (the float unit,
below); the other two units' second pass confirmed the first rather than finding anything further.
A later task with real separate-context reviewers available should not read this shape as the
loop's steady state.

### Unit 1 — fixed-point (near-miss): `MaterialType.cpp` / `MaterialType.hpp`

- **Manifest row:** MaterialType.cpp
- **Manifest row:** MaterialType.hpp
- **Reviewer 1:** `src/libslic3r/MaterialType.cpp:21` against
  `crates/slic3r-materialtype/src/material_type.rs:60` — the `ABS` row's seven numeric fields
  (`190, 300, 50, 65, 1, 0.1, 100`) transcribe exactly, covered by the `// PORT SRC:
  MaterialType.cpp:21` line directly above it; `src/libslic3r/MaterialType.cpp:108-120` against
  `crates/slic3r-materialtype/src/material_type.rs:216-223` — `get_temperature_range`'s
  always-write-the-default-then-overwrite-on-a-hit behaviour (`:110-111`, `:114-115`) is
  reproduced by the `match`'s two arms returning `(info.min_temp, info.max_temp, true)` /
  `(DEFAULT_MIN_TEMP, DEFAULT_MAX_TEMP, false)`, not merely the hit path. No divergence found.
- **Reviewer 2:** `src/libslic3r/MaterialType.cpp:101-106` against
  `crates/slic3r-materialtype/src/material_type.rs:212-214` — `find()`'s linear scan and
  case-sensitive, exact-string `name` comparison (`info.name == name`, `:104`) match
  `info.name == name` in `find()`'s Rust body exactly, including the not-found `nullptr`/`None`
  case; `src/libslic3r/MaterialType.cpp:31` (`FLEX`, `adhesion_coefficient` written as the bare
  integer `1` meaning `1.0`) against
  `crates/slic3r-materialtype/src/material_type.rs:80` — the `1f64` suffix form normalises to
  the exempt literal `1` under `port/port-status`'s own rule while still typing as `f64`,
  checked against every other whole-number float field in the table, not only this row. No
  divergence found.
- **Fixer:** no fix needed; both passes agreed.

### Unit 2 — float: `GCode/PchipInterpolatorHelper.cpp` / `GCode/PchipInterpolatorHelper.hpp`

- **Manifest row:** GCode/PchipInterpolatorHelper.cpp
- **Manifest row:** GCode/PchipInterpolatorHelper.hpp
- **Reviewer 1:** `src/libslic3r/GCode/PchipInterpolatorHelper.cpp:59-81` against
  `crates/slic3r-gcode/src/pchip_interpolator_helper.rs:107-130` — `computePCHIP`'s derivative
  recurrence (`d_[i] = (w1 + w2) / (w1 / delta_[i-1] + w2 / delta_[i])`, `:76`, matching
  `pchip_interpolator_helper.rs:126`) transcribes with the same operator precedence and operand
  order; `src/libslic3r/GCode/PchipInterpolatorHelper.cpp:86-104` against
  `crates/slic3r-gcode/src/pchip_interpolator_helper.rs:135-155` — `interpolate`'s
  `std::lower_bound` (`:90`) against `Vec::partition_point(|&v| v < xi)` (`:145`): both return
  the first index whose element is not less than `xi` on a sorted ascending slice, so `i = that
  index - 1` agrees on both sides, including the exactly-on-a-knot queries this unit's own test
  exercises (`3.0`, `4.25`, `6.0`, `8.0`, `10.0`). No divergence found in either cited range.
- **Reviewer 2 (found a divergence):** `src/libslic3r/GCode/PchipInterpolatorHelper.cpp:43-54`
  against `crates/slic3r-gcode/src/pchip_interpolator_helper.rs:85-96` — the first pass's own
  module doc comment, and this row's first `tolerance` reason text, both claimed `sortData()`'s
  `std::sort -> slice::sort_unstable` idiom mapping leaves a divergence when two input points
  share an x-coordinate. Checked against the vendored line: `std::sort` on
  `std::vector<std::pair<double,double>>` (`:44-48`) uses `std::pair`'s default `operator<`,
  which compares **both** fields in order — the sort key is `(x, y)`, not `x` alone — so two
  elements compare equal only when `x` *and* `y` both match, and swapping genuinely-equal
  elements is not observable. There is nothing here for an unstable sort to diverge from a
  stable one on, with or without a duplicate `x`. The claim was wrong.
- **Fixer:** corrected `sort_data`'s doc comment
  (`crates/slic3r-gcode/src/pchip_interpolator_helper.rs:85-96`) and the test's own comment
  (`:170`) to state the real reasoning above, and rewrote this row's `tolerance` reason text
  in `port/manifest.tsv` to the same effect (the measured bound stands — `0`, over this proof's
  own distinct-`x` input — the withdrawn claim was about *why* a tolerance row rather than an
  exact one, not about the measured number itself). `cargo test -p slic3r-gcode pchip` re-run
  clean after the fix (both rows still pass at `tier=float`).

### Unit 3 — throw / non-const-ref-out-param: `InstanceLock.cpp` / `InstanceLock.hpp`

- **Manifest row:** InstanceLock.cpp
- **Manifest row:** InstanceLock.hpp
- **Reviewer 1:** `src/libslic3r/InstanceLock.cpp:122-148` against
  `crates/slic3r-instancelock/src/instance_lock.rs:212-264` — the constructor's `if (depth==0 &&
  now>=cooldown_until && open_lock_file(...))` guard, the retry loop's `try_lock`/deadline/sleep
  sequence, and `++depth` *before* `m_locked = file_lock != nullptr` (`:146-148`, matching
  `instance_lock.rs:263-264`) all appear in the same order in the port;
  `src/libslic3r/InstanceLock.cpp:150-161` against
  `crates/slic3r-instancelock/src/instance_lock.rs:277-291` — the destructor's own body runs
  before its member destructors release the mutex in C++ (reverse declaration order), which the
  port reproduces by running `Drop::drop`'s explicit body on the taken `guard` before that local
  `guard` goes out of scope and releases the reentrant lock itself. No divergence found.
- **Reviewer 2:** `src/libslic3r/InstanceLock.cpp:37-54` against
  `crates/slic3r-instancelock/src/instance_lock.rs:69-99` — `NativeFileLock`'s constructor
  (`O_RDWR|O_CREAT|O_CLOEXEC`, mode `0644`, throwing `std::system_error` on failure) and
  `try_lock()`'s `EWOULDBLOCK`/`EINTR` → "keep trying", anything else → the unit's other throw,
  against `OpenOptions`'s equivalent flags/mode and `io::ErrorKind::WouldBlock`/`Interrupted` →
  `Ok(false)`, anything else → `Err`; `src/libslic3r/InstanceLock.hpp:52-53` (the `private:`
  non-const-reference out-params this leg is chosen for) against
  `crates/slic3r-instancelock/src/instance_lock.rs:165-186` — `open_lock_file`/`defer` take `Slot
  &slot` in the vendored source and `&mut SlotState` here, both mutating shared state the caller
  already holds locked, matching the idiom map's "a non-const reference out-parameter -> a
  return value" by returning `bool`/`()` rather than adopting a second `&mut` parameter to mirror
  the C++ signature. No divergence found. Noted, not a finding: this unit's shim can drive only
  the public surface (`InstanceLock`'s constructor, destructor, `locked()`) — `open_lock_file`/
  `defer` are `private:`, so the out-params and the caught throw are exercised transitively, and
  the record above says so rather than claiming a direct comparison that was not made.
- **Fixer:** no fix needed; both passes agreed.

## What the two review passes did not catch

Recorded for the same reason the review methodology note above is: `just port-check` (R16's
`// PORT SRC:` marker checks, R19's proof-marker-differential check), run after both review
passes above had already signed off on all three units, found 545 findings across all three --
markers missing the `src/libslic3r/` path prefix `resolve_vendored` requires, float literals
written in a decimal form that does not textually match the vendored line's bare-integer
spelling, two symbolic POSIX flag values and one octal-notation mismatch needing the external
marker form instead, and both `MaterialType`'s and `InstanceLock`'s proofs reading as
non-differential to the text scanner because their `use ... as oracle;` import is invisible to
it. None of this was a *behavioural* divergence the "assume the module is wrong, cite
`source:line` against `module:line`" review loop is built to find — every value both review
passes checked was already correct — it was a mechanical-convention gap neither review pass
checked for, because neither read `port/port-status`'s own source before reviewing. Fixed in this
branch's "Fix `// PORT SRC:` markers and proof-marker detection" commit, with `just port-check`
confirming 0 findings and `cargo test` confirming every proof still passes. A later task's
reviewers should run `just port-check` as part of their own review, not only `cargo test`, given
this measured gap.

## Corrections to `docs/PORTING.md`

None. Every rule this trial actually exercised — the idiom map's "non-const reference
out-parameter -> a return value" (units 1 and 3), the error-set rule (units 2 and 3), the
`std::sort`/`slice::sort_unstable` idiom entry (unit 2, where the adversarial-review finding above
was about this task's own draft misreading that entry, not about the entry itself) — held exactly
as stated. What this trial *did* find is two mismatches between a row's seeded `proof path` cell
and the mechanism that cell names (`MaterialType`'s rows name proof path 3, the
include-the-vendored-`.cpp` mechanism, but `MaterialType`'s methods are externally linked, not
internal-linkage, so including the `.cpp` whole would duplicate-define them against the archive;
`GCode/PchipInterpolatorHelper.hpp`'s row names proof path 2, but the dump corpus it is seeded
from carries no field this unit produces) — both handled the same way
`crates/slic3r-commondefs/src/common_defs.rs`'s own `// PROOF: 2` finding already established
(keep the cell's marker literal, prove the row by the mechanism that actually works, record the
mismatch in the module and the pull request) before this trial ran. That precedent, not a new
`docs/PORTING.md` rule, is what already governs a seeded-proof-path mismatch; this trial adds two
more measured instances of it rather than a reason to change the document.
