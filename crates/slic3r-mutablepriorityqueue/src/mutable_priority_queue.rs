//! Port of `MutablePriorityQueue.hpp` (task 53-foundation-int128-queue, plan step 20): a binary
//! min-heap whose elements know their own index (`IndexSetter`), so an element already in the
//! queue can be removed or re-prioritised in place -- the thing a plain `std::priority_queue`
//! cannot do -- plus `MutableSkipHeapPriorityQueue`, a cache-friendlier variant built on
//! `SkipHeapAddressing`'s "binary heap of binary miniheaps" layout.
//!
//! `T: Copy` is this port's rendering of the vendored
//! `static_assert(std::is_trivially_copyable<T>::value, ...)`: Rust's type system enforces the
//! same property structurally (a `T` that is not `Copy` fails to compile against this bound), so
//! no runtime or const-time assertion is needed to reproduce it.
//!
//! **NDEBUG/Release semantics chosen, not yet proven.** The normative oracle
//! (`vendor/OrcaSlicer/build/arm64`) is built `-DCMAKE_BUILD_TYPE=Release`, which defines
//! `NDEBUG`. Three vendored methods (`clear`, `pop`, `remove` on both classes) read differently
//! depending on `NDEBUG`: the `#ifdef NDEBUG` / `#endif` pair around `if (ResetIndexWhenRemoved)`
//! means that guard is live only in a Release build, and in a Debug build the preprocessor
//! instead deletes just that `if` line, leaving the mark-as-removed loop unconditional. This port
//! ports the Release reading -- `if RESET_INDEX_WHEN_REMOVED { ... }` -- because the Release
//! library is the normative oracle. **An earlier version of this sentence claimed the Release
//! reading "is the only configuration its proof can ever observe"; that is false, and the
//! opposite is true.** The proof's own translation unit (`crates/slic3r-ffi/cxx/proof_units/
//! mutable_priority_queue.cpp`) `#include`s this header fresh into a `cxx_build` TU that never
//! links `liblibslic3r.a`, and that TU is compiled under `cargo build`/`cargo test`'s `dev`/`test`
//! profile, which (`crates/slic3r-ffi/build.rs`'s own measured comment) adds no `-DNDEBUG`. So
//! the proof's own oracle side compiles the *unconditional* (Debug) reading of this guard, not
//! the Release one this module implements -- and no committed test tells the two apart: the
//! `ResetIndexWhenRemoved = true` destructor-marks comparisons (below) coincide under either
//! reading, and the `= false` order comparisons never read the index marks at all. The port's
//! choice of the Release reading is still right, because the Release library is what this task's
//! proof is supposed to be normative against -- but that choice is unproven by anything committed
//! here, and a comparison that did witness `ResetIndexWhenRemoved = false`'s index marks would
//! fail. Giving the proof's cxx translation unit NDEBUG parity with the oracle is out of this
//! task's write list (`build.rs` is not a file a foundation task may touch) and is recorded there
//! as a named hazard already; this module's own doc previously mischaracterised it as resolved.

/// `constexpr auto InvalidQueueID = std::numeric_limits<size_t>::max();`
///
/// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:10
pub const INVALID_QUEUE_ID: usize = usize::MAX;

/// `template<typename T, typename IndexSetter, typename LessPredicate, const bool
/// ResetIndexWhenRemoved = false> class MutablePriorityQueue`
///
/// `IndexSetter` mirrors a C++ callable of signature `void(T&, size_t)` (`FnMut`, since the
/// vendored callable may itself carry mutable state -- none of the call sites below need `T`
/// itself to be mutated through any path other than this setter, matching the original's own
/// `m_index_setter(item, idx)` calls). `LessPredicate` mirrors `bool(const T&, const T&) const`.
/// **`Fn`, never `FnMut`, is this port's own narrowing choice, not one the vendored source
/// requires**: `m_less_predicate` is a plain, non-`const` data member
/// (`src/libslic3r/MutablePriorityQueue.hpp:52`), called from the non-`const` methods
/// `update_heap_up`/`update_heap_down` (`:150`, `:181`), and no `const LessPredicate&` appears
/// anywhere in the file -- the original places no const requirement on it at all. `Fn` only
/// restricts what callers may pass (no predicate that itself needs `&mut self`), never a
/// behaviour divergence, so it is kept as the simpler bound, stated here as the port's own
/// choice rather than attributed to a source constraint that does not exist.
///
/// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:12
pub struct MutablePriorityQueue<
    T,
    IndexSetter,
    LessPredicate,
    const RESET_INDEX_WHEN_REMOVED: bool = false,
> where
    T: Copy,
    IndexSetter: FnMut(&mut T, usize),
    LessPredicate: Fn(&T, &T) -> bool,
{
    heap: Vec<T>,
    index_setter: IndexSetter,
    less_predicate: LessPredicate,
}

impl<T, IndexSetter, LessPredicate, const RESET_INDEX_WHEN_REMOVED: bool>
    MutablePriorityQueue<T, IndexSetter, LessPredicate, RESET_INDEX_WHEN_REMOVED>
where
    T: Copy,
    IndexSetter: FnMut(&mut T, usize),
    LessPredicate: Fn(&T, &T) -> bool,
{
    /// `MutablePriorityQueue(IndexSetter &&index_setter, LessPredicate &&less_predicate)` --
    /// "It is recommended to use make_mutable_priority_queue() for construction", ported below
    /// as the free function of the same name.
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:18
    pub fn new(index_setter: IndexSetter, less_predicate: LessPredicate) -> Self {
        MutablePriorityQueue {
            heap: Vec::new(),
            index_setter,
            less_predicate,
        }
    }

    /// `void reserve(size_t cnt) { m_heap.reserve(cnt); }`
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:25
    pub fn reserve(&mut self, cnt: usize) {
        self.heap.reserve(cnt);
    }

    /// `size_t size() const { return m_heap.size(); }`
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:33
    pub fn size(&self) -> usize {
        self.heap.len()
    }

    /// `bool empty() const { return m_heap.empty(); }`
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:34
    pub fn empty(&self) -> bool {
        self.heap.is_empty()
    }

    /// `T& top() { return m_heap.front(); }`
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:29
    pub fn top(&mut self) -> &mut T {
        &mut self.heap[0]
    }

    /// `iterator begin() { return m_heap.begin(); }` / `const_iterator cbegin() const`, folded
    /// into one slice-backed pair of accessors (`iter`/`iter_mut`) rather than four separate
    /// iterator-typedef methods -- the simplest translation that keeps the same meaning (every
    /// element, heap order, mutable or not), not a reproduction of C++ iterator typedefs Rust has
    /// no equivalent vocabulary for.
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:40
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.heap.iter()
    }

    /// See `iter` above.
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:41
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.heap.iter_mut()
    }

    /// ```c++
    /// void clear()
    /// {
    /// #ifdef NDEBUG
    ///     if (ResetIndexWhenRemoved)
    /// #endif
    ///     {
    ///         for (size_t idx = 0; idx < m_heap.size(); ++ idx)
    ///             m_index_setter(m_heap[idx], std::numeric_limits<size_t>::max());
    ///     }
    ///     m_heap.clear();
    /// }
    /// ```
    /// Release/`NDEBUG` reading only -- see the module doc.
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:63
    pub fn clear(&mut self) {
        if RESET_INDEX_WHEN_REMOVED {
            for item in &mut self.heap {
                (self.index_setter)(item, INVALID_QUEUE_ID);
            }
        }
        self.heap.clear();
    }

    /// `void push(const T &item)` / `void push(T &&item)` -- `T: Copy` collapses the two
    /// overloads (a copy and a move of a trivially copyable type are the same bits) into one
    /// by-value method.
    ///
    /// **Statement-order divergence, undocumented until now.** The vendored body is
    /// `m_heap.emplace_back(item); m_index_setter(m_heap.back(), idx);` (`:81`-`:82`) -- the
    /// setter runs on the element already inside the vector, after the push. This port calls the
    /// setter on the caller's local copy first and pushes afterward. Equivalent for every `T` and
    /// setter shape this unit's own proof uses (both the item-field setters and the external-
    /// array setters), but it is a real reordering of two observable side effects, unlike
    /// `update_heap_up`/`update_heap_down`'s by-index rewrite, which changes *how* the same two
    /// writes happen rather than their order (see that method's own doc). Named here rather than
    /// restored, since restoring it needs no behaviour change, only a style the rest of this
    /// module does not otherwise use.
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:26
    pub fn push(&mut self, mut item: T) {
        let idx = self.heap.len();
        (self.index_setter)(&mut item, idx);
        self.heap.push(item);
        self.update_heap_up(0, idx);
    }

    /// ```c++
    /// void pop()
    /// {
    ///     assert(! m_heap.empty());
    /// #ifdef NDEBUG
    ///     if (ResetIndexWhenRemoved)
    /// #endif
    ///     { m_index_setter(m_heap.front(), std::numeric_limits<size_t>::max()); }
    ///     if (m_heap.size() > 1) {
    ///         m_heap.front() = m_heap.back();
    ///         m_heap.pop_back();
    ///         m_index_setter(m_heap.front(), 0);
    ///         update_heap_down(0, m_heap.size() - 1);
    ///     } else
    ///         m_heap.clear();
    /// }
    /// ```
    /// `assert(! m_heap.empty())` is compiled out of the oracle's Release build (standard
    /// `NDEBUG` behaviour) and is ported as `debug_assert!`, never a live panic.
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:96
    pub fn pop(&mut self) {
        debug_assert!(!self.heap.is_empty());
        if RESET_INDEX_WHEN_REMOVED {
            (self.index_setter)(&mut self.heap[0], INVALID_QUEUE_ID);
        }
        if self.heap.len() > 1 {
            let last = *self.heap.last().expect("just checked len() > 1");
            self.heap[0] = last;
            self.heap.pop();
            (self.index_setter)(&mut self.heap[0], 0);
            let bottom = self.heap.len() - 1;
            self.update_heap_down(0, bottom);
        } else {
            self.heap.clear();
        }
    }

    /// ```c++
    /// void remove(size_t idx)
    /// {
    ///     assert(idx < m_heap.size());
    /// #ifdef NDEBUG
    ///     if (ResetIndexWhenRemoved)
    /// #endif
    ///     { m_index_setter(m_heap[idx], std::numeric_limits<size_t>::max()); }
    ///     if (idx + 1 == m_heap.size()) {
    ///         m_heap.pop_back();
    ///         return;
    ///     }
    ///     m_heap[idx] = m_heap.back();
    ///     m_index_setter(m_heap[idx], idx);
    ///     m_heap.pop_back();
    ///     update_heap_down(idx, m_heap.size() - 1);
    ///     update_heap_up(0, idx);
    /// }
    /// ```
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:117
    pub fn remove(&mut self, idx: usize) {
        debug_assert!(idx < self.heap.len());
        if RESET_INDEX_WHEN_REMOVED {
            (self.index_setter)(&mut self.heap[idx], INVALID_QUEUE_ID);
        }
        if idx + 1 == self.heap.len() {
            self.heap.pop();
            return;
        }
        let last = *self
            .heap
            .last()
            .expect("idx + 1 != len(), so len() >= 2 here");
        self.heap[idx] = last;
        (self.index_setter)(&mut self.heap[idx], idx);
        self.heap.pop();
        let bottom = self.heap.len() - 1;
        self.update_heap_down(idx, bottom);
        self.update_heap_up(0, idx);
    }

    /// `void update(size_t idx) { T item = m_heap[idx]; remove(idx); push(item); }`
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:31
    pub fn update(&mut self, idx: usize) {
        let item = self.heap[idx];
        self.remove(idx);
        self.push(item);
    }

    /// ```c++
    /// void update_heap_up(size_t top, size_t bottom)
    /// {
    ///     size_t childIdx = bottom;
    ///     T *child = &m_heap[childIdx];
    ///     for (;;) {
    ///         size_t parentIdx = (childIdx - 1) >> 1;
    ///         if (childIdx == 0 || parentIdx < top)
    ///             break;
    ///         T *parent = &m_heap[parentIdx];
    ///         if (! m_less_predicate(*parent, *child)) {
    ///             T tmp = *parent;
    ///             m_index_setter(tmp,    childIdx);
    ///             m_index_setter(*child, parentIdx);
    ///             m_heap[parentIdx] = *child;
    ///             m_heap[childIdx]  = tmp;
    ///         }
    ///         childIdx = parentIdx;
    ///         child = parent;
    ///     }
    /// }
    /// ```
    /// Ported by index rather than by pointer (`child`/`parent` become `child_idx`/`parent_idx`
    /// read afresh each iteration): the vendored pointers are never aliased across an iteration
    /// that also writes through them, so this is a like-for-like translation of "the value at
    /// this index", not a behaviour change -- the port's own idiom for "iterator and pointer
    /// arithmetic" (`docs/PORTING.md`'s idiom map) applies the same way here.
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:140
    fn update_heap_up(&mut self, top: usize, bottom: usize) {
        let mut child_idx = bottom;
        loop {
            let parent_idx = (child_idx.wrapping_sub(1)) >> 1;
            if child_idx == 0 || parent_idx < top {
                break;
            }
            if !(self.less_predicate)(&self.heap[parent_idx], &self.heap[child_idx]) {
                let mut tmp = self.heap[parent_idx];
                (self.index_setter)(&mut tmp, child_idx);
                let mut child_val = self.heap[child_idx];
                (self.index_setter)(&mut child_val, parent_idx);
                self.heap[parent_idx] = child_val;
                self.heap[child_idx] = tmp;
            }
            child_idx = parent_idx;
        }
    }

    /// ```c++
    /// void update_heap_down(size_t top, size_t bottom)
    /// {
    ///     size_t parentIdx = top;
    ///     T *parent = &m_heap[parentIdx];
    ///     for (;;) {
    ///         size_t childIdx = (parentIdx << 1) + 1;
    ///         if (childIdx > bottom)
    ///             break;
    ///         T *child = &m_heap[childIdx];
    ///         size_t child2Idx = childIdx + 1;
    ///         if (child2Idx <= bottom) {
    ///             T *child2 = &m_heap[child2Idx];
    ///             if (! m_less_predicate(*child, *child2)) {
    ///                 child = child2;
    ///                 childIdx = child2Idx;
    ///             }
    ///         }
    ///         if (m_less_predicate(*parent, *child))
    ///             return;
    ///         T tmp = *parent;
    ///         m_index_setter(tmp,    childIdx);
    ///         m_index_setter(*child, parentIdx);
    ///         m_heap[parentIdx] = *child;
    ///         m_heap[childIdx] = tmp;
    ///         parentIdx = childIdx;
    ///         parent = child;
    ///     }
    /// }
    /// ```
    /// Same by-index translation as `update_heap_up` above.
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:164
    fn update_heap_down(&mut self, top: usize, bottom: usize) {
        let mut parent_idx = top;
        loop {
            let mut child_idx = (parent_idx << 1) + 1;
            if child_idx > bottom {
                break;
            }
            let child2_idx = child_idx + 1;
            if child2_idx <= bottom
                && !(self.less_predicate)(&self.heap[child_idx], &self.heap[child2_idx])
            {
                child_idx = child2_idx;
            }
            if (self.less_predicate)(&self.heap[parent_idx], &self.heap[child_idx]) {
                return;
            }
            let mut tmp = self.heap[parent_idx];
            (self.index_setter)(&mut tmp, child_idx);
            let mut child_val = self.heap[child_idx];
            (self.index_setter)(&mut child_val, parent_idx);
            self.heap[parent_idx] = child_val;
            self.heap[child_idx] = tmp;
            parent_idx = child_idx;
        }
    }
}

impl<T, IndexSetter, LessPredicate, const RESET_INDEX_WHEN_REMOVED: bool> std::ops::Index<usize>
    for MutablePriorityQueue<T, IndexSetter, LessPredicate, RESET_INDEX_WHEN_REMOVED>
where
    T: Copy,
    IndexSetter: FnMut(&mut T, usize),
    LessPredicate: Fn(&T, &T) -> bool,
{
    type Output = T;
    /// `T& operator[](std::size_t idx) noexcept { return m_heap[idx]; }` (and its `const`
    /// overload) -- `Index`/`IndexMut` together port both.
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:35
    fn index(&self, idx: usize) -> &T {
        &self.heap[idx]
    }
}

impl<T, IndexSetter, LessPredicate, const RESET_INDEX_WHEN_REMOVED: bool> Drop
    for MutablePriorityQueue<T, IndexSetter, LessPredicate, RESET_INDEX_WHEN_REMOVED>
where
    T: Copy,
    IndexSetter: FnMut(&mut T, usize),
    LessPredicate: Fn(&T, &T) -> bool,
{
    /// `~MutablePriorityQueue() { clear(); }` -- does more than drop its fields under
    /// `RESET_INDEX_WHEN_REMOVED`: `clear()` writes `INVALID_QUEUE_ID` through `index_setter`
    /// into every element still queued, and the setter's whole purpose (see this module's own
    /// doc on `IndexSetter`) is to reach state that outlives the queue (e.g. a field on a
    /// pointee the queue never owns). Omitting this `Drop` left that external state stale after
    /// a non-empty queue went out of scope (adversarial review, this task) -- the idiom map's
    /// rule for a destructor that does more than drop its fields.
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:22
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T, IndexSetter, LessPredicate, const RESET_INDEX_WHEN_REMOVED: bool> std::ops::IndexMut<usize>
    for MutablePriorityQueue<T, IndexSetter, LessPredicate, RESET_INDEX_WHEN_REMOVED>
where
    T: Copy,
    IndexSetter: FnMut(&mut T, usize),
    LessPredicate: Fn(&T, &T) -> bool,
{
    fn index_mut(&mut self, idx: usize) -> &mut T {
        &mut self.heap[idx]
    }
}

/// ```c++
/// template<typename T, const bool ResetIndexWhenRemoved, typename IndexSetter, typename
/// LessPredicate>
/// MutablePriorityQueue<T, IndexSetter, LessPredicate, ResetIndexWhenRemoved>
/// make_mutable_priority_queue(IndexSetter &&index_setter, LessPredicate &&less_predicate)
/// {
///     return MutablePriorityQueue<T, IndexSetter, LessPredicate, ResetIndexWhenRemoved>(
///         std::forward<IndexSetter>(index_setter), std::forward<LessPredicate>(less_predicate));
/// }
/// ```
///
/// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:56
pub fn make_mutable_priority_queue<
    T,
    const RESET_INDEX_WHEN_REMOVED: bool,
    IndexSetter,
    LessPredicate,
>(
    index_setter: IndexSetter,
    less_predicate: LessPredicate,
) -> MutablePriorityQueue<T, IndexSetter, LessPredicate, RESET_INDEX_WHEN_REMOVED>
where
    T: Copy,
    IndexSetter: FnMut(&mut T, usize),
    LessPredicate: Fn(&T, &T) -> bool,
{
    MutablePriorityQueue::new(index_setter, less_predicate)
}

/// ```c++
/// template <std::size_t blocking>
/// struct SkipHeapAddressing
/// {
///     static const constexpr std::size_t block_size = blocking;
///     static const constexpr std::size_t block_mask = block_size - 1;
///     static_assert((block_size & block_mask) == 0U, "block size must be 2^n for some integer n");
///     ...
/// };
/// ```
/// A zero-sized, const-generic addressing scheme -- every method is `static` in the original and
/// so becomes a free associated function here, with no instance. `BLOCK_SIZE` must be a power of
/// two, the same precondition the vendored `static_assert` states and `BLOCK_MASK` below now
/// enforces, not just documents (adversarial review, this task: flagged as a `pub` const-generic
/// API whose precondition was unenforced).
///
/// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:206
pub struct SkipHeapAddressing<const BLOCK_SIZE: usize>;

impl<const BLOCK_SIZE: usize> SkipHeapAddressing<BLOCK_SIZE> {
    /// `static const constexpr std::size_t block_size = blocking;`
    pub const BLOCK_SIZE: usize = BLOCK_SIZE;
    /// `static const constexpr std::size_t block_mask = block_size - 1;`
    ///
    /// `static_assert((block_size & block_mask) == 0U, "block size must be 2^n for some integer
    /// n");` -- enforced here, not merely documented. Rust has no direct const-generic
    /// `static_assert`, but a `const` item is evaluated at monomorphization time by any code path
    /// that reads it, and every method below reads `BLOCK_MASK` -- directly, through
    /// `block_offset`/`is_padding`/`is_block_root`/`child_of`/`parent_of`, or (for
    /// `is_block_leaf`, the one method whose own expression needs only `BLOCK_SIZE`) through an
    /// explicit `let _ = Self::BLOCK_MASK;` added for exactly this reason -- so an instantiation
    /// with a non-power-of-two `BLOCK_SIZE` is a compile error rather than silent wrong
    /// addressing, from every public entry point.
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:212
    pub const BLOCK_MASK: usize = {
        assert!(
            BLOCK_SIZE != 0 && (BLOCK_SIZE & (BLOCK_SIZE - 1)) == 0,
            "block size must be 2^n for some integer n"
        );
        BLOCK_SIZE - 1
    };

    /// `static inline bool is_block_leaf(std::size_t node_no) noexcept { return (node_no &
    /// (block_size >> 1)) != 0U; }`
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:242
    pub const fn is_block_leaf(node_no: usize) -> bool {
        // Forces `BLOCK_MASK`'s own `const assert!()` (see its doc comment) to evaluate even
        // through this entry point, which otherwise reads only `BLOCK_SIZE` and would let
        // `SkipHeapAddressing::<24>::is_block_leaf(_)` compile silently with wrong addressing.
        let _: usize = Self::BLOCK_MASK;
        (node_no & (Self::BLOCK_SIZE >> 1)) != 0
    }

    /// `static inline bool is_padding(std::size_t node_no) noexcept { return
    /// block_offset(node_no) == 0U; }`
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:244
    pub const fn is_padding(node_no: usize) -> bool {
        Self::block_offset(node_no) == 0
    }

    /// `static inline bool is_block_root(std::size_t node_no) noexcept { return
    /// block_offset(node_no) == 1U; }`
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:248
    const fn is_block_root(node_no: usize) -> bool {
        Self::block_offset(node_no) == 1
    }

    /// `static inline std::size_t block_offset(std::size_t node_no) noexcept { return node_no &
    /// block_mask; }`
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:250
    const fn block_offset(node_no: usize) -> usize {
        node_no & Self::BLOCK_MASK
    }

    /// `static inline std::size_t block_base(std::size_t node_no) noexcept { return node_no &
    /// ~block_mask; }`
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:252
    const fn block_base(node_no: usize) -> usize {
        node_no & !Self::BLOCK_MASK
    }

    /// `static inline std::size_t child_no(std::size_t node_no) noexcept { assert(is_block_leaf(node_no));
    /// return node_no & (block_mask >> 1); }`
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:254
    const fn child_no(node_no: usize) -> usize {
        debug_assert!(Self::is_block_leaf(node_no));
        node_no & (Self::BLOCK_MASK >> 1)
    }

    /// ```c++
    /// static inline std::size_t child_of(std::size_t node_no) noexcept {
    ///     if (! is_block_leaf(node_no))
    ///         return node_no + block_offset(node_no);
    ///     return (block_base(node_no) + 1 + child_no(node_no) * 2) * block_size + 1;
    /// }
    /// ```
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:214
    pub const fn child_of(node_no: usize) -> usize {
        if !Self::is_block_leaf(node_no) {
            node_no + Self::block_offset(node_no)
        } else {
            (Self::block_base(node_no) + 1 + Self::child_no(node_no) * 2) * Self::BLOCK_SIZE + 1
        }
    }

    /// ```c++
    /// static inline std::size_t parent_of(std::size_t node_no) noexcept {
    ///     auto const node_root = block_base(node_no);
    ///     if (! is_block_root(node_no))
    ///         return node_root + block_offset(node_no) / 2;
    ///     auto const parent_base = block_base(node_root / block_size - 1);
    ///     auto const child = ((node_no - block_size) / block_size - parent_base) / 2;
    ///     return parent_base + block_size / 2 + child;
    /// }
    /// ```
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:224
    ///
    /// **`wrapping_*`, not plain arithmetic, in the `else` branch.** `size_t` is unsigned, so a
    /// C++ `size_t` subtraction that goes negative is defined (not undefined) behaviour: it wraps
    /// modulo 2^64, in every build configuration, not only when `NDEBUG` is set (unlike the
    /// assert-guarded cases elsewhere in this module). `update_heap_up`/`update_heap_down` call
    /// `parent_of`/`child_of` before checking whether the loop should actually stop (`size_t
    /// parentIdx = address::parent_of(childIdx); if (childIdx == 1 || ...) break;`), so
    /// `parent_of(1)` really is called once per `update_heap_up` -- `is_block_root(1)` is true,
    /// so this `else` branch runs, `node_root` is `0`, and `node_root / block_size - 1`
    /// underflows -- every time, not just at a boundary; its wrapped result is computed and then
    /// immediately discarded by the caller. Reproducing the wraparound (rather than panicking,
    /// which plain `-` would do in a debug build) is what "exact equality against the oracle"
    /// requires here, even though the oracle's own wrapped value is likewise never read.
    pub const fn parent_of(node_no: usize) -> usize {
        let node_root = Self::block_base(node_no);
        if !Self::is_block_root(node_no) {
            node_root + Self::block_offset(node_no) / 2
        } else {
            let parent_base = Self::block_base((node_root / Self::BLOCK_SIZE).wrapping_sub(1));
            let child = ((node_no.wrapping_sub(Self::BLOCK_SIZE)) / Self::BLOCK_SIZE)
                .wrapping_sub(parent_base)
                / 2;
            parent_base
                .wrapping_add(Self::BLOCK_SIZE / 2)
                .wrapping_add(child)
        }
    }
}

/// Default block size every call site in this module uses, matching the vendored template
/// parameter's own default.
///
/// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:259
pub const DEFAULT_BLOCK_SIZE: usize = 32;

/// `template<typename T, typename IndexSetter, typename LessPredicate, std::size_t blocking = 32,
/// const bool ResetIndexWhenRemoved = false> class MutableSkipHeapPriorityQueue` -- the
/// cache-friendly variant, a binary heap of binary miniheaps addressed by `SkipHeapAddressing`.
/// Index `0` is permanently unused padding (see `reserve`/`push`/`pop_back` below); a non-empty
/// queue therefore always holds at least two elements in `heap`.
///
/// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:260
pub struct MutableSkipHeapPriorityQueue<
    T,
    IndexSetter,
    LessPredicate,
    const BLOCK_SIZE: usize = DEFAULT_BLOCK_SIZE,
    const RESET_INDEX_WHEN_REMOVED: bool = false,
> where
    // `Default` is this port's rendering of the vendored `m_heap.emplace_back(T())` in `push`
    // below -- a value-initialised placeholder for the permanently unused padding slot at index
    // `0` -- which the base `MutablePriorityQueue` above never needs (it has no padding slot).
    T: Copy + Default,
    IndexSetter: FnMut(&mut T, usize),
    LessPredicate: Fn(&T, &T) -> bool,
{
    heap: Vec<T>,
    index_setter: IndexSetter,
    less_predicate: LessPredicate,
}

type Address<const BLOCK_SIZE: usize> = SkipHeapAddressing<BLOCK_SIZE>;

impl<
        T,
        IndexSetter,
        LessPredicate,
        const BLOCK_SIZE: usize,
        const RESET_INDEX_WHEN_REMOVED: bool,
    >
    MutableSkipHeapPriorityQueue<
        T,
        IndexSetter,
        LessPredicate,
        BLOCK_SIZE,
        RESET_INDEX_WHEN_REMOVED,
    >
where
    T: Copy + Default,
    IndexSetter: FnMut(&mut T, usize),
    LessPredicate: Fn(&T, &T) -> bool,
{
    /// `MutableSkipHeapPriorityQueue(IndexSetter &&index_setter, LessPredicate &&less_predicate)`
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:267
    pub fn new(index_setter: IndexSetter, less_predicate: LessPredicate) -> Self {
        MutableSkipHeapPriorityQueue {
            heap: Vec::new(),
            index_setter,
            less_predicate,
        }
    }

    /// `void reserve(size_t cnt) { m_heap.reserve(cnt + ((cnt + (address::block_size - 1)) /
    /// (address::block_size - 1))); }`
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:275
    pub fn reserve(&mut self, cnt: usize) {
        self.heap.reserve(
            cnt + (cnt + (Address::<BLOCK_SIZE>::BLOCK_SIZE - 1))
                / (Address::<BLOCK_SIZE>::BLOCK_SIZE - 1),
        );
    }

    /// `T& top() { return m_heap[1]; }`
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:279
    pub fn top(&mut self) -> &mut T {
        &mut self.heap[1]
    }

    /// `size_t size() const noexcept { return m_heap.size() - (m_heap.size() + address::block_size
    /// - 1) / address::block_size; }`
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:283
    // The literal `(x + n - 1) / n` shape (clippy's `manual_div_ceil`) is the vendored
    // expression transcribed as written, not idiomatic Rust's `.div_ceil()` -- faithfulness
    // over idiom, this intent's own rule (docs/PORTING.md); the two are equivalent here.
    #[allow(clippy::manual_div_ceil)]
    pub fn size(&self) -> usize {
        self.heap.len()
            - (self.heap.len() + Address::<BLOCK_SIZE>::BLOCK_SIZE - 1)
                / Address::<BLOCK_SIZE>::BLOCK_SIZE
    }

    /// `bool empty() const { return m_heap.empty(); }`
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:284
    pub fn empty(&self) -> bool {
        self.heap.is_empty()
    }

    /// ```c++
    /// void clear()
    /// {
    /// #ifdef NDEBUG
    ///     if (ResetIndexWhenRemoved)
    /// #endif
    ///     {
    ///         for (size_t idx = 0; idx < m_heap.size(); ++ idx)
    ///             if (! address::is_padding(idx))
    ///                 m_index_setter(m_heap[idx], std::numeric_limits<size_t>::max());
    ///     }
    ///     m_heap.clear();
    /// }
    /// ```
    /// Release/`NDEBUG` reading only -- see the module doc.
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:314
    pub fn clear(&mut self) {
        if RESET_INDEX_WHEN_REMOVED {
            for idx in 0..self.heap.len() {
                if !Address::<BLOCK_SIZE>::is_padding(idx) {
                    (self.index_setter)(&mut self.heap[idx], INVALID_QUEUE_ID);
                }
            }
        }
        self.heap.clear();
    }

    /// `void pop_back() noexcept { assert(m_heap.size() > 1); assert(!
    /// address::is_padding(m_heap.size() - 1)); m_heap.pop_back(); if
    /// (address::is_padding(m_heap.size() - 1)) m_heap.pop_back(); }`
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:291
    fn pop_back(&mut self) {
        debug_assert!(self.heap.len() > 1);
        debug_assert!(!Address::<BLOCK_SIZE>::is_padding(self.heap.len() - 1));
        self.heap.pop();
        if Address::<BLOCK_SIZE>::is_padding(self.heap.len() - 1) {
            self.heap.pop();
        }
    }

    /// `void push(const T &item)` / `void push(T &&item)` -- see `MutablePriorityQueue::push`'s
    /// own doc for why `T: Copy` collapses the two overloads into one.
    ///
    /// Same statement-order divergence as the base queue's `push` -- see that method's own doc.
    /// Vendored: `m_heap.emplace_back(item); m_index_setter(m_heap.back(), idx);`
    /// (`:335`-`:336`).
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:276
    pub fn push(&mut self, item: T) {
        if Address::<BLOCK_SIZE>::is_padding(self.heap.len()) {
            // `T()`: a value-initialised placeholder, never the real `item` -- it occupies the
            // permanently unused padding slot and is never read back.
            self.heap.push(T::default());
        }
        let idx = self.heap.len();
        let mut item = item;
        (self.index_setter)(&mut item, idx);
        self.heap.push(item);
        self.update_heap_up(1, idx);
    }

    /// ```c++
    /// void pop()
    /// {
    ///     assert(! m_heap.empty());
    /// #ifdef NDEBUG
    ///     if (ResetIndexWhenRemoved)
    /// #endif
    ///     { m_index_setter(m_heap[1], std::numeric_limits<size_t>::max()); }
    ///     if (m_heap.size() > 2) {
    ///         m_heap[1] = m_heap.back();
    ///         this->pop_back();
    ///         m_index_setter(m_heap[1], 1);
    ///         update_heap_down(1, m_heap.size() - 1);
    ///     } else
    ///         m_heap.clear();
    /// }
    /// ```
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:352
    pub fn pop(&mut self) {
        debug_assert!(!self.heap.is_empty());
        if RESET_INDEX_WHEN_REMOVED {
            (self.index_setter)(&mut self.heap[1], INVALID_QUEUE_ID);
        }
        if self.heap.len() > 2 {
            let last = *self.heap.last().expect("just checked len() > 2");
            self.heap[1] = last;
            self.pop_back();
            (self.index_setter)(&mut self.heap[1], 1);
            let bottom = self.heap.len() - 1;
            self.update_heap_down(1, bottom);
        } else {
            self.heap.clear();
        }
    }

    /// ```c++
    /// void remove(size_t idx)
    /// {
    ///     assert(idx < m_heap.size());
    ///     assert(! address::is_padding(idx));
    /// #ifdef NDEBUG
    ///     if (ResetIndexWhenRemoved)
    /// #endif
    ///     { m_index_setter(m_heap[idx], std::numeric_limits<size_t>::max()); }
    ///     if (idx + 1 == m_heap.size()) {
    ///         this->pop_back();
    ///         return;
    ///     }
    ///     m_heap[idx] = m_heap.back();
    ///     m_index_setter(m_heap[idx], idx);
    ///     this->pop_back();
    ///     update_heap_down(idx, m_heap.size() - 1);
    ///     update_heap_up(1, idx);
    /// }
    /// ```
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:374
    pub fn remove(&mut self, idx: usize) {
        debug_assert!(idx < self.heap.len());
        debug_assert!(!Address::<BLOCK_SIZE>::is_padding(idx));
        if RESET_INDEX_WHEN_REMOVED {
            (self.index_setter)(&mut self.heap[idx], INVALID_QUEUE_ID);
        }
        if idx + 1 == self.heap.len() {
            self.pop_back();
            return;
        }
        let last = *self
            .heap
            .last()
            .expect("idx + 1 != len(), so len() >= 2 here");
        self.heap[idx] = last;
        (self.index_setter)(&mut self.heap[idx], idx);
        self.pop_back();
        let bottom = self.heap.len() - 1;
        self.update_heap_down(idx, bottom);
        self.update_heap_up(1, idx);
    }

    /// `void update(size_t idx) { assert(! address::is_padding(idx)); T item = m_heap[idx];
    /// remove(idx); push(item); }`
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:281
    pub fn update(&mut self, idx: usize) {
        debug_assert!(!Address::<BLOCK_SIZE>::is_padding(idx));
        let item = self.heap[idx];
        self.remove(idx);
        self.push(item);
    }

    /// ```c++
    /// void update_heap_up(size_t top, size_t bottom)
    /// {
    ///     assert(! address::is_padding(top));
    ///     assert(! address::is_padding(bottom));
    ///     size_t childIdx = bottom;
    ///     T *child = &m_heap[childIdx];
    ///     for (;;) {
    ///         size_t parentIdx = address::parent_of(childIdx);
    ///         if (childIdx == 1 || parentIdx < top)
    ///             break;
    ///         T *parent = &m_heap[parentIdx];
    ///         if (! m_less_predicate(*parent, *child)) {
    ///             T tmp = *parent;
    ///             m_index_setter(tmp,    childIdx);
    ///             m_index_setter(*child, parentIdx);
    ///             m_heap[parentIdx] = *child;
    ///             m_heap[childIdx]  = tmp;
    ///         }
    ///         childIdx = parentIdx;
    ///         child = parent;
    ///     }
    /// }
    /// ```
    /// By-index translation -- see `MutablePriorityQueue::update_heap_up`'s own doc.
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:398
    fn update_heap_up(&mut self, top: usize, bottom: usize) {
        debug_assert!(!Address::<BLOCK_SIZE>::is_padding(top));
        debug_assert!(!Address::<BLOCK_SIZE>::is_padding(bottom));
        let mut child_idx = bottom;
        loop {
            let parent_idx = Address::<BLOCK_SIZE>::parent_of(child_idx);
            if child_idx == 1 || parent_idx < top {
                break;
            }
            if !(self.less_predicate)(&self.heap[parent_idx], &self.heap[child_idx]) {
                let mut tmp = self.heap[parent_idx];
                (self.index_setter)(&mut tmp, child_idx);
                let mut child_val = self.heap[child_idx];
                (self.index_setter)(&mut child_val, parent_idx);
                self.heap[parent_idx] = child_val;
                self.heap[child_idx] = tmp;
            }
            child_idx = parent_idx;
        }
    }

    /// ```c++
    /// void update_heap_down(size_t top, size_t bottom)
    /// {
    ///     assert(! address::is_padding(top));
    ///     assert(! address::is_padding(bottom));
    ///     size_t parentIdx = top;
    ///     T *parent = &m_heap[parentIdx];
    ///     for (;;) {
    ///         size_t childIdx = address::child_of(parentIdx);
    ///         if (childIdx > bottom)
    ///             break;
    ///         T *child = &m_heap[childIdx];
    ///         size_t child2Idx = childIdx + (address::is_block_leaf(parentIdx) ?
    ///         address::block_size : 1);
    ///         if (child2Idx <= bottom) {
    ///             T *child2 = &m_heap[child2Idx];
    ///             if (! m_less_predicate(*child, *child2)) {
    ///                 child = child2;
    ///                 childIdx = child2Idx;
    ///             }
    ///         }
    ///         if (m_less_predicate(*parent, *child))
    ///             return;
    ///         T tmp = *parent;
    ///         m_index_setter(tmp,    childIdx);
    ///         m_index_setter(*child, parentIdx);
    ///         m_heap[parentIdx] = *child;
    ///         m_heap[childIdx] = tmp;
    ///         parentIdx = childIdx;
    ///         parent = child;
    ///     }
    /// }
    /// ```
    /// By-index translation -- see `MutablePriorityQueue::update_heap_down`'s own doc.
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:423
    fn update_heap_down(&mut self, top: usize, bottom: usize) {
        debug_assert!(!Address::<BLOCK_SIZE>::is_padding(top));
        debug_assert!(!Address::<BLOCK_SIZE>::is_padding(bottom));
        let mut parent_idx = top;
        loop {
            let mut child_idx = Address::<BLOCK_SIZE>::child_of(parent_idx);
            if child_idx > bottom {
                break;
            }
            let child2_idx = child_idx
                + if Address::<BLOCK_SIZE>::is_block_leaf(parent_idx) {
                    Address::<BLOCK_SIZE>::BLOCK_SIZE
                } else {
                    1
                };
            if child2_idx <= bottom
                && !(self.less_predicate)(&self.heap[child_idx], &self.heap[child2_idx])
            {
                child_idx = child2_idx;
            }
            if (self.less_predicate)(&self.heap[parent_idx], &self.heap[child_idx]) {
                return;
            }
            let mut tmp = self.heap[parent_idx];
            (self.index_setter)(&mut tmp, child_idx);
            let mut child_val = self.heap[child_idx];
            (self.index_setter)(&mut child_val, parent_idx);
            self.heap[parent_idx] = child_val;
            self.heap[child_idx] = tmp;
            parent_idx = child_idx;
        }
    }
}

impl<
        T,
        IndexSetter,
        LessPredicate,
        const BLOCK_SIZE: usize,
        const RESET_INDEX_WHEN_REMOVED: bool,
    > std::ops::Index<usize>
    for MutableSkipHeapPriorityQueue<
        T,
        IndexSetter,
        LessPredicate,
        BLOCK_SIZE,
        RESET_INDEX_WHEN_REMOVED,
    >
where
    T: Copy + Default,
    IndexSetter: FnMut(&mut T, usize),
    LessPredicate: Fn(&T, &T) -> bool,
{
    type Output = T;
    /// `T& operator[](std::size_t idx) noexcept { assert(! address::is_padding(idx)); return
    /// m_heap[idx]; }` (and its `const` overload)
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:285
    fn index(&self, idx: usize) -> &T {
        debug_assert!(!Address::<BLOCK_SIZE>::is_padding(idx));
        &self.heap[idx]
    }
}

impl<
        T,
        IndexSetter,
        LessPredicate,
        const BLOCK_SIZE: usize,
        const RESET_INDEX_WHEN_REMOVED: bool,
    > std::ops::IndexMut<usize>
    for MutableSkipHeapPriorityQueue<
        T,
        IndexSetter,
        LessPredicate,
        BLOCK_SIZE,
        RESET_INDEX_WHEN_REMOVED,
    >
where
    T: Copy + Default,
    IndexSetter: FnMut(&mut T, usize),
    LessPredicate: Fn(&T, &T) -> bool,
{
    fn index_mut(&mut self, idx: usize) -> &mut T {
        debug_assert!(!Address::<BLOCK_SIZE>::is_padding(idx));
        &mut self.heap[idx]
    }
}

impl<
        T,
        IndexSetter,
        LessPredicate,
        const BLOCK_SIZE: usize,
        const RESET_INDEX_WHEN_REMOVED: bool,
    > Drop
    for MutableSkipHeapPriorityQueue<
        T,
        IndexSetter,
        LessPredicate,
        BLOCK_SIZE,
        RESET_INDEX_WHEN_REMOVED,
    >
where
    T: Copy + Default,
    IndexSetter: FnMut(&mut T, usize),
    LessPredicate: Fn(&T, &T) -> bool,
{
    /// `~MutableSkipHeapPriorityQueue() { clear(); }` -- same reasoning as
    /// `MutablePriorityQueue`'s own `Drop` impl above.
    ///
    /// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:271
    fn drop(&mut self) {
        self.clear();
    }
}

/// `template<typename T, std::size_t BlockSize, const bool ResetIndexWhenRemoved, typename
/// IndexSetter, typename LessPredicate> MutableSkipHeapPriorityQueue<T, IndexSetter,
/// LessPredicate, BlockSize, ResetIndexWhenRemoved> make_miniheap_mutable_priority_queue(...)`
///
/// // PORT SRC: src/libslic3r/MutablePriorityQueue.hpp:305
pub fn make_miniheap_mutable_priority_queue<
    T,
    const BLOCK_SIZE: usize,
    const RESET_INDEX_WHEN_REMOVED: bool,
    IndexSetter,
    LessPredicate,
>(
    index_setter: IndexSetter,
    less_predicate: LessPredicate,
) -> MutableSkipHeapPriorityQueue<T, IndexSetter, LessPredicate, BLOCK_SIZE, RESET_INDEX_WHEN_REMOVED>
where
    T: Copy + Default,
    IndexSetter: FnMut(&mut T, usize),
    LessPredicate: Fn(&T, &T) -> bool,
{
    MutableSkipHeapPriorityQueue::new(index_setter, less_predicate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use slic3r_proof::proof_units::mutable_priority_queue::{
        destructor_marks_base, destructor_marks_skip_heap, push_pop_order, remove_then_pop_order,
        skip_heap_push_pop_order, skip_heap_remove_then_pop_order,
    };

    /// The proof's one shared instantiation, on both the Rust and the C++ side
    /// (`crates/slic3r-ffi/cxx/proof_units/mutable_priority_queue.cpp`'s `ProofItem`): a POD
    /// carrying an `i64` sort key and a `usize` queue index, so this task's proof exercises real
    /// heap-order behaviour rather than a type the vendored template never actually instantiates
    /// with.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct ProofItem {
        key: i64,
        idx: usize,
    }

    /// Hand-written, not `#[derive(Default)]`: the shim's own `ProofItem`
    /// (`crates/slic3r-ffi/cxx/proof_units/mutable_priority_queue.cpp`) declares
    /// `size_t idx = InvalidQueueID;`, so `T()` (the skip heap's padding-slot placeholder,
    /// `MutablePriorityQueue.hpp:333`/`:344`) value-initialises `idx` to `InvalidQueueID` on the
    /// C++ side, not `0` (adversarial review, this task: `#[derive(Default)]` gave `0`, silently
    /// disagreeing with the comment above claiming the two sides mirror each other "exactly").
    /// The padding slot is never read back by any path this module's own differential
    /// exercises, so the two defaults were never observed to diverge, but the claim should be
    /// true regardless of whether anything currently reads it.
    impl Default for ProofItem {
        fn default() -> Self {
            ProofItem {
                key: 0,
                idx: INVALID_QUEUE_ID,
            }
        }
    }

    fn set_idx(item: &mut ProofItem, idx: usize) {
        item.idx = idx;
    }

    fn less(a: &ProofItem, b: &ProofItem) -> bool {
        a.key < b.key
    }

    // A POD carrying a sort key and a stable "slot" (its position in the input array), distinct
    // from `ProofItem` above: the index-setter here does not write into the item itself, it
    // writes into an external `Rc<RefCell<Vec<usize>>>` keyed by `slot` -- the shape that makes
    // `Drop`'s effect observable from outside the queue at all, mirroring the real call shape at
    // `ShortestPath.cpp:791`/`AStar.hpp:91` (an index-setter writing into a field on an object
    // the queue does not own). Mirrors
    // `crates/slic3r-ffi/cxx/proof_units/mutable_priority_queue.cpp`'s own `SlotItem` exactly.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    struct SlotItem {
        key: i64,
        slot: usize,
    }

    fn slot_less(a: &SlotItem, b: &SlotItem) -> bool {
        a.key < b.key
    }

    /// Pushes `keys` (slot `i` <- `keys[i]`) into a `RESET_INDEX_WHEN_REMOVED = true` base
    /// queue, drops the queue (running its `Drop` impl, not an explicit `clear()` call), and
    /// returns the external mark each slot was left with -- the proof that this port's `Drop`
    /// impl, not just its `clear()` method, performs the index-setter writes.
    fn rust_destructor_marks_base(keys: &[i64]) -> Vec<usize> {
        use std::cell::RefCell;
        use std::rc::Rc;

        let marks = Rc::new(RefCell::new(vec![INVALID_QUEUE_ID; keys.len()]));
        {
            let marks_for_setter = Rc::clone(&marks);
            let set_idx = move |item: &mut SlotItem, idx: usize| {
                marks_for_setter.borrow_mut()[item.slot] = idx;
            };
            let mut q = make_mutable_priority_queue::<SlotItem, true, _, _>(set_idx, slot_less);
            for (i, &key) in keys.iter().enumerate() {
                q.push(SlotItem { key, slot: i });
            }
            // `q`'s `Drop` impl runs here, at the end of this block -- not an explicit `clear()`.
        }
        Rc::try_unwrap(marks)
            .expect("the queue that held the only other Rc clone has been dropped")
            .into_inner()
    }

    /// Same as `rust_destructor_marks_base` above, for the skip-heap variant.
    fn rust_destructor_marks_skip_heap(keys: &[i64]) -> Vec<usize> {
        use std::cell::RefCell;
        use std::rc::Rc;

        let marks = Rc::new(RefCell::new(vec![INVALID_QUEUE_ID; keys.len()]));
        {
            let marks_for_setter = Rc::clone(&marks);
            let set_idx = move |item: &mut SlotItem, idx: usize| {
                marks_for_setter.borrow_mut()[item.slot] = idx;
            };
            let mut q =
                make_miniheap_mutable_priority_queue::<SlotItem, DEFAULT_BLOCK_SIZE, true, _, _>(
                    set_idx, slot_less,
                );
            for (i, &key) in keys.iter().enumerate() {
                q.push(SlotItem { key, slot: i });
            }
            // `q`'s `Drop` impl runs here, at the end of this block -- not an explicit `clear()`.
        }
        Rc::try_unwrap(marks)
            .expect("the queue that held the only other Rc clone has been dropped")
            .into_inner()
    }

    fn rust_push_pop_order(keys: &[i64]) -> Vec<i64> {
        let mut q = make_mutable_priority_queue::<ProofItem, false, _, _>(set_idx, less);
        for &key in keys {
            q.push(ProofItem { key, idx: 0 });
        }
        let mut out = Vec::with_capacity(keys.len());
        while !q.empty() {
            out.push(q.top().key);
            q.pop();
        }
        out
    }

    fn rust_skip_heap_push_pop_order(keys: &[i64]) -> Vec<i64> {
        let mut q =
            make_miniheap_mutable_priority_queue::<ProofItem, DEFAULT_BLOCK_SIZE, false, _, _>(
                set_idx, less,
            );
        for &key in keys {
            q.push(ProofItem { key, idx: 0 });
        }
        let mut out = Vec::with_capacity(keys.len());
        while !q.empty() {
            out.push(q.top().key);
            q.pop();
        }
        out
    }

    const KEY_SETS: &[&[i64]] = &[
        &[5, 3, 8, 1, 9, 2, 7, 4, 6, 0],
        &[1],
        &[2, 1],
        &[1, 1, 1, 1],
        &[-5, -3, -8, -1, -9, -2, -7, -4, -6, 0, 100, -100],
        &[0, 0, 0, 1, 1, 1, -1, -1, -1],
        &[64, 63, 62, 61, 60, 59, 58, 57, 56, 55, 54, 53, 52, 51, 50],
    ];

    // Key sets that cross `DEFAULT_BLOCK_SIZE` (32): the skip heap's `is_block_leaf`/
    // `parent_of`/`child_of` have a cross-miniheap branch (`SkipHeapAddressing::parent_of`'s
    // `is_block_root` case, `MutablePriorityQueue.hpp:224`/`:214`) that every `KEY_SETS` entry
    // above is too short to reach -- the longest is 15 elements, and with the padding slot at
    // index 0 that never leaves the first 32-element miniheap. Adversarial review, this task.
    const KEY_SETS_CROSSING_BLOCK_BOUNDARY: &[&[i64]] = &[
        &[
            40, 39, 38, 37, 36, 35, 34, 33, 32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19,
            18, 17, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1,
        ],
        &[
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
        ],
        &[
            17, 3, 29, 41, 2, 38, 11, 25, 6, 33, 20, 1, 35, 14, 27, 9, 40, 4, 22, 31, 8, 19, 36,
            13, 28, 5, 24, 39, 10, 23, 1, 30, 7, 21, 34, 15, 26, 2, 37, 12, 32, 16, 3, 18,
        ],
    ];

    fn all_key_sets() -> impl Iterator<Item = &'static [i64]> {
        KEY_SETS
            .iter()
            .copied()
            .chain(KEY_SETS_CROSSING_BLOCK_BOUNDARY.iter().copied())
    }

    // PROOF: 1
    #[test]
    fn mutable_priority_queue_push_pop_order_matches_oracle() {
        for &keys in KEY_SETS {
            let oracle = push_pop_order(&keys.to_vec());
            assert_eq!(
                rust_push_pop_order(keys),
                oracle,
                "push/pop order for {keys:?}"
            );
        }
    }

    // PROOF: 1
    #[test]
    fn mutable_priority_queue_remove_then_pop_order_matches_oracle() {
        for &keys in KEY_SETS {
            for remove_at in 0..keys.len() {
                let mut q = make_mutable_priority_queue::<ProofItem, false, _, _>(set_idx, less);
                for &key in keys {
                    q.push(ProofItem { key, idx: 0 });
                }
                q.remove(remove_at);
                let mut actual = Vec::with_capacity(keys.len() - 1);
                while !q.empty() {
                    actual.push(q.top().key);
                    q.pop();
                }
                let oracle = remove_then_pop_order(&keys.to_vec(), remove_at);
                assert_eq!(
                    actual, oracle,
                    "remove({remove_at}) then pop order for {keys:?}"
                );
            }
        }
    }

    // PROOF: 1
    #[test]
    fn mutable_skip_heap_priority_queue_push_pop_order_matches_oracle() {
        for keys in all_key_sets() {
            let oracle = skip_heap_push_pop_order(&keys.to_vec());
            assert_eq!(
                rust_skip_heap_push_pop_order(keys),
                oracle,
                "skip-heap push/pop order for {keys:?}"
            );
        }
    }

    // PROOF: 1
    //
    // Adversarial review (this task, third pass): `remove` on the skip heap had no oracle
    // comparison -- it is the only path that reaches `pop_back`'s double-pop padding logic
    // (`MutablePriorityQueue.hpp:291-297`) from a non-top index, and it is what the real call
    // sites (`ShortestPath.cpp`, `AStar.hpp`) hit. `remove_at` skips index `0`, the always-
    // padding slot for every `KEY_SETS` entry (none reaches the 32-element block boundary).
    #[test]
    fn mutable_skip_heap_priority_queue_remove_then_pop_order_matches_oracle() {
        for &keys in KEY_SETS {
            for remove_at in 1..=keys.len() {
                let mut q = make_miniheap_mutable_priority_queue::<
                    ProofItem,
                    DEFAULT_BLOCK_SIZE,
                    false,
                    _,
                    _,
                >(set_idx, less);
                for &key in keys {
                    q.push(ProofItem { key, idx: 0 });
                }
                q.remove(remove_at);
                let mut actual = Vec::with_capacity(keys.len() - 1);
                while !q.empty() {
                    actual.push(q.top().key);
                    q.pop();
                }
                let oracle = skip_heap_remove_then_pop_order(&keys.to_vec(), remove_at);
                assert_eq!(
                    actual, oracle,
                    "skip-heap remove({remove_at}) then pop order for {keys:?}"
                );
            }
        }
    }

    // PROOF: 1
    #[test]
    fn mutable_priority_queue_destructor_marks_match_oracle() {
        for keys in all_key_sets() {
            // `as i64`: the bridge carries a mark as a plain `i64` (decision 1,
            // `docs/PORTING.md`), never a `usize` -- `InvalidQueueID` (`usize::MAX`) crosses as
            // the C++ side's `static_cast<int64_t>(SIZE_MAX)`, which is `-1` on this (LP64,
            // two's-complement) target, the same reinterpretation this cast performs on the Rust
            // side, so the two sides compare on the same bits.
            let actual: Vec<i64> = rust_destructor_marks_base(keys)
                .into_iter()
                .map(|mark| mark as i64)
                .collect();
            let oracle = destructor_marks_base(&keys.to_vec());
            assert_eq!(actual, oracle, "base queue destructor marks for {keys:?}");
        }
    }

    // PROOF: 1
    #[test]
    fn mutable_skip_heap_priority_queue_destructor_marks_match_oracle() {
        for keys in all_key_sets() {
            let actual: Vec<i64> = rust_destructor_marks_skip_heap(keys)
                .into_iter()
                .map(|mark| mark as i64)
                .collect();
            let oracle = destructor_marks_skip_heap(&keys.to_vec());
            assert_eq!(actual, oracle, "skip-heap destructor marks for {keys:?}");
        }
    }
}

// PORT STATUS
// source: src/libslic3r/MutablePriorityQueue.hpp
// confidence: high
// tier: exact
// oracle: built
