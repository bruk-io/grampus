//! Port of `src/libslic3r/Execution/ExecutionSeq.hpp` (task `56-foundation-point`, plan step 20):
//! the sequential, single-threaded [`execution::Traits`] specialisation (`crates/slic3r-execution/
//! src/execution.rs`, task `55-foundation-libslic3r-execution`'s own unit). `template-heavy: yes`
//! in `port/manifest.tsv`.
//!
//! **`IsSequentialEP_`/`IsSequentialEP`/`SequentialEPOnly` (`:22-34`) are not ported as separate
//! items.** They are a second SFINAE gate, layered on top of `IsExecutionPolicy_`, that lets
//! `Traits<EP>` be specialised for *any* sequential policy at once (`ExecutionSeq` here, and
//! `std::execution::sequenced_policy` under the `PRUSASLICER_USE_EXECUTION_STD` macro this port
//! never defines -- `just oracle-build` does not pass it, so that arm is dead code at the pin).
//! Rust has no partial specialisation to mirror: [`ExecutionSeq`] below `impl`s
//! [`execution::Traits`] directly, the same move `execution.rs`'s module doc already states for
//! the outer `IsExecutionPolicy_` gate.
//!
//! **`_Mtx` (`:39`) collapses to `()`.** A C++ "lockable that never blocks" needs a type with
//! `lock()`/`unlock()` methods because `std::lock_guard` calls them unconditionally; a sequential
//! policy has nothing to serialise in the first place, so the Rust side needs no mutex type at
//! all for [`Traits::SpinningMutex`]/[`Traits::BlockingMutex`] -- `()` is both, and no call site
//! in this port's own units locks either one (the lock sites are inside concrete port modules
//! that use these policies, all out of scope for this task).
//!
//! **`loop_`'s two overloads (`:41-51`) collapse to one `for` loop.** The iterator-vs-integer
//! SFINAE split exists so `tbb::blocked_range<It>` and `tbb::blocked_range<I>` (an integer range)
//! can share one dispatch; `execution.rs`'s own [`execution::Traits`] trait (task
//! `55-foundation-libslic3r-execution`'s design choice, not this task's) already states the
//! iterator-only shape every policy implements -- a C++ integer range becomes a `Range<i64>`,
//! which already implements [`Iterator`] on the Rust side, so there is no second branch to carry
//! over here.

use crate::execution::{ExecutionPolicy, Traits};

/// `struct ExecutionSeq {};` (`:16`) plus `template<> struct
/// IsExecutionPolicy_<ExecutionSeq> : public std::true_type {};` (`:18`) -- the opt-in gate
/// `execution.rs`'s module doc explains collapses to "implements [`ExecutionPolicy`]" directly.
/// `static constexpr ExecutionSeq ex_seq = {};` (`:20`) needs no Rust counterpart: a unit struct
/// has exactly one value already, so a named constant instance would only duplicate it.
#[derive(Debug, Default, Clone, Copy)]
pub struct ExecutionSeq;

impl Traits for ExecutionSeq {
    /// `using SpinningMutex = _Mtx;` (`:54`) -- see the module doc's `_Mtx` note.
    type SpinningMutex = ();
    /// `using BlockingMutex = _Mtx;` (`:55`).
    type BlockingMutex = ();

    /// `static size_t max_concurrency(const EP &) { return 1; }` (`:82`).
    fn max_concurrency(_ep: &Self) -> usize {
        1
    }

    /// `static void for_each(const EP &, It from, It to, Fn &&fn, size_t = 1)` (`:57-65`),
    /// forwarding to `loop_` -- see the module doc for why the two `loop_` overloads collapse to
    /// one plain `for` here. `granularity` is accepted and ignored, exactly as the vendored
    /// signature's trailing `/* ignore granularity */ = 1` comment states.
    fn for_each<It, F>(_ep: &Self, iter: It, mut fn_: F, _granularity: usize)
    where
        It: Iterator,
        F: FnMut(It::Item),
    {
        for item in iter {
            fn_(item);
        }
    }

    /// `static T reduce(const EP &, I from, I to, const T &init, MergeFn &&mergefn, AccessFn
    /// &&access, size_t = 1)` (`:67-80`): accumulate `init` by folding `mergefn(acc,
    /// access(item))` over the range, in iteration order -- the same fixed, serial order
    /// `execution.rs`'s own `OneThread` test policy and `cxx/proof_units/execution.cpp`'s
    /// `ProofSerialEP` already use for the generic dispatch layer this specialises.
    fn reduce<It, M, T, A>(
        _ep: &Self,
        iter: It,
        init: T,
        mut mergefn: M,
        mut accessfn: A,
        _granularity: usize,
    ) -> T
    where
        It: Iterator,
        M: FnMut(T, T) -> T,
        A: FnMut(It::Item) -> T,
    {
        let mut acc = init;
        for item in iter {
            acc = mergefn(acc, accessfn(item));
        }
        acc
    }
}

impl ExecutionPolicy for ExecutionSeq {}

#[cfg(test)]
mod tests {
    use super::*;
    use slic3r_proof::proof_units;
    use slic3r_proof::{compare_unit, Oracle};

    #[test]
    fn max_concurrency_is_always_one() {
        assert_eq!(crate::execution::max_concurrency(&ExecutionSeq), 1);
    }

    #[test]
    fn for_each_visits_every_item_in_order() {
        let data = vec![10, 20, 30];
        let mut seen = Vec::new();
        crate::execution::for_each(&ExecutionSeq, data.iter().copied(), |x| seen.push(x), 1);
        assert_eq!(seen, vec![10, 20, 30]);
    }

    /// Proof path 2 (R19). **Finding for the regeneration pass, recorded here and in the pull
    /// request body**: this row's `pipeline stage` cell (`posContouring`) is mechanically seeded
    /// (`port/inventory`'s own rule) and no committed stage dump carries a `posContouring.dump`
    /// file at all (`port/dumps/*/` holds `posSlice` through `posSimplifyPath`, `psSkirtBrim` and
    /// `psWipeTower` -- never `posContouring`) -- checked with `ls port/dumps/*/*.dump`, not
    /// assumed. `ExecutionSeq.hpp` is header-only dispatch with no pipeline stage of its own in
    /// the first place: like `Execution.hpp` (`crates/slic3r-execution/src/execution.rs`'s own
    /// module doc, same finding), its output is whatever concrete loop body a caller supplies, and
    /// no in-scope caller of this specific policy exists yet (the callers are Phase A/B fan-out
    /// units, out of scope for this foundation task). `docs/PORTING.md`'s "Decisions recorded on
    /// 2026-11-25", decision 1, generalises the per-unit shim mechanism beyond proof paths 1 and
    /// 3 -- `cxx/proof_units/object_id.cpp`'s `path2_header_observations` and `execution.cpp`'s
    /// `path2_reduce_doubles_and_sums` already prove a `proof path: 2` row this same way -- so
    /// this test does the same: `cxx/proof_units/execution_seq.cpp` calls the *real* vendored
    /// `Slic3r::ex_seq` global (`ExecutionSeq.hpp:20`) through `Slic3r::execution::reduce`, the
    /// actual dispatch code `Traits<ExecutionSeq>::reduce` above specialises for, not a
    /// reimplementation of it.
    // PROOF: 2
    #[test]
    fn execution_seq_reduce_matches_the_oracle() {
        let data = vec![1i64, 2, 3, 4, 5];
        let actual = crate::execution::reduce(
            &ExecutionSeq,
            data.iter().copied(),
            0i64,
            |a: i64, b: i64| a + b,
            |x: i64| x * 3,
            1,
        );
        let oracle = proof_units::execution_seq::path2_reduce_triples_and_sums(&data);
        compare_unit("Execution/ExecutionSeq.hpp", Oracle::Built, actual, oracle).expect(
            "ExecutionSeq's Traits<ExecutionSeq>::reduce must dispatch identically to the \
             vendored specialisation's own serial loop",
        );
    }
}

// PORT STATUS
// source: src/libslic3r/Execution/ExecutionSeq.hpp
// confidence: medium
// tier: exact
// oracle: built
