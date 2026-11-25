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

Before choosing the three rows, check whether `port/foundation.tsv`'s tasks — the hand-designed
foundation types (`Point`, `Polygon`, `ExPolygon`, `Config`, and the rest plan step 20 cut as
`34-foundation-spike`'s batches, started as tasks `52`–`55` and onward) — are actually on `main`
with real content, not only crate skeletons. The task names this as a hard wait ("it also waits
for the foundation tasks cut from `port/foundation.tsv`"), not a preference, because an ordinary
file-unit's module needs the foundation types' shape to compile when its vendored source names
them.

**Risk, found during this step, carried into every step after it:** at the time this plan is
written, `git log origin/main` shows only task `34-foundation-spike` merged (the crate skeletons
and `port/foundation.tsv` itself); every foundation task branch (`52-foundation-common-defs`,
`53-foundation-int128-queue`, `54-foundation-objectid-clonable`,
`55-foundation-libslic3r-execution`, and the rest `foundation.tsv` names) has only a `Start task`
commit, with no real type content, anywhere reachable from `origin/main`. Every crate skeleton
sampled (`slic3r-point`, `slic3r-boundingbox`, `slic3r-line`) holds nothing but the doc-comment
skeleton `34-foundation-spike` wrote. In particular, **no Rust `Point` type exists on `main`**, and
the fixed-point unit this task must choose is explicitly described as one "whose ported surface
returns integer-coordinate output" — which is a `Point`-shaped surface in every candidate this plan
found among the non-foundation, non-template-heavy rows of `port/manifest.tsv`. That requirement
cannot be met honestly without `Point` (or an equivalent coordinate type) existing on `main` first.

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
2. A float candidate: `MaterialType.cpp`/`.hpp` (`crates/slic3r-materialtype`, crate-level 0, no
   dependency on any unlanded foundation crate) is the best fit found — `get_adhesion_coefficient`,
   `get_yield_strength` and `get_thermal_length` each return a `double` through a non-const
   reference out-parameter, a lookup over a small fixed table, with no geometry dependency.
   `LocalesUtils.cpp`'s `string_to_double_decimal_point`/`atof_decimal_point` (crate-level 0,
   `crates/slic3r-localesutils`) is the fallback if `MaterialType`'s table proves unsuitable for a
   tolerance (its comparison is closer to exact string-driven parsing than to a tolerance-worthy
   arithmetic result).
3. A throw/non-const-ref-out-param candidate: searched across every in-scope row outside
   `port/foundation.tsv`, for a `.hpp` with a real non-const-reference parameter *and* a real
   `throw` statement (not merely a comment mentioning one) in the same unit's `.cpp`. No row
   combines both honestly without depending on an unlanded foundation type: `InstanceLock.cpp` has
   a real `throw std::system_error`, but takes no non-const reference out-parameter; the rows that
   do take one (`MaterialType`, `GCodeStatistics`, `PchipInterpolatorHelper`) have no real `throw`
   in their `.cpp`. This leg could not be completed honestly either, independent of Step 1's
   finding.

### Step 3: Stop here and report rather than force a match
- delivers: R28

Per the task's own closing instruction — "stop working around it, push what you have, and report
exactly what failed" — this plan does not invent a fixed-point unit against an unlanded `Point`
type, does not relax "a real `throw`" to a comment, and does not fabricate the two adversarial
review verdicts `docs/WORKFLOW.md`'s trial-run record requires (this task's environment has no
tool to run a reviewer in a genuinely separate context; see this task's pull request for the
note). The plan at this point is the honest state: `MaterialType` is a real, buildable candidate
for the float leg, but the fixed-point leg is blocked on a foundation task, and the
throw/out-param leg has no honest candidate under today's non-foundation rows. If a reader wants
this task carried further once a `Point`-shaped foundation task lands on `main`, that is a follow-on
to this trial, not a silent substitution here.

## Risks

**The stated risk (per the task):** `docs/PORTING.md` was written from reading the vendored tree,
not from porting anything, and this is the first time any unit goes through it end to end; a rule
in it may turn out wrong, under-specified or impossible to follow. Where that happens, the plan
says the fix is to correct the document in this task's pull request, never to work around it in a
module. This trial could not reach that point for two of the three units (Step 1, Step 2), so
`docs/PORTING.md` is corrected only for whatever surfaces while building the float unit, if
anything does; the pull request states plainly if it finds nothing to correct, rather than
inventing a correction to look complete.

**The risk this plan found, beyond the one the task named:** the task's own dependency — the
foundation tasks — had not landed when this task started, which blocks the fixed-point leg
entirely and narrows the throw/out-param leg to nothing found. A trial run that cannot seat all
three legs cannot fully validate `docs/PORTING.md` against all three equality tiers and the
ownership/borrow rule the throw/out-param leg exists to exercise; the two-leg record this task can
honestly produce is a partial trial, not the full one plan step 21 describes, and is reported as
such.

**Process risk:** the loop calls for two adversarial reviewers in separate contexts. This task's
environment provides no tool to spawn an isolated reviewing agent; the pull request says exactly
how review was actually done for whatever is built, rather than recording a verdict no real review
produced.

## Proof

- `git log origin/main --oneline | grep -i foundation` and a read of `crates/slic3r-point/src/lib.rs`
  (and siblings) are the proof for Step 1's finding, quoted in the pull request.
- For the float unit, if built: `cargo test -p slic3r-materialtype` passing, with
  `crates/slic3r-proof`'s `tiers::compare_unit` called at the tolerance tier the row declares, and
  `// PROOF:` / `PORT STATUS` markers present.
- `just port-status --per-commit`, `just port-check`, `just inventory-check`, `just guard` and
  `cargo build --workspace` / `cargo test --workspace`, run and quoted in the pull request with
  their real output, not asserted without running them.
- `git diff port/manifest.tsv` limited to the float row's three judgement cells.
- Anything this plan could not complete is named in the pull request as not finished, with the
  reason and the command that showed it, rather than marked done.
