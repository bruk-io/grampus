---
kind: task
step: 7
delivers: []
size: 5
---
# Task: Build the vendored dependency superbuild on macOS arm64

## What to build

Part 1 of the plan's step 7, "Build the vendored libslic3r on this platform": **stage one, the
dependency superbuild**. Step 7 delivers no requirement by itself, which is why this file's
`delivers` is empty; it is groundwork that steps 8, 10, 18, 20, 21, 22, 23 and 24 all sit behind.
Part 2 of the step is task `13-build-libslic3r`, stage two, which builds the main project against
the prefix this task produces. This task waits on nothing and starts immediately, alongside step 1
and step 11.

**Add `oracle/build-libslic3r`** and the `just oracle-build` recipe. Step 7 names one script and
one recipe, so this task creates both and part 2 adds stage two to the same script rather than a
second one. The script takes no sub-command and no stage option: step 7 authorises neither, and an
interface invented here would be a contract part 2 never agreed to. At the end of this task
`oracle/build-libslic3r` runs stage one and stops, and `just oracle-build` runs it; part 2 extends
the same script to run stage one and then stage two in order, under the same recipe. Nothing waits
on `just oracle-build` meaning both stages before part 2 lands — step 16 creates the macOS job
without that line and part 2 adds it. Like every script outside the Rust workspace it is a
standalone executable with PEP 723 inline metadata and no file extension, run through `uv`.

Stage one is this, verbatim:

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

**Each of these is load-bearing, and dropping one breaks the build or the oracle silently.**

- **The build directory is `vendor/OrcaSlicer/deps/build/arm64` and is not configurable.** That is
  upstream's default location, and it matters because of OCCT's patch step. `OCCT.cmake:41` runs
  `git apply ${OCCT_DIRECTORY_FLAG} ...`, and `OCCT_DIRECTORY_FLAG` is set only when `IN_GIT_REPO`
  is true (`OCCT.cmake:28-30`), which comes from `git rev-parse --is-inside-work-tree` run by
  `deps/CMakeLists.txt:451-463` in the configure process's own working directory. `BINARY_DIR_REL`
  in that flag is computed relative to `${CMAKE_SOURCE_DIR}/..`, which is `vendor/OrcaSlicer`. A
  build directory outside that work tree leaves the flag unset or pointing at the wrong root, and
  OCCT then builds **unpatched** — which configures and compiles fine and fails much later, in
  meshing, with no mention of a patch.
- **The 21 per-dep targets are named rather than the `deps` target.** `deps/CMakeLists.txt` has no
  `SLIC3R_GUI` option at all, and its `deps` target (`:506`) always depends on `dep_wxWidgets`,
  `dep_wxInspector`, `dep_FFMPEG` and the libdatachannel package, none of which a GUI-off build
  needs and all of which cost build time.
- **`CMAKE_POLICY_VERSION_MINIMUM=3.10` is mandatory with CMake 4.x**, and is passed both in the
  environment and as `-D`. The deps project force-sets it into its own cache
  (`deps/CMakeLists.txt:1-3`), but each dependency is a separate CMake run driven through
  `ExternalProject`, so the environment variable is what reaches those.
- **`CMAKE_IGNORE_PREFIX_PATH="/opt/local;/usr/local;/opt/homebrew"` keeps Homebrew and MacPorts out
  of the build.** Do not drop or narrow it to make a dependency configure.
- **`dep_python3` builds a bundled CPython 3.12.13** (`deps/python3/python3.cmake:5`) and is
  required even with the GUI off. It is the longest single item in the stage.
- **`dep_SLVS` exists only because the deps project's `SLIC3R_CAD` option defaults ON**
  (`deps/CMakeLists.txt:66`, the gate at `:392`), which is the value stage two needs
  (`-DSLIC3R_CAD=ON`). So stage one is configured without overriding `SLIC3R_CAD` and the target is
  valid; passing `-DSLIC3R_CAD=OFF` here would make `dep_SLVS` an unknown target and would
  mismatch stage two.
- **The prefix the stage produces is
  `vendor/OrcaSlicer/deps/build/arm64/OrcaSlicer_dep/usr/local`.** `DESTDIR` is assigned at
  `deps/CMakeLists.txt:86` (`set(DESTDIR "${CMAKE_BINARY_DIR}/OrcaSlicer_dep" CACHE PATH ... FORCE)`)
  and gains its `usr/local/` suffix at `:95-96`, conditionally on `FLATPAK` being false, which is
  its default (`:65`). Do not read `:68-73` for this: that block is the `USE_OLD_DESTDIR`
  backward-compatibility branch and only tests whether the old `destdir` name exists. The prefix is
  what part 2's `CMAKE_PREFIX_PATH` reads. This task produces it and does not consume it.

**Tee the stage's output to a log, because the build is the only record of what it did.** The deps
superbuild passes no `ExternalProject` `LOG_*` option anywhere — `deps/CMakeLists.txt:507-508`
records why ("I'm not using any of the `LOG_xxx` options in `ExternalProject_Add()` commands because
they seem to generate bogus build files") — so no `dep_*-stamp/*-patch-*.log` is ever written and a
step's command text exists only on the build's stdout. So the script tees the stage-one
`cmake --build` output to `oracle/.cache/build/deps-arm64.log`, which is gitignored with the rest of
`oracle/.cache/` (task `01-oracle-fetch` adds that line). That log is the evidence for the OCCT
patch check below.

**Measure the first build.** It starts from zero downloads with no compiler cache, and two later
decisions read the measurement: part 2 writes it into `vendor/README.md`'s outcome record, and step
16 chooses between a per-pull-request macOS job and a `main`-only one from it. So the script prints,
at the end of a stage-one run, the wall-clock it took and the on-disk size of both
`vendor/OrcaSlicer/deps/build/arm64` and the install prefix, and this task's pull request body
carries those numbers.

**What this task does not do.** All of it is part 2 of step 7, task `13-build-libslic3r`, so this
list is the boundary rather than a gap: the stage-two configure of `vendor/OrcaSlicer` itself;
configuring stage two **twice** for the `OCCT_LIBS` cache trap at
`vendor/OrcaSlicer/src/libslic3r/CMakeLists.txt:746`; `-DSLIC3R_GUI=OFF`, `-DSLIC3R_CAD=ON`,
`-DBUILD_TESTS=ON`, `-DSLIC3R_STATIC=1`; `-DCMAKE_CXX_FLAGS="-ffp-contract=off"`; building the
`libslic3r`, `libslic3r_tests` and `fff_print_tests` targets; the `ctest` run; clearing a stale
`NLopt_DIR` with `cmake -U NLopt_DIR -U NLopt_LIBS .`; the `vendor/README.md` outcome record;
`docs/ORACLE.md`'s built-oracle flags; and the `just oracle-build` line in the macOS job of
`.github/workflows/ci.yml`. Write none of them here.

**Why this is a size 5, and why it is not split further.** Unknowns dominate: nobody upstream builds
this configuration on macOS — `SLIC3R_GUI=OFF` is exercised by one upstream script only inside a
Linux docker image, no upstream workflow sets it, and the caveat is recorded in this repository's
`vendor/README.md` and not upstream — and the stage still has to produce 21 dependency targets
including a bundled CPython, under a CMake policy minimum that only matters on CMake 4.x, with
Homebrew and MacPorts excluded, from a build directory whose location OCCT's patch step depends on.
That is spec concern C7. It is not split further because a superbuild is one unit of failure: the 21
targets share one configure, one toolchain and one install prefix, a single dependency that will not
build fails the stage, and the policy minimum, the prefix exclusion and the work-tree constraint are
properties of that one configure rather than of any one dependency. Splitting it would give two
tasks the same configure to write.

**C7's decision governs this task: a failure is reported, not worked around.** If a dependency
cannot be made to build, the task reports which target failed, with its error and what was tried,
and stops. It does not fall back to a Homebrew, MacPorts or system copy of that library, does not
substitute a different version, does not drop a target from the list, and does not edit anything
under `vendor/OrcaSlicer/`. The work that depends on this step stops with it.

**What this task must not touch.** Nothing under `vendor/OrcaSlicer/` changes and the submodule
pointer stays at `f92a7c065c5718b8ef1baac1cd704df2911238f4`. Both build directories are gitignored
upstream — `/build*/` and `deps/build*` in `vendor/OrcaSlicer/.gitignore` — so building in place
does not dirty the submodule, and this task checks that rather than assuming it. No `CMakeCache.txt`
is edited by hand. Nothing is added to `.asdlc/checks` or `.github/workflows/ci.yml`.

## Done when

- `oracle/build-libslic3r` exists with no file extension, the `uv` shebang and a PEP 723 inline
  metadata block, and `just oracle-build` runs it.
- `oracle/build-libslic3r` runs the stage-one configure and build verbatim as listed above,
  including `-DCMAKE_IGNORE_PREFIX_PATH="/opt/local;/usr/local;/opt/homebrew"`,
  `-DCMAKE_POLICY_VERSION_MINIMUM=3.10` and `CMAKE_POLICY_VERSION_MINIMUM=3.10` exported in the
  environment.
- `just oracle-build` exits 0 on a machine with no prebuilt dependencies and no compiler cache.
- Where the line above is answered no, this task's pull request body names the failing `dep_*`
  target, quotes its error and lists what was tried, and the branch contains no edit under
  `vendor/OrcaSlicer/`, no Homebrew, MacPorts or system substitution, and no target removed from the
  21 — which is the spec's C7 decision that a failure is reported rather than worked around.
- `vendor/OrcaSlicer/deps/build/arm64/OrcaSlicer_dep/usr/local` exists and holds an installed
  library or include tree for each of the 21 named targets.
- The script names those 21 targets exactly as listed, and
  `grep -n 'dep_wxWidgets\|dep_wxInspector\|dep_FFMPEG\|datachannel\|--target deps'
  oracle/build-libslic3r` finds nothing.
- OCCT was patched, not just built:
  `grep -n 'git apply --directory deps/build/arm64/dep_OCCT-prefix/src/dep_OCCT' oracle/.cache/build/deps-arm64.log`
  finds the patch invocation and no bare `git apply --verbose` line precedes it, and the change
  `deps/OCCT/0001-BRepMesh-seam-pcurve-selection.patch` makes is present in
  `vendor/OrcaSlicer/deps/build/arm64/dep_OCCT-prefix/src/dep_OCCT`.
- The bundled CPython was built: the install prefix holds a `python3.12` binary or
  `libpython3.12` library.
- `grep -n '^CMAKE_IGNORE_PREFIX_PATH' vendor/OrcaSlicer/deps/build/arm64/CMakeCache.txt` prints
  exactly one entry whose value is `/opt/local;/usr/local;/opt/homebrew`, and
  `grep -nE '^[A-Za-z0-9_]+(_INCLUDE_DIR|_LIBRARY|_DIR|_ROOT):' vendor/OrcaSlicer/deps/build/arm64/CMakeCache.txt | grep '/opt/homebrew'`
  finds nothing. The second half is what catches a dependency resolved out of Homebrew;
  `CMAKE_IGNORE_PREFIX_PATH` suppresses `find_package` prefixes and does not keep a Homebrew *tool*
  path such as `CMAKE_MAKE_PROGRAM=/opt/homebrew/bin/ninja` out of the cache, so an unscoped
  `grep '/opt/homebrew'` would fail a correctly configured build.
- The script prints the stage's wall-clock and the on-disk size of
  `vendor/OrcaSlicer/deps/build/arm64` and of the install prefix, and both numbers are in the pull
  request body, for part 2's `vendor/README.md` record and step 16's fallback decision.
- `git -C vendor/OrcaSlicer status --porcelain` is empty after the build, and
  `git -C vendor/OrcaSlicer rev-parse HEAD` prints `f92a7c065c5718b8ef1baac1cd704df2911238f4`.
- The branch carries none of part 2's work:
  `grep -n 'ffp-contract\|SLIC3R_GUI\|SLIC3R_STATIC\|BUILD_TESTS\|ctest\|NLopt_DIR\|libslic3r_tests'
  oracle/build-libslic3r` finds nothing, and the branch changes neither `vendor/README.md` nor
  `docs/ORACLE.md` nor `.github/workflows/ci.yml`.
