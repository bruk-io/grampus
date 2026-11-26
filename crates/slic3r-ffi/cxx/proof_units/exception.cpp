// Proof shim for the `Exception.hpp` unit (R19, proof path 2, task 52-foundation-common-defs,
// `port/foundation.tsv` row `foundation-01`), compiled onto `crates/slic3r-ffi`'s shared
// `cxx_build::Build` by this crate's `build.rs` -- the one sanctioned route a ported unit
// reaches the oracle by (`docs/PORTING.md`, "Decisions recorded on 2026-11-25").
// `crates/slic3r-exception`'s proof calls this through
// `slic3r_proof::proof_units::exception`.
//
// No field in the committed stage-dump corpus carries an exception message, an object id or any
// other value this hierarchy produces, and no oracle run that reaches a compared dump throws
// (`crates/slic3r-commondefs/src/common_defs.rs`'s doc comment states the same grep for the
// shared `posSlice`-seeded "2" cell). Decision 4 names the per-unit proof bridges, not only the
// dump corpus, as a "2" row's oracle entry point, so this shim constructs each concrete type
// directly and reads its real, compiled `what()`/`objectId()` -- never throwing or catching a
// C++ exception across the bridge, which `cxx` aborts on without the `Result` convention this
// shim does not use.
#include "exception.h"

#include "Exception.hpp"

#include <vector>

namespace slic3r_ffi {
namespace proof_units {
namespace exception {

rust::String path2_exception_what(rust::Str msg) {
    return rust::String(Slic3r::Exception(std::string(msg)).what());
}
rust::String path2_critical_exception_what(rust::Str msg) {
    return rust::String(Slic3r::CriticalException(std::string(msg)).what());
}
rust::String path2_runtime_error_what(rust::Str msg) {
    return rust::String(Slic3r::RuntimeError(std::string(msg)).what());
}
rust::String path2_logic_error_what(rust::Str msg) {
    return rust::String(Slic3r::LogicError(std::string(msg)).what());
}
rust::String path2_hard_crash_what(rust::Str msg) {
    return rust::String(Slic3r::HardCrash(std::string(msg)).what());
}
rust::String path2_invalid_argument_what(rust::Str msg) {
    return rust::String(Slic3r::InvalidArgument(std::string(msg)).what());
}
rust::String path2_out_of_range_what(rust::Str msg) {
    return rust::String(Slic3r::OutOfRange(std::string(msg)).what());
}
rust::String path2_io_error_what(rust::Str msg) {
    return rust::String(Slic3r::IOError(std::string(msg)).what());
}
rust::String path2_file_io_error_what(rust::Str msg) {
    return rust::String(Slic3r::FileIOError(std::string(msg)).what());
}
rust::String path2_host_network_error_what(rust::Str msg) {
    return rust::String(Slic3r::HostNetworkError(std::string(msg)).what());
}
rust::String path2_export_error_what(rust::Str msg) {
    return rust::String(Slic3r::ExportError(std::string(msg)).what());
}
rust::String path2_placeholder_parser_error_what(rust::Str msg) {
    return rust::String(Slic3r::PlaceholderParserError(std::string(msg)).what());
}

rust::String path2_slicing_error_what(rust::Str msg) {
    return rust::String(Slic3r::SlicingError(std::string(msg)).what());
}
uint64_t path2_slicing_error_default_object_id(rust::Str msg) {
    return static_cast<uint64_t>(Slic3r::SlicingError(std::string(msg)).objectId());
}
uint64_t path2_slicing_error_object_id(rust::Str msg, uint64_t object_id) {
    return static_cast<uint64_t>(
        Slic3r::SlicingError(std::string(msg), static_cast<size_t>(object_id)).objectId());
}

rust::String path2_slicing_errors_from_message_what(rust::Str msg) {
    return rust::String(Slic3r::SlicingErrors(std::string(msg)).what());
}
rust::String path2_slicing_errors_from_vector_what(uint32_t count) {
    std::vector<Slic3r::SlicingError> errors(count, Slic3r::SlicingError("e"));
    return rust::String(Slic3r::SlicingErrors(errors).what());
}
int64_t path2_slicing_errors_from_vector_count(uint32_t count) {
    std::vector<Slic3r::SlicingError> errors(count, Slic3r::SlicingError("e"));
    return static_cast<int64_t>(Slic3r::SlicingErrors(errors).errors_.size());
}

} // namespace exception
} // namespace proof_units
} // namespace slic3r_ffi
