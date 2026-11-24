---
kind: task
step: 14
delivers: [R14, R15, R16, R29]
depends: [21-port-status-markers, 16-ownership-spike, 19-porting-contract, 20-workflow-document]
size: 5
---
# Task: Derive each manifest row's status and gate the burn-down

## What to build

This is **part 2 of plan step 14**, "port-status", so its `delivers` is the whole step's. Part 1
(task `21-port-status-markers`) added the executable `port/port-status`, the `just port-status`
recipe, and the text-based per-module checks: the `PORT STATUS` trailer and its agreement with the
module's `port/manifest.tsv` row, the trailer's source path existing at the pin, `// SAFETY:` on
every `unsafe` block, `// PORT SRC:` beside every ported numeric constant with the
placeholder-constant check, and the marker whitelist. Part 3 (task `23-perf-backlog`) adds the
per-marker report assertion and the perf backlog. This task adds the rest of step 14 to the same
script: the status derivation, the burn-down gate, the required-fuzz-target check, the two
document-section checks, and the separation of the command's two modes.

Every check added here is **text-based**: the script reads files as lines, never parses Rust and
never builds. Step 14 requires that, because the script has to run over Phase A drafts, which are
not required to compile, are not valid Rust, and which `cargo fmt` never sees because no `mod`
line reaches them.

**Status derivation, which is the authority.** The script derives every `port/manifest.tsv` row's
status from the Rust tree and never reads the row's `status` cell as a fact. For a row, the
module's path is the one its `target crate` and `module` columns name, `crates/<crate>/src/<module>.rs`.
The rule is the one plan step 11 states for `port/inventory`, and **this script's computation is the
authority, which is not the same as the two computations agreeing**:

- `drafted` — a file exists at that path, it carries a `PORT STATUS` trailer naming that row's
  source path, and no `mod` declaration reaches it.
- `ported` — a `mod` declaration reaches it, it holds no `todo!()` and no `unimplemented!()`, and
  its proof is present, by the convention below.
- `not-started` — anything else.

**Where the two computations differ, and why that is safe.** `14-manifest-columns` states plainly
that `port/inventory` evaluates only the text-visible half of the rule — the target file present
with a trailer naming the row, the `mod` line, and the absence of `todo!()` and
`unimplemented!()` — and hands the proof half to this script. So `port/inventory`'s cell can read
`ported` for a module this script derives as `drafted`, and the manifest's cell is informational
while this script's derivation is what the gate uses. Say so in the script's own comment. **And no
check in this repository evaluates a proof's *outcome*:** this script checks the proof is present,
`cargo test --workspace` in the same macOS job is the only thing that runs it, and the spec's R26
gate can therefore read zero rows remaining while a proof is failing — which is why plan step 23's
last task puts the gate in that job beside `cargo test --workspace` and not somewhere else.

**The proof-presence convention, fixed here**, because `proof path` holds one of R19's three
mechanisms and "a test that names it" would otherwise be three different greps: the proof is a test
in the unit's own crate, either a file under `crates/<crate>/tests/` or a `#[cfg(test)]` module in
the module file, which carries, on its own line beside a test whose name contains the module's
snake_case name, the literal line

```
// PROOF: <the row's proof path cell, verbatim>
```

The script looks for that line and that test name and nothing else. A module with a `mod` line, no
`todo!()` and no such marker derives as `drafted`, not `ported`, which is the line below a reviewer
answers. `35-trial-run`, the first task that writes a proof, writes it in this form.

A `mod` declaration reaches the file when an uncommented `mod <module>;` line for it exists in the
crate's `lib.rs` or in a module file above it. Because the script never builds, the *result* of a
unit's proof is `cargo test`'s to report, not this script's; the script checks the proof is there
and prints that it judged presence and not outcome. That is why plan step 23's last task puts the
gate in the macOS job of `.github/workflows/ci.yml` beside `cargo test --workspace`, where a
failing proof is a red check on the same job.

**The burn-down and the counts the script prints.** It counts and prints `todo!()`,
`unimplemented!()`, and modules gated out of the build — a file under `crates/*/src/` that carries
a `PORT STATUS` trailer and that no `mod` declaration reaches, which is exactly how a Phase A draft
sits in the tree. It prints how many in-scope manifest rows have a derived status other than
`ported`. It prints the count of rows in `port/ownership.tsv` whose `class` column is `unknown`,
and never fails on that count. It prints all of these in both modes.

**The required-fuzz-target check.** A row whose `fuzz target` column is `required` and whose
derived status is `ported` fails unless a target naming its module exists under
`crates/slic3r-ffi/fuzz/fuzz_targets/`, at `<crate>_<module>.rs`. The check fires only at
`ported`: a row whose derived status is `not-started` or `drafted` is not failed by it, because a
Phase A draft has no proof and no target yet and the fuzz target is part of a Phase B unit's work.
This is the one half of R19 a script can hold rather than a reviewer, and plan step 23 carries it
in a Phase B task's "Done when".

**The two document-section checks, and the twelve heading strings they match exactly.** The script
matches on the exact heading text, not on keywords, because `19-porting-contract` and
`20-workflow-document` both state that this check reads their headings and that renaming one must
break it. The strings are:

In `docs/PORTING.md`: `## Crate map`, `## Type map`, `## Idiom map`, `## Don't-translate list`,
`## Banned-substitution list` and `## Rules`; and under `## Rules`, six `###` subsections,
`### Scaling conversions`, `### Float widths`, `### Rounding modes`, `### Error sets`,
`### Number formatting` and `### Linked libraries` — the check requires all six, which is what
R12's "the six rules" means.

In `docs/WORKFLOW.md`: `## Unit of work`, `## The two phases`, `## The loop`,
`## The rejection rule`, `## Conflict discipline` and `## Trial-run record`.

It reads headings, not prose quality, and on failure it names the missing heading by its exact
text.

**The two modes, which is the part that has to be exactly right.** `port/port-status` takes
`--per-commit`, which task 21 adds; this task gives the flag its full meaning rather than creating
it. `--per-commit`
runs exactly the subset plan step 16 puts in `.asdlc/checks`: task 21's trailers and their
agreement with the manifest row, the source paths at the pin, `// SAFETY:`, `// PORT SRC:` and the
placeholder-constant check, the marker whitelist, task 23's per-marker report assertion, the
required-fuzz-target check above, and the two document-section checks. It **passes while a row's
derived status is `not-started` or `drafted`**, because a module that is honestly a draft is not a
fault, and that is what lets a hundred concurrent port branches each pass the checks they are held
to.

`just port-status` with no argument is all of that **plus the gate**: it fails while the burn-down
of `todo!()`, `unimplemented!()` and gated-out modules is not zero, and it fails while any in-scope
manifest row's derived status is other than `ported`, printing how many remain. That is R16's and
R26's wording, so it is true only at the end of plan step 23, which is why it is not a check a
task's pull request passes. The script forwards nothing and takes no file in either mode.

**What this task must not do.** It adds no third mode: `--pr-body <file>` and
`just pr-body-check` belong to plan step 21 and are cut as task `36-pr-body-check`. It adds no
line to `.asdlc/checks` and no step or job to `.github/workflows/ci.yml` — plan step 16 does that,
cut as task `24-port-check-ci`, and the gate's own CI line is added by plan step 23's last task.
It does not regenerate, rewrite or whole-file diff `port/reports/perf-backlog.md`,
`divergences.md`, `order-containers.md` or `order-comparators.md`, and it adds no per-marker
assertion; task 23 owns those. It writes no cell of `port/manifest.tsv`: a row's status moves
because a Rust file appeared, not because anybody edited a line, and that is what keeps a hundred
task branches from colliding on one line of one file. It adds no check that gates on throughput,
instruction count or allocation, here or anywhere.

**What it waits on.** Task 21 for the script and its per-module checks; task
`19-porting-contract` and task `20-workflow-document` for the two documents whose sections it
checks; task `16-ownership-spike` for `port/ownership.tsv`, whose `unknown` rows it counts. The
spike commits a row per raw-pointer field, which is all the count needs; the rows' classes are
filled by the tasks cut from `port/ownership-batches.tsv` in a later pass, and this task does not
wait for those. The manifest columns it reads — `target crate`, `module`, `in scope`,
`fuzz target`, `proof path` — come from task `14-manifest-columns`, which task 21 already waits
on. Nothing here waits for a ported module to exist: with an empty port tree every in-scope row
derives as `not-started`, `--per-commit` passes and the bare command fails with the full remaining
count, and that is the correct behaviour on the day this lands.

**Why it is a 5, and why it is not split further.** The derived status is the authority the whole
fan-out rests on — it is what makes progress a property of the tree instead of a cell a hundred
branches edit — and it has to agree exactly with the rule `port/inventory` uses. The two modes
cannot be built apart from it, because the mode split is defined by which of these results fails:
`--per-commit` has to stay green over drafts and over an empty port tree, and the bare command has
to be the completion gate. The required-fuzz-target check and the two document-section checks ride
along in the same subset and are small beside that.

## Done when

- `port/port-status` derives every `port/manifest.tsv` row's status from the Rust tree by the rule
  above, and reads no row's `status` cell.
- `just port-status --per-commit` exits 0 on a tree where every in-scope row's derived status is
  `not-started`, and on one where some are `drafted`.
- `just port-status` with no argument exits non-zero on that same tree and prints how many in-scope
  rows remain and the counts of `todo!()`, `unimplemented!()` and modules gated out of the build.
- The remaining-rows line uses the exact wording the plan's recorded proof for R26 names, so that on
  a tree with no in-scope rows remaining `just port-status` exits 0 and prints
  `0 in-scope rows remaining`, and on a tree with rows remaining it prints that count in the same
  wording.
- A module with a `mod` line, no `todo!()` and no `unimplemented!()` but no `// PROOF:` marker
  beside a test naming it derives as `drafted` and not as `ported`.
- Both modes print the burn-down counts, the remaining in-scope row count, and the count of
  `unknown` rows in `port/ownership.tsv`; only the no-argument mode fails on them.
- `just port-status --per-commit` exits non-zero, naming the heading by its exact text, when
  `## Rounding modes` is renamed in `docs/PORTING.md` or `## The rejection rule` is renamed in
  `docs/WORKFLOW.md`, and exits 0 with all twelve headings present.
- `just port-status --per-commit` exits non-zero for a manifest row whose `fuzz target` is
  `required`, whose derived status is `ported`, and for which
  `crates/slic3r-ffi/fuzz/fuzz_targets/<crate>_<module>.rs` does not exist — that exact path, so a
  file naming the module at any other path does not satisfy the check; it exits 0 for such a row
  whose derived status is `not-started` or `drafted`.
- The script never invokes `cargo`, `cmake` or a build, and never parses Rust: a file that is not
  valid Rust is read as lines and checked.
- `git diff` on the task's branch shows no change to `.asdlc/checks`, `.github/workflows/ci.yml`,
  `port/manifest.tsv` or any file under `port/reports/`.
- The script has no `--pr-body` mode and no perf-backlog regeneration.
