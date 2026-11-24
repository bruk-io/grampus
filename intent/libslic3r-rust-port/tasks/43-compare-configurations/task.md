---
kind: task
step: 27
delivers: [R27]
depends: [18-divergences-regeneration, 27-oracle-determinism, 42-compare-fixtures]
size: 3
---
# Task: Compare the port and the oracle on every configuration

## What to build

This is the whole of the plan's Step 27, "Every configuration", which delivers R27.

Add `just compare-configurations`, which runs the port against the oracle for **each configuration
named in the configuration table of `docs/ORACLE.md`** — base FFF, tree support, wipe tower with
multi-filament, and fuzzy skin — and reports, for each of the three compared artifacts, the level
reached.

**Two artifacts are compared, not three: `gcode` and `statistics-json`.** The G-code is
`plate_1.gcode` and the statistics JSON is `plate_1.statistics.json`, compared by
`crates/oracle-compare/src/json.rs`. The **`settings-json` row of
`port/reports/determinism.tsv` is not read**, for the reason task `41-compare-benchy` gives for the
same decision: the port consumes the oracle's resolved configuration rather than exporting one
(R11), so there is no port-side settings JSON to compare, and that configuration's keys are already
compared inside the G-code's `CONFIG_BLOCK`, which R11 puts in scope and which the port writes.
`determinism.tsv`'s three-artifact breakdown comes from step 6, which compares the oracle against
*itself*; a port-versus-oracle run is not that. Do not add an `--export-settings` to
`crates/grampus-cli` to fill the gap: R10 fixes the port's surface at five calls and R11 puts
preset resolution and configuration export out of scope.

**One line per configuration, artifact and fixture named in the row.** The enumeration is: for each
configuration row of `docs/ORACLE.md`, for each fixture that row names, for each of the two
artifacts. R25's own words are that every configuration row "names the fixtures it uses", in the
plural, so the count is not fixed here. The two lookups are keyed differently and the enumeration
has to respect both: the **gate** is read on the configuration, fixture and artifact, and the
**normative oracle** on the configuration and fixture.

**Where the command lives.** The recipe goes in `justfile`. The plan's "Files that change" names no
new script under `oracle/` for this command, so the driver is a subcommand of the comparator's own
binary, `crates/oracle-compare/src/main.rs`, which `just compare <a> <b>` is already wired to.
**The subcommand is `configurations`**, beside the `benchy` subcommand task `41-compare-benchy`
adds and the `fixtures` subcommand task `42-compare-fixtures` adds, so the three cannot collide in
`main.rs`. It **calls the two functions of the `gates` module in
`crates/oracle-compare/src/lib.rs`** that task `41-compare-benchy` adds — the gate lookup for a
configuration, fixture and artifact, and the normative-oracle lookup for a configuration and
fixture — and adds no reader of either `.tsv` of its own. That is not a preference: task
`41-compare-benchy` is reviewed against the line that
`grep -rl 'determinism.tsv|oracle-parity.tsv' crates/*/src/` names
`crates/oracle-compare/src/lib.rs` alone, and a reader added in `main.rs` here would make that line
false on merge. Do not read a Markdown table where a `.tsv` exists.

**The oracle run.** Each configuration and fixture is sliced through
`just oracle-slice <fixture> <configuration> --export-statistics --out <dir>`, using a directory the
subcommand owns per run. `--out` is part of step 2's documented interface, so passing it is not
"another flag": what the plan forbids is a flag **outside** that interface, and without `--out` the
output lands in a scratch directory step 2 does not say it prints, leaving the driver no stated way
to find `plate_1.gcode` and `plate_1.statistics.json`. `--export-settings` is not passed, because
the settings JSON is not compared here. A configuration that row marks as not
constructible through the pinned CLI is run instead through Step 8's built-oracle configuration
driver (`oracle/stage-dump`), which is C12's accepted fallback; do not work such a row around by
patching presets.

**The port run.** The port is `crates/grampus-cli`'s binary, taking that configuration's resolved
configuration from `port/goldens/configs/` — the port's configuration input is the oracle's
resolved config and never preset JSON — and the one instance transform the fixture's
`vendor/README.md` record names.

**Which oracle, read and never hardcoded.** Select the oracle per configuration from the
`normative oracle` column of `port/reports/oracle-parity.tsv` (`release` or `built`), for that
configuration and the fixture its row names, compare against that oracle's goldens, and **print
which oracle was used on every line**, by the same rule the plan states for Step 25. A
configuration and fixture with no row in that file is a failure, not a default to the release
binary. The file is the one `just oracle-parity` commits in Step 10.

**Which gate, read and never hardcoded.** For each configuration and artifact, read the `gate`
column of `port/reports/determinism.tsv` (`L0` to `L4`, or `none`) and fail unless the level the
comparator reports for that cell reaches it. A configuration and artifact with no row in that file
is a failure. The command holds the port to what Step 6 measured and to nothing else, so if Step 6
measured a gate weaker than `L1` this command moves with it.

**A gate of `none` is reported, not failed.** Where the gate is `none` the command prints the cell
as a named divergence and keeps going, and the run can still exit 0. That is C1's recorded
decision: the pin holds, the oracle is not patched, the port is made deterministic, and each
divergence is named and checked at the level Step 6 measured instead of being chased to byte
parity. Fuzzy skin is expected here, because its displacement generator is seeded per thread from
`std::random_device` (`vendor/OrcaSlicer/src/libslic3r/Feature/FuzzySkin/FuzzySkin.cpp:37-44`).

**A reported divergence with no row in `port/reports/divergences.md` is a failure**, so the command
and that report cannot drift apart. **The match is on the configuration, fixture and artifact whose
gate is `none`**, which is exactly what source 4 of Step 13's regeneration contributes: task
`18-divergences-regeneration` writes a per-row source cell and, for its `determinism` rows, the
configuration, fixture and artifact that row came from, so this command matches on that triple and
fails naming it when no row holds it. All three are needed because a configuration may name two
fixtures whose gates differ. The command **writes no row into `divergences.md` itself**: the
`none` gates it reports reach that file because `port/reports/determinism.tsv` is the fourth of the
four named sources `just audit-order` regenerates from in Step 13, which is what makes those rows
survive a regeneration. If a divergence has no row, the fix is a regeneration pass on `main`, not
an edit here.

Any tolerance the comparator applies comes only from the configuration's `docs/ORACLE.md` row,
which must carry reason text for it, and the comparator prints which tolerance it used; this
command adds no tolerance of its own.

**CI.** This step adds exactly one line, `just compare-configurations`, to the **macOS arm64 job**
of `.github/workflows/ci.yml`. It needs the oracle, so it does not go in `.asdlc/checks` or in
`just port-check`, which must keep passing on a machine with no oracle binary and no libslic3r
build. Step 16 created that job with two lines and named the eight later steps that each add one,
so a conflict there is an ordinary merge conflict with a sibling comparison task, not a mechanism
problem.

**What this task must not touch.** Nothing under `vendor/OrcaSlicer/`, and not the submodule
pointer. It does not edit `port/reports/divergences.md`, `port/reports/determinism.tsv`,
`port/reports/oracle-parity.tsv` or the generated gate table in `docs/ORACLE.md` — all four are
generated by other commands and read here. It does not change `oracle/slice`, the comparator's
levels, or `just compare-benchy` and `just compare-fixtures`.

**What it waits for beyond its `depends`.** `port/goldens/configs/` comes from Step 9 (task
12-config-goldens) and `port/reports/oracle-parity.tsv` from Step 10 (task 28-oracle-parity); both
are reached transitively, through 42-compare-fixtures to 41-compare-benchy, which names
`28-oracle-parity` in its own `depends` for exactly this reason. The command can only exit 0 once the port
actually slices, so it also waits for the Phase B compile tasks cut from
`port/taskcuts/phase-b.tsv` in later passes, after which `just port-status` passes with
`0 in-scope rows remaining`.

## Done when

- `just compare-configurations` exists in `justfile` and runs the port and the oracle for all four
  configurations in `docs/ORACLE.md`'s configuration table: base FFF, tree support, wipe tower with
  multi-filament, and fuzzy skin.
- It prints one line per configuration, artifact and fixture named in that configuration's
  `docs/ORACLE.md` row, over the two artifacts `gcode` and `statistics-json`, and each line names
  the level reached, the gate it was held to, and the oracle it ran against.
- It reads no `settings-json` row of `port/reports/determinism.tsv`, and
  `crates/grampus-cli` gains no `--export-settings`: a reviewer greps the subcommand for
  `settings-json` and finds only the comment saying why it is not compared.
- It exits 0 on a tree where every configuration and artifact reaches the gate
  `port/reports/determinism.tsv` records for it.
- It exits non-zero, naming the cell, when a configuration and artifact reaches a level weaker than
  its gate in `port/reports/determinism.tsv`.
- A cell whose gate is `none` is printed as a named divergence and does not make the command exit
  non-zero; the fuzzy-skin run demonstrates this.
- It exits non-zero, naming the configuration, fixture and artifact, when a divergence it reports
  has no row in `port/reports/divergences.md` carrying that triple — which a reviewer can see by
  removing that row and re-running.
- It exits non-zero when a configuration and artifact has no row in
  `port/reports/determinism.tsv`, and when a configuration and fixture has no row in
  `port/reports/oracle-parity.tsv` — neither falls back to the release binary.
- The oracle each line names is the `normative oracle` value from `port/reports/oracle-parity.tsv`
  for that configuration and fixture, and a reviewer can change that cell from `release` to `built`
  and see the printed oracle change with it.
- A run leaves `port/reports/divergences.md`, `port/reports/determinism.tsv` and
  `port/reports/oracle-parity.tsv` byte-identical, which `git status` after a run shows.
- A configuration marked in `docs/ORACLE.md` as not constructible through the pinned CLI is run
  through Step 8's built-oracle configuration driver, and its lines say so.
- `.github/workflows/ci.yml`'s macOS arm64 job has a step running `just compare-configurations`,
  and `.asdlc/checks` and `just port-check` are unchanged.
- Nothing under `vendor/OrcaSlicer/` changes and the submodule pointer is still
  `f92a7c065c5718b8ef1baac1cd704df2911238f4`.
- The subcommand is named `configurations`, and
  `grep -rl 'determinism.tsv\|oracle-parity.tsv' crates/*/src/` still names
  `crates/oracle-compare/src/lib.rs` alone after this task, so it added no reader of its own.
- The oracle invocation is `just oracle-slice <fixture> <configuration> --export-statistics
  --out <dir>` and no flag outside step 2's interface:
  `grep -n 'oracle-slice' crates/oracle-compare/src` shows that line and no `--datadir`.
- `cargo test -p oracle-compare` passes, including tests over committed `.tsv` and `.md` fragments
  under `crates/oracle-compare/tests/data/` covering: a gate of `none` reported without failing, a
  missing `determinism.tsv` row, a missing `oracle-parity.tsv` row, and a reported divergence with
  no `divergences.md` row for that configuration, fixture and artifact — so each of the failure
  modes above
  has an automated counterpart beside the hand-mutation check.
- `just port-status` without `--per-commit` exits 0 and prints `0 in-scope rows remaining`, which is
  what the port actually slicing depends on and what this command's exit 0 rests on.
- `just check` and `just port-check` both pass.
