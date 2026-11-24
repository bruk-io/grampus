---
kind: task
step: 6
delivers: [R3]
depends: [04-oracle-slice, 05-oracle-provenance, 09-comparator-gcode-levels, 10-comparator-json, 11-fixture-set, 25-stage-dumper]
size: 5
---
# Task: Measure the oracle against itself and publish the gate table

## What to build

The whole of plan step 6, "Measure the oracle against itself and publish the gate table", in
`intent/libslic3r-rust-port/plan.md`, delivering R3 in `intent/libslic3r-rust-port/spec.md`. Read
both, and R4 for the levels and the per-artifact normalisation.

This is where the gate every later comparison reads comes from. Tasks `41-compare-benchy`,
`42-compare-fixtures` and `43-compare-configurations` do not assert a level; they read the one this
task measures.

Files this task adds or changes:

- `oracle/determinism` — a standalone executable with PEP 723 inline metadata and no file
  extension, run through `uv`, as every script outside the Rust workspace in this repository is.
- `justfile` — gains `just oracle-determinism`.
- `port/reports/determinism.tsv`
- `port/reports/determinism.md`
- `docs/ORACLE.md` — gains the gate table.

Files this task leaves alone. `port/reports/divergences.md`: this report's `none` rows reach that
file because `port/reports/determinism.tsv` is one of the four named sources task
`18-divergences-regeneration` regenerates it from, so a row written here would be wiped by the next
regeneration. Nothing under `vendor/OrcaSlicer/`; the submodule pointer stays at
`f92a7c065c5718b8ef1baac1cd704df2911238f4`. `.github/workflows/ci.yml`, because step 6 adds no CI
line.

### the runs

For each configuration named in the configuration table of `docs/ORACLE.md` — base FFF, tree
support, wipe tower with multi-filament, fuzzy skin, and any other the port needs — and for each
fixture that row names, run the oracle at least five times. Five per configuration is the minimum,
not the target.

Each run goes through task `04-oracle-slice`'s interface and no other flag:

```
just oracle-slice <fixture> <configuration> --export-settings --export-statistics --out <dir>
```

Each run gets a fresh `--out`. `--datadir` is deliberately not selectable: `oracle/slice` always
makes a fresh one inside its own scratch directory, which is what keeps the seven `printhost_*`
credential fields of an exported configuration empty, so this script must not reach around that.

### the comparison

Compare every pair of runs, which is ten pairs for five runs, for each of the three artifacts: the
G-code with the comparator task `09-comparator-gcode-levels` completes, through `just compare <a>
<b>`; the resolved-configuration JSON and the statistics JSON with task `10-comparator-json`'s
comparison, each with the normalisation R4 declares for it and no other.

An artifact's gate is the strongest level every pair reaches, and `none` where no level is reached by
every pair. A gate of `none` is a measurement, not a failure of this command, and the command still
exits 0.

**The level is read from `--format json` and never from an exit code.** Task
`08-comparator-workspace` states that a script of this kind must not parse prose and that
`just compare` without `--require` exits 0 at every level including `none`, so this script passes
`--format json` and reads the level out of the object. Where a JSON arm of
`just compare --settings` or `just compare --statistics` exits non-zero on a `none` result, that
exit is recorded as a `none` gate and not propagated: the only non-zero exits this script passes on
are a usage error and an unreadable file.

### the two generated files

`port/reports/determinism.tsv` is the file tasks 41, 42 and 43 read at run time, which is why it
exists beside the Markdown: four separate tasks must not each write a Markdown parser. **Its header
row is this, literally**, because task `18-divergences-regeneration` reads the `gate` column by that
name and fails on a rename, and tasks 41 to 43 read the rest:

```
configuration	fixture	artifact	run count	normalisation	gate	oracle
```

`artifact` is one of `gcode`, `settings-json` and `statistics-json`; `gate` is `L0` to `L4` or
`none`; `oracle` is `release` or `built`. **There is one row per configuration, fixture and
artifact** — the `fixture` column is there because a configuration row in `docs/ORACLE.md` names the
fixtures it uses, in the plural, and the five runs are per fixture, so a per-configuration row would
have to aggregate gates across fixtures and no aggregation rule would be a measurement of anything.
The sibling report of task `28-oracle-parity` is keyed the same way for the same reason.

`port/reports/determinism.md` is generated from the `.tsv` and is what a person reads: one row per
configuration and fixture, one sub-row per compared artifact, each sub-row naming the normalisation R4 declares
for that artifact and recording whether the runs were byte-identical with no normalisation, with
that normalisation, or not at all. Neither file is authored by hand, and re-running the command
leaves both unchanged apart from their timestamps.

The Markdown report carries the provenance header from `oracle/provenance` (task
`05-oracle-provenance`), called as `oracle/provenance --oracle <release|built> --syntax markdown`
with both of that script's required options passed explicitly — `release` for the rows the pinned
binary produced and `built` for the rows task `25-stage-dumper`'s driver produced, matching the
row's own `oracle` column — and it formats no header of its own. The static-analysis reports are excluded from that
check by name; this one is covered by it.

`docs/ORACLE.md` gains the gate table, generated from the report with the same per-configuration and
per-artifact breakdown, each row naming the oracle that produced it. It is generated, not authored.

### what this task must not pre-decide

The base FFF row is where the whole design's claim that L1 is the gate comes from. It is produced by
this command and quoted from nowhere — not from the spec, not from the plan, not from the informal
runs the spec's design section mentions and says nothing in this repository records. Do not write a
level into any of the three files by hand, and do not shape the measurement to reach one. If the base
FFF G-code row comes back weaker than L1, that is the answer, and tasks 41 to 43 move with it; say so
plainly in the pull request.

Fuzzy skin is expected to land at `none`, because its displacement generator is a `thread_local`
`std::mt19937` seeded from `std::random_device`, in `random_value()`
(`vendor/OrcaSlicer/src/libslic3r/Feature/FuzzySkin/FuzzySkin.cpp:37-43`), so its displaced points
vary per run and per scheduling and no level is reachable, L4's identical move count included.
Record what is measured either way.

### the configurations the pinned CLI may refuse

C12's decision, which is why this task depends on `25-stage-dumper`: a configuration the pinned CLI
cannot construct is run through the built-oracle driver that task added, **in its `--set` mode
together with that configuration's `docs/ORACLE.md` overrides**, which writes the same three
artifacts, and its rows' `oracle` column says which oracle produced them. The driver's other mode,
`--config` over a golden under `port/goldens/configs/`, cannot serve these rows: task
`12-config-goldens` produces a golden by running the pinned CLI with `--export-settings`, so a
configuration the CLI refuses has no golden by construction. Do not add a dependency on
`12-config-goldens` for this; it would be a dependency on a branch that cannot exist. A configuration neither can construct is recorded with a
gate of `none` for every artifact and the reason in the row, and is reported rather than worked
around.

Only those rows wait on task 25. The base FFF rows need tasks 04, 05, 09, 10 and 11 alone, so do not
block the whole report on the dumper.

### what makes this a 5

The gate every later comparison reads comes from here: at least five runs per configuration over
three artifacts, pairwise comparison, a `.tsv` and a report that regenerate identically, and a gate
table in `docs/ORACLE.md`.

It is not split. The runs, the pairwise evaluation and the two generated files are one pipeline — the
`.tsv` is the pipeline's output and the report and the gate table are derived from it — and the rows
that need the built-oracle driver are that same pipeline with a different driver, not separate work.

## Done when

- `just oracle-determinism` exits 0 and prints one line per configuration, fixture and artifact with
  its run count and its gate.
- `port/reports/determinism.tsv` is committed, its first line is exactly the header row written
  above — `configuration`, `fixture`, `artifact`, `run count`, `normalisation`, `gate`, `oracle`,
  tab-separated — it has one row per configuration, fixture and artifact, and no row has a run count
  below five.
- `port/reports/determinism.md` is committed, is generated from the `.tsv`, and has one row per
  configuration and fixture with a sub-row per artifact naming that artifact's normalisation and
  whether the runs matched with none, with it, or not at all.
- `oracle/provenance --check port/reports/determinism.md` exits 0.
- Running `just oracle-determinism` a second time leaves both committed files unchanged apart from
  their timestamps: a reviewer runs it and `git diff` shows timestamp lines and nothing else.
- `docs/ORACLE.md` holds the gate table with the same per-configuration and per-artifact breakdown,
  generated from the report, each row naming the oracle that produced it.
- A reviewer reads the base FFF G-code row and finds a run count of at least five and a gate this
  command measured.
- Every row's `oracle` column names the oracle that produced its runs, and a row produced through
  task `25-stage-dumper`'s driver says so.
- A row with a gate of `none` is reported without failing the command, and a reviewer can see one.
- Every invocation of the **pinned release binary** goes through task `04-oracle-slice`'s interface:
  a reviewer greps the script and finds `just oracle-slice` with the fixture, the configuration,
  `--export-settings`, `--export-statistics` and `--out`, and no other flag and no `--datadir`.
- Every invocation of the **built oracle** goes through task `25-stage-dumper`'s driver in its
  `--set` mode with that configuration's `docs/ORACLE.md` overrides, and the script invokes no
  `--config` mode and no oracle any other way.
- The script reads every level from `just compare ... --format json` and never from an exit code:
  `grep -n 'format json' oracle/determinism` finds it, and a `none` result from a JSON arm is
  recorded as a `none` gate rather than failing the run.
- The branch writes no row into `port/reports/divergences.md`, changes no file under
  `vendor/OrcaSlicer/` — `git status --porcelain vendor/OrcaSlicer` prints nothing — and leaves
  `.github/workflows/ci.yml` untouched.
