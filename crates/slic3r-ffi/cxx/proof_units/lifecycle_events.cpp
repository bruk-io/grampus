// Proof path 2 (R19) for the `LifecycleEvents.hpp` unit, written as a genuine differential
// comparison rather than a transcription-consistency check against hand-copied literals.
//
// `port/manifest.tsv`'s row for this unit fixes its `proof path` cell to `2` (the committed
// stage dumps), but no pipeline-stage dump carries any `LifecycleEvent`-shaped output to
// differential-compare against: `fire_lifecycle_event` is never called from inside
// `WipeTower.cpp` (the manifest's `psWipeTower` cell is inventory-derived metadata about where
// the header sits relative to the pipeline, not a claim about call sites -- see
// `crates/slic3r-lifecycleevents/src/lifecycle_events.rs`'s module doc for the measured call
// sites). `docs/PORTING.md`'s "Decisions recorded on 2026-11-25" decision 4 names the oracle's
// own entry points as "the per-unit proof bridges `slic3r-proof` re-exports ... plus the
// committed stage dumps" -- not restricted to a row whose `proof path` cell says `1` or `3` --
// so this unit's `// PROOF: 2` marker (matching its own cell, verbatim, as `port/port-status`'s
// `proof_present` requires) is proven through this bridge instead: a real comparison against the
// vendored header's own `switch` statements, which is strictly stronger than, and replaces, an
// earlier version of this unit's proof that only compared its own output against a hand-
// transcribed string table (flagged by adversarial review as tautological, this task).
//
// `LifecycleEvents.hpp` is header-only with no externally linked symbol of its own (both
// functions are `inline`), so this translation unit `#include`s the vendored header directly,
// unmodified, and calls its functions fresh -- the same shape
// `cxx/proof_units/int128.cpp`/`cxx/proof_units/mutable_priority_queue.cpp` use for a header-only
// unit. The enum value crosses the bridge as a plain `int32_t` ordinal (decision 1: "a plain
// integer ... never a libslic3r type"), `static_cast` back to the enum on this side -- valid for
// every ordinal this unit's own proof passes, since both enums declare no explicit discriminant
// values (0, 1, 2, ... in declaration order).
#include "lifecycle_events.h"

#include "LifecycleEvents.hpp"

#include <cstdint>

namespace slic3r_ffi {
namespace proof_units {
namespace lifecycle_events {

rust::String lifecycle_event_to_string(int32_t event) {
    return rust::String(
        Slic3r::lifecycle_event_to_string(static_cast<Slic3r::LifecycleEvent>(event)));
}

rust::String lifecycle_evt_code_to_string(int32_t code) {
    return rust::String(
        Slic3r::lifecycle_evt_code_to_string(static_cast<Slic3r::LifecycleEvtCode>(code)));
}

} // namespace lifecycle_events
} // namespace proof_units
} // namespace slic3r_ffi
