# Vendored sources

## OrcaSlicer

`vendor/OrcaSlicer` is a git submodule of the upstream OrcaSlicer repository. It is the
reference implementation grampus ports from, and the source the test oracle is built from.
Nothing in it is modified.

| | |
|---|---|
| Upstream | https://github.com/OrcaSlicer/OrcaSlicer (formerly `SoftFever/OrcaSlicer`) |
| Pinned tag | `v2.5.2` (released 2026-11-21) |
| Pinned commit | `f92a7c065c5718b8ef1baac1cd704df2911238f4` |
| License | AGPL-3.0 (`vendor/OrcaSlicer/LICENSE.txt`) |

### What "libslic3r version" means here

libslic3r is not a separately released library. It lives in-tree at
`vendor/OrcaSlicer/src/libslic3r` and has no version of its own, so the version grampus
targets is **the OrcaSlicer commit above**. Two version strings exist in
`vendor/OrcaSlicer/version.inc`:

- `SoftFever_VERSION "2.5.2"`: the OrcaSlicer release.
- `SLIC3R_VERSION "02.08.01.55"`: inherited from the Bambu Studio lineage. It is written into
  project files but does not identify a libslic3r release.

Any golden file or oracle output must record the pinned commit, not either string.

### Why the whole repository, not just `src/libslic3r`

libslic3r cannot be built on its own. Its CMake target needs the bundled libraries in
`deps_src/`, the dependency superbuild in `deps/`, and the top-level `CMakeLists.txt` and
`cmake/` modules. The upstream test suites and test models are also outside `src/libslic3r`.

### Getting it

The submodule is marked shallow, so a fresh clone only fetches the pinned commit:

```sh
git clone --recurse-submodules --shallow-submodules <grampus-url>
# or, in an existing clone:
git submodule update --init --depth 1
```

The checkout is about 570 MB (130 MB of git data, the rest working tree).

### Layout

Paths are relative to `vendor/OrcaSlicer`.

| Path | Contents |
|---|---|
| `src/libslic3r/` | The slicing engine. The port target. |
| `src/slic3r/` | wxWidgets GUI. Out of scope. |
| `src/libvgcode/` | G-code viewer library used by the GUI. Out of scope. |
| `src/OrcaSlicer.cpp` | Application and CLI entry point. |
| `deps_src/` | Small libraries built in-tree: `clipper2`, `admesh`, `libnest2d`, `libigl`, `mcut`, `qhull`, `miniz`, `nlohmann`, and others. |
| `deps/` | Superbuild recipes for large external libraries: Boost, TBB, CGAL, OpenVDB, OCCT, OpenCV, Eigen, and others. |
| `tests/libslic3r/`, `tests/fff_print/` | Upstream Catch2 suites that drive libslic3r directly. The template for the oracle. |
| `tests/data/` | Small test models. |
| `resources/profiles/` | Printer, filament and process profiles (JSON). |
| `version.inc` | Version strings. |

### libslic3r at this pin

- **Size:** 605 C/C++ source files, about 340,000 lines.
- **Subsystems** (subdirectories): `Algorithm`, `Arachne`, `CAD`, `CSGMesh`, `Execution`,
  `Feature`, `Fill`, `Format`, `GCode`, `Geometry`, `Optimize`, `SLA`, `Support`,
  `TextureBake`, `TextureToColor`.
- **Coordinates:** `coord_t` is `int64_t` (`libslic3r.h`). The scaling factor is a runtime
  global, `SCALING_FACTOR`: `1e-6` mm per unit normally, `1e-5` for large printers.
- **Polygon clipping:** Clipper2 (`deps_src/clipper2`), wrapped by `ClipperUtils.hpp`.
- **Build targets:** `libslic3r` and `libslic3r_cgal`, both static libraries, defined in
  `src/libslic3r/CMakeLists.txt`.
- **Linked libraries:** Eigen, admesh, libigl, libnest2d, miniz, OpenCV, assimp, Boost, cereal,
  Clipper2, Draco, glu-libtess, JPEG, PNG, CGAL, mcut, libnoise, qhull, qoi, semver, TBB, zlib,
  OpenSSL, OCCT, EXPAT, OpenVDB, SLVS.

### Pipeline steps

From `src/libslic3r/Print.hpp`. These are the stage boundaries the oracle dumps at.

Per object (`PrintObjectStep`), in order: `posSlice`, `posPerimeters`,
`posEstimateCurledExtrusions`, `posPrepareInfill`, `posInfill`, `posIroning`, `posContouring`,
`posSupportMaterial`, `posSimplifyPath`, `posSimplifySupportPath`,
`posDetectOverhangsForLift`, `posSimplifyWall`, `posSimplifyInfill`.

Per print (`PrintStep`): `psWipeTower` (also tool ordering), `psSkirtBrim`, `psGCodeExport`,
`psConflictCheck`.

### Relevant upstream build options

From the top-level `CMakeLists.txt`:

| Option | Default | Meaning |
|---|---|---|
| `SLIC3R_GUI` | on | Build the GUI. Whether libslic3r builds cleanly with this off is untested here. |
| `SLIC3R_CAD` | on | Parametric CAD tab; pulls in more of OCCT. |
| `BUILD_TESTS` | off | Build the Catch2 suites under `tests/`. |
| `ORCA_TOOLS` | off | Build upstream developer tools. |

`build_release_macos.sh` is the upstream macOS build script.

### Changing the pin

Moving the pin changes what "correct" means for every oracle comparison, so treat it as a
deliberate migration:

1. `git -C vendor/OrcaSlicer fetch --depth 1 origin tag <new-tag>`
2. `git -C vendor/OrcaSlicer checkout <new-tag>`
3. Update the tag, date and commit in the table above, and re-check the "libslic3r at this
   pin" and "Pipeline steps" sections against the new tree.
4. Regenerate all oracle outputs and review the differences.
5. Commit the submodule pointer and this file together.

### Oracle release asset

`oracle/fetch-release` (R1) does not build OrcaSlicer from this submodule. It downloads the
matching v2.5.2 release binary and verifies it against the value recorded here before using it
as the test oracle, so the check is not circular:

| | |
|---|---|
| Release tag | `v2.5.2` |
| Release asset | `OrcaSlicer_Mac_universal_V2.5.2.dmg` |
| Recorded sha256 | `6ea1afcdffb7e9ab011e2d6f6db665cb18463d1c4774591cd5db7ee0225479d9` |

That sha256 was measured on a download of that asset during this intent's research. It is the
one value `oracle/fetch-release` reads; the script never rewrites it, and a mismatch fails the
command instead. See `docs/ORACLE.md` for how the fetched binary is used.

### Building libslic3r locally (`just oracle-build`)

`oracle/build-libslic3r` drives the vendored tree's own two CMake projects: stage one, the
dependency superbuild at `deps/`, then stage two, the main project itself with the GUI off
(`SLIC3R_GUI=OFF`, `SLIC3R_CAD=ON`, `-ffp-contract=off`). Nothing under `vendor/OrcaSlicer/` is
written to by either stage; both build directories (`deps/build/arm64` and `build/arm64`) are
gitignored upstream.

**No upstream workflow builds this configuration on macOS.** `SLIC3R_GUI=OFF` with
`SLIC3R_CAD=ON` is exercised by exactly one upstream script
(`scripts/CAD/run-kernel-tests.sh:145-158`), inside a Linux docker image, for one target. This is
the first time it has been attempted on macOS arm64, as `intent/libslic3r-rust-port/spec.md`'s
C7 anticipated.

**Stage one (the dependency superbuild): measured, working.**

| | Wall-clock | Build directory | Install prefix |
|---|---|---|---|
| Stage one | 3 minutes 47 seconds (227.3s), configure 0.9s + build 226.4s | 3.3G | 625M |

Measured in [PR #7](https://github.com/bruk-io/grampus/pull/7) (`02-deps-superbuild`), from an
empty `deps/build/arm64` with `deps/DL_CACHE` also empty, with no compiler cache. Transcribed
here, not re-measured: this task does not rebuild stage one from an empty prefix.

**Stage two (the main project): does not configure on this platform. Reported, not worked
around, per C7's decision.**

The exact command from `intent/libslic3r-rust-port/tasks/13-build-libslic3r/task.md`, run from
a clean `vendor/OrcaSlicer/build/arm64`, fails at its first configure:

```
CMake Error at /opt/homebrew/share/cmake/Modules/FindPackageHandleStandardArgs.cmake:227 (message):
  Could NOT find LibLZMA (missing: LIBLZMA_INCLUDE_DIR)
Call Stack (most recent call first):
  /opt/homebrew/share/cmake/Modules/FindPackageHandleStandardArgs.cmake:591 (_FPHSA_FAILURE_MESSAGE)
  /opt/homebrew/share/cmake/Modules/FindLibLZMA.cmake:129 (find_package_handle_standard_args)
  /opt/homebrew/share/cmake/Modules/CMakeFindDependencyMacro.cmake:78 (find_package)
  deps/build/arm64/OrcaSlicer_dep/usr/local/lib/cmake/boost_iostreams-1.84.0/boost_iostreams-config.cmake:57 (find_dependency)
  deps/build/arm64/OrcaSlicer_dep/usr/local/lib/cmake/Boost-1.84.0/BoostConfig.cmake:67 (find_package)
  deps/build/arm64/OrcaSlicer_dep/usr/local/lib/cmake/Boost-1.84.0/BoostConfig.cmake:128 (boostcfg_find_component)
  CMakeLists.txt:778 (find_package)
```

**Root cause, confirmed by inspection, not guessed.** Stage one's own superbuild detected
Homebrew's `xz` (`liblzma`) at *its own* build time (via `pkg-config`, unconstrained by the flag
below) and compiled lzma support into `libboost_iostreams.a`. Its exported CMake package config,
`boost_iostreams-config.cmake`, therefore calls `find_dependency(LibLZMA)` unconditionally on
every consumer configure — not something stage two's command line can skip or make optional,
because it reflects how the already-built, frozen stage-one artifact was compiled. `lzma.h`
exists on this machine only under `/opt/homebrew/include` (`xz` 5.8.4) and under a `miniconda3`
prefix; it is not in stage one's own install prefix, not in `vendor/OrcaSlicer/deps_src/`, and
not in the Xcode SDK (the SDK has only `liblzma.tbd`, a linkable stub with no header — which is
why CMake finds the *library* via the SDK but still reports the *header* missing). Stage two's
command line mandates `-DCMAKE_IGNORE_PREFIX_PATH="/opt/local;/usr/local;/opt/homebrew"`, so the
one place the header exists is the one place this build is required not to look.

I confirmed this diagnosis by configuring once, off the record, with
`-DLIBLZMA_INCLUDE_DIR=/opt/homebrew/opt/xz/include -DLIBLZMA_LIBRARY=/opt/homebrew/opt/xz/lib/liblzma.dylib`
added to the command: configure then completes in full (`-- Generating done`, exit 0) with no
other error. That confirms LibLZMA is the only blocker reached so far, and that nothing else past
it is broken by this configuration — but I did not keep that flag, and `oracle/build-libslic3r`
does not carry it. Pointing explicitly at Homebrew to satisfy a dependency `CMAKE_IGNORE_PREFIX_PATH`
is specifically there to keep out is the same class of workaround C7's decision and
`02-deps-superbuild`'s own PR (which found and reported, rather than suppressed, a Homebrew link
in the bundled CPython's `_lzma` module) both declined. The three things task.md explicitly
forbids — patching vendored source, hand-editing `CMakeCache.txt`, and turning `SLIC3R_CAD` off —
were also considered and are not what this is; none would reach a real fix either, since the
missing header is upstream of all three.

**What this means for the OCCT_LIBS double-configure question.** Unanswered: configure fails
inside the top-level `CMakeLists.txt`'s dependency discovery, before `add_subdirectory(src)` ever
reaches `src/libslic3r/CMakeLists.txt:746`, so neither configure pass gets far enough to write or
read `OCCT_LIBS`. `vendor/OrcaSlicer/build/arm64/CMakeCache.txt` after the failed first pass has
no `OCCT_LIBS` entry at all. The second configure is kept in `oracle/build-libslic3r` regardless,
exactly as task.md specifies, since removing it would not get past this failure either.

**Stage two's measured cost:** none to report. It fails at configure, consistently, in a few
seconds, before anything is compiled; there is no build wall-clock or disk-size figure for a
build that never starts. No libslic3r, libslic3r_tests or fff_print_tests target was built, and
neither upstream test suite ran.

**Status:** `just oracle-build` exits non-zero. Stage one still resolves correctly to no work on
a repeat run (confirmed: 0.5s, configure 0.4s + build 0.1s, on a checkout whose prefix was already
installed). Stage two is blocked. Per C7's decision, this is reported rather than worked around;
`25-stage-dumper`, `28-oracle-parity`, `31-proof-harness` and every port unit depend on this
working and are blocked by this finding.

### Licensing

OrcaSlicer is AGPL-3.0. Code in grampus written by reading or translating this source is a
derivative work and has to be AGPL-3.0 as well.
