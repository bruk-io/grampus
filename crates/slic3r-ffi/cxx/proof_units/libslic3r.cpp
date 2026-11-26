// Proof-path-1 shim for the `libslic3r.cpp`/`libslic3r.h` unit (R19), compiled onto
// `crates/slic3r-ffi`'s shared `cxx_build::Build` by this crate's `build.rs` -- the one
// sanctioned route a ported unit reaches the oracle by (`docs/PORTING.md`, "Decisions recorded
// on 2026-11-25"). `crates/slic3r-libslic3r`'s proof calls this through
// `slic3r_proof::proof_units::libslic3r`.
//
// `src/libslic3r/libslic3r.cpp`'s one in-scope line, `double SCALING_FACTOR =
// SCALING_FACTOR_INTERNAL;` (:3), sits at file scope, *before* `namespace Slic3r {` opens
// (`libslic3r.h:123`), so the externally linked symbol is the global `::SCALING_FACTOR`, never
// `Slic3r::SCALING_FACTOR` -- verified directly against the built archive:
// `nm vendor/OrcaSlicer/build/arm64/src/libslic3r/Release/liblibslic3r.a` shows
// `D _SCALING_FACTOR`, a defined data symbol with no namespace mangling (see this task's pull
// request body for the exact command and output).
//
// The quoted include below names the vendored header as `"libslic3r/libslic3r.h"`, not the bare
// `"libslic3r.h"` every in-directory vendored file uses for it: this shim's own declarations
// header is named `libslic3r.h` too (the shim-naming convention names it after the unit's
// manifest `target module` stem, which collides with the vendored header's own name here), and a
// quoted `#include` always checks the including file's own directory first -- a bare
// `"libslic3r.h"` from this file would re-resolve to the sibling declarations header, never
// reaching the vendored one. The subdirectory-qualified spelling is not invented for this file:
// it is the exact form other in-tree libslic3r translation units outside `src/libslic3r/` itself
// already use for this header (e.g. `src/libslic3r/Fill/FillAdaptive.cpp:13`), and it resolves
// here because `build.rs` puts `vendor/OrcaSlicer/src` (not just `.../src/libslic3r`) on the
// include path.
#include "libslic3r.h"

#include "libslic3r/libslic3r.h"

namespace slic3r_ffi {
namespace proof_units {
namespace libslic3r {

double path1_scaling_factor() {
    return ::SCALING_FACTOR;
}

} // namespace libslic3r
} // namespace proof_units
} // namespace slic3r_ffi
