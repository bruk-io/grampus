//! Port of `src/libslic3r/Execution/Execution.hpp` (task `55-foundation-libslic3r-execution`,
//! plan step 20): the execution-policy trait surface every concrete policy (`ExecutionSeq.hpp`,
//! `ExecutionTBB.hpp` -- a sibling foundation task's own units, not this one's) specialises.
//! `template-heavy: yes` in `port/manifest.tsv`; "the execution policy is a 129-line trait
//! surface" (task.md's own words for why this task is sized the way it is).
//!
//! This header defines no concrete policy and runs no loop itself -- it only dispatches to
//! `Traits<EP>`, which a concrete policy specialises. The idiom map's "`tbb::parallel_for`,
//! `tbb::parallel_reduce` -> ... the port sums in a fixed order and is serial" rule, and the
//! `// PERF(port):`/`// FIDELITY(port):` markers that rule requires, therefore belong to
//! whichever module actually writes a loop body (`crates/slic3r-execution/src/execution_seq.rs`
//! and `execution_tbb.rs`, a sibling foundation task's own units) -- not to this one, which never
//! iterates anything itself.
//!
//! **Collapsed by this port, not reimplemented as separate items**: `template<class EP> struct
//! IsExecutionPolicy_ : public std::false_type {}` and `template<class EP> constexpr bool
//! IsExecutionPolicy = IsExecutionPolicy_<remove_cvref_t<EP>>::value` (`:15-18`) are a SFINAE
//! opt-in gate -- a concrete policy "specialises" `IsExecutionPolicy_<EP>` to `true_type` to
//! become a valid `EP`. Rust's own trait-implemented-or-not *is* that same compile-time gate with
//! nothing left to carry over: a type is a valid execution policy exactly when it implements
//! [`ExecutionPolicy`] below, which is the direct translation of "specialises
//! `IsExecutionPolicy_<EP>` to `true`" -- there is no separate boolean constant to port, and
//! `ExecutionPolicyOnly<EP, T>` (`:20-21`) likewise becomes an ordinary `where EP: ExecutionPolicy`
//! bound on each function rather than a standalone alias (the same SFINAE-to-trait-bound move
//! `crates/slic3r-libslic3r/src/libslic3r.rs`'s module doc states once for the whole port).
//!
//! **A finding for the regeneration pass, recorded in this task's pull request body, not here**:
//! this row's `proof path` cell (`2`) and `pipeline stage` cell (`posContouring`) are
//! mechanically seeded, like `enum_bitmask.hpp`'s (see that module's own doc comment for the
//! full note). A generic dispatch layer with no concrete policy to run writes no output any
//! committed stage dump could carry.
//!
//! **Not a dump read, but a real proof against the built oracle, not a hand-derived literal.**
//! `docs/PORTING.md`'s "Decisions recorded on 2026-11-25", decision 1, generalised the per-unit
//! shim mechanism beyond proof paths 1 and 3 -- `crates/slic3r-ffi/cxx/proof_units/
//! object_id.cpp`'s `path2_header_observations` already proves a `proof path: 2` row this same
//! way. This module's test does the same: `crates/slic3r-ffi/cxx/proof_units/execution.cpp`
//! specialises `Slic3r::IsExecutionPolicy_` and `Slic3r::execution::Traits` for a minimal, real,
//! serial proof-local policy (the dispatch template under test is generic in `EP`, not in which
//! concrete policy parameterises it -- a real one, `ExecutionSeq.hpp`/`ExecutionTBB.hpp`, is a
//! sibling foundation task's own unit) and calls the vendored `Slic3r::execution::reduce` free
//! function through it, and the test below compares the port's own [`reduce`] dispatch against
//! that oracle value through `compare_unit`. The `PORT STATUS` trailer's `oracle: built` is
//! therefore an accurate description of this row's proof, not the least-false placeholder an
//! earlier version of this module doc recorded.

/// `template<class EP, class En = void> struct Traits {};` (`:27`), specialised per concrete
/// policy (`ExecutionSeq.hpp`, `ExecutionTBB.hpp`): "each execution policy should declare two
/// types of mutexes ... these types should satisfy the `BasicLockable` concept" (`:31-32`) and
/// the three operations below. A Rust `impl Traits for ConcretePolicy` is that same
/// specialisation.
pub trait Traits: Sized {
    /// `template<class EP> using SpinningMutex = typename Traits<EP>::SpinningMutex;` (`:33`).
    type SpinningMutex;
    /// `template<class EP> using BlockingMutex = typename Traits<EP>::BlockingMutex;` (`:34`).
    type BlockingMutex;

    /// `size_t max_concurrency(const EP &ep)` (`:37-41`, the `Traits<EP>::max_concurrency` body
    /// this free function forwards to).
    fn max_concurrency(ep: &Self) -> usize;

    /// `void for_each(const EP &ep, It from, It to, Fn &&fn, size_t granularity = 1)`
    /// (`:45-49`). `It from, It to` is a begin/end iterator pair in C++; a Rust [`Iterator`]
    /// already carries its own end (`.next()` returning `None`), so the pair collapses to the
    /// one iterator argument below -- the idiom map's "iterator and pointer arithmetic -> slices
    /// and indices" rule applied to a *range*, not just a single pointer. `granularity` has no
    /// Rust default-argument spelling, so it stays an explicit parameter at every call site
    /// (C++'s `= 1` default becomes the caller writing `1` outright).
    fn for_each<It, F>(ep: &Self, iter: It, fn_: F, granularity: usize)
    where
        It: Iterator,
        F: FnMut(It::Item);

    /// `T reduce(const EP &ep, I from, I to, const T &init, MergeFn &&mergefn, AccessFn
    /// &&accessfn, size_t granularity = 1)` (`:55-73`); `from`/`to` collapse into `iter`, as
    /// [`Traits::for_each`]'s own doc comment explains.
    fn reduce<It, M, T, A>(
        ep: &Self,
        iter: It,
        init: T,
        mergefn: M,
        accessfn: A,
        granularity: usize,
    ) -> T
    where
        It: Iterator,
        M: FnMut(T, T) -> T,
        A: FnMut(It::Item) -> T;
}

/// `template<class EP> constexpr bool IsExecutionPolicy = ...` collapsed into a trait -- see the
/// module doc. A concrete policy implements this (with no methods of its own to add) to become a
/// valid `EP` for [`max_concurrency`], [`for_each`], [`reduce`] and [`accumulate`] below.
pub trait ExecutionPolicy: Traits {}

/// `template<class EP, class = ExecutionPolicyOnly<EP>> size_t max_concurrency(const EP &ep)`
/// (`:37-41`).
pub fn max_concurrency<EP: ExecutionPolicy>(ep: &EP) -> usize {
    EP::max_concurrency(ep)
}

/// `template<class EP, class It, class Fn, class = ExecutionPolicyOnly<EP>> void for_each(const
/// EP &ep, It from, It to, Fn &&fn, size_t granularity = 1)` (`:45-49`); `from`/`to` collapse
/// into `iter`, as [`Traits::for_each`]'s own doc comment explains.
pub fn for_each<EP, It, F>(ep: &EP, iter: It, fn_: F, granularity: usize)
where
    EP: ExecutionPolicy,
    It: Iterator,
    F: FnMut(It::Item),
{
    EP::for_each(ep, iter, fn_, granularity)
}

/// `template<class EP, class I, class MergeFn, class T, class AccessFn, class =
/// ExecutionPolicyOnly<EP>> T reduce(const EP &ep, I from, I to, const T &init, MergeFn
/// &&mergefn, AccessFn &&accessfn, size_t granularity = 1)` (`:55-73`); `from`/`to` collapse
/// into `iter`, as [`Traits::for_each`]'s own doc comment explains.
pub fn reduce<EP, It, M, T, A>(
    ep: &EP,
    iter: It,
    init: T,
    mergefn: M,
    accessfn: A,
    granularity: usize,
) -> T
where
    EP: ExecutionPolicy,
    It: Iterator,
    M: FnMut(T, T) -> T,
    A: FnMut(It::Item) -> T,
{
    EP::reduce(ep, iter, init, mergefn, accessfn, granularity)
}

/// `template<class EP, class I, class MergeFn, class T, class = ExecutionPolicyOnly<EP>> T
/// reduce(const EP &ep, I from, I to, const T &init, MergeFn &&mergefn, size_t granularity = 1)`
/// (`:77-92`): the overload with `accessfn` omitted, `[](const auto &i) { return i; }` (`:91`)
/// substituted in its place. Rust has no overload resolution on arity, so this is a
/// separately named function rather than a second `reduce` -- it forwards to [`reduce`] with
/// `std::convert::identity` as `accessfn`, exactly mirroring the original's own forwarding body.
pub fn reduce_identity<EP, It, M, T>(
    ep: &EP,
    iter: It,
    init: T,
    mergefn: M,
    granularity: usize,
) -> T
where
    EP: ExecutionPolicy,
    It: Iterator<Item = T>,
    M: FnMut(T, T) -> T,
{
    reduce(ep, iter, init, mergefn, std::convert::identity, granularity)
}

/// `template<class EP, class I, class T, class AccessFn, class = ExecutionPolicyOnly<EP>> T
/// accumulate(const EP &ep, I from, I to, const T &init, AccessFn &&accessfn, size_t granularity
/// = 1)` (`:94-108`): `reduce` with `std::plus<T>{}` (`:106`) as `mergefn`.
pub fn accumulate<EP, It, T, A>(ep: &EP, iter: It, init: T, accessfn: A, granularity: usize) -> T
where
    EP: ExecutionPolicy,
    It: Iterator,
    T: std::ops::Add<Output = T>,
    A: FnMut(It::Item) -> T,
{
    reduce(ep, iter, init, std::ops::Add::add, accessfn, granularity)
}

/// `template<class EP, class I, class T, class = ExecutionPolicyOnly<EP>> T accumulate(const EP
/// &ep, I from, I to, const T &init, size_t granularity = 1)` (`:111-124`): [`accumulate`] with
/// `accessfn` omitted, the same identity substitution [`reduce_identity`] documents.
pub fn accumulate_identity<EP, It, T>(ep: &EP, iter: It, init: T, granularity: usize) -> T
where
    EP: ExecutionPolicy,
    It: Iterator<Item = T>,
    T: std::ops::Add<Output = T>,
{
    accumulate(ep, iter, init, std::convert::identity, granularity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use slic3r_proof::proof_units;
    use slic3r_proof::{compare_unit, Oracle};

    /// A minimal, real test policy exercising the dispatch this module defines -- not the thing
    /// under test standing in for itself, but the smallest genuine `impl Traits` this header's
    /// own contract allows, since no concrete policy exists yet (`ExecutionSeq.hpp`/
    /// `ExecutionTBB.hpp` are a sibling foundation task's own units). Its own loop bodies are
    /// plain, single-threaded Rust: that is a real property of this test policy, not a
    /// substitute for Execution.hpp's own logic, which is pure dispatch and holds none of a
    /// loop's behaviour itself.
    struct OneThread;

    impl Traits for OneThread {
        type SpinningMutex = ();
        type BlockingMutex = ();

        fn max_concurrency(_ep: &Self) -> usize {
            1
        }

        fn for_each<It, F>(_ep: &Self, iter: It, mut fn_: F, _granularity: usize)
        where
            It: Iterator,
            F: FnMut(It::Item),
        {
            for item in iter {
                fn_(item);
            }
        }

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

    impl ExecutionPolicy for OneThread {}

    #[test]
    fn max_concurrency_dispatches_to_traits() {
        assert_eq!(max_concurrency(&OneThread), 1);
    }

    #[test]
    fn for_each_dispatches_and_visits_every_item() {
        let data = vec![1, 2, 3, 4];
        let mut seen = Vec::new();
        for_each(&OneThread, data.iter().copied(), |x| seen.push(x), 1);
        assert_eq!(seen, vec![1, 2, 3, 4]);
    }

    /// Proof path 2 (R19). No committed stage dump carries `Execution.hpp`'s output (see the
    /// module doc's "finding for the regeneration pass"), so this proof calls
    /// `crates/slic3r-ffi/cxx/proof_units/execution.cpp`'s shim, which dispatches through the
    /// real vendored `Slic3r::execution::reduce` free function -- decision 1's shim mechanism
    /// proving a `proof path: 2` row through the header's own code, exactly as
    /// `crates/slic3r-objectid/src/object_id.rs`'s `path2_header_observations` proof does for
    /// `ObjectID.hpp`, rather than through the dump reader.
    // PROOF: 2
    #[test]
    fn execution_reduce_dispatches_with_merge_and_access() {
        let data = vec![1i64, 2, 3, 4];
        let actual = reduce(
            &OneThread,
            data.iter().copied(),
            0i64,
            |a: i64, b: i64| a + b,
            |x: i64| x * 2,
            1,
        );
        let oracle = proof_units::execution::path2_reduce_doubles_and_sums(&data);
        compare_unit("Execution/Execution.hpp", Oracle::Built, actual, oracle).expect(
            "the port's reduce must dispatch identically to the vendored template's own forwarding",
        );
    }

    #[test]
    fn reduce_identity_omits_accessfn() {
        let data = vec![1i64, 2, 3];
        let actual = reduce_identity(&OneThread, data.into_iter(), 0i64, |a, b| a + b, 1);
        assert_eq!(actual, 6);
    }

    #[test]
    fn accumulate_sums_with_plus() {
        let data = vec![1i64, 2, 3, 4, 5];
        let actual = accumulate(&OneThread, data.iter().copied(), 0i64, |x| x, 1);
        assert_eq!(actual, 15);
    }

    #[test]
    fn accumulate_identity_omits_accessfn() {
        let data = vec![10i64, 20, 30];
        let actual = accumulate_identity(&OneThread, data.into_iter(), 0i64, 1);
        assert_eq!(actual, 60);
    }
}

// PORT STATUS
// source: src/libslic3r/Execution/Execution.hpp
// confidence: medium
// tier: exact
// oracle: built
