// Proof shim for the `GCode/PchipInterpolatorHelper.cpp`/`.hpp` unit (R19), the float leg of
// task `35-trial-run`. Compiled onto `crates/slic3r-ffi`'s shared `cxx_build::Build` by this
// crate's `build.rs` -- the one sanctioned route a ported unit reaches the oracle by
// (`docs/PORTING.md`, "Decisions recorded on 2026-11-25"). `crates/slic3r-gcode`'s proof calls
// this through `slic3r_proof::proof_units::pchip_interpolator_helper`.
//
// `PchipInterpolatorHelper`'s methods are ordinary (non-inline, non-template) member functions
// defined in the vendored `.cpp`, so they are externally linked symbols in the archive the
// derived link line carries (`liblibslic3r.a`) -- this row's declared proof path 1. `cxx`
// aborts the process on an uncaught C++ exception crossing the bridge, which this shim does not
// use: `path1_constructing_throws` catches `std::invalid_argument` itself and reports whether
// it was thrown as a plain `bool`, rather than letting it escape.
#include "pchip_interpolator_helper.h"

#include "GCode/PchipInterpolatorHelper.hpp"

#include <stdexcept>

namespace slic3r_ffi {
namespace proof_units {
namespace pchip_interpolator_helper {

rust::Vec<double> path1_interpolate_many(rust::Vec<double> x, rust::Vec<double> y, rust::Vec<double> queries) {
    const std::vector<double> xv(x.begin(), x.end());
    const std::vector<double> yv(y.begin(), y.end());
    PchipInterpolatorHelper helper(xv, yv);
    rust::Vec<double> out;
    for (const double q : queries) {
        out.push_back(helper.interpolate(q));
    }
    return out;
}

bool path1_constructing_throws(rust::Vec<double> x, rust::Vec<double> y) {
    const std::vector<double> xv(x.begin(), x.end());
    const std::vector<double> yv(y.begin(), y.end());
    try {
        PchipInterpolatorHelper helper(xv, yv);
        (void)helper;
        return false;
    } catch (const std::invalid_argument &) {
        return true;
    }
}

} // namespace pchip_interpolator_helper
} // namespace proof_units
} // namespace slic3r_ffi
