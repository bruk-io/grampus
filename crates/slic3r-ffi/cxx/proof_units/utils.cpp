// Proof shim for the `Utils.hpp`/`utils.cpp` unit (task 57-foundation-utils, R19), compiled onto
// `crates/slic3r-ffi`'s shared `cxx_build::Build` by this crate's `build.rs` -- the one
// sanctioned route a ported unit reaches the oracle by (`docs/PORTING.md`, "Decisions recorded
// on 2026-11-25"). `crates/slic3r-utils`'s proof calls this through
// `slic3r_proof::proof_units::utils`.
//
// **Route note, recorded here and in this task's pull request body (`docs/WORKFLOW.md`,
// "Decisions that govern over a task's file list", corollary 1, plus a second divergence this
// comment also covers): the manifest row's `proof path` cell is `3`, but this shim is shaped
// like proof path 1, not proof path 3.** Path 3 means `#include`ing the vendored `.cpp` whole
// to reach an internal-linkage helper. `utils.cpp` was measured before this shim was written:
//
//   nm -m vendor/OrcaSlicer/build/arm64/src/libslic3r/CMakeFiles/libslic3r.dir/Release/utils.cpp.o \
//     | grep -E '^\S+ \(.*\) external ' | grep -v weak | c++filt | sort -u | wc -l
//   -> 99
//
// 99 strong, defined, externally-linked `Slic3r::` symbols -- not the 1-4 symbol range
// `cxx/proof_units/clipper_z_utils.cpp`'s own sweep found viable for the same `#include` trick
// (it measured 94 proof-path-3 `.cpp` files and the chosen one, at 2 symbols, was already tied
// for the practical minimum). `crates/slic3r-ffi/build.rs`'s own doc comment states that the
// derived link line reproduces the oracle's `libslic3r_tests`/`fff_print_tests` link, i.e. the
// whole `liblibslic3r.a` archive -- so every one of these 99 symbols (`set_logging_level`,
// `data_dir`, `is_plain_file`, `split_string`, `xml_escape`, ...) is already provided by that
// archive on every build of this crate. `#include`ing `utils.cpp` here a second time would
// define all 99 again in this translation unit, and `ld` would extract both archives' copies
// into the same link the moment anything needed even one of them -- a duplicate-symbol
// collision across effectively the whole file, not the few-symbol risk `ClipperZUtils.cpp`
// accepted. This is far past the "may not be `#include`d here this way" line that file's header
// comment states, so path 3 is not attempted for this unit.
//
// R19's stated fallback applies instead: "the helper is proven through its enclosing file's
// exported surface." `Slic3r::get_utf8_sequence_length(const char*, size_t)` (`utils.cpp:1390-
// 1452`) is one of those 99 externally-linked symbols, is pure (no global state, no I/O), and
// takes/returns only plain integer data -- exactly the shape decision 1 sanctions for a shim
// payload. This shim declares the vendored header and calls that symbol directly, the same
// shape `cxx/proof_units/object_id.cpp`'s path-1 shims use, with no `#include` of any `.cpp`.
//
// The marker on the unit's `// PROOF: 3` test still reads `3`, because `port/port-status`'s
// `proof_present` matches the manifest row's own cell verbatim (`docs/WORKFLOW.md`, "How a
// proof is recognised"); the route actually taken is path 1 in shape, recorded here and in the
// pull request body rather than silently reconciled with the cell.
#include "utils.h"

#include "Utils.hpp"

namespace slic3r_ffi {
namespace proof_units {
namespace utils {

// `seq` must be non-empty: the vendored function itself reads `seq[0]` with no length check
// (`utils.cpp:1393`), so an empty `seq` would be undefined behaviour on the oracle side too.
// The Rust-side proof test never calls this with an empty vector.
std::uint64_t path1_get_utf8_sequence_length(rust::Vec<std::uint8_t> seq) {
    return static_cast<std::uint64_t>(
        Slic3r::get_utf8_sequence_length(reinterpret_cast<const char *>(seq.data()), seq.size()));
}

} // namespace utils
} // namespace proof_units
} // namespace slic3r_ffi
