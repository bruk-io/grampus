//! Proof path 1 (R19) for the `MutablePriorityQueue.hpp` unit.
//!
//! Both `MutablePriorityQueue` and `MutableSkipHeapPriorityQueue` are templates with no
//! externally linked symbol of their own, so `cxx/proof_units/mutable_priority_queue.cpp`
//! `#include`s the vendored header directly and instantiates it fresh rather than reaching a
//! symbol in `liblibslic3r.a` -- the same shape `src/proof_units/int128.rs` uses for a
//! header-only unit. Keys and marks cross the bridge as plain `i64`s (decision 1,
//! `docs/PORTING.md`), never a libslic3r type.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::mutable_priority_queue")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/mutable_priority_queue.h");

        /// Pushes every key into a `MutablePriorityQueue<_, _, _, false>` then pops it empty,
        /// returning the keys in pop order.
        fn push_pop_order(keys: &Vec<i64>) -> Vec<i64>;
        /// Same as `push_pop_order`, but calls `remove(remove_at)` once after every key is
        /// pushed and before any pop.
        fn remove_then_pop_order(keys: &Vec<i64>, remove_at: usize) -> Vec<i64>;
        /// Same as `push_pop_order`, for `MutableSkipHeapPriorityQueue<_, _, _, 32, false>`.
        fn skip_heap_push_pop_order(keys: &Vec<i64>) -> Vec<i64>;
        /// Pushes every key into a `ResetIndexWhenRemoved = true` base queue, drops it without
        /// calling `clear()` explicitly, and returns the external index-setter mark each slot was
        /// left with by the destructor.
        fn destructor_marks_base(keys: &Vec<i64>) -> Vec<i64>;
        /// Same as `destructor_marks_base`, for the skip-heap variant.
        fn destructor_marks_skip_heap(keys: &Vec<i64>) -> Vec<i64>;
    }
}

pub use ffi::{
    destructor_marks_base, destructor_marks_skip_heap, push_pop_order, remove_then_pop_order,
    skip_heap_push_pop_order,
};
