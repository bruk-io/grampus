---
kind: plan
---
# Plan: Port libslic3r to Rust with 1:1 functionality

Every area of concern in the spec, C1 to C14, carries a recorded Decision, so **no step in this
plan waits on a concern**. Where a decision changes the order of work it is named in the step
that carries it out.

Four steps cannot be sized until earlier work is on `main`, because their unit lists come out of
`port/manifest.tsv` or the vendored headers and neither artifact exists yet: Step 12 (the
ownership manifest), Step 20 (the foundation types), Step 22 (Phase A drafts) and Step 23 (Phase
B compiles). Each of those steps says which column orders its units, how many units go in a task,
and what a task's "Done when" is, and each is cut spike-first: the first task commits the step's
own cut file, and the rest of the step is cut from that file in a later pass. Because one draft
message has to carry every task's file in full, a phase with more rows than that holds is cut in
several passes, one pass per crate-level level, and each step says so.

The research notes behind the spec live outside this repository and may disappear, so every
command, flag, asset name, hash and build trap this plan relies on is written out here.

## Files that change

- Cargo.toml (new)
- Cargo.lock (new)
- rust-toolchain.toml (new)
- .gitignore
- justfile
- .asdlc/checks
- .github/workflows/ci.yml
- README.md
- vendor/README.md
- docs/ORACLE.md (new)
- docs/PORTING.md (new)
- docs/WORKFLOW.md (new)
- oracle/fetch-release (new)
- oracle/slice (new)
- oracle/provenance (new)
- oracle/export-fixture (new)
- oracle/assert-placement (new)
- oracle/determinism (new)
- oracle/build-libslic3r (new)
- oracle/parity (new)
- oracle/stage-dump/CMakeLists.txt (new)
- oracle/stage-dump/dump_main.cpp (new)
- oracle/stage-dump/dump_seams.cpp (new)
- oracle/stage-dump/dump_seams.hpp (new)
- oracle/stage-dump/README.md (new)
- fixtures/3DBenchy.stl (new)
- fixtures/20mm_cube.stl (new)
- fixtures/frog_legs.stl (new)
- crates/oracle-compare/Cargo.toml (new)
- crates/oracle-compare/src/lib.rs (new)
- crates/oracle-compare/src/gcode.rs (new)
- crates/oracle-compare/src/levels.rs (new)
- crates/oracle-compare/src/json.rs (new)
- crates/oracle-compare/src/stages.rs (new)
- crates/oracle-compare/src/main.rs (new)
- crates/oracle-compare/tests/levels.rs (new)
- crates/oracle-compare/tests/normalisation.rs (new)
- crates/oracle-compare/tests/json.rs (new)
- crates/oracle-compare/tests/data/ (new)
- crates/slic3r-ffi/Cargo.toml (new)
- crates/slic3r-ffi/build.rs (new)
- crates/slic3r-ffi/src/lib.rs (new)
- crates/slic3r-ffi/src/clipper2.rs (new)
- crates/slic3r-ffi/src/admesh.rs (new)
- crates/slic3r-ffi/src/qhull.rs (new)
- crates/slic3r-ffi/src/libnest2d.rs (new)
- crates/slic3r-ffi/src/voronoi.rs (new)
- crates/slic3r-ffi/src/proof.rs (new)
- crates/slic3r-ffi/cxx/proof_shims.cpp (new)
- crates/slic3r-ffi/cxx/include_vendored.cpp (new)
- crates/slic3r-ffi/tests/ffi_smoke.rs (new)
- crates/slic3r-ffi/fuzz/Cargo.toml (new)
- crates/slic3r-ffi/fuzz/fuzz_targets/ (new)
- crates/slic3r-proof/Cargo.toml (new)
- crates/slic3r-proof/src/lib.rs (new)
- crates/slic3r-proof/src/dumps.rs (new)
- crates/slic3r-proof/src/tiers.rs (new)
- crates/slic3r-proof/tests/trial_units.rs (new)
- crates/grampus-cli/Cargo.toml (new)
- crates/grampus-cli/src/main.rs (new)
- crates/grampus-cli/src/surface.rs (new)
- crates/grampus-cli/src/resolved_config.rs (new)
- crates/grampus-cli/tests/surface.rs (new)
- crates/slic3r-*/ (new)
- port/inventory (new)
- port/manifest.tsv (new)
- port/cut.tsv (new)
- port/crate-skeletons (new)
- port/regenerate (new)
- port/ownership (new)
- port/ownership.tsv (new)
- port/audit-order (new)
- port/port-status (new)
- port/cut-tasks (new)
- port/taskcuts/phase-a.tsv (new)
- port/taskcuts/phase-b.tsv (new)
- port/ownership-batches.tsv (new)
- port/foundation.tsv (new)
- port/goldens/ (new)
- port/reports/determinism.md (new)
- port/reports/determinism.tsv (new)
- port/reports/oracle-parity.md (new)
- port/reports/oracle-parity.tsv (new)
- port/reports/order-containers.md (new)
- port/reports/order-comparators.md (new)
- port/reports/divergences.md (new)
- port/reports/perf-backlog.md (new)

Nothing under `vendor/OrcaSlicer/` changes, and the submodule pointer stays at
`f92a7c065c5718b8ef1baac1cd704df2911238f4`. Scripts outside the Rust workspace
(`oracle/*`, `port/*` without an extension) are standalone executables with PEP 723 inline
metadata, run through `uv`, as the repository's conventions require.

## Order of work

Three steps start on day one and need nothing built first: **Step 1** (fetch the pinned binary),
**Step 7** (build the vendored libslic3r) and **Step 11** (the manifest and the crate cut). Step 7
and Step 11 read only the vendored tree.

Three tracks grow from them, and they join each other in named places rather than running clear
to the end:

- **The release-binary oracle:** Steps 1, 2, 3, 4, then 9, then 6. Step 5 (the comparator) joins
  after Step 2 and brings the Rust workspace with it.
- **The locally built oracle:** Step 7, then 8, then 10. Step 8 and Step 10 reach back into the
  release-binary track: Step 8 needs Steps 3 and 5, and Step 10 needs Steps 4, 5 and 9.
- **The manifest:** Step 11, then 12 and 13 side by side, then 15, then 14.

The two joins between the first two tracks, which is why they do not run independently: **Step 6
waits on Step 8** for any configuration the pinned CLI cannot construct (C12), and **Step 10
waits on Steps 4, 5 and 9** as well as on Steps 7 and 8.

Each step body carries its own `waits on` line, and that line is the authoritative one; the list
here says only where a step sits. Steps 16 to 21 join the tracks: 16 (checks and CI) after 5 and
14, 17 (FFI) after 5, 7, 11 and 16, 18 (proof harness) after 5, 7, 8, 11, 16 and 17, 19 (the
port's surface) after 5, 9, 11 and 15, 20 (crate skeletons and the foundation types) after 5, 7,
11, 15, 17, 18 and 19, 21 (the trial run) after 14, 15, 18 and 20.

**No port unit starts before Step 7 works.** That is C7's decision, and it covers Steps 20, 21,
22 and 23. Steps 22 and 23 also wait on Step 21.

Step 24 is the exception among the comparison steps: R20 and C3 make the per-stage comparison the
*first* comparison against the oracle, so `just compare-stages` is built during Step 23's fan-out,
as soon as the first pipeline stage's files are all ported, and only its CI line waits for Step 23
to finish. The end-to-end gates, Steps 25 to 27, wait on Step 23 and on Step 6's gate table.

### Step 1: Fetch and verify the pinned oracle binary
- delivers: R1

Add `oracle/fetch-release` and the `just oracle-fetch` recipe. The script downloads the release
asset `OrcaSlicer_Mac_universal_V2.5.2.dmg` with
`gh release download v2.5.2 -R OrcaSlicer/OrcaSlicer -p 'OrcaSlicer_Mac_universal_V2.5.2.dmg'`
into `oracle/.cache/download/`, a directory of its own outside `/Applications`, verifies its
sha256 against the value recorded in `vendor/README.md`, mounts it read-only
(`hdiutil attach -readonly -nobrowse`), copies the bundle to `oracle/.cache/app/OrcaSlicer.app`,
detaches, and verifies that
`/usr/libexec/PlistBuddy -c "Print :CFBundleShortVersionString" .../Contents/Info.plist` prints
`2.5.2`. Any mismatch exits non-zero and the script never rewrites the recorded hash.

The recorded value is
`6ea1afcdffb7e9ab011e2d6f6db665cb18463d1c4774591cd5db7ee0225479d9`. Commit it to
`vendor/README.md` in this step, before the first fetch, so the check is not circular. The
binary the later steps use is
`oracle/.cache/app/OrcaSlicer.app/Contents/MacOS/OrcaSlicer`; that is the GUI binary, which is
also the CLI and runs headless with no `DISPLAY`. Add `oracle/.cache/` to `.gitignore`.

`docs/ORACLE.md` is created here with its first two sections: the asset and its hash, and the
statement that `/Applications/OrcaSlicer.app` (2.4.2) and `/Applications/BambuStudio.app` are
never read by any command in this repository.

### Step 2: Run the oracle headlessly on a named configuration
- delivers: R2

Add `oracle/slice` and the recipe

```
just oracle-slice <fixture> <configuration> [--export-settings] [--export-statistics] [--out <dir>]
```

which forwards its arguments to `oracle/slice`. That is the whole interface, and Steps 6, 9, 10,
25, 26 and 27 call it with no other flag. `<fixture>` is a frozen basename in `fixtures/`, which
Step 4 fills, or a path to a model file, which is how this step's own proof runs before Step 4
exists. `--export-settings` and `--export-statistics` are off by default and switch on the
oracle's two JSON exports, written beside the G-code as `settings.json` and the oracle's own
`plate_1.statistics.json`. `--out` defaults to a fresh directory under the scratch directory, and
`--datadir` is never selectable: it is always a fresh directory inside the same scratch
directory, because that is what keeps the seven `printhost_*` credential fields of an exported
config empty.

The script resolves `<configuration>` to a row in the configuration table of `docs/ORACLE.md`,
`cd`s into a fresh scratch directory (a failing run writes `00000.log` into `$PWD`, which would
dirty the working tree), and invokes the binary from Step 1 as:

```
"$BIN" --datadir "$SCRATCH/datadir" \
  --load-settings "<machine json>;<process json>" \
  --load-filaments "<filament json>" \
  --slice 0 --arrange 0 --outputdir "$OUT" \
  [--export-settings "$OUT/settings.json"] [--export-statistics] <fixture>
```

Shape facts the script must keep, each of which silently corrupts a comparison otherwise:
`--load-settings` takes one `;`-separated string holding machine and process together;
`--load-filaments` is always passed explicitly, because omitting it still exits 0 and slices with
a different filament; the filament path is pinned exactly, because a filament that does not match
the machine's nozzle is accepted without warning; `--outputdir` is mandatory, because omitting it
exits 0 and writes no G-code anywhere; `--arrange 0` is always passed (R24); there is no
`-o`/`--output`, the name is `plate_<N>.gcode`; and `--datadir` is fresh per run.

Two shapes the oracle will not catch for us, so the script does:

- **A configuration row missing its filament path is rejected by the script, not by the oracle.**
  Omitting `--load-filaments` exits 0 and slices with a different filament — 9,910 differing
  lines against the pinned-filament run — so `oracle/slice` exits non-zero before it invokes the
  binary when the row names no filament, and names the row. A filament path that is named but
  absent from disk is the oracle's own `CLI_CONFIG_FILE_ERROR`.
- **`--arrange 0` is read from the pin, not yet run.** The option is live
  (`vendor/OrcaSlicer/src/libslic3r/PrintConfig.cpp:12935-12940`, `coInt`, default `0`) and its
  handler sets `need_arrange = false` (`src/OrcaSlicer.cpp:5072-5078`), but the one invocation on
  record omitted `--arrange` entirely and let the CLI's arranger run. The combination this
  repository depends on — `--arrange 0` with a mesh the fixture already places — is therefore
  first exercised here. This step's "Done when" is that exact invocation exiting 0 with a
  non-empty `plate_1.gcode`, and the Risks section carries the failure case beside C12.

The script exits non-zero unless a non-empty `plate_1.gcode` was written, and it maps the
oracle's negative exit codes, which reach the shell modulo 256, to named failures by converting
`rc - 256 if rc > 127 else rc` and looking the value up in a table transcribed from
`vendor/OrcaSlicer/src/libslic3r/Utils.hpp:42-94`: `0` `CLI_SUCCESS`, `-2`
`CLI_INVALID_PARAMS`, `-5` `CLI_CONFIG_FILE_ERROR`, `-17` `CLI_PROCESS_NOT_COMPATIBLE`, `-50`
`CLI_NO_SUITABLE_OBJECTS`, `-51` `CLI_VALIDATE_ERROR`, `-60`
`CLI_NO_SUITABLE_OBJECTS_AFTER_SKIP`, `-100` `CLI_SLICING_ERROR`, `-101`
`CLI_GCODE_PATH_CONFLICTS`. The script does not read `result.json`: macOS does not write one.

`docs/ORACLE.md` gains the configuration table. Every row names the machine, process and
filament preset paths under `vendor/OrcaSlicer/resources/profiles/BBL/` — the app bundle's own
presets are packed into an opaque `BBL.opc` and its loose `machine/` and `process/` directories
are empty, so the vendored tree is the only source — plus the explicit setting overrides that
switch the row's feature on, and the fixtures it uses. The base FFF row is
`machine/Bambu Lab X1 Carbon 0.4 nozzle.json`, `process/0.20mm Standard @BBL X1C.json`,
`filament/Bambu PLA Basic @BBL X1C.json`. Rows for tree support, wipe tower with
multi-filament, and fuzzy skin are added here as overrides over the base row; C12's decision
applies, and a row the pinned CLI cannot construct is marked so and handed to Step 8's driver
rather than worked around.

### Step 3: Record what produced every oracle output
- delivers: R7

Add `oracle/provenance`, which prints the provenance header every golden file, dump and report
carries, and which the scripts of Steps 2, 4, 6, 8, 10 and the comparator all call rather than
each formatting their own. The header records: the pinned commit
`f92a7c065c5718b8ef1baac1cd704df2911238f4`, read from
`git -C vendor/OrcaSlicer rev-parse HEAD` and asserted against that literal; which oracle
produced the output and its build flags; the OS (`sw_vers`); the architecture (`uname -m`); the
compiler version (`clang --version`); the value of `SCALING_FACTOR`; the full preset set and the
configuration's overrides; the fixture's basename; and the fixture's resolved instance transform
from its `vendor/README.md` record (R24).

The header is a comment block whose syntax suits the artifact: `;` lines for G-code goldens **and
for Step 8's stage dumps**, which are line-oriented text and carry the same `;` block, a
`provenance` object for JSON, and a Markdown table for the reports an oracle produced.

`oracle/provenance --check <path>...` exits non-zero when a file it covers has no header or has
one that disagrees with the pin. What it covers is named here and nowhere else, because the
header's items are undefined for a report no oracle produced: everything under `port/goldens/`,
every stage dump, and the two reports an oracle run writes, `port/reports/determinism.md` and
`port/reports/oracle-parity.md`. The static-analysis reports —
`port/reports/order-containers.md`, `order-comparators.md`, `divergences.md` and
`perf-backlog.md`, which are grepped out of the vendored tree and the port with no oracle and no
fixture — are excluded by name, and the check fails if it is pointed at one. That check joins
`just port-check` in Step 16 with exactly those paths.

### Step 4: Freeze the fixture set and fix placement
- delivers: R24, R25

Add `oracle/export-fixture` and `just fixture-export`, which turns a vendored model into a
committed STL that is already positioned on the target plate. It does that in two moves, because
**the pinned CLI has no XY placement at all**:

```
"$BIN" --datadir "$SCRATCH/datadir" --arrange 0 --export-stls "$SCRATCH/stl" <vendored model>
```

`--export-stls` takes a directory (`vendor/OrcaSlicer/src/libslic3r/PrintConfig.cpp:12764`,
`coString`; handler at `src/OrcaSlicer.cpp:6420-6425`) and writes one file per object through
`CLI::output_filepath`, which names it `obj_<index>_<object name>` with `.stl` appended and
creates the directory (`src/OrcaSlicer.cpp:8461-8516`). There is no option that sets the output
name, so the script renames the single file written there to the frozen basename. The exported
mesh carries whatever instance transform the load applied, because `store_stl(path, object, true)`
serialises `ModelObject::mesh()` (`src/libslic3r/Format/STL.cpp:53-57`), and for a mesh input that
is the default instance plus the Z-only `ensure_on_bed` the CLI applies at `src/OrcaSlicer.cpp:2003`
— so Z lands on the bed and X and Y are the model's own.

The script then applies the XY translation itself, on the exported STL, and that translation is
the second move. It is the script's work and not the oracle's because the CLI's `align_xy` and
`center` transform options **do not exist at the pin**: both `def = this->add(...)` calls are
inside a `/* */` block in `CLITransformConfigDef`
(`vendor/OrcaSlicer/src/libslic3r/PrintConfig.cpp:12903` and `:12930`, inside one commented block
that ends at `:12933`), so the only registered transforms
are `arrange`, `repetitions`, `ensure_on_bed`, `assemble`, `convert_unit`, `orient`, `repair`,
`rotate`, `rotate_x`, `rotate_y`, `ground_largest_face`, `ground_face_normal`,
`ground_face_point` and `scale`. Their *handlers* are still live code in `src/OrcaSlicer.cpp`
— `align_xy` at `:5065-5071` — which is why this is written down: a reader who finds the handler
will think the flag works.

The translation puts each mesh's XY bounding-box centre at `(128, 128)`, the centre of the X1C's
printable area (`printable_area` `0x0 256x0 256x256 0x256`, inherited from
`resources/profiles/BBL/machine/fdm_bbl_3dp_001_common.json:123`), away from that machine's
`bed_exclude_area` of `0x0 18x0 18x28 0x28` in the corner. The script computes the offset from the
mesh's own bounding box rather than from a figure written down here, and asserts afterwards that
the result is inside the printable area and does not meet the exclusion area — the benchy's extent
is not measured in this plan, because its vendored form is Draco-compressed and the assertion is
what covers it. It matters that something does this: `tests/data/20mm_cube.obj` spans 0 to 20 in
both X and Y at the pin, so its own coordinates lie inside that exclusion area, and with
`--arrange 0` nothing in the oracle would move it out. After the translation the fixture's
resolved instance transform is the identity and `ensure_on_bed` is a no-op, which is the one
transform the `vendor/README.md` row records.

The basenames are frozen because the model's basename —
not its path — is embedded in both the G-code (`; printing object <basename> id:0 copy 0`) and
the statistics JSON (`objects[0].name`); no absolute path leaks into either, so the fixtures are
reproducible across machines.

The three fixtures are `3DBenchy.stl` from
`vendor/OrcaSlicer/resources/handy_models/3DBenchy.drc`, `20mm_cube.stl` from
`vendor/OrcaSlicer/tests/data/20mm_cube.obj`, and `frog_legs.stl` from
`vendor/OrcaSlicer/tests/data/frog_legs.obj`. Each gets a row in `vendor/README.md` naming its
frozen basename, the vendored path it came from, the single resolved instance transform, and the
license the publisher states — recorded as 3DBenchy.com's CC0 1.0 for the benchy, with the
explicit note that upstream records no license for `resources/handy_models/`, which has no
license file.

Add `oracle/assert-placement`, which reads a fixture's `vendor/README.md` row and the bounding box
the oracle reports for a run, and exits non-zero when they disagree. Steps 25 and 26 call it, so
`just compare-benchy` and `just compare-fixtures` fail on an unexpected placement. Each fixture
holds one object, so each record names one transform; the multi-filament configuration assigns
more than one filament to that single object rather than adding a second one. With `--arrange 0`
the oracle's only remaining placement step is the Z-only `ModelObject::ensure_on_bed` it applies
to every mesh input, and the committed fixture already satisfies it, so the transform is the
identity.

This step waits on Steps 1, 2 and 3, and Step 2's own proof runs before it: that proof uses
`vendor/OrcaSlicer/tests/data/frog_legs.obj` as a path rather than a frozen fixture, because that
model's own coordinates — 34.4 to 86.5 in X and 9.1 to 58.9 in Y — are inside the printable area
and clear of the exclusion area, and it is therefore the one vendored model that `--arrange 0`
can slice before this step exists.

### Step 5: The comparator
- delivers: R4

Add the workspace and `crates/oracle-compare`, a library plus a binary, with `just compare <a> <b>`
wired to the binary.

**This step owns the workspace, and every later crate depends on it.** It adds `Cargo.toml`,
`rust-toolchain.toml` and `/target/` in `.gitignore`. The root `Cargo.toml` uses
`members = ["crates/*"]` and nothing else, so **no later step and no port task ever edits the root
manifest to add a crate** — the directory appearing is enough. `Cargo.lock` is committed, because
the oracle comparison has to be reproducible on the one platform its goldens are valid for; the
steps that may change it are the ones that add a crate or a dependency — Steps 5, 17, 18, 19 and
Step 20's skeleton task — and a Phase A or Phase B port task never adds a dependency, so it never
touches the lock file. The toolchain `rust-toolchain.toml` pins is what Step 16 installs in CI.
Steps 17, 18, 19 and 20 all wait on this step for the workspace.

The comparator takes two G-code files and evaluates L0 to L4 independently, reporting every level
that passes, the strongest of them, and the first failing location at the next stronger level;
when no level passes it reports `none` and the first differing byte. The levels are not assumed
nested, which is why each is evaluated rather than stopping at the first pass.

`src/levels.rs` holds the five levels: L0 identical bytes; L1 identical bytes after the declared
normalisation; L2 identical token stream, command plus axis words, comments dropped; L3 identical
move count and order with numbers inside the declared per-axis tolerance, reporting max and RMS
deviation per axis; L4 per-layer and per-role aggregates — move count, extruded length, E delta,
bounding box, travel distance, layer count — inside the declared aggregate tolerance. `src/gcode.rs`
holds the parser and the localisation by the `; FEATURE:`, `; Z_HEIGHT:`, `; CHANGE_LAYER` and
per-object label comments that Step 24 reuses.

The declared normalisation is exactly the four line kinds upstream itself strips in
`vendor/OrcaSlicer/tests/fff_print/test_slicing_pipeline_hook.cpp:83-96`: a line starting
`; generated by `, a line starting `; model label id: `, a line containing
`printing object, unique label id: `, and a line containing `slicing_pipeline_plugin`. Nothing
else in the G-code is normalised, and `tests/normalisation.rs` asserts that list against the
vendored file so a drift in either is caught.

`src/json.rs` compares the two JSON artifacts: `--export-settings` byte for byte with no
normalisation, allowing only the `slicing_pipeline_plugin` key to differ when the compared pair
includes a run with the hook armed, since arming requires that key to be non-empty
(`src/libslic3r/Print.cpp:3306-3310`); and `--export-statistics` with the single field
`generator` normalised, which the oracle writes as the static
`SLIC3R_APP_NAME + " " + SoftFever_VERSION` (`src/libslic3r/GCode/GCodeStatistics.cpp:177`).

Every tolerance defaults to exact: zero per-axis deviation, zero aggregate deviation. The
comparator takes a tolerance only from a manifest row or a `docs/ORACLE.md` configuration row,
refuses a tolerance with no reason text in that row, and prints which tolerance it used.
`tests/levels.rs` and `tests/json.rs` cover each level and each failure report against committed
fragments under `tests/data/`, including a pair taken from two real runs of Step 2.

### Step 6: Measure the oracle against itself and publish the gate table
- delivers: R3

Add `oracle/determinism` and `just oracle-determinism`, which runs each configuration named in
`docs/ORACLE.md` at least five times, each with a fresh `--datadir` and `--outputdir`, with
`--export-settings` and `--export-statistics` on, compares every pair with Step 5's comparator,
and regenerates `port/reports/determinism.tsv` **and** `port/reports/determinism.md` from it.

The `.tsv` is the file Steps 25, 26 and 27 read at run time, and it exists so that four separate
tasks do not each write a Markdown parser against a table they have never seen. Its columns are:
configuration, artifact (`gcode`, `settings-json`, `statistics-json`), run count, normalisation,
gate (`L0` to `L4` or `none`), and the oracle that produced the runs. The Markdown report is
generated from it and is what a person reads; neither is authored by hand.

The report has one row per configuration and one sub-row per compared artifact — the G-code, the
`--export-settings` JSON and the `--export-statistics` JSON. Each sub-row names the normalisation
R4 declares for that artifact
and records whether the five runs were byte-identical with no normalisation, with that
normalisation, or not at all; its gate is the strongest level all five runs reach against each
other, and `none` where they reach none. The base FFF row is where the whole design's
"L1 is the gate" claim comes from, so it is produced by this command and quoted from nowhere.
Re-running the command leaves a committed report unchanged apart from its timestamps.

`docs/ORACLE.md` gains the gate table with the same per-artifact breakdown, generated from the
report rather than authored. A configuration the pinned CLI cannot construct (C12) is run through
Step 8's built-oracle driver instead, and its rows say which oracle produced them; a
configuration neither can construct is recorded with a gate of `none`. Fuzzy skin is expected to
land at `none` whatever happens, because its displacement generator is seeded per thread from
`std::random_device` (`src/libslic3r/Feature/FuzzySkin/FuzzySkin.cpp:37-44`), so Step 27 reports
it as a named divergence instead of failing.

This step waits on Steps 2, 3, 4 and 5, and on Step 8 for any configuration the CLI cannot
construct.

### Step 7: Build the vendored libslic3r on this platform

Groundwork. It delivers no requirement by itself, and Steps 8, 10, 18 and every port unit need
it; C7's decision is that it is attempted first, as its own piece of work, and that a failure is
reported rather than worked around.

Add `oracle/build-libslic3r` and `just oracle-build`, a two-stage CMake build that drives the
vendored tree's own two projects and writes nothing into its tracked files. Both build
directories are gitignored upstream (`/build*/`, `deps/build*`), so building in place does not
dirty the submodule.

Stage one, the dependency superbuild, configured at `vendor/OrcaSlicer/deps` into
`vendor/OrcaSlicer/deps/build/arm64` — the upstream default location, which matters because
OCCT's patch step only applies when `git rev-parse --is-inside-work-tree` succeeds in the build
directory:

```
export CMAKE_POLICY_VERSION_MINIMUM=3.10
cmake -S vendor/OrcaSlicer/deps -B vendor/OrcaSlicer/deps/build/arm64 -G Ninja \
  -DCMAKE_BUILD_TYPE=Release -DCMAKE_OSX_ARCHITECTURES=arm64 \
  -DCMAKE_OSX_DEPLOYMENT_TARGET=12.0 \
  -DCMAKE_IGNORE_PREFIX_PATH="/opt/local;/usr/local;/opt/homebrew" \
  -DCMAKE_POLICY_VERSION_MINIMUM=3.10
cmake --build vendor/OrcaSlicer/deps/build/arm64 --target \
  dep_Boost dep_TBB dep_Eigen dep_CGAL dep_OCCT dep_OpenCV dep_OpenVDB dep_Assimp dep_Draco \
  dep_NLopt dep_Cereal dep_Qhull dep_GLFW dep_PNG dep_EXPAT dep_JPEG dep_libnoise \
  dep_OpenSSL dep_CURL dep_python3 dep_SLVS
```

The per-dep targets are named rather than the `deps` target, because `deps/CMakeLists.txt` has no
`SLIC3R_GUI` option and its `deps` target always includes wxWidgets, wxInspector, FFmpeg and
libdatachannel. `CMAKE_POLICY_VERSION_MINIMUM` is mandatory with CMake 4.x.
`CMAKE_IGNORE_PREFIX_PATH` keeps Homebrew out of the build. The bundled Python 3.12.13
(`dep_python3`) is required even with the GUI off.

Stage two, the main project, configured against that prefix and **configured twice**, because
`src/libslic3r/CMakeLists.txt:746` writes `OCCT_LIBS` to the cache at the end of its configure
while the top-level consumer reads what is already there, and the first configure otherwise links
a stale toolkit list:

```
DEPS=vendor/OrcaSlicer/deps/build/arm64/OrcaSlicer_dep/usr/local
cmake -S vendor/OrcaSlicer -B vendor/OrcaSlicer/build/arm64 -G "Ninja Multi-Config" \
  -DCMAKE_PREFIX_PATH="$DEPS" -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_OSX_ARCHITECTURES=arm64 -DCMAKE_OSX_DEPLOYMENT_TARGET=12.0 \
  -DCMAKE_IGNORE_PREFIX_PATH="/opt/local;/usr/local;/opt/homebrew" \
  -DCMAKE_POLICY_VERSION_MINIMUM=3.10 \
  -DSLIC3R_GUI=OFF -DSLIC3R_CAD=ON -DBUILD_TESTS=ON -DSLIC3R_STATIC=1 \
  -DCMAKE_CXX_FLAGS="-ffp-contract=off"
cmake --build vendor/OrcaSlicer/build/arm64 --config Release \
  --target libslic3r libslic3r_tests fff_print_tests
ctest --test-dir vendor/OrcaSlicer/build/arm64/tests/fff_print -C Release --output-on-failure
```

`SLIC3R_GUI=OFF` with `SLIC3R_CAD=ON` is the one such combination upstream exercises
(`scripts/CAD/run-kernel-tests.sh:145-158`), and `SLIC3R_CAD=OFF` would flip a second untried
axis, so it stays ON. Targets are named and `all` is never built, because
`tests/slic3rutils/CMakeLists.txt` links `libslic3r_gui`, which does not exist with the GUI off.
`-C Release` on `ctest` is not optional with a multi-config generator. `-ffp-contract=off` is the
flag C2's decision makes normative: without it the compiler fuses multiply-add expressions that
Rust never fuses, including the two-dimensional cross product that decides orientation branches.
A stale `NLopt_DIR` in the cache is cleared with `cmake -U NLopt_DIR -U NLopt_LIBS .`, never by
editing `CMakeCache.txt`.

`vendor/README.md` records the outcome: that nobody upstream builds this configuration on macOS,
what was needed to make it work, and the measured wall-clock and disk cost of the first build.
`docs/ORACLE.md` gains the built oracle's exact flags. The upstream suites are built and run once
here as evidence the library works, not as a gate.

**This step adds `just oracle-build` to the macOS job of `.github/workflows/ci.yml`**, once it
works, together with the cache of `vendor/OrcaSlicer/build/arm64` that Step 16 specifies. Step 16
creates that job without this recipe, the same way it leaves every gate to the step that makes it
pass. Whichever of the two lands second adds its half; neither is written in terms of the other's
line.

### Step 8: Dump the pipeline's intermediate state
- delivers: R5

Add `oracle/stage-dump/`, a C++ program plus its `CMakeLists.txt`, linked against the
`libslic3r` built in Step 7, and `just oracle-dump <fixture> <configuration>`. It drives the
library through the sequence `vendor/OrcaSlicer/tests/fff_print/test_helpers.hpp` uses — build a
model, apply a config, validate, process, export G-code — and registers a hook with
`Print::set_slicing_pipeline_hook_fn` (`vendor/OrcaSlicer/src/libslic3r/Print.hpp:1064`, whose
`SlicingPipelineHookFn` alias is at `:1061`).
The hook's signature is `void(Print&, const PrintObject*, SlicingPipelineStepPlugin)`: it carries
no layer, so it fires once per step, and the dumper walks the object's layers itself at each
seam. It is armed only when the `slicing_pipeline_plugin` config option is non-empty
(`Print.cpp:3306-3310`), which the program sets as the upstream test does, with
`config.set_key_value("slicing_pipeline_plugin", new ConfigOptionStrings({"probe"}))`, and resets
with `set_slicing_pipeline_hook_fn(nullptr)` afterwards.

One dump per seam that fires, at exactly the seams R5 enumerates, which this step does not
restate. `dump_seams.cpp` reads the state each seam leaves — `Layer::lslices`,
`LayerRegion::slices`, `raw_slices`, `perimeters`, `thin_fills`, `fill_surfaces`, `fills`,
`PrintObject::support_layers()`, `object_skirt()` — and serialises integer `coord_t` coordinates,
never round-tripping through float. Every dump carries Step 3's provenance header.

The program is also the driver C12's decision names: it takes a configuration either as the
oracle's resolved-config JSON or as in-code `set_key_value` overrides, so a configuration the
pinned CLI refuses can still be constructed, and Step 6 uses it for those rows.

A check in the same step proves the dumping build's exported G-code is identical at L1 — Step 5's
comparator — to the same build with the hook unregistered. L1 and not byte for byte, because
arming the hook adds a `slicing_pipeline_plugin` line to the exported config block, which is the
fourth line kind the normalisation covers and the same comparison upstream makes in
`tests/fff_print/test_slicing_pipeline_hook.cpp:98-125`.

`vendor/README.md`'s "Pipeline steps" table is corrected here to say which steps have a hook seam
and which do not.

This step waits on Steps 3, 5 and 7.

### Step 9: Resolved-config goldens
- delivers: R11

Add `port/goldens/configs/`, holding one resolved configuration per `docs/ORACLE.md` row, each
produced by `just oracle-slice ... --export-settings` on an unarmed run with a throwaway
`--datadir`, each carrying Step 3's provenance header. These are the port's configuration input:
preset files, the `inherits` chain and preset-bundle resolution are out of scope, so a benchy
difference cannot be caused by preset resolution. The goldens are byte-stable across runs with no
normalisation, which Step 6's report records.

Document in `docs/ORACLE.md`, from a committed script-measured comparison rather than from
memory, the relationship between the exported JSON and the G-code's `CONFIG_BLOCK`: the JSON
carries keys the G-code does not, among them the seven `printhost_*` credential fields, which is
why the throwaway datadir is mandatory; and the G-code carries two keys the JSON does not,
`first_layer_temperature` and `first_layer_bed_temperature`, which the port derives at emission
time. Emission is not resolution and stays in scope: the port writes the `HEADER_BLOCK` and
`CONFIG_BLOCK` from the resolved input, selecting and formatting the same keys the oracle writes
(`src/libslic3r/GCode.cpp:4255-4335`), and the print-time estimator whose line the header carries
(`src/libslic3r/GCode/GCodeProcessor.cpp:1310`) is in scope because L1 compares both.

This step waits on Steps 2 and 3.

### Step 10: Report whether the two oracles agree
- delivers: R6

Add `oracle/parity` and `just oracle-parity`, which slices every fixture in the committed set with
both oracles — the pinned release binary through `oracle/slice`, the locally built library through
`oracle/stage-dump`'s export path — from the same resolved config and the same instance transform,
compares with Step 5's comparator, and regenerates `port/reports/oracle-parity.tsv` **and**
`port/reports/oracle-parity.md` from it. The report records whether the two reach L1 against each
other, lists every difference found, and names the built oracle's exact CMake flags, including
`-ffp-contract=off`.

The `.tsv` is what Steps 25, 26 and 27 read at run time to pick their oracle, for the same reason
Step 6 emits one: columns configuration, fixture, artifact, level reached between the two oracles,
and `normative oracle` holding `release` where they reach L1 and `built` where they do not. Those
steps read this file and never decide from prose.

C2's decision is what the report feeds: the built library is normative for every per-unit and
per-stage proof and macOS arm64 is the one platform goldens are valid for; for the end-to-end
gates the release binary is normative where this report shows L1. Where it does not, the built
oracle becomes normative for that fixture and configuration, the disagreement becomes a row in
`port/reports/divergences.md`, and goldens are kept per oracle. The step writes those rows and
says so in the report rather than leaving the reader to infer it. Those rows reach
`port/reports/divergences.md` through Step 13's regeneration, which reads this report as one of
its named sources; this step writes its own report and never edits `divergences.md` by hand.

**This step adds `just oracle-parity` to the macOS job of `.github/workflows/ci.yml`.** Step 16
creates that job without it.

This step waits on Steps 4, 5, 7, 8 and 9.

### Step 11: The scope manifest and the crate cut
- delivers: R8, R9

Add `port/inventory`, `just inventory` and `just inventory-check`, which regenerate
`port/manifest.tsv` and `port/cut.tsv`. This step reads only the vendored source and starts on day
one alongside Steps 1 and 7. It also adds `port/cut-tasks` and `just task-cuts <phase>`, the
script Steps 12, 20, 22 and 23 each run in their first task to commit their own cut file. It lives
here because the manifest and the cut are what it batches, and no later step creates it. One of
its phases reads a second file: `just task-cuts ownership` also reads `port/ownership.tsv` for its
field list, so that phase is runnable only once Step 12's spike has written that file, and the
script says so and exits non-zero rather than batching nothing.

`port/manifest.tsv` has one row per libslic3r source file at the pin, with columns: path,
subsystem, lines, file-level include-DAG level, crate-level level, in scope or out with a reason,
target crate and module, pipeline stage or `none`, template-heavy, equality tier, tolerance,
comparison function, proof path, fuzz target, and status.

**Which columns the script derives, which it carries forward, and which it computes from the Rust
tree.** This is the whole answer to how a generated file can also be the port's progress record,
and `just inventory-check` depends on it:

- **Derived from the vendored tree** on every run: path, subsystem, lines, both levels, in scope
  or out with its reason, target crate and module, and pipeline stage. A run recomputes these from
  the source and overwrites whatever is committed.
- **Seeded by the script, then carried forward verbatim** from the committed `port/manifest.tsv`:
  template-heavy, equality tier, tolerance, comparison function, proof path, and fuzz target.
  These are judgements a script can only guess at — they are set by Step 21's trial and by review
  in the pull request that first fills a row — so `port/inventory` writes a stated default into a
  row that is new (the two rules below for template-heavy and fuzz target, `exact` tier, empty
  tolerance and comparison function, and the proof path the row's scope implies) and then keeps
  each of those cells exactly as it finds it on every later run.
- **Derived from the Rust tree, not stored as a fact anyone edits:** status. A row's status is
  `drafted` when a file exists at its target path carrying a `PORT STATUS` trailer that names it
  and no `mod` declaration reaches that file, `ported` when a `mod` line reaches it and it holds
  no `todo!()` or `unimplemented!()` and its R19 proof passes, and `not-started` otherwise. Step
  14's `port/port-status` computes it the same way from the same tree, and that computation is the
  authority.

So `just inventory-check` — which regenerates into a scratch directory and fails when the
committed file differs — **diffs the derived columns only**, and ignores the carried-forward
columns and the status column entirely. **A port task never edits `port/manifest.tsv`.** A row's
status advances because the task added a Rust file, not because anybody wrote a cell, which is
what lets a hundred task branches run at once without colliding on one line of one file.

`template-heavy` is `yes` for a row that is type-and-trait design rather than a file-unit, which
is what Step 20's set is. The script seeds it by a rule written into the script: an in-scope header
with no same-stem `.cpp` sibling, or an in-scope header where `template<`, `constexpr` and `inline`
definitions make up more than a quarter of its non-comment lines. The column exists because that
set cannot be read off the file layout — at the pin `Point.hpp`, `Config.hpp`, `Model.hpp` and
`libslic3r.h` all have a same-stem `.cpp`, so a "header with no `.cpp` sibling" rule reaches only
`AABBTreeIndirect.hpp` — and it is carried forward rather than recomputed because the seed rule
will read some rows wrongly, `Model.hpp` the likeliest, and those are corrected in the review of
Step 11's own pull request and in Step 20's spike.

`fuzz target` is `required` for an in-scope row whose equality tier is exact and whose ported
surface takes `Point`, `Points`, `Polygon`, `Polygons`, `ExPolygon` or `ExPolygons`, which is R19's
"fixed-point units that take polygons or points", and `n/a` otherwise. The script sets it from the
signatures it finds in the header; a row where that reading is wrong is corrected in review and
then carried forward like the other judgement columns.

The file set is one row per `.h`, `.hpp`, `.cpp` or `.mm` file under
`vendor/OrcaSlicer/src/libslic3r`, including `MacUtils.mm` and `Format/ModelIO.mm`, which
`src/libslic3r/CMakeLists.txt:604-610` appends under `if (APPLE)`, and excluding the generated
`libslic3r_version.h`, which exists only as `libslic3r_version.h.in`. The script asserts the row
count against the set it finds on disk, not against a number written in a document.

Both `.mm` files are rows and both are **deducted, with their reason in the row**, so no porting
rule has to cover Objective-C++: `MacUtils.mm` is 23 lines holding
`is_macos_support_boost_add_file_log` and `is_mac_version_15`, two OS-version predicates no
in-scope caller reaches, and `Format/ModelIO.mm` is 26 lines whose whole content is
`make_temp_stl_with_modelio`, an Apple-only importer that converts an asset to a temporary STL
before the tree's own readers run, and every committed fixture is already an STL. That keeps the
unit rule in Steps 15 and 22 — one `.cpp` plus its same-stem `.hpp` — true of every unit that
gets a task.

Two traps in the build file the script must handle, because both change the answer: the source
variable is misspelled upstream as `lisbslic3r_sources`, and the `libslic3r_cgal` list puts two
files on one line, so splitting on newlines loses half of them.

Scope is computed in the four passes the spec states — the include closure over
`Model::read_from_file`, `Print::apply`, `Print::process` and `Print::export_gcode`; the same-stem
`.cpp` sibling of every header in the closure; an orphan pass over every `.cpp` that defines
methods of a class declared in an in-closure header; then the named deductions, each written into
its row as a reason. The script asserts that `LayerRegion.cpp`, `ContourZ.cpp` and `BeltPurge.cpp`
are in the result, because passes 1 and 2 provably cannot reach them: they have no header and no
`#include` in the tree names them. It also resolves includes case-sensitively, because
`Support/SupportMaterial.cpp:68` has `#include "utils.hpp"` inside a debug-only branch and the
tracked file is `Utils.hpp`, which on this filesystem silently double-counts two files.

SLA is marked in or out per file with its reason, never by directory: `SLA/IndexedMesh` is in
scope because Z-contouring calls it from `ContourZ.cpp`, `AABBMesh.cpp`, `PrintObject.cpp` and
`Layer.hpp`; the SLA fields carried on the model type are ported as data only; the hollowing, pad
and concave-hull implementations are out. `GCode/GCodeStatistics.{cpp,hpp}` is in scope, because
the application calls it at `src/OrcaSlicer.cpp:7231`.

Crate levels come from a cut, not from the graph. The intra-library include graph is acyclic at
file granularity but at module granularity is one cycle, so the script computes a named
feedback-arc cut, writes each cut edge to `port/cut.tsv` as a file and the header it includes, and
derives the crate-level level from the cut graph so re-running is deterministic. The two mutual
clusters the cut cannot break, `GCode.cpp` with `GCode/*` and `Model` with `Format`, are single
crates. C10's decision accepts the computed cut as the crate map, reviewed like any other change
in this step's pull request and changed afterwards only by a pull request that regenerates the
manifest.

The manifest carries no coverage column. Coverage-measured execution during a benchy slice was
nothing's precondition and nothing gated on it, and a column no step owns is a column that drifts,
so it is not in the file. Whoever wants the measurement later takes it from a build of Step 7 and
records it outside the manifest.

### Step 12: The ownership manifest
- delivers: R30

Add `port/ownership` and `just ownership`, and commit `port/ownership.tsv`: one row per
raw-pointer field, and per field that is a container of raw pointers, declared in a header the
manifest marks in scope, with columns header path, line, declaring type, field, C++ type, class
(`owned`, `shared`, `borrowed`, `back-reference`, `ffi`, `unknown`), the `path:line` of the
evidence for that class, and the Rust type porters use. The script lists the fields from the
vendored headers and fails when a field has no row, a row names a field that does not exist at the
pin, or a row other than `unknown` has no evidence.

The evidence this codebase offers is comments over raw pointers rather than types, which is why
the column exists: `ExtrusionEntityCollection.hpp:39` `// we own these entities` against `:19`
`// Removed elements are NOT being deleted.`, and `Model.hpp:189` `Model *m_model;` as a borrow.
C13's decision brought this artifact into the intent, so the file is committed before the first
fan-out pull request is opened, and Step 15's `docs/PORTING.md` tells porters to take the Rust
type from it verbatim. `just port-status` prints the count of `unknown` rows.

**What `unknown` is allowed to mean**, so the artifact cannot pass empty. R30's title is that
every raw pointer is classified before fan-out, and a count alone does not hold that. An
`unknown` row is only accepted when it carries the `path:line` of the use that leaves the class
unclear and the conservative Rust type a porter must use until it is resolved, and
`just ownership-check` fails on an `unknown` row missing either, exactly as it fails on a
non-`unknown` row with no evidence. A row whose `class` cell is still blank is allowed while this
step's own batch tasks are open — the spike commits a row per field before any of them is
classified — and **this step is done when no in-scope row's class is blank**, which is the one
line a reviewer answers yes or no to. This step's last task removes that allowance from
`port/ownership`, so a blank class is a failure from then on and no later work can reopen one.

**How the tasks are cut.** The raw-pointer surface of roughly 420 files is not one task. The first
task cut from this step is a spike: it adds `port/ownership`, runs `just task-cuts ownership`, and
commits the field list together with `port/ownership-batches.tsv`, which batches the fields by
subsystem into rows of at most 120 fields each, in the manifest's crate-level order. The remaining
tasks are cut from that file in a later pass, one task per row; each classifies its rows with
cited evidence, and `just ownership-check` passes for the whole file throughout, because an
unclassified field is a missing row and the spike commits every row.

`just ownership-check` regenerates the field list into a scratch directory and fails when the
committed file disagrees with the headers at the pin, which is what Step 16 puts in the checks a
task passes; plain `just ownership` rewrites the committed file and is for a person regenerating
deliberately.

This step waits on Step 11.

### Step 13: The order-dependence audits
- delivers: R18

Add `port/audit-order` and `just audit-order`, which regenerate two reports over the in-scope
files: `port/reports/order-containers.md`, every unordered container that is iterated rather than
only looked up, and `port/reports/order-comparators.md`, every comparator that is not a total
order. Each site records the port's choice and whether it diverges from the oracle.

The same command regenerates `port/reports/divergences.md`, which is fully derived and never
hand-edited. **Its sources are exactly these four, and no step contributes to it any other way:**

1. the rows of `port/reports/order-containers.md` and `port/reports/order-comparators.md` that
   this command just wrote;
2. every `// FIDELITY(port):` marker in the port tree, which is how a module declares a site;
3. `port/reports/oracle-parity.tsv`, every row whose two oracles do not reach L1 — those are the
   rows Step 10 produces, and Step 10 writes its own report and nothing else;
4. `port/reports/determinism.tsv`, every row whose gate is `none` — those are the configurations
   Step 27 reports as named divergences rather than failures, and Step 27 likewise writes nothing
   into `divergences.md`.

Naming all four is what makes the regeneration deterministic and stops Steps 10 and 27 from
having rows wiped by the next run.

The port's choices, which `docs/PORTING.md` then carries: sum in a fixed order, use total-order
comparators, and use insertion-ordered maps wherever iteration order reaches output, so the port
is deterministic where the oracle is not. Each such site in the port carries a
`// FIDELITY(port):` marker, which is what makes the regeneration possible.

`just audit-check` is the drift-detecting variant: it regenerates all three reports into a scratch
directory and fails when a committed one differs. **It is not a check a port task passes.** A
whole-file diff of a generated report cannot be, because a hundred task branches each adding a
marker would each commit a regeneration that the next merge makes stale, and the staleness is a
red check rather than a text conflict. So `just audit-check` runs on `main` only, in the
regeneration pass Step 14 defines, and what a task passes is the per-marker assertion that step
specifies instead.

This step also adds `port/regenerate` and `just regenerate`, which runs `just audit-order` and the
perf-backlog regeneration together and is the command that pass invokes, so there is one place
that rewrites the generated reports.

This step waits on Step 11, and runs beside Step 12.

### Step 14: port-status
- delivers: R14, R15, R16, R29

Add `port/port-status` and `just port-status`, the one command the porting work is held to. It
fails unless: every Rust module that ports a libslic3r file ends with a `PORT STATUS` trailer
naming its source path, `confidence: high|medium|low`, its equality tier and the oracle that
proves it; that source path exists at the pin and matches the module's manifest row; every
`unsafe` block carries a `// SAFETY:` comment; every ported numeric constant carries a
`// PORT SRC: <vendored path>:<line>` marker beside it; and every port marker is one of
`// TODO(port):`, `// PERF(port):`, `// PORT NOTE:`, `// SAFETY:`, `// PORT SRC:` or
`// FIDELITY(port):`. It prints the count per marker kind.

It counts the burn-down: `todo!()`, `unimplemented!()`, and modules gated out of the build — a
file under `crates/*/src/` that carries a `PORT STATUS` trailer and that no `mod` declaration
reaches, which is exactly how Step 22's Phase A drafts sit in the tree. It fails on a placeholder
constant, meaning a numeric constant whose `// PORT SRC:` marker names a vendored line that does
not contain that literal. It prints the count of `unknown` rows in `port/ownership.tsv`.

**It derives every row's status from the tree** by the rule Step 11 states — trailer present and
no `mod` line reaching it is `drafted`; `mod` line, no `todo!()` or `unimplemented!()`, and a
passing R19 proof is `ported`; otherwise `not-started` — and it fails while any in-scope row's
*derived* status is other than `ported`, printing how many remain. It never reads a status cell as
a fact. That is the manifest-to-module direction Step 23 closes, and it is why a port task proves
its progress by adding a file rather than by editing a shared line.

It fails a unit whose manifest row has `fuzz target` set to `required` and that has no target
under `crates/slic3r-ffi/fuzz/fuzz_targets/` naming that module, which is R19's second half and
the one part of R19 a script can hold rather than a reviewer.

Every check this script makes is **text-based**: it reads files as lines and never parses Rust,
because it has to run over Phase A drafts that are not valid Rust and that `cargo fmt` never sees,
since no `mod` line reaches them.

It fails unless `docs/PORTING.md` contains the sections R12 names and `docs/WORKFLOW.md` contains
the sections R13 names, reading headings and not prose quality.

**The generated reports, and who may write them.** `port/reports/perf-backlog.md` is the only
place a knowingly slower translation lives, and it is the handover to the following intent. It is
regenerated from the `// PERF(port):` markers, together with Step 13's three reports, by
`just regenerate`. Two different checks sit on it, and keeping them apart is what lets the fan-out
run:

- **What a port task passes** is a per-marker assertion, `just port-status --per-commit`: every
  `// PERF(port):` and `// FIDELITY(port):` marker the branch adds is well formed — on its own
  line, naming the module's source path and the reason in the shape the report's row needs — and
  every marker already in the tree still has its row. It never regenerates a report and never
  compares one as a whole file, so two branches that each add a marker do not conflict and neither
  goes stale when the other merges.
- **What the repository does for the task** is the regeneration pass: a scheduled task that runs
  `just regenerate` on `main`, commits whatever moved, and runs `just audit-check` and the
  perf-backlog whole-file diff, which is where those two drift checks live. It is the only writer
  of `port/reports/perf-backlog.md`, `divergences.md`, `order-containers.md` and
  `order-comparators.md`, and a port task's branch leaves all four alone. `docs/WORKFLOW.md` says
  so in the words Step 15 writes.

No check this step adds gates on throughput, instruction count or allocation, and none is added
anywhere else in this plan.

`just port-status` with no argument is all of the above, including the burn-down to zero and the
in-scope-rows-remaining gate, which is how R16 and R26 are worded and which is therefore true
only at the end of Step 23. It is not what a task's pull request passes. The script also takes
`--per-commit`, which runs exactly the subset R23 puts in `.asdlc/checks` — the trailers and their
agreement with the manifest row, the source paths at the pin, `// SAFETY:`, `// PORT SRC:` and the
placeholder-constant check, the marker whitelist, the per-marker report assertion above, the
required-fuzz-target check, and the document sections — and which passes while a row's derived
status is still `not-started` or `drafted`, because a module that is honestly a draft is not a
fault. It prints the burn-down, the remaining row count and the `unknown` count in both modes and
fails on them only without the flag. Step 16 puts `--per-commit` in the checks a task passes, and
Steps 25, 26 and 27 call the gate.

The script's third mode is `--pr-body <file>`, which Step 21 specifies and which Step 21 also
wires to its own recipe, `just pr-body-check <file>`. That one spelling is used in Step 21, in the
CI step it adds and in this plan's Proof section; `just port-status` forwards nothing, because it
runs the per-commit subset and the gate and takes no file.

This step waits on Steps 11, 12, 13 and 15, because it checks their artifacts.

### Step 15: The translation contract and the workflow document
- delivers: R12, R13

Write `docs/PORTING.md` and `docs/WORKFLOW.md`, both committed before any unit is ported.

`docs/PORTING.md` carries the crate map, generated from `port/cut.tsv` and which agents may not
invent; a type map; an idiom map; a don't-translate list; a banned-substitution list; and the six
rules the spec states, transcribed with their citations. The scaling rule is stated per API,
because the five entry points do not behave alike: `scaled<coord_t>()` and the Eigen overload
(`Point.hpp:669-690`) are truncating division by the `SCALING_FACTOR` global and are ported as
truncating division, with rounding banned — the `std::round` on `Point.hpp:676` is commented out
in the original, under a `TODO: is the rounding necessary?` that reads as an invitation; the
`scale_()` macro (`libslic3r.h:106`) is a plain `double` division that truncates nothing, and the
floating-output `scaled<Tout>()` overload (`Point.hpp:660-667`) does not truncate either, so where
the original converts their result to an integer type the port reproduces that truncation at the
call site and not inside the API; `unscale_()` (`libslic3r.h:107`), `unscale<T>()`
(`libslic3r.h:137-138`) and `unscaled<T>()` (`Point.hpp:691-699`) are multiplications and never
truncate. In every case the port divides by the global and never multiplies by its reciprocal.
The banned-substitution list rejects idiom "improvements", naming rounding the scaling
conversions, substituting a standard hash map where iteration order reaches output, and
introducing a parallel reduction where the original's order is load-bearing. The document tells
porters to take the Rust type of every raw-pointer field from `port/ownership.tsv` verbatim.

`docs/WORKFLOW.md` states the unit of work — one libslic3r source file plus its header to one Rust
module, except the template-heavy and header-only files, which Step 20 hand-designs once before
fan-out; the two phases, where a Phase A draft is not required to compile and Phase B compiles
bottom-up by the crate-level level the cut defines, with compiler errors as the work queue; the
loop of one implementer, then two adversarial reviewers in separate contexts who each read the
diff **and** the vendored source file the module came from, are told to assume the module is
wrong, cite every divergence as `source:line` against `module:line`, and may not implement, then
one fixer; the rejection rule that a workaround needing a paragraph of justification is wrong
code; the conflict discipline, in the two forms the work actually takes; and the three-unit trial
run, which Step 21 fills in.

**The conflict discipline is written for both forms, because the fan-out is a hundred-plus task
branches built at once and not a shared worktree.** For agents sharing a worktree: commit named
files only, no `git stash` or `git reset`, no build commands, no slow commands. For concurrent
branches, a rule about files rather than commands — **a port task writes only its own modules**,
`crates/<crate>/src/<module>.rs` at the paths its row names, plus the crate's `lib.rs` `mod` line
in Phase B and its own test files. It writes none of these, which the repository regenerates for
it: `port/manifest.tsv` (status is derived from the tree, Step 11), `port/reports/perf-backlog.md`,
`port/reports/divergences.md`, `port/reports/order-containers.md`,
`port/reports/order-comparators.md` (all four owned by Step 14's regeneration pass), the root
`Cargo.toml` (a `crates/*` glob, Step 5), and `Cargo.lock` (no port task adds a dependency). The
document lists those seven paths, so a builder does not discover the rule on the first fan-out
pull request.

A Phase B task's `mod` line in `lib.rs` is the one shared file a port task does write, and it is a
one-line append that git merges; the document says to add the line in the crate's documented
alphabetical position so two branches appending to the same `lib.rs` conflict textually rather
than silently reordering.

It also states where a Phase A draft lives and why, because that is what keeps `main` building: a
draft is written at its final path, `crates/<crate>/src/<module>.rs`, and no `mod` declaration
names it, so `cargo build` never reads it and CI stays green while hundreds of non-compiling
drafts sit in the tree. Phase B's work on a module is to make it compile and add its `mod` line in
the crate's `lib.rs`. `just port-status` counts a trailer-carrying file that no `mod` line reaches
as a module gated out of the build, which is the burn-down R16 requires.

This step waits on Steps 11, 12 and 13.

### Step 16: Split the checks between the document job and the oracle job
- delivers: R23

`.asdlc/checks` gains the code checks that need neither the oracle binary nor a built libslic3r,
as a further line beside `just check`, which keeps its present meaning of linting the asdlc
documents. The new line is `just port-check`, a recipe that runs `just port-status --per-commit`,
`just inventory-check`, `just ownership-check`, `oracle/provenance --check` over the paths Step 3
names, and `cargo fmt --check`. Two more lines are added to that recipe later by the steps that
make them pass, the same way the macOS gates are: Step 17's substitute check and Step 18's
mock check. `just audit-check` and the perf-backlog whole-file diff are **not** in it — Steps 13
and 14 put those on the regeneration pass, because a whole-file diff of a generated report cannot
be a check that a hundred concurrent branches each pass.

Every check in `port-check` reads only the vendored source and the repository's own files, so a
task that touches no code still passes with the document checks alone, and the document checks
never wait on a build. The `-check` variants are the ones that regenerate into a scratch directory
and diff; the plain `just ownership`, `just audit-order`, `just inventory` and `just regenerate`
are for a person, or the regeneration pass, rewriting a committed file deliberately.
`just port-status` without `--per-commit` — the completion gate — runs in the macOS job beside the
comparisons, not here.

**A line in `.asdlc/checks` does not make CI run it.** That file is what a task is held to;
`.github/workflows/ci.yml`'s ubuntu job runs `just check` as a literal step and reads no line of
it, and `asdlc` has no command that does. So this step adds to the ubuntu job, by name:

- a step that runs `just port-check`, beside the existing `Checks` step that runs `just check`;
- a Rust toolchain step that installs the version `rust-toolchain.toml` pins — `dtolnay/rust-toolchain`
  given that version, or any action that honours the file — because `cargo fmt --check` is in
  `port-check` and that job sets up only `astral-sh/setup-uv@v7` and `extractions/setup-just@v3`
  today, so without it a task that touches no code fails the checks it is held to;
- a shallow submodule initialisation, `git submodule update --init --depth 1 vendor/OrcaSlicer`,
  because the new checks read the vendored tree, with the job's comment that nothing here reads
  the submodule corrected.

That is also what `README.md:34` already says the project intends — "Code checks get added to the
`check` recipe as code arrives" — so `README.md`'s checks and CI bullets are corrected here to
describe two recipes and the toolchain step rather than one recipe.

**The macOS arm64 job.** It is created here for everything that needs the oracle binary, a local
libslic3r build, or a comparison against either, and it is created with two lines only:
`just oracle-fetch` and `cargo test --workspace`. Every other line is added by the step that makes
it pass, because a job that fails on every pull request until the port is finished gates nothing:
`just oracle-build` by Step 7, `just oracle-parity` by Step 10, `just pr-body-check` by Step 21
(in the ubuntu job, since it needs no build), `just port-status` without `--per-commit` by Step 23,
and `just compare-stages`, `compare-benchy`, `compare-fixtures` and `compare-configurations` by
Steps 24 to 27. So eight later steps each add one line to a workflow file this step creates, and
each of them names its own line; those are ordinary merge conflicts between those eight, not a
mechanism problem, and naming them here is what keeps a builder from writing the whole job twice.

**What the macOS job runs on.** Not every pull request. `just oracle-build` is a dependency
superbuild plus the main project from zero downloads with no compiler cache, and the fan-out is a
hundred-plus pull requests, so the job carries a path filter: it runs when the pull request
touches `crates/**`, `oracle/**`, `port/**`, `fixtures/**`, `justfile`, `Cargo.toml`, `Cargo.lock`
or the workflow itself, and is skipped otherwise, which is how R23's promise that a document-only
pull request passes with the document checks alone stays true of the whole workflow and not only
of `.asdlc/checks`. It caches two things, both keyed on the submodule pin and the CMake flags
Step 7 records: the dependency superbuild's install prefix
`vendor/OrcaSlicer/deps/build/arm64/OrcaSlicer_dep`, which is the only part upstream's own
workflow keeps, and `vendor/OrcaSlicer/build/arm64`, the main project's build tree, without which
every pull request rebuilds libslic3r from scratch. Step 7 measures the first build's wall-clock
and disk cost, and if either exceeds the runner's limit or the cache's, the fallback stated here
is that `just oracle-build` and `just oracle-parity` run on `main` only and the per-pull-request
macOS job keeps `just oracle-fetch` and `cargo test --workspace`; the Risks section carries that
cost.

**The branch guard needs nothing from this step, and here is why**, so no builder discovers it on
the first fan-out pull request. `just guard` is `asdlc branch check`, which judges a branch against
its documents on `main`; a task's branch may change files outside `intent/` freely and the only
file it may add under `intent/` is its own plan, which a size-8 task starts with
`asdlc plan new <slug>/<task>`. A port task writes only `crates/**` and its tests, so it passes
the guard as it stands, and the manifest it takes its paths from does not have to be an asdlc
document for that to hold.

This step waits on Steps 5 and 14 — Step 5 because `cargo fmt --check` and
`cargo test --workspace` need the workspace it creates — and picks up the later recipes as the
steps above add them.

### Step 17: Bind the vendored and superbuild C and C++ libraries
- delivers: R17

Add `crates/slic3r-ffi`, whose `build.rs` compiles the vendored sources in
`vendor/OrcaSlicer/deps_src/` exactly as they are at the pin — unmodified, and with
`-ffp-contract=off`, which `build.rs` passes as `cxx_build::Build::flag("-ffp-contract=off")` on
every translation unit it compiles, Step 18's `cxx/include_vendored.cpp` included. That flag is
not an optimisation preference: C2's decision makes the library built with it the normative oracle
for every per-unit and per-stage proof, so a reference compiled without it is not the oracle R19's
proofs are supposed to compare against, and the compiler would fuse multiply-add expressions that
Rust never fuses. `just port-check` gains a check, written in this step, that the flag appears in
both places it has to — `oracle/build-libslic3r`'s `CMAKE_CXX_FLAGS` and this `build.rs` — so the
two cannot drift apart; it is one of the two lines Step 16 leaves to a later step.

The libraries it bridges with `cxx`:
Clipper2, whose vendored copy is patched so no published crate matches it, admesh, libnest2d and
qhull. Boost.Polygon's Voronoi builder, used at `src/libslic3r/Geometry/Voronoi.hpp` and
`Arachne/SkeletalTrapezoidation.hpp`, has no vendored source — Boost is a superbuild project under
`vendor/OrcaSlicer/deps/Boost/` — so it is bound against the version that recipe pins, from the
prefix Step 7 builds. NLopt is bound only if the manifest marks an in-scope caller; at the pin it
is absent from `src/libslic3r/CMakeLists.txt`'s `target_link_libraries`, reaches libslic3r only
transitively through libnest2d, and its only libslic3r includer is
`SLA/SupportTreeBuildsteps.cpp`, which is out of scope, so the expected outcome is that it is not
bound and the row says so.

Add to `just port-check` a check that fails when any crate manifest in the workspace depends on a
named substitute for one of these libraries, with the substitute list written into the check.
Replacing them with Rust implementations is out of scope for this intent; C4's decision leaves
that to a later intent.

This step waits on Step 5 for the workspace, Step 7 for the Boost prefix and the built library,
Step 11 for the crate map, and Step 16 for the `just port-check` recipe the two checks above are
added to.

### Step 18: The per-unit proof harness
- delivers: R19

Add `crates/slic3r-proof`, the harness every ported unit's test uses, and wire its three proof
paths so that a task for any file can say which one applies from its manifest row's `proof path`
column:

1. FFI to the library built from the pin, for externally linked symbols — `crates/slic3r-ffi`
   gains a link against `vendor/OrcaSlicer/build/arm64` from Step 7, which is the library built
   with `-ffp-contract=off` and therefore the oracle C2's decision makes normative.
2. The committed stage dumps from Step 8, read by `crates/slic3r-proof/src/dumps.rs`, for whole
   pipeline steps.
3. A test translation unit inside the FFI crate, `cxx/include_vendored.cpp`, that `#include`s the
   vendored `.cpp` for internal-linkage code: 49 of 268 `.cpp` files declare an anonymous
   namespace and 112 declare static free functions, none with external linkage, and the pin
   forbids adding an export. Including the vendored file does not modify it, so the pin holds.
   It is compiled by the same `build.rs`, so it carries `-ffp-contract=off` too; a proof against a
   translation unit compiled without it would be a proof against the wrong reference. Where even
   that is not possible the manifest row says so and the helper is proven through its enclosing
   file's exported surface.

`src/tiers.rs` applies the tier the unit's manifest row declares: exact equality of
integer-coordinate output for fixed-point units, the row's declared tolerance and comparison
function for float units. No test substitutes a stand-in for the code under test or for the
oracle, and `just port-check` fails a test module that mocks either — the check this step adds to
that recipe, by a rule it writes down rather than leaving to judgement: a file under
`crates/*/tests/` or a `#[cfg(test)]` module in a crate that holds a ported module fails when it
declares or names an identifier matching `mock`, `stub`, `fake` or `dummy` in any case, and there
is no allowlist, because the proof paths above make a stand-in unnecessary.

Add `crates/slic3r-ffi/fuzz/`, a `cargo-fuzz` crate, and the `just fuzz <target>` recipe that runs
one of its targets, with the differential fuzz target **shape**
that fixed-point units taking polygons or points use against the C++ through FFI: one target per
qualifying unit, at `fuzz/fuzz_targets/<crate>_<module>.rs`, generating arbitrary `Point`,
`Polygon` and `ExPolygon` inputs and asserting exact equality of integer-coordinate output between
the Rust module and the same call through FFI. No ported module exists yet at this step, so the
committed example target drives the FFI side alone — it round-trips arbitrary polygons through the
bridge and asserts the coordinates come back unchanged — which makes the crate, its
`Arbitrary` implementations and the `just fuzz <target>` recipe real rather than described. Step
20 commits the first truly differential target, against its first foundation module.

Which units need one is not left to a reviewer: Step 11's `fuzz target` column marks a row
`required` by the rule it states, Step 14's `port-status` fails a `required` unit with no target
naming its module, and Steps 22 and 23 carry it in their "Done when". C5's decision is that these
targets and the `// SAFETY:` rule are all this intent requires, and that sanitizers and Miri belong
to the following intent — which is why the targets are a check and not an aspiration.

This step waits on Steps 5, 7, 8, 11, 16 and 17.

### Step 19: The port's slicing surface and CLI skeleton
- delivers: R10

Add `crates/grampus-cli` with `src/surface.rs`, the GUI-free sequence the port exposes — build a
model, apply a config, validate, process, export G-code — mirroring what
`vendor/OrcaSlicer/tests/fff_print/test_helpers.hpp:85-108` uses, and nothing from
`src/slic3r/GUI/PartPlate`. The port runs no arranger: `init_print`'s own `arrange` parameter
(`test_helpers.hpp:86-89`, default `true`) is not part of the surface, because Step 4 fixes
placement in the fixture instead. C14's decision leaves arrange and orient to the following
intent.

`src/resolved_config.rs` reads the goldens Step 9 commits: bed shape, exclusion areas,
wipe-tower placement and filament maps reach the port as resolved configuration values, and the
Bambu-printer flag the oracle derives from the printer model is set explicitly by the caller,
because validation and G-code depend on it. The two keys that exist only in the G-code config
block, `first_layer_temperature` and `first_layer_bed_temperature`, are derived by the emission
path Step 9 specifies. `src/main.rs` is the binary Steps 24 to 27 call; until Step 23 lands it exits non-zero with
a named "not yet ported" error rather than a panic, and `tests/surface.rs` asserts the surface's
shape and the config round-trip.

This step waits on Steps 5, 9, 11 and 15.

### Step 20: Create the crates and hand-design the foundation types

Groundwork. It delivers no requirement by itself, and Steps 21, 22 and 23 need it. Like Steps 12,
22 and 23 it cannot be sized until `port/manifest.tsv` is on `main`, so it is cut spike-first.

**The crates come first, in one commit, and no port task ever creates one.** Nothing before this
step creates the roughly 43 crates the cut defines, and a hundred Phase A tasks each creating a
crate skeleton on its own branch is a hundred-way conflict. So this step adds `port/crate-skeletons`
and `just crate-skeletons`, which reads `port/cut.tsv` and writes, for every crate the cut names,
`crates/<crate>/Cargo.toml` and a `crates/<crate>/src/lib.rs` holding only the crate's doc comment
and the `mod` lines Phase B will fill — none at first. Re-running it adds a missing crate and
changes no existing file, so it is safe after the cut is regenerated. The root `Cargo.toml` needs
no edit, because Step 5 made it `members = ["crates/*"]`.

**Then the foundation types.** The template-heavy and header-only files are not file-units: they
are type-and-trait design, and every other unit depends on their shape, so they are designed once,
by hand, before fan-out, as `docs/WORKFLOW.md` states. The set is taken from `port/manifest.tsv`
rather than listed here: the in-scope rows the manifest's `template-heavy` column marks `yes`,
which is the column Step 11 adds for exactly this purpose, seeds by the rule Step 11 states, and
carries forward. The column is necessary rather than convenient: at the pin `Point.hpp`,
`Config.hpp`, `Model.hpp` and `libslic3r.h` all have a same-stem `.cpp`, so a "header with no
`.cpp` sibling" rule would reach only `AABBTreeIndirect.hpp` and leave the four heaviest
foundation headers to the fan-out. This step's spike is where the seeded set is corrected before
any task is cut from it. Header-only
files are 76 files and 14,840 of the library's 340,112 lines, 4.36 per cent, so the set is small
and the rest of the library stays a coherent file-per-module port.

These modules are ported, not drafted: each compiles, carries its `PORT STATUS` trailer and its
`mod` line, has its R19 proof, and has its differential fuzz target where its row says `required`.
The first such target is committed here, which is the one Step 18's harness left open. The scaling
entry points land here, which is why Step 15's per-API rule is written before this step runs.

**How the tasks are cut.** The first task cut from this step is a spike. It adds
`port/crate-skeletons`, commits the generated crate skeletons, runs `just task-cuts foundation`
and commits `port/foundation.tsv`: the `template-heavy` rows in crate-level order then path, with
the task's name, its unit count, its line sum and its unit paths, batched to at most 4 units or
2,500 lines — a smaller batch than Phase A's because each unit here is hand-designed, compiles,
and carries a proof. The remaining tasks are cut from that file in a later pass, one task per row,
each depending on the spike and on the tasks of the crates below it in the cut graph.

This step waits on Steps 5, 7, 11, 15, 17, 18 and 19.

### Step 21: The three-unit trial run
- delivers: R28

Port three units through the full loop of `docs/WORKFLOW.md` — one implementer, two adversarial
reviewers in separate contexts, one fixer — covering one unit of each equality tier: one
fixed-point unit held to exact integer-coordinate equality, one float unit held to its row's
declared tolerance and comparison function, and one unit with a non-const reference out-param and
a `throw`. The three are chosen from `port/manifest.tsv` by those columns.

The trial's output corrects `docs/PORTING.md`: whatever the three units show the contract gets
wrong is fixed in the document in this step's pull request, before any fan-out pull request is
opened. The trial is recorded in `docs/WORKFLOW.md` in the same form a unit's pull request uses:
both review verdicts, each citing `source:line` against `module:line`, and the module's manifest
row.

Add the check that holds every later unit to it: `port/port-status --pr-body <file>`, wired to its
own recipe `just pr-body-check <file>`, which is the one spelling used in this step, in the CI
step it adds and in this plan's Proof section. It fails unless the body records both review
verdicts in that form and names a manifest row that exists. It fails closed — an empty, absent or
non-conforming body is a failure, since that is the one case the check exists to catch — and it is
skipped only for a pull request that adds no module, which it decides from the diff and not from
the body. It runs as its own step of the ubuntu job of
`.github/workflows/ci.yml`, which writes the body to a file with
`gh pr view "$PR" --json body --jq .body` under the workflow's `GITHUB_TOKEN`, so this step also
adds `pull-requests: read` to the workflow's `permissions`. It reads the pull request and so cannot
live in `.asdlc/checks`, which a task also runs locally, and it needs no build.

This step waits on Steps 14, 15, 18 and 20, and **every fan-out step waits on it**.

### Step 22: Phase A — draft every in-scope unit

Groundwork. It delivers no requirement by itself: a Phase A draft is not required to compile, and
Step 23 is what turns these drafts into ported modules. Its units come from
`port/manifest.tsv`, which does not exist until Step 11 is on `main`, so this step cannot be
sized yet and is cut spike-first, then in several passes.

Each unit is one libslic3r `.cpp` plus its same-stem `.hpp` — one manifest row each, one unit
between them — drafted as `crates/<crate>/src/<module>.rs` at the path the manifest's target crate
and module columns name, with no `mod` declaration pointing at it, so `main` keeps building. A
draft follows `docs/PORTING.md`, carries its `PORT STATUS` trailer with
`confidence: high|medium|low`, its equality tier and the oracle that will prove it, and leaves
`// TODO(port):` rather than a guess.

**A draft is not exempt from the marker checks, and it does not need to be**, which is the thing
two builders would otherwise decide differently. `just port-status --per-commit` is text-based
(Step 14) and reads a file that need not be valid Rust. Its constant rule fires only on a constant
the draft actually writes, so the rule for a draft is: transcribe a numeric constant and it
carries a `// PORT SRC: <vendored path>:<line>` marker whose line really holds that literal; do
not transcribe it, and leave `// TODO(port):` in its place. Neither choice fails the check, and a
draft that invents a constant with no marker fails it, which is the case the check exists for.
`// SAFETY:` applies the same way, to the `unsafe` blocks the draft writes. `cargo fmt --check`
never sees a draft, because no `mod` line reaches it. `docs/WORKFLOW.md` carries this paragraph.

**How the tasks are cut.** The first task cut from this step is a spike. It runs
`just task-cuts phase-a`, the script Step 11 adds, and commits `port/taskcuts/phase-a.tsv`; the
remaining tasks of this step are cut from that file in later passes, one task per row, **one pass
per crate-level level**, because one draft message has to carry every task's file in full and
roughly a hundred rows do not fit in one. The script
orders the in-scope rows by `crate-level level` ascending, then file-level include-DAG level
ascending, then path — Phase A needs no order for correctness, so this one is chosen because it
is deterministic and groups a crate's units together — and batches consecutive units **within one
target crate** until either 6 units or 2,500 source lines is reached, whichever comes first. A
unit of more than 2,500 lines is a task on its own; `PrintConfig.cpp` at 14,438 lines and
`GCode.cpp` at 11,805 are the extreme cases, and the script marks a unit over 5,000 lines as
needing its own plan when it is cut. Rows the manifest marks out of scope get no task, and the
rows Step 20 already ported are skipped. Each row of `phase-a.tsv` carries the task's name, its
target crate, its crate-level level, its unit count, its line sum and its unit paths. A task of
this step is one task's worth of work — 6 units or 2,500 lines of straight transcription against a
contract that is already written down — and the cutter sizes it from that, not from guessing.

**A Phase A task's "Done when".** For every unit the row names, a file exists at the manifest's
target path with a `PORT STATUS` trailer naming that source path; no `mod` declaration reaches it;
`just port-status --per-commit` reports the unit's derived status as `drafted`;
`just port-check` passes; and the pull request body carries both review verdicts in the form Step
21 fixed, which `just pr-body-check` holds.

**The task writes no shared file.** Status is derived from the tree, so the task commits its own
modules and nothing else; the seven paths it must leave alone are the ones `docs/WORKFLOW.md`
lists (Step 15). Its branch changes nothing under `intent/`, so it passes `just guard` as Step 16
explains.

This step waits on Steps 11, 15, 20 and 21.

### Step 23: Phase B — compile bottom-up, crate by crate
- delivers: R26

Make every drafted module compile, in crate-level order, with compiler errors as the work queue,
until `just port-status` passes: no in-scope manifest row's derived status is other than `ported`,
and the burn-down of `todo!()`, `unimplemented!()` and modules gated out of the build is zero. That
state is what R21 and R22 are not claimed without. Its units come from `port/cut.tsv` and
`port/manifest.tsv`, so like Step 22 it is cut spike-first, then in several passes.

Work on a module here is to make it compile against the crate map, add its `mod` line to the
crate's `lib.rs`, and give it its R19 proof at the tier its manifest row declares, plus its
differential fuzz target where the row says `required`. Doing those is what makes its derived
status `ported`; there is no cell to move. A unit's module is not finished while a `todo!()`
remains in it. Where a draft turns out to be wrong the vendored source is re-read, not patched
around: the rejection rule in
`docs/WORKFLOW.md` is what settles that.

**How the tasks are cut.** The first task cut from this step is a spike. It runs
`just task-cuts phase-b`, the script Step 11 adds, and commits `port/taskcuts/phase-b.tsv`; the
remaining tasks are cut from that file in later passes, one task per row, **one pass per
crate-level level**, for the same draft-message reason Step 22 gives. The script orders crates by
`crate-level level` ascending, and within a crate orders units by file-level include-DAG level
ascending, then path.

One task per crate, except that a crate whose in-scope line sum exceeds 2,500 lines is split along
that order into consecutive batches of at most 6 units or 2,500 lines — the same batch as Phase A,
because the work per unit here is strictly harder: making it compile, adding its `mod` line,
giving it its R19 proof and its fuzz target where the row requires one. A unit longer than that is
its own batch; `PrintConfig.cpp` at 14,438 lines and `GCode.cpp` at 11,805 are the extreme cases.
Most crates are split, and the two mutual clusters the cut cannot break, `GCode.cpp` with
`GCode/*` and `Model` with `Format`, are each one crate and are split into many batches.

Each row carries the task's name, its crate, its **batch index**, its unit count, its line sum,
its unit paths and the crates it depends on. **Batch N of a crate depends on batch N-1 of the same
crate**, by that batch index, as well as on the tasks of every crate below it in the cut graph.
That serialisation is what makes a partial crate compile at all, and it works because of which
graph is cyclic: the cycle that forced the crate cut is at *module* granularity — 39 modules and
321,722 lines in one component — while the include graph **at file granularity is acyclic**,
verified by Tarjan over the library's sources (R8). So ordering a crate's batches by file-level
include-DAG level means every unit in batch N **includes** only units in batches 1 to N-1 of its
own crate, or units in crates below it, which is why the ordering is chosen. It is not a
guarantee: Rust's `use` graph is not the `#include` graph, and a C++ file at a lower include level
can be called by a higher one through a declaration it does not itself include. A batch that
therefore does not `cargo check` is evidence that the crate cut or the batch boundary is wrong,
and the answer is to re-cut it — `just task-cuts phase-b` after the cut is corrected — never to
stub the missing module out.

**A Phase B task's "Done when"**, in two forms, because a non-final batch of a split crate cannot
be asked to compile the crate:

- **Every batch.** Every unit the row names has its `mod` line in the crate's `lib.rs`, no
  `todo!()` or `unimplemented!()`, a passing R19 proof at its manifest tier, a fuzz target where
  its row says `required`, and a derived status of `ported` as `just port-status --per-commit`
  computes it; `cargo check -p <crate>` passes; `just port-check` passes; and the pull request
  body carries both review verdicts in the form Step 21 fixed.
- **The crate's last batch, additionally.** `cargo build -p <crate>` and `cargo test -p <crate>`
  pass. That is the batch the crate is finished at, and the one every crate above it in the cut
  graph depends on.

As in Phase A the task writes only its own modules, plus the one `mod` line per unit in the
crate's `lib.rs` at the documented position; it edits no manifest cell, no generated report, no
root `Cargo.toml` and no `Cargo.lock`.

The last task of this step adds `just port-status` — the gate, without `--per-commit` — to the
macOS job of `.github/workflows/ci.yml`, which is the moment that gate starts holding.

This step waits on Step 22. Steps 25, 26 and 27 wait on all of it; Step 24's command waits only on
its first stage-completing batch, and Step 24's CI line on all of it.

### Step 24: Compare the benchy stage by stage
- delivers: R20

Add `just compare-stages`, which slices the benchy fixture with the port and compares, for every
stage that is fully ported, the port's stage output against that stage's oracle dump from Step 8,
at the stage's declared tier, and reports the first stage that differs. The gate applies from the
first completed pipeline step and not from the first ported unit: no stage output exists until
every file that stage reaches is ported.

"Fully ported" is **derived**, not read: the command takes every in-scope manifest row whose
`pipeline stage` column names that stage — the column holds a stage name or `none`, and nothing
else — and the stage is fully ported when every one of those rows has a derived status of
`ported`, which `port/port-status` computes from the tree. The oracle for a stage dump is always
the locally built library, which C2's decision makes normative for every per-stage proof, so this
command has no oracle to choose.

For the stages with no seam — G-code emission and conflict check, which C11's decision accepts as
a gap rather than filling with a pseudo-stage dump — the command falls back to Step 5's levels on
the exported G-code, localised by the `; FEATURE:`, `; Z_HEIGHT:` and per-object label comments,
and says in its output that it is in fallback rather than reporting the last seam as the first
difference.

**This step is built during Step 23, not after it**, and that is the point of R20. C3's decision
makes the per-stage comparison on the benchy the *first* comparison against the oracle and the
end-to-end comparison the acceptance gate; a step that only exists once the whole port is finished
would deliver R20's words and none of its signal. So the two halves are ordered separately, and
the "waits on" line applies to each:

- **The command.** `just compare-stages` waits on Steps 5, 8 and 19 and on nothing else: Step 8's
  committed dumps and Step 19's surface are everything it reads, so it can be written and
  committed before any stage is complete. Those three are what a task cutter puts in its
  `depends`; no row of `port/taskcuts/phase-b.tsv` is labelled as completing a stage, so "the
  first stage to complete" is not a task anything can depend on. How many stages it prints a level
  for is a run-time property of the tree it runs against: a stage whose rows are not all `ported`
  prints `not ported`, not a failure, so the command is useful from the day it lands and gets
  stronger with every Phase B batch that merges.
- **The CI line.** `just compare-stages` joins the macOS job of `.github/workflows/ci.yml` after
  Step 23, like every other gate: it cannot pass while stages are missing, and Step 16's rule is
  that a gate joins the job in the step that makes it pass and not before.

### Step 25: Slice the benchy with the port and with the oracle
- delivers: R21

Add `just compare-benchy`, which slices `fixtures/3DBenchy.stl` with the port and with the oracle
from the same resolved config and the same instance transform, calls `oracle/assert-placement` and
fails when the oracle's run reports an object bounding box the fixture record does not name, and
reports the level reached. It passes when the port reaches the level
`port/reports/determinism.tsv` records as the base FFF configuration's G-code gate and matches
that configuration's statistics-JSON gate as well — the gate is read from that file at run time,
so if Step 6 measured it weaker than L1 this command moves with it.

**Which oracle it runs against is read, not hardcoded.** C2's decision makes the pinned release
binary normative for the end-to-end gates only where R6 shows the two oracles agree at L1, and the
built oracle normative for the fixture and configuration where they do not, with goldens kept per
oracle. So this command selects its oracle per fixture and configuration from
`port/reports/oracle-parity.tsv`'s `normative oracle` column, exactly the way it reads its gate
from `port/reports/determinism.tsv`, compares against that oracle's goldens, and **prints which
oracle it used** on every line, so a reader never has to infer it. A fixture and configuration
with no row in that file is a failure, not a default to the release binary.

C3's decision is that this end-to-end comparison is the acceptance gate rather than the first
output, and that no throwaway bridge to the vendored C++ is built to get an earlier one.

This step adds `just compare-benchy` to the macOS job of `.github/workflows/ci.yml`.

This step waits on Steps 4, 5, 6, 9, 19 and 23.

### Step 26: Every committed fixture
- delivers: R22

Add `just compare-fixtures`, which runs the port against the oracle on each fixture in the
committed set with the Bambu Lab X1 Carbon 0.4 nozzle machine, the `0.20mm Standard @BBL X1C`
process and the `Bambu PLA Basic @BBL X1C` filament, calls `oracle/assert-placement` for each, and
reports the level reached; it passes when each reaches the level `port/reports/determinism.tsv`
records as that configuration's G-code gate. It selects its oracle per fixture from
`port/reports/oracle-parity.tsv` and prints which one it used, by the same rule Step 25 states,
and compares against that oracle's goldens.

C6's decision makes this the measure of "usable to slice files for 3D printing" for this intent:
the port slices a mesh plus a resolved configuration, the project-format and preset layers stay
out, and the owner's own print on an X1 Carbon stays their own act and is not a requirement here.

This step adds `just compare-fixtures` to the macOS job of `.github/workflows/ci.yml`.

This step waits on Step 25.

### Step 27: Every configuration
- delivers: R27

Add `just compare-configurations`, which runs the port against the oracle for each configuration
named in `docs/ORACLE.md` — base FFF, tree support, wipe tower and multi-filament, fuzzy skin —
and fails unless each reaches, for each compared artifact, the level
`port/reports/determinism.tsv` records as that configuration's gate for it. It selects its oracle
per configuration from `port/reports/oracle-parity.tsv` and prints which one it used, by the same
rule Step 25 states. Where a gate is `none` the command reports the divergence and does not fail,
which is C1's decision: the pin holds, the oracle is not patched, the port is deterministic, and
each divergence is named and checked at the gate Step 6 measured.

The command fails if a divergence it reports has no row in `port/reports/divergences.md`, so the
two cannot drift apart. It writes no row itself: the `none` gates it reports reach that file
because `port/reports/determinism.tsv` is one of the four sources Step 13's `just audit-order`
regenerates from, which is why those rows survive a regeneration.

This step adds `just compare-configurations` to the macOS job of `.github/workflows/ci.yml`.

This step waits on Steps 6, 13 and 26.

## Risks

**The riskiest step is Step 7, building the vendored libslic3r on this platform.** `SLIC3R_GUI=OFF`
is supported by the upstream CMake logic and used by one upstream script, but only inside a Linux
docker image, only for `libslic3r_tests`, and never on macOS: no upstream workflow sets it, and
nobody anywhere builds `fff_print_tests` that way. The dependency superbuild still requires most
of the heavy libraries with the GUI off, including a bundled CPython 3.12.13, and nothing is
prebuilt here, so the first build starts from zero downloads with no compiler cache. If it cannot
be made to work, R5, R6, R19 and R20 lose their per-stage oracle, two of R19's three proof paths
go with it, and no single ported unit can be proven: C7 is that concern, and its decision is that
this is reported and the dependent work stops rather than being worked around. Steps 8, 10, 18,
20, 21, 22, 23 and 24 all sit behind it, which is why it starts on day one in parallel with two
tracks that do not need it.

**That build's cost is paid per pull request, and the fan-out is a hundred-plus of them.** Step 16
puts `just oracle-build` on a macOS arm64 runner, and the first build starts from zero downloads
with no compiler cache, so an uncached run on every port pull request is the plan's largest
unmeasured operational cost. The mitigations are in Step 16 and each has a failure mode: the path
filter, which does not help because every port pull request touches `crates/**`; the two caches —
the superbuild prefix and `vendor/OrcaSlicer/build/arm64` — which help only while they stay inside
the runner's cache limit and the pin and flags are unchanged; and the stated fallback, that
`just oracle-build` and `just oracle-parity` move to `main` only if the measurement Step 7 records
makes the per-pull-request job impractical. That fallback costs the fan-out its per-unit proof in
CI, leaving it to the local run and the `main` job, and it is named here so the choice is made on
Step 7's measurement rather than discovered.

**`--arrange 0` is read from the pin and not yet run.** Every oracle invocation in this plan
carries it, and R24 rests on it, but the one recorded run omitted `--arrange` entirely and let the
CLI's arranger place the mesh. The option and its handler are read at the pin
(`PrintConfig.cpp:12935-12940`, `OrcaSlicer.cpp:5072-5078`), which is the standard this plan holds
itself to, but the combination of a disabled arranger with a fixture the export script placed is
first exercised in Step 2 and Step 4. It is C12's sibling: if the pinned CLI will not slice a mesh
without arranging it, Steps 2, 4, 6, 9, 10, 25 and 26 all move to Step 8's built-oracle driver,
which constructs the model in code and needs no CLI transform at all, and the release binary stops
being usable as an end-to-end oracle — which would make C2's decision resolve to the built oracle
everywhere. Step 2's "Done when" is that invocation exiting 0 with a non-empty `plate_1.gcode`, so
this is found on day one rather than at Step 25.

**Step 11's cut decides the whole order of work.** A 62-edge feedback arc set carrying 156 of
2,696 include references makes the module graph acyclic, and most of those edges are stray single
includes, but which edges get cut is the crate map. C10 owns that choice; the mitigation is that
the script computes and commits the cut, every level is derived from it, and the cut is reviewed in
the pull request that commits it.

**Step 6 may come back weaker than L1 for the non-base configurations, and that is a result, not a
failure.** A float reduction over a concurrently filled vector feeds tree-support geometry, a
float sum runs over hash iteration order in the wipe tower, and fuzzy skin is seeded per thread
from entropy, so no Rust port can reach L1 against the oracle there. C1 and C12 carry it: the
gates move with the measurement and the divergences are named. The related risk is C12's other
half — that a configuration cannot be constructed through the pinned CLI at all, as the one
attempt on record failed on a 2.4.2 build with exit 239, 251 or 254 and no diagnostic. Step 8's
driver is the fallback, which is why Step 6 waits on Step 8 for those rows.

**Step 10 may show the two oracles disagree**, in which case C2's decision splits the goldens per
oracle and adds divergence rows, and every later comparison has to say which oracle it ran
against. The expectation that a fused multiply-add cannot move a nanometre-grid coordinate is an
argument, not a measurement on this library.

**Steps 22 and 23 are the bulk of the work and the hardest to keep honest.** Scope is around
245,000 lines across roughly 420 files, and C8 records that nothing in this plan makes that
smaller. The specific failure modes are the ones the Bun port shipped despite a green suite —
ownership transfer, destructor against `Drop`, and widened or narrowed error sets — which is why
Step 12's ownership manifest, Step 15's per-API scaling and error-set rules and Step 21's trial
come before any fan-out, and why Step 14's burn-down to zero and Step 18's per-unit differential
proof are checks and not sentiments. C5 is the residual: `// SAFETY:` comments and differential
fuzz targets are all this intent requires, and undefined behaviour reachable from safe code can
survive them.

**The second failure mode of a hundred-plus parallel branches is the shared file, not the wrong
translation.** The conflict surface, in one place: the manifest's progress record, the four
generated reports, the root `Cargo.toml` members list, `Cargo.lock`, and `.github/workflows/ci.yml`
and the `justfile`, which eighteen of the twenty-seven steps edit. The first four are removed as
conflicts by mechanism rather than by discipline — status is derived from the tree (Step 11), the
reports are regenerated by one pass on `main` and a task passes a per-marker assertion instead
(Step 14), the root manifest is a `crates/*` glob (Step 5), and no port task adds a dependency.
The last two stay ordinary merge conflicts between the steps that add a line to them, which is why
each such step names its own line: Steps 7, 10, 21, 23, 24, 25, 26 and 27 for the CI file. The
residual risk is that one of the four mechanisms is implemented as a whole-file diff anyway, in
which case the fan-out stalls on red checks after every merge; `docs/WORKFLOW.md`'s list of the
seven paths a port task may not write is what a reviewer checks that against.

**Step 24's blind spot is the largest in-scope subsystem.** `SlicingPipelineStepPlugin` has no
value for `psGCodeExport` or `psConflictCheck`, so a divergence anywhere in the G-code modules —
tens of thousands of lines, and the ones that produce the artifact Steps 25 and 26 compare — can
be detected but not localised to a stage. C11 owns it and its decision accepts the fallback.

Smaller ones, each named where it bites: the goldens are valid for macOS arm64 only (C2), and
Step 16's second CI job is the only place they are checked; the port still links C and C++ at the
end of this intent (C4); `SLA/IndexedMesh` is ported although the intent says SLA is out (C9);
3MF projects and preset resolution stay out, so the port slices a mesh plus a resolved config and
not a project from the slicer's own workflow (C6); and arrange and orient are deducted although
the application calls them (C14).

## Proof

- **R1** — `just oracle-fetch` exits 0 and prints the asset's sha256 and `2.5.2`; run with the
  recorded hash altered by one character it exits non-zero and prints the expected and measured
  values; `grep -r '/Applications/' justfile oracle port` finds nothing.
- **R2** — `just oracle-slice vendor/OrcaSlicer/tests/data/frog_legs.obj base-fff` exits 0 and
  prints the path and byte size of a non-empty `plate_1.gcode`, and does so before Step 4 exists,
  which is why the proof names a vendored path rather than a frozen fixture; after Step 4,
  `just oracle-slice 20mm_cube base-fff` does the same. The negative case is a configuration row
  with the machine and process presets removed, so `--load-settings` is omitted: the command exits
  non-zero and prints the named failure `CLI_VALIDATE_ERROR` for exit `-51` arriving as 205, which
  is the failure on record. A row with its filament path removed is the other negative, and it is
  the *script* that fails it, before the binary runs, naming the row — because the oracle itself
  exits 0 on a missing `--load-filaments` and silently slices with a different filament.
- **R3** — `just oracle-determinism` regenerates `port/reports/determinism.tsv` and
  `port/reports/determinism.md` from it and prints one line
  per configuration and artifact with its run count and gate; running it twice leaves the
  committed file unchanged apart from its timestamp; a person reads the base FFF row and finds a
  run count of at least five and a gate.
- **R4** — `cargo test -p oracle-compare` passes `tests/levels.rs`, `tests/normalisation.rs` and
  `tests/json.rs`; `just compare a.gcode b.gcode` on two real runs of Step 2 prints every level
  that passes, the strongest, and the first failing location at the next level, and on two
  unrelated files prints `none` with the first differing byte.
- **R5** — `just oracle-dump 20mm_cube base-fff` writes one dump per fired seam and prints their
  count; the check in the same recipe prints `L1` for the armed build's G-code against the
  unarmed build's and exits non-zero at any weaker level; `grep -c '^; ' <dump>` shows the
  provenance header, which a stage dump carries as a `;` comment block by Step 3, and a person
  reads one dump and finds integer coordinates.
- **R6** — `just oracle-parity` regenerates `port/reports/oracle-parity.tsv` and
  `port/reports/oracle-parity.md` from it and prints the level the two oracles reach per fixture
  and which oracle each row makes normative; a person reads the report and finds the built oracle's
  CMake flags including `-ffp-contract=off`, and a row for every difference found.
- **R7** — `oracle/provenance --check` over the paths Step 3 names — `port/goldens/`, the stage
  dumps, `port/reports/determinism.md` and `port/reports/oracle-parity.md` — exits 0 and prints the
  number of files checked; it exits non-zero on a file with a missing or stale header, and
  non-zero when it is pointed at one of the four static-analysis reports, which carry no oracle and
  no fixture and are excluded by name; a person reads one golden's header and finds all nine
  recorded items.
- **R8** — `just inventory` regenerates `port/manifest.tsv` and `port/cut.tsv` and prints the row
  count and the cut's edge count; `just inventory-check` exits 0 and prints nothing on a clean
  tree, and non-zero with a diff when a *derived* column is edited by hand — it prints nothing when
  a carried-forward column such as `tolerance` is changed, which is how a port task's pull request
  passes it, and it never compares the status column; running `just inventory` after a tier or
  tolerance has been set leaves those cells exactly as they were; the script's own assertions
  print the presence of `LayerRegion.cpp`, `ContourZ.cpp` and `BeltPurge.cpp`.
- **R9** — `awk -F'\t' '$1 ~ /SLA\//' port/manifest.tsv` prints every `SLA/` file with its own in
  or out decision and reason; a person checks that `SLA/IndexedMesh.cpp` and `.hpp` are in and
  that `SLA/Hollowing.cpp`, `SLA/Pad.cpp` and `SLA/ConcaveHull.cpp` are out.
- **R10** — `cargo test -p grampus-cli` passes `tests/surface.rs`, which asserts the sequence the
  surface exposes and that no arranger is reachable from it; `grep -r 'PartPlate' crates/` finds
  nothing.
- **R11** — `cargo test -p grampus-cli` passes the resolved-config round-trip in
  `tests/surface.rs`; `oracle/provenance --check port/goldens/configs` exits 0;
  `grep -o '"printhost_apikey": *"[^"]*"' port/goldens/configs/*.json` prints the key with an
  empty value for every golden, which is the value and not a match count; and a person reads
  `docs/ORACLE.md` and finds the two derived G-code-only keys named.
- **R12** — `just port-status --per-commit` exits 0 and prints that `docs/PORTING.md` holds the
  crate map, type
  map, idiom map, don't-translate list, banned-substitution list and the six rules; it exits
  non-zero when a section is removed; a person reads the scaling rule and finds one statement per
  API with its citation.
- **R13** — `just port-status --per-commit` exits 0 and prints that `docs/WORKFLOW.md` holds the
  unit of work,
  the two phases, the loop, the rejection rule, the conflict discipline and the trial-run record;
  it exits non-zero when one is removed.
- **R14** — `just port-status --per-commit` exits 0 and prints the count per marker kind; it exits
  non-zero on a module whose trailer is missing, whose source path does not exist at the pin or
  disagrees with its manifest row, on an `unsafe` block with no `// SAFETY:`, on a numeric constant
  with no `// PORT SRC:`, and on an unrecognised port marker.
- **R15** — `just regenerate` rewrites `port/reports/perf-backlog.md` from the `// PERF(port):`
  markers and prints the marker count, and the regeneration pass's own check exits non-zero when
  the committed file differs from a fresh regeneration;
  `just port-status --per-commit` exits non-zero on a `// PERF(port):` marker that is malformed or
  has no row in that report, which is the per-marker form a task passes instead; a person reads
  `.asdlc/checks`, the `justfile` and `.github/workflows/ci.yml` and finds no check that gates on
  throughput, instruction count or allocation.
- **R16** — `just port-status` prints the counts of `todo!()`, `unimplemented!()` and modules gated
  out of the build, and exits 0 with all three at zero at the end of Step 23, which is before R21
  is claimed; `just port-status --per-commit` exits non-zero on a numeric constant whose
  `// PORT SRC:` line in the vendored file does not contain that literal.
- **R17** — `cargo test -p slic3r-ffi` passes `tests/ffi_smoke.rs`, which calls into Clipper2,
  admesh, libnest2d, qhull and Boost.Polygon's Voronoi builder through the bridge; `just
  port-check` exits non-zero when a crate manifest is given a dependency on a named substitute for
  one of them, and non-zero when `-ffp-contract=off` is missing from either
  `oracle/build-libslic3r`'s `CMAKE_CXX_FLAGS` or `crates/slic3r-ffi/build.rs`, so the two
  compilations the proofs compare against cannot drift apart.
- **R18** — `just audit-order` regenerates `port/reports/order-containers.md`,
  `port/reports/order-comparators.md` and `port/reports/divergences.md` and prints the site count
  of each and the count it took from each of its four named sources; `just audit-check` then exits
  0 and prints nothing, and non-zero with a diff when one of the three is edited by hand, which is
  the regeneration pass's check and not a task's; `just port-status --per-commit` exits non-zero on
  a `// FIDELITY(port):` marker that is malformed or has no row in `divergences.md`; and a person
  reads the two audit reports and finds, for each site, the port's choice and whether it diverges.
- **R19** — `cargo test --workspace` passes, and each ported unit's test names its proof path; a
  person picks a manifest row, reads its `proof path` column and finds the matching test in the
  module's crate.
  `awk -F'\t' 'NR==1{for(i=1;i<=NF;i++)c[$i]=i} NR>1 && $c["fuzz target"]=="required"' port/manifest.tsv`
  prints every row that needs a differential fuzz target, and for each of them
  `ls crates/slic3r-ffi/fuzz/fuzz_targets/<crate>_<module>.rs` finds it;
  `just port-status --per-commit` exits non-zero when a `required` row has no such target, and
  `just fuzz <target>` runs one. `just port-check` exits non-zero on a test module that names a
  `mock`, `stub`, `fake` or `dummy` identifier, which is the rule Step 18 writes down.
- **R20** — `just compare-stages` prints one line per stage, with the level reached for a stage
  whose every manifest row has a derived status of `ported` and `not ported` for the rest, names
  the first stage that differs, and prints `fallback: exported G-code` for G-code emission and
  conflict check rather than naming `psSkirtBrim` as the difference; it is run and prints a
  level for at least one stage before Step 23 ends, which is what makes it the first comparison.
- **R21** — `just compare-benchy` exits 0 and prints the level reached for the G-code and the
  statistics JSON, the gate it read from `port/reports/determinism.tsv`, and the name of the
  oracle it read from `port/reports/oracle-parity.tsv` and ran against; it exits non-zero when the
  oracle's bounding box does not match the fixture record, and when the fixture and configuration
  have no row in `oracle-parity.tsv`.
- **R22** — `just compare-fixtures` exits 0 and prints one line per fixture with the level
  reached, the gate it was held to and the oracle it ran against.
- **R23** — `cat .asdlc/checks` shows `just check` and `just port-check`; `just port-check` exits 0
  on a machine with no oracle binary and no libslic3r build, and at every point in the work,
  including while in-scope manifest rows' derived status is still `not-started`. A person reads
  `.github/workflows/ci.yml` and finds, in the ubuntu job, a `Checks` step running `just check`,
  a step running `just port-check`, a step running `just pr-body-check`, and the toolchain and
  shallow-submodule steps they need — because a line in `.asdlc/checks` does not make CI run it —
  and finds every check that needs the oracle binary, a libslic3r build or a comparison against
  either in the macOS arm64 job, with that job's path filter beside it. A pull request that
  changes only a document passes with the document checks alone and does not start the macOS job.
- **R24** — `grep -n 'arrange' oracle/slice` shows `--arrange 0` passed on every invocation, and
  Step 2's run with it exits 0 with a non-empty `plate_1.gcode`; `oracle/assert-placement` exits
  non-zero on a run whose bounding box the fixture record does not name, which
  `just compare-benchy` and `just compare-fixtures` both call; `grep -rn 'align.xy\|--center' oracle/`
  finds nothing, because neither option exists at the pin and the XY placement is
  `oracle/export-fixture`'s own translation; a person reads `vendor/README.md` and finds one
  resolved instance transform per fixture, which is the identity after that translation.
- **R25** — `ls fixtures/` prints exactly `3DBenchy.stl`, `20mm_cube.stl` and `frog_legs.stl`; a
  person reads `vendor/README.md` and finds, for each, its frozen basename, the vendored path it
  came from, its transform and its license, with the absence of an upstream license for
  `resources/handy_models/` stated; every `docs/ORACLE.md` configuration row names the fixtures it
  uses.
- **R26** — `just port-status` exits 0 and prints `0 in-scope rows remaining`; before Step 23 ends
  the same command exits non-zero and prints how many remain, which is why it is the gate the
  macOS job gains in Step 23 and not one of the checks a task passes.
- **R27** — `just compare-configurations` exits 0 and prints one line per configuration and
  artifact with the level reached, the gate it was held to and the oracle it ran against, reports a
  `none` gate as a named divergence without failing, and exits non-zero when a reported divergence
  has no row in `port/reports/divergences.md`.
- **R28** — `just pr-body-check <file>` prints both recorded verdicts and the manifest row
  it found and exits 0; it exits non-zero on a body with one verdict, with no manifest row, and on
  an empty body, for a pull request whose diff adds a module; a person reads `docs/WORKFLOW.md` and
  finds the three-unit trial recorded in the same form, dated before the first fan-out pull
  request.
- **R29** — `just port-status --per-commit` exits non-zero and names the missing section when a
  heading is removed from `docs/PORTING.md` or `docs/WORKFLOW.md`, and exits 0 with both complete.
- **R30** — `just ownership-check` exits 0 and prints the row count; it exits non-zero when a
  raw-pointer field in an in-scope header has no row, when a row names a field that does not exist
  at the pin, when a row other than `unknown` has no evidence, when an `unknown` row has no
  `path:line` for what leaves its class unclear or no conservative Rust type, and — once Step 12's
  last task has removed the allowance — when any row's class is blank; `just port-status` prints
  the count of `unknown` rows.
