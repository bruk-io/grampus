---
kind: task
step: 7
delivers: []
depends: [02-deps-superbuild]
size: 5
---
# Task: Build the vendored libslic3r with the GUI off and -ffp-contract=off

## What to build

This is **part 2 of step 7** of `intent/libslic3r-rust-port/plan.md` ("Build the vendored
libslic3r on this platform"): stage two, the main project, the records it writes, and its one CI
line. Part 1, `02-deps-superbuild`, is stage one, the dependency superbuild. Step 7 delivers no
requirement by itself; it is the groundwork the locally built oracle rests on, and C7's recorded
decision in `intent/libslic3r-rust-port/spec.md` is that it is attempted first, as its own piece
of work, and that a failure is **reported rather than worked around**.

**Stage two, added to `oracle/build-libslic3r`.** That script and the `just oracle-build` recipe
are created by `02-deps-superbuild`; this task extends them with the main project's configure,
build and test. The script takes no stage option — `02-deps-superbuild` authorises none — so after
this task `oracle/build-libslic3r` runs stage one and then stage two in order and `just oracle-build`
means both. **Stage one is therefore made idempotent here**: where the install prefix below already
holds stage one's output, the script re-runs stage one's configure and build, which Ninja resolves
to no work, and the run proceeds to stage two. That is what makes the first "Done when" below
answerable on a checkout whose prefix is already installed, and what keeps the macOS job from
rebuilding 21 dependencies on every run. Do not change stage one's flags or its build directory: it is configured at
`vendor/OrcaSlicer/deps` into `vendor/OrcaSlicer/deps/build/arm64`, which is the upstream default
location, and OCCT's patch step only applies when `git rev-parse --is-inside-work-tree` succeeds
in that directory. Stage two is configured against the prefix stage one installs,
`vendor/OrcaSlicer/deps/build/arm64/OrcaSlicer_dep/usr/local`, and is **configured twice**:

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

Each part of that is load-bearing, and the script carries a comment saying why:

- **Configured twice**, as a cheap precaution against the late `OCCT_LIBS` cache write at
  `vendor/OrcaSlicer/src/libslic3r/CMakeLists.txt:746`
  (`set(OCCT_LIBS "${OCCT_LIBS}" CACHE INTERNAL ...)`), which happens at the end of that
  directory's configure. Write the comment in the script that way and no stronger: the only known
  consumer of the cached value outside that file is `orcaslicer_copy_dlls`
  (top-level `CMakeLists.txt:1077`, reading `OCCT_LIBS` at `:1109` and `:1117`), whose call sites
  are inside `if (WIN32)` blocks (`src/CMakeLists.txt:257`, `tests/CMakeLists.txt:42`), and
  libslic3r's own link at `src/libslic3r/CMakeLists.txt:776` uses the directory-scope value set at
  `:698` and `:735`. So on macOS arm64 the first configure is not demonstrably stale, and the second
  configure is kept because it is cheap and because what the first configure linked is otherwise
  unrecorded. Where the first macOS configure turns out to link correctly, record that in
  `vendor/README.md` rather than carrying an unverified reason in the script.
- **Named targets, never `all`.** `vendor/OrcaSlicer/tests/slic3rutils/CMakeLists.txt:77` links
  `libslic3r_gui`, which does not exist with the GUI off, so `all` cannot be built.
- **`-C Release`** is not optional on `ctest` with a multi-config generator.
- **`-ffp-contract=off`** is the flag C2's decision makes normative: without it the compiler
  fuses multiply-add expressions that Rust never fuses, including the two-dimensional cross
  product that decides orientation branches. `29-ffi-bridge-core` later adds a check that this
  spelling and `crates/slic3r-ffi/build.rs` carry the same flag, so keep it in
  `CMAKE_CXX_FLAGS` where that check can find it.
- **`SLIC3R_GUI=OFF` with `SLIC3R_CAD=ON`** is the one such combination upstream exercises
  (`vendor/OrcaSlicer/scripts/CAD/run-kernel-tests.sh:145-158`), and only for `libslic3r_tests`,
  inside a Linux docker image. `SLIC3R_CAD=OFF` would flip a second untried axis, so CAD stays
  ON even if OFF would configure sooner.
- **`CMAKE_POLICY_VERSION_MINIMUM=3.10`** is mandatory with CMake 4.x, and
  `CMAKE_IGNORE_PREFIX_PATH` keeps Homebrew out of the build.
- A stale `NLopt_DIR` in the cache is cleared with `cmake -U NLopt_DIR -U NLopt_LIBS .`, never by
  editing `CMakeCache.txt`.

**Both upstream suites are built, and both are run**, once here as evidence the library works and
not as a gate: nobody upstream builds `fff_print_tests` with the GUI off, on any platform. So the
script runs two `ctest` invocations, the `tests/fff_print` one above and

```
ctest --test-dir vendor/OrcaSlicer/build/arm64/tests/libslic3r -C Release --output-on-failure
```

so that `libslic3r_tests`, which the named-targets line builds, is exercised and not merely linked.
Both lines are part of `oracle/build-libslic3r`, so they run with the recipe; "not a gate" means no
separate check or CI step is added for the suites, and a suite failure is written into
`vendor/README.md` and reported, not patched around. Record what both say either way.

**What must not be touched.** Nothing under `vendor/OrcaSlicer/` is modified and the submodule
pointer stays at `f92a7c065c5718b8ef1baac1cd704df2911238f4`. Both build directories are
gitignored upstream (`/build*/`, `deps/build*`), so building in place does not dirty the
submodule. Do not patch a vendored source, do not hand-edit `CMakeCache.txt`, and do not turn
`SLIC3R_CAD` off to make the configure succeed. If the build cannot be made to work, write what
was tried into `vendor/README.md` and stop: the tasks behind this one (`25-stage-dumper`,
`28-oracle-parity`, `31-proof-harness` and every port unit) stop with it, which is C7's decision.

**The records.** `vendor/README.md` gains the outcome: that nobody upstream builds this
configuration on macOS, what was needed to make it work, and the measured **wall-clock and disk
cost of the first cold build, recorded as two halves**, because `24-port-check-ci` reads the figure
out of `vendor/README.md` to decide whether `just oracle-build` and `just oracle-parity` move to
`main` only, and nothing else in the repository carries it:

- **Stage one's half is transcribed, not re-measured.** `02-deps-superbuild` prints stage one's
  wall-clock and the on-disk size of `vendor/OrcaSlicer/deps/build/arm64` and of the install prefix,
  and carries both in its own pull request body, which is not in the repository. This task copies
  those two figures into `vendor/README.md` and says they came from that pull request, naming it.
  Do not re-run stage one from an empty prefix to re-measure it.
- **Stage two's half is measured here**, from an empty `vendor/OrcaSlicer/build/arm64` with no
  compiler cache: its wall-clock and the on-disk size of that build tree. A warm rebuild's time is
  not that number.

The record states the two halves separately and their sum, so a reader of `24-port-check-ci` knows
which figure is measured where. `docs/ORACLE.md` gains the built oracle's
exact flags, including `-ffp-contract=off`; that file is created by `01-oracle-fetch`, so append a
section rather than rewriting it.

**Its own CI line, which does not wait for `24-port-check-ci`.** This task adds a step running
`just oracle-build` to the macOS arm64 job of `.github/workflows/ci.yml`, together with a cache of
`vendor/OrcaSlicer/build/arm64` keyed on the submodule pin and the CMake flags above. Step 16 of
the plan creates that job without this recipe, the same way it leaves every gate to the step that
makes it pass, so whichever of the two lands second adds its half and neither is written in terms
of the other's line. If the job is not in the file yet, create it holding **only** this step and
this cache; the path filter, `just oracle-fetch`, `cargo test --workspace` and the
superbuild-prefix cache `vendor/OrcaSlicer/deps/build/arm64/OrcaSlicer_dep` are
`24-port-check-ci`'s half and are not written here. **The split of the two caches is exactly this,
and `24-port-check-ci` says the same**: the main build tree's cache is this task's, because it is
this task's step that fills it and only this task knows the flags its key names; the superbuild
prefix's cache is that task's. Neither task writes the other's cache block.

**What makes this a 5.** The configuration is one no CI anywhere runs: GUI off with CAD on is
exercised by a single upstream script, on Linux only, for a single target, so the first
configure's failures are unknown in advance. The project has to be configured twice because
`OCCT_LIBS` reaches the cache late; the targets have to be named individually because
`slic3rutils` links a GUI library that does not exist with the GUI off; and the first build's
measured cost is an input another task's decision depends on. It is not split further because
stage two is one configure-build-test sequence whose failures are only visible by running it, and
the records and the CI line are a few lines each on top of that run.

## Done when

- `just oracle-build` exits 0 on a checkout where stage one has already installed its prefix, runs
  stage one and then stage two in that order with stage one resolving to no work, and its stage two
  configures `vendor/OrcaSlicer/build/arm64` twice with the flags above before building.
- `vendor/OrcaSlicer/build/arm64` holds built `libslic3r`, `libslic3r_tests` and `fff_print_tests`
  targets, and no invocation in the script builds `all`.
- `oracle/build-libslic3r` runs both
  `ctest --test-dir vendor/OrcaSlicer/build/arm64/tests/fff_print -C Release --output-on-failure`
  and `ctest --test-dir vendor/OrcaSlicer/build/arm64/tests/libslic3r -C Release --output-on-failure`,
  and both runs' outcomes are written into `vendor/README.md` as evidence; no separate check or CI
  step gates on the suites.
- `git -C vendor/OrcaSlicer status --porcelain` prints nothing after a full `just oracle-build`,
  and `git diff --submodule` shows the pointer still at
  `f92a7c065c5718b8ef1baac1cd704df2911238f4`.
- `grep -n 'ffp-contract' oracle/build-libslic3r` shows `-ffp-contract=off` inside
  `CMAKE_CXX_FLAGS` for stage two.
- `grep -n 'SLIC3R_CAD' oracle/build-libslic3r` shows `ON`, and no `SLIC3R_CAD=OFF` anywhere in
  the script.
- `vendor/README.md` records four figures and their source: stage one's wall-clock and disk,
  transcribed from `02-deps-superbuild`'s pull request and attributed to it, and stage two's
  wall-clock and disk, measured here from an empty `vendor/OrcaSlicer/build/arm64` with no compiler
  cache; it states that no upstream workflow builds this configuration on macOS, and lists what was
  needed to make it work.
- `docs/ORACLE.md` lists the built oracle's exact CMake flags, including `-ffp-contract=off`.
- `.github/workflows/ci.yml`'s macOS arm64 job runs `just oracle-build` and caches
  `vendor/OrcaSlicer/build/arm64` with a key naming the submodule pin and those CMake flags, and
  this task's diff to that file adds nothing else — in particular no cache of
  `vendor/OrcaSlicer/deps/build/arm64/OrcaSlicer_dep`, which is `24-port-check-ci`'s.
- No file under `vendor/OrcaSlicer/` is changed, no `CMakeCache.txt` is edited by hand, and the
  script clears a stale `NLopt_DIR` with `cmake -U NLopt_DIR -U NLopt_LIBS .` if it clears one at
  all.
