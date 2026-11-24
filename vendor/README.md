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
here, not re-measured: this task does not rebuild stage one from an empty prefix on purpose.
It was, however, necessarily built once for real in this task's own worktree (that worktree had
no prefix yet), measured at 226.9s; PR #7's 227.3s is kept as the recorded figure per task.md's
"transcribed, not re-measured" instruction, with this run noted here so the two records do not
read as contradicting each other.

The install prefix's on-disk size drifts with the exact set of dependency versions resolved at
build time: PR #7 measured 625M; a `du -sh` of the same prefix path in this task's own build
measures 637M. Both are real measurements of the same prefix at different times, not a
transcription error; 625M stays the recorded figure (PR #7's, per the same instruction), with
637M noted here so a reader sizing a CI cache from this table sees the spread.

**Stage two (the main project): does not configure against this host's stage-one prefix
(see below for why that is not the same as "this platform"). Reported, not worked around, per
C7's decision.**

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

**Root cause, confirmed by inspection — and corrected from an earlier reading of the same
evidence.** `liblzma` is not detected via `pkg-config`: stage one's Boost is built through its
own CMake build (not `b2`), and the `FindLibLZMA.cmake` module its dependency discovery calls
does a plain `find_path(LIBLZMA_INCLUDE_DIR lzma.h)` — no `pkg-config` hint anywhere in that
module. What actually lets `/opt/homebrew/include` through is a bug in the *forwarding* of
`CMAKE_IGNORE_PREFIX_PATH`, not its absence. Stage one's top-level configure
(`oracle/build-libslic3r`'s own `run_stage_one`) correctly passes
`-DCMAKE_IGNORE_PREFIX_PATH=/opt/local;/usr/local;/opt/homebrew` to `vendor/OrcaSlicer/deps`'s
superbuild. That superbuild then forwards the value to every dependency sub-build, unquoted, at
`deps/CMakeLists.txt:231` and `:285`: `-DCMAKE_IGNORE_PREFIX_PATH:STRING=${CMAKE_IGNORE_PREFIX_PATH}`.
Because `CMAKE_IGNORE_PREFIX_PATH` is itself a `;`-separated CMake list and this substitution is
unquoted, CMake flattens it into three separate list elements before `ExternalProject_Add` ever
sees one argument — confirmed directly in the generated
`vendor/OrcaSlicer/deps/build/arm64/build.ninja`, where dep_Boost's own configure command carries
`-DCMAKE_IGNORE_PREFIX_PATH:STRING=/opt/local /usr/local /opt/homebrew` as three bare words, not
one quoted value. The result lands in
`dep_Boost-prefix/src/dep_Boost-build/CMakeCache.txt:336`: `CMAKE_IGNORE_PREFIX_PATH:STRING=/opt/local`
— only the first element survived. `/usr/local` and `/opt/homebrew` are silently *not* ignored by
that sub-build, so `FindLibLZMA` finds Homebrew's header
(`CMakeCache.txt:631`, `LIBLZMA_INCLUDE_DIR:PATH=/opt/homebrew/include`) and library
(`:637`, the Xcode SDK's headerless `.tbd` stub), and `BOOST_IOSTREAMS_ENABLE_LZMA:BOOL=ON`
(`:92`) gets compiled into `libboost_iostreams.a`. Its exported package config,
`boost_iostreams-config.cmake:57`, then calls `find_dependency(LibLZMA)` unconditionally on every
consumer configure, which is what stage two's configure trips over. This bug lives entirely in
`vendor/OrcaSlicer/deps/CMakeLists.txt`, which this task (and `02-deps-superbuild` before it) is
forbidden from changing.

Upstream's own `deps/Boost/Boost.cmake` turns bzip2 and zstd off explicitly for Boost.Iostreams
(`-DBOOST_IOSTREAMS_ENABLE_BZIP2:BOOL=OFF`, `...ZSTD:BOOL=OFF`) but leaves lzma to plain feature
detection, with no corresponding off-switch. So whether this particular sub-build ends up linking
lzma is **host-contingent**, not a fixed property of this platform: it depends on whether
`/usr/local` or `/opt/homebrew` (the two prefixes the forwarding bug fails to exclude) happen to
hold an `lzma.h` on the machine running stage one. On this machine they do, via Homebrew's `xz`.
A clean host without it would not trip this — stage one's Boost would then export no
`find_dependency(LibLZMA)` at all, and stage two's configure would not hit this error here. So
**"does not configure on this platform" overstates what was shown**: the accurate statement is
that stage two does not configure against *the stage-one prefix as built on this host*, because
that prefix's Boost embeds this host's Homebrew lzma — a contamination bug in how
`CMAKE_IGNORE_PREFIX_PATH` is forwarded, not a platform limitation. The one environment that would
settle whether a clean host avoids it is a hosted CI runner with no Homebrew `xz`; see "Its own CI
line" below for whether that run has actually exercised this.

I confirmed the diagnosis is otherwise complete — nothing else past LibLZMA is broken by this
configuration — by configuring once, off the record, with
`-DLIBLZMA_INCLUDE_DIR=/opt/homebrew/opt/xz/include -DLIBLZMA_LIBRARY=/opt/homebrew/opt/xz/lib/liblzma.dylib`
added to the command: configure then completes in full (`-- Generating done`, exit 0) with no
other error, both on the first pass and on a second, identical pass. I did not keep that flag,
and `oracle/build-libslic3r` does not carry it: pointing explicitly at Homebrew to satisfy a
dependency `CMAKE_IGNORE_PREFIX_PATH` is specifically there to keep out is the same class of
workaround C7's decision and `02-deps-superbuild`'s own PR (which found and reported, rather than
suppressed, a Homebrew link in the bundled CPython's `_lzma` module) both declined. The three
things task.md explicitly forbids — patching vendored source, hand-editing `CMakeCache.txt`, and
turning `SLIC3R_CAD` off — were also considered and are not what this is; none would reach a real
fix either, since the forwarding bug is upstream of all three. The real next step, left to
whoever picks this back up (not attempted here, since it touches stage one and task.md forbids
changing stage one's flags in this task): get the full ignore list to survive the forwarding at
`deps/CMakeLists.txt:231`/`:285` — unverified what that takes, since the bug is in vendored source
this task cannot edit to test a fix against.

**What this means for the OCCT_LIBS double-configure question.** Against the shipped command, this
is unanswered: configure fails inside the top-level `CMakeLists.txt`'s dependency discovery,
before `add_subdirectory(src)` ever reaches `src/libslic3r/CMakeLists.txt:746`, so neither
configure pass gets far enough to write or read `OCCT_LIBS`, and
`vendor/OrcaSlicer/build/arm64/CMakeCache.txt` after the failed first pass has no `OCCT_LIBS`
entry at all. But task.md asks for this to be answered "where the first macOS configure turns out
to link correctly", and the off-the-record diagnostic above does reach that point, so the answer
is in hand from that probe, not from the shipped command: in a scratch build directory, with the
identical stage-two flags plus the two `LIBLZMA_*` paths,
`OCCT_LIBS` after the first configure pass and after the second are byte-identical —
`TKFillet;TKOffset;TKBool;TKDESTEP;TKDE;TKXCAF;TKXSBase;TKVCAF;TKCAF;TKLCAF;TKCDF;TKV3d;TKService;TKMesh;TKBO;TKPrim;TKHLR;TKShHealing;TKTopAlgo;TKGeomAlgo;TKBRep;TKGeomBase;TKG3d;TKG2d;TKMath;TKernel`
— 26 toolkits, with the three CAD-only entries (`TKDESTEP`, `TKDE`, `TKXCAF`'s group) correctly
prepended per `src/libslic3r/CMakeLists.txt:735`. `ninja -t targets` in that scratch tree also
shows the three named phony targets (`libslic3r`, `libslic3r_tests`, `fff_print_tests`) exist, as
Done-when line 2 expects of the names, though nothing was built there. Measured via that
diagnostic configure with LibLZMA supplied, not via the shipped command, and not re-run against
the real `vendor/OrcaSlicer/build/arm64` tree — the second configure stays in
`oracle/build-libslic3r` regardless, exactly as task.md specifies, since on the real tree removing
it would not get past the LibLZMA failure either way.

**Stage two's measured cost:** none to report. It fails at configure, consistently, in a few
seconds, before anything is compiled; there is no build wall-clock or disk-size figure for a
build that never starts. No libslic3r, libslic3r_tests or fff_print_tests target was built, and
neither upstream test suite ran.

**Status:** `just oracle-build` exits non-zero. Stage one still resolves correctly to no work on
a repeat run (confirmed: 0.6s, configure 0.5s + build 0.1s, on a checkout whose prefix was already
installed). Stage two is blocked. Per C7's decision, this is reported rather than worked around;
`25-stage-dumper`, `28-oracle-parity`, `31-proof-harness` and every port unit depend on this
working and are blocked by this finding.

**A stale `NLopt_DIR`.** `oracle/build-libslic3r` does not implement task.md's documented
`cmake -U NLopt_DIR -U NLopt_LIBS .` clear, because it was never needed against a fresh build
tree. The one place it would first matter is the CI job's cached `vendor/OrcaSlicer/build/arm64`
(see "Its own CI line" below): a restored tree carries a previous run's `CMakeCache.txt`, so a
future change that moves `NLopt`'s install location could hit the stale-cache failure the clear
exists for. Left out for now and named here rather than added speculatively to a path that has
never executed.

**Its own CI line.** The macOS arm64 job added to `.github/workflows/ci.yml` initially requested
`runs-on: macos-14`, which turned out to be a retired hosted-runner image (GitHub's
`actions/runner-images` releases show no `macos-14`/`macos-14-arm64` build since 2026-08-31,
while `macos-15` and `macos-26` keep shipping). Both of this PR's first two CI runs on that label
came back `cancelled` with zero steps executed, confirmed via the Actions API
(`"status":"cancelled"`, 0 billable minutes, no steps) — not a build failure, a runner that could
never be scheduled. Changed to `runs-on: macos-15` (still arm64 by default) so the job actually
executes.

That run then surfaced a second, genuinely new finding: it failed inside *stage one*, at
`dep_MPFR` (pulled in transitively by `dep_OCCT`), with `/bin/sh: autoreconf: command not found`
(exit 127) — `vendor/OrcaSlicer/deps/MPFR/MPFR.cmake` runs `autoreconf -f -i` as its configure
step, and `macos-15`'s hosted image carries Homebrew itself but not the `autoconf`/`automake`/
`libtool` formulae, which a Homebrew-equipped dev machine (this one) already has. This is a gap
in the CI job's own scaffolding, not stage one's CMake flags or anything under
`vendor/OrcaSlicer/`, so a `brew install autoconf automake libtool` step was added before the
build step — the same category of addition as the existing `setup-uv`/`setup-just` steps, needed
for this task's one step to run at all. Whether the job then reaches the same LibLZMA failure as
this host once that gap is closed (confirming the "host-contingent" framing above) or configures
cleanly (proving it) is for the PR's next CI run to show.

### Licensing

OrcaSlicer is AGPL-3.0. Code in grampus written by reading or translating this source is a
derivative work and has to be AGPL-3.0 as well.
