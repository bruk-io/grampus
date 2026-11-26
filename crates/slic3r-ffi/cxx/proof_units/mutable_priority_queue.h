#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart at <crate-name>/src/proof_units/mutable_priority_queue.rs.h on the
// include path it sets up, and `cxx/proof_units/mutable_priority_queue.cpp` is the one
// translation unit that defines the functions below.
#include "slic3r-ffi/src/proof_units/mutable_priority_queue.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace mutable_priority_queue {

rust::Vec<int64_t> push_pop_order(const rust::Vec<int64_t> &keys);
rust::Vec<int64_t> remove_then_pop_order(const rust::Vec<int64_t> &keys, size_t remove_at);
rust::Vec<int64_t> skip_heap_push_pop_order(const rust::Vec<int64_t> &keys);
rust::Vec<int64_t> destructor_marks_base(const rust::Vec<int64_t> &keys);
rust::Vec<int64_t> destructor_marks_skip_heap(const rust::Vec<int64_t> &keys);

} // namespace mutable_priority_queue
} // namespace proof_units
} // namespace slic3r_ffi
