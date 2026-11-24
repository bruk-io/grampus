---
kind: task
step: 10
delivers: [R6]
depends: [09-comparator-gcode-levels, 10-comparator-json, 11-fixture-set, 12-config-goldens, 13-build-libslic3r, 25-stage-dumper, 26-stage-dump-seams]
size: 5
---
# Task: Report whether the two oracles agree

## What to build

This is the whole of the plan's step 10. It answers, by measurement, whether the pinned v2.5.2
release binary and the locally built `libslic3r` produce the same output, and publishes the answer
in the form every later comparison reads at run time.

Add `oracle/parity` — a standalone executable with PEP 723 inline metadata and no file extension,
run through `uv`, like every other script under `oracle/` — and the recipe `just oracle-parity`,
which forwards to it.

**What it runs, stated per side rather than per golden**, because the two sides take their
configuration by different routes and neither can be handed the other's. For every configuration row
in `docs/ORACLE.md` and every fixture that row names, it slices twice, at the same instance
transform the fixture's `vendor/README.md` row records:

- **the pinned release binary**, through
  `oracle/slice <fixture> <configuration> --export-settings --export-statistics`, which resolves the
  configuration from that row's preset paths and overrides. There is no option that feeds a golden
  to that script — plan step 2 fixes its whole interface, and the golden is a run's *output*, not an
  input — so the release side is driven by the row, and the golden is what the release side would
  reproduce.
- **the locally built library**, through `oracle/stage-dump`'s export path — the program built
  against `vendor/OrcaSlicer/build/arm64` — run with the hook never registered and
  `slicing_pipeline_plugin` left empty, which is the unarmed run whose G-code task
  `25-stage-dumper` already proves identical at L1 to its armed run. For a configuration the pinned
  CLI can construct it is run as `oracle/stage-dump --config port/goldens/configs/<row>.json`, the
  golden task `12-config-goldens` commits. For a configuration `docs/ORACLE.md` marks as **not**
  constructible through the pinned CLI there is no golden, by construction — `12-config-goldens`
  produces one by running that CLI — so it is run as `oracle/stage-dump --set` with that row's
  `docs/ORACLE.md` overrides, and the row records that the release binary could not produce the
  artifact and no release-side run is attempted.

**The built oracle's run shape is always unarmed**, with the hook never registered. There is no
armed branch: an armed run would need the `slicing_pipeline_plugin` key exempted in the
`--export-settings` comparison, and this task compares that artifact byte for byte with nothing
exempted, so the two cannot both hold. The report still states the shape for each row, and the
answer is `unarmed` for every one of them.

**What it compares.** `crates/oracle-compare`, through its library rather than by re-parsing its
printed output: the G-code at L0 to L4, the `--export-settings` JSON byte for byte with no
normalisation, and the `--export-statistics` JSON with the single `generator` field normalised.

**What it writes.** `port/reports/oracle-parity.tsv` first, then
`port/reports/oracle-parity.md` generated from it — neither authored by hand. The `.tsv` is the
file the later comparison steps read at run time, so that four separate tasks do not each write a
Markdown parser against a table they have never seen. **Its header row is this, literally**, because
task `18-divergences-regeneration` reads two of these columns by name and fails on a rename, and
tasks 41 to 43 read them too:

```
configuration	fixture	artifact	level reached	normative oracle
```

`artifact` is one of `gcode`, `settings-json` and `statistics-json`, the same vocabulary
`port/reports/determinism.tsv` uses. `level reached` is the level the two oracles reach against each
other for that artifact.

**`normative oracle` is one value per configuration-and-fixture pair, repeated identically in each
of that pair's three artifact rows.** It is `release` only when **every** artifact row of the pair
reaches L1, and `built` otherwise. The granularity is stated because a level is reached per artifact
— the G-code can reach L1 while the statistics JSON does not — while the downstream reader is
per pair: task `41-compare-benchy` builds one helper that returns the normative oracle for a fixture
and a configuration, and tasks 42 and 43 reuse that one helper, so a pair with three disagreeing
cells would leave it no answer.

The Markdown report carries `oracle/provenance`'s header, called as
`oracle/provenance --oracle built --syntax markdown` with both of that script's required options
passed explicitly — `built`, because this report's own subject is the built oracle's flags, which it
names below. It names the built oracle's
exact CMake flags including `-ffp-contract=off` — read out of `oracle/build-libslic3r` rather than
retyped, so the two cannot drift — and lists every difference found, each with the first failing
location the comparator reported. It also states, as the recorded decision behind it has it, that
the built library is normative for every per-unit and per-stage proof, that the release binary is
normative for the end-to-end gates only where a row reaches L1, and that goldens are kept per
oracle for a row that says `built`.

**Three shapes a later command would otherwise break on, so this task settles them.** The
end-to-end comparison commands fail when a fixture and configuration have no row in this file, so
a row is written for every combination they will ask about rather than only for the pairs that ran:
a configuration `docs/ORACLE.md` marks as not constructible through the pinned CLI gets a row whose
level says the release binary could not produce it and whose `normative oracle` is `built`; an
artifact only one of the two oracles produces gets a row saying so rather than being absent; and a
run that fails on either side is a row and a non-zero exit, not a silent omission. Each of the
three is stated in the report in words as well as in the column, so a reader never infers it.

**What it must not touch.** It never edits `port/reports/divergences.md`. The rows this report
produces reach that file through the audit regeneration, which reads
`port/reports/oracle-parity.tsv` as one of its four named sources; writing them by hand here is
what a later regeneration would wipe. It also writes nothing into `port/goldens/` and changes no
other report.

Add `just oracle-parity` as its own step of the macOS arm64 job in `.github/workflows/ci.yml`,
touching no other line of that job. That job is created by the task that splits the checks between
the document job and the oracle job, with the oracle recipes left to the steps that make them pass;
if it is not on `main` when this task is built, say so in the pull request and let the line land
with whichever of the two is second, rather than writing the job here.

`oracle/provenance --check` covers `port/reports/oracle-parity.md` by name, so the generated header
has to satisfy it.

**Why this is a 5.** It runs both oracles over the fixture set from one resolved config, writes the
tsv whose `normative oracle` column three later steps read at run time, and names every difference.
The unknown is the result itself: a disagreement splits the goldens per oracle, which is a recorded
decision this task's measurement triggers rather than a question it answers. It is not split,
because the tsv and the report are generated from the same run and its CI line belongs with it.

**What it waits for.** The comparator's G-code and JSON levels; the frozen fixtures; the
resolved-configuration goldens; the built `libslic3r`; and the stage dumper with its seam
serialisation, which is the only export path the built oracle has. If the vendored library cannot
be built on this platform, this task stops and that is reported, as the recorded decision requires;
it is not worked around with the release binary alone.

## Done when

- `just oracle-parity` exits 0 and prints one line per configuration, fixture and artifact, each
  with the level the two oracles reach and which oracle that row makes normative.
- `port/reports/oracle-parity.tsv` is committed, its first line is exactly the header row written
  above — `configuration`, `fixture`, `artifact`, `level reached`, `normative oracle`,
  tab-separated — and it has a row for every configuration, fixture and artifact combination
  `docs/ORACLE.md` names, including the rows for a configuration the pinned CLI cannot construct and
  for an artifact only one oracle produces.
- `port/reports/oracle-parity.md` is committed, is generated from the `.tsv`, carries the
  provenance header, names the built oracle's exact CMake flags including `-ffp-contract=off`, and
  lists one entry per difference found with its first failing location.
- Every artifact row of a configuration-and-fixture pair carries the same `normative oracle` cell,
  and it is `release` exactly where all three of that pair's artifact rows reach L1 and `built`
  everywhere else.
- `oracle/provenance --check port/reports/oracle-parity.md` exits 0.
- Running `just oracle-parity` twice leaves both committed files unchanged apart from their
  timestamps.
- For every row whose configuration the pinned CLI can construct, the built oracle consumed that
  row's `port/goldens/configs/` golden and the release oracle ran that row's presets and overrides;
  for every row whose configuration it cannot, the built oracle ran `oracle/stage-dump --set` with
  that row's overrides and the row says the release binary could not produce the artifact. Both
  sides used the instance transform the fixture's `vendor/README.md` row records.
- The report states the built oracle's run shape for every row, and it is `unarmed` for every one of
  them; `grep -n 'armed' oracle/parity` shows no code path that registers the hook.
- The branch changes no line of `port/reports/divergences.md`, nothing under `port/goldens/`, and
  no file under `vendor/OrcaSlicer/`.
- `just oracle-parity` runs as its own step of the macOS arm64 job in
  `.github/workflows/ci.yml` — or, where that job is not yet on `main`, the pull request says so
  and names the line it will add.
