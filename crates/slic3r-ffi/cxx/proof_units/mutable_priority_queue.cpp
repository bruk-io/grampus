// Proof path 1 (R19) for the `MutablePriorityQueue.hpp` unit: both `MutablePriorityQueue` and
// `MutableSkipHeapPriorityQueue` are templates with no externally linked symbol of their own, so
// this translation unit `#include`s the vendored header directly, unmodified, and instantiates
// it the same way `crates/slic3r-mutablepriorityqueue`'s own Rust module does, compiled onto
// `crates/slic3r-ffi`'s shared `cxx_build::Build` (`-ffp-contract=off`, R17).
// `crates/slic3r-mutablepriorityqueue`'s proof calls these through
// `slic3r_proof::proof_units::mutable_priority_queue` -- the one sanctioned route a ported unit
// reaches the oracle by (`docs/PORTING.md`, "Decisions recorded on 2026-11-25").
// `MutablePriorityQueue.hpp` includes nothing beyond `<assert.h> <cstddef> <limits>
// <type_traits> <utility> <vector>`, so this translation unit needs no other vendored or
// superbuild include path.
#include "mutable_priority_queue.h"

#include "MutablePriorityQueue.hpp"

#include <cstddef>
#include <cstdint>
#include <vector>

namespace {

// The proof's one shared instantiation (mirrored exactly on the Rust side by
// `crates/slic3r-mutablepriorityqueue/src/mutable_priority_queue.rs`'s own `#[cfg(test)]`
// `ProofItem`): a POD carrying an `int64_t` sort key and a `size_t` queue index.
struct ProofItem {
    int64_t key = 0;
    size_t idx = InvalidQueueID;
};

void set_proof_item_index(ProofItem &item, size_t idx) {
    item.idx = idx;
}

bool proof_item_less(const ProofItem &a, const ProofItem &b) {
    return a.key < b.key;
}

} // namespace

namespace slic3r_ffi {
namespace proof_units {
namespace mutable_priority_queue {

rust::Vec<int64_t> push_pop_order(const rust::Vec<int64_t> &keys) {
    auto queue =
        make_mutable_priority_queue<ProofItem, false>(set_proof_item_index, proof_item_less);
    for (int64_t key : keys) {
        ProofItem item;
        item.key = key;
        queue.push(item);
    }
    rust::Vec<int64_t> out;
    while (!queue.empty()) {
        out.push_back(queue.top().key);
        queue.pop();
    }
    return out;
}

rust::Vec<int64_t> remove_then_pop_order(const rust::Vec<int64_t> &keys, size_t remove_at) {
    auto queue =
        make_mutable_priority_queue<ProofItem, false>(set_proof_item_index, proof_item_less);
    for (int64_t key : keys) {
        ProofItem item;
        item.key = key;
        queue.push(item);
    }
    queue.remove(remove_at);
    rust::Vec<int64_t> out;
    while (!queue.empty()) {
        out.push_back(queue.top().key);
        queue.pop();
    }
    return out;
}

rust::Vec<int64_t> skip_heap_push_pop_order(const rust::Vec<int64_t> &keys) {
    auto queue = make_miniheap_mutable_priority_queue<ProofItem, 32, false>(
        set_proof_item_index, proof_item_less);
    for (int64_t key : keys) {
        ProofItem item;
        item.key = key;
        queue.push(item);
    }
    rust::Vec<int64_t> out;
    while (!queue.empty()) {
        out.push_back(queue.top().key);
        queue.pop();
    }
    return out;
}

} // namespace mutable_priority_queue
} // namespace proof_units
} // namespace slic3r_ffi

// Adversarial review (task 53-foundation-int128-queue): the three functions above never
// instantiate `ResetIndexWhenRemoved = true`, so none of them observes `clear()`'s (and
// therefore the destructor's) index-setter writes -- the one behaviour
// `~MutablePriorityQueue()`/`~MutableSkipHeapPriorityQueue()` exist for. These two functions
// instantiate `true` with an `IndexSetter` that writes into a `std::vector<size_t>` the queue
// does not own (mirroring the real call shape at `ShortestPath.cpp:791`/`AStar.hpp:91`, where the
// setter reaches a field on an object outside the queue), push every key, then let the queue run
// off the end of its scope so its destructor -- not an explicit `clear()` call -- runs, and
// report what the external vector holds afterward.

namespace {

// A POD carrying a sort key and a stable "slot" (its position in the input array), distinct from
// `ProofItem` above: the index-setter here does not write into the item itself, it writes into
// an external array keyed by `slot`, which is the shape that makes the destructor's effect
// observable from outside the queue at all.
struct SlotItem {
    int64_t key = 0;
    size_t slot = 0;
};

bool slot_item_less(const SlotItem &a, const SlotItem &b) {
    return a.key < b.key;
}

} // namespace

namespace slic3r_ffi {
namespace proof_units {
namespace mutable_priority_queue {

// Pushes `keys.size()` items (slot `i` <- `keys[i]`) into a `ResetIndexWhenRemoved = true` base
// queue, drops the queue without calling `clear()` explicitly, then reports the external mark
// the destructor left behind for each slot (`InvalidQueueID` if the destructor ran the
// mark-as-removed loop, the slot's last-assigned heap index otherwise).
rust::Vec<int64_t> destructor_marks_base(const rust::Vec<int64_t> &keys) {
    std::vector<size_t> external_marks(keys.size(), InvalidQueueID);
    {
        auto index_setter = [&external_marks](SlotItem &item, size_t idx) {
            external_marks[item.slot] = idx;
        };
        auto queue = make_mutable_priority_queue<SlotItem, true>(index_setter, slot_item_less);
        for (size_t i = 0; i < keys.size(); ++i) {
            SlotItem item;
            item.key = keys[i];
            item.slot = i;
            queue.push(item);
        }
        // `queue`'s destructor runs here, at the end of this block -- not an explicit `clear()`.
    }
    rust::Vec<int64_t> out;
    for (size_t mark : external_marks) {
        out.push_back(static_cast<int64_t>(mark));
    }
    return out;
}

// Same as `destructor_marks_base` above, for the skip-heap variant.
rust::Vec<int64_t> destructor_marks_skip_heap(const rust::Vec<int64_t> &keys) {
    std::vector<size_t> external_marks(keys.size(), InvalidQueueID);
    {
        auto index_setter = [&external_marks](SlotItem &item, size_t idx) {
            external_marks[item.slot] = idx;
        };
        auto queue = make_miniheap_mutable_priority_queue<SlotItem, 32, true>(
            index_setter, slot_item_less);
        for (size_t i = 0; i < keys.size(); ++i) {
            SlotItem item;
            item.key = keys[i];
            item.slot = i;
            queue.push(item);
        }
        // `queue`'s destructor runs here, at the end of this block -- not an explicit `clear()`.
    }
    rust::Vec<int64_t> out;
    for (size_t mark : external_marks) {
        out.push_back(static_cast<int64_t>(mark));
    }
    return out;
}

} // namespace mutable_priority_queue
} // namespace proof_units
} // namespace slic3r_ffi
