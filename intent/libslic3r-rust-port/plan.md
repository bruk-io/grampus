---
kind: plan
---
# Plan: Port libslic3r to Rust with 1:1 functionality

Every area of concern in the spec, C1 to C14, carries a recorded Decision, so **no step in this
plan waits on a concern**. Where a decision changes the order of work it is named in the step
that carries it out.

Two steps cannot be sized until earlier work is on `main`, because their unit lists come out of
`port/manifest.tsv`, which does not exist yet: Step 22 (Phase A drafts) and Step 23 (Phase B
compiles). Each of those steps says which column orders its units, how many units go in a task,
and what a port task's "Done when" is, and each is cut spike-first so the rest is cut from a
committed file once the manifest is merged.

The research notes behind the spec live outside this repository and may disappear, so every
command, flag, asset name, hash and build trap this plan relies on is written out here.

## Files that change

- Cargo.toml (new)
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
- port/ownership (new)
- port/ownership.tsv (new)
- port/audit-order (new)
- port/port-status (new)
- port/cut-tasks (new)
- port/taskcuts/phase-a.tsv (new)
- port/taskcuts/phase-b.tsv (new)
- port/goldens/ (new)
- port/reports/determinism.md (new)
- port/reports/oracle-parity.md (new)
- port/reports/order-containers.md (new)
- port/reports/order-comparators.md (new)
- port/reports/divergences.md (new)
- port/reports/perf-backlog.md (new)

Nothing under `vendor/OrcaSlicer/` changes, and the submodule pointer stays at
`f92a7c065c5718b8ef1baac1cd704df2911238f4`. Scripts outside the Rust workspace
(`oracle/*`, `port/*` without an extension) are standalone executables with PEP 723 inline
metadata, run through `uv`, as the repository's conventions require.

## Order of work

Three tracks start at once and share nothing:

- **The release-binary oracle:** Steps 1, 2, 3, 4, then 9 and 6. Step 5 (the comparator) joins
  after Step 2, and Step 9 (the resolved-config goldens) after Steps 2 and 3.
- **The locally built oracle:** Step 7, then 8, then 10. Step 7 starts immediately; it reads only
  the vendored tree.
- **The manifest:** Step 11, then 12 and 13 side by side, then 15, then 14. Step 11 reads only
  the vendored tree and needs no oracle.

Steps 16 to 21 join the tracks: 16 (checks and CI) after 14, 17 (FFI) after 11, 18 (proof
harness) after 7, 8 and 17, 19 (the port's surface) after 15, 20 (foundation types) after 19,
21 (the trial run) after 14, 15, 18 and 20.

**No port unit starts before Step 7 works.** That is C7's decision, and it covers Steps 20, 21,
22 and 23. Steps 22 and 23 also wait on Step 21. The end-to-end gates, Steps 24 to 27, wait on
Step 23 and on Step 6's gate table.

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

Add `oracle/slice` and the `just oracle-slice <fixture> <configuration>` recipe. The script
resolves `<configuration>` to a row in the configuration table of `docs/ORACLE.md`, `cd`s into a
fresh scratch directory (a failing run writes `00000.log` into `$PWD`, which would dirty the
working tree), and invokes the binary from Step 1 as:

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
`-o`/`--output`, the name is `plate_<N>.gcode`; and `--datadir` is fresh per run, which also
keeps the seven `printhost_*` credential fields of an exported config empty.

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

The header is a comment block whose syntax suits the artifact: `;` lines for G-code goldens, a
`provenance` object for JSON, a Markdown table for reports. `oracle/provenance --check <file>`
exits non-zero when a file under `port/goldens/` or `port/reports/` has no header or has one that
disagrees with the pin, and that check joins `just port-check` in Step 16.

### Step 4: Freeze the fixture set and fix placement
- delivers: R24, R25

Add `oracle/export-fixture` and `just fixture-export`, which asks the binary from Step 1 to
export a vendored model as STL, already positioned on the target plate, and writes it to
`fixtures/` under its frozen basename. The basenames are frozen because the model's basename —
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
to every mesh input.

### Step 5: The comparator
- delivers: R4

Add the workspace (`Cargo.toml`, `rust-toolchain.toml`, `/target/` in `.gitignore`) and
`crates/oracle-compare`, a library plus a binary, with `just compare <a> <b>` wired to the
binary. It takes two G-code files and evaluates L0 to L4 independently, reporting every level
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
and regenerates `port/reports/determinism.md`. The report has one row per configuration and one
sub-row per compared artifact — the G-code, the `--export-settings` JSON and the
`--export-statistics` JSON. Each sub-row names the normalisation R4 declares for that artifact
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

### Step 8: Dump the pipeline's intermediate state
- delivers: R5

Add `oracle/stage-dump/`, a C++ program plus its `CMakeLists.txt`, linked against the
`libslic3r` built in Step 7, and `just oracle-dump <fixture> <configuration>`. It drives the
library through the sequence `vendor/OrcaSlicer/tests/fff_print/test_helpers.hpp` uses — build a
model, apply a config, validate, process, export G-code — and registers a hook with
`Print::set_slicing_pipeline_hook_fn` (`vendor/OrcaSlicer/src/libslic3r/Print.hpp:1060-1063`).
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
compares with Step 5's comparator, and regenerates `port/reports/oracle-parity.md`. The report
records whether the two reach L1 against each other, lists every difference found, and names the
built oracle's exact CMake flags, including `-ffp-contract=off`.

C2's decision is what the report feeds: the built library is normative for every per-unit and
per-stage proof and macOS arm64 is the one platform goldens are valid for; for the end-to-end
gates the release binary is normative where this report shows L1. Where it does not, the built
oracle becomes normative for that fixture and configuration, the disagreement becomes a row in
`port/reports/divergences.md`, and goldens are kept per oracle. The step writes those rows and
says so in the report rather than leaving the reader to infer it.

This step waits on Steps 4, 5, 7, 8 and 9.

### Step 11: The scope manifest and the crate cut
- delivers: R8, R9

Add `port/inventory`, `just inventory` and `just inventory-check`, which regenerate
`port/manifest.tsv` and `port/cut.tsv`; `just inventory-check` regenerates into a scratch
directory and fails when the committed files differ, so re-running leaves them unchanged. This
step reads only the vendored source and starts on day one alongside Steps 1 and 7.

`port/manifest.tsv` has one row per libslic3r source file at the pin, with columns: path,
subsystem, lines, file-level include-DAG level, crate-level level, in scope or out with a reason,
target crate and module, pipeline stage or `none`, equality tier, tolerance, comparison function,
proof path, and status. Status starts at `not-started` and later becomes `drafted` then `ported`.
The file set is one row per `.h`, `.hpp`, `.cpp` or `.mm` file under
`vendor/OrcaSlicer/src/libslic3r`, including `MacUtils.mm` and `Format/ModelIO.mm`, which
`src/libslic3r/CMakeLists.txt:604-610` appends under `if (APPLE)`, and excluding the generated
`libslic3r_version.h`, which exists only as `libslic3r_version.h.in`. The script asserts the row
count against the set it finds on disk, not against a number written in a document.

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

The manifest also carries a column for coverage-measured execution during a benchy slice,
populated once Step 7 exists. It is not a precondition for anything, and nothing downstream gates
on it.

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
order. Each site records the port's choice and whether it diverges from the oracle. The command
also regenerates `port/reports/divergences.md` from those rows plus every `// FIDELITY(port):`
marker in the tree, so the file names every place the port differs from the oracle and the level
it is checked at instead; Steps 10 and 27 add to it through the same command.

The port's choices, which `docs/PORTING.md` then carries: sum in a fixed order, use total-order
comparators, and use insertion-ordered maps wherever iteration order reaches output, so the port
is deterministic where the oracle is not. Each such site in the port carries a
`// FIDELITY(port):` marker, which is what makes the regeneration possible. `just audit-check` is
the drift-detecting variant Step 16 puts in the checks a task passes: it regenerates all three
reports into a scratch directory and fails when a committed one differs.

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
not contain that literal. It fails while any in-scope manifest row has a status other than
`ported`, and prints how many rows remain, which is the manifest-to-module direction Step 23
closes. It prints the count of `unknown` rows in `port/ownership.tsv`.

It fails unless `docs/PORTING.md` contains the sections R12 names and `docs/WORKFLOW.md` contains
the sections R13 names, reading headings and not prose quality. And it regenerates
`port/reports/perf-backlog.md` from the `// PERF(port):` markers and fails when the committed file
differs: that report is the only place a knowingly slower translation lives, and the handover to
the following intent. No check this step adds gates on throughput, instruction count or
allocation, and none is added anywhere else in this plan.

`just port-status` with no argument is all of the above, including the burn-down to zero and the
in-scope-rows-remaining gate, which is how R16 and R26 are worded and which is therefore true
only at the end of Step 23. It is not what a task's pull request passes. The script also takes
`--per-commit`, which runs exactly the subset R23 puts in `.asdlc/checks` — the trailers and their
agreement with the manifest row, the source paths at the pin, `// SAFETY:`, `// PORT SRC:` and the
placeholder-constant check, the marker whitelist, the document sections, the `unknown` count and
the perf-backlog drift — and which passes while rows are still `not-started` or `drafted`, because
a module that is honestly a draft is not a fault. It prints the burn-down and the remaining row
count in both modes and fails on them only without the flag. Step 16 puts `--per-commit` in the
checks a task passes, and Steps 25, 26 and 27 call the gate.

This step waits on Steps 11, 12, 13 and 15, because it checks their artifacts.

### Step 15: The translation contract and the workflow document
- delivers: R12, R13

Write `docs/PORTING.md` and `docs/WORKFLOW.md`, both committed before any unit is ported.

`docs/PORTING.md` carries the crate map, generated from `port/cut.tsv` and which agents may not
invent; a type map; an idiom map; a don't-translate list; a banned-substitution list; and the six
rules the spec states, transcribed with their citations. The scaling rule is stated per API,
because the five entry points do not behave alike: `scaled<coord_t>()` and the Eigen overload
(`Point.hpp:669-690`) are truncating division by the `SCALING_FACTOR` global and are ported as
truncating division, with rounding banned — the `std::round` on `Point.hpp:677` is commented out
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
code; the conflict discipline for agents sharing a worktree — commit named files only, no
`git stash` or `git reset`, no build commands, no slow commands; and the three-unit trial run,
which Step 21 fills in.

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
as further lines beside `just check`, which keeps its present meaning of linting the asdlc
documents. The new line is `just port-check`, a recipe that runs `just port-status --per-commit`,
`just inventory-check`, `just ownership-check`, `just audit-check`,
`oracle/provenance --check` over `port/goldens/` and `port/reports/`, and `cargo fmt --check`.
Every one of those reads only the vendored source and the repository's own files, so a task that
touches no code still passes with the document checks alone, and the document checks never wait on
a build. The `-check` variants are the ones that regenerate into a scratch directory and diff, so
CI detects drift instead of rewriting a committed file and leaving the tree dirty; the plain
`just ownership`, `just audit-order` and `just inventory` are for a person regenerating
deliberately. `just port-status` without `--per-commit` — the completion gate — runs in the macOS
job beside the comparisons, not here.

`.github/workflows/ci.yml` gains a shallow submodule initialisation in the existing ubuntu job —
`git submodule update --init --depth 1 vendor/OrcaSlicer` — because the new checks read the
vendored tree, and its comment that nothing here reads the submodule is corrected. It gains a
second job on a macOS arm64 runner for everything that needs the oracle binary, a local
libslic3r build, or a comparison against either: `just oracle-fetch`, `just oracle-build`,
`cargo test --workspace` and `just oracle-parity`. A gate joins that job in the step that makes it
pass and not before, because a job that fails on every pull request until the port is finished
gates nothing: `just port-status` without `--per-commit` is added by Step 23, and each comparison
recipe by its own step among Steps 24 to 27. Each of those steps names the line it adds. That job caches the dependency superbuild's install prefix,
`vendor/OrcaSlicer/deps/build/arm64/OrcaSlicer_dep`, which is the only part upstream's own
workflow keeps.

`README.md`'s layout table and getting-started section are updated here to name the new
directories, the recipes and the two jobs.

This step waits on Step 14, and picks up the later comparison recipes as they are added.

### Step 17: Bind the vendored and superbuild C and C++ libraries
- delivers: R17

Add `crates/slic3r-ffi`, whose `build.rs` compiles the vendored sources in
`vendor/OrcaSlicer/deps_src/` exactly as they are at the pin and bridges them with `cxx`:
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

This step waits on Step 11 for the crate map and on Step 7 for the Boost prefix.

### Step 18: The per-unit proof harness
- delivers: R19

Add `crates/slic3r-proof`, the harness every ported unit's test uses, and wire its three proof
paths so that a task for any file can say which one applies from its manifest row's `proof path`
column:

1. FFI to the library built from the pin, for externally linked symbols — `crates/slic3r-ffi`
   gains a link against `vendor/OrcaSlicer/build/arm64` from Step 7.
2. The committed stage dumps from Step 8, read by `crates/slic3r-proof/src/dumps.rs`, for whole
   pipeline steps.
3. A test translation unit inside the FFI crate, `cxx/include_vendored.cpp`, that `#include`s the
   vendored `.cpp` for internal-linkage code: 49 of 268 `.cpp` files declare an anonymous
   namespace and 112 declare static free functions, none with external linkage, and the pin
   forbids adding an export. Including the vendored file does not modify it, so the pin holds.
   Where even that is not possible the manifest row says so and the helper is proven through its
   enclosing file's exported surface.

`src/tiers.rs` applies the tier the unit's manifest row declares: exact equality of
integer-coordinate output for fixed-point units, the row's declared tolerance and comparison
function for float units. No test substitutes a stand-in for the code under test or for the
oracle, and `just port-check` fails a test module that mocks either. Add
`crates/slic3r-ffi/fuzz/` with the differential fuzz target shape that fixed-point units taking
polygons or points use against the C++ through FFI; C5's decision is that these targets and the
`// SAFETY:` rule are all this intent requires, and that sanitizers and Miri belong to the
following intent.

This step waits on Steps 7, 8, 11 and 17.

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

This step waits on Steps 9, 11 and 15.

### Step 20: Hand-design the foundation types

Groundwork. It delivers no requirement by itself, and Steps 21, 22 and 23 need it.

The template-heavy and header-only files are not file-units: they are type-and-trait design, and
every other unit depends on their shape, so they are designed once, by hand, before fan-out, as
`docs/WORKFLOW.md` states. The set is taken from `port/manifest.tsv` rather than listed here: the
in-scope rows whose path ends `.hpp`/`.h` and that have no same-stem `.cpp` sibling, plus the
rows the manifest marks template-heavy. At the pin the heaviest of them are `Point.hpp`,
`Config.hpp`, `AABBTreeIndirect.hpp`, `libslic3r.h` and `Model.hpp`, and header-only files are
about 4 per cent of the library's lines, so the set is small and the rest of the library is a
coherent file-per-module port.

These modules are ported, not drafted: each compiles, carries its `PORT STATUS` trailer and its
`mod` line, and has its R19 proof. The scaling entry points land here, which is why Step 15's
per-API rule is written before this step runs.

This step waits on Steps 7, 15, 17, 18 and 19.

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

Add the check that holds every later unit to it: `port/port-status --pr-body <file>`, which fails
unless the body records both review verdicts in that form and names a manifest row that exists.
It fails closed — an empty, absent or non-conforming body is a failure, since that is the one case
the check exists to catch — and it is skipped only for a pull request that adds no module, which
it decides from the diff and not from the body. It runs as its own step of the ubuntu job of
`.github/workflows/ci.yml`, which writes the body to a file with
`gh pr view "$PR" --json body --jq .body` under the workflow's `GITHUB_TOKEN`, so this step also
adds `pull-requests: read` to the workflow's `permissions`. It reads the pull request and so
cannot live in
`.asdlc/checks`, which a task also runs locally, and it needs no build.

This step waits on Steps 14, 15, 18 and 20, and **every fan-out step waits on it**.

### Step 22: Phase A — draft every in-scope unit

Groundwork. It delivers no requirement by itself: a Phase A draft is not required to compile, and
Step 23 is what turns these drafts into ported modules. Its units come from
`port/manifest.tsv`, which does not exist until Step 11 is on `main`, so this step cannot be
sized yet and is cut in two passes.

Each unit is one libslic3r `.cpp` plus its same-stem `.hpp` — one manifest row each, one unit
between them — drafted as `crates/<crate>/src/<module>.rs` at the path the manifest's target crate
and module columns name, with no `mod` declaration pointing at it, so `main` keeps building. A
draft follows `docs/PORTING.md`, carries its `PORT STATUS` trailer with
`confidence: high|medium|low`, its equality tier and the oracle that will prove it, and leaves
`// TODO(port):` rather than a guess.

**How the tasks are cut.** The first task cut from this step is a spike. It runs
`just task-cuts phase-a`, which is `port/cut-tasks`, and commits `port/taskcuts/phase-a.tsv`; the
remaining tasks of this step are cut from that file in a later pass, one task per row. The script
orders the in-scope rows by `crate-level level` ascending, then file-level include-DAG level
ascending, then path — Phase A needs no order for correctness, so this one is chosen because it
is deterministic and groups a crate's units together — and batches consecutive units **within one
target crate** until either 6 units or 2,500 source lines is reached, whichever comes first. A
unit of more than 2,500 lines is a task on its own; `PrintConfig.cpp` at 14,438 lines and
`GCode.cpp` at 11,805 are the extreme cases, and the script marks a unit over 5,000 lines as
needing its own plan when it is cut. Rows the manifest marks out of scope get no task, and the
rows Step 20 already ported are skipped. Each row of `phase-a.tsv` carries the task's name, its
target crate, its unit count, its line sum and its unit paths.

**A Phase A task's "Done when".** For every unit the row names, a file exists at the manifest's
target path with a `PORT STATUS` trailer naming that source path; no `mod` declaration reaches it;
the unit's manifest `status` is `drafted`; `just port-check` passes; and the pull request body
carries both review verdicts in the form Step 21 fixed.

This step waits on Steps 11, 15, 20 and 21.

### Step 23: Phase B — compile bottom-up, crate by crate
- delivers: R26

Make every drafted module compile, in crate-level order, with compiler errors as the work queue,
until `just port-status` passes: no in-scope manifest row has a status other than `ported`, and
the burn-down of `todo!()`, `unimplemented!()` and modules gated out of the build is zero. That
state is what R21 and R22 are not claimed without. Its units come from `port/cut.tsv` and
`port/manifest.tsv`, so like Step 22 it is cut in two passes.

Work on a module here is to make it compile against the crate map, add its `mod` line to the
crate's `lib.rs`, give it its R19 proof at the tier its manifest row declares, and move its status
to `ported`. A unit's module is not finished while a `todo!()` remains in it. Where a draft turns
out to be wrong the vendored source is re-read, not patched around: the rejection rule in
`docs/WORKFLOW.md` is what settles that.

**How the tasks are cut.** The first task cut from this step is a spike. It runs
`just task-cuts phase-b`, which is `port/cut-tasks`, and commits `port/taskcuts/phase-b.tsv`; the
remaining tasks are cut from that file in a later pass, one task per row, and each row's task
depends on the tasks of every crate below it in the cut graph. The script orders crates by
`crate-level level` ascending, and within a crate orders units by file-level include-DAG level
ascending, then path. One task per crate, except that a crate whose in-scope line sum exceeds
8,000 lines is split along that order into consecutive batches of at most 8,000 lines; the two
mutual clusters the cut cannot break, `GCode.cpp` with `GCode/*` and `Model` with `Format`, are
each one crate and are expected to be split this way. Each row carries the task's name, its crate,
its batch index, its unit count, its line sum, its unit paths and the crates it depends on.

**A Phase B task's "Done when".** `cargo build -p <crate>` and `cargo test -p <crate>` pass; every
unit the row names has its `mod` line, status `ported`, no `todo!()` or `unimplemented!()`, and a
passing R19 proof at its manifest tier; `just port-check` passes; and the pull request body
carries both review verdicts in the form Step 21 fixed.

The last task of this step adds `just port-status` — the gate, without `--per-commit` — to the
macOS job of `.github/workflows/ci.yml`, which is the moment that gate starts holding.

This step waits on Step 22, and Steps 24 to 27 wait on it.

### Step 24: Compare the benchy stage by stage
- delivers: R20

Add `just compare-stages`, which slices the benchy fixture with the port and compares, for every
stage the manifest's `pipeline stage` column marks as fully ported, the port's stage output
against that stage's oracle dump from Step 8, at the stage's declared tier, and reports the first
stage that differs. The gate applies from the first completed pipeline step and not from the first
ported unit: no stage output exists until every file that stage reaches is ported, which the
manifest's column is what decides.

For the stages with no seam — G-code emission and conflict check, which C11's decision accepts as
a gap rather than filling with a pseudo-stage dump — the command falls back to Step 5's levels on
the exported G-code, localised by the `; FEATURE:`, `; Z_HEIGHT:` and per-object label comments,
and says in its output that it is in fallback rather than reporting the last seam as the first
difference.

This step adds `just compare-stages` to the macOS job of `.github/workflows/ci.yml`.

This step waits on Steps 5, 8 and 23, and becomes meaningful as soon as the first pipeline stage's
files are all ported.

### Step 25: Slice the benchy with the port and with the oracle
- delivers: R21

Add `just compare-benchy`, which slices `fixtures/3DBenchy.stl` with the port and with the pinned
oracle from the same resolved config and the same instance transform, calls
`oracle/assert-placement` and fails when the oracle's run reports an object bounding box the
fixture record does not name, and reports the level reached. It passes when the port reaches the
level `port/reports/determinism.md` records as the base FFF configuration's G-code gate and
matches that configuration's statistics-JSON gate as well — the gate is read from that report at
run time, so if Step 6 measured it weaker than L1 this command moves with it.

C3's decision is that this end-to-end comparison is the acceptance gate rather than the first
output, and that no throwaway bridge to the vendored C++ is built to get an earlier one.

This step adds `just compare-benchy` to the macOS job of `.github/workflows/ci.yml`.

This step waits on Steps 4, 5, 6, 9, 19 and 23.

### Step 26: Every committed fixture
- delivers: R22

Add `just compare-fixtures`, which runs the port against the oracle on each fixture in the
committed set with the Bambu Lab X1 Carbon 0.4 nozzle machine, the `0.20mm Standard @BBL X1C`
process and the `Bambu PLA Basic @BBL X1C` filament, calls `oracle/assert-placement` for each, and
reports the level reached; it passes when each reaches the level the determinism report records as
that configuration's G-code gate.

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
`port/reports/determinism.md` records as that configuration's gate for it. Where a gate is `none`
the command reports the divergence against `port/reports/divergences.md` and does not fail, which
is C1's decision: the pin holds, the oracle is not patched, the port is deterministic, and each
divergence is named and checked at the gate Step 6 measured.

The command fails if a divergence it reports has no row in `port/reports/divergences.md`, so the
two cannot drift apart, and Step 13's `just audit-order` is what regenerates that file.

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
- **R2** — `just oracle-slice 20mm_cube base-fff` exits 0 and prints the path and byte size of a
  non-empty `plate_1.gcode`; the same command with a configuration row whose filament path is
  removed exits non-zero and prints the named failure, `CLI_VALIDATE_ERROR`, for exit `-51`
  arriving as 205.
- **R3** — `just oracle-determinism` regenerates `port/reports/determinism.md` and prints one line
  per configuration and artifact with its run count and gate; running it twice leaves the
  committed file unchanged apart from its timestamp; a person reads the base FFF row and finds a
  run count of at least five and a gate.
- **R4** — `cargo test -p oracle-compare` passes `tests/levels.rs`, `tests/normalisation.rs` and
  `tests/json.rs`; `just compare a.gcode b.gcode` on two real runs of Step 2 prints every level
  that passes, the strongest, and the first failing location at the next level, and on two
  unrelated files prints `none` with the first differing byte.
- **R5** — `just oracle-dump 20mm_cube base-fff` writes one dump per fired seam and prints their
  count; the check in the same recipe prints `L1` for the armed build's G-code against the
  unarmed build's and exits non-zero at any weaker level; `grep -c '^; ' dump` shows the
  provenance header, and a person reads one dump and finds integer coordinates.
- **R6** — `just oracle-parity` regenerates `port/reports/oracle-parity.md` and prints the level
  the two oracles reach per fixture; a person reads the report and finds the built oracle's CMake
  flags including `-ffp-contract=off`, and a row for every difference found.
- **R7** — `oracle/provenance --check port/goldens port/reports` exits 0 and prints the number of
  files checked; it exits non-zero on a file with a missing or stale header; a person reads one
  golden's header and finds all nine recorded items.
- **R8** — `just inventory` regenerates `port/manifest.tsv` and `port/cut.tsv` and prints the row
  count and the cut's edge count; `just inventory-check` exits 0 and prints nothing on a clean
  tree, and non-zero with a diff when either file is edited by hand; the script's own assertions
  print the presence of `LayerRegion.cpp`, `ContourZ.cpp` and `BeltPurge.cpp`.
- **R9** — `awk -F'\t' '$1 ~ /^SLA\//' port/manifest.tsv` prints every `SLA/` file with its own in
  or out decision and reason; a person checks that `SLA/IndexedMesh.cpp` and `.hpp` are in and
  that `SLA/Hollowing.cpp`, `SLA/Pad.cpp` and `SLA/ConcaveHull.cpp` are out.
- **R10** — `cargo test -p grampus-cli` passes `tests/surface.rs`, which asserts the sequence the
  surface exposes and that no arranger is reachable from it; `grep -r 'PartPlate' crates/` finds
  nothing.
- **R11** — `cargo test -p grampus-cli` passes the resolved-config round-trip in
  `tests/surface.rs`; `oracle/provenance --check port/goldens/configs` exits 0;
  `grep -c printhost_apikey port/goldens/configs/*.json` shows the key present and empty, and a
  person reads `docs/ORACLE.md` and finds the two derived G-code-only keys named.
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
- **R15** — `just port-status --per-commit` regenerates `port/reports/perf-backlog.md` from the
  `// PERF(port):` markers and exits non-zero when the committed file differs; a person reads
  `.asdlc/checks`, the `justfile` and `.github/workflows/ci.yml` and finds no check that gates on
  throughput, instruction count or allocation.
- **R16** — `just port-status` prints the counts of `todo!()`, `unimplemented!()` and modules gated
  out of the build, and exits 0 with all three at zero at the end of Step 23, which is before R21
  is claimed; `just port-status --per-commit` exits non-zero on a numeric constant whose
  `// PORT SRC:` line in the vendored file does not contain that literal.
- **R17** — `cargo test -p slic3r-ffi` passes `tests/ffi_smoke.rs`, which calls into Clipper2,
  admesh, libnest2d, qhull and Boost.Polygon's Voronoi builder through the bridge; `just
  port-check` exits non-zero when a crate manifest is given a dependency on a named substitute for
  one of them.
- **R18** — `just audit-order` regenerates `port/reports/order-containers.md`,
  `port/reports/order-comparators.md` and `port/reports/divergences.md` and prints the site count
  of each; `just audit-check` then exits 0 and prints nothing, and non-zero with a diff when one of
  the three is edited by hand; `just port-status --per-commit` exits non-zero on a site in the port
  with no `// FIDELITY(port):` marker.
- **R19** — `cargo test --workspace` passes, and each ported unit's test names its proof path; a
  person picks a manifest row, reads its `proof path` column and finds the matching test in the
  module's crate; `grep -rn 'mock\|stub' crates/*/tests` finds nothing standing in for the code
  under test or for the oracle.
- **R20** — `just compare-stages` prints one line per stage the manifest marks fully ported with
  the level reached, names the first stage that differs, and prints `fallback: exported G-code`
  for G-code emission and conflict check rather than naming `psSkirtBrim` as the difference.
- **R21** — `just compare-benchy` exits 0 and prints the level reached for the G-code and the
  statistics JSON together with the gate it read from `port/reports/determinism.md`; it exits
  non-zero when the oracle's bounding box does not match the fixture record.
- **R22** — `just compare-fixtures` exits 0 and prints one line per fixture with the level reached
  and the gate it was held to.
- **R23** — `cat .asdlc/checks` shows `just check` and `just port-check`; `just port-check` exits 0
  on a machine with no oracle binary and no libslic3r build, and at every point in the work,
  including while in-scope manifest rows are still `not-started`; the ubuntu job of
  `.github/workflows/ci.yml` runs those and the pull-request-body check, and a person reads the
  file and finds every check that needs the oracle binary, a libslic3r build or a comparison
  against either in the macOS arm64 job; a pull request that changes only a document passes with
  the document checks alone.
- **R24** — `grep -n 'arrange 0' oracle/slice` shows the flag passed on every invocation;
  `oracle/assert-placement` exits non-zero on a run whose bounding box the fixture record does not
  name, which `just compare-benchy` and `just compare-fixtures` both call; a person reads
  `vendor/README.md` and finds one resolved instance transform per fixture.
- **R25** — `ls fixtures/` prints exactly `3DBenchy.stl`, `20mm_cube.stl` and `frog_legs.stl`; a
  person reads `vendor/README.md` and finds, for each, its frozen basename, the vendored path it
  came from, its transform and its license, with the absence of an upstream license for
  `resources/handy_models/` stated; every `docs/ORACLE.md` configuration row names the fixtures it
  uses.
- **R26** — `just port-status` exits 0 and prints `0 in-scope rows remaining`; before Step 23 ends
  the same command exits non-zero and prints how many remain, which is why it is the gate the
  macOS job gains in Step 23 and not one of the checks a task passes.
- **R27** — `just compare-configurations` exits 0 and prints one line per configuration and
  artifact with the level reached and the gate it was held to, reports a `none` gate as a named
  divergence without failing, and exits non-zero when a reported divergence has no row in
  `port/reports/divergences.md`.
- **R28** — `just port-status --pr-body <file>` prints both recorded verdicts and the manifest row
  it found and exits 0; it exits non-zero on a body with one verdict, with no manifest row, and on
  an empty body, for a pull request whose diff adds a module; a person reads `docs/WORKFLOW.md` and
  finds the three-unit trial recorded in the same form, dated before the first fan-out pull
  request.
- **R29** — `just port-status --per-commit` exits non-zero and names the missing section when a
  heading is removed from `docs/PORTING.md` or `docs/WORKFLOW.md`, and exits 0 with both complete.
- **R30** — `just ownership-check` exits 0 and prints the row count; it exits non-zero when a
  raw-pointer field in an in-scope header has no row, when a row names a field that does not exist
  at the pin, and when a row other than `unknown` has no evidence; `just port-status` prints the
  count of `unknown` rows.
