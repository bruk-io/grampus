---
kind: task
step: 25
delivers: [R21]
depends: [09-comparator-gcode-levels, 10-comparator-json, 11-fixture-set, 12-config-goldens, 27-oracle-determinism, 33-port-surface, 38-phase-b-spike, 28-oracle-parity]
size: 5
---
# Task: Slice the benchy with the port and with the oracle

## What to build

The whole of plan step 25, which delivers R21 and is this intent's acceptance gate: the benchy is
sliced by the port and by the oracle and the G-code is compared.

Add the recipe `just compare-benchy`. It slices `fixtures/3DBenchy.stl` with the port and with the
oracle, from the same resolved configuration and the same instance transform, and reports the
level reached for the G-code and for the statistics JSON. C3's decision is that this end-to-end
comparison is the acceptance gate rather than the first output, and that no throwaway bridge to
the vendored C++ is built to get an earlier one — the earlier signal is `just compare-stages`,
plan step 24.

**Where the pieces go.** The recipe in the `justfile` is orchestration only. Every judgement — the
gate lookup, the oracle selection, the comparison and the printed line — goes into
`crates/oracle-compare`, which plan step 5 already makes the one reader of a committed tolerance
table and the one place a level is decided:

- Two functions in `crates/oracle-compare/src/lib.rs`, in a `gates` module this task adds:
  one that returns the gate for a configuration and an artifact by reading
  `port/reports/determinism.tsv`, and one that returns the normative oracle for a fixture and a
  configuration by reading `port/reports/oracle-parity.tsv`. **Tasks `42-compare-fixtures` and
  `43-compare-configurations` call these two functions and add no second reader of either file**,
  which is why plan step 6 emits a `.tsv` at all: so that four separate tasks do not each write a
  parser against a table they have never seen.
- A `benchy` subcommand of the existing binary in `crates/oracle-compare/src/main.rs`, which the
  recipe invokes and which does the run: select the oracle, slice with it, slice with the port,
  assert the placement, compare, print. Steps 26 and 27 add a sibling subcommand each, over the
  same code.
- The G-code levels come from `src/levels.rs` and `src/gcode.rs` (task
  `09-comparator-gcode-levels`) and the statistics JSON from `src/json.rs` (task
  `10-comparator-json`). No level logic and no normalisation is written again here.

**What the run does.**

1. Resolve the base FFF configuration — the row `docs/ORACLE.md`'s configuration table names, whose
   presets are `machine/Bambu Lab X1 Carbon 0.4 nozzle.json`,
   `process/0.20mm Standard @BBL X1C.json` and `filament/Bambu PLA Basic @BBL X1C.json` under
   `vendor/OrcaSlicer/resources/profiles/BBL/`, and which plan step 2's proof spells `base-fff`.
2. Read the gate from `port/reports/determinism.tsv` **at run time**, for that configuration, for
   the two artifacts this comparison has: the `gcode` row and the `statistics-json` row. R21 holds
   the port to both. The `settings-json` row is not read, because the port consumes the oracle's
   resolved configuration rather than exporting one (R11). Reading the gate rather than asserting
   it is what makes this command move with the measurement: if task `27-oracle-determinism`
   measured the base FFF G-code gate weaker than L1, this command is held to that weaker level and
   nothing here needs changing.
3. Select the oracle per fixture and configuration from
   `port/reports/oracle-parity.tsv`'s `normative oracle` column, compare against that oracle's
   goldens, and **print which oracle was used on every line**, so a reader never has to infer it. A
   fixture and configuration with no row in that file is a failure, not a default to the release
   binary. That is C2's decision: the pinned release binary is normative for the end-to-end gates
   only where R6 shows the two oracles reach L1, the locally built library is normative for the
   fixture and configuration where they do not, and goldens are kept per oracle.
4. Slice with the selected oracle: the pinned release binary through
   `just oracle-slice 3DBenchy base-fff --export-statistics` (task `04-oracle-slice`), or the
   locally built library through `oracle/stage-dump`'s export path (tasks `25-stage-dumper` and
   `26-stage-dump-seams`) where the parity report makes `built` normative.
5. Slice with the port: the `grampus-cli` binary (task `33-port-surface`), given
   `fixtures/3DBenchy.stl` and the base FFF resolved-configuration golden under
   `port/goldens/configs/` (task `12-config-goldens`), which is the port's configuration input and
   the reason a benchy difference cannot be caused by preset resolution. Extend
   `crates/grampus-cli/src/main.rs`'s argument handling if a flag this run needs is missing; do not
   change the sequence `crates/grampus-cli/src/surface.rs` exposes and add nothing from
   `src/slic3r/GUI/PartPlate`.
6. Call `oracle/assert-placement` (task `11-fixture-set`) and exit non-zero when the oracle's run
   reports an object bounding box that `3DBenchy.stl`'s `vendor/README.md` record does not name.
   That record names one resolved instance transform, the identity, because the fixture is already
   positioned on the plate and `--arrange 0` leaves the Z-only `ensure_on_bed` as the oracle's only
   remaining placement step (R24).
7. Compare and print: for the G-code every level that passes, the strongest of them, the first
   failing location at the next stronger level, the gate it was held to and the oracle it ran
   against; for the statistics JSON the same, with the single `generator` field normalised.

**Add `just compare-benchy` to the macOS arm64 job of `.github/workflows/ci.yml`.** Plan step 16
creates that job with two lines and leaves every gate to the step that makes it pass; this is this
step's own line and the task adds no other step's.

**Run-time inputs this task does not generate.** `port/reports/determinism.tsv` is committed by
task `27-oracle-determinism` and `port/reports/oracle-parity.tsv` by task `28-oracle-parity`, and
**both of those tasks are in this one's `depends`** — `28-oracle-parity` is named there because
nothing else in this task's dependency chain reaches it, and this command's first "Done when" line
cannot be answered without a base-FFF `3DBenchy` row in that file. Both files are read at run time
and neither is written, regenerated or defaulted around here; a missing file
or a missing row is a failure with the file and the row named in the message.

The columns this command reads are the literal headers those two tasks commit: `gate` from
`determinism.tsv`, keyed on its `configuration`, `fixture` and `artifact` columns, and
`normative oracle` from `oracle-parity.tsv`, keyed on its `configuration` and `fixture` columns —
one value per pair, which is why the oracle lookup takes a fixture and a configuration and not an
artifact.

**What makes this a 5.** It is the acceptance gate, and the first command in the repository to
read both generated `.tsv` files at run time rather than from prose — the gate from
`determinism.tsv`, the normative oracle from `oracle-parity.tsv`, with a missing row a failure
rather than a default — on top of driving two slicers, the placement assertion and two artifact
comparisons. It is not split, because tasks `42-compare-fixtures` and `43-compare-configurations`
reuse exactly this gate-and-oracle selection and it has to exist once, in one place, with its own
tests.

**When it lands.** Plan step 25 waits on all of step 23, because R21 is not claimed while any
in-scope manifest row's derived status is other than `ported` (R26). This task depends on the spike
`38-phase-b-spike` and **also waits for every task cut from that spike's
`port/taskcuts/phase-b.tsv` in the later passes** — those tasks do not exist yet, and
`just port-status` without `--per-commit` passing is the signal that they are all merged. Nothing
under `vendor/OrcaSlicer/` changes.

## Done when

- `just compare-benchy` exits 0 and prints, for the G-code and for the statistics JSON, the level
  reached, the gate it read from `port/reports/determinism.tsv` and the name of the oracle it read
  from `port/reports/oracle-parity.tsv` and ran against.
- The level reached for the G-code is at least the gate the base FFF `gcode` row of
  `port/reports/determinism.tsv` records, and the level reached for the statistics JSON is at
  least that configuration's `statistics-json` gate.
- `just compare-benchy` exits non-zero when the oracle's run reports an object bounding box that
  `3DBenchy.stl`'s `vendor/README.md` record does not name, which `oracle/assert-placement`
  decides.
- `just compare-benchy` exits non-zero, naming the file and the row it looked for, when the fixture
  and configuration have no row in `port/reports/oracle-parity.tsv`, and when the configuration and
  artifact have no row in `port/reports/determinism.tsv`.
- The gate lookup and the oracle selection are two functions in the `gates` module of
  `crates/oracle-compare/src/lib.rs`, and
  `grep -rl 'determinism.tsv\|oracle-parity.tsv' crates/*/src/` names that file alone, so tasks
  `42-compare-fixtures` and `43-compare-configurations` have one place to call. The grep is scoped
  to the crates' sources because the tests below load committed `.tsv` fragments by name and assert
  on the failure messages, so both strings necessarily appear under
  `crates/oracle-compare/tests/` as well.
- `cargo test -p oracle-compare` passes, including tests over committed `.tsv` fragments under
  `crates/oracle-compare/tests/data/` that cover: a gate read for each of the two artifacts, a
  `normative oracle` of `release`, one of `built`, and a missing row in each file.
- The macOS arm64 job of `.github/workflows/ci.yml` has a step, added by this task, running
  `just compare-benchy`, and no other line of that job is changed.
- `just port-status` without `--per-commit` exits 0 and prints `0 in-scope rows remaining`, which
  is what R26 requires before R21 is claimed.
- `just check` and `just port-check` pass, `just guard` reports nothing, nothing under
  `vendor/OrcaSlicer/` is modified, and the branch writes none of the seven paths
  `docs/WORKFLOW.md` lists as the repository's own.
