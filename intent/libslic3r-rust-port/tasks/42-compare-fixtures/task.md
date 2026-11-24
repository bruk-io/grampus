---
kind: task
step: 26
delivers: [R22]
depends: [41-compare-benchy]
size: 3
---
# Task: Compare the port and the oracle on every committed fixture

## What to build

The whole of plan step 26, which delivers R22. C6's decision makes this the measure of "usable to
slice files for 3D printing" for this intent: the port slices a mesh plus a resolved
configuration, the project-format and preset layers stay out, and the owner's own print on an X1
Carbon stays their own act and is not a requirement here.

Add the recipe `just compare-fixtures`. It runs the port against the oracle on **each fixture in
the committed set** with **one** named configuration, calls `oracle/assert-placement` for each,
and reports the level reached per fixture.

**The configuration is one and is named in R22**: the Bambu Lab X1 Carbon 0.4 nozzle machine, the
`0.20mm Standard @BBL X1C` process and the `Bambu PLA Basic @BBL X1C` filament — the base FFF row
of `docs/ORACLE.md`'s configuration table, which plan step 2's proof spells `base-fff`. Unlike
task `41-compare-benchy` this command is held to **the G-code gate only**: R22 names that gate and
no other artifact.

**The fixture set comes from `vendor/README.md`'s fixture records**, which R25 makes the authority
on what a fixture is, **and it is enumerated through `oracle/assert-placement --list`** — the
listing mode task `11-fixture-set` adds to the script that already reads those records for each
fixture's resolved instance transform. The `fixtures` subcommand shells out to it and reads one line
per record: frozen basename, vendored path, transform, license. It does **not** parse
`vendor/README.md` itself. That is deliberate and it is the reason the listing mode exists: plan
step 6 emits a `.tsv` precisely "so that four separate tasks do not each write a Markdown parser
against a table they have never seen", and a Markdown-table parser inside `crates/oracle-compare`
would be the same mistake. At the pin the set is `3DBenchy.stl`, `20mm_cube.stl` and
`frog_legs.stl`, and a fixture added later by committing a row there is picked up without changing
this command.

**It adds no new selection logic.** Task `41-compare-benchy` commits the two functions in the
`gates` module of `crates/oracle-compare/src/lib.rs`; this command calls them: it reads the
gate from `port/reports/determinism.tsv` at run time, selects the oracle per fixture and
configuration from `port/reports/oracle-parity.tsv`'s `normative oracle` column, compares against
that oracle's goldens, and **prints which oracle was used on every line**, so a reader never has to
infer it. A fixture and configuration with no row in that file is a failure, not a default to the
release binary. That rule is stated once, in task `41-compare-benchy`, and this task implements no
second reading of it and no second parser for either `.tsv`.

The work is a `fixtures` subcommand of the binary in `crates/oracle-compare/src/main.rs`, beside
the `benchy` subcommand task `41-compare-benchy` adds, with the `justfile` recipe as orchestration
only. Per fixture it slices with the selected oracle — the pinned release binary through
`just oracle-slice <fixture> base-fff`, or the locally built library through `oracle/stage-dump`'s
export path where the parity report makes `built` normative — slices with the `grampus-cli` binary
from the base FFF resolved-configuration golden under `port/goldens/configs/`, calls
`oracle/assert-placement` with that fixture's `vendor/README.md` row, compares the G-code with
`crates/oracle-compare`'s levels, and prints one line.

**Add `just compare-fixtures` to the macOS arm64 job of `.github/workflows/ci.yml`.** That is this
step's own line and the task adds no other step's.

**The real work is in the reporting**, which is why this is a 3 and not smaller: three fixtures,
each with its own selected oracle, its own gate lookup, its own placement assertion and its own
reported level, in output a reader can act on — one line per fixture naming the level reached, the
gate it was held to and the oracle it ran against, and a non-zero exit that names which fixture
failed and why.

**When it lands.** Plan step 26 waits on step 25, so this task waits on
`41-compare-benchy` and on nothing else; everything step 25 waits for, including every task cut
from `38-phase-b-spike`'s `port/taskcuts/phase-b.tsv`, is already behind it. Nothing under
`vendor/OrcaSlicer/` changes.

## Done when

- `just compare-fixtures` exits 0 and prints one line per fixture in the committed set — three at
  the pin — each naming the level reached for the G-code, the gate it was held to and the oracle it
  ran against.
- Each fixture reaches at least the gate the base FFF `gcode` row of
  `port/reports/determinism.tsv` records, read at run time and not written into this command.
- `just compare-fixtures` exits non-zero, naming the fixture, when `oracle/assert-placement`
  reports a bounding box that fixture's `vendor/README.md` record does not name.
- `just compare-fixtures` exits non-zero, naming the file and the row it looked for, when a fixture
  and configuration has no row in `port/reports/oracle-parity.tsv`.
- No second reader of `port/reports/determinism.tsv` or `port/reports/oracle-parity.tsv` is added:
  `grep -rl 'determinism.tsv\|oracle-parity.tsv' crates/*/src/` still names only
  `crates/oracle-compare/src/lib.rs`, and this command calls the two `gates` functions task
  `41-compare-benchy` committed.
- The fixture set is enumerated through `oracle/assert-placement --list`, so it comes from
  `vendor/README.md`'s fixture records and not from a list
  written into the command.
- `cargo test -p oracle-compare` passes.
- The macOS arm64 job of `.github/workflows/ci.yml` has a step, added by this task, running
  `just compare-fixtures`, and no other line of that job is changed.
- `just check` and `just port-check` pass, `just guard` reports nothing, nothing under
  `vendor/OrcaSlicer/` is modified, and the branch writes none of the seven paths
  `docs/WORKFLOW.md` lists as the repository's own.
