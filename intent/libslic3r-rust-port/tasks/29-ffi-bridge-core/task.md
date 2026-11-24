---
kind: task
step: 17
delivers: [R17]
depends: [08-comparator-workspace, 13-build-libslic3r, 14-manifest-columns, 24-port-check-ci]
size: 5
---
# Task: Bind the vendored Clipper2 through cxx and pin the build flags

## What to build

This is part 1 of the plan's step 17: the `slic3r-ffi` crate, its `build.rs`, the Clipper2 bridge,
and the two checks that join `just port-check`. Part 2 — admesh, qhull, libnest2d, Boost.Polygon's
Voronoi builder and the NLopt determination — is the task that depends on this one.

Add `crates/slic3r-ffi` with `Cargo.toml`, `build.rs`, `src/lib.rs`, `src/clipper2.rs` and
`tests/ffi_smoke.rs`. The root `Cargo.toml` is not edited: its `members` is `crates/*`, so the
directory appearing is enough. `Cargo.lock` is committed with whatever this crate's dependencies
add to it.

**`build.rs`, and the facts from the pin it has to keep.** It uses `cxx_build` and compiles the
vendored Clipper2 sources in `vendor/OrcaSlicer/deps_src/clipper2/` exactly as they are at the pin.
Nothing under `vendor/OrcaSlicer/` is edited, generated into, or patched.

- The sources are the five the vendored CMake names
  (`deps_src/clipper2/CMakeLists.txt:26-32`): `Clipper2Lib/src/clipper.engine.cpp`,
  `clipper.offset.cpp`, `clipper.rectclip.cpp`, `clipper.triangulation.cpp` and `clipper2_z.cpp`.
  **Two include directories are needed, not one.** `Clipper2Lib/include` (`:37-39`) is the
  library's own, and the **superbuild prefix's `include` directory** is the second, because
  `Clipper2Lib/src/clipper.engine.cpp:14` is `#include <oneapi/tbb/scalable_allocator.h>` — it
  calls `scalable_malloc` at `:35` — and the vendored CMake supplies that header transitively
  through `target_link_libraries(Clipper2 PRIVATE TBB::tbbmalloc)`
  (`deps_src/clipper2/CMakeLists.txt:42`), which `cxx_build` has no equivalent of. Take the prefix
  from the same environment variable, with the same default, that resolves the link search path
  below, so the include path and the link path cannot disagree. The standard is C++17 (`:5-6`).
- `USINGZ` is never defined on the command line. `clipper2_z.cpp` defines it itself and then
  `#include`s the other four translation units (`Clipper2Lib/src/clipper2_z.cpp:1-9`), and
  `clipper.core.h:22-25` puts that build in the namespace `Clipper2Lib_Z` beside the plain
  `Clipper2Lib`. That is why compiling all five together is not a duplicate-symbol error, and why
  the library carries both the 2D and the Z-enabled surface — which is the surface
  `src/libslic3r/ClipperZUtils.cpp:4` reaches through `<clipper2/clipper2_z.hpp>`.
- Every translation unit this crate compiles carries `-ffp-contract=off`, passed as
  `cxx_build::Build::flag("-ffp-contract=off")`, including the units later tasks add to the same
  build. That flag is not a preference: the library built with it is the normative oracle for every
  per-unit and per-stage proof, so a reference compiled without it is not the oracle those proofs
  are supposed to compare against, and the compiler would otherwise fuse multiply-add expressions
  that Rust never fuses.
- Clipper2's engine nodes are allocated through `tbbmalloc` in the vendored copy
  (`deps_src/clipper2/CMakeLists.txt:41-42`, `Clipper2Lib/src/clipper.engine.cpp:32`), so the crate
  links `tbbmalloc` from the dependency superbuild's install prefix — the one
  `oracle/build-libslic3r` produces, `vendor/OrcaSlicer/deps/build/arm64/OrcaSlicer_dep/usr/local`.
  `build.rs` takes that prefix from an environment variable with that path as its default, emits
  the `cargo:rustc-link-search`, `cargo:rustc-link-lib` and `cargo:rerun-if-env-changed` lines for
  it, and fails with a named error when the prefix is absent rather than falling back to a system
  or Homebrew TBB — the oracle build passes `CMAKE_IGNORE_PREFIX_PATH` to keep Homebrew out, so a
  Homebrew `tbbmalloc` would not be the same library the oracle links.

**`src/clipper2.rs`.** The `cxx` bridge over the Clipper2 entry points libslic3r itself calls,
through `src/libslic3r/ClipperUtils.{hpp,cpp}` for the 2D surface and
`src/libslic3r/ClipperZUtils.{hpp,cpp}` for the `Clipper2Lib_Z` surface. It starts with what the
smoke test needs — one boolean operation and one offset on integer paths through `Clipper2Lib`, and
one call through `Clipper2Lib_Z` — and grows per ported unit in later tasks. The bridge only passes
values through; it reimplements nothing, and no Rust polygon-clipping crate is substituted for any
part of it. The vendored copy is not stock Clipper2 2.0.1: beside the `tbbmalloc` allocation it
changes behaviour in the offsetter, at `Clipper2Lib/src/clipper.offset.cpp:301` (the step count is
rounded the way Clipper1 did, so round offsets keep their vertices) and `:377` (concave corners are
joined at the crossing of both edge offsets), which is why no published crate matches it and why
this is a binding rather than a dependency.

**`src/lib.rs`** carries, in its crate-level doc comment, a table with one row per library this
crate is responsible for — Clipper2, admesh, qhull, libnest2d, Boost.Polygon's Voronoi builder and
NLopt — with the columns: library, where its source comes from at the pin, bound or not bound, and
the `path:line` evidence for that. This task fills Clipper2's row and marks the other five as the
second task's; that table is where the NLopt determination is recorded.

**The two checks, both added as lines of the `just port-check` recipe in the `justfile`.** That
recipe is created by the task that splits the checks between the two CI jobs, which leaves exactly
these lines to the step that makes them pass; this task adds no new check script and no CI job.

1. **The flag cannot drift.** The check fails unless `-ffp-contract=off` appears both in
   `oracle/build-libslic3r`'s `CMAKE_CXX_FLAGS` and in `crates/slic3r-ffi/build.rs`, and its
   message names which of the two is missing it. The two compilations the proofs compare against
   are otherwise free to diverge silently.
2. **No substitutes.** The check fails when any `crates/*/Cargo.toml` in the workspace declares a
   dependency on a named Rust substitute for one of the six libraries, naming the manifest and the
   dependency. The substitute list is written into the check, covers all six, and is reviewed in
   this task's pull request; replacing these libraries with Rust implementations is out of scope
   for this intent by a recorded decision, so there is no allowlist.

**Two things this crate is not.** It ports no libslic3r source file, so no `PORT STATUS` trailer
belongs in it and it gets no manifest row of its own — the manifest's crate map is what later port
tasks read to know which crate calls this one. And the proof harness's rule against stand-ins
applies to its tests, which a later task turns into a check: no identifier in `tests/ffi_smoke.rs`
matching `mock`, `stub`, `fake` or `dummy`, in any case. Any `unsafe` block this crate writes
carries a `// SAFETY:` comment, which `just port-status --per-commit` already checks.

**Why this is a 5.** It is a dependency outside the repository compiled from vendored sources
exactly as they are at the pin, with `-ffp-contract=off` on every translation unit and a check that
the flag cannot drift from the CMake build. Clipper2's vendored copy is patched, so no published
crate matches it, and the superbuild prefix, the `tbbmalloc` link and the two-namespace build are
each a way for the bridge to compile and still be the wrong reference. It is not split further
because the crate, its `build.rs` and its first bridge are one piece of work: neither half proves
anything alone.

**What it waits for.** The Rust workspace; the dependency superbuild and the built `libslic3r`,
which is where the prefix and `tbbmalloc` come from; the manifest's crate map; and the
`just port-check` recipe the two checks are added to.

## Done when

- `cargo test -p slic3r-ffi` passes, and `tests/ffi_smoke.rs` has a case that calls Clipper2
  through the bridge in both the plain and the Z-enabled namespace and asserts on integer
  coordinates the C++ returned.
- `build.rs` compiles exactly the five vendored Clipper2 sources
  `deps_src/clipper2/CMakeLists.txt:26-32` names, from `vendor/OrcaSlicer/deps_src/`, with
  `Clipper2Lib/include` **and the superbuild prefix's `include` directory** on the include path —
  the latter for `<oneapi/tbb/scalable_allocator.h>`, `clipper.engine.cpp:14` — and C++17 set, and
  defines `USINGZ` nowhere on the command line.
- Every translation unit `build.rs` compiles is given `-ffp-contract=off`, set on the shared
  `cxx_build::Build` before any `.file()` call so a unit a later task adds cannot miss it.
- The crate links `tbbmalloc` from the superbuild install prefix, the prefix comes from an
  environment variable defaulting to
  `vendor/OrcaSlicer/deps/build/arm64/OrcaSlicer_dep/usr/local`, and a missing prefix fails the
  build with a named error rather than linking a system or Homebrew TBB.
- `just port-check` exits non-zero, naming the side at fault, when `-ffp-contract=off` is removed
  from either `oracle/build-libslic3r`'s `CMAKE_CXX_FLAGS` or `crates/slic3r-ffi/build.rs`, and
  exits 0 with both present.
- `just port-check` exits non-zero, naming the manifest and the dependency, when a
  `crates/*/Cargo.toml` is given a dependency on a name in the substitute list, and that list has
  entries for all six libraries.
- `src/lib.rs`'s crate doc table has a row for each of the six libraries, with Clipper2's filled in
  and its `path:line` evidence cited.
- `git -C vendor/OrcaSlicer status --porcelain` prints nothing after a full build and test run, and
  the branch does not edit the root `Cargo.toml`.
- No identifier in `tests/ffi_smoke.rs` matches `mock`, `stub`, `fake` or `dummy`, and every
  `unsafe` block in the crate carries a `// SAFETY:` comment.
