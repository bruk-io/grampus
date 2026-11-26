//! Port of `src/libslic3r/MTUtils.hpp` (task `56-foundation-point`, plan step 20): a grab-bag of
//! thread-safety and numeric-sequence helpers. `template-heavy: yes` in `port/manifest.tsv`.
//!
//! **`SpinMutex` (`:20-31`) is not ported as a lockable type.** It exists upstream only so
//! [`CachedObject`]'s `std::lock_guard` has something to acquire; this port's own
//! [`CachedObject::get`]/[`CachedObject::invalidate`] below take `&mut self` instead, which the
//! borrow checker already turns into the same "only one caller touches the cache at a time"
//! guarantee a spin lock gives at runtime -- a real Rust property (exclusive access is checked at
//! compile time, not spun for at runtime), not a stand-in for the original's locking. No in-scope
//! caller of this task's own units needs a *shared* (`Arc`-style) cached object, so there is
//! nothing here that still needs runtime locking the way a C++ `const T&`-returning cache,
//! accessed from several threads, does.
//!
//! **`all_of` (`:79-86`) is not ported as a standalone item.** `std::all_of` over a container,
//! applying `static_cast<bool>` to each element, *is* Rust's own [`Iterator::all`] with
//! `bool::from`/truthiness already built in -- there is no behaviour left for a wrapper function
//! to add, so an in-scope caller writes `container.iter().all(|v| ...)` directly rather than
//! calling a `slic3r_mtutils::all_of` that would only forward to it.
//!
//! **`linspace_vector`/`linspace_array`/`grid` (`:91-139`) are ported as plain functions over
//! `f64`**, the one in-scope floating type every real call site this header's own callers use
//! (`docs/PORTING.md`'s "Float widths" rule: a field or call site is ported at the width its own
//! vendored declaration uses, and nothing here names a narrower one). Each still computes
//! `start + i * stride` **per element**, exactly as the vendored `std::generate` lambda does --
//! never as a running accumulator (`cur += stride`), which drifts from the multiplicative form by
//! float rounding over enough elements and would be a silent divergence from the oracle this
//! `exact`-tier row is held to.
//!
//! **`linspace_array<N, T>` (`:108-120`) collapses into [`linspace_vector`].** A C++
//! `std::array<T, N>` with `N` fixed at compile time and a `std::vec<T>` with a runtime length
//! compute the identical sequence; the only difference is storage, and Rust's `[T; N]` is not
//! needed by any in-scope caller of this task's own units (none exists yet -- this is foundation
//! design, read before any fan-out unit calls it). `linspace_array` is therefore not ported as a
//! second function: a future caller that specifically needs a fixed-size array takes
//! [`linspace_vector`]'s `Vec<f64>` and converts, or this module gains a second entry point then,
//! named in that caller's own pull request.

/// `template<class T> class CachedObject` (`:34-77`): a lazily refreshed cache, invalidated by
/// [`CachedObject::invalidate`] and refreshed on the next [`CachedObject::get`]. The vendored
/// `Setter` (`std::function<void(T&)>`, `:38`) is a boxed closure here, the same shape every
/// stored-closure field in this port takes.
// PORT NOTE: src/libslic3r/MTUtils.hpp:20-31 -- `SpinMutex` is not ported as a lockable type;
// see the module doc's own `SpinMutex` paragraph for why `&mut self` already gives this struct
// the same "one caller at a time" guarantee a spin lock gives at runtime.
pub struct CachedObject<T> {
    obj: T,
    valid: bool,
    setter: Box<dyn FnMut(&mut T)>,
}

impl<T> CachedObject<T> {
    /// `CachedObject(Setter &&fn, Args &&... args)` (`:52-54`): the vendored constructor
    /// forwards `args` into `T`'s own constructor and starts invalid, so the very first
    /// [`CachedObject::get`] runs `fn` before returning. This port takes the already-constructed
    /// initial value directly rather than forwarding constructor arguments -- Rust has no
    /// variadic perfect forwarding, and every in-scope caller of this task's own units
    /// constructs `T` the same way it would pass `args` upstream.
    pub fn new(initial: T, setter: impl FnMut(&mut T) + 'static) -> Self {
        CachedObject {
            obj: initial,
            valid: false,
            setter: Box::new(setter),
        }
    }

    /// `void invalidate(Fn &&fn)` (`:60-65`): run `fn` (typically a mutation of whatever external
    /// state the setter reads), then mark the cache invalid so the next [`CachedObject::get`]
    /// refreshes it. The vendored signature locks `m_lck` for the duration; this port's `&mut
    /// self` already excludes a concurrent [`CachedObject::get`]/[`CachedObject::invalidate`] on
    /// the same object at compile time -- see the module doc's `SpinMutex` note for why that is
    /// the faithful Rust shape here, not a workaround.
    pub fn invalidate(&mut self, fn_: impl FnOnce()) {
        fn_();
        self.valid = false;
    }

    /// `const T &get()` (`:68-76`): refresh through the setter if invalid, then return the
    /// up-to-date value. The vendored signature returns `const T&`, aliasing the cache's own
    /// storage; this port returns `&T` for the same reason, borrowed from `&mut self` for exactly
    /// as long as the setter needs to run.
    pub fn get(&mut self) -> &T {
        if !self.valid {
            (self.setter)(&mut self.obj);
            self.valid = true;
        }
        &self.obj
    }
}

/// `template<class T, class I, class = IntegerOnly<I>> inline std::vector<T>
/// linspace_vector(const ArithmeticOnly<T> &start, const T &stop, const I &n)` (`:92-106`): `n`
/// equally spaced values starting at `start` (inclusive), each computed as `start + i * stride`
/// with `stride = (stop - start) / n` -- see the module doc for why this is ported over `f64`
/// and why the per-element multiplicative form is load-bearing.
pub fn linspace_vector(start: f64, stop: f64, n: usize) -> Vec<f64> {
    let stride = (stop - start) / (n as f64);
    (0..n).map(|i| start + (i as f64) * stride).collect()
}
// TODO(port): src/libslic3r/MTUtils.hpp:108-120 -- `linspace_array<N, T>`; no in-scope caller
// of this task's own units needs the fixed-size-array form (see the module doc's own
// `linspace_array` paragraph). Not ported as `all_of` and `SpinMutex` are (a decision this
// module's own code has no behaviour left to replace), but genuinely deferred for the next
// caller that needs it, which is why this is a `// TODO(port):`, not a `// PORT NOTE:`.
// PORT NOTE: src/libslic3r/MTUtils.hpp:79-86 -- `all_of` is not ported as a standalone item; see
// the module doc's own `all_of` paragraph for why `Iterator::all` already is it.

/// `template<class T> inline std::vector<ArithmeticOnly<T>> grid(const T &start, const T &stop,
/// const T &stride)` (`:126-139`): equally spaced values from `start` (inclusive) up to the
/// closest multiple of `stride` at or below `stop`, `ceil((stop - start) / stride)` of them, each
/// computed the same per-element way as [`linspace_vector`].
pub fn grid(start: f64, stop: f64, stride: f64) -> Vec<f64> {
    let count = ((stop - start) / stride).ceil();
    // `size_t(std::ceil(...))` (`:131`): the vendored cast truncates toward zero, and `ceil`
    // already returns a non-negative integer-valued float for every in-scope call shape (`stop
    // >= start`, `stride > 0`), so the two agree; a negative `count` here is a logic error in the
    // caller's own arguments, not a case this function silently repairs.
    let n = count as usize;
    (0..n).map(|i| start + (i as f64) * stride).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use slic3r_proof::{compare_unit, Oracle};

    #[test]
    fn linspace_vector_starts_inclusive_and_steps_by_stride() {
        let values = linspace_vector(0.0, 10.0, 5);
        assert_eq!(values, vec![0.0, 2.0, 4.0, 6.0, 8.0]);
    }

    #[test]
    fn grid_covers_up_to_the_closest_multiple_at_or_below_stop() {
        let values = grid(0.0, 10.0, 3.0);
        assert_eq!(values, vec![0.0, 3.0, 6.0, 9.0]);
    }

    #[test]
    fn cached_object_refreshes_only_when_invalid() {
        // `Rc<Cell<i32>>` (shared with the closure, which `CachedObject` boxes and owns) is a
        // real, observable count of how many times the setter itself actually ran -- not a
        // count the test increments by hand regardless of what `CachedObject` does, which an
        // earlier version of this test did and which would have passed even with a
        // `get`/`invalidate` that never called the setter at all.
        let refresh_count = std::rc::Rc::new(std::cell::Cell::new(0));
        let counter = refresh_count.clone();
        let mut cache = CachedObject::new(0i32, move |obj: &mut i32| {
            counter.set(counter.get() + 1);
            *obj += 10;
        });
        assert_eq!(refresh_count.get(), 0, "unrefreshed until the first get()");
        assert_eq!(*cache.get(), 10, "first get() refreshes once, from 0 to 10");
        assert_eq!(refresh_count.get(), 1);
        assert_eq!(
            *cache.get(),
            10,
            "a second get() with no invalidation must not re-run the setter"
        );
        assert_eq!(refresh_count.get(), 1);
        cache.invalidate(|| {});
        assert_eq!(
            refresh_count.get(),
            1,
            "invalidate() itself does not refresh"
        );
        assert_eq!(
            *cache.get(),
            20,
            "the next get() after invalidate() refreshes again, 10 to 20"
        );
        assert_eq!(refresh_count.get(), 2);
    }

    /// Proof path 2 (R19). **Finding for the regeneration pass, recorded here and in the pull
    /// request body**: this row's `pipeline stage` cell (`posPrepareInfill`) is mechanically
    /// seeded, and no dump field under `port/dumps/*/posPrepareInfill.dump` corresponds to
    /// `MTUtils.hpp`'s own output -- that dump's fields are `PrintObject`/`Layer`/`LayerRegion`
    /// geometry (`oracle/stage-dump/README.md`'s field list), while `MTUtils.hpp` is a generic,
    /// data-independent numeric-sequence header with no pipeline stage of its own. Checked, not
    /// assumed: `grep -c 'field=' port/dumps/*/posPrepareInfill.dump` finds none of this header's
    /// three ported functions among the named fields. Following `execution.rs`'s and
    /// `execution_seq.rs`'s identical finding, this proof instead drives the *real* vendored
    /// `Slic3r::grid` function directly through `cxx/proof_units/mt_utils.cpp`
    /// (`docs/PORTING.md`'s decision 1, generalised to a `proof path: 2` header-only row).
    // PROOF: 2
    #[test]
    fn mt_utils_grid_matches_the_oracle() {
        let actual = grid(1.0, 10.0, 2.5);
        let oracle = slic3r_proof::proof_units::mt_utils::path2_grid(1.0, 10.0, 2.5);
        compare_unit("MTUtils.hpp", Oracle::Built, actual, oracle)
            .expect("grid must match the vendored std::generate loop exactly, element by element");
    }
}

// PORT STATUS
// source: src/libslic3r/MTUtils.hpp
// confidence: medium
// tier: exact
// oracle: built
