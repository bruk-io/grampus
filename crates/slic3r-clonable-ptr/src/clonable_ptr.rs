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

// PORT STATUS
// source: src/libslic3r/clonable_ptr.hpp
// confidence: medium
// tier: exact
// oracle: built
