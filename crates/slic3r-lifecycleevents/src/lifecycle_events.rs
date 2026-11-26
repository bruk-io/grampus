//! Port of `LifecycleEvents.hpp` (task 53-foundation-int128-queue, plan step 20): application
//! lifecycle events (project, slicing, plate editing, preset, printer connection and job
//! activity) that other subsystems -- chiefly the plugin layer above libslic3r -- may observe.
//! Lives in its own foundation crate rather than the plugin layer because some events fire from
//! inside the slicing engine itself (`fire_lifecycle_event` below).
//!
//! **Proof path 2, and why this unit's proof is a genuine differential rather than a
//! transcription-consistency check.** The manifest row's `pipeline stage` cell is `psWipeTower`,
//! inventory-derived metadata about where this file sits relative to the pipeline, not a claim
//! that `fire_lifecycle_event` is called from `WipeTower.cpp` -- it is not: call sites in
//! `libslic3r` are `Print.cpp` (`SliceStarted`, `SliceGeometryFinished`,
//! `GCodeExportStarted`/`GCodeExportFinished`), `GCode.cpp`
//! (`GCodeExportStarted`/`GCodeExportFinished`) and `Preset.cpp:3593` (`PresetSaved`). So no
//! committed stage dump carries any lifecycle-event output to differential-compare against --
//! there is nothing in `port/dumps/*/psWipeTower.dump` this module's own behaviour could diverge
//! from, and an earlier version of this module's proof compared its own output only against a
//! hand-transcribed string table, which adversarial review (this task) flagged as tautological.
//!
//! `docs/PORTING.md`'s "Decisions recorded on 2026-11-25" decision 4 names the oracle's own
//! entry points as "the per-unit proof bridges `slic3r-proof` re-exports ... plus the committed
//! stage dumps" -- not restricted to a row whose `proof path` cell says `1` or `3` -- so this
//! row's `// PROOF: 2` marker (matching its manifest cell, verbatim, as `port/port-status`'s
//! `proof_present` requires) is proven below through `slic3r_proof::proof_units::
//! lifecycle_events`, a new bridge this task adds for exactly this unit
//! (`crates/slic3r-ffi/cxx/proof_units/lifecycle_events.{h,cpp}`,
//! `crates/slic3r-ffi/src/proof_units/lifecycle_events.rs`): a real comparison of this module's
//! `lifecycle_event_to_string`/`lifecycle_evt_code_to_string` against the vendored header's own
//! `switch` statements, called fresh through `cxx` rather than against a literal this test
//! transcribed by hand. `port/manifest.tsv` itself is on this task's must-not-write list, so the
//! row's `proof path` cell stays `2`; only the mechanism behind that marker changed.

use std::sync::{Arc, Condvar, LazyLock, Mutex};

/// `enum class LifecycleEvent { ... };` -- variant order and therefore discriminant order
/// (R14's type map: "an enum keeping the original's discriminant values") matches the vendored
/// declaration exactly; neither side assigns an explicit value, so both default to 0, 1, 2, ...
/// in declaration order.
///
/// // PORT SRC: src/libslic3r/LifecycleEvents.hpp:19
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LifecycleEvent {
    // Project (3mf)
    NewProject,
    ProjectOpened,
    ProjectBeforeSave,
    ProjectAfterSave,
    ProjectClosed,
    ProjectDirtyChanged,

    // Slicing pipeline
    SliceStarted,
    SliceGeometryFinished,
    GCodeExportStarted,
    GCodeExportFinished,
    SlicingJobComplete,

    // Plate/model editing
    ObjectAdded,
    ObjectDeleted,
    ObjectTransformed,
    ObjectChanged,
    ObjectRenamed,
    PlateCreated,
    PlateDeleted,
    PlateSelected,
    PlateRenamed,

    // Preset
    PresetSelected,
    PresetSaved,

    // Printer/device
    PrintStateChanged,
    DeviceOnline,
    DeviceOffline,
    DeviceDiscovered,
    DeviceSelected,
    UploadStarted,
    UploadFinished,

    // Print/send jobs
    PrintJobStarted,
    PrintJobFinished,
    SendJobStarted,
    SendJobFinished,
}

/// `enum class LifecycleEvtCode { Ok, Error, Warn };` -- the vendored comment about `ERROR`/`OK`
/// colliding with Windows macros (`wingdi.h`) is a C++-only concern with no Rust analogue, so it
/// is not transcribed as a marker; the three variants and their order are.
///
/// // PORT SRC: src/libslic3r/LifecycleEvents.hpp:68
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LifecycleEvtCode {
    Ok,
    Error,
    Warn,
}

/// `struct LifecycleEventContext { ... };`
///
/// // PORT SRC: src/libslic3r/LifecycleEvents.hpp:70
#[derive(Clone)]
pub struct LifecycleEventContext {
    /// The primary subject identifier for the event (a project/output path, preset name, device
    /// id, or object name); event-specific, never status or prose, and may be empty when the
    /// event has no single subject.
    pub name: String,
    /// Outcome of the operation this event represents. For state-change and start events, `Ok`
    /// means the event occurred, not that a future operation succeeded.
    pub code: LifecycleEvtCode,
    /// Optional human-readable detail or diagnostic text; not a stable parsing contract.
    pub msg: String,
    /// Stable subject/object identifier, when the source model provides one.
    pub id: String,
    /// Previous value for rename and other before/after events.
    pub previous_name: String,
    /// Device identifier for printer and job events.
    pub device_id: String,
    /// Job identifier when the originating queue/task provides one.
    pub job_id: String,
    /// Source subsystem or operation detail, suitable for filtering but not guaranteed
    /// exhaustive across versions.
    pub source: String,
    /// Plate, object, or volume index when the source uses an index rather than a stable id.
    pub index: i32,
    /// Aggregate project dirty state for `ProjectDirtyChanged`.
    pub dirty: bool,
    /// Optional host-side cancellation probe. Background slicing and G-code export events set
    /// this to the originating `Print`'s cancellation state so dispatch can stop before calling
    /// the next capability. `Arc`, not a bare closure type, so this context stays `Clone` the
    /// same way the vendored `std::function` makes the C++ struct copyable.
    pub cancellation_check: Option<Arc<dyn Fn() -> bool + Send + Sync>>,
}

impl Default for LifecycleEventContext {
    /// The vendored struct's own in-class default member initializers: `code = Ok`, `index = -1`,
    /// `dirty = false`, every `std::string` field empty, `cancellation_check` (a default-
    /// constructed `std::function`) empty.
    ///
    /// // PORT SRC: src/libslic3r/LifecycleEvents.hpp:102
    fn default() -> Self {
        LifecycleEventContext {
            name: String::new(),
            code: LifecycleEvtCode::Ok,
            msg: String::new(),
            id: String::new(),
            previous_name: String::new(),
            device_id: String::new(),
            job_id: String::new(),
            source: String::new(),
            index: -1,
            dirty: false,
            cancellation_check: None,
        }
    }
}

/// ```c++
/// inline std::string lifecycle_event_to_string(LifecycleEvent event)
/// {
///     switch (event) {
///     case LifecycleEvent::NewProject: return "NewProject";
///     ...
///     default: return "Unknown";
///     }
/// }
/// ```
/// Transcribed verbatim, one arm per vendored `case`, in the same order; Rust's exhaustiveness
/// check makes the vendored `default:` arm unreachable here (every variant has its own arm), so
/// it is not ported as a `_ =>` catch-all -- there is no way to construct a `LifecycleEvent` this
/// match does not already cover.
///
/// // PORT SRC: src/libslic3r/LifecycleEvents.hpp:113
pub fn lifecycle_event_to_string(event: LifecycleEvent) -> &'static str {
    match event {
        LifecycleEvent::NewProject => "NewProject",
        LifecycleEvent::ProjectOpened => "ProjectOpened",
        LifecycleEvent::ProjectBeforeSave => "ProjectBeforeSave",
        LifecycleEvent::ProjectAfterSave => "ProjectAfterSave",
        LifecycleEvent::ProjectClosed => "ProjectClosed",
        LifecycleEvent::ProjectDirtyChanged => "ProjectDirtyChanged",

        LifecycleEvent::SliceStarted => "SliceStarted",
        LifecycleEvent::SliceGeometryFinished => "SliceGeometryFinished",
        LifecycleEvent::GCodeExportStarted => "GCodeExportStarted",
        LifecycleEvent::GCodeExportFinished => "GCodeExportFinished",
        LifecycleEvent::SlicingJobComplete => "SlicingJobComplete",

        LifecycleEvent::ObjectAdded => "ObjectAdded",
        LifecycleEvent::ObjectDeleted => "ObjectDeleted",
        LifecycleEvent::ObjectTransformed => "ObjectTransformed",
        LifecycleEvent::ObjectChanged => "ObjectChanged",
        LifecycleEvent::ObjectRenamed => "ObjectRenamed",
        LifecycleEvent::PlateCreated => "PlateCreated",
        LifecycleEvent::PlateDeleted => "PlateDeleted",
        LifecycleEvent::PlateSelected => "PlateSelected",
        LifecycleEvent::PlateRenamed => "PlateRenamed",

        LifecycleEvent::PresetSelected => "PresetSelected",
        LifecycleEvent::PresetSaved => "PresetSaved",
        LifecycleEvent::PrintStateChanged => "PrintStateChanged",

        LifecycleEvent::DeviceOnline => "DeviceOnline",
        LifecycleEvent::DeviceOffline => "DeviceOffline",
        LifecycleEvent::DeviceDiscovered => "DeviceDiscovered",
        LifecycleEvent::DeviceSelected => "DeviceSelected",

        LifecycleEvent::UploadStarted => "UploadStarted",
        LifecycleEvent::UploadFinished => "UploadFinished",
        LifecycleEvent::PrintJobStarted => "PrintJobStarted",
        LifecycleEvent::PrintJobFinished => "PrintJobFinished",
        LifecycleEvent::SendJobStarted => "SendJobStarted",
        LifecycleEvent::SendJobFinished => "SendJobFinished",
    }
}

/// ```c++
/// inline std::string lifecycle_evt_code_to_string(LifecycleEvtCode code)
/// {
///     switch (code) {
///     case LifecycleEvtCode::Ok: return "Ok";
///     case LifecycleEvtCode::Error: return "Error";
///     case LifecycleEvtCode::Warn: return "Warn";
///     default: return "Unknown";
///     }
/// }
/// ```
/// Same exhaustiveness note as `lifecycle_event_to_string` above.
///
/// // PORT SRC: src/libslic3r/LifecycleEvents.hpp:158
pub fn lifecycle_evt_code_to_string(code: LifecycleEvtCode) -> &'static str {
    match code {
        LifecycleEvtCode::Ok => "Ok",
        LifecycleEvtCode::Error => "Error",
        LifecycleEvtCode::Warn => "Warn",
    }
}

/// `using LifecycleHookFn = std::function<void(LifecycleEvent, const LifecycleEventContext&)>;`
/// -- `Arc`, not `Box`, so `fire_lifecycle_event` below can cheaply clone the installed hook out
/// of the mutex before calling it without holding the lock, the same way the vendored
/// `fn = state.fn;` copies the (copyable) `std::function`.
///
/// // PORT SRC: src/libslic3r/LifecycleEvents.hpp:172
pub type LifecycleHookFn = Arc<dyn Fn(LifecycleEvent, &LifecycleEventContext) + Send + Sync>;

/// `struct LifecycleHookState { std::mutex mutex; std::condition_variable cv; LifecycleHookFn fn;
/// std::size_t active_dispatches = 0; bool accepting = false; };` -- the mutex and condition
/// variable become `Mutex`/`Condvar` at the cell level (`LifecycleHookCell` below) rather than
/// fields of this struct, since Rust's `Mutex<T>` owns the data it guards instead of sitting
/// beside it; `fn`, `active_dispatches` and `accepting` are the state the mutex actually guards.
///
/// // PORT SRC: src/libslic3r/LifecycleEvents.hpp:176
struct LifecycleHookState {
    hook_fn: Option<LifecycleHookFn>,
    active_dispatches: usize,
    accepting: bool,
}

/// The `Mutex`/`Condvar` pair every `LifecycleHookState` access goes through -- see that struct's
/// own doc for why they sit here instead of inside it.
struct LifecycleHookCell {
    state: Mutex<LifecycleHookState>,
    cv: Condvar,
}

/// `inline LifecycleHookState& lifecycle_hook_state() { static LifecycleHookState state; return
/// state; }` -- a function-local `static` in C++, ported as a `static` backed by `LazyLock`
/// (Rust has no function-local `static` with non-trivial initialisation); both give exactly one
/// instance for the whole process, initialised on first use.
///
/// // PORT SRC: src/libslic3r/LifecycleEvents.hpp:185
static LIFECYCLE_HOOK_STATE: LazyLock<LifecycleHookCell> = LazyLock::new(|| LifecycleHookCell {
    state: Mutex::new(LifecycleHookState {
        hook_fn: None,
        active_dispatches: 0,
        accepting: false,
    }),
    cv: Condvar::new(),
});

fn lifecycle_hook_state() -> &'static LifecycleHookCell {
    &LIFECYCLE_HOOK_STATE
}

/// ```c++
/// class LifecycleDispatchGuard
/// {
/// public:
///     explicit LifecycleDispatchGuard(LifecycleHookState& state) : m_state(state) {}
///     ~LifecycleDispatchGuard()
///     {
///         std::lock_guard<std::mutex> lock(m_state.mutex);
///         --m_state.active_dispatches;
///         if (m_state.active_dispatches == 0)
///             m_state.cv.notify_all();
///     }
///     LifecycleDispatchGuard(const LifecycleDispatchGuard&) = delete;
///     LifecycleDispatchGuard& operator=(const LifecycleDispatchGuard&) = delete;
/// private:
///     LifecycleHookState& m_state;
/// };
/// ```
/// `#[derive(..)] Clone`/`Copy` are not derived (matching the deleted copy constructor/assignment
/// above), and there is no `Default`/manual `Clone` impl either -- a `&'static LifecycleHookCell`
/// field alone does not forbid deriving `Clone`, so the absence is the deliberate choice here,
/// not an oversight: a cloned guard would double-decrement `active_dispatches` on two `Drop`s for
/// one dispatch, exactly what the vendored type's deleted copy members prevent.
///
/// // PORT SRC: src/libslic3r/LifecycleEvents.hpp:191
struct LifecycleDispatchGuard {
    cell: &'static LifecycleHookCell,
}

impl LifecycleDispatchGuard {
    fn new(cell: &'static LifecycleHookCell) -> Self {
        LifecycleDispatchGuard { cell }
    }
}

impl Drop for LifecycleDispatchGuard {
    fn drop(&mut self) {
        let mut state = self
            .cell
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.active_dispatches -= 1;
        if state.active_dispatches == 0 {
            self.cell.cv.notify_all();
        }
    }
}

/// ```c++
/// inline void set_lifecycle_hook_fn(LifecycleHookFn fn)
/// {
///     detail::LifecycleHookState& state = detail::lifecycle_hook_state();
///     if (fn) {
///         std::lock_guard<std::mutex> lock(state.mutex);
///         state.fn        = std::move(fn);
///         state.accepting = true;
///         return;
///     }
///     std::unique_lock<std::mutex> lock(state.mutex);
///     state.accepting = false;
///     state.fn        = nullptr;
///     state.cv.wait(lock, [&state] { return state.active_dispatches == 0; });
/// }
/// ```
/// `Option<LifecycleHookFn>` renders the vendored "empty `std::function` means uninstall"
/// convention explicitly: `Some(fn)` is the installing call, `None` the uninstalling one, rather
/// than relying on a callable's own truthiness.
///
/// // PORT SRC: src/libslic3r/LifecycleEvents.hpp:218
pub fn set_lifecycle_hook_fn(hook_fn: Option<LifecycleHookFn>) {
    let cell = lifecycle_hook_state();
    if let Some(hook_fn) = hook_fn {
        let mut state = cell
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.hook_fn = Some(hook_fn);
        state.accepting = true;
        return;
    }
    let mut state = cell
        .state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    state.accepting = false;
    state.hook_fn = None;
    let _state = cell
        .cv
        .wait_while(state, |s| s.active_dispatches != 0)
        .unwrap_or_else(std::sync::PoisonError::into_inner);
}

/// ```c++
/// inline void fire_lifecycle_event(LifecycleEvent event, const LifecycleEventContext& ctx)
/// {
///     detail::LifecycleHookState& state = detail::lifecycle_hook_state();
///     LifecycleHookFn             fn;
///     {
///         std::lock_guard<std::mutex> lock(state.mutex);
///         if (!state.accepting || !state.fn)
///             return;
///         fn = state.fn;
///         ++state.active_dispatches;
///     }
///     detail::LifecycleDispatchGuard guard(state);
///     fn(event, ctx);
/// }
/// ```
///
/// // PORT SRC: src/libslic3r/LifecycleEvents.hpp:234
pub fn fire_lifecycle_event(event: LifecycleEvent, ctx: &LifecycleEventContext) {
    let cell = lifecycle_hook_state();
    let hook_fn: LifecycleHookFn;
    {
        let mut state = cell
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !state.accepting || state.hook_fn.is_none() {
            return;
        }
        hook_fn = state.hook_fn.clone().expect("just checked is_none() above");
        state.active_dispatches += 1;
    }

    let _guard = LifecycleDispatchGuard::new(cell);
    hook_fn(event, ctx);
}

#[cfg(test)]
mod tests {
    use super::*;
    use slic3r_proof::dumps;
    use slic3r_proof::proof_units::lifecycle_events::{
        oracle_lifecycle_event_to_string, oracle_lifecycle_evt_code_to_string,
    };

    /// Every `LifecycleEvent` variant, in the same order the header declares the enum -- the
    /// ordinal each cast to `i32` below produces is this declaration order (see the module doc
    /// for why that is also the discriminant order the vendored `enum class` uses).
    const EVENTS: &[LifecycleEvent] = &[
        LifecycleEvent::NewProject,
        LifecycleEvent::ProjectOpened,
        LifecycleEvent::ProjectBeforeSave,
        LifecycleEvent::ProjectAfterSave,
        LifecycleEvent::ProjectClosed,
        LifecycleEvent::ProjectDirtyChanged,
        LifecycleEvent::SliceStarted,
        LifecycleEvent::SliceGeometryFinished,
        LifecycleEvent::GCodeExportStarted,
        LifecycleEvent::GCodeExportFinished,
        LifecycleEvent::SlicingJobComplete,
        LifecycleEvent::ObjectAdded,
        LifecycleEvent::ObjectDeleted,
        LifecycleEvent::ObjectTransformed,
        LifecycleEvent::ObjectChanged,
        LifecycleEvent::ObjectRenamed,
        LifecycleEvent::PlateCreated,
        LifecycleEvent::PlateDeleted,
        LifecycleEvent::PlateSelected,
        LifecycleEvent::PlateRenamed,
        LifecycleEvent::PresetSelected,
        LifecycleEvent::PresetSaved,
        LifecycleEvent::PrintStateChanged,
        LifecycleEvent::DeviceOnline,
        LifecycleEvent::DeviceOffline,
        LifecycleEvent::DeviceDiscovered,
        LifecycleEvent::DeviceSelected,
        LifecycleEvent::UploadStarted,
        LifecycleEvent::UploadFinished,
        LifecycleEvent::PrintJobStarted,
        LifecycleEvent::PrintJobFinished,
        LifecycleEvent::SendJobStarted,
        LifecycleEvent::SendJobFinished,
    ];

    // PROOF: 2
    #[test]
    fn lifecycle_events_strings_match_the_oracle() {
        // A genuine differential: `oracle_lifecycle_event_to_string`/
        // `oracle_lifecycle_evt_code_to_string` call the vendored header's own `switch`
        // statements fresh, through `crates/slic3r-ffi/cxx/proof_units/lifecycle_events.cpp`,
        // compared against this port's own `lifecycle_event_to_string`/
        // `lifecycle_evt_code_to_string` -- not against a string this test transcribed by hand.
        // The enum casts to `i32` directly (no explicit discriminants on either side -- see the
        // module doc), which is the ordinal the bridge's `static_cast` reads back on the C++
        // side.
        for &event in EVENTS {
            let ordinal = event as i32;
            assert_eq!(
                lifecycle_event_to_string(event),
                oracle_lifecycle_event_to_string(ordinal),
                "lifecycle_event_to_string({event:?})"
            );
        }

        for &code in &[
            LifecycleEvtCode::Ok,
            LifecycleEvtCode::Error,
            LifecycleEvtCode::Warn,
        ] {
            let ordinal = code as i32;
            assert_eq!(
                lifecycle_evt_code_to_string(code),
                oracle_lifecycle_evt_code_to_string(ordinal),
                "lifecycle_evt_code_to_string({code:?})"
            );
        }
    }

    /// Not `// PROOF`-marked: a schema check on the committed dump format itself (its
    /// provenance header is enforced against the pin by `dumps::read_dump_default`), kept as an
    /// ordinary test because the module doc explains why no field of this dump carries
    /// lifecycle-event output for `lifecycle_events_strings_match_the_oracle` above to use.
    #[test]
    fn psw_wipe_tower_dump_reads_and_names_its_own_stage() {
        let dump = dumps::read_dump_default("20mm_cube-base-fff", "psWipeTower")
            .expect("port/dumps/20mm_cube-base-fff/psWipeTower.dump must read");
        assert_eq!(dump.seam, "psWipeTower");
    }

    #[test]
    fn lifecycle_event_context_default_matches_vendored_initializers() {
        let ctx = LifecycleEventContext::default();
        assert_eq!(ctx.name, "");
        assert_eq!(ctx.code, LifecycleEvtCode::Ok);
        assert_eq!(ctx.msg, "");
        assert_eq!(ctx.id, "");
        assert_eq!(ctx.previous_name, "");
        assert_eq!(ctx.device_id, "");
        assert_eq!(ctx.job_id, "");
        assert_eq!(ctx.source, "");
        assert_eq!(ctx.index, -1);
        assert!(!ctx.dirty);
        assert!(ctx.cancellation_check.is_none());
    }

    #[test]
    fn fire_lifecycle_event_dispatches_only_while_accepting() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let calls = Arc::new(AtomicUsize::new(0));
        let ctx = LifecycleEventContext {
            name: "unit-test".to_string(),
            ..LifecycleEventContext::default()
        };

        // Not accepting yet (no hook installed): fire_lifecycle_event is a silent no-op.
        fire_lifecycle_event(LifecycleEvent::SliceStarted, &ctx);
        assert_eq!(calls.load(Ordering::SeqCst), 0);

        let calls_in_hook = Arc::clone(&calls);
        set_lifecycle_hook_fn(Some(Arc::new(move |_event, _ctx| {
            calls_in_hook.fetch_add(1, Ordering::SeqCst);
        })));
        fire_lifecycle_event(LifecycleEvent::SliceStarted, &ctx);
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        // Uninstalling (None) stops acceptance; a later fire is again a no-op.
        set_lifecycle_hook_fn(None);
        fire_lifecycle_event(LifecycleEvent::SliceGeometryFinished, &ctx);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}

// PORT STATUS
// source: src/libslic3r/LifecycleEvents.hpp
// confidence: high
// tier: exact
// oracle: built
