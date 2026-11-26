// Proof-path-2 shim for the `enum_bitmask.hpp` unit (R19), compiled onto `crates/slic3r-ffi`'s
// shared `cxx_build::Build` by this crate's `build.rs` -- the one sanctioned route a ported unit
// reaches the oracle by (`docs/PORTING.md`, "Decisions recorded on 2026-11-25"). This unit's
// manifest row carries `proof path: 2`, but `enum_bitmask.hpp` is a header-only class template
// with no pipeline stage of its own: no in-scope instantiation's output reaches any committed
// stage dump (see `crates/slic3r-enum-bitmask/src/enum_bitmask.rs`'s module doc). Decision 1's
// shim mechanism is not limited to proof paths 1 and 3 -- `cxx/proof_units/object_id.cpp`'s
// `path2_header_observations` already proves a `proof path: 2` row this same way, driving the
// vendored header's own inline code directly rather than reading a dump -- so this shim does the
// same for `enum_bitmask.hpp`: it instantiates the real vendored `Slic3r::enum_bitmask<T>`
// template (`libslic3r/enum_bitmask.hpp`) with a proof-local three-value enum (the production
// enum this header is actually instantiated with, `CutUtils.hpp:18`'s `ModelObjectCutAttribute`,
// pulls in `Model.hpp` and is not needed: the class template itself, not which enum parameterises
// it, is what this row ports) and reconstructs the combined bitmask through the template's own
// public `has()` method -- never through its private `m_bits` field, which this translation unit
// has no access to.
#include "enum_bitmask.h"

#include "libslic3r/enum_bitmask.hpp"

namespace Slic3r {

// Proof-local enum, standing in for a real in-scope one the same way `cxx/proof_units/
// object_id.cpp`'s `ProofObject` stands in for `ObjectBase`'s own derived types: the template
// under test (`Slic3r::enum_bitmask<T>`) is generic in its enum parameter, so any three-value
// enum exercises the exact same vendored logic a real caller's enum would.
enum class ProofCutAttribute : int { OptA, OptB, OptC };
using ProofCutAttributes = enum_bitmask<ProofCutAttribute>;
// `ENABLE_ENUM_BITMASK_OPERATORS` (`enum_bitmask.hpp:57`) must be invoked unqualified inside
// `namespace Slic3r`, exactly as `CutUtils.hpp:20` invokes it for its own enum -- it expands to
// an unqualified `template<> struct is_enum_bitmask_type<x>`.
ENABLE_ENUM_BITMASK_OPERATORS(ProofCutAttribute);

} // namespace Slic3r

namespace slic3r_ffi {
namespace proof_units {
namespace enum_bitmask {

int64_t path2_combined_mask() {
    using Slic3r::ProofCutAttribute;
    // `ModelObjectCutAttribute::KeepUpper | ... ::KeepLower | ... ::KeepAsParts`
    // (`CutUtils.hpp:37-39`), reproduced with the proof-local enum in the same OR order.
    const Slic3r::ProofCutAttributes combined =
        ProofCutAttribute::OptA | ProofCutAttribute::OptB | ProofCutAttribute::OptC;
    // Reconstructed bit-for-bit through the template's own public `has()`
    // (`enum_bitmask.hpp:43-44`), never through its private `m_bits`: `mask_value(o) = 1 <<
    // underlying_type(o)` (`:18`), so `OptA` (0) -> bit `1`, `OptB` (1) -> bit `2`, `OptC` (2) ->
    // bit `4`.
    const int64_t bit_a = combined.has(ProofCutAttribute::OptA) ? 1 : 0;
    const int64_t bit_b = combined.has(ProofCutAttribute::OptB) ? 2 : 0;
    const int64_t bit_c = combined.has(ProofCutAttribute::OptC) ? 4 : 0;
    return bit_a | bit_b | bit_c;
}

} // namespace enum_bitmask
} // namespace proof_units
} // namespace slic3r_ffi
