---
kind: task
step: 18
delivers: [R19]
depends: [08-comparator-workspace, 13-build-libslic3r, 14-manifest-columns, 24-port-check-ci, 29-ffi-bridge-core]
size: 5
---
# Task: Build the per-unit proof harness and its tiers

## What to build

This is **part 1 of plan step 18, "The per-unit proof harness"**, which delivers R19; `delivers`
names the whole step because the step is split. Part 1 is the harness crate, the tier dispatch,
proof paths 1 and 3, and the mock check. Part 2 is task `32-proof-dumps-fuzz`: proof path 2
(`crates/slic3r-proof/src/dumps.rs`) and the `cargo-fuzz` crate. Read plan step 18 and spec R19
before starting.

Add `crates/slic3r-proof`, the harness every ported unit's test uses, and wire two of the step's
three proof paths so that a task for any file can say which one applies from its manifest row's
`proof path` column.

**The crate.** `crates/slic3r-proof/Cargo.toml`, `crates/slic3r-proof/src/lib.rs` and
`crates/slic3r-proof/src/tiers.rs`. It depends on `crates/slic3r-ffi` (task
`29-ffi-bridge-core`). The root `Cargo.toml` is **not** edited: it is `members = ["crates/*"]`
(plan step 5), so the directory appearing is enough. `Cargo.lock` may change — plan step 5 names
step 18 among the steps that may change it.

**`src/tiers.rs` — the tier dispatch from the manifest row.** This is the one API every ported
unit's test calls, so its shape is fixed here rather than left to the builder. The entry point is

```rust
pub fn compare_unit<T>(source_path: &str, oracle: Oracle, actual: T, expected: T)
    -> Result<(), TierError>
```

with `Oracle` an enum of `Release` and `Built`. Three things follow from that signature and each is
the answer to a question the builder would otherwise have to guess:

- **The unit identifies itself by passing its vendored source path.** `tiers.rs` does not parse the
  caller module's `PORT STATUS` trailer at run time; the caller passes the same path its trailer
  names, and `tiers.rs` looks the row up by it.
- **The manifest is located from the workspace root**, taken from an environment variable whose
  default is `CARGO_MANIFEST_DIR`'s workspace root plus `port/manifest.tsv`; where the file is
  absent it fails with a named error rather than comparing at a default. No path to it is hardcoded.
- **The oracle is passed in by the caller**, because no manifest column holds it: the four columns
  `tiers.rs` reads are `equality tier`, `tolerance`, `comparison function` and `proof path`, and the
  oracle lives in the module's own `PORT STATUS` trailer, which this harness does not read.

It reads the unit's row in `port/manifest.tsv` by that source path, and applies the tier the row
declares:

- an `exact` row: exact equality of integer-coordinate output, with no tolerance path reachable;
- a float row: the row's declared `tolerance` and `comparison function`, and nothing else.

It refuses, with a named error rather than a comparison, a row whose `tolerance` cell carries no
reason text — the same rule plan step 5 gives the comparator — and a row whose `proof path` cell
is empty. Both a passing and a failing comparison print the unit's manifest row path, its tier,
its proof path and the oracle, so R19's "each ported unit's test names its proof path" is
readable from the test output. The columns it reads (`equality tier`, `tolerance`,
`comparison function`, `proof path`) are the carried-forward judgement columns task
`14-manifest-columns` defines; the harness reads them and never writes a manifest cell.

**Proof path 1 — FFI to the library built from the pin.** `crates/slic3r-ffi/build.rs`, which
task `29-ffi-bridge-core` creates, gains a link against the `libslic3r` that
`just oracle-build` (task `13-build-libslic3r`) builds at `vendor/OrcaSlicer/build/arm64`. That
is the library built with `-ffp-contract=off` and therefore the oracle C2's decision makes
normative. `src/libslic3r/CMakeLists.txt:612` is `add_library(libslic3r STATIC ...)` and `:633`
adds a second archive, `libslic3r_cgal`, so with the `Ninja Multi-Config` generator plan step 7
uses the archives sit under `vendor/OrcaSlicer/build/arm64/src/libslic3r/Release/`. `build.rs`
discovers them under the build tree rather than hardcoding a path that exists on one machine.

**The rest of the link line is not the superbuild prefix, and assuming it is leaves undefined
symbols** — which is this task's main uncertainty, so it is written out. The dependencies
`target_link_libraries(libslic3r ...)` names (`src/libslic3r/CMakeLists.txt:764-794`, plus the
`APPLE` block at `:796-799` and the `SLIC3R_CAD` block at `:806-807`) come from three different
places:

- **The superbuild prefix** `vendor/OrcaSlicer/deps/build/arm64/OrcaSlicer_dep/usr/local/lib`:
  Eigen, OCCT, Boost, Cereal, Draco, JPEG, PNG, TBB, ZLIB, OpenSSL, OpenCV, Assimp, and Qhull where
  the determination in task `30-ffi-bridge-libraries` says the build linked that one.
- **The main project's own build tree** `vendor/OrcaSlicer/build/arm64`: about half of them are
  built from `vendor/OrcaSlicer/deps_src/` by the main project rather than by the superbuild
  (`deps_src/CMakeLists.txt:17-33`, with `add_library(... STATIC ...)` at
  `deps_src/admesh/CMakeLists.txt:4`, `deps_src/libnest2d/CMakeLists.txt:24`,
  `deps_src/qhull/CMakeLists.txt:135` and `deps_src/semver/CMakeLists.txt:5`), so admesh,
  libnest2d, qhull, Clipper2, glu-libtess, mcut, qoi, semver, miniz, libigl and `libslic3r_cgal`
  sit under the build tree and are **not** in the prefix.
- **macOS frameworks and the CAD solver**: `Foundation` and `ModelIO` from the `APPLE` block, which
  link as `-framework` and not as prefix libraries, and `SLVS::slvs` from the `SLIC3R_CAD` block,
  since the oracle build sets `SLIC3R_CAD=ON`.

So `build.rs` **derives the link line from the CMake build tree rather than enumerating names**: it
reads the generated link command of the `libslic3r_tests` or `fff_print_tests` target out of
`vendor/OrcaSlicer/build/arm64` — Ninja's `build.ninja`, or that target's `link.txt` — and emits the
`cargo:rustc-link-search`, `cargo:rustc-link-lib` and `cargo:rustc-link-arg` lines from it, plus the
C++ standard library. That is the one approach that cannot be wrong about which of the three places
a given archive is in, and it stays correct when the pin's own dependency list changes. Clipper2
appears twice — this crate already compiles it from `deps_src/clipper2` in task
`29-ffi-bridge-core`, and the build tree also provides it as an archive — so the derivation drops
the build tree's `Clipper2` archive, keeps the crate's own compilation, and says so in a comment.

When that build is absent, the build script fails with a message naming
`just oracle-build` rather than producing a crate that links nothing.

**Proof path 3 — a test translation unit that includes a vendored `.cpp`.** Add
`crates/slic3r-ffi/cxx/include_vendored.cpp`, compiled by the same `build.rs` and therefore with
`-ffp-contract=off`; a proof against a translation unit compiled without that flag would be a
proof against the wrong reference. It exists because the pin forbids adding an export and 49 of
268 `.cpp` files declare an anonymous namespace while 112 declare static free functions, none
with external linkage. `#include`ing the vendored file does not modify it, so the pin holds, and
nothing under `vendor/OrcaSlicer/` is edited.

Its header comment states the rule for what may be included there, because the constraint is a
linker one: a vendored `.cpp` included in this translation unit also defines that file's
external-linkage symbols here, while the same symbols are in the archive the crate links, so only
a file whose internal-linkage helpers are the thing being proven is included, and the shim exposes
the helper rather than re-exporting the file's own surface. Where even that is impossible, the
manifest row says so and the helper is proven through its enclosing file's exported surface
(R19).

Add `crates/slic3r-ffi/cxx/proof_shims.cpp` and `crates/slic3r-ffi/src/proof.rs`, the `cxx`
bridge for the proof calls, with a `mod proof;` line in `crates/slic3r-ffi/src/lib.rs`. The shims
take and return plain integer-coordinate data, so no proof reaches into libslic3r's own types from
Rust. Note that `coord_t` is `int64_t` at the pin (`src/libslic3r/libslic3r.h:45-48`, where the
`int32_t` alternative is inside `#if 0`), so a coordinate is carried as a 64-bit integer and never
through an `f64`.

No ported module exists at this step, so this task commits the harness's own proof: **two shims in
`crates/slic3r-ffi/cxx/`, and two tests in `crates/slic3r-proof/tests/harness_paths.rs`** — one per
path: a call to an externally linked `libslic3r` symbol for path 1, and a call to a `static` or
anonymous-namespace helper of an in-scope `.cpp` through the included translation unit for path 3 —
each compared through `src/tiers.rs` at that unit's manifest tier. Pick both units from
`port/manifest.tsv`. The tests live in `crates/slic3r-proof` and not in `crates/slic3r-ffi` because
they compare through `src/tiers.rs`, and `slic3r-proof` is the crate that depends on
`slic3r-ffi`; putting them in `slic3r-ffi` would need a dev-dependency back the other way.
`crates/slic3r-proof/tests/trial_units.rs` is task `35-trial-run`'s file and is not created here.

**The mock check, as a written rule.** Plan step 16 leaves two lines of the `just port-check`
recipe to later steps, and this is one of them (the other is task `29-ffi-bridge-core`'s
substitute check; neither is written in terms of the other's line, and the conflict between the
two is an ordinary merge conflict in the `justfile`). Add the check as its own line in the
`port-check` recipe, implemented as a standalone executable `port/check-mocks` — a PEP 723 script
run through `uv`, like every other `port/*` script. It is **not** a mode of `port/port-status`:
plan step 14 enumerates that script's three modes, and tasks `21-port-status-markers` and
`22-port-status-gates` own it. The rule is written into the script and is not left to judgement:

> A file under `crates/*/tests/`, or a `#[cfg(test)]` module in a crate that holds a ported module
> — a crate holding a file with a `PORT STATUS` trailer — fails when it declares or names an
> identifier matching `mock`, `stub`, `fake` or `dummy` in any case. There is no allowlist,
> because the three proof paths make a stand-in unnecessary.

Like `port/port-status` it is **text-based**: it reads files as lines and never parses Rust,
because it has to pass over Phase A drafts that are not valid Rust (plan step 14).

**What makes this a 5.** Every ported unit's test runs through this crate, so getting the tier
dispatch and the two paths wrong is wrong for the whole fan-out. Three things carry real
uncertainty: the tier dispatch has to read a manifest row written by another task and refuse a
malformed one; the link against the built library pulls in the whole superbuild prefix, which
nobody upstream links from outside CMake; and the included-translation-unit path has a
duplicate-symbol constraint that only appears at link time. It is not split further because the
link and the included translation unit are both `build.rs` work in one crate and are proven by the
same pair of tests.

**What it waits on.** `08-comparator-workspace` for the Rust workspace and the committed
`Cargo.lock`; `13-build-libslic3r` for the library proof paths 1 and 3 link against — without
`just oracle-build` on the machine neither path runs; `14-manifest-columns` for the four columns
`src/tiers.rs` reads; `24-port-check-ci` for the `just port-check` recipe the mock-check line is
added to and for the macOS job that runs `cargo test --workspace`; `29-ffi-bridge-core` for the
`crates/slic3r-ffi` crate, its `build.rs` and the `-ffp-contract=off` flag its own check asserts.

**What it must not touch.** Nothing under `vendor/OrcaSlicer/`, and the submodule pointer stays at
`f92a7c065c5718b8ef1baac1cd704df2911238f4`. The root `Cargo.toml`. `port/manifest.tsv`.
The four generated reports — `port/reports/perf-backlog.md`, `divergences.md`,
`order-containers.md`, `order-comparators.md`. `port/port-status`, which tasks
`21-port-status-markers` and `22-port-status-gates` own; this branch's only `justfile` change is
the one `port-check` line. `crates/slic3r-proof/src/dumps.rs`,
`crates/slic3r-ffi/fuzz/` and the `just fuzz` recipe, which are task `32-proof-dumps-fuzz`'s.
`crates/slic3r-proof/tests/trial_units.rs`, which is task `35-trial-run`'s. No file is added under
`intent/`.

## Done when

- `crates/slic3r-proof` exists with `Cargo.toml`, `src/lib.rs` and `src/tiers.rs`, and
  `cargo test -p slic3r-proof` passes on a machine where `just oracle-build` has run.
- `src/tiers.rs` exposes `compare_unit(source_path, oracle, actual, expected)`, which takes a
  unit's `equality tier`, `tolerance`, `comparison function` and `proof path` from the
  `port/manifest.tsv` row that `source_path` names, compares an `exact` row by exact equality of
  integer-coordinate output, and compares a float row by that row's declared tolerance and
  comparison function.
- `src/tiers.rs` locates `port/manifest.tsv` from an environment variable defaulting to the
  workspace root, hardcodes no path to it, and fails with a named error when the file is absent;
  and it takes the oracle as the caller's argument, because no manifest column holds it.
- A row that declares a tolerance with no reason text, and a row with an empty `proof path` cell,
  are each refused with a named error that names the row, rather than compared.
- Both a passing and a failing comparison print the unit's manifest row path, its tier, its proof
  path and the oracle.
- `crates/slic3r-ffi/build.rs` derives its link line from the CMake build tree — the generated link
  command of `libslic3r_tests` or `fff_print_tests` under `vendor/OrcaSlicer/build/arm64` — rather
  than from a list of names, so the `libslic3r` and `libslic3r_cgal` archives, the `deps_src/`
  archives built into that tree, the superbuild prefix's libraries and the `Foundation` and
  `ModelIO` frameworks all reach the link; it drops the build tree's `Clipper2` archive in favour
  of this crate's own compilation of it, with a comment saying why; and it fails with a message
  naming `just oracle-build` when that build is absent.
- `cargo test -p slic3r-proof` links with no undefined symbol, which is the check that the derived
  link line is complete.
- `build.rs` passes `-ffp-contract=off` to every translation unit it compiles, these two included,
  by setting it on the shared `cxx_build::Build` before any `.file()` call — so
  `crates/slic3r-ffi/cxx/proof_shims.cpp` and `crates/slic3r-ffi/cxx/include_vendored.cpp` cannot
  miss it. Task `29-ffi-bridge-core`'s check is a text check that the literal appears in
  `oracle/build-libslic3r`'s `CMAKE_CXX_FLAGS` and in `crates/slic3r-ffi/build.rs`; it does not
  confirm anything about these two files.
- `crates/slic3r-proof/tests/harness_paths.rs` holds both of the harness's own proofs: one calling
  an externally linked `libslic3r` symbol through `crates/slic3r-ffi/src/proof.rs` for path 1, and
  one calling a `static` or anonymous-namespace helper of an in-scope vendored `.cpp` through
  `cxx/include_vendored.cpp` for path 3, each compared through `src/tiers.rs` at that unit's
  manifest tier. Neither lives in `crates/slic3r-ffi`, and
  `crates/slic3r-proof/tests/trial_units.rs` does not exist on this branch.
- `cxx/include_vendored.cpp` carries a header comment stating which vendored files may be included
  there and why — the duplicate-symbol constraint against the linked archive — and names R19's
  fallback for a helper that cannot be reached even this way.
- `git -C vendor/OrcaSlicer status --porcelain` prints nothing and
  `git -C vendor/OrcaSlicer rev-parse HEAD` prints
  `f92a7c065c5718b8ef1baac1cd704df2911238f4`.
- `just port-check` runs `port/check-mocks` as its own line, and `port/port-status` is unchanged
  by this branch.
- `port/check-mocks` exits non-zero on a file under `crates/*/tests/`, and on a `#[cfg(test)]`
  module in a crate holding a `PORT STATUS` module, that declares or names an identifier matching
  `mock`, `stub`, `fake` or `dummy` in any case; it exits 0 on the tree as committed; the rule is
  written in the script and there is no allowlist.
- `port/check-mocks` reads files as lines, parses no Rust, and exits 0 over a trailer-carrying
  file that is not valid Rust and that no `mod` line reaches.
- `just port-check` passes, `cargo test --workspace` passes, and the branch changes no file under
  `intent/`, no file under `vendor/OrcaSlicer/`, not the root `Cargo.toml`, not
  `port/manifest.tsv` and none of the four generated reports.
