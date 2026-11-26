#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart at <crate-name>/src/proof_units/instance_lock.rs.h on the include
// path it sets up, and `cxx/proof_units/instance_lock.cpp` is the one translation unit that
// defines the function below.
#include "slic3r-ffi/src/proof_units/instance_lock.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace instance_lock {

// A scripted sequence over `Slic3r::InstanceLock`'s public surface (the constructor, the
// destructor and `locked()` -- `open_lock_file`/`defer` are `private:` statics this shim cannot
// reach without the access-control workaround the rejection rule forbids, see
// instance_lock.cpp's own header comment), against the real file at `path`. Returns one `i64`
// (`1`/`0`) per observation, in order:
//
// 0. an empty path is a no-op: not locked.
// 1. a fresh guard on a real path locks it.
// 2. a nested guard on the *same* path, same thread, while the outer guard is still open, is
//    also considered locked (the reentrant mutex lets the same thread back in, and the shared
//    slot's file lock is already held).
// 3. the outer guard, after the nested one has dropped, is still considered locked.
// 4. once the outer guard drops too (the real `flock` releases), a fresh guard on the same path
//    locks it again.
rust::Vec<int64_t> path1_observations(rust::Str path);

} // namespace instance_lock
} // namespace proof_units
} // namespace slic3r_ffi
