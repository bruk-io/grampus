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
// `nm vendor/OrcaSlicer/build/arm64/src/libslic3r/Release/liblibslic3r.a | awk '/\.o:/{f=$0}
// /_SCALING_FACTOR/{print f": "$0}' | grep -v ' U '` prints exactly one defining line,
// `libslic3r.cpp.o:: 0000000000001c70 D _SCALING_FACTOR` -- a defined data symbol (`D`) with no
// namespace mangling, in the one object file that should define it; every other object file's
// own `U _SCALING_FACTOR` (several, e.g. `LineSplit.cpp.o`, `RegionExpansion.cpp.o`) is an
// undefined reference to that same global, consistent with it being read, never redefined,
// elsewhere in the archive.
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

// Added on a review pass, for the `libslic3r.h` unit's own `// PROOF: 2` test
// (`crates/slic3r-libslic3r/tests/libslic3r_proof.rs`): that test used to read a scaled
// coordinate from a real committed stage dump but then compare the port's `unscale::<f64>` of
// it only against a hand-derived literal `10.0` -- the oracle produced the input but neither
// side of the actual comparison (`docs/PORTING.md` decision 4, second half, the same
// `proof-marker-not-differential` shape `cxx/proof_units/format.cpp` closes for the
// `format.hpp` unit). This calls the vendored template directly, `Slic3r::unscale<double>(v)`
// (`libslic3r.h:137-138`), so the test can compare the port's own `unscale::<f64>` against what
// the oracle itself computes for the exact same scaled input.
double path2_unscale_double(int64_t scaled) {
    return ::Slic3r::unscale<double>(static_cast<::coord_t>(scaled));
}

} // namespace libslic3r
} // namespace proof_units
} // namespace slic3r_ffi
