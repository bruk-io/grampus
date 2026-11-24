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
`vendor/README.md`), built with `--config Release`, and tested with `ctest -C Release`, using:

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

**This configuration does not currently build against this host's stage-one prefix.** Stage two's
first configure fails — see `vendor/README.md`'s "Building libslic3r locally" section for the
exact error, the confirmed root cause (stage one's own `CMAKE_IGNORE_PREFIX_PATH` is truncated by
an argument-forwarding bug in vendored `deps/CMakeLists.txt`, so Homebrew's `xz` leaks into stage
one's Boost build on any host that has it), and what was tried. This is host-contingent, not a
fixed property of macOS arm64: a host with no Homebrew `xz` would not trip it. No upstream
workflow runs `SLIC3R_GUI=OFF`/`SLIC3R_CAD=ON` on macOS at all; the one upstream script that runs
it at all (`scripts/CAD/run-kernel-tests.sh:145-158`) does so only on Linux. The flags above are
what this oracle is specified to use once this is resolved, not evidence that it currently
produces a working binary.
