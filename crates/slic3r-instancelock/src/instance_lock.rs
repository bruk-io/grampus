//! Port of `InstanceLock.cpp`/`InstanceLock.hpp`
//! (`vendor/OrcaSlicer/src/libslic3r/InstanceLock.{cpp,hpp}`): a scoped write lock on a file
//! shared by every running instance of the application, serialising threads of this process
//! through a reentrant lock and other processes through an advisory OS file lock
//! (`flock(2)` -- this port targets macOS/Linux, never the vendored `_WIN32` arm).
//!
//! **The throw/non-const-ref-out-param leg of this task's trial**
//! (`intent/libslic3r-rust-port/tasks/35-trial-run/task.md`). `open_lock_file`/`defer`
//! (`InstanceLock.hpp:52-53`) are non-const-reference out-params on the vendored source's
//! private `Slot&`; the idiom map renders a non-const reference out-parameter as a return
//! value, never a `&mut` parameter adopted to mirror the signature -- here as `&mut SlotState`
//! reached through the reentrant guard itself, the natural Rust shape for "mutate the state
//! this lock protects" (see [`SlotState`]). `NativeFileLock`'s constructor and `try_lock()`
//! throw `std::system_error` (`InstanceLock.cpp:42`, `:52`); `open_lock_file` *catches* both
//! (`:101-120`) and converts them into `false` plus a cool-down -- the error never escapes the
//! unit, so the port's error-set rule here is its sharper mirror: [`open_lock_file`] (below)
//! must not widen that caught, swallowed error into a surfaced `Result` either, and it does
//! not -- it returns a plain `bool`, exactly as the vendored function does.
//!
//! **What this module does not port.** `BOOST_LOG_TRIVIAL` calls (`InstanceLock.cpp:70`, `:148`,
//! `:157`) are a logging side effect with no return value and no port crate's logging facility
//! to call into yet; `defer`'s `reason` parameter is accepted and ignored for the same reason a
//! later task, once a logging foundation exists, would wire in. This is recorded, not silently
//! dropped: every `defer` call site below still names the reason that would be logged.
//!
//! **Ownership.** `m_slot` (`InstanceLock.hpp:55`, a raw `Slot*`) is `port/ownership.tsv`'s
//! `borrowed` row, Rust type `&'static Slot` (the registry never frees a slot). This module
//! reaches that same borrow through [`InstanceLockGuard`], which holds the reentrant lock (and
//! therefore the `&'static Slot` it was taken from) for the whole lifetime of an `InstanceLock`
//! -- the vendored `m_slot_guard` field's own job (`InstanceLock.hpp:56`) -- rather than a
//! second, redundant `&'static Slot` field alongside it.

use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::io;
use std::ops::{Deref, DerefMut};
use std::os::unix::io::AsRawFd;
use std::sync::{Condvar, Mutex, OnceLock};
use std::thread::{self, ThreadId};
use std::time::{Duration, Instant};

// SAFETY: this is the declaration of `flock(2)`'s own C signature, not a hand-written unsafe
// operation; every call site below comments the invariant it relies on. libc is always linked
// on the platforms `just oracle-build` targets (macOS/Linux), so no new dependency is added.
extern "C" {
    fn flock(fd: i32, operation: i32) -> i32;
}

/// `flock(2)`'s operation flags (POSIX-standardised values, not platform-specific the way
/// `O_CREAT`'s numeric value is -- `std::fs::OpenOptions` is used for the open call for exactly
/// that reason, below). The vendored source writes the symbolic macro names
/// (`LOCK_EX | LOCK_NB`, `InstanceLock.cpp:44`; `LOCK_UN`, `:43`), never these numbers, so there
/// is no vendored line to cite -- each gets the "constant whose origin is outside the vendored
/// tree" marker form instead (`docs/PORTING.md`, "Decisions recorded on 2026-11-25", decision 2).
const LOCK_EX: i32 = 2;
// PORT SRC: external: POSIX.1-2017 (IEEE Std 1003.1-2017) <sys/file.h> -- LOCK_NB, the
// non-blocking flag bit for flock(2), value 4 on every platform `just oracle-build` targets
const LOCK_NB: i32 = 4;
// PORT SRC: external: POSIX.1-2017 (IEEE Std 1003.1-2017) <sys/file.h> -- LOCK_UN, the unlock
// operation for flock(2), value 8 on every platform `just oracle-build` targets
const LOCK_UN: i32 = 8;

/// `flock(2)` rather than an `fcntl` lock: it belongs to this open file description, so any
/// other code in the process that opens and closes the lock file (a backup or an export walking
/// the data dir) cannot drop it (`InstanceLock.cpp:32-36`).
struct NativeFileLock {
    file: std::fs::File,
}

impl NativeFileLock {
    /// `NativeFileLock(const char *path)` (`InstanceLock.cpp:37-41`): `::open(path, O_RDWR |
    /// O_CREAT | O_CLOEXEC, 0644)`, throwing `std::system_error` on failure. `OpenOptions` is
    /// used instead of a hand-rolled `open(2)` call so this never hardcodes `O_CREAT`'s
    /// platform-specific numeric value; `std::fs::File` on Unix already sets close-on-exec the
    /// same way `O_CLOEXEC` does.
    fn new(path: &str) -> io::Result<Self> {
        use std::os::unix::fs::OpenOptionsExt;
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            // PORT SRC: external: matches src/libslic3r/InstanceLock.cpp:39's literal `0644`
            // (C octal notation) -- the same permission value, written in Rust's `0o` octal
            // syntax, so the digits do not appear verbatim on the cited line for a plain lookup
            .mode(0o644)
            .open(path)?;
        Ok(NativeFileLock { file })
    }

    /// `try_lock()` (`InstanceLock.cpp:44-50`): `flock(m_fd, LOCK_EX | LOCK_NB)`. `EWOULDBLOCK`
    /// and `EINTR` both mean "could not take it right now, caller polls again" (`:48-50`); every
    /// other errno is the unit's other throw path (`:52`), surfaced here as `Err` instead.
    fn try_lock(&self) -> io::Result<bool> {
        // SAFETY: `self.file`'s descriptor is open for the lifetime of `self`, and `flock` does
        // not retain it past the call.
        let rc = unsafe { flock(self.file.as_raw_fd(), LOCK_EX | LOCK_NB) };
        if rc == 0 {
            return Ok(true);
        }
        let err = io::Error::last_os_error();
        match err.kind() {
            io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted => Ok(false),
            _ => Err(err),
        }
    }

    /// `~NativeFileLock()`/`unlock()` (`InstanceLock.cpp:43`): releases the advisory lock.
    fn unlock(&self) {
        // SAFETY: same descriptor-lifetime invariant as `try_lock`.
        unsafe {
            flock(self.file.as_raw_fd(), LOCK_UN);
        }
    }
}

/// How long a guard waits for the file lock before giving up and starting the cool-down
/// (`InstanceLock.hpp:34`, `default_timeout`).
// PORT SRC: src/libslic3r/InstanceLock.hpp:34
pub const DEFAULT_TIMEOUT: Duration = Duration::from_millis(1000);
/// `cooldown` (`InstanceLock.hpp:37`): for this long after a failed open/lock attempt, guards
/// leave the file alone.
// PORT SRC: src/libslic3r/InstanceLock.hpp:37
const COOLDOWN: Duration = Duration::from_millis(10_000);

/// `InstanceLock::Slot` (`InstanceLock.cpp:70-79`): one per lock file path, shared by every
/// guard in the process. `reentrant` is `std::recursive_mutex mutex` (`InstanceLock.hpp:52`'s
/// `std::unique_lock<std::recursive_mutex> m_slot_guard` is what an `InstanceLock` holds for its
/// whole lifetime); `state` is everything the vendored `Slot` struct otherwise carries.
struct Slot {
    reentrant: ReentrantMutex<SlotState>,
}

/// The mutable fields of the vendored `Slot` (`InstanceLock.cpp:70-79`), guarded together by
/// `Slot::reentrant` exactly as the vendored source guards them together by `Slot::mutex`.
struct SlotState {
    /// Non-null exactly while this process holds the file lock (`std::unique_ptr<NativeFileLock>
    /// file_lock`).
    file_lock: Option<NativeFileLock>,
    depth: u32,
    cooldown_until: Instant,
}

impl SlotState {
    /// A time in the past relative to any later `Instant::now()` read (the monotonic clock
    /// never runs backwards), reproducing the vendored `time_point{}` default's "no cool-down
    /// has ever been set yet" without assuming `Instant` has a portable zero/epoch to construct.
    fn new() -> Self {
        SlotState {
            file_lock: None,
            depth: 0,
            cooldown_until: Instant::now(),
        }
    }
}

/// `InstanceLock::slot_for()` (`InstanceLock.cpp:81-90`): never freed, so every `&'static Slot`
/// this returns stays valid for the process's life -- `port/ownership.tsv`'s classification for
/// the vendored `m_slot` field.
fn slot_for(lock_file_path: &str) -> &'static Slot {
    static REGISTRY: OnceLock<Mutex<HashMap<String, &'static Slot>>> = OnceLock::new();
    let registry = REGISTRY.get_or_init(|| Mutex::new(HashMap::new()));
    let mut registry = registry
        .lock()
        .expect("the slot registry mutex is never poisoned");
    if let Some(&slot) = registry.get(lock_file_path) {
        return slot;
    }
    let slot: &'static Slot = Box::leak(Box::new(Slot {
        reentrant: ReentrantMutex::new(SlotState::new()),
    }));
    registry.insert(lock_file_path.to_string(), slot);
    slot
}

/// `InstanceLock::defer()` (`InstanceLock.cpp:91-95`): starts the cool-down. Called with the
/// slot's reentrant lock held, exactly as the vendored function requires its caller to hold
/// `slot`'s mutex. `reason` is accepted (and named at every call site below) but not logged --
/// see this module's own doc comment for why.
fn defer(state: &mut SlotState, _reason: &str) {
    state.cooldown_until = Instant::now() + COOLDOWN;
}

/// `InstanceLock::open_lock_file()` (`InstanceLock.cpp:98-120`): creates the lock file if needed
/// and opens it, or starts the cool-down. Called with the slot's reentrant lock held. Returns
/// `true`/writes `state.file_lock` the same way the vendored out-param/return pair does; the
/// `NativeFileLock::new` error this catches never escapes as a `Result`, matching the vendored
/// function's own `catch (const std::exception &e)` (`:115-119`).
fn open_lock_file(state: &mut SlotState, lock_file_path: &str) -> bool {
    match NativeFileLock::new(lock_file_path) {
        Ok(lock) => {
            state.file_lock = Some(lock);
            true
        }
        Err(e) => {
            defer(
                state,
                &format!(
                    "Cannot open lock file {lock_file_path}: {e} (check its owner and permissions)"
                ),
            );
            false
        }
    }
}

/// `Slic3r::InstanceLock` (`InstanceLock.hpp:28-56`).
pub struct InstanceLock {
    /// `m_slot_guard` (`InstanceLock.hpp:56`): held for this guard's whole lifetime, released
    /// only on `Drop` -- see this module's own doc comment on why this is the one field a
    /// faithful port needs, carrying `m_slot`'s own `&'static Slot` borrow inside it. `None`
    /// exactly when the vendored constructor's `if (lock_file_path.empty()) return;` early-outs
    /// (`InstanceLock.cpp:124-125`), leaving `m_slot` null.
    guard: Option<ReentrantGuard<'static, SlotState>>,
    locked: bool,
}

impl InstanceLock {
    /// `InstanceLock(const std::string&, std::chrono::milliseconds = default_timeout)`
    /// (`InstanceLock.cpp:122-148`).
    pub fn new(lock_file_path: &str) -> Self {
        Self::with_timeout(lock_file_path, DEFAULT_TIMEOUT)
    }

    /// Same constructor, with an explicit timeout (`InstanceLock.hpp:40`'s own reason this
    /// exists: so a test can shorten it).
    pub fn with_timeout(lock_file_path: &str, timeout: Duration) -> Self {
        if lock_file_path.is_empty() {
            return InstanceLock {
                guard: None,
                locked: false,
            };
        }
        let slot = slot_for(lock_file_path);
        let mut guard = slot.reentrant.lock();
        let now = Instant::now();
        if guard.depth == 0
            && now >= guard.cooldown_until
            && open_lock_file(&mut guard, lock_file_path)
        {
            let deadline = now + timeout;
            let mut taken = false;
            loop {
                match guard
                    .file_lock
                    .as_ref()
                    .expect("open_lock_file just set it on the branch above")
                    .try_lock()
                {
                    Ok(true) => {
                        taken = true;
                        break;
                    }
                    Ok(false) => {}
                    Err(e) => {
                        defer(&mut guard, &format!("Cannot lock {lock_file_path}: {e}"));
                        break;
                    }
                }
                if Instant::now() >= deadline {
                    defer(
                        &mut guard,
                        &format!(
                            "Another instance has held {lock_file_path} for over {} ms",
                            timeout.as_millis()
                        ),
                    );
                    break;
                }
                // PORT SRC: src/libslic3r/InstanceLock.cpp:146
                std::thread::sleep(Duration::from_millis(5));
            }
            if !taken {
                guard.file_lock = None;
            }
        }
        // Counted last, so a throw above -- read in this port as an early `defer`/`false` --
        // leaves the slot exactly as it was found (`InstanceLock.cpp:146-147`'s own comment).
        guard.depth += 1;
        let locked = guard.file_lock.is_some();
        InstanceLock {
            guard: Some(guard),
            locked,
        }
    }

    /// `locked()` (`InstanceLock.hpp:44`).
    pub fn locked(&self) -> bool {
        self.locked
    }
}

impl Drop for InstanceLock {
    /// `~InstanceLock()` (`InstanceLock.cpp:150-161`).
    fn drop(&mut self) {
        let Some(mut guard) = self.guard.take() else {
            return;
        };
        guard.depth -= 1;
        if guard.depth == 0 {
            if let Some(lock) = guard.file_lock.take() {
                lock.unlock();
            }
        }
        // `guard` drops here, releasing the reentrant lock -- the vendored `m_slot_guard`'s own
        // destructor timing.
    }
}

/// A reentrant mutex (`std::recursive_mutex`'s own guarantee -- the same thread may lock it
/// again without deadlocking itself, which is exactly what lets one thread hold a nested
/// `InstanceLock` on the same path while an outer one is still open). Rust's `std::sync::Mutex`
/// is not reentrant, and `std` has no recursive-mutex type, so this is a small, direct
/// equivalent of the one C++ standard-library primitive the vendored source uses that Rust's
/// standard library does not provide -- not a second "foundation type" competing with anything
/// else in the port, and private to this module.
struct ReentrantMutex<T> {
    state: Mutex<ReentrantState>,
    cond: Condvar,
    data: UnsafeCell<T>,
}

struct ReentrantState {
    owner: Option<ThreadId>,
    depth: u32,
}

// SAFETY: `data` is only ever read or written while a `ReentrantGuard` is held, and the
// `Mutex`/`Condvar` on `state` serialise who may hold a guard across threads -- the same
// contract `std::sync::Mutex<T>` itself relies on `unsafe impl Sync for Mutex<T>` for.
unsafe impl<T: Send> Send for ReentrantMutex<T> {}
// SAFETY: see above; `&ReentrantMutex<T>` is safe to share across threads because every access
// to `data` is serialised through `lock()`.
unsafe impl<T: Send> Sync for ReentrantMutex<T> {}

impl<T> ReentrantMutex<T> {
    fn new(value: T) -> Self {
        ReentrantMutex {
            state: Mutex::new(ReentrantState {
                owner: None,
                depth: 0,
            }),
            cond: Condvar::new(),
            data: UnsafeCell::new(value),
        }
    }

    /// Blocks until this thread holds the lock (immediately, if it already does), then returns
    /// a guard. Mirrors `std::recursive_mutex::lock()`.
    fn lock(&self) -> ReentrantGuard<'_, T> {
        let this_thread = thread::current().id();
        let mut state = self
            .state
            .lock()
            .expect("the reentrant mutex's own state mutex is never poisoned");
        loop {
            match state.owner {
                Some(owner) if owner == this_thread => {
                    state.depth += 1;
                    break;
                }
                None => {
                    state.owner = Some(this_thread);
                    state.depth = 1;
                    break;
                }
                Some(_) => {
                    state = self
                        .cond
                        .wait(state)
                        .expect("the reentrant mutex's own state mutex is never poisoned");
                }
            }
        }
        ReentrantGuard { mutex: self }
    }
}

struct ReentrantGuard<'a, T> {
    mutex: &'a ReentrantMutex<T>,
}

impl<T> Deref for ReentrantGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: this thread holds the reentrant lock (that is what owning a `ReentrantGuard`
        // means), so no other thread may be inside its own `deref`/`deref_mut` concurrently.
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T> DerefMut for ReentrantGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: same invariant as `deref` above.
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T> Drop for ReentrantGuard<'_, T> {
    fn drop(&mut self) {
        let mut state = self
            .mutex
            .state
            .lock()
            .expect("the reentrant mutex's own state mutex is never poisoned");
        state.depth -= 1;
        if state.depth == 0 {
            state.owner = None;
            self.mutex.cond.notify_one();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slic3r_proof::proof_units::instance_lock;
    use slic3r_proof::{compare_unit, Oracle};

    /// A fresh, process-unique path under the system temp directory -- a real file this proof
    /// creates, locks and cleans up, never a stand-in. The oracle and the port each use their
    /// own path (not the same file): `flock(2)` excludes by open file description, and this
    /// proof's two sides -- the oracle's `Slic3r::InstanceLock` and this module's own -- run in
    /// the *same* test process, so sharing one path would make them contend over the real OS
    /// lock instead of each independently exercising the same mechanism.
    fn fresh_lock_path(tag: &str) -> String {
        format!(
            "{}/grampus-instance-lock-proof-{tag}-{}-{}.lock",
            std::env::temp_dir().display(),
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock is after the Unix epoch")
                .as_nanos()
        )
    }

    /// The same five observations `path1_observations` scripts (see
    /// `cxx/proof_units/instance_lock.cpp`'s own doc comment), driven against this module's own
    /// [`InstanceLock`] instead of the vendored source's.
    fn ported_observations(path: &str) -> Vec<i64> {
        let mut obs = Vec::with_capacity(4);

        let empty_guard = InstanceLock::new("");
        obs.push(i64::from(empty_guard.locked()));

        {
            let outer = InstanceLock::new(path);
            obs.push(i64::from(outer.locked()));
            {
                let inner = InstanceLock::new(path);
                obs.push(i64::from(inner.locked()));
            }
            obs.push(i64::from(outer.locked()));
        }

        let again = InstanceLock::new(path);
        obs.push(i64::from(again.locked()));

        obs
    }

    /// Both the `InstanceLock.cpp` and `InstanceLock.hpp` manifest rows declare proof path `1`,
    /// so one test carrying one `// PROOF: 1` marker satisfies `port/port-status`'s
    /// `proof_present` check for each row independently. **What this drives, and what it
    /// cannot**: `open_lock_file`/`defer` are `private:` statics the shim cannot reach directly
    /// (see `cxx/proof_units/instance_lock.cpp`'s own header comment); this compares the
    /// *public* surface's observable behaviour -- whether a guard ends up `locked()` -- across
    /// a fresh lock, a same-thread nested lock, and a re-lock after release, which exercises the
    /// out-params and the caught throw transitively rather than comparing either one directly.
    // PROOF: 1
    #[test]
    fn instance_lock_observations_match_the_oracle() {
        let oracle_path = fresh_lock_path("oracle");
        let port_path = fresh_lock_path("port");

        let oracle_obs = instance_lock::path1_observations(&oracle_path);
        let ported_obs = ported_observations(&port_path);

        compare_unit(
            "InstanceLock.cpp",
            Oracle::Built,
            ported_obs.clone(),
            oracle_obs.clone(),
        )
        .expect(
            "the port's locked()-sequence must match the oracle's, observation for observation",
        );
        compare_unit("InstanceLock.hpp", Oracle::Built, ported_obs, oracle_obs).expect(
            "the port's locked()-sequence must match the oracle's, observation for observation",
        );

        let _ = std::fs::remove_file(&oracle_path);
        let _ = std::fs::remove_file(&port_path);
    }
}

// PORT STATUS
// source: src/libslic3r/InstanceLock.cpp
// source: src/libslic3r/InstanceLock.hpp
// confidence: medium
// tier: exact
// oracle: built
