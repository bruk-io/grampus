---
kind: plan
---
# Plan: Port three units through the full review loop (the trial run)

## Files that change

- `docs/PORTING.md` — corrected for whatever the three units show the contract gets wrong (or
  stated as finding no correction, if that is what happens).
- `docs/WORKFLOW.md` — gains the dated trial-run record: both review verdicts, `source:line`
  against `module:line`, and each unit's manifest row.
- `crates/<crate-a>/src/<module-a>.rs` (new) — the fixed-point unit.
- `crates/<crate-b>/src/<module-b>.rs` (new) — the float unit.
- `crates/<crate-c>/src/<module-c>.rs` (new) — the throw/non-const-ref-out-param unit.
- `crates/<crate-a>/src/lib.rs`, `crates/<crate-b>/src/lib.rs`, `crates/<crate-c>/src/lib.rs` —
  each gains one `mod` line, alphabetically placed.
- `crates/<crate-a>/tests/<module-a>.rs`, `crates/<crate-b>/tests/<module-b>.rs`,
  `crates/<crate-c>/tests/<module-c>.rs` (new, or `#[cfg(test)]` modules in the files above) —
  each unit's proof.
- `crates/slic3r-ffi/fuzz/fuzz_targets/<crate>_<module>.rs` (new) — one per chosen row whose
  `fuzz target` column says `required`.
- `port/manifest.tsv` — only the `equality tier`, `tolerance` and `comparison function` cells of
  the float unit's row (the fixed-point and throw/out-param rows are already `exact` and need no
  cell changed).
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
`libslic3r.h`/`Execution`) — have draft branches with only a `Start task` commit each, no real
type content reachable from `origin/main`; the other 29, including `56-foundation-point`
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
1. A fixed-point candidate: integer-coordinate output. **None exists among the rows this task may
   touch**, because every such row's vendored source takes or returns `Point`/`Polygon`-shaped
   types, and `Point` is not yet real on `main` (Step 1). This leg of the trial cannot be built
   until at least `Point`'s foundation task lands.
2. A float candidate: **none found that is honest.** `MaterialType.cpp`/`.hpp` (crate-level 0) was
   first recorded here as "a genuine tolerance candidate", but that is wrong on the vendored
   source: `get_adhesion_coefficient`/`get_yield_strength`/`get_thermal_length`
   (`MaterialType.cpp:136-168`) copy a `double` out of a `static const` table of literals
   (`MaterialType.cpp:18-100`+) or assign a `constexpr` default (`MaterialType.cpp:13-15`) — no
   add, multiply, divide, sqrt or transcendental anywhere in the unit. Exact equality is the right
   criterion for it, so it does not answer to the task's float description ("floating-point
   arithmetic makes exact equality the wrong criterion"). `LocalesUtils.cpp`'s
   `string_to_double_decimal_point`/`atof_decimal_point` (crate-level 0) is string parsing, not
   arithmetic, for the same reason. A unit that does real floating-point arithmetic exists —
   `SlicingAdaptive::next_layer_height` (`SlicingAdaptive.cpp:109-194`, `sqrt`, division, running
   sums) — but it takes `const ModelObject&`/a `SlicingParameters` built from `Slicing.hpp`, both
   unlanded foundation types (`port/foundation.tsv` rows `foundation-10`, `foundation-28`), so it
   is blocked the same way the fixed-point leg is. The honest count for this leg today is 0, not 1.
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

### Step 3: Stop here and report rather than force a match
- delivers: R28

This plan does not invent a fixed-point unit against an unlanded `Point` type, does not fabricate
a tolerance reason for a unit that does no floating-point arithmetic, and does not fabricate the
two adversarial review verdicts `docs/WORKFLOW.md`'s trial-run record requires (no subagent tool
was available in the session that built this branch; see this task's pull request for the note —
this is a property of that session, not a claim that the workflow's two-reviewer loop needs new
tooling in general, since the loop's reviewers are supplied by the orchestration around a task's
builder rather than by an in-session tool). The plan at this point is the honest state: of the
three legs, exactly one — the throw/non-const-ref-out-param leg, on `InstanceLock` — is honestly
buildable today; the fixed-point leg is blocked on `56-foundation-point` (`Point.cpp`/`Point.hpp`,
`port/foundation.tsv` row `foundation-05`), and the float leg has no candidate that both performs
floating-point arithmetic and is free of an unlanded foundation dependency. One buildable leg does
not let this task make its one comparison against a single revision of `docs/PORTING.md` across
all three equality tiers, so the decision below is still to build nothing — but the leg that is
blocked and the leg that is clear were swapped in the first version of this plan, and that is
corrected here rather than carried forward. If a reader wants this task carried further once
`56-foundation-point` (and a foundation-independent, arithmetic float candidate, if one exists
among rows not yet searched for that specific property) lands on `main`, that is a follow-on to
this trial, not a silent substitution here.

## Risks

**The stated risk (per the task):** `docs/PORTING.md` was written from reading the vendored tree,
not from porting anything, and this is the first time any unit goes through it end to end; a rule
in it may turn out wrong, under-specified or impossible to follow. Where that happens, the plan
says the fix is to correct the document in this task's pull request, never to work around it in a
module. This trial does not reach that point for any of the three units (Step 2: the fixed-point
and float legs have no honest candidate today, and building the one buildable leg alone — the
throw/out-param leg, on `InstanceLock` — would not be the three-tier comparison this task exists
to make), so `docs/PORTING.md` is not corrected by this pull request; a follow-on that completes
the trial corrects it from what building actually shows, or states plainly that it found nothing
to correct.

**The risk this plan found, beyond the one the task named:** the task's own dependency — the
foundation tasks — had not landed when this task started, which blocks the fixed-point leg
entirely and leaves the float leg with no candidate that is both free of an unlanded foundation
dependency and does real floating-point arithmetic. A trial run that cannot seat all three legs
cannot fully validate `docs/PORTING.md` against all three equality tiers and the ownership/borrow
rule the throw/out-param leg exists to exercise; a one-leg record would be a partial trial, not the
full one plan step 21 describes, so none is built and none is reported as a trial.

**Process risk:** the loop calls for two adversarial reviewers in separate contexts. The session
that built this branch had no tool to spawn an isolated reviewing agent; the pull request says
exactly how review was actually done for whatever is built, rather than recording a verdict no
real review produced. This is a property of that session, not of the workflow: the loop's two
reviewers are roles the orchestration around a task's builder supplies, not an in-session tool a
builder must have of its own.

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
- `vendor/OrcaSlicer/src/libslic3r/MaterialType.cpp:13-168`, `SlicingAdaptive.hpp`/`.cpp` and
  `InstanceLock.hpp`/`.cpp` (source:line citations in Step 2 above) are the proof for which leg is
  actually buildable, quoted in the pull request.
- `just port-status --per-commit`, `just port-check`, `just inventory-check`, `just guard` and
  `cargo build --workspace` / `cargo test --workspace`, run and quoted in the pull request with
  their real output, not asserted without running them.
- `git diff port/manifest.tsv` — empty, since no row's judgement cells are written when no unit is
  built.
- Anything this plan could not complete is named in the pull request as not finished, with the
  reason and the command that showed it, rather than marked done.
