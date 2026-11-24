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

**Stage two (the main project): does not configure, on either host tested — this dev machine and a
from-scratch GitHub-hosted `macos-15` runner. Both of those hosts carry Homebrew's `xz`, which is
part of the mechanism below, so a host without it is still untested. Reported, not worked around,
per C7's decision.**

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

**Root cause: the two stages disagree about where `lzma.h` is, and stage one's argument forwarding
is why.** This supersedes two earlier readings of the same error written on this branch, one of
which ("not host-contingent", "the forwarding bug is a red herring") the evidence below
contradicts. Four facts, each from a named file or log line:

1. **The exported config declares a `find_dependency` per compression backend that was enabled at
   Boost's build time.** `boost_iostreams-config.cmake:56-57` (in stage one's install prefix,
   `deps/build/arm64/OrcaSlicer_dep/usr/local/lib/cmake/boost_iostreams-1.84.0/`) carries exactly
   `find_dependency(ZLIB)` and `find_dependency(LibLZMA)`, with no `if()` guard around either, and
   **no entry at all** for BZip2 or Zstd — the two that `deps/Boost/Boost.cmake:52-53` forces off
   (`-DBOOST_IOSTREAMS_ENABLE_BZIP2:BOOL=OFF`, `...ZSTD:BOOL=OFF`). The generated file therefore
   mirrors what that build detected, which the CI log prints directly:
   `-- Boost.Iostreams: ZLIB ON, BZip2 OFF, LZMA ON (multithreaded), Zstd OFF` (line 1759 of run
   [45387169000](https://github.com/bruk-io/grampus/actions/runs/45387169000)'s `oracle-build`
   job log). ZLIB and LZMA on → two `find_dependency` calls; BZip2 and Zstd off → none.
2. **Stage one's sub-builds see only `/opt/local` of the three ignored prefixes.** Stage one's
   top-level configure (`oracle/build-libslic3r`'s `run_stage_one`) passes
   `-DCMAKE_IGNORE_PREFIX_PATH=/opt/local;/usr/local;/opt/homebrew`, and
   `vendor/OrcaSlicer/deps/CMakeLists.txt:231`/`:285` forwards it to every dependency sub-build
   unquoted (`-DCMAKE_IGNORE_PREFIX_PATH:STRING=${CMAKE_IGNORE_PREFIX_PATH}`). The value is itself
   a `;`-separated CMake list, so the unquoted substitution flattens it into three arguments before
   `ExternalProject_Add` sees one. Confirmed on both hosts: in the generated
   `deps/build/arm64/build.ninja` here, and on the runner at log line 1693, dep_Boost's configure
   command carries `-DCMAKE_IGNORE_PREFIX_PATH:STRING=/opt/local /usr/local /opt/homebrew` as three
   bare words; `deps/build/arm64/dep_Boost-prefix/src/dep_Boost-build/CMakeCache.txt:336` then
   records `CMAKE_IGNORE_PREFIX_PATH:STRING=/opt/local` alone. **The truncation fires on the CI
   runner exactly as it does here** — that is the claim an earlier version of this file got wrong.
3. **So Boost.Iostreams resolved `lzma.h` under `/opt/homebrew`, and LZMA went ON.** The cleanest
   proof is dep_Boost's own cache on this machine: `CMakeCache.txt:631` holds
   `LIBLZMA_INCLUDE_DIR:PATH=/opt/homebrew/include` while `:637` holds the *library* from the Xcode
   SDK (`.../MacOSX.sdk/usr/lib/liblzma.tbd`) — a Homebrew header paired with an SDK stub. The CI
   runner did the same: log line 1756 prints
   `-- Found LibLZMA: .../MacOSX.sdk/usr/lib/liblzma.tbd (found version "5.8.4")`, and
   `FindLibLZMA.cmake:95-102` derives that version string *only* by parsing
   `${LIBLZMA_INCLUDE_DIR}/lzma/version.h`, so a header was read there too. That runner image does
   carry Homebrew's `xz` 5.8.4 — the same log shows stage one's bundled CPython compiling
   `_lzmamodule.c` with `/usr/bin/cc -I/opt/homebrew/Cellar/xz/5.8.4/include` (line 38149).
4. **Stage two honours the full ignore list, so the same header is excluded.** Stage two's
   top-level configure receives all three prefixes as one quoted list, nothing truncates it, and
   `/opt/homebrew` is genuinely out. The Xcode SDK ships `liblzma.tbd` but no `lzma.h` and no
   `lzma/version.h` (checked under `xcrun --show-sdk-path`: neither file exists), so
   `find_library` still succeeds and `find_path` does not — which is exactly why the error names
   `LIBLZMA_INCLUDE_DIR` and not the library.

So the failure is the seam between (2) and (4): stage one built and advertised an lzma-backed
Boost.Iostreams on the strength of a header stage two is not allowed to see. The forwarding bug is
not a red herring — it is the load-bearing link.

**What is still untested.** Both hosts tried carry Homebrew `xz` 5.8.4, so the clean-host case has
not been run. The reasoning for it — unverified, stated as a prediction and not as a result — is
that with no `lzma.h` anywhere, stage one would log `LZMA OFF`, the generated config would carry
no `find_dependency(LibLZMA)` at all (the way it already carries none for BZip2 or Zstd), and
stage two would get past `CMakeLists.txt:778`. Nothing here measures that.

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
turning `SLIC3R_CAD` off — were also considered and are not what this is; none of them addresses
the stage-one/stage-two disagreement described above.

**The cheapest untried lead, named here rather than taken.** Because the mechanism is the
truncated ignore list, the first thing to try is **in this repository, not in vendored code**: make
the full three-prefix list survive the unquoted forwarding at
`vendor/OrcaSlicer/deps/CMakeLists.txt:231`/`:285` — for instance by escaping the `;` separators in
the value stage one's own invocation in `oracle/build-libslic3r` passes, so each sub-build receives
one argument instead of three. If stage one's dep_Boost then cannot see `/opt/homebrew/include`, it
should detect `LZMA OFF` and emit a config with no `find_dependency(LibLZMA)`, and stage two should
get past `CMakeLists.txt:778`. **This is untested.** It is not tried in this task for two reasons,
both from task.md: it changes stage one's flags ("Do not change stage one's flags or its build
directory"), and verifying it needs a from-scratch stage-one rebuild that this task is told not to
do ("Do not re-run stage one from an empty prefix to re-measure it"). Whoever resumes stage two
should try it before anything heavier.

Heavier options, all worse and all listed only so the order is on record: setting
`BOOST_IOSTREAMS_ENABLE_LZMA:BOOL=OFF` in vendored `deps/Boost/Boost.cmake` beside the existing
`BZIP2`/`ZSTD` lines would plausibly work too, but it edits a file under `vendor/OrcaSlicer/`,
which task.md forbids outright; vendoring a header shim, or accepting the loss of libslic3r's (if
any) lzma-backed iostream use, are further down still. Which of these is right is unverified and
deliberately left open.

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

That run then surfaced a second, genuinely new finding, in two parts, both inside *stage one* at
`dep_MPFR` (pulled in transitively by `dep_OCCT`): `vendor/OrcaSlicer/deps/MPFR/MPFR.cmake` runs
`autoreconf -f -i` as its configure step, which regenerates a Makefile carrying doc-build targets.
`macos-15`'s hosted image carries Homebrew itself but not every formula a Homebrew-equipped dev
machine (this one) already has. First run on this label: `/bin/sh: autoreconf: command not found`
(exit 127) at configure. After adding `autoconf`/`automake`/`libtool`: configure passed, and the
*build* step then failed with `missing: line 85: makeinfo: command not found` (exit 2) — so
`texinfo` was missing too. Both are a gap in the CI job's own scaffolding, not stage one's CMake
flags or anything under `vendor/OrcaSlicer/`, so `brew install autoconf automake libtool texinfo`
was added before the build step — the same category of addition as the existing
`setup-uv`/`setup-just` steps, needed for this task's one step to run at all.

With that gap closed, the job **completed stage one in full**, for real, from an empty prefix on a
from-scratch `macos-15` runner, on three separate runs: 21 dependencies, no further missing tools.
All three stage-one summaries, each printed by `oracle/build-libslic3r` from an equally cold start
(no superbuild cache exists — that is `24-port-check-ci`'s half), are listed because they span
almost a factor of two:

| Run | Stage one wall-clock | Build directory | Install prefix |
|---|---|---|---|
| [45382513074](https://github.com/bruk-io/grampus/actions/runs/45382513074) | 26m11s (1571.1s), configure 2.3s + build 1568.9s | 3.3G | 622M |
| [45387169000](https://github.com/bruk-io/grampus/actions/runs/45387169000) | 43m35s (2615.2s), configure 3.4s + build 2611.8s | 3.3G | 622M |
| [45398085129](https://github.com/bruk-io/grampus/actions/runs/45398085129) | 48m53s (2934.0s), configure 2.8s + build 2931.2s | 3.3G | 622M |

None of them is the recorded figure — PR #7's 227.3s on this dev machine is, per task.md — and the
spread is hosted-runner variance, not a difference in what was built: same 21 dependencies, same
3.3G build tree, same 622M prefix every time. All three are listed because a reader sizing a CI
timeout from one of them would be misled by any single one (the job's current 90-minute timeout is
set against the slowest, not the fastest). Note also that the install prefix measures **622M** on
every CI run, against 625M in PR #7 and 637M in this worktree: three real measurements of the same
prefix path, same kind of spread as noted above the table.

Stage two then hit the *identical* `Could NOT find LibLZMA (missing: LIBLZMA_INCLUDE_DIR)` error,
at the identical call stack, as this dev host. That second host is what the root-cause section
above is built on — but note what it does and does not show: the `macos-15` image **does** carry
Homebrew `xz` 5.8.4 (log line 38149 of run 45387169000) and the ignore-list truncation **does**
fire there (line 1693), so it reproduces the same mechanism rather than ruling it out. A host
without Homebrew `xz` has not been tried.

CI's own build-tree cache never populated across any of these runs: `actions/cache` saves in its
post step only when the job succeeded, and every run here failed, which the job's step list shows
as `Cache the main project's build tree → success` (restore) with
`Post Cache the main project's build tree → skipped` (save). It will start populating once
`just oracle-build` exits 0, same as any other. `save-always: true` would force the save, but it is
deprecated in `actions/cache` and documented as not working as intended, so it is not used here.

### Licensing

OrcaSlicer is AGPL-3.0. Code in grampus written by reading or translating this source is a
derivative work and has to be AGPL-3.0 as well.
