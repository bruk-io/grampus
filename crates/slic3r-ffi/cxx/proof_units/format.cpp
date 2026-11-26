// Proof-path-2 shim for the `format.hpp` unit (R19), compiled onto `crates/slic3r-ffi`'s shared
// `cxx_build::Build` by this crate's `build.rs` -- the one sanctioned route a ported unit
// reaches the oracle by (`docs/PORTING.md`, "Decisions recorded on 2026-11-25").
//
// Added on a review pass, replacing a `// PROOF: 2` test that was not actually differential
// (`docs/PORTING.md` decision 4, second half): the test it proves compared the port's own
// `format("%1%", print_z)` output only against a hand-transcribed literal `"0.2"`, so the oracle
// produced neither side of the comparison -- a `proof-marker-not-differential` shape, the same
// gap `cxx/proof_units/enum_bitmask.cpp` and `cxx/proof_units/execution.cpp` close for their own
// `proof path: 2` rows (`format.hpp` has no pipeline stage of its own either: no committed stage
// dump carries a `boost::format`-rendered string). This shim calls the real vendored
// `Slic3r::format` -- which forwards to the pin's own linked `boost::format`, not a
// reimplementation -- so the test can compare the port's rendering against what the oracle
// itself produces for the exact same input.
//
// Returns the rendered string as its UTF-8 bytes (`rust::Vec<int64_t>`), the one shape
// decision 1's carried-type list sanctions for a non-numeric result ("a plain integer or a
// plain floating-point value, a `rust::Vec` of those, ... -- never a libslic3r type"): that list
// does not include `rust::String`, so this shim does not return one.
#include "format.h"

#include "format.hpp"

namespace slic3r_ffi {
namespace proof_units {
namespace format {

rust::Vec<int64_t> path2_format_positional_bytes(double v) {
    const std::string s = Slic3r::format("%1%", v);
    rust::Vec<int64_t> out;
    out.reserve(s.size());
    for (unsigned char c : s) {
        out.push_back(static_cast<int64_t>(c));
    }
    return out;
}

} // namespace format
} // namespace proof_units
} // namespace slic3r_ffi
