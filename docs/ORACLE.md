# The oracle

The test oracle is the real OrcaSlicer binary, run headlessly, that every port unit is proven
against. This document is added to by each oracle step; `oracle/fetch-release` (step 1) writes
only the two sections below.

## The asset and its hash

`just oracle-fetch` fetches and verifies this release:

| | |
|---|---|
| Release tag | `v2.5.2` |
| Release asset | `OrcaSlicer_Mac_universal_V2.5.2.dmg` |
| Recorded sha256 | `6ea1afcdffb7e9ab011e2d6f6db665cb18463d1c4774591cd5db7ee0225479d9` (`vendor/README.md` is the source of truth; this table quotes it) |
| Bundle written to | `oracle/.cache/app/OrcaSlicer.app` |
| Binary used | `oracle/.cache/app/OrcaSlicer.app/Contents/MacOS/OrcaSlicer` |

The binary is the GUI bundle's executable. It is also the CLI and runs headless with no
`DISPLAY`; there is no separate console executable in the bundle. `oracle/fetch-release` never
writes the recorded sha256 — it only reads it and fails when a download does not match it.

## What is never read

`/Applications/OrcaSlicer.app` (2.4.2) and `/Applications/BambuStudio.app` are never read by any
command in this repository. The oracle is always the pinned binary `just oracle-fetch` stages
under `oracle/.cache/`, never an app already installed on the machine running the command.

## Configurations

`oracle/slice` (step 2) resolves `just oracle-slice <fixture> <configuration>`'s `<configuration>`
argument to a row below by the `Configuration` column, exactly. Every preset path is relative to
`vendor/OrcaSlicer/resources/profiles/BBL/`. `Filament(s)` holds one or more `;`-separated preset
paths, loaded in order via `--load-filaments`; `Overrides` holds zero or more `;`-separated
`key=value` pairs, passed to the binary as `--<key with _ replaced by ->=<value>` — the CLI's
equals form, which every option type (including a `coBool` one) accepts as a value without the
space-separated form's risk of being parsed as a stray positional file argument instead. A row
naming no filament is rejected by `oracle/slice` itself, before the binary runs.

Until `11-fixture-set` lands, `Fixtures` names a path under `vendor/OrcaSlicer/`, not a frozen
`fixtures/` basename.

| Configuration | Machine | Process | Filament(s) | Overrides | Fixtures |
|---|---|---|---|---|---|
| `base-fff` | `machine/Bambu Lab X1 Carbon 0.4 nozzle.json` | `process/0.20mm Standard @BBL X1C.json` | `filament/Bambu PLA Basic @BBL X1C.json` | (none) | `vendor/OrcaSlicer/tests/data/frog_legs.obj` |
| `tree-support` | `machine/Bambu Lab X1 Carbon 0.4 nozzle.json` | `process/0.20mm Standard @BBL X1C.json` | `filament/Bambu PLA Basic @BBL X1C.json` | `enable_support=1`; `support_type=tree(auto)` | `vendor/OrcaSlicer/tests/data/frog_legs.obj` |
| `wipe-tower-multi-filament` | `machine/Bambu Lab X1 Carbon 0.4 nozzle.json` | `process/0.20mm Standard @BBL X1C.json` | `filament/Bambu PLA Basic @BBL X1C.json`; `filament/Bambu PLA Basic @BBL X1C.json` | `enable_prime_tower=1`; `outer_wall_filament_id=2` | `vendor/OrcaSlicer/tests/data/frog_legs.obj` |
| `fuzzy-skin` | `machine/Bambu Lab X1 Carbon 0.4 nozzle.json` | `process/0.20mm Standard @BBL X1C.json` | `filament/Bambu PLA Basic @BBL X1C.json` | `fuzzy_skin=external` | `vendor/OrcaSlicer/tests/data/frog_legs.obj` |

All four rows were attempted directly on the pinned 2.5.2 binary (C12) and all four slice through
`just oracle-slice` with exit 0 and a non-empty `plate_1.gcode`; none is marked "not constructible
through the pinned CLI". This differs from the only attempt on record (a 2.4.2 build, modified
process JSON, exit 239/251/254): the mechanism here is the CLI's own `--<key>=<value>` override
flags (every key named above is CLI-exposed — not `nocli` — in
`vendor/OrcaSlicer/src/libslic3r/PrintConfig.cpp`), not a doctored preset file, and `settings.json`
exported from each run confirms the override landed in the resolved config.

**`wipe-tower-multi-filament`'s third element, measured.** task.md names three things for this row:
`enable_prime_tower=1`, two `--load-filaments` entries so the print has two filaments, and "the
single object's filament assignment switching between them". `enable_prime_tower=1` alone does not
reach the third element — the row's own `settings.json` shows the key is already `1` in the
inherited process preset, so the override is a no-op by itself, and without a per-region or
per-height assignment the single mesh never draws from the second filament (confirmed: with only
`enable_prime_tower=1`, `; filament used [mm]` reads `589.35, 0.00` and no `; FEATURE: Prime tower`
line appears). `outer_wall_filament_id` (`coInt`, CLI-exposed, not `nocli`,
`vendor/OrcaSlicer/src/libslic3r/PrintConfig.cpp:5927`) assigns the single object's outer walls to
the second filament by region, which is what the row's override cell now adds. Measured on the
pinned 2.5.2 binary through `just oracle-slice`: exit 0, `plate_1.gcode` 632859 bytes,
`; filament used [mm] = 786.64, 555.17`, 20 `; FEATURE: Prime tower` markers, 14 toolchanges — the
single object's filament assignment does switch between the two loaded filaments. `--load-custom-gcodes`'s
JSON `ToolChange` items are a second, separately-measured mechanism for a per-height switch on this
same mesh (print_z below `frog_legs.obj`'s 2 mm height), but this task builds neither
`--load-custom-gcodes` nor `--load-assemble-list` support into `oracle/slice` — the row's override
cell is enough on its own, so no second mechanism is added here.

**`fuzzy-skin`'s G-code is not byte-reproducible run to run.** Three consecutive
`just oracle-slice vendor/OrcaSlicer/tests/data/frog_legs.obj fuzzy-skin` runs measured
`plate_1.gcode` at 535737, 534845 and 532275 bytes; `base-fff` and `tree-support` reproduce
byte-exactly across repeated runs, so the variance is specific to this row, not the harness. The
cause is in the pin: `vendor/OrcaSlicer/src/libslic3r/Feature/FuzzySkin/FuzzySkin.cpp:38-42`
seeds each run's `random_value()` from `std::random_device`/a thread id, with no fixed seed, and
that value is consumed by the perimeter generator at `FuzzySkin.cpp:320,337,375,393`. Every run
still exits 0 with a non-empty `plate_1.gcode`, which is all this row's Done-when line requires;
`27-oracle-determinism` is where a normalisation or tolerance for this row's byte comparison has
to be decided.

## The locally built oracle

`just oracle-build` (`oracle/build-libslic3r`) builds `src/libslic3r` itself from the vendored
submodule, as a second, from-source oracle alongside the release-binary one above. Stage two,
the main project, is configured with generator `-G "Ninja Multi-Config"` (twice — see
`vendor/README.md`), built with `--config Release`, and tested with
`ctest -C Release --output-on-failure --timeout 600` (the per-test bound exists because one
upstream test can deadlock rather than fail; `vendor/README.md` has the measurement), using:

```
-DCMAKE_PREFIX_PATH="<stage one's install prefix>"
-DCMAKE_BUILD_TYPE=Release
-DSLIC3R_GUI=OFF
-DSLIC3R_CAD=ON
-DBUILD_TESTS=ON
-DSLIC3R_STATIC=1
-DCMAKE_CXX_FLAGS="-ffp-contract=off"
-DCMAKE_C_FLAGS="-ffp-contract=off"
-DCMAKE_OSX_ARCHITECTURES=arm64
-DCMAKE_OSX_DEPLOYMENT_TARGET=12.0
-DCMAKE_IGNORE_PREFIX_PATH="/opt/local;/usr/local;/opt/homebrew"
-DCMAKE_POLICY_VERSION_MINIMUM=3.10
```

against the prefix stage one (the dependency superbuild) installs, building the named targets
`libslic3r`, `libslic3r_tests` and `fff_print_tests` (never the aggregate `all` target, which
pulls in a GUI library that does not exist with `SLIC3R_GUI=OFF`). `-ffp-contract=off` is
normative, not incidental: without it the compiler fuses multiply-add expressions Rust never
fuses, and `29-ffi-bridge-core` checks that the `CMAKE_CXX_FLAGS` spelling and
`crates/slic3r-ffi/build.rs` carry the same flag. `CMAKE_C_FLAGS` carries the identical flag too —
task.md names only the C++ spelling, but the orientation/incircle predicates it exists to protect
(Shewchuk's routines in `deps_src/mcut/source/shewchuk.c`, linked into both `libslic3r` and
`libslic3r_cgal`) are C, not C++, so the faithful reading passes it to both languages.

Stage one (the dependency superbuild) is configured with generator `-G Ninja` and:

```
-DCMAKE_BUILD_TYPE=Release
-DCMAKE_OSX_ARCHITECTURES=arm64
-DCMAKE_OSX_DEPLOYMENT_TARGET=12.0
-DCMAKE_TOOLCHAIN_FILE=<generated: sets CMAKE_IGNORE_PREFIX_PATH to the same three prefixes>
-DCMAKE_POLICY_VERSION_MINIMUM=3.10
```

Stage one reaches the same three ignored prefixes through a toolchain file rather than
`-DCMAKE_IGNORE_PREFIX_PATH` because the superbuild forwards that cache entry to each dependency's
own CMake invocation unquoted, which truncates the list to its first element; a toolchain file has
no list separator to split on, so it survives. `oracle/build-libslic3r` generates the file and
carries the measurements; `vendor/README.md`'s "Building libslic3r locally" section has the full
account. Without it, Boost.Iostreams resolved `lzma.h` under `/opt/homebrew`, exported
`find_dependency(LibLZMA)` in its CMake config, and stage two — which does honour the whole ignore
list — could not satisfy it (`Could NOT find LibLZMA (missing: LIBLZMA_INCLUDE_DIR)`).

**No upstream workflow builds this configuration on macOS.** `SLIC3R_GUI=OFF` with
`SLIC3R_CAD=ON` is exercised by exactly one upstream script
(`scripts/CAD/run-kernel-tests.sh:145-158`), on Linux only, for one target. The flags above are
measured: both stages complete, the three named targets build, and both upstream Catch2 suites
pass in full. `vendor/README.md` records the wall-clock, the disk cost and the suite outcomes.
