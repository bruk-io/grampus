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
