---
kind: task
step: 21
delivers: [R28]
depends: [22-port-status-gates, 19-porting-contract, 20-workflow-document, 31-proof-harness, 32-proof-dumps-fuzz, 34-foundation-spike]
size: 8
---
# Task: Port three units through the full review loop

## What to build

This is part 1 of plan step 21, "The three-unit trial run": the trial itself and the corrections
it makes to `docs/PORTING.md` and `docs/WORKFLOW.md`. Part 2 is task `36-pr-body-check`, which
adds `port/port-status --pr-body <file>` and its CI step; **this task adds neither of those**, and
`delivers: [R28]` is the whole step's.

Every fan-out step waits on this one. No Phase A or Phase B pull request is opened before it is
on `main`.

### Its first commit is a plan of its own

Start with `asdlc plan new libslic3r-rust-port/35-trial-run` and commit that plan before any code.
That is the one file this branch may add under `intent/`, and the guard allows it.

The risk the plan talks through: **`docs/PORTING.md` may be wrong in a way only the trial
reveals.** The contract was written (task `19-porting-contract`) from reading the vendored tree,
not from porting anything, and this is the first time any unit is taken through it end to end.
The plan says what happens when a rule turns out to be wrong, under-specified or impossible to
follow — the rule is corrected in this task's pull request, not worked around in the module — and
what happens when the three units cannot be found with the properties below, or when a chosen
unit's manifest `proof path` turns out not to work for it. A correction that changes the shape of
a unit's pull request body is called out in the plan, because task `36-pr-body-check` encodes
that shape.

### Choosing the three units

Take them from `port/manifest.tsv`, by its columns, among in-scope rows:

1. **A fixed-point unit**, `equality tier` exact, whose ported surface returns
   integer-coordinate output, held to exact equality.
2. **A float unit**, whose floating-point arithmetic makes exact equality the wrong criterion, held
   to a `tolerance` and a `comparison function` **that this task sets on its row**. No row declares
   either when this task starts: task `14-manifest-columns` seeds every row with
   `equality tier: exact`, an empty `tolerance` and an empty `comparison function`, and names this
   task as the one that fills them. So choosing a float unit means choosing the row *and* writing
   its three judgement cells, with the tolerance's reason text in the cell, as part of this task's
   pull request — which is also what makes a tolerance path reachable in
   `crates/slic3r-proof/src/tiers.rs` at all, since an `exact` row has none.
3. **A unit with a non-const reference out-param and a `throw`** in its vendored source. The
   manifest has no column for that, so find it by reading the vendored headers of candidate
   in-scope rows; it is the unit that exercises the contract's rule against widening or narrowing
   an error set, and its out-param is what exercises the ownership and borrow translation.

Prefer three small units in different crates, each with a usable `proof path`, and prefer units
whose crate's foundation types are already on `main`. Record in the pull request which rows were
chosen and why each one answers to its description.

### The loop, run as `docs/WORKFLOW.md` states it

For each of the three units, run the full loop, with no step collapsed into another:

- one implementer, who reads the vendored source file the unit comes from and `docs/PORTING.md`;
- then **two adversarial reviewers in separate contexts**, each of whom reads the diff **and** the
  vendored source file, is told to assume the module is wrong, cites every divergence as
  `source:line` against `module:line`, and **may not implement**;
- then one fixer.

The rejection rule applies: a workaround that needs a paragraph of justification is wrong code,
and the answer is to re-read the vendored source rather than patch around it.

### What each of the three modules ends as

These are ported modules, not Phase A drafts. For each unit:

- `crates/<crate>/src/<module>.rs` at the path its manifest row's target crate and module columns
  name, compiling;
- its `mod` line in that crate's `lib.rs`, at the alphabetical position `docs/WORKFLOW.md`
  documents;
- a `PORT STATUS` trailer naming its source path, `confidence: high|medium|low`, its equality
  tier and the oracle that proves it;
- no `todo!()` and no `unimplemented!()`;
- its proof **as a test in the unit's own crate** — `crates/<crate>/tests/<module>.rs`, or a
  `#[cfg(test)]` module in the module file — calling `crates/slic3r-proof`'s
  `tiers::compare_unit` at the tier its row declares, by the proof path its row names: FFI to the
  library built from the pin, a committed stage dump under `port/dumps/`, or the test translation
  unit `crates/slic3r-ffi/cxx/include_vendored.cpp`, with no stand-in for the code under test or
  for the oracle. It carries, beside a test whose name contains the module's snake_case name, the
  line `// PROOF: <the row's proof path cell>`, which is the convention task
  `22-port-status-gates` reads and the only thing that makes a module derive as `ported` rather
  than `drafted`. **It does not go in `crates/slic3r-proof/`**: tasks `31-proof-harness` and
  `32-proof-dumps-fuzz` reserve `crates/slic3r-proof/tests/trial_units.rs` for this task, and this
  task declines it — a proof living in `slic3r-proof` would not be "a test in the module's crate"
  and the three units would never derive as `ported`. This branch adds no file under
  `crates/slic3r-proof/`, and that reservation can be dropped;
- a differential fuzz target at `crates/slic3r-ffi/fuzz/fuzz_targets/<crate>_<module>.rs` where
  its row's `fuzz target` column says `required`, runnable with `just fuzz <target>`;
- a `// PORT SRC: <vendored path>:<line>` marker beside every transcribed numeric constant, a
  `// SAFETY:` comment on every `unsafe` block, and only the six whitelisted port markers.

### The two document changes

- **`docs/PORTING.md` is corrected** from whatever the three units show the contract gets wrong,
  in this task's pull request, before any fan-out pull request is opened. A correction keeps the
  sections the document-section check reads — crate map, type map, idiom map, don't-translate
  list, banned-substitution list and the six rules — and keeps the crate map generated from
  `port/cut.tsv` rather than hand-written.
- **`docs/WORKFLOW.md` gains the trial-run record**, in the same form a unit's pull request uses:
  for each of the three units, both review verdicts, each citing `source:line` against
  `module:line`, and the module's manifest row. That record is the "trial-run record" section
  `just port-status --per-commit` checks for, and task `36-pr-body-check` encodes its shape, so
  write it as the form every later unit is held to rather than as prose about the trial. Date the
  record, so a reader can see it was written before the first fan-out pull request was opened.
- **If `port/port-status --pr-body` and `just pr-body-check` are already on `main` when this task
  lands**, which is possible because task `36-pr-body-check` does not wait on this one, then a
  correction to the recorded body form is carried into that mode's pattern in this task's pull
  request, and this task's own pull request body passes `just pr-body-check`. That task says
  whichever of the two lands second reconciles the pattern with the recorded form; this is that
  sentence from this side. The must-not list below then forbids only **creating** the mode, the
  recipe and the CI step, not correcting the pattern they match.

### Why it is one task and not three

The whole value is comparing three different equality tiers against **one** revision of
`docs/PORTING.md`. Three tasks each editing that document would conflict in it and lose the
comparison, and a correction found by the float unit has to be visible to the fixed-point one
before either lands.

### Why this is an 8

Three units of three different equality tiers, each taken through an implementer, two adversarial
reviewers in separate contexts and a fixer, each with its own proof at its own tier, and the
contract corrected from what they show. The risk is that the contract is wrong in a way only the
trial reveals, which is why the first commit is a plan of its own.

### What it waits for and what it must not touch

Beyond its `depends`, **it also waits for the foundation tasks cut from `port/foundation.tsv`**
after the spike `34-foundation-spike` — the three units are ordinary file-units and need the
foundation types' shape and their crates, which those tasks port.

The three proofs link the library `just oracle-build` builds and, where a chosen unit's row names
proof path 2, read a dump `just oracle-dump <fixture> <configuration>` wrote, so **they run on
macOS arm64 where `just oracle-build` has run**, and `just oracle-dump` too where any of the three
rows names proof path 2 — the same condition tasks `31-proof-harness` and `32-proof-dumps-fuzz`
state for their own tests. Without it the bridge's build script fails with a message naming
`just oracle-build`, which is expected rather than a fault.

It must not:

- add `port/port-status --pr-body`, `just pr-body-check`, the CI step that runs it, or the
  `pull-requests: read` permission — creating any of those is task `36-pr-body-check`'s;
- write five of the seven paths `docs/WORKFLOW.md` forbids a port task:
  `port/reports/perf-backlog.md`, `port/reports/divergences.md`,
  `port/reports/order-containers.md`, `port/reports/order-comparators.md` and the root
  `Cargo.toml`;
- touch any derived column of `port/manifest.tsv`, or any cell of any row but the three it chose.
  **The three chosen rows' `equality tier`, `tolerance` and `comparison function` cells it does
  write**, which is the one exception and is why `port/manifest.tsv` is not in the list above: those
  are the carried-forward judgement columns task `14-manifest-columns` names this task as the filler
  of, and `just inventory-check` ignores them by design;
- add a **third-party** dependency to any crate. The path dependencies the cut names and the path
  dev-dependency on `slic3r-proof` are generated by `34-foundation-spike`'s
  `port/crate-skeletons`, so the three units' crates already have what their proofs need and no
  `Cargo.toml` is edited here; where one of them turns out to be missing, the fix is in that
  generator and not an edit here. `Cargo.lock` therefore changes only if that generator is
  corrected, and holds workspace-member edges only either way;
- change anything under `vendor/OrcaSlicer/` or move the submodule pointer, which stays at
  `f92a7c065c5718b8ef1baac1cd704df2911238f4`;
- add any file under `intent/` other than this task's own plan.

## Done when

- `intent/libslic3r-rust-port/tasks/35-trial-run/plan.md` is committed as this branch's first
  commit and names the risk that the contract is wrong in a way only the trial reveals.
- Three units are ported, in three modules under `crates/*/src/`, and the pull request names the
  three `port/manifest.tsv` rows and states which of the three descriptions each answers: the
  fixed-point unit held to exact integer-coordinate equality, the float unit held to its row's
  declared tolerance and comparison function, and the unit with a non-const reference out-param
  and a `throw`.
- Each of the three modules ends with a `PORT STATUS` trailer naming its source path,
  `confidence`, its equality tier and its oracle; each has its `mod` line in its crate's `lib.rs`;
  none holds a `todo!()` or `unimplemented!()`.
- `just port-status --per-commit` reports the derived status of all three as `ported`.
- Each of the three has a passing proof **in its own crate** — `crates/<crate>/tests/<module>.rs`
  or a `#[cfg(test)]` module in the module file — calling `crates/slic3r-proof`'s
  `tiers::compare_unit` at its row's tier by its row's named proof path, carrying a
  `// PROOF: <proof path>` line beside a test whose name contains the module; and
  `cargo test -p <crate>` passes for each of the three crates, on a machine where
  `just oracle-build` has run.
- No file is added under `crates/slic3r-proof/`, and
  `crates/slic3r-proof/tests/trial_units.rs` does not exist.
- Every one of the three whose row says `fuzz target: required` has a target at
  `crates/slic3r-ffi/fuzz/fuzz_targets/<crate>_<module>.rs`, and `just fuzz <target>` runs it.
- No test file or `#[cfg(test)]` module added by this task names a `mock`, `stub`, `fake` or
  `dummy` identifier.
- `just port-check`, `just check` and `just guard` pass; `cargo build --workspace` and
  `cargo test --workspace` pass, on the same machine condition as above.
- The float unit's row carries a non-empty `tolerance` with its reason text and a
  `comparison function` the comparator implements, and its proof exercises that tolerance path
  rather than exact equality.
- `docs/PORTING.md` carries every correction the trial produced, and the pull request lists each
  correction with the unit that forced it — or states explicitly that the three units found none.
- `docs/WORKFLOW.md`'s trial-run record holds, for each of the three units, both review verdicts
  with `source:line` against `module:line` citations and the module's manifest row, is dated, and
  that date is before the first fan-out pull request; `just port-status --per-commit` still reports
  both documents' required sections present.
- `git diff port/manifest.tsv` touches only the `equality tier`, `tolerance` and
  `comparison function` cells of the three chosen rows, and `just inventory-check` still exits 0
  and prints nothing.
- `git diff` shows no change to the root `Cargo.toml`, any file under `port/reports/`,
  `.github/workflows/ci.yml` or `.asdlc/checks`; any change to `Cargo.lock` holds workspace-member
  edges only and the pull request says what forced it.
- Nothing under `vendor/OrcaSlicer/` changed and `git submodule status` still shows
  `f92a7c065c5718b8ef1baac1cd704df2911238f4`.
