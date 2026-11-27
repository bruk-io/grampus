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

The marker has one further use, added by `docs/PORTING.md`'s "Decisions recorded on 2026-11-27",
decision 1: a site where the vendored code has undefined behaviour on some input and the port
substitutes a defined result. Decision 2 of the same section says what any marker's stated reason
must rest on, and what a reviewer who proposes one must supply.

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

**Addendum, 2026-11-26 (PR #55):** two reviewers running in separate contexts did examine this
pull request, independently of each other and of the pass above, before it was marked ready --
the "second review pass" entries under each unit below are that pass, recorded as each reviewer
actually gave it (including outright rejections), with what was fixed and in which commit noted
underneath. This is the real separate-context review the paragraph above says a later task
should have; it landed on this task's own pull request rather than on a fan-out one, because
this pull request was still open when it ran.

### Unit 1 — fixed-point (near-miss): `MaterialType.cpp` / `MaterialType.hpp`

- **Manifest row:** MaterialType.cpp
- **Manifest row:** MaterialType.hpp
- **Reviewer 1:** `src/libslic3r/MaterialType.cpp:21` against
  `crates/slic3r-materialtype/src/material_type.rs:68` — the `ABS` row's seven numeric fields
  (`190, 300, 50, 65, 1, 0.1, 100`) transcribe exactly, covered by the `// PORT SRC:
  MaterialType.cpp:21` line directly above it; `src/libslic3r/MaterialType.cpp:108-120` against
  `crates/slic3r-materialtype/src/material_type.rs:235-240` — `get_temperature_range`'s
  always-write-the-default-then-overwrite-on-a-hit behaviour (`:110-111`, `:114-115`) is
  reproduced by the `match`'s two arms returning `(info.1, info.2, true)` /
  `(DEFAULT_MIN_TEMP, DEFAULT_MAX_TEMP, false)`, not merely the hit path. No divergence found.
- **Reviewer 2:** `src/libslic3r/MaterialType.cpp:101-106` against
  `crates/slic3r-materialtype/src/material_type.rs:228-230` — `find()`'s linear scan and
  case-sensitive, exact-string `name` comparison (`info.name == name`, `:104`) match
  `info.0 == name` in `find()`'s Rust body exactly (tuple index `0` is the vendored struct's
  `name` field, kept a tuple struct so a 75-row table stays one row per line under `cargo fmt`
  -- see this module's own doc comment on `MaterialTypeInfo`), including the not-found
  `nullptr`/`None` case; `src/libslic3r/MaterialType.cpp:31` (`FLEX`, whose `thermal_length`
  field is written as the bare integer `1000` meaning `1000.0` -- its `adhesion_coefficient`,
  `0.5`, is genuinely fractional, not an instance of this, and the record's earlier claim that
  this row's `adhesion_coefficient` is "the bare integer `1`" was simply wrong) against
  `crates/slic3r-materialtype/src/material_type.rs:88` — the `1000f64` suffix form normalises to
  the bare digits `1000`, which the row's own `// PORT SRC: MaterialType.cpp:31` marker holds
  (the vendored line writes `1000` literally), checked against every other whole-number float
  field in the table the same way. No
  *behavioural* divergence found — every value both passes checked was correct.
- **Fixer:** no behavioural fix needed from either pass above. **Not a clean pass, though**: at
  the point both reviewers signed off, `just port-check` had not yet run, and when it did it
  found this unit's row derived `drafted`, not `ported` — all 80 `// PORT SRC:` markers in this
  module were missing the `src/libslic3r/` prefix `resolve_vendored` requires, and the test's
  `use ... as oracle;` import made its proof read as non-differential to the text scanner. Fixed
  in the "Fix `// PORT SRC:` markers..." commit; see "What the two review passes did not catch"
  below for why neither review pass caught it.

**Second review pass (two separate-context reviewers on PR #55, after this unit had already
landed) — recorded as each reviewer actually gave it, not softened:**

- **Reviewer 1: REJECT.** `src/libslic3r/MaterialType.cpp:18` (`MaterialType::all()`) against
  `crates/slic3r-materialtype/src/material_type.rs:75` (as it then stood) — `all()` is a public
  member of the ported class with an in-scope caller (`src/libslic3r/PrintConfig.cpp:3514`,
  manifest row `in`, crate `slic3r-printconfig`) and was not ported at all: the table was a
  private `static MATERIAL_TYPES` with no `pub fn all()`. `src/libslic3r/MaterialType.cpp:13`
  (`constexpr double DEFAULT_ADHESION_COEFFICIENT = 1.0;`) against
  `crates/slic3r-materialtype/src/material_type.rs:62` (as it then stood,
  `const DEFAULT_ADHESION_COEFFICIENT: f64 = 1f64;`) — carried no `// PORT SRC:` marker of either
  form, relying on the `f64` suffix landing in `port/port-status`'s exempt `0`/`1`/`2` set, which
  its own docstring states a float can never do by design. Also, separately and blocking
  regardless of this unit: the trial-run record above (this section, as it stood before this
  pass) had 4 of its citation pairs failing to resolve against the files as committed — the ABS
  row, the `find()` citation, the `get_temperature_range` citation and the FLEX row's
  `adhesion_coefficient` value (`0.5`, not the "bare integer `1`" the record stated) — contrary
  to the builder's own claim, in commit `bc93251`, that these had been "re-verified against
  final line numbers".
- **Reviewer 2: ACCEPT, pending the same two fixes.** Independently reached the identical two
  findings: `all()` missing with an in-scope caller at `PrintConfig.cpp:3514`, and
  `DEFAULT_ADHESION_COEFFICIENT` (and `DEFAULT_CHAMBER_MIN_TEMP`, `MaterialType.cpp:11`)
  unmarked and exempt-set-reliant — proved on a copied tree with `./port/port-status --tree`:
  the faithful `1.0` spelling with no marker is a `missing-port-src` finding, the same spelling
  with a `// PORT SRC: src/libslic3r/MaterialType.cpp:13` marker is clean. Also confirmed the
  75-row table itself transcribes exactly (parsed and diffed all 75 rows against the vendored
  table: 0 mismatches), so the finding is these two constants and the accessor, not the table.
- **Fixed in commit `166c759`** ("crates: fix findings from PR #55's two new review passes"):
  added `pub fn all() -> &'static [MaterialTypeInfo]` over the existing table
  (`material_type.rs:219-225`, citing `MaterialType.cpp:18-99`/`MaterialType.hpp:21`); gave
  `DEFAULT_CHAMBER_MIN_TEMP` and `DEFAULT_ADHESION_COEFFICIENT` real `// PORT SRC:` markers
  (`MaterialType.cpp:11`, `:13`) and rewrote `DEFAULT_ADHESION_COEFFICIENT` as `1.0`, deleting
  the comment that had documented the exempt-set reliance as a technique.
  `cargo test -p slic3r-materialtype` and `./port/port-status` (0 findings) confirmed clean
  after the fix. **This record's own first review pass's wrong `module:line` citations above
  are corrected in commit `2d6071a`** (the documentation commit that added this second-pass
  section) — the `get_temperature_range`, `find()` and FLEX-row entries, re-derived against
  `material_type.rs` as committed in `166c759` — rather than left wrong beside a note that
  they were: a citation that does not resolve is corrected, not preserved as history.

### Unit 2 — float: `GCode/PchipInterpolatorHelper.cpp` / `GCode/PchipInterpolatorHelper.hpp`

- **Manifest row:** GCode/PchipInterpolatorHelper.cpp
- **Manifest row:** GCode/PchipInterpolatorHelper.hpp
- **Reviewer 1:** `src/libslic3r/GCode/PchipInterpolatorHelper.cpp:59-81` against
  `crates/slic3r-gcode/src/pchip_interpolator_helper.rs:113-139` — `computePCHIP`'s derivative
  recurrence (`d_[i] = (w1 + w2) / (w1 / delta_[i-1] + w2 / delta_[i])`, `:76`, matching
  `pchip_interpolator_helper.rs:137`) transcribes with the same operator precedence and operand
  order; `src/libslic3r/GCode/PchipInterpolatorHelper.cpp:86-104` against
  `crates/slic3r-gcode/src/pchip_interpolator_helper.rs:152-175` — `interpolate`'s
  `std::lower_bound` (`:90`) against `Vec::partition_point(|&v| v < xi)` (`:178`): both return
  the first index whose element is not less than `xi` on a sorted ascending slice, so `i = that
  index - 1` agrees on both sides, including the exactly-on-a-knot queries this unit's own test
  exercises (`3.0`, `4.25`, `6.0`, `8.0`, `10.0`). No divergence found in either cited range.
- **Reviewer 2 (found a divergence):** `src/libslic3r/GCode/PchipInterpolatorHelper.cpp:43-54`
  against `crates/slic3r-gcode/src/pchip_interpolator_helper.rs:86-96` — the first pass's own
  module doc comment, and this row's first `tolerance` reason text, both claimed `sortData()`'s
  `std::sort -> slice::sort_unstable` idiom mapping leaves a divergence when two input points
  share an x-coordinate. Checked against the vendored line: `std::sort` on
  `std::vector<std::pair<double,double>>` (`:44-48`) uses `std::pair`'s default `operator<`,
  which compares **both** fields in order — the sort key is `(x, y)`, not `x` alone — so two
  elements compare equal only when `x` *and* `y` both match, and swapping genuinely-equal
  elements is not observable. There is nothing here for an unstable sort to diverge from a
  stable one on, with or without a duplicate `x`. The claim was wrong.
- **Fixer:** corrected `sort_data`'s doc comment
  (`crates/slic3r-gcode/src/pchip_interpolator_helper.rs:86-96`) and the test's own comment
  (near `:204`) to state the real reasoning above, and rewrote this row's `tolerance` reason text
  in `port/manifest.tsv` to the same effect (the measured bound stands — `0`, over this proof's
  own distinct-`x` input — the withdrawn claim was about *why* a tolerance row rather than an
  exact one, not about the measured number itself). `cargo test -p slic3r-gcode pchip` re-run
  clean after the fix (both rows still pass at `tier=float`).

**Second review pass (two separate-context reviewers on PR #55):**

- **Reviewer 1:** accepted the unit's behaviour outright (derivative recurrence, `lower_bound`/
  `partition_point` agreement and the Hermite basis all checked clean) but flagged, as a
  reservation rather than a finding against either module: the row's `tolerance: 0` with
  `comparison function: abs` makes the criterion `|actual - expected| <= 0`, indistinguishable
  from exact equality, so task `35-trial-run`'s own stated reason for choosing a float unit --
  "whose floating-point arithmetic makes exact equality the wrong criterion" -- is not actually
  true of this leg, and the row's reason text justified the tier circularly ("the row stays
  float ... because this is the trial's designated float leg").
- **Reviewer 2:** accepted the unit outright, including the tolerance point: independently
  verified the `0` is a real measurement by forcing the row's tolerance cell to `-1` on a probe
  copy of `port/manifest.tsv` and confirming the proof then fails with a reported deviation of
  `0.0`, so this leg genuinely exercises `Tier::Float`'s dispatch path with a real, measured
  bound rather than a placeholder. Also independently re-confirmed Reviewer 1's own earlier
  finding on this unit (the `std::sort`/NaN divergence) and found one more: `sort_data`'s
  `.partial_cmp(b).expect(...)` and `interpolate`'s four `.expect(...)` calls are live panics at
  sites where the vendored source has undefined behaviour (a NaN breaking `std::sort`'s strict-
  weak-ordering precondition, `x_.front()`/`y_.front()` on an empty vector) rather than a throw
  -- a real behavioural divergence, previously unmarked.
- **Fixed in commits `166c759`** (added `// PORT NOTE:` markers at both panic sites, naming the
  vendored UB each one stands in for) **and `1d7d555`** (rewrote this row's `tolerance` reason
  text in `port/manifest.tsv` to state plainly that this leg exercises the tolerance-dispatch
  path rather than a non-zero bound, so fan-out does not read `tolerance: 0` as this leg's
  general pattern -- the number itself was not changed, since both reviewers independently
  measured it as real). `cargo test -p slic3r-gcode` re-run clean after both fixes. The two
  `// PORT NOTE:` insertions shift every later line of `pchip_interpolator_helper.rs` by +5
  (sort_data's) and +10 (interpolate's, on top of the first); the first review pass's own
  citations above (`:113-139`, `:137`, `:152-175`, `:178`, and the fixer's "near `:204`") are
  given at their post-fix line numbers, re-derived from the file as committed in `166c759`,
  not the numbers either pass actually saw.

### Unit 3 — throw / non-const-ref-out-param: `InstanceLock.cpp` / `InstanceLock.hpp`

- **Manifest row:** InstanceLock.cpp
- **Manifest row:** InstanceLock.hpp
- **Reviewer 1:** `src/libslic3r/InstanceLock.cpp:124-154` against
  `crates/slic3r-instancelock/src/instance_lock.rs:259-317` — the constructor's `if (depth==0 &&
  now>=cooldown_until && open_lock_file(...))` guard, the retry loop's `try_lock`/deadline/sleep
  sequence, and `++depth` *before* `m_locked = file_lock != nullptr` (`:152-153`, matching
  `instance_lock.rs:311-312`) all appear in the same order in the port;
  `src/libslic3r/InstanceLock.cpp:156-168` against
  `crates/slic3r-instancelock/src/instance_lock.rs:325-339` — the destructor's own body runs
  before its member destructors release the mutex in C++ (reverse declaration order), which the
  port reproduces by running `Drop::drop`'s explicit body on the taken `guard` before that local
  `guard` goes out of scope and releases the reentrant lock itself. No divergence found.
- **Reviewer 2:** `src/libslic3r/InstanceLock.cpp:39-53` against
  `crates/slic3r-instancelock/src/instance_lock.rs:94-131` — `NativeFileLock`'s constructor
  (`O_RDWR|O_CREAT|O_CLOEXEC`, mode `0644`, throwing `std::system_error` on failure) and
  `try_lock()`'s `EWOULDBLOCK`/`EINTR` → "keep trying", anything else → the unit's other throw,
  against `OpenOptions`'s equivalent flags/mode and `io::ErrorKind::WouldBlock`/`Interrupted` →
  `Ok(false)`, anything else → `Err`; `src/libslic3r/InstanceLock.hpp:52-53` (the `private:`
  non-const-reference out-params this leg is chosen for) against
  `crates/slic3r-instancelock/src/instance_lock.rs:212-237` — `open_lock_file`/`defer` take `Slot
  &slot` in the vendored source and `&mut SlotState` here, both mutating shared state the caller
  already holds locked, matching the idiom map's "a non-const reference out-parameter -> a
  return value" by returning `bool`/`()` rather than adopting a second `&mut` parameter to mirror
  the C++ signature. No *behavioural* divergence found — every value both passes checked was
  correct. Noted, not a finding: this unit's shim can drive only the public surface
  (`InstanceLock`'s constructor, destructor, `locked()`) — `open_lock_file`/`defer` are
  `private:`, so the out-params and the caught throw are exercised transitively, and the record
  above says so rather than claiming a direct comparison that was not made.
- **Fixer:** no behavioural fix needed from either pass above. **Not a clean pass, though**: at
  the point both reviewers signed off, `just port-check` had not yet run, and when it did it
  found this unit's row derived `drafted`, not `ported` — the symbolic POSIX flock flags and the
  octal-notation open mode needed the external marker form instead of a vendored-line citation,
  `DEFAULT_TIMEOUT`/`COOLDOWN`/the retry sleep's `5` were missing a marker entirely, and the
  test's `use ... as oracle;` import made its proof read as non-differential to the text scanner.
  Fixed in the "Fix `// PORT SRC:` markers..." commit; see "What the two review passes did not
  catch" below for why neither review pass caught it.

**Second review pass (two separate-context reviewers on PR #55) — the most consequential of the
three units in this pass, recorded as each reviewer actually gave it:**

- **Reviewer 1: REJECT.** `src/libslic3r/InstanceLock.cpp:47` (`::flock(m_fd, LOCK_EX |
  LOCK_NB)`) against `crates/slic3r-instancelock/src/instance_lock.rs:56` (as it then stood) —
  the `// PORT SRC: external:` marker cited POSIX.1-2017 `<sys/file.h>` as LOCK_NB's authority
  (and the sibling marker at `:59` for LOCK_UN); POSIX.1-2017 specifies neither `flock()` nor
  that header — the values were right, the cited standard was not. `src/libslic3r/InstanceLock.
  cpp:84` (`std::map<std::string, std::unique_ptr<Slot>>`) against
  `crates/slic3r-instancelock/src/instance_lock.rs:160` (as it then stood, `HashMap<String,
  &'static Slot>`) — contrary to `docs/PORTING.md`'s type-map row for `std::map` (key-ordered),
  with no correction recorded. `src/libslic3r/InstanceLock.cpp:152-153` against
  `instance_lock.rs:277` (as it then stood) — the port's behaviour was right, but the comment
  beside it cited `InstanceLock.cpp:146-147`, which is `sleep_for` and a closing brace, one of
  roughly a dozen wrong prose citations in this module (every machine-checked `// PORT SRC:`
  marker resolved correctly; only the citations a human reads did not).
- **Reviewer 2: REJECT, pending two fixes.** Independently found the same `// PORT SRC:
  external:` POSIX mischaracterisation, the same `HashMap`/`std::map` substitution, and roughly
  fifteen further wrong prose citations by one to six lines (`locked()` is `InstanceLock.hpp:47`,
  not `:44`; `LOCK_EX | LOCK_NB` is `:47`, not `:44`; the destructor is `:156-168`, not
  `:150-161`; the module's own claim of three `BOOST_LOG_TRIVIAL` calls at `:70`, `:148`, `:157`
  -- the vendored file has exactly two, at `:97` and `:164`). Beyond citations, found a real
  soundness bug neither the first pass nor Reviewer 1 of this pass caught: the hand-written
  `ReentrantMutex`'s `unsafe impl Send/Sync` (`instance_lock.rs:330`, as it then stood) was not
  paired with a `!Send` guard the way `std::sync::Mutex`/`MutexGuard` is, so `ReentrantGuard` and
  the public `InstanceLock` were `Send` — verified by compiling a replica with an
  `assert_send::<InstanceLock>()` probe, which compiled before the fix below and fails to
  compile after it. A guard moved to a second thread while the owning thread still held the
  reentrant lock could reach `SlotState` through `DerefMut` from both threads at once; the
  vendored `std::unique_lock<std::recursive_mutex> m_slot_guard` (`InstanceLock.hpp:56`) is never
  moved across threads in the vendored code, so the port was strictly weaker than the source
  here, not equivalent.
- **Fixed in commit `166c759`**: re-cited LOCK_EX/LOCK_NB/LOCK_UN's external markers away from
  POSIX to the BSD/macOS SDK header the values were verified against
  (`usr/include/sys/fcntl.h:506-508`, reached through `<sys/file.h>`) and dropped the
  "POSIX-standardised" claim; changed the slot registry from `HashMap` to `BTreeMap`;
  added a `PhantomData<*const ()>` field to `ReentrantGuard` (the same device `MutexGuard` uses)
  so it -- and therefore `InstanceLock` -- is `!Send`, and rewrote the `Send`/`Sync` `SAFETY`
  comments to state the real invariant instead of the false claim that this was "the same
  contract `std::sync::Mutex<T>` itself relies on"; re-derived every wrong prose citation this
  **module itself** carried, including the `BOOST_LOG_TRIVIAL` count, against the file as
  committed (see `instance_lock.rs`'s own doc comments for those corrected citations).
  `cargo test -p slic3r-instancelock` and `./port/port-status` (0 findings) confirmed clean
  after the fix. **This record's own first review pass's wrong `module:line` citations, at the
  top of this unit** (the constructor at `:254-312`, `:306-307`, the destructor at `:320-334`,
  `NativeFileLock`'s constructor/`try_lock` at `:89-126`, and `open_lock_file`/`defer` at
  `:207-232`) **are corrected in commit `2d6071a`**, the documentation commit that added this
  second-pass section, re-derived against `instance_lock.rs` as committed in `166c759` — rather
  than left wrong beside a note that they were. The second-pass Reviewer 1/Reviewer 2 entries
  immediately above this bullet are deliberately **not** touched by either commit: their
  `module:line` halves (`:56`, `:59`, `:160`, `:330`, `:277`) describe the file as it stood
  *before* the fix, which is what "as it then stood" marks them as, on purpose.
- **Third pass on this same pull request (2026-11-26), two majors, both fixed.** The re-citation
  above was itself only half right: the three markers `166c759` wrote named a *second* authority,
  "glibc's `<bits/file.h>`", that does not exist — glibc has no `bits/file.h`; it defines all
  four `LOCK_*` macros directly in `misc/sys/file.h` (installed as `<sys/file.h>`), and again in
  `sysdeps/unix/sysv/linux/bits/fcntl-linux.h` under `#ifdef __USE_MISC`, both verified against
  glibc's own source. The false clause stood on `crates/slic3r-instancelock/src/instance_lock.rs:60`,
  `:66` and `:71` as `233c6d9` left them. A false cited origin in an external marker is the same defect class this
  unit had already been rejected over twice, so the three markers and the doc comment above them
  now name glibc's `misc/sys/file.h` (the macOS SDK half, `usr/include/sys/fcntl.h:506-508`,
  was re-verified against this machine's SDK and is unchanged). Separately, `fresh_lock_path`
  (the proof's own test helper) called `Path::canonicalize()` on the git-ignored
  `oracle/.cache/tmp/` and `.expect`ed it to exist, so on a checkout where no earlier step had
  created that directory `cargo test -p slic3r-instancelock` would have panicked in the helper
  instead of running the proof — it passed only because another crate's test binary happened to
  create the directory first, a cross-binary ordering dependence this project's own unit-test
  rules name as an anti-pattern, and this unit is the exemplar the fan-out copies. The helper now
  calls `std::fs::create_dir_all` before canonicalizing, the pattern its two siblings
  (`crates/slic3r-proof/tests/manifest_env.rs:29`, `crates/slic3r-proof/tests/dumps_proof.rs:118`)
  already use; it stood at `:479-482` before the fix. Neither fix changes a line of ported logic;
  the live `module:line` citations at the top of this unit were re-derived against the file as
  committed, because the doc comment gained two lines and each of the three markers one, which
  shifts every line after the old `:71` by five.
- **LOCK_EX and the `0o644` open mode, two related findings also raised on this pass (majors, not
  blockers): fixed and recorded, respectively.** `LOCK_EX`'s value (`2`) carries no `// PORT SRC:`
  marker even though the module's own doc comment claimed all three flock flags had one — `2` is
  in `port/port-status`'s exempt `0`/`1`/`2` set, so the check never required one, but the task's
  own rule ("a `// PORT SRC:` marker beside every transcribed numeric constant") does not carry
  that exemption. Fixed in the same commit by giving `LOCK_EX` the matching external marker.
  The `0o644` open-mode marker's text claimed to "match" `InstanceLock.cpp:39`'s `0644` under the
  external form, which `docs/PORTING.md` decision 2 reserves for a constant whose origin is
  genuinely outside the vendored tree — this one's origin is a vendored line, which the marker
  itself names, so the form was being used outside its sanctioned case. Measured, on this same
  pass, that the alternative (a plain vendored-line marker) actually fails `port-status`'s own
  text match (`0o644`'s normalised text is not a substring of a line spelling `0644`), so no
  marker spelling both passes the checker and cites only a value genuinely outside the vendored
  tree. Kept the external form, with the comment rewritten in commit `166c759` to say plainly
  that this is a `port-status` gap being worked around rather than a case decision 2 covers —
  recorded here, and in `docs/PORTING.md`'s new decision 8 (see "Corrections to docs/PORTING.md"
  below), rather than silently kept as before.

## What the two review passes did not catch

Recorded for the same reason the review methodology note above is: `just port-check` (R16's
`// PORT SRC:` marker checks, R19's proof-marker-differential check), run after both review
passes above had already signed off on all three units, found 545 findings across all three --
markers missing the `src/libslic3r/` path prefix `resolve_vendored` requires, float literals
written in a decimal form that does not textually match the vendored line's bare-integer
spelling, two symbolic `flock(2)` flag values and one octal-notation mismatch needing the external
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

**One correction, found on the second, separate-context review pass (addendum above) rather than
the first sequential pass** — the first pass's own note that "every rule this trial actually
exercised ... held exactly as stated" is withdrawn to that extent; it was true of the rules the
first pass actually checked, not of this one.

**`docs/PORTING.md` decision 8 (new): a transcribed literal must be spelled so its own text, not
only its value, matches the cited line.** `port/port-status`'s `check_numeric_constants` matches
a cited vendored line by the literal's *normalised text*, not its numeric value. This trial's
second review pass found two concrete, measured instances of a transcribed constant getting no
real benefit from the marker discipline because of this: `DEFAULT_ADHESION_COEFFICIENT`/
`DEFAULT_CHAMBER_MIN_TEMP` (`slic3r-materialtype`) were written with an `f64` suffix specifically
so they landed, unmarked, in the exempt `0`/`1`/`2` set rather than citing their real vendored
line; and the `0o644` open-mode constant (`slic3r-instancelock`) cannot cite its real vendored
line at all, because Rust's `0o644` is not a textual substring of C's `0644`, so the constant is
carried under the external-marker form though its origin is not actually outside the vendored
tree. Both are fixed in `crates/slic3r-materialtype/src/material_type.rs` and
`crates/slic3r-instancelock/src/instance_lock.rs` (commit `166c759`); decision 8
(`docs/PORTING.md`, "Decisions recorded on 2026-11-26", commit `0a3a903`) states the rule, both
failure shapes, and what to do when no Rust spelling can be made to match (the external form,
used as a stated workaround rather than under decision 2's "origin outside the vendored tree"
criterion). Every one of the ~385 remaining in-scope rows that transcribes a numeric constant is
subject to this rule; fan-out should read it before transcribing its first literal.

Every other rule this trial exercised — the idiom map's "non-const reference out-parameter -> a
return value" (units 1 and 3), the error-set rule (units 2 and 3), the `std::sort`/
`slice::sort_unstable` idiom entry (unit 2, where the adversarial-review finding above was about
this task's own draft misreading that entry, not about the entry itself) — held exactly as
stated, including under the second review pass. What this trial also found, besides decision 8,
is two mismatches between a row's seeded `proof path` cell and the mechanism that cell names
(`MaterialType`'s rows name proof path 3, the include-the-vendored-`.cpp` mechanism, but
`MaterialType`'s methods are externally linked, not internal-linkage, so including the `.cpp`
whole would duplicate-define them against the archive; `GCode/PchipInterpolatorHelper.hpp`'s row
names proof path 2, but the dump corpus it is seeded from carries no field this unit produces) —
both handled the same way `crates/slic3r-commondefs/src/common_defs.rs`'s own `// PROOF: 2`
finding already established (keep the cell's marker literal, prove the row by the mechanism that
actually works, record the mismatch in the module and the pull request) before this trial ran.
That precedent, not a new `docs/PORTING.md` rule, is what already governs a seeded-proof-path
mismatch; this trial adds two more measured instances of it rather than a reason to change the
document.
