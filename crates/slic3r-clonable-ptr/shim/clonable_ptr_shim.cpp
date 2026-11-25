// Proof shim for `clonable_ptr.hpp` (task 54-foundation-objectid-clonable, R19), compiled by this
// crate's own `build.rs` and linked into `crates/slic3r-clonable-ptr`'s `#[cfg(test)]` proof --
// `proof_present`'s "a `#[cfg(test)]` module inside the module file itself" branch. See that
// `build.rs` for why this crate carries its own build step instead of calling through
// `crates/slic3r-ffi`.
//
// `#include`s the vendored header verbatim and unmodified. `clonable_ptr.hpp` includes only
// `"assert.h"`, so this translation unit needs no include path beyond `src/libslic3r` itself.
//
// `clonable_ptr<T>` is a template, so it has no compiled code of its own until something
// instantiates it. The pin's one in-scope instantiation is `clonable_ptr<const ConfigOption>`
// (`Config.hpp:2348-2349`), which would drag the whole of `Config.hpp` and its dependency set
// into this translation unit for no gain: what the proof measures is the *template's* own
// behaviour -- the copy constructor's call to the pointee's virtual `clone()`, `release()`'s
// non-destroying hand-off, `reset`, `swap`, `operator bool` and the six comparison operators --
// not `ConfigOption`'s. So the element type below is proof input, exactly as an `int64_t` is
// input to task 53-foundation-int128-queue's `Int128.hpp` proof; the oracle under test is the
// vendored template's own instantiated code.
//
// `ProofPayload` is deliberately shaped like the pin's own pointee: a polymorphic class with a
// virtual `clone()` returning a fresh heap copy, and a virtual destructor, which is what
// `clonable_ptr`'s copy constructor (`:46-49`, `rhs.px->clone()`) and `destroy()` (`:128-132`,
// `delete px`) each require.
#include "clonable_ptr.hpp"

#include <cstdint>

namespace {

int g_live_payloads = 0;

class ProofPayload {
public:
    explicit ProofPayload(int64_t value) : m_value(value) { ++g_live_payloads; }
    ProofPayload(const ProofPayload &rhs) : m_value(rhs.m_value) { ++g_live_payloads; }
    virtual ~ProofPayload() { --g_live_payloads; }
    virtual ProofPayload *clone() const { return new ProofPayload(*this); }
    int64_t value() const { return m_value; }

private:
    int64_t m_value;
};

} // namespace

extern "C" {

int32_t slic3r_clonable_ptr_shim_observations(int64_t *out, int32_t capacity) {
    g_live_payloads = 0;

    // `clonable_ptr()` (`:36-39`) and `operator bool()` (`:104-107`).
    const Slic3r::clonable_ptr<ProofPayload> empty;
    const int64_t default_is_empty = static_cast<bool>(empty) ? 1 : 0;
    const int64_t default_get_is_null = (empty.get() == nullptr) ? 1 : 0;

    // `explicit clonable_ptr(T*)` (`:41-44`), `operator*` (`:110-114`) and `operator->`
    // (`:115-119`).
    Slic3r::clonable_ptr<ProofPayload> held(new ProofPayload(41));
    const int64_t held_is_non_empty = static_cast<bool>(held) ? 1 : 0;
    const int64_t held_value_through_star = (*held).value();
    const int64_t held_value_through_arrow = held->value();

    // Copy construction (`:46-49`) clones through the pointee's own virtual `clone()`: a new
    // allocation with an equal value, so the two pointers are *not* equal (the comparison
    // operators at `:145-168` compare `get()`, the addresses, never the pointees).
    const Slic3r::clonable_ptr<ProofPayload> copied(held);
    const int64_t copy_is_non_empty = static_cast<bool>(copied) ? 1 : 0;
    const int64_t copy_value_equals_source = (copied->value() == held->value()) ? 1 : 0;
    const int64_t copy_is_a_distinct_allocation = (copied.get() != held.get()) ? 1 : 0;
    const int64_t copy_compares_equal_to_source = (copied == held) ? 1 : 0;
    const int64_t copy_compares_unequal_to_source = (copied != held) ? 1 : 0;
    const int64_t live_payloads_after_copy = static_cast<int64_t>(g_live_payloads);

    // Copying an empty `clonable_ptr` clones nothing (`rhs ? ... : nullptr`).
    const Slic3r::clonable_ptr<ProofPayload> copied_empty(empty);
    const int64_t copy_of_empty_is_empty = static_cast<bool>(copied_empty) ? 1 : 0;

    // `swap` (`:90-95`) exchanges the two native pointers.
    Slic3r::clonable_ptr<ProofPayload> left(new ProofPayload(1));
    Slic3r::clonable_ptr<ProofPayload> right(new ProofPayload(2));
    left.swap(right);
    const int64_t left_value_after_swap = left->value();
    const int64_t right_value_after_swap = right->value();
    const int64_t live_payloads_after_swap = static_cast<int64_t>(g_live_payloads);

    // `reset()` (`:77-80`) destroys the pointee; `reset(T*)` (`:82-87`) destroys it and takes
    // the new one.
    left.reset();
    const int64_t reset_leaves_empty = static_cast<bool>(left) ? 1 : 0;
    const int64_t live_payloads_after_reset = static_cast<int64_t>(g_live_payloads);
    right.reset(new ProofPayload(7));
    const int64_t reset_with_takes_the_new_value = right->value();
    const int64_t live_payloads_after_reset_with = static_cast<int64_t>(g_live_payloads);

    // `release()` (`:98-101`) gives up ownership *without* destroying the object: the live count
    // is unchanged where `reset()` above decremented it, which is the whole distinction.
    Slic3r::clonable_ptr<ProofPayload> released(new ProofPayload(9));
    const int64_t live_payloads_before_release = static_cast<int64_t>(g_live_payloads);
    released.release();
    const int64_t release_leaves_empty = static_cast<bool>(released) ? 1 : 0;
    const int64_t release_does_not_destroy =
        (static_cast<int64_t>(g_live_payloads) == live_payloads_before_release) ? 1 : 0;

    // The ordering operators (`:153-168`) compare the addresses. Compared here as the *relation*
    // between two pointers taken in a known address order, never as an address value, which
    // would be meaningless across the two languages.
    ProofPayload *const low_raw = new ProofPayload(0);
    ProofPayload *const high_raw = new ProofPayload(0);
    ProofPayload *const first = (low_raw < high_raw) ? low_raw : high_raw;
    ProofPayload *const last = (low_raw < high_raw) ? high_raw : low_raw;
    const Slic3r::clonable_ptr<ProofPayload> lower(first);
    const Slic3r::clonable_ptr<ProofPayload> higher(last);
    const int64_t lower_is_less = (lower < higher) ? 1 : 0;
    const int64_t lower_is_less_or_equal = (lower <= higher) ? 1 : 0;
    const int64_t higher_is_greater = (higher > lower) ? 1 : 0;
    const int64_t higher_is_greater_or_equal = (higher >= lower) ? 1 : 0;
    const int64_t self_is_less_or_equal = (lower <= lower) ? 1 : 0;
    const int64_t self_is_strictly_less = (lower < lower) ? 1 : 0;

    const int64_t values[] = {
        default_is_empty,
        default_get_is_null,
        held_is_non_empty,
        held_value_through_star,
        held_value_through_arrow,
        copy_is_non_empty,
        copy_value_equals_source,
        copy_is_a_distinct_allocation,
        copy_compares_equal_to_source,
        copy_compares_unequal_to_source,
        live_payloads_after_copy,
        copy_of_empty_is_empty,
        left_value_after_swap,
        right_value_after_swap,
        live_payloads_after_swap,
        reset_leaves_empty,
        live_payloads_after_reset,
        reset_with_takes_the_new_value,
        live_payloads_after_reset_with,
        release_leaves_empty,
        release_does_not_destroy,
        lower_is_less,
        lower_is_less_or_equal,
        higher_is_greater,
        higher_is_greater_or_equal,
        self_is_less_or_equal,
        self_is_strictly_less,
    };
    const int32_t count = static_cast<int32_t>(sizeof(values) / sizeof(values[0]));
    if (out == nullptr || capacity < count) {
        return -count;
    }
    for (int32_t i = 0; i < count; ++i) {
        out[i] = values[i];
    }
    return count;
}

} // extern "C"
