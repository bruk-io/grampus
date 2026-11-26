#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart -- `rust::String` and the bridge's own signatures -- at
// <crate-name>/src/proof_units/exception.rs.h on the include path it sets up, and
// `cxx/proof_units/exception.cpp` is the one translation unit that defines the functions below.
#include "slic3r-ffi/src/proof_units/exception.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace exception {

// One function per concrete type in `Exception.hpp`'s hierarchy (`:13-28`): constructs the type
// from `msg` through its inherited `std::runtime_error` constructor and returns `what()`
// unchanged -- the behaviour every variant of this port's own `Exception` enum assumes.
rust::String path2_exception_what(rust::Str msg);
rust::String path2_critical_exception_what(rust::Str msg);
rust::String path2_runtime_error_what(rust::Str msg);
rust::String path2_logic_error_what(rust::Str msg);
rust::String path2_hard_crash_what(rust::Str msg);
rust::String path2_invalid_argument_what(rust::Str msg);
rust::String path2_out_of_range_what(rust::Str msg);
rust::String path2_io_error_what(rust::Str msg);
rust::String path2_file_io_error_what(rust::Str msg);
rust::String path2_host_network_error_what(rust::Str msg);
rust::String path2_export_error_what(rust::Str msg);
rust::String path2_placeholder_parser_error_what(rust::Str msg);

// `SlicingError` (`:31-40`): the one-argument constructor's `what()` and default `objectId()`
// (`0`), and the two-argument constructor's `objectId()`.
rust::String path2_slicing_error_what(rust::Str msg);
uint64_t path2_slicing_error_default_object_id(rust::Str msg);
uint64_t path2_slicing_error_object_id(rust::Str msg, uint64_t object_id);

// `SlicingErrors` (`:42-49`): the inherited one-argument constructor forwards `msg` verbatim;
// the `std::vector<SlicingError>` constructor always sets the message to the literal `"Errors"`
// (`:46`) regardless of how many elements the vector holds, and `errors_` (`:48`, public) ends
// up exactly that many.
rust::String path2_slicing_errors_from_message_what(rust::Str msg);
rust::String path2_slicing_errors_from_vector_what(uint32_t count);
int64_t path2_slicing_errors_from_vector_count(uint32_t count);

// The four `catch`-set predicates (`Exception.hpp:18-27`'s inheritance graph) against a real
// `try`/`catch`, confined to this translation unit (see the `.cpp` for why). `type_index` names
// one of the fourteen concrete types in the hierarchy, 0-13 in the same order this unit's own
// `Exception` enum lists them.
uint8_t path2_catch_set_bitmask(uint8_t type_index);

} // namespace exception
} // namespace proof_units
} // namespace slic3r_ffi
