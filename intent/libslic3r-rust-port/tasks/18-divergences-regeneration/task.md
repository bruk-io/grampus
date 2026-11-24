---
kind: task
step: 13
delivers: [R18]
depends: [17-order-audits]
size: 3
---
# Task: Regenerate divergences.md from its four named sources

## What to build

This is part 2 of step 13 of `intent/libslic3r-rust-port/plan.md`, "The order-dependence audits",
which is why its `delivers` is the whole step's. Part 1, `17-order-audits`, added `port/audit-order`
and the two static-analysis reports. This task extends that same script with
`port/reports/divergences.md`, and adds `port/regenerate`, `just regenerate` and `just audit-check`.

**What it waits for.** `17-order-audits`, for `port/audit-order`, `just audit-order` and the two
committed reports. It waits for nothing else, and two of the four sources below do not exist yet by
design.

**Extend `just audit-order` to regenerate `port/reports/divergences.md`**, which is fully derived
and never hand-edited. **Each row carries, as its first cell, which of the four sources below
contributed it** — `order-audit`, `fidelity-marker`, `oracle-parity` or `determinism` — then the
place that makes the port differ from the oracle, the level it is checked at instead, and, for the
`determinism` rows, the configuration, **fixture** and artifact whose gate is `none` — all three,
because `determinism.tsv` has one row per configuration, fixture and artifact and a configuration
may name two fixtures whose gates differ, so a pair alone would not identify the row. The per-row
source cell is not decoration: `23-perf-backlog` asserts that every `fidelity-marker` row still has
its marker in the tree, and rows from the other three sources have no marker by design, so without
that cell its assertion would fail on every one of them; and `43-compare-configurations` matches a
reported `none` gate against a row of this file on that configuration, fixture and artifact triple,
which is why the `determinism` rows carry all three. **Its sources are exactly these four and no step contributes to it
any other way:**

1. The rows of `port/reports/order-containers.md` and `port/reports/order-comparators.md` that the
   same run just wrote, whose diverges-from-the-oracle cell is yes. Read them from the run's own
   output, not from the committed files, so one run cannot mix two generations.
2. Every `// FIDELITY(port):` marker in the port tree under `crates/`, which is how a module
   declares a site. No ported module exists at this point, so this source contributes zero rows
   today; the grep has to work and report zero rather than fail on an empty or absent tree.
3. `port/reports/oracle-parity.tsv`, every row whose two oracles do not reach L1. That file is
   written by `28-oracle-parity`, which does not depend on this task, so it is absent when this task
   runs.
4. `port/reports/determinism.tsv`, every row whose gate is `none`. That file is written by
   `27-oracle-determinism`, likewise absent when this task runs.

**The care that makes this a task and not a line: a regeneration must not wipe the rows sources 3
and 4 contribute.** `28-oracle-parity` and `27-oracle-determinism` each write their own report and
never edit `divergences.md`, so if a later run of `just audit-order` reads their `.tsv` files wrongly
and contributes nothing, their divergences disappear from the one file that is supposed to name every
one of them, with nothing failing. Two rules in the script prevent that, and they have to be kept
apart:

- **An absent source contributes zero rows and is not an error.** That is the state on this branch
  for sources 3 and 4, and the state for source 2 until the first module lands.
- **A source that is present but unreadable is a failure.** The script reads the two `.tsv` files by
  column name and exits non-zero naming the column it could not find, rather than treating a column
  rename as zero rows. **The three header strings are literal and fixed here**, because the two
  tasks that write those files are built in separate worktrees and neither depends on this one:
  `port/reports/oracle-parity.tsv` carries `level reached` and `normative oracle`, and
  `port/reports/determinism.tsv` carries `gate`. The `normative oracle` spelling, with a space, is
  the plan's own; `level reached` and `gate` are fixed here because the plan describes those columns
  without giving a header string. `27-oracle-determinism` and `28-oracle-parity` write exactly these
  headers and their task files say so, so what this rule catches is a rename after the fact rather
  than a disagreement at birth.

And `divergences.md` carries a section that names all four sources and, for each, whether it was
present and how many rows it contributed. A run that lost rows because a file was missing is then
visible in the committed file itself rather than only in the absence of rows nobody is counting.

**Add `port/regenerate` and `just regenerate`**, the one command that rewrites the generated
reports, so there is a single place that does it. Here it runs `just audit-order`.
`23-perf-backlog`, part 3 of step 14, adds the perf-backlog regeneration to this same script — leave
that place for it and do not write the perf-backlog half, which needs the `// PERF(port):` markers
`21-port-status-markers` defines.

**Add `just audit-check`**, the drift-detecting variant: it regenerates all three reports —
`order-containers.md`, `order-comparators.md` and `divergences.md` — into a scratch directory and
fails when a committed one differs. **It is not a check a port task passes, and this task must not
make it one.** A whole-file diff of a generated report cannot be: a hundred task branches each
adding a `// FIDELITY(port):` marker would each commit a regeneration that the next merge makes
stale. Step 13 puts `just audit-check` on the regeneration pass that runs on `main`, which
`23-perf-backlog` defines, and what a port task passes instead is that step's per-marker assertion.
So `just audit-check` goes in the `justfile` and into neither `.asdlc/checks` nor `just port-check`.

Because `just audit-check` diffs whole files, `divergences.md` is byte-stable on the same terms part
1 set for the other two: rows sorted deterministically, paths relative to the repository root, no
timestamp and no machine-specific path.

**What makes this a 3.** The four sources are named in the plan, the rule for each is mechanical, and
there is no judgement about the tree left to make — part 1 made it. The real work is the pair of
rules above and a test for each, plus the three-report `audit-check` and the `port/regenerate`
seam that `23-perf-backlog` extends.

**What this task does not do.** It does not change part 1's two reports or the rules that produce
them. It does not write `port/reports/perf-backlog.md`. It adds no line to `.asdlc/checks` and none
to `just port-check`. `divergences.md` carries no provenance header: step 3 of the plan excludes it
by name from what `oracle/provenance --check` covers and that check fails if it is pointed at one, so
do not add a header and do not add the path to the provenance check's list. It does not edit
`port/manifest.tsv` and changes nothing under `vendor/OrcaSlicer/`. When
`27-oracle-determinism` and `28-oracle-parity` land, their rows reach `divergences.md` through the
regeneration pass; nothing here is edited for them.

## Done when

- `just audit-order` regenerates `port/reports/divergences.md` as well as the two reports part 1
  added, and `port/reports/divergences.md` is committed.
- `just audit-order` **prints** the row count of `divergences.md` and the count it took from each of
  the four named sources, which together with part 1's two row counts is the whole of the plan's
  recorded proof line for R18.
- `just audit-check` exits 0 and **prints nothing** on a clean tree, which is the other half of that
  proof line.
- The three header strings the script reads are the literals `level reached`, `normative oracle` and
  `gate`: `grep -n 'level reached\|normative oracle\|gate' port/audit-order` finds them, and they
  are the spellings `27-oracle-determinism` and `28-oracle-parity` commit.
- `port/reports/divergences.md` carries a section naming all four sources and, for each, whether it
  was present and how many rows it contributed.
- Every row of `port/reports/divergences.md` carries a first cell holding one of `order-audit`,
  `fidelity-marker`, `oracle-parity` or `determinism`, and every `determinism` row additionally
  names the configuration, the fixture and the artifact whose gate is `none`.
- A fresh clone at this commit regenerates all three reports byte for byte, so no cell of
  `divergences.md` is hand-written, and running `just audit-order` twice leaves all three
  byte-identical.
- With `port/reports/oracle-parity.tsv` and `port/reports/determinism.tsv` absent — which is the
  state of this branch — `just audit-order` exits 0 and `divergences.md` records both sources as
  absent with zero rows.
- The pull request shows that dropping in an `oracle-parity.tsv` holding one row whose oracles do not
  reach L1 and a `determinism.tsv` holding one row whose gate is `none`, then re-running
  `just audit-order`, adds exactly those two rows and removes no row contributed by sources 1 and 2.
- The pull request shows that with one of those two files present but its `level reached`,
  `normative oracle` or `gate` column renamed, `just audit-order` exits non-zero and names the
  column it could not find.
- A `// FIDELITY(port):` marker added anywhere under `crates/` produces a row in
  `divergences.md`; with no such marker in the tree the source contributes zero rows and the run
  still exits 0. **The file carrying that marker is an uncommitted scratch file**: `crates/` is
  created by `08-comparator-workspace`, which this task does not depend on, so the demonstration
  goes in the pull request body and this branch commits nothing under `crates/`.
- `port/regenerate` exists as a `uv`-run standalone script with PEP 723 inline metadata,
  `just regenerate` is a recipe that runs it, and it runs `just audit-order`; no perf-backlog
  regeneration is written here.
- `just audit-check` is a recipe that regenerates all three reports into a scratch directory, exits 0
  against the committed tree, and exits non-zero after a single byte is changed in any one of them.
- `just audit-check` appears in neither `.asdlc/checks` nor `just port-check`.
- `port/reports/divergences.md` carries no provenance header and its path is not added to what
  `oracle/provenance --check` covers.
- `port/reports/order-containers.md` and `port/reports/order-comparators.md` change only as a
  regeneration by the extended script, and the rules part 1 wrote for them are unchanged.
- No `port/reports/perf-backlog.md` is written, `port/manifest.tsv` is unchanged, nothing under
  `vendor/OrcaSlicer/` changed, and the submodule pointer is still
  `f92a7c065c5718b8ef1baac1cd704df2911238f4`.
- The branch changes no file under `intent/`, so `just guard` passes, and `just check` passes.
