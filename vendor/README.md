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

**Both stages build, and both upstream test suites pass in full.** The measured figures are below.
Two things had to be fixed to get there, both of them in stage one and both about the same thing —
a dependency resolving a library from the host that stage two then cannot see. Read them before
changing either stage's flags.

#### The first thing needed: stage one has to be hermetic

Stage two used to fail at its first configure, every time, on both hosts tried:

```
CMake Error at .../FindPackageHandleStandardArgs.cmake:227 (message):
  Could NOT find LibLZMA (missing: LIBLZMA_INCLUDE_DIR)
Call Stack (most recent call first):
  ...
  deps/build/arm64/OrcaSlicer_dep/usr/local/lib/cmake/boost_iostreams-1.84.0/boost_iostreams-config.cmake:57 (find_dependency)
  deps/build/arm64/OrcaSlicer_dep/usr/local/lib/cmake/Boost-1.84.0/BoostConfig.cmake:67 (find_package)
  CMakeLists.txt:778 (find_package)
```

The two stages disagreed about where `lzma.h` is, because stage one's dependency sub-builds did not
really ignore the host prefixes stage one was told to ignore:

1. **The superbuild truncates the ignore list on the way to each dependency.**
   `vendor/OrcaSlicer/deps/CMakeLists.txt:231` (and `:285`, the cross-compile branch) forwards
   `-DCMAKE_IGNORE_PREFIX_PATH:STRING=${CMAKE_IGNORE_PREFIX_PATH}` **unquoted**, and the value is
   itself a `;`-separated CMake list, so it flattens into three argv words before
   `ExternalProject_Add` sees one. Each dependency's own cache then recorded
   `CMAKE_IGNORE_PREFIX_PATH:STRING=/opt/local` alone, and CMake discarded the other two with
   `CMake Warning: Ignoring extra path from command line: "/usr/local"`.
2. **So a dependency resolved host headers.** dep_Boost's own cache held
   `LIBLZMA_INCLUDE_DIR:PATH=/opt/homebrew/include` paired with the Xcode SDK's
   `liblzma.tbd`, and its configure logged
   `Boost.Iostreams: ZLIB ON, BZip2 OFF, LZMA ON (multithreaded), Zstd OFF`. dep_OpenVDB found the
   same Homebrew header, and dep_Boost also found Homebrew's `zstd`.
3. **The exported CMake config then demanded it back.** `boost_iostreams-config.cmake:56-57`
   declares one unguarded `find_dependency` per backend enabled at Boost's build time — with LZMA
   on, `find_dependency(LibLZMA)`, and no entry at all for the BZip2 and Zstd that
   `deps/Boost/Boost.cmake:52-53` forces off.
4. **Stage two cannot satisfy it.** Stage two is one CMake project, so its ignore list arrives
   intact and `/opt/homebrew` really is excluded; the Xcode SDK ships `liblzma.tbd` but neither
   `lzma.h` nor `lzma/version.h`, so `find_library` succeeds and `find_path` does not — which is
   why the error named `LIBLZMA_INCLUDE_DIR` and not the library.

**The fix: stage one passes the ignore list through a toolchain file, not through
`-DCMAKE_IGNORE_PREFIX_PATH`.** `oracle/build-libslic3r` generates
`oracle/.cache/build/ignore-host-prefixes.cmake` from its own `HOST_PREFIXES` list and passes it as
`-DCMAKE_TOOLCHAIN_FILE`. `deps/CMakeLists.txt:238` forwards `CMAKE_TOOLCHAIN_FILE` to each
dependency on the same lines that truncate the other value, but a file path holds no list separator
to split on, so it survives; CMake reads it before `project()` in every dependency's own configure,
and the normal variable it sets there shadows the truncated cache entry. Nothing under
`vendor/OrcaSlicer/` is modified, and stage two's flags are unchanged.

Measured, after the change: dep_Boost logs
`Boost.Iostreams: ZLIB ON, BZip2 OFF, LZMA OFF, Zstd OFF`, the installed
`boost_iostreams-config.cmake` carries **no** `find_dependency(LibLZMA)` at all (the way it already
carried none for BZip2 or Zstd), and stage two configures, builds and tests.

**Escaping the `;` separators does not work**, though it reads as the obvious fix and an earlier
version of this file recommended it as the cheapest untried lead. Tried, in a scratch superbuild
that reproduces the forwarding verbatim: passing the value with `\;` separators stores the
backslashes literally, so the superbuild's *own* value degrades to a one-element list and the
dependency still receives `/opt/local` alone. The other candidates recorded then — editing
`BOOST_IOSTREAMS_ENABLE_LZMA` in vendored `deps/Boost/Boost.cmake`, vendoring a header shim,
pointing `LIBLZMA_INCLUDE_DIR` at Homebrew by hand — are all unnecessary now, and the first and
third are the kind of workaround C7's decision declines.

#### The second thing needed: OpenSSL has to reach the prefix before Boost configures

An ignore list cannot close every route to the host, because `pkg-config` does not read one. Found
by the host-prefix scan stage one now prints with its summary (see the next section), on the second
of two cold runs of the fixed script: dep_Boost had resolved
**Homebrew's** OpenSSL —
`OPENSSL_INCLUDE_DIR:PATH=/opt/homebrew/Cellar/openssl@4/4.1.0/include` and four more entries,
including the `pkgcfg_lib__OPENSSL_*` pair that names the route — while on the first cold run it had
resolved the superbuild's own (`Found OpenSSL: .../OrcaSlicer_dep/usr/local`).

It is a race, not a host difference. dep_Boost calls `find_package(OpenSSL)`, but
`deps/Boost/Boost.cmake:61` declares only `DEP_Boost_DEPENDS ZLIB`, so nothing orders Boost after
OpenSSL and ninja is free to configure Boost first. When the prefix does not hold OpenSSL yet,
`FindOpenSSL`'s pkg-config hint finds Homebrew's, and `CMAKE_IGNORE_PREFIX_PATH` has no say in it.

`oracle/build-libslic3r` therefore builds `dep_OpenSSL` on its own before asking for the rest of
the list, which is deterministic in the right direction: `CMAKE_PREFIX_PATH` is searched ahead of a
find module's hints, so once the prefix holds OpenSSL, that is what every dependency resolves.
Nothing under `vendor/OrcaSlicer/` changes — only the order in which the script asks for targets.
It is the one part of stage one that cannot overlap with anything else, so stage one's summary
reports its wall-clock separately: **11.2s** of a 234.1s cold stage one, and nothing on a repeat
run. With it in place, dep_Boost resolved the prefix's OpenSSL on every subsequent run and the
host-resolved list stayed at the two dep_OpenCV entries below.

Worth knowing what this did and did not affect: on the run where Boost compiled against Homebrew's
OpenSSL headers, stage two still configured, built and passed both suites in full, and neither
built test executable linked anything under `/opt/homebrew` — libslic3r links none of the Boost
libraries that use OpenSSL. So the symptom was a silent host dependency in stage one's own output,
not a broken oracle.

#### What of the host the build still touches

**Full hermeticity was not reached, and this is the exact extent of what is left.** Two residuals
survive, neither of which the ignore list can reach and neither of which gets into the oracle;
`oracle/build-libslic3r` measures and prints both with stage one's summary every run, rather than
leaving them asserted here:

- **dep_OpenCV resolves Homebrew's Python framework at configure time** —
  `PYTHON3_INCLUDE_DIR` and `PYTHON3_LIBRARY` under
  `/opt/homebrew/Frameworks/Python.framework/Versions/3.15`. dep_OpenCV *is* a CMake dependency and
  it does receive the full ignore list; OpenCV's own Python detection simply does not route through
  it. **Nothing in the install prefix links `libpython`** (checked with `otool -L` over every
  `.dylib` and `.so` in it), which is what makes this harmless rather than merely untidy: it
  affects what OpenCV's configure decided about optional Python bindings, not what libslic3r links.
- **The bundled CPython's `_lzma` module links `/opt/homebrew/opt/xz/lib/liblzma.5.dylib`** — the
  one host library anything in the install prefix links, by the same `otool -L` sweep.
  `deps/python3/python3.cmake` builds CPython with `./configure`, which never reads CMake's ignore
  list, so neither the old flag nor the toolchain file reaches it. Already found and reported by
  `02-deps-superbuild` (PR #7).

So: **one host library is linked anywhere in the build, and it is not linked into the oracle.**
Two further notes on what that leaves:

- **Build tools** — `ninja`, `pkg-config`, `git`, `cmake` itself — are still Homebrew's (26 cache
  entries). Ignoring a prefix does not displace a tool, and should not: `find_program` still
  resolves them under `/opt/homebrew` while `find_path(lzma.h)` no longer resolves the header
  there. A tool's path is not linked into anything.
- **The dep_OpenCV entries are what is left of five.** Before the toolchain file, stage one also
  recorded dep_Boost's Homebrew `lzma` and `zstd` and dep_OpenVDB's Homebrew `lzma`; those three
  are gone, and the two OpenCV Python entries remain.
- **Nothing host-resolved reaches the oracle itself.** `otool -L` on both built test executables
  lists only `/usr/lib` dylibs and `/System/Library` frameworks — `libiconv`, `libc++`, `libz`,
  `libobjc`, `libSystem`, and the AppKit/Foundation/IOKit/CoreFoundation/ModelIO frameworks. No
  `/opt/homebrew`, no `/usr/local`, no `/opt/local`.

#### Measured cost

Stage one's half is transcribed from [PR #7](https://github.com/bruk-io/grampus/pull/7)
(`02-deps-superbuild`) as task.md instructs; stage two's half is measured here. Both halves were
also measured together in this worktree on the final hermetic script, from an empty
`deps/build/arm64` **and** an empty `build/arm64`, with no compiler cache, through
`just oracle-build`, which exited 0:

| | Wall-clock | Build directory | Install prefix |
|---|---|---|---|
| Stage one, recorded figure (PR #7, pre-hermetic flags) | 3m47s (227.3s), configure 0.9s + build 226.4s | 3.3G | 625M |
| Stage one, measured here (hermetic flags, warm `DL_CACHE`) | 3m54s (234.1s), configure 1.1s + build 233.0s, of which `dep_OpenSSL` alone 11.2s | 3.3G | 625M |
| Stage two, measured here (hermetic flags, empty build tree) | 1m53s (113.2s), configure 6.3s + 0.8s + build 106.0s | 1.1G | — |
| **Both stages, measured here** | **5m47s (347.3s)** | **4.4G** | 625M |

**These are figures from an unloaded machine, and they move a lot with what else is running.**
Two further cold runs of the same script measured stage one at 337.2s and 339.7s and stage two at
204.8s and 203.3s while 28 cores carried 40 artificial spin loops — 1.4x to 1.8x the table's
figures, on identical input. Read them as what this recipe costs when it has the machine, not as a
precision measurement.

The two stage-one figures agree to within 3%, so making stage one hermetic did not measurably
change its cost, its build tree or its prefix. 625M is also what PR #7 measured, which settles a
spread this file previously recorded as unexplained drift: see "Idempotence" below for where 637M
comes from. Stage two's figures exclude the two `ctest` runs, measured right after the build so the
disk figure never includes test artifacts.

**That 5m47s is a dev-machine figure (28 cores), and it is not what this recipe costs in CI.** The
same cold recipe on a GitHub-hosted `macos-15` runner measured **46m12s for stage one and 35m08s
for stage two** — about 89 minutes of runner time end to end, roughly **15x** the table above
(tabulated with the suite times under "Its own workflow" below). `24-port-check-ci`, which reads
this section to decide whether `just oracle-build` and `just oracle-parity` stay on every pull
request or move to `main` only, should size that decision on **the CI figures, not on the table
above**, and should note that a cached run skips both builds entirely. It is why the oracle build
has its own workflow with a paths filter rather than a step in `ci.yml`.

#### What the build produces, and what the test suites say

Stage two builds the three named targets and nothing else (never `all`:
`tests/slic3rutils/CMakeLists.txt:77` links `libslic3r_gui`, which does not exist with the GUI
off). In `vendor/OrcaSlicer/build/arm64`:

| Artefact | Path under `build/arm64` | Size |
|---|---|---|
| `libslic3r` | `src/libslic3r/Release/liblibslic3r.a` | 74M |
| `libslic3r_cgal` (linked into the above) | `src/libslic3r/Release/liblibslic3r_cgal.a` | 12M |
| `libslic3r_tests` | `tests/libslic3r/Release/libslic3r_tests.app/Contents/MacOS/libslic3r_tests` | 104M |
| `fff_print_tests` | `tests/fff_print/Release/fff_print_tests.app/Contents/MacOS/fff_print_tests` | 73M |

Both upstream Catch2 suites are run once by the script as evidence the library works, **not as a
gate**: a suite failure is reported and logged and does not fail `just oracle-build`, because
nobody upstream builds `fff_print_tests` with the GUI off on any platform. Both passed in full on
every run of the final script:

| Suite | `ctest` outcome | Time |
|---|---|---|
| `tests/fff_print` | 100% tests passed, 0 tests failed out of 657 | 121.5s |
| `tests/libslic3r` | 100% tests passed, 0 tests failed out of 1718 | 63.8s |

No test was skipped, excluded or deselected; both runs are
`ctest --test-dir <dir> -C Release --output-on-failure --timeout 600` over the whole suite.

**`tests/fff_print` has an intermittent deadlock, and that is why `--timeout 600` is there.** Any
test in it can hang instead of failing. Measured over **seven** full runs of this suite here, five
were clean and two were not:

| Run | Machine | Outcome |
|---|---|---|
| 5 clean runs | unloaded, and one with 40 spin loops on 28 cores | 657 of 657 passed |
| one run | unloaded | `131 - Belt brim on an object layer precedes its perimeters, once` hung at 0% CPU for 22 minutes |
| one run | 40 spin loops on 28 cores | tests `506` and `528` hung; both ended by `--timeout 600` |

**The mechanism.** `sample` on the first stuck process put every thread in
`name_tbb_thread_pool_threads_set_locale` (`src/libslic3r/Thread.cpp`), whose `parallel_for` over
`tbb::this_task_arena::max_concurrency()` single-element chunks has each chunk block on a condition
variable until *all* of them have been entered — so it hangs whenever TBB does not schedule that
many chunks concurrently. `Print::process` calls it, so every test that slices can hit it. Upstream
code; nothing to do with this configuration's flags. Load makes it likelier but is neither
necessary (the unloaded occurrence) nor sufficient (one loaded run passed in full), and a targeted
probe of test 131 alone did not reproduce it in 30 attempts (15 idle, 15 under that load).

**The timeout is measured to work**, which matters because the suites are evidence and not a gate:
a hang has to be *reported*, not left to stall the recipe or, in CI, burn the job's whole timeout.
`catch_discover_tests` (`tests/CMakeLists.txt:88`) sets no per-test `TIMEOUT` property — zero
occurrences in the generated `*_tests-*.cmake`, `*_include.cmake` and `CTestTestfile.cmake` — so
ctest's `--timeout` is what governs. On the run above it fired exactly as intended:
`***Timeout 600.03 sec`, the remaining 151 tests still ran, the script reported
`fff_print suite FAILED (exit 8)`, and `just oracle-build` still exited 0. 600s is ~16x the slowest
test measured here (37.1s, with exactly one test of 657 over 10s), so it cannot fire on a healthy
test even on a much slower runner.

#### Idempotence, and the one thing a repeat run still does

A third and any later run of `just oracle-build` is a genuine no-op in both stages — stage one
`ninja: no work to do.` in 0.7s, stage two 1.8s for its two configures — and exits 0, with both
suites passing in full each time.

The **second** run is not quite free, and this is upstream's own doing rather than this script's:
`deps/python3/python3.cmake` adds a `relink_ssl_extensions` step only when a glob of already-built
`_ssl*.so`/`_hashlib*.so` matches, which cannot be true during a cold configure and is true
afterwards. So the first repeat run spends 11.8s re-linking and re-installing the bundled CPython,
which is also where the install prefix grows from 625M to **637M**. It exits 0, stage two is still
a no-op (2.7s), and from the third run on nothing happens in either stage.

#### `OCCT_LIBS` and the second configure

Stage two is configured twice, as task.md specifies, because
`src/libslic3r/CMakeLists.txt:746` writes `OCCT_LIBS` to the cache at the end of that directory's
configure. On macOS arm64 the first configure is **not** stale, now measured against the shipped
command rather than a diagnostic probe: `OCCT_LIBS` after the first pass and after the second are
byte-identical, 26 toolkits —
`TKFillet;TKOffset;TKBool;TKDESTEP;TKDE;TKXCAF;TKXSBase;TKVCAF;TKCAF;TKLCAF;TKCDF;TKV3d;TKService;TKMesh;TKBO;TKPrim;TKHLR;TKShHealing;TKTopAlgo;TKGeomAlgo;TKBRep;TKGeomBase;TKG3d;TKG2d;TKMath;TKernel`
— with the CAD-only entries correctly prepended per `src/libslic3r/CMakeLists.txt:735`, and the
library links correctly either way. The second configure stays because it is cheap (0.8s) and
because task.md specifies it; the script's comment says exactly that and no more.

#### A stale `NLopt_DIR`

`oracle/build-libslic3r` does not implement task.md's documented
`cmake -U NLopt_DIR -U NLopt_LIBS .` clear, because it has never been needed against a fresh build
tree. The one place it would first matter is a restored CI cache of
`vendor/OrcaSlicer/build/arm64`, which carries a previous run's `CMakeCache.txt`: a future change
that moves NLopt's install location could hit the stale-cache failure the clear exists for. Left
out rather than added speculatively to a path that has never executed.

#### Its own workflow

The oracle build is `.github/workflows/oracle.yml`, not a job in `ci.yml`, and it runs on
`workflow_dispatch`, on pushes to `main`, and on pull requests — in the last two cases only when
the pull request or push touches `oracle/build-libslic3r`, that workflow, `.gitmodules` or the
`vendor/OrcaSlicer` submodule path. It does not run on every pull request: on a hosted macOS
runner this recipe costs tens of minutes to hours, and almost no change in this repository can
affect it.

The job caches both build trees, keyed on the submodule pin and a hash of
`oracle/build-libslic3r` (which is where both stages' flags live, including the generated toolchain
file's content). Restore and save are separate `actions/cache/restore` and `actions/cache/save`
steps with `if: always()` on the saves, rather than the combined `actions/cache` action, because
that action saves in a post step that runs **only on job success** — which is why the dependency
superbuild was rebuilt from scratch on every earlier, failing run of this job. (`save-always: true`
is the other way to get that behaviour and is deprecated, documented as not working as intended.)

Two things had to be added before the job could run at all, both found from its own logs and both
scaffolding rather than build flags. `runs-on: macos-14` was a retired hosted-runner image
(`actions/runner-images` has shipped no `macos-14` build since 2026-08-31 while `macos-15` and
`macos-26` keep shipping weekly); both of this branch's first two runs on that label came back
`cancelled` with zero steps executed and 0 billable minutes, confirmed through the Actions API —
not a build failure, a runner that could never be scheduled. And stage one's `dep_MPFR` (pulled in
transitively by `dep_OCCT`) runs `autoreconf -f -i` as its configure step and then needs `makeinfo`
for the Makefile that regenerates: `macos-15`'s image carries Homebrew but neither
`autoconf`/`automake`/`libtool` nor `texinfo`, which showed up as `autoreconf: command not found`
(exit 127) and then `missing: line 85: makeinfo: command not found` (exit 2). One
`brew install autoconf automake libtool texinfo` step covers both.

**What a cold run of the whole recipe costs on a hosted runner, measured.** Run
[45457280332](https://github.com/bruk-io/grampus/actions/runs/45457280332), on `macos-15`, from
nothing — no cache for either tree — with the hermetic stage one:

| | Wall-clock | Build directory | Install prefix |
|---|---|---|---|
| Stage one | 46m12s (2772.1s), configure 2.8s + build 2769.3s | 3.3G | 622M |
| Stage two | 35m08s (2108.9s), configure 7.6s + 1.0s + build 2100.3s | 1.0G | — |
| `tests/fff_print` | 277.6s | 657 of 657 passed | — |
| `tests/libslic3r` | 114.2s | 1718 of 1718 passed | — |

About 89 minutes of runner time end to end, against 5m47s for the same two stages on a 28-core dev
machine — **roughly 15x**, which is why this workflow has a paths filter instead of being a job on
every pull request, and what `24-port-check-ci` should size its decision on. Both caches saved
(1.24 GiB for the superbuild, 409 MiB for the main build tree), so a later run on the same pin and
script skips both builds.

That run also found fix 2 above on a second host, independently: its stage-one summary listed
**seven** host-resolved entries, the two dep_OpenCV Python ones and dep_Boost's five Homebrew
OpenSSL ones, because it predates the `dep_OpenSSL`-first ordering. The LibLZMA half was already
fixed there — `Boost.Iostreams: ZLIB ON, BZip2 OFF, LZMA OFF, Zstd OFF` in its log — so the
toolchain file works on a runner as it does here. Neither suite hit the deadlock on that run, and
nothing timed out.

Before stage two worked, the job ran repeatedly against the pre-hermetic script and completed
**stage one in full** on every from-scratch run that got past
the autotools gap, which is independent confirmation that `02-deps-superbuild`'s stage one is
sound on a machine that is not this one. Its wall-clock there spanned 26m11s to 48m53s:

| Run | Stage one wall-clock | Build directory | Install prefix |
|---|---|---|---|
| [45382513074](https://github.com/bruk-io/grampus/actions/runs/45382513074) | 26m11s (1571.1s), configure 2.3s + build 1568.9s | 3.3G | 622M |
| [45387169000](https://github.com/bruk-io/grampus/actions/runs/45387169000) | 43m35s (2615.2s), configure 3.4s + build 2611.8s | 3.3G | 622M |
| [45398085129](https://github.com/bruk-io/grampus/actions/runs/45398085129) | 48m53s (2934.0s), configure 2.8s + build 2931.2s | 3.3G | 622M |

Those runs measured the **pre-hermetic** script, and every one of them then failed stage two on the
LibLZMA error above — the same error, the same call stack, on a clean runner image that also carries
Homebrew's `xz` 5.8.4 (visible in run 45387169000's log at line 38149, where stage one's bundled
CPython compiles `_lzmamodule.c` with `-I/opt/homebrew/Cellar/xz/5.8.4/include`) and where the
ignore-list truncation fires exactly as it does here (line 1693). Further runs on this branch landed
inside that span rather than widening it (45408522763 at 40m57s, 45416651382 at 42m03s,
45427388980 at 37m59s), all with the same 3.3G tree and 622M prefix; that list is the runs measured
when this was written, not a count of every run the branch accumulated. The span, not any single
figure, is what to read off this table: it is hosted-runner variance over an identical build, and it
is what the workflow's timeout is sized against.

### Licensing

OrcaSlicer is AGPL-3.0. Code in grampus written by reading or translating this source is a
derivative work and has to be AGPL-3.0 as well.
