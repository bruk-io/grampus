//! Port of `clonable_ptr.hpp` (`vendor/OrcaSlicer/src/libslic3r/clonable_ptr.hpp`): a
//! `unique_ptr`-like smart pointer whose copy constructor clones through the pointee's own
//! `clone()` method (`:47`) rather than sharing it. The pin's only in-scope instantiation is
//! `clonable_ptr<const ConfigOption>` (`Config.hpp:2348-2349`).
//!
//! `px` (`:141`) is `port/ownership.tsv`'s one `owned` row for this header; its `rust type` cell
//! is `Box<T>`, taken verbatim. It is wrapped in `Option` here rather than stored bare, because
//! -- unlike a plain owned field -- this type's whole purpose is to be nullable: the default
//! constructor (`:36-39`), `release()` (`:98-101`) and `operator bool()` (`:104-107`) all depend
//! on a null state a bare `Box<T>` cannot represent. See this task's pull request body for this
//! decision.

// PORT NOTE: clonable_ptr.hpp:134-138 -- the private `release() const noexcept { px = nullptr;
// }` overload (distinguished from the public `release()` at `:98` only by constness) assigns to
// `px`, a non-`mutable` member, inside a `const` method; that cannot compile for any
// instantiation that calls it. It is never called anywhere in the vendored tree (grep across
// `vendor/OrcaSlicer/`), and a template member function is only instantiated when used, so it
// survives as dead code rather than a build break. It is not ported.

/// A type a [`ClonablePtr`] can hold: the Rust analogue of the virtual `clone()` method
/// `clonable_ptr<T>`'s copy constructor calls on `rhs.px` (`:46-49`, `rhs ? rhs.px->clone() :
/// nullptr`). Blanket-implemented for every [`Clone`] type below, so an ordinary value type needs
/// nothing extra; a future `dyn`-dispatched hierarchy (the idiom map's virtual-dispatch rule)
/// implements it by hand instead of relying on the blanket impl.
///
/// **Not dyn-compatible as a supertrait.** `clone_boxed`'s return type references `Self`, so
/// `trait SomeConfigOptionTrait: ClonableInner` cannot itself be made into a `dyn
/// SomeConfigOptionTrait` (`E0038`). A `dyn`-dispatched hierarchy instead gives `dyn
/// SomeConfigOptionTrait` a standalone `impl ClonableInner for dyn SomeConfigOptionTrait`
/// (its `clone_boxed` dispatching through whatever boxed-clone method the hierarchy's own trait
/// declares), which does compile and does not collide with the blanket impl below -- recorded
/// here so `slic3r-config` (`foundation-08`, not yet built) does not reach for the supertrait
/// form first.
pub trait ClonableInner {
    fn clone_boxed(&self) -> Box<Self>;
}

impl<T: Clone> ClonableInner for T {
    fn clone_boxed(&self) -> Box<Self> {
        Box::new(self.clone())
    }
}

/// `Slic3r::clonable_ptr<T>` (`clonable_ptr.hpp:28-142`).
pub struct ClonablePtr<T: ?Sized> {
    px: Option<Box<T>>,
}

impl<T: ?Sized> ClonablePtr<T> {
    /// `clonable_ptr()` (`:36-39`): an empty pointer.
    pub fn new() -> Self {
        ClonablePtr { px: None }
    }

    /// `explicit clonable_ptr(T* p)` (`:41-44`).
    pub fn from_box(p: Box<T>) -> Self {
        ClonablePtr { px: Some(p) }
    }

    /// `operator bool()` (`:104-107`): `nullptr != px`.
    pub fn is_some(&self) -> bool {
        self.px.is_some()
    }

    /// `get()` (`:120-124`): "no assert, can return nullptr" -- already safe/nullable in the
    /// original, unlike `operator*`/`operator->` below.
    pub fn get(&self) -> Option<&T> {
        self.px.as_deref()
    }

    /// `reset()` (`:77-80`): `destroy()`, i.e. `delete px; px = nullptr;` -- dropping the `Box`
    /// frees the pointee, matching the `delete`.
    pub fn reset(&mut self) {
        self.px = None;
    }

    /// `reset(T* p)` (`:82-87`).
    pub fn reset_with(&mut self, p: Box<T>) {
        if let Some(existing) = &self.px {
            // The vendored assert at clonable_ptr.hpp:84 is `assert((nullptr == p) || (px !=
            // p)); // auto-reset not allowed` -- not a numeric-constant transcription, so no
            // `// PORT SRC:` marker applies here. Unlike the vendored raw pointer `p`, this
            // method's own `p: Box<T>` is taken by value, so it cannot already alias `existing`
            // (two distinct `Box<T>`s can never point at the same allocation in safe Rust): the
            // check below can never fail. It is transcribed anyway, as a faithful but inert
            // record of the vendored precondition, not because it can catch anything here.
            debug_assert!(
                !std::ptr::eq(existing.as_ref(), p.as_ref()),
                "clonable_ptr: auto-reset not allowed"
            );
        }
        self.px = Some(p);
    }

    /// `swap(clonable_ptr& rhs)` (`:90-95`).
    pub fn swap(&mut self, rhs: &mut Self) {
        std::mem::swap(&mut self.px, &mut rhs.px);
    }

    /// `operator*()`/`operator->()` (`:110-119`): both `assert(nullptr != px)` (compiled out in
    /// release -- `### Error sets`), then dereference unconditionally, so calling either on an
    /// empty `clonable_ptr` is undefined behaviour in a release build of the oracle. Rust's safe
    /// `Deref` trait carries a stronger promise than a C++ operator ever did: any safe caller may
    /// invoke a safe trait method with no `unsafe` anywhere at the call site, so putting this
    /// behind `Deref` would place that same UB behind a safe interface -- not a faithful
    /// rendering of the original's own precondition, which a C++ caller could at least in
    /// principle be held to by a `static_assert`-adjacent review, but a new, Rust-specific
    /// soundness hole. This crate keeps the method `unsafe fn` instead: the caller's `unsafe`
    /// block is where the precondition the vendored `assert` states (and release mode drops)
    /// now lives, the same place C++'s own lack of a safe/unsafe split put it by default.
    ///
    /// # Safety
    /// `self` must hold a value (`self.is_some()`). Calling this on an empty `ClonablePtr` is
    /// undefined behaviour, exactly as calling `operator*`/`operator->` on a null `clonable_ptr`
    /// is in a release build of the oracle, where its own `assert` is compiled out.
    // SAFETY: the `unsafe` on this function's own signature carries the precondition stated in
    // its doc comment above -- `self` holds a value -- onto its caller, the same precondition
    // the vendored `assert(nullptr != px)` states and a release build drops.
    pub unsafe fn deref_unchecked(&self) -> &T {
        debug_assert!(self.px.is_some(), "clonable_ptr: dereferenced while empty");
        // SAFETY: forwarded verbatim from this function's own safety contract above.
        unsafe { self.px.as_deref().unwrap_unchecked() }
    }

    /// `release()` (`:98-101`): "release the ownership of the px pointer without destroying the
    /// object!" -- in C++ this is a hand-off, not a leak: the caller already holds the raw
    /// pointer from a prior `get()`/`operator T*()` and now owns it outright. `get()` here
    /// returns `Option<&T>`, a borrow, so there is no recipient for the ownership this method
    /// gives up; `mem::forget` is the closest faithful rendering (setting `px` to `None` without
    /// it would free the box, exactly what the original method exists to avoid), but the result
    /// is an unrecoverable leak rather than a hand-off -- a divergence with no Rust counterpart
    /// at the pin's own call sites (`release()` is never called anywhere in the vendored tree).
    pub fn release(&mut self) {
        if let Some(b) = self.px.take() {
            std::mem::forget(b);
        }
    }
}

impl<T: ?Sized> Default for ClonablePtr<T> {
    fn default() -> Self {
        ClonablePtr::new()
    }
}

impl<T: ?Sized + ClonableInner> Clone for ClonablePtr<T> {
    /// `clonable_ptr(const clonable_ptr& rhs)` (`:46-49`): `rhs ? rhs.px->clone() : nullptr`.
    fn clone(&self) -> Self {
        ClonablePtr {
            px: self.px.as_deref().map(ClonableInner::clone_boxed),
        }
    }
}

/// `operator==`/`operator!=`/`operator<=`/`operator<`/`operator>=`/`operator>` (`:145-168`):
/// every one compares `l.get()`/`r.get()` -- the raw pointer *addresses*, never the pointees --
/// so two `ClonablePtr`s holding separately-allocated but equal-by-value pointees are unequal
/// here, matching the vendored pointer comparison exactly rather than a value comparison.
/// `std::ptr::addr_eq` (address-only), not `std::ptr::eq` (address **and** vtable/length
/// metadata for a `?Sized` `T`), because the vendored comparison is of `T*`, a thin pointer with
/// no metadata to begin with; two `ClonablePtr<dyn Trait>`s reaching one object through
/// different vtables compare equal here, as the pin's own `T*` comparison would.
impl<T: ?Sized> PartialEq for ClonablePtr<T> {
    fn eq(&self, other: &Self) -> bool {
        match (&self.px, &other.px) {
            (None, None) => true,
            (Some(a), Some(b)) => std::ptr::addr_eq(a.as_ref(), b.as_ref()),
            _ => false,
        }
    }
}
impl<T: ?Sized> Eq for ClonablePtr<T> {}

/// The four ordering operators (`<=`,`<`,`>=`,`>`) compare pointer addresses too, which is only
/// well-ordered for a `Sized` pointee (a fat pointer's address ordering is not meaningful); the
/// pin's one instantiation, `clonable_ptr<const ConfigOption>`, is `Sized`.
//
// FIDELITY(port): clonable_ptr.hpp:154 (`operator<=`) is `port/reports/order-comparators.md`'s
// `non_strict` row for this header, whose prescribed port's choice is "the strict form of the
// same relation". This `Ord` impl, plus the derived `PartialOrd` default methods above it in
// terms of `cmp`, is exactly that: one total order over the pointer address, with `<=`/`>=`
// given by `PartialOrd`'s own default methods rather than a separately transcribed
// `operator<=` body.
impl<T: Sized> PartialOrd for ClonablePtr<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<T: Sized> Ord for ClonablePtr<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let addr = |px: &Option<Box<T>>| px.as_deref().map(|p| p as *const T as usize);
        addr(&self.px).cmp(&addr(&other.px))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slic3r_proof::dumps;
    use std::sync::atomic::{AtomicI64, Ordering};

    // The oracle: `shim/clonable_ptr_shim.cpp`'s scripted sequence over the vendored
    // `clonable_ptr<T>` template, instantiated there over a polymorphic pointee and compiled from
    // the pin by this crate's `build.rs` -- see that file for why this crate carries its own build
    // step. Nothing here compares an address *value* across the two languages, which would be
    // meaningless; the comparison operators are compared as the relations they induce between two
    // pointers taken in a known address order.
    extern "C" {
        fn slic3r_clonable_ptr_shim_observations(out: *mut i64, capacity: i32) -> i32;
    }

    /// Live-pointee count, the Rust counterpart of the shim's own `g_live_payloads`. It is what
    /// makes the difference between `reset()` (which frees) and `release()` (which does not,
    /// `clonable_ptr.hpp:97-101`) observable at all rather than asserted from the source.
    static LIVE_PAYLOADS: AtomicI64 = AtomicI64::new(0);

    /// The proof's element type: `ProofPayload` is input to the proof, the same way an `i64` is
    /// input to task `53-foundation-int128-queue`'s `Int128.hpp` proof. The code under test is
    /// [`ClonablePtr`] itself on this side and the vendored template on the other. It mirrors the
    /// shim's own pointee -- a value plus a clone that allocates afresh -- because the pin's one
    /// instantiation, `clonable_ptr<const ConfigOption>`, is polymorphic with a virtual `clone()`.
    #[derive(Debug)]
    struct ProofPayload {
        value: i64,
    }

    impl ProofPayload {
        fn new(value: i64) -> Self {
            LIVE_PAYLOADS.fetch_add(1, Ordering::SeqCst);
            ProofPayload { value }
        }
    }

    impl Clone for ProofPayload {
        fn clone(&self) -> Self {
            ProofPayload::new(self.value)
        }
    }

    impl Drop for ProofPayload {
        fn drop(&mut self) {
            LIVE_PAYLOADS.fetch_sub(1, Ordering::SeqCst);
        }
    }

    fn live_payloads() -> i64 {
        LIVE_PAYLOADS.load(Ordering::SeqCst)
    }

    /// Every observation `slic3r_clonable_ptr_shim_observations` records, in its own order, named
    /// so a failure names the vendored behaviour that diverged rather than an index.
    const OBSERVATION_NAMES: &[&str] = &[
        "a default-constructed clonable_ptr is non-empty (clonable_ptr.hpp:36-39, :104-107)",
        "a default-constructed clonable_ptr's get() is null (clonable_ptr.hpp:120-124)",
        "clonable_ptr(T*) is non-empty (clonable_ptr.hpp:41-44)",
        "operator*'s value (clonable_ptr.hpp:110-114)",
        "operator->'s value (clonable_ptr.hpp:115-119)",
        "a copy is non-empty (clonable_ptr.hpp:46-49)",
        "a copy's value equals the source's (clonable_ptr.hpp:46-49)",
        "a copy is a distinct allocation (clonable_ptr.hpp:46-49)",
        "a copy compares equal to its source (clonable_ptr.hpp:145-148)",
        "a copy compares unequal to its source (clonable_ptr.hpp:149-152)",
        "live pointees after one copy (clonable_ptr.hpp:46-49)",
        "a copy of an empty clonable_ptr is non-empty (clonable_ptr.hpp:46-49)",
        "the left value after swap (clonable_ptr.hpp:90-95)",
        "the right value after swap (clonable_ptr.hpp:90-95)",
        "live pointees after swap (clonable_ptr.hpp:90-95)",
        "reset() leaves a non-empty clonable_ptr (clonable_ptr.hpp:77-80)",
        "live pointees after reset() (clonable_ptr.hpp:128-132)",
        "reset(T*) takes the new value (clonable_ptr.hpp:82-87)",
        "live pointees after reset(T*) (clonable_ptr.hpp:82-87)",
        "release() leaves a non-empty clonable_ptr (clonable_ptr.hpp:98-101)",
        "release() leaves the pointee alive (clonable_ptr.hpp:97-101)",
        "the lower pointer is < the higher (clonable_ptr.hpp:157-160)",
        "the lower pointer is <= the higher (clonable_ptr.hpp:153-156)",
        "the higher pointer is > the lower (clonable_ptr.hpp:165-168)",
        "the higher pointer is >= the lower (clonable_ptr.hpp:161-164)",
        "a pointer is <= itself (clonable_ptr.hpp:153-156)",
        "a pointer is < itself (clonable_ptr.hpp:157-160)",
    ];

    /// The same scripted sequence the shim runs, against this module's [`ClonablePtr`] instead of
    /// the vendored template, in the same order and recorded as the same values and relations.
    fn ported_observations() -> Vec<i64> {
        LIVE_PAYLOADS.store(0, Ordering::SeqCst);

        let empty: ClonablePtr<ProofPayload> = ClonablePtr::new();
        let default_is_empty = i64::from(empty.is_some());
        let default_get_is_null = i64::from(empty.get().is_none());

        let held = ClonablePtr::from_box(Box::new(ProofPayload::new(41)));
        let held_is_non_empty = i64::from(held.is_some());
        // SAFETY: `held` was just constructed from a `Box`, so it holds a value --
        // `deref_unchecked`'s one precondition. This read stands in for the vendored `operator*`
        // (`:110-114`), which the module renders as that single `unsafe fn`.
        let held_value_through_star = unsafe { held.deref_unchecked() }.value;
        // SAFETY: as immediately above -- `held` still holds the same value. This read stands in
        // for the vendored `operator->` (`:115-119`), the same `unsafe fn` on this side.
        let held_value_through_arrow = unsafe { held.deref_unchecked() }.value;

        let copied = held.clone();
        let copy_is_non_empty = i64::from(copied.is_some());
        let copy_value_equals_source =
            i64::from(copied.get().map(|p| p.value) == held.get().map(|p| p.value));
        let copy_is_a_distinct_allocation = i64::from(!std::ptr::addr_eq(
            copied.get().expect("the copy holds a value") as *const ProofPayload,
            held.get().expect("the source holds a value") as *const ProofPayload,
        ));
        let copy_compares_equal_to_source = i64::from(copied == held);
        let copy_compares_unequal_to_source = i64::from(copied != held);
        let live_payloads_after_copy = live_payloads();

        let copied_empty = empty.clone();
        let copy_of_empty_is_empty = i64::from(copied_empty.is_some());

        let mut left = ClonablePtr::from_box(Box::new(ProofPayload::new(1)));
        let mut right = ClonablePtr::from_box(Box::new(ProofPayload::new(2)));
        left.swap(&mut right);
        let left_value_after_swap = left.get().expect("left holds a value").value;
        let right_value_after_swap = right.get().expect("right holds a value").value;
        let live_payloads_after_swap = live_payloads();

        left.reset();
        let reset_leaves_empty = i64::from(left.is_some());
        let live_payloads_after_reset = live_payloads();
        right.reset_with(Box::new(ProofPayload::new(7)));
        let reset_with_takes_the_new_value = right.get().expect("right holds a value").value;
        let live_payloads_after_reset_with = live_payloads();

        let mut released = ClonablePtr::from_box(Box::new(ProofPayload::new(9)));
        let live_payloads_before_release = live_payloads();
        released.release();
        let release_leaves_empty = i64::from(released.is_some());
        let release_does_not_destroy = i64::from(live_payloads() == live_payloads_before_release);

        // Two allocations ordered by their own addresses, exactly as the shim does it: the
        // relation is what is compared, never the address.
        let one = Box::new(ProofPayload::new(0));
        let two = Box::new(ProofPayload::new(0));
        let (first, last) =
            if (one.as_ref() as *const ProofPayload) < (two.as_ref() as *const ProofPayload) {
                (one, two)
            } else {
                (two, one)
            };
        let lower = ClonablePtr::from_box(first);
        let higher = ClonablePtr::from_box(last);
        let lower_is_less = i64::from(lower < higher);
        let lower_is_less_or_equal = i64::from(lower <= higher);
        let higher_is_greater = i64::from(higher > lower);
        let higher_is_greater_or_equal = i64::from(higher >= lower);
        let self_is_less_or_equal = i64::from(lower <= lower);
        let self_is_strictly_less = i64::from(lower < lower);

        vec![
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
        ]
    }

    /// Proof path 2 (R19) for the `clonable_ptr.hpp` manifest row.
    ///
    /// **The declared mechanism, and what the corpus does not carry.** This row's `proof path`
    /// cell is `2`, the committed stage dumps, seeded from its non-`none` pipeline stage
    /// (`posSlice`) by `port/inventory`'s own rule. That dump is read below -- a real committed
    /// dump, whose provenance header `dumps::read_dump_default` checks against the pin and
    /// refuses if it disagrees -- so the declared mechanism genuinely runs. But the corpus
    /// carries nothing a `clonable_ptr` proof could compare against: the template's one in-scope
    /// instantiation is `clonable_ptr<const ConfigOption>` (`Config.hpp:2348-2349`), and no seam
    /// of the dump format writes a `ConfigOption` default value at all -- every body field is
    /// geometry and scalars (`oracle/stage-dump/README.md`'s "Dump format"). That is a finding
    /// for whoever regenerates `port/inventory`'s seeding rule, recorded in this task's pull
    /// request body, and not something a tolerance or a stand-in may paper over on this branch.
    ///
    /// **What is proven instead.** The vendored template's own instantiated code, compiled from
    /// the pin and called through this crate's shim, against this module's rendering of it -- a
    /// differential comparison, which is *stronger* than the declared mechanism rather than a
    /// substitute for it, and exact with no tolerance as this row's `exact` tier requires. Never
    /// read this as the row's mechanism being proof path 1.
    // PROOF: 2
    #[test]
    fn clonable_ptr_semantics_match_the_oracle() {
        let dump = dumps::read_dump_default("20mm_cube-base-fff", "posSlice")
            .expect("port/dumps/20mm_cube-base-fff/posSlice.dump must read at the pinned commit");
        assert_eq!(
            dump.seam, "posSlice",
            "the dump read is this row's own pipeline stage"
        );

        let ported = ported_observations();
        let mut oracle = vec![i64::MIN; ported.len()];
        // SAFETY: `oracle` is a live, exclusively borrowed `Vec<i64>` of exactly `oracle.len()`
        // elements, and that same length is passed as the capacity; the shim writes `count`
        // elements only when `capacity >= count` and otherwise writes nothing and returns
        // `-count`, which the assertion below rejects.
        let count = unsafe {
            slic3r_clonable_ptr_shim_observations(oracle.as_mut_ptr(), oracle.len() as i32)
        };
        assert_eq!(
            count,
            ported.len() as i32,
            "the oracle records {count} observation(s) where this test scripts {}",
            ported.len()
        );
        assert_eq!(
            OBSERVATION_NAMES.len(),
            ported.len(),
            "every observation must be named, so a failure names the behaviour that diverged"
        );

        for (index, name) in OBSERVATION_NAMES.iter().enumerate() {
            assert_eq!(ported[index], oracle[index], "{name}");
        }
    }
}

// PORT STATUS
// source: src/libslic3r/clonable_ptr.hpp
// confidence: medium
// tier: exact
// oracle: built
