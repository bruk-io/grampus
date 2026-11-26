//! Port of `src/libslic3r/Execution/ExecutionTBB.hpp` (task `56-foundation-point`, plan step 20):
//! the Intel-TBB-backed [`execution::Traits`] specialisation (`crates/slic3r-execution/
//! src/execution.rs`, task `55-foundation-libslic3r-execution`'s own unit). `template-heavy: yes`
//! in `port/manifest.tsv`.
//!
//! **`tbb::parallel_for`/`tbb::parallel_reduce` are not reimplemented.** `docs/PORTING.md`'s idiom
//! map is unconditional here: "the port sums in a fixed order and is serial... This is the
//! unconditional default at every such site; parallelising a reduction requires a committed audit
//! that establishes the site's order-independence, and no such audit exists today... so no port
//! task may parallelise a reduction yet." [`ExecutionTbb::for_each`]/[`ExecutionTbb::reduce`]
//! below are therefore a single serial `for` loop over the whole range, in iteration order,
//! exactly like [`crate::execution_seq::ExecutionSeq`]'s own loop bodies -- carrying the
//! `// PERF(port):` and `// FIDELITY(port):` markers the idiom map requires at exactly this site,
//! since dropping real parallelism is a deferred optimisation (R15) *and* a deliberate order
//! choice (sum in TBB's own chunked, non-deterministic combination order vs. this port's fixed
//! left-to-right fold) that belongs in `port/reports/divergences.md`.
//!
//! **`tbb::spin_mutex`/`std::mutex` (`:40-41`) collapse to `()`,** the same move
//! `execution_seq.rs`'s module doc makes for `ExecutionSeq.hpp`'s `_Mtx`: this port's loop bodies
//! are serial, so [`Traits::SpinningMutex`]/[`Traits::BlockingMutex`] lock nothing, and a type
//! that exists only to be locked and unlocked with no effect needs no real mutex behind it.
//!
//! **`loop_`'s two overloads (`:27-37`) collapse to one `for` loop**, for the same reason
//! `execution_seq.rs`'s module doc gives: `execution.rs`'s [`crate::execution::Traits`] trait
//! already commits every policy to the one-iterator shape, so the iterator-vs-integer-range
//! SFINAE split this header also carries has nothing left to dispatch on here.
//!
//! **`tbb::blocked_range`/`tbb::task_arena` are not ported.** Both are TBB's own types, named on
//! the "Linked libraries" rule's R17 list of libraries the port calls through FFI rather than
//! reimplements where an in-scope caller needs them -- `ExecutionTbb` below needs neither, because
//! it never calls into TBB at all; `granularity` is accepted and ignored for the same reason
//! `ExecutionSeq.hpp`'s own sequential policy ignores it (`execution_seq.rs`'s own `for_each`).

/// `struct ExecutionTBB {};` (`:18`) plus `template<> struct
/// IsExecutionPolicy_<ExecutionTBB> : public std::true_type {};` (`:19`) -- the same opt-in-gate
/// collapse `execution_seq.rs`'s module doc explains for `ExecutionSeq`. `static constexpr
/// ExecutionTBB ex_tbb = {};` (`:22`) needs no counterpart for the same reason `ex_seq` does not.
#[derive(Debug, Default, Clone, Copy)]
pub struct ExecutionTbb;

impl crate::execution::Traits for ExecutionTbb {
    /// `using SpinningMutex = tbb::spin_mutex;` (`:40`) -- see the module doc.
    type SpinningMutex = ();
    /// `using BlockingMutex = std::mutex;` (`:41`).
    type BlockingMutex = ();

    /// `static size_t max_concurrency(const ExecutionTBB &) { return
    /// tbb::this_task_arena::max_concurrency(); }` (`:73-76`). The port's own loop bodies below
    /// are serial (see the module doc), so there is exactly one worker to report: `1`, the same
    /// value `ExecutionSeq.hpp`'s own `max_concurrency` returns, rather than a value this port
    /// cannot compute without linking TBB's task-arena machinery it otherwise never calls.
    // PERF(port): src/libslic3r/Execution/ExecutionTBB.hpp:73-76 -- the vendored
    // `max_concurrency` reports the real TBB task arena's worker count; this port's own
    // `for_each`/`reduce` below are a single serial loop (docs/PORTING.md's idiom map, TBB row:
    // "the port sums in a fixed order and is serial"), so `1` is the faithful answer for *this*
    // port's own concurrency, not a stand-in for the oracle's.
    fn max_concurrency(_ep: &Self) -> usize {
        1
    }

    /// `static void for_each(const ExecutionTBB &, It from, It to, Fn &&fn, size_t granularity)`
    /// (`:43-51`): `tbb::parallel_for` over chunks of `granularity`, each chunk walked serially by
    /// `loop_`. The port drops the parallel chunking and keeps one serial walk over the whole
    /// range, in iteration order -- see the module doc's "not reimplemented" note.
    // PERF(port): src/libslic3r/Execution/ExecutionTBB.hpp:43-51 -- the vendored body dispatches
    // `tbb::parallel_for` across `granularity`-sized chunks on however many worker threads
    // `tbb::this_task_arena::max_concurrency()` reports; this port runs one serial loop instead
    // (no audit yet licenses parallelising it -- docs/PORTING.md's idiom map, TBB row).
    fn for_each<It, F>(_ep: &Self, iter: It, mut fn_: F, _granularity: usize)
    where
        It: Iterator,
        F: FnMut(It::Item),
    {
        for item in iter {
            fn_(item);
        }
    }

    /// `static T reduce(const ExecutionTBB &, I from, I to, const T &init, MergeFn &&mergefn,
    /// AccessFn &&access, size_t granularity = 1)` (`:53-71`): `tbb::parallel_reduce` folds each
    /// chunk's partial result with `mergefn`, then combines the chunks' partials with `mergefn`
    /// again, in whatever order TBB's work-stealing scheduler finishes them -- an order this port
    /// does not reproduce. The port instead folds `mergefn(acc, access(item))` once per item,
    /// left to right over the whole range: the same fixed, serial reduction order
    /// `ExecutionSeq.hpp`'s own `reduce` uses, which is what makes this port deterministic where
    /// the oracle, for a non-associative `mergefn`, is not.
    // PERF(port): src/libslic3r/Execution/ExecutionTBB.hpp:53-71 -- the vendored body combines
    // per-chunk partial reductions in TBB's own (not fixed) completion order; this port folds the
    // whole range left to right instead (same reasoning as `for_each` above).
    // FIDELITY(port): src/libslic3r/Execution/ExecutionTBB.hpp:53-71 -- for a `mergefn` that is
    // associative but not commutative, or not exactly associative under floating-point rounding,
    // the vendored chunked/parallel combination order and this port's fixed left-to-right fold can
    // produce different bit patterns for the same logical reduction; R18's answer is this port's
    // own fixed order, deterministic where the oracle is not, recorded here for
    // `port/reports/divergences.md`'s regeneration pass per `docs/WORKFLOW.md` "Conflict
    // discipline".
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

impl crate::execution::ExecutionPolicy for ExecutionTbb {}

#[cfg(test)]
mod tests {
    use super::*;
    use slic3r_proof::proof_units;
    use slic3r_proof::{compare_unit, Oracle};

    #[test]
    fn max_concurrency_is_one_for_this_ports_serial_loop() {
        assert_eq!(crate::execution::max_concurrency(&ExecutionTbb), 1);
    }

    #[test]
    fn for_each_visits_every_item_in_order() {
        let data = vec![1, 2, 3, 4];
        let mut seen = Vec::new();
        crate::execution::for_each(&ExecutionTbb, data.iter().copied(), |x| seen.push(x), 2);
        assert_eq!(seen, vec![1, 2, 3, 4]);
    }

    /// Proof path 2 (R19). **Same finding as `execution_seq.rs`'s identical comment**: this row's
    /// `pipeline stage` cell (`posContouring`) is mechanically seeded and no `posContouring.dump`
    /// exists under `port/dumps/*/`; `ExecutionTBB.hpp` is header-only dispatch with no in-scope
    /// caller yet. The oracle side here calls the *real* vendored `Slic3r::ex_tbb` global
    /// (`ExecutionTBB.hpp:22`) through `Slic3r::execution::reduce` -- which really does dispatch
    /// through TBB's `parallel_reduce` on the oracle's side -- so this proof is also the one place
    /// that checks the FIDELITY(port) divergence above is a divergence in *order* only: with a
    /// commutative, exactly-associative `mergefn` (integer addition), both sides must still agree
    /// numerically even though one parallelises and the other does not.
    // PROOF: 2
    #[test]
    fn execution_tbb_reduce_matches_the_oracle_for_an_associative_merge() {
        let data = vec![1i64, 2, 3, 4, 5, 6];
        let actual = crate::execution::reduce(
            &ExecutionTbb,
            data.iter().copied(),
            0i64,
            |a: i64, b: i64| a + b,
            |x: i64| x + 1,
            2,
        );
        let oracle = proof_units::execution_tbb::path2_reduce_increments_and_sums(&data);
        compare_unit("Execution/ExecutionTBB.hpp", Oracle::Built, actual, oracle).expect(
            "ExecutionTbb's reduce must agree with the vendored TBB-backed specialisation for an \
             associative, commutative merge, despite the fixed-vs-parallel order difference the \
             FIDELITY(port) marker above records",
        );
    }
}

// PORT STATUS
// source: src/libslic3r/Execution/ExecutionTBB.hpp
// confidence: medium
// tier: exact
// oracle: built
