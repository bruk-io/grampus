// Proof path 3 (R19, plan step 18, task 31-proof-harness): a test translation unit that
// `#include`s a vendored `.cpp` whole, so this crate can reach a `static` or anonymous-namespace
// helper the pin never exports -- 49 of 268 vendored `.cpp` files declare an anonymous namespace
// and 112 declare `static` free functions, none with external linkage, and the pin
// (f92a7c065c5718b8ef1baac1cd704df2911238f4) forbids adding an export. `#include`ing the file
// does not modify it, so the pin holds; nothing under vendor/OrcaSlicer/ is edited by this file.
//
// WHICH VENDORED FILES MAY BE #include'd HERE, AND WHY (the duplicate-symbol constraint):
//
// A vendored `.cpp` `#include`d here also defines *that file's externally linked* symbols in
// this translation unit -- and the same symbols already exist in `liblibslic3r.a`, which
// `crates/slic3r-ffi/build.rs`'s derived link line also links. On this pin's macOS `ld`, a
// static archive member is only extracted when something still-unresolved needs it, so as long
// as nothing else this crate links needs that vendored `.cpp`'s externally linked symbols, the
// archive's own copy of that translation unit is simply never pulled in, and there is no
// duplicate definition. This was verified empirically for the file chosen below, by compiling
// and linking it against this crate's derived link line directly with no undefined or duplicate
// symbol (see this task's pull request body for the command and its output) -- it is not an
// assumption. A vendored file whose externally linked symbols are *already* needed elsewhere in
// this crate's link (so the archive member would be unavoidably extracted too) would collide,
// and may not be `#include`d here this way. Where a helper's enclosing file cannot be included
// at all (because of that collision, or for any other reason), the manifest row says so and the
// helper is proven through its enclosing file's exported surface instead -- R19's stated
// fallback.
//
// COMPILE FLAGS ALSO DIFFER FROM THE ORACLE'S OWN COMPILATION OF THE INCLUDED FILE, BEYOND
// `-ffp-contract=off`: this translation unit is compiled at this crate's own optimisation level
// (`-O0`, no `-DNDEBUG`) and without libslic3r's own `DEFINES`, not at the oracle's `-O3
// -DNDEBUG`. That is irrelevant to a pure data repack such as `to_paths64` below, but a later
// candidate file must also be checked for `assert()` (live at `-O0`/no `NDEBUG`, compiled out in
// the oracle) and for code guarded by one of libslic3r's own `DEFINES`, not only for the
// duplicate-symbol collision above.
//
// THE DUPLICATE-SYMBOL CONSTRAINT APPLIED TO THIS FILE'S OWN CHOICE: `ClipperZUtils.cpp` declares
// two externally-linked functions, `Slic3r::ClipperZUtils::clip_zpaths` and `::area` (`to_paths64`
// and `append_zpaths`, below, are both `static`). Two is low, but it is NOT the minimum, and this
// file is not chosen for being the minimum. The sweep, re-measured: every in-scope `proof path`
// `3` `.cpp` row whose object this worktree's oracle build tree holds (92 objects, under both
// `libslic3r.dir/Release/` and `libslic3r_cgal.dir/Release/`), counting per object the strong
// (non-weak), *defined* (not undefined-reference) externally-linked `Slic3r::` symbols reported by
// `nm -m`, demangled -- `MeshRepair.cpp` 1 (`Slic3r::repair_mesh`), `ClipperZUtils.cpp` 2,
// `MutablePolygon.cpp` 5, `Algorithm/LineSplit.cpp` 6, `Time.cpp` 7, and up from there.
// `MeshRepair.cpp`, not this file, is the minimum. No candidate has 0, so no choice of file
// removes this constraint outright: it can only be made smaller, and it has to be documented
// wherever it lands.
//
// WHY THIS FILE AND NOT `MeshRepair.cpp`, THE 1-SYMBOL MINIMUM: `MeshRepair.cpp` is 1215 lines
// that pull CGAL into whatever translation unit includes them. Measured at this file's own flags
// (`-std=c++17 -O0 -ffp-contract=off`, no `-DNDEBUG`, this crate's include set): a translation
// unit that `#include`s `MeshRepair.cpp` compiles in 9.39s and emits exactly that one symbol,
// against 1.50s for the `#include` of `ClipperZUtils.cpp` below -- a cost every consumer of
// `slic3r-ffi` pays whenever this translation unit is rebuilt. Emitting `Slic3r::repair_mesh` here
// would also drag CGAL/GMP/MPFR link dependencies, and CGAL's own live assertions, into exactly
// the `-O0`/no-`NDEBUG` configuration the paragraph above warns every candidate must be checked
// against. `MeshRepair.cpp` does carry one advantage this file does not: its manifest row is
// `fuzz target` `n/a` where this file's is `required`, so the collision the next paragraph forbids
// could not arise for it at all. So relocating the path-3 proof there is a real option for a
// later task -- it compiles today -- declined here for the compile-cost and assertion reasons
// above, and because the link as it stands is provably collision-free.
//
// THE COLLISION THIS FILE'S TWO SYMBOLS LEAVE STANDING: including this file here also defines
// `clip_zpaths` and `area` in this crate's own archive, alongside `liblibslic3r.a`'s copy of the
// same two symbols. Nothing today needs either symbol resolved, so
// neither archive member is ever extracted and there is no collision (verified: see this task's
// pull request body for the command and its output). A later task MUST NOT add, anywhere in
// `crates/slic3r-ffi` or a crate that links it, a direct FFI binding that calls the oracle's
// `clip_zpaths` or `area` through `liblibslic3r.a` (for example a differential-fuzz proof path 1
// for `slic3r-clipperzutils`) while this file is still `#include`d here: doing so would make `ld`
// extract both archives' copies of those two symbols into the same link, which is a duplicate
// definition. The `slic3r-clipperzutils` port task should route any need for `clip_zpaths`/`area`
// through this same included translation unit instead (as `path3_to_paths64` below already does
// for `to_paths64`), or stop `#include`ing `ClipperZUtils.cpp` here first. This branch does not
// record this constraint on `port/manifest.tsv`'s `ClipperZUtils.cpp` row -- task
// `31-proof-harness` may not touch that file; see this task's pull request body.
//
// THIS FILE'S CHOICE: `src/libslic3r/ClipperZUtils.cpp` (`port/manifest.tsv`'s `proof path` `3`
// row). Its `static` `to_paths64` (internal linkage, reachable only in this translation unit,
// after the `#include` below) converts one `Slic3r::ClipperZUtils::ZPath` into one
// `Clipper2Lib_Z::Path64`, both built from plain `coord_t` (int64_t) coordinates -- a pure
// repack, no arithmetic, which is exactly what `crates/slic3r-proof/tests/harness_paths.rs`
// checks comes back unchanged, through `tiers::compare_unit` at that row's declared tier
// (`exact`).

#include "proof_shims.h"

#include "ClipperZUtils.cpp"

namespace slic3r_ffi {
namespace proof {

rust::Vec<ProofZPoint> path3_to_paths64(const rust::Vec<ProofZPoint> &path) {
    Slic3r::ClipperZUtils::ZPath cpp_path;
    cpp_path.reserve(path.size());
    for (const ProofZPoint &p : path) {
        cpp_path.emplace_back(p.x, p.y, p.z);
    }

    // `to_paths64` is `static` (internal linkage): reachable only because this translation unit
    // `#include`s `ClipperZUtils.cpp` above, never through a header declaration.
    Slic3r::ClipperZUtils::ZPaths cpp_paths{cpp_path};
    auto result = Slic3r::ClipperZUtils::to_paths64(cpp_paths);

    rust::Vec<ProofZPoint> out;
    if (!result.empty()) {
        for (const auto &pt : result[0]) {
            out.push_back(ProofZPoint{pt.x, pt.y, pt.z});
        }
    }
    return out;
}

} // namespace proof
} // namespace slic3r_ffi
