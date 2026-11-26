---
kind: plan
---
# Plan: Port three units through the full review loop (the trial run)

## Files that change

- `docs/PORTING.md` — corrected for whatever the three units show the contract gets wrong (or
  stated as finding no correction, if that is what happens).
- `docs/WORKFLOW.md` — gains the dated trial-run record: both review verdicts, `source:line`
  against `module:line`, and each unit's manifest row.
- `crates/slic3r-materialtype/src/material_type.rs` (new) — the fixed-point (near-miss) unit.
- `crates/slic3r-gcode/src/pchip_interpolator_helper.rs` (new) — the float unit.
- `crates/slic3r-instancelock/src/instance_lock.rs` (new) — the throw/non-const-ref-out-param unit.
- `crates/slic3r-materialtype/src/lib.rs`, `crates/slic3r-gcode/src/lib.rs`,
  `crates/slic3r-instancelock/src/lib.rs` — each gains one `mod` line, alphabetically placed.
- `crates/slic3r-materialtype/tests/material_type.rs`,
  `crates/slic3r-gcode/tests/pchip_interpolator_helper.rs`,
  `crates/slic3r-instancelock/tests/instance_lock.rs` (new, or `#[cfg(test)]` modules in the
  files above) — each unit's proof.
- `crates/slic3r-ffi/cxx/proof_units/material_type.{h,cpp}`,
  `crates/slic3r-ffi/src/proof_units/material_type.rs` (and the same triple for
  `pchip_interpolator_helper` and `instance_lock`) — each unit's own, new proof shim
  (`docs/PORTING.md`, "Decisions recorded on 2026-11-25", decision 1).
- No fuzz target: none of the three chosen rows' `fuzz target` column says `required`.
- `port/manifest.tsv` — only `PchipInterpolatorHelper.cpp`/`.hpp`'s `equality tier`, `tolerance`
  and `comparison function` cells (the fixed-point and throw/out-param rows are already `exact`
  and need no cell changed).
- `intent/libslic3r-rust-port/tasks/35-trial-run/plan.md` — this file.

## Order of work

### Step 1: Confirm the dependency state before choosing units
- delivers: R28

Before choosing the three rows, check whether `port/foundation.tsv`'s 33 batches — the
hand-designed foundation types (`Point`, `Polygon`, `ExPolygon`, `Config`, and the rest plan step
20 cut as `34-foundation-spike`'s batches, one task per row: `52-foundation-common-defs` through
`84-foundation-precise-seam`) — are actually on `main` with real content, not only crate
skeletons. The task names this as a hard wait ("it also waits for the foundation tasks cut from
`port/foundation.tsv`"), not a preference, because an ordinary file-unit's module needs the
foundation types' shape to compile when its vendored source names them.

**Risk, found during this step, carried into every step after it:** at the time this plan is
written, `git log origin/main` shows only task `34-foundation-spike` merged (the crate skeletons
and `port/foundation.tsv` itself). Of the 33 foundation batches, 0 are merged: four —
`52-foundation-common-defs`, `53-foundation-int128-queue`, `54-foundation-objectid-clonable` and
`55-foundation-libslic3r-execution` (`foundation-01` through `foundation-04`: `CommonDefs`/
`Exception`/`I18N`, `Int128`/`MutablePriorityQueue`, `ObjectID`/`clonable_ptr`,
`libslic3r.h`/`Execution`) — have open draft branches, three of them (`52`, `54`, `55`) now
carrying real ported type commits and `53` still a `Start task` commit only, but **none of the four
is reachable from `origin/main`**, so none of their types is on `main` for a module here to compile
against; the other 29, including `56-foundation-point`
(`foundation-05`: `Point.cpp`/`Point.hpp`, among others), have **no branch at all**. None of the
four started batches is the one that designs `Point`, `Polygon`, `ExPolygon` or `Config` — those
are `56-foundation-point` (`foundation-05`), `62-foundation-polygon` (`foundation-11`),
`65-foundation-expolygon-flow` (`foundation-14`) and `59-foundation-config` (`foundation-08`),
none of which has even a branch yet. Every crate skeleton sampled (`slic3r-point`,
`slic3r-boundingbox`, `slic3r-line`) holds nothing but the doc-comment skeleton
`34-foundation-spike` wrote. In particular, **no Rust `Point` type exists on `main`**, and the
fixed-point unit this task must choose is explicitly described as one "whose ported surface
returns integer-coordinate output" — which is a `Point`-shaped surface in every candidate this plan
found among the non-foundation, non-template-heavy rows of `port/manifest.tsv`. That requirement
cannot be met honestly without `Point` (or an equivalent coordinate type) existing on `main` first,
which is to say without `56-foundation-point` landing.

This plan does not route around that by porting `Point`, `Polygon` or any other foundation type
itself: that is `34-foundation-spike`'s hand-design, cut into the `foundation-*` tasks this task
explicitly must not do the work of, and `asdlc branch check` would object to a crate's shape
changing underneath it from a non-owning branch regardless.

### Step 2: Choose the three rows from what can honestly be built today
- delivers: R28

Search `port/manifest.tsv`'s in-scope rows, excluding every path `port/foundation.tsv` already
claims, for:
1. A fixed-point candidate: integer-coordinate output. No row answers this as literally as a
   `Point`-shaped surface would, because `Point` is not yet real on `main` (Step 1), and this plan
   does not invent one. `MaterialType.hpp`'s `get_temperature_range(const std::string&, int&, int&)`
   (and its four siblings `get_chamber_temperature_range`/`get_adhesion_coefficient`/
   `get_yield_strength`/`get_thermal_length`, `MaterialType.cpp:94-168`) is the closest honest match
   available today: an `exact`-tier row whose ported surface returns plain integer output (two
   `int` out-params, `min_temp`/`max_temp`) from a static, deterministic lookup table, with zero
   dependency on any unlanded foundation crate. **Recorded precisely, not inflated:** this is an
   integer-output exact-tier unit, not a spatial-coordinate one — the gap this document names is
   real and not papered over. It is also a legitimate second demonstration of the idiom map's
   "non-const reference out-parameter -> a return value" rule (R19's own worked case, independent
   of the throw/out-param leg below, which exercises the same idiom against a `throw`).
2. A float candidate: this document's earlier revision wrote "none found that is honest" and
   checked `MaterialType`, `LocalesUtils` and `SlicingAdaptive` only — correctly ruling out the
   first two (no arithmetic) and the third (blocked on `Slicing.hpp`'s foundation types) — but
   never checked `GCode/PchipInterpolatorHelper.cpp`/`.hpp` against this leg at all, only against
   leg 3 below (where it was correctly ruled out for carrying no non-const-ref out-param). That was
   the gap: `PchipInterpolatorHelper` is a free-standing C++ class with **no `#include` of any
   libslic3r header and no `namespace Slic3r`** — a dependency-free leaf, unlike every geometry
   candidate this plan rejected in Step 1. Its `computePCHIP()` (`PchipInterpolatorHelper.cpp:59-81`)
   and `interpolate()` (`:86-104`) do real floating-point arithmetic — division
   (`w1 / delta_[i-1]`), the cubic Hermite polynomial evaluation — over `double`, so exact equality
   is the wrong criterion for it in the same way the task's own description states, and
   `setData()` (`:30-38`) throws `std::invalid_argument` on malformed input (irrelevant to this
   leg, relevant to none of the three descriptions, but recorded since every throw in a chosen
   unit's source has to survive the port's error-set rule regardless of which leg chose it). This
   leg **is** honestly buildable today, corrected from the earlier "none found" finding.
3. A throw/non-const-ref-out-param candidate: searched across every in-scope row outside
   `port/foundation.tsv`, for a `.hpp` with a real non-const-reference parameter *and* a real
   `throw` statement (not merely a comment mentioning one) in the same unit's `.cpp`. This was
   first recorded here as finding no match; that is wrong. `InstanceLock.hpp:52-53` declares
   `static bool open_lock_file(Slot &slot, ...)` and `static void defer(Slot &slot, ...)`, both
   non-const-reference out-params, mutated at `InstanceLock.cpp:96,109,112,115`; the same unit
   throws `std::system_error` at `InstanceLock.cpp:42` and `:52`. `InstanceLock` is crate-level 0,
   with no dependency on any unlanded foundation crate — this leg **is** honestly buildable today.
   (`GCodeStatistics.hpp:14` and `PchipInterpolatorHelper.hpp:27` were recorded as having "no real
   `throw`"; that is also wrong — `GCodeStatistics.cpp:197` throws `ExportError` and
   `PchipInterpolatorHelper.cpp:32` throws `std::invalid_argument` — but neither takes a
   non-const-reference out-param, so neither matches regardless. Real matches also exist at
   `GCode/ToolOrdering.hpp:375-377`, `GCode/GCodeProcessor.hpp:659,662` and
   `GCodeWriter.hpp:111-117`, each with throws in its `.cpp`, but each sits at crate-level 14 or 23
   and depends on unlanded foundation types, so none of those three is buildable today the way
   `InstanceLock` is.)

### Step 3: All three legs are buildable today — build them
- delivers: R28

The prior revision of this document stopped here, having found only one of three legs buildable
and reasoned (correctly, given that premise) that a one-leg run would satisfy none of the
Done-when lines and would spend the document's one-time correction opportunity on an incomplete
comparison. That premise is now corrected by Step 2 above: all three legs — `MaterialType`
(fixed-point near-miss), `PchipInterpolatorHelper` (float) and `InstanceLock`
(throw/non-const-ref-out-param) — are foundation-independent and buildable today, in three
different crates (`slic3r-materialtype`, `slic3r-gcode`, `slic3r-instancelock`), each with a
usable proof path and no dependency on any row `port/foundation.tsv` has not yet landed. This plan
therefore builds all three, through the full loop `docs/WORKFLOW.md` states, in increasing order of
engineering risk: `MaterialType` first (a static table lookup, lowest risk), then
`PchipInterpolatorHelper` (real floating-point arithmetic, needs a measured tolerance), then
`InstanceLock` last (OS-level file locking and a hand-rolled reentrant mutex, highest risk — if it
does not come together, the other two are pushed and its own failure is reported with the error
text rather than forced).

**What `InstanceLock`'s proof can and cannot reach.** `open_lock_file` and `defer` are `private:`
statics (`InstanceLock.hpp:52-53`); their symbols are exported from the archive (confirmed below),
but C++ access control still blocks a shim from calling them directly, and redeclaring the mangled
symbol to dodge that would be exactly the workaround `docs/WORKFLOW.md`'s rejection rule forbids.
The proof therefore drives the unit through its **public** surface only (the constructor,
destructor and `locked()`), exercising the out-params and the caught throw transitively rather
than comparing either one directly — recorded here so the unit's own pull request states that
plainly instead of claiming a direct comparison it does not make.

**What `open_lock_file` actually does with the throw it catches.** `NativeFileLock`'s constructor
and `try_lock()` throw `std::system_error` (`InstanceLock.cpp:42`, `:52`); `open_lock_file`
*catches* both (`:101-120`) and converts them into `false` plus a `defer` cool-down — the error
never escapes the unit. The port's error-set rule is therefore not "propagate the same throw" but
its sharper mirror: the Rust port must not widen a caught, swallowed error into a surfaced
`Result` either.

## Risks

**The stated risk (per the task):** `docs/PORTING.md` was written from reading the vendored tree,
not from porting anything, and this is the first time any unit goes through it end to end; a rule
in it may turn out wrong, under-specified or impossible to follow. Where that happens, the plan
says the fix is to correct the document in this task's pull request, never to work around it in a
module. The three units built under Step 3 are what tests this; the pull request names each
correction they force, or states plainly that a given unit found none.

**The risk this plan found, beyond the one the task named, now narrowed:** the foundation tasks
cut from `port/foundation.tsv` had not landed when this task started (Step 1), which rules out
every candidate whose vendored source names `Point`, `Polygon`, `ExPolygon`, `PrintConfig` or
another hand-designed foundation type — in particular, it rules out a literally spatial
fixed-point candidate, which is why `MaterialType` (Step 2) is recorded as a documented near-miss
on that one description rather than a full match. It does not rule out all three legs, as the
first revision of this document concluded: `PchipInterpolatorHelper` and `InstanceLock` are both
dependency-free leaves, and the corrected Step 2 above found them. The residual risk this leaves
for a later task is real and not this task's to close: the fixed-point leg's closest honest match
today answers "exact tier, integer output" rather than "integer-coordinate output", and a `Point`
type landing on `main` (`56-foundation-point`) may reveal a cleaner fixed-point candidate that a
later trial, or `docs/PORTING.md`'s own "Foundation types" section, should prefer over
`MaterialType` once it exists — this task does not block on that landing, since the task's own
text licenses the nearest faithful answer where its description is silent on what to do if no
exact match exists.

**Not a risk, withdrawn:** an earlier version of this plan recorded "no tool to run a reviewer in a
genuinely separate context" as a second blocker on the trial. That was wrong and is withdrawn.
`docs/WORKFLOW.md:95`'s two adversarial reviewers are roles the orchestration around a task's
builder supplies, not an in-session tool the builder has to carry; two reviewers did review this
pull request in separate contexts and their verdicts are recorded in its body. The foundation
dependency is the only blocker on this trial, and it is enough on its own.

**If a chosen unit's manifest `proof path` turns out not to work for it** (task `35-trial-run`'s
"What to build"): the row's `proof path` column names one of three mechanisms — FFI to the
pinned library, a committed stage dump, or the `include_vendored.cpp` translation unit — and if
the one a chosen row names cannot actually prove that unit (for example because the stage dump a
proof-path-2 row needs was never captured, or the FFI symbol a proof-path-1 row needs is not
exported from the built archive), the fix is to substitute the next mechanism that *can* prove the
unit honestly, in the same order the row's own column would try them, and to say in the pull
request which row's proof path was substituted, which mechanism replaced it, and why — never to
weaken the tier (an `exact` row does not become a `tolerance` row to dodge a broken proof path) and
never to mark the unit `ported` on a proof that does not run. Whoever is running the unit's own
loop decides the substitution, same as any other correction this task's pull request carries, since
it is the implementer and fixer roles that touch the proof, not a role outside the loop.

**A correction that changes the shape of a unit's pull request body** (task `35-trial-run`'s "What
to build", third bullet of "The two document changes"): task `36-pr-body-check` is already merged
(PR #44) and its pattern is already live on `main` as `port/port-status`'s `VERDICT_LABEL_RE`,
`CITATION_RE` and `MANIFEST_ROW_LINE_RE` (`port/port-status:2578-2601`), read by
`just pr-body-check`. Task `36-pr-body-check`'s own task says whichever of the two lands second
reconciles the pattern with the recorded form; since it landed first, this plan's job is to carry
any correction the real trial finds into that mode's regexes rather than invent a new body shape of
its own. This pull request adds no module, so `just pr-body-check` is skipped on it rather than
exercised (`port-status (pr-body): skipped -- the diff against origin/main adds no module`); a
follow-on that actually builds a unit is held to the live pattern as it stands, unless that
follow-on's own trial finds the pattern wrong, in which case it corrects both the pattern and this
sentence.

## Proof

- `git log origin/main --oneline | grep -i foundation` and a read of `crates/slic3r-point/src/lib.rs`
  (and siblings) are the proof for Step 1's finding, quoted in the pull request.
- `vendor/OrcaSlicer/src/libslic3r/MaterialType.cpp:13-168`,
  `GCode/PchipInterpolatorHelper.cpp`/`.hpp` and `InstanceLock.hpp`/`.cpp` (source:line citations
  in Step 2 above) are the proof for which three rows were chosen, quoted in the pull request.
- `nm -gjU vendor/OrcaSlicer/build/arm64/src/libslic3r/Release/liblibslic3r.a | grep -i instancelock`
  is the proof that the throw/out-param leg's row has a working proof path 1, run and quoted
  rather than assumed from the header's `private:`.
- For `PchipInterpolatorHelper`, the tolerance cell is written from a measured maximum deviation
  between the port and the oracle over the proof's own fixed input, not chosen before measuring;
  the pull request quotes the measured value and the command that produced it.
- `cargo test -p slic3r-materialtype -p slic3r-gcode -p slic3r-instancelock`,
  `just port-status --per-commit`, `just port-check`, `just inventory-check`, `just guard` and
  `cargo build --workspace` / `cargo test --workspace`, run and quoted in the pull request with
  their real output, not asserted without running them.
- `git diff port/manifest.tsv` — touches only the three chosen rows' `equality tier`, `tolerance`
  and `comparison function` cells (in practice, only `PchipInterpolatorHelper`'s two rows change
  from their seeded `exact`; `MaterialType`'s and `InstanceLock`'s rows are already `exact` and
  correct).
- Anything this plan could not complete is named in the pull request as not finished, with the
  reason and the command that showed it, rather than marked done.
