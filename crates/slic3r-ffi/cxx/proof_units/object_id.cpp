// Proof shims for the `ObjectID.cpp`/`ObjectID.hpp` unit (R19, proof paths 1 and 2), compiled
// onto `crates/slic3r-ffi`'s shared `cxx_build::Build` by this crate's `build.rs` -- the one
// sanctioned route a ported unit reaches the oracle by (`docs/PORTING.md`, "Decisions recorded
// on 2026-11-25"). `crates/slic3r-objectid`'s proofs call these through
// `slic3r_proof::proof_units::object_id`.
//
// `Slic3r::wipe_tower_object_id()`/`wipe_tower_instance_id()` (`src/libslic3r/ObjectID.cpp`) are
// externally linked symbols in the archive `just oracle-build` produces, reached through
// `build.rs`'s derived link line (read from the CMake build tree, not enumerated). They are
// deterministic per test binary: `ObjectID.cpp`'s file-static `s_last_id` starts at 0, so the
// first `Slic3r::ObjectBase` a process constructs has id 1 and the second id 2. Every caller of
// these two therefore has to be the only test in its binary; both
// `crates/slic3r-proof/tests/harness_paths.rs` and
// `crates/slic3r-objectid/tests/object_id_proof_path_1.rs` carry that constraint at the top of
// their own files.
//
// `path2_header_observations()` is the `ObjectID.hpp` row's shim: a scripted sequence over the
// vendored header's own inline methods, returned as a vector of observations. Every observation
// is a *delta* or a relation, never an absolute id, so the sequence yields the same vector
// wherever the process-global `s_last_id`/`s_last_timestamp` counters happen to stand when it
// runs -- the proof cannot become order-dependent on what else a test binary did first. See the
// Rust side for what the committed stage-dump corpus does and does not carry for this row.
//
// `ObjectID.hpp:4-5` includes `<cereal/access.hpp>` and `<cereal/types/base_class.hpp>`, which
// the dependency superbuild's include prefix (already on this crate's shared include set)
// provides.
#include "object_id.h"

#include "ObjectID.hpp"

#include <cstddef>
#include <cstdint>

namespace {

// Gives the proof access to `ObjectBase`'s `protected` members the way the vendored tree's own
// derived types reach them (`ObjectWithTimestamp`, `CutObjectBase`, `Model.hpp`'s hierarchy):
// by inheriting, not by casting or by editing the header. The element type is proof input, the
// same way an `int64_t` is input to the `Int128.hpp` proof; the oracle under test is the
// vendored header's own inline code.
class ProofObject : public Slic3r::ObjectBase {
public:
    ProofObject() : Slic3r::ObjectBase() {}
    // `ObjectBase(int)` (ObjectID.hpp:82), the invalid-id constructor; the parameter is ignored
    // upstream too.
    explicit ProofObject(int ignored) : Slic3r::ObjectBase(ignored) {}
    using Slic3r::ObjectBase::assign_new_unique_ids_recursive;
    using Slic3r::ObjectBase::copy_id;
    using Slic3r::ObjectBase::set_invalid_id;
    using Slic3r::ObjectBase::set_new_unique_id;
};

class ProofTimestamped : public Slic3r::ObjectWithTimestamp {
public:
    ProofTimestamped() : Slic3r::ObjectWithTimestamp() {}
    using Slic3r::ObjectWithTimestamp::copy_timestamp;
    using Slic3r::ObjectWithTimestamp::touch;
};

} // namespace

namespace slic3r_ffi {
namespace proof_units {
namespace object_id {

uint64_t path1_wipe_tower_object_id() {
    return static_cast<uint64_t>(Slic3r::wipe_tower_object_id().id);
}

uint64_t path1_wipe_tower_instance_id() {
    return static_cast<uint64_t>(Slic3r::wipe_tower_instance_id().id);
}

rust::Vec<int64_t> path2_header_observations() {
    // `ObjectBase()` mints a fresh id; the next one is exactly one higher. Recorded as a delta,
    // never as an absolute id (ObjectID.hpp:100, `++s_last_id`).
    ProofObject first;
    ProofObject second;
    const int64_t consecutive_ids_differ_by =
        static_cast<int64_t>(second.id().id) - static_cast<int64_t>(first.id().id);

    // `ObjectBase(int)` (ObjectID.hpp:82) assigns the invalid id; `ObjectID::valid()`/`invalid()`
    // (`:36-37`) read it.
    ProofObject invalid(-1);
    const int64_t invalid_is_invalid = invalid.id().invalid() ? 1 : 0;
    const int64_t invalid_is_not_valid = invalid.id().valid() ? 1 : 0;
    const int64_t fresh_is_valid = first.id().valid() ? 1 : 0;

    // `set_new_unique_id()` (`:89`) mints again, so it too advances by one from the counter's
    // current head; `set_invalid_id()` (`:90`) and `copy_id()` (`:92`) are value assignments.
    ProofObject mutated;
    const int64_t before_mutation = static_cast<int64_t>(mutated.id().id);
    mutated.set_new_unique_id();
    const int64_t set_new_unique_id_advances_by =
        static_cast<int64_t>(mutated.id().id) - before_mutation;
    mutated.set_invalid_id();
    const int64_t set_invalid_id_gives_invalid = mutated.id().invalid() ? 1 : 0;
    mutated.copy_id(first);
    const int64_t copy_id_matches_source = (mutated.id() == first.id()) ? 1 : 0;

    // `assign_new_unique_ids_recursive()` (`:95`) on `ObjectBase` itself only reassigns its own
    // id, so it advances by one exactly as `set_new_unique_id()` does.
    ProofObject recursive;
    const int64_t before_recursive = static_cast<int64_t>(recursive.id().id);
    recursive.assign_new_unique_ids_recursive();
    const int64_t assign_recursive_advances_by =
        static_cast<int64_t>(recursive.id().id) - before_recursive;

    // `ObjectBase::timestamp()` (`:74`) is always `0`; `ObjectWithTimestamp::timestamp()` (`:132`)
    // overrides it, and `touch()` (`:135`) pre-increments `s_last_timestamp`.
    const int64_t object_base_timestamp = static_cast<int64_t>(first.timestamp());
    ProofTimestamped stamped;
    const int64_t timestamp_is_nonzero_before_touch = (stamped.timestamp() != 0) ? 1 : 0;
    // Two touches on the *same* object, and the gap between them -- not the gap from the
    // object's initial `1` to its first touch, which would depend on where the process-global
    // `s_last_timestamp` already stood when this ran.
    stamped.touch();
    const int64_t after_first_touch = static_cast<int64_t>(stamped.timestamp());
    stamped.touch();
    const int64_t consecutive_touches_differ_by =
        static_cast<int64_t>(stamped.timestamp()) - after_first_touch;
    ProofTimestamped other_stamped;
    const int64_t distinct_timestamps_match =
        stamped.timestamp_matches(other_stamped) ? 1 : 0;
    other_stamped.copy_timestamp(stamped);
    const int64_t copied_timestamps_match = stamped.timestamp_matches(other_stamped) ? 1 : 0;
    const int64_t id_and_timestamp_match_after_copy =
        stamped.object_id_and_timestamp_match(other_stamped) ? 1 : 0;

    // `ObjectInstanceID` (`:46-56`): `instance_id` defaults to `size_t(-1)`, and `operator<`
    // (`:52-55`) is lexicographic in (object_id, instance_id).
    Slic3r::ObjectInstanceID default_instance;
    const int64_t default_instance_id_is_size_t_minus_one =
        (default_instance.instance_id == static_cast<size_t>(-1)) ? 1 : 0;
    const Slic3r::ObjectInstanceID low{Slic3r::ObjectID(1), 1};
    const Slic3r::ObjectInstanceID high_by_object{Slic3r::ObjectID(2), 0};
    const Slic3r::ObjectInstanceID high_by_instance{Slic3r::ObjectID(1), 2};
    const int64_t instance_order_is_object_major = (low < high_by_object) ? 1 : 0;
    const int64_t instance_order_breaks_ties_on_instance = (low < high_by_instance) ? 1 : 0;

    // `CutObjectBase` (`:147-209`): the default constructor's invalid id, check sum `1` and zero
    // connectors; `operator<` (`:165`) compares the id alone.
    Slic3r::CutObjectBase cut;
    const int64_t cut_default_id_is_invalid = cut.id().invalid() ? 1 : 0;
    const int64_t cut_default_check_sum = static_cast<int64_t>(cut.check_sum());
    const int64_t cut_default_connectors_cnt = static_cast<int64_t>(cut.connectors_cnt());
    cut.increase_check_sum(7);
    cut.increase_connectors_cnt(3);
    const int64_t cut_check_sum_after_increase = static_cast<int64_t>(cut.check_sum());
    const int64_t cut_connectors_cnt_after_increase =
        static_cast<int64_t>(cut.connectors_cnt());
    const Slic3r::CutObjectBase cut_low(Slic3r::ObjectID(1), 0, 0);
    const Slic3r::CutObjectBase cut_high(Slic3r::ObjectID(2), 99, 99);
    const int64_t cut_order_is_id_only = (cut_low < cut_high) ? 1 : 0;
    const int64_t cut_equality_is_id_only =
        (cut_low == Slic3r::CutObjectBase(Slic3r::ObjectID(1), 42, 42)) ? 1 : 0;
    Slic3r::CutObjectBase cut_copied;
    cut_copied.copy(cut_high);
    const int64_t cut_copy_is_equal = cut_copied.is_equal(cut_high) ? 1 : 0;
    cut_copied.invalidate();
    const int64_t cut_invalidate_check_sum = static_cast<int64_t>(cut_copied.check_sum());

    const int64_t values[] = {
        consecutive_ids_differ_by,
        invalid_is_invalid,
        invalid_is_not_valid,
        fresh_is_valid,
        set_new_unique_id_advances_by,
        set_invalid_id_gives_invalid,
        copy_id_matches_source,
        assign_recursive_advances_by,
        object_base_timestamp,
        timestamp_is_nonzero_before_touch,
        consecutive_touches_differ_by,
        distinct_timestamps_match,
        copied_timestamps_match,
        id_and_timestamp_match_after_copy,
        default_instance_id_is_size_t_minus_one,
        instance_order_is_object_major,
        instance_order_breaks_ties_on_instance,
        cut_default_id_is_invalid,
        cut_default_check_sum,
        cut_default_connectors_cnt,
        cut_check_sum_after_increase,
        cut_connectors_cnt_after_increase,
        cut_order_is_id_only,
        cut_equality_is_id_only,
        cut_copy_is_equal,
        cut_invalidate_check_sum,
    };
    rust::Vec<int64_t> out;
    for (const int64_t value : values) {
        out.push_back(value);
    }
    return out;
}

} // namespace object_id
} // namespace proof_units
} // namespace slic3r_ffi
