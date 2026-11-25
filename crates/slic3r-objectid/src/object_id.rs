//! Port of `ObjectID.cpp`/`ObjectID.hpp`
//! (`vendor/OrcaSlicer/src/libslic3r/ObjectID.{cpp,hpp}`): the process-global identifier every
//! mutable `Model`/`Print` object carries, used upstream to synchronise the front end with the
//! back end and to key the Undo/Redo stack.
//!
//! **Inheritance.** `ObjectWithTimestamp` and `CutObjectBase` inherit `ObjectBase` in the
//! vendored source and override its virtual `timestamp()`/none respectively; Rust has no
//! inheritance, so each is its own struct here that embeds an [`ObjectBase`] and forwards to its
//! methods (the idiom map's "a method on a class -> a method on the corresponding Rust type").
//! `ObjectBase::timestamp()` always returns `0` (`:74`); it is not a trait with an overridable
//! default, because no in-scope caller needs virtual dispatch across the three types through one
//! interface today -- see this task's pull request body.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::OnceLock;

// PORT NOTE: ObjectID.hpp:4-5,42-44,106-109,142-144 -- the `cereal` `serialize`/
// `load_and_construct` templates, the `friend class cereal::access` declarations and the
// `friend class Slic3r::UndoRedo::StackImpl` declaration are the Undo/Redo serialization
// boundary. `port/manifest.tsv` carries no row for `UndoRedo.hpp`/`.cpp` (out of scope), so that
// boundary is not ported.

// PORT NOTE: ObjectID.cpp:25 -- the commented-out `CEREAL_REGISTER_TYPE(Slic3r::ObjectBase)` is
// dead source (`## Don't-translate list`'s "upstream code that is commented out") and is not
// revived.

/// `Slic3r::ObjectID` (`ObjectID.hpp:22-44`): valid IDs are strictly positive; `0` is the
/// invalid/default value (`:27`, `:36-37`). All six comparison operators (`:29-34`) compare only
/// `id`, so deriving `PartialEq`/`Eq`/`PartialOrd`/`Ord` on the one field reproduces each of them
/// exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct ObjectID {
    pub id: usize,
}

impl ObjectID {
    /// `ObjectID(size_t id)` (`:25`).
    pub fn new(id: usize) -> Self {
        ObjectID { id }
    }

    /// `valid()` (`:36`).
    pub fn valid(&self) -> bool {
        self.id != 0
    }

    /// `invalid()` (`:37`).
    pub fn invalid(&self) -> bool {
        self.id == 0
    }
}

/// `Slic3r::ObjectInstanceID` (`ObjectID.hpp:46-56`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObjectInstanceID {
    pub object_id: ObjectID,
    pub instance_id: usize,
}

impl Default for ObjectInstanceID {
    /// `size_t instance_id { size_t(-1) }` (`:48`): `usize::MAX` is `size_t(-1)` for an unsigned
    /// 64-bit `size_t`, the platform this port targets.
    fn default() -> Self {
        ObjectInstanceID {
            object_id: ObjectID::default(),
            instance_id: usize::MAX,
        }
    }
}

/// Process-global counter backing `ObjectBase::generate_new_id` (`ObjectID.cpp:6`,
/// `ObjectID.hpp:100-101`). The vendored counter is a plain, non-atomic `size_t`, and the header
/// comment states instances are expected only from the main thread (`ObjectID.hpp:62-63`); an
/// `AtomicUsize` with `Relaxed` ordering reproduces the same "increment, then read the new value"
/// sequence without the data race that a bare mutable global would be (undefined behaviour in
/// Rust), while adding no cross-thread synchronisation guarantee the original lacks either.
static NEXT_OBJECT_ID: AtomicUsize = AtomicUsize::new(0);

/// `ObjectBase::generate_new_id()` (`ObjectID.hpp:100-101`): `return ObjectID(++s_last_id);` --
/// pre-increment, so `fetch_add` (which returns the *old* value) is read as `old + 1`.
fn generate_new_id() -> ObjectID {
    ObjectID::new(NEXT_OBJECT_ID.fetch_add(1, Ordering::Relaxed) + 1)
}

/// `ObjectBase::Timestamp` (`ObjectID.hpp:67`, `uint64_t`).
pub type Timestamp = u64;

/// `Slic3r::ObjectBase` (`ObjectID.hpp:64-110`): the common base `Model`, `ModelObject`,
/// `ModelVolume`, `ModelInstance`, `ModelMaterial`, `Print`, `PrintObject`, `SLAPrint` and
/// `SLAPrintObject` derive from, to carry a unique [`ObjectID`]. See this module's own doc
/// comment for how inheritance is ported.
#[derive(Debug, Clone, Copy)]
pub struct ObjectBase {
    m_id: ObjectID,
}

impl ObjectBase {
    /// `ObjectBase()` (`:79`): assigns a new, process-unique id.
    pub fn new() -> Self {
        ObjectBase {
            m_id: generate_new_id(),
        }
    }

    /// `ObjectBase(int)` (`:82`): assigns the invalid id, to be replaced by an existing id copied
    /// from elsewhere.
    pub fn new_invalid() -> Self {
        ObjectBase {
            m_id: ObjectID::default(),
        }
    }

    /// `ObjectBase(const ObjectID id)` (`:84`).
    pub fn with_id(id: ObjectID) -> Self {
        ObjectBase { m_id: id }
    }

    /// `id()` (`:69`).
    pub fn id(&self) -> ObjectID {
        self.m_id
    }

    /// `timestamp()` (`:74`): `ObjectBase`'s own default always returns `0`.
    pub fn timestamp(&self) -> Timestamp {
        0
    }

    /// `set_new_unique_id()` (`:89`).
    pub fn set_new_unique_id(&mut self) {
        self.m_id = generate_new_id();
    }

    /// `set_invalid_id()` (`:90`).
    pub fn set_invalid_id(&mut self) {
        self.m_id = ObjectID::default();
    }

    /// `copy_id(const ObjectBase &rhs)` (`:92`).
    pub fn copy_id(&mut self, rhs: &ObjectBase) {
        self.m_id = rhs.id();
    }

    /// `assign_new_unique_ids_recursive()` (`:95`): `ObjectBase`'s own body only reassigns its
    /// own id; a derived type that owns other `ObjectBase`-derived instances overrides this to
    /// recurse into them (no such override exists in this module -- see the module doc comment).
    pub fn assign_new_unique_ids_recursive(&mut self) {
        self.set_new_unique_id();
    }
}

impl Default for ObjectBase {
    fn default() -> Self {
        ObjectBase::new()
    }
}

/// Process-global counter backing `ObjectWithTimestamp::touch`, mirroring [`NEXT_OBJECT_ID`]'s
/// own reasoning. `s_last_timestamp` starts at `1` (`ObjectID.cpp:21`), not `0`.
static NEXT_TIMESTAMP: AtomicU64 = AtomicU64::new(1);

/// `Slic3r::ObjectWithTimestamp` (`ObjectID.hpp:112-145`): embeds an [`ObjectBase`] (the
/// vendored type inherits from it) plus its own timestamp, used to decide whether the Undo/Redo
/// stack needs a fresh snapshot.
#[derive(Debug, Clone, Copy)]
pub struct ObjectWithTimestamp {
    base: ObjectBase,
    // The first timestamp is non-zero, as zero timestamp means the timestamp is not reliable
    // (`:138-139`).
    m_timestamp: Timestamp,
}

impl ObjectWithTimestamp {
    /// `ObjectWithTimestamp()` (`:117`, `= default`): a new base id, timestamp `1`.
    pub fn new() -> Self {
        ObjectWithTimestamp {
            base: ObjectBase::new(),
            m_timestamp: 1,
        }
    }

    /// `ObjectWithTimestamp(int)` (`:120`): `ObjectBase(-1)`, an invalid id.
    pub fn new_invalid() -> Self {
        ObjectWithTimestamp {
            base: ObjectBase::new_invalid(),
            m_timestamp: 1,
        }
    }

    /// `id()` (inherited from [`ObjectBase::id`]).
    pub fn id(&self) -> ObjectID {
        self.base.id()
    }

    /// `timestamp()` (`:132`): overrides [`ObjectBase::timestamp`]'s always-`0` default.
    pub fn timestamp(&self) -> Timestamp {
        self.m_timestamp
    }

    /// `timestamp_matches(const ObjectWithTimestamp &rhs)` (`:133`).
    pub fn timestamp_matches(&self, rhs: &ObjectWithTimestamp) -> bool {
        self.m_timestamp == rhs.m_timestamp
    }

    /// `object_id_and_timestamp_match(const ObjectWithTimestamp &rhs)` (`:134`).
    pub fn object_id_and_timestamp_match(&self, rhs: &ObjectWithTimestamp) -> bool {
        self.id() == rhs.id() && self.m_timestamp == rhs.m_timestamp
    }

    /// `touch()` (`:135`): `m_timestamp = ++s_last_timestamp;` -- pre-increment, read as
    /// `old + 1`, matching [`generate_new_id`]'s own idiom.
    pub fn touch(&mut self) {
        self.m_timestamp = NEXT_TIMESTAMP.fetch_add(1, Ordering::Relaxed) + 1;
    }

    /// `copy_timestamp(const ObjectWithTimestamp& rhs)` (`:125`).
    pub fn copy_timestamp(&mut self, rhs: &ObjectWithTimestamp) {
        self.m_timestamp = rhs.m_timestamp;
    }
}

impl Default for ObjectWithTimestamp {
    fn default() -> Self {
        ObjectWithTimestamp::new()
    }
}

/// `Slic3r::CutObjectBase` (`ObjectID.hpp:147-209`): embeds an [`ObjectBase`] (the vendored type
/// inherits from it) plus the running totals the Cut tool keeps across a model's connectors.
#[derive(Debug, Clone, Copy)]
pub struct CutObjectBase {
    base: ObjectBase,
    m_check_sum: usize,
    m_connectors_cnt: usize,
}

impl CutObjectBase {
    /// `CutObjectBase()` (`:156`) and `CutObjectBase(int)` (`:159`): both `ObjectBase(-1)`, an
    /// invalid id, check sum `1`, zero connectors.
    pub fn new() -> Self {
        CutObjectBase {
            base: ObjectBase::new_invalid(),
            m_check_sum: 1,
            m_connectors_cnt: 0,
        }
    }

    /// `CutObjectBase(ObjectID id, size_t check_sum, size_t connectors_cnt)` (`:161`).
    pub fn with_id(id: ObjectID, check_sum: usize, connectors_cnt: usize) -> Self {
        CutObjectBase {
            base: ObjectBase::with_id(id),
            m_check_sum: check_sum,
            m_connectors_cnt: connectors_cnt,
        }
    }

    /// `id()` (inherited from [`ObjectBase::id`]).
    pub fn id(&self) -> ObjectID {
        self.base.id()
    }

    /// `copy(const CutObjectBase &rhs)` (`:168-173`).
    pub fn copy(&mut self, rhs: &CutObjectBase) {
        self.base.copy_id(&rhs.base);
        self.m_check_sum = rhs.check_sum();
        self.m_connectors_cnt = rhs.connectors_cnt();
    }

    /// `invalidate()` (`:184-189`).
    pub fn invalidate(&mut self) {
        self.base.set_invalid_id();
        self.m_check_sum = 1;
        self.m_connectors_cnt = 0;
    }

    /// `init()` (`:191`).
    pub fn init(&mut self) {
        self.base.set_new_unique_id();
    }

    /// `has_same_id(const CutObjectBase &rhs)` (`:192`).
    pub fn has_same_id(&self, rhs: &CutObjectBase) -> bool {
        self.id() == rhs.id()
    }

    /// `is_equal(const CutObjectBase &rhs)` (`:193`).
    pub fn is_equal(&self, rhs: &CutObjectBase) -> bool {
        self.id() == rhs.id()
            && self.check_sum() == rhs.check_sum()
            && self.connectors_cnt() == rhs.connectors_cnt()
    }

    /// `check_sum()` (`:195`).
    pub fn check_sum(&self) -> usize {
        self.m_check_sum
    }

    /// `set_check_sum(size_t cs)` (`:196`).
    pub fn set_check_sum(&mut self, cs: usize) {
        self.m_check_sum = cs;
    }

    /// `increase_check_sum(size_t cnt)` (`:197`).
    pub fn increase_check_sum(&mut self, cnt: usize) {
        self.m_check_sum += cnt;
    }

    /// `connectors_cnt()` (`:199`).
    pub fn connectors_cnt(&self) -> usize {
        self.m_connectors_cnt
    }

    /// `increase_connectors_cnt(size_t connectors_cnt)` (`:200`).
    pub fn increase_connectors_cnt(&mut self, connectors_cnt: usize) {
        self.m_connectors_cnt += connectors_cnt;
    }
}

impl Default for CutObjectBase {
    fn default() -> Self {
        CutObjectBase::new()
    }
}

/// `operator==`/`operator<` (`:165-166`): both compare only the id, ignoring `m_check_sum` and
/// `m_connectors_cnt` -- `operator<` is `other.id() > this->id()`, i.e. `this->id() <
/// other.id()`, the same relation [`ObjectID`]'s own derived `Ord` gives.
impl PartialEq for CutObjectBase {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}
impl Eq for CutObjectBase {}
impl PartialOrd for CutObjectBase {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for CutObjectBase {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id().cmp(&other.id())
    }
}

static WIPE_TOWER_OBJECT: OnceLock<ObjectBase> = OnceLock::new();
static WIPE_TOWER_INSTANCE: OnceLock<ObjectBase> = OnceLock::new();

/// `Slic3r::wipe_tower_object_id()` (`ObjectID.cpp:9-13`): `static ObjectBase mine; return
/// mine.id();` -- a function-local static, lazily constructed exactly once per process.
/// `OnceLock` is the same one-time, thread-safe lazy initialisation C++11 guarantees for a
/// function-local `static`, so this is a direct idiom match rather than a divergence.
pub fn wipe_tower_object_id() -> ObjectID {
    WIPE_TOWER_OBJECT.get_or_init(ObjectBase::new).id()
}

/// `Slic3r::wipe_tower_instance_id()` (`ObjectID.cpp:15-19`).
pub fn wipe_tower_instance_id() -> ObjectID {
    WIPE_TOWER_INSTANCE.get_or_init(ObjectBase::new).id()
}

// PORT STATUS
// source: src/libslic3r/ObjectID.cpp
// source: src/libslic3r/ObjectID.hpp
// confidence: medium
// tier: exact
// oracle: built
