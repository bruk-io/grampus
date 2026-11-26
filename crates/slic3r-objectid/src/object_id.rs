//! Port of `ObjectID.cpp`/`ObjectID.hpp`
//! (`vendor/OrcaSlicer/src/libslic3r/ObjectID.{cpp,hpp}`): the process-global identifier every
//! mutable `Model`/`Print` object carries, used upstream to synchronise the front end with the
//! back end and to key the Undo/Redo stack.
//!
//! **Inheritance.** `ObjectWithTimestamp` and `CutObjectBase` inherit `ObjectBase` in the
//! vendored source; Rust has no inheritance, so each is its own struct here that embeds an
//! [`ObjectBase`] and forwards to its methods (the idiom map's "a method on a class -> a method
//! on the corresponding Rust type").
//!
//! **Virtual dispatch.** `ObjectBase` declares two virtual methods, `timestamp()` (`:74`) and
//! `assign_new_unique_ids_recursive()` (`:95`). Both are overridden beyond the three types this
//! module ports: `timestamp()` by `ModelConfigObject` (`vendor/OrcaSlicer/src/libslic3r/
//! Model.hpp:106`, `public ObjectBase, public ModelConfig`), and
//! `assign_new_unique_ids_recursive()` by `ModelObject`/`ModelVolume`
//! (`Model.hpp:595`, `:1176`), dispatched through a base pointer at `Model.cpp:213`/`:215`. The
//! idiom map assigns a hierarchy's dispatch mechanism to its foundation design -- this task -- so
//! it is chosen once, here, as the [`ObjectBaseDispatch`] trait below, rather than left for
//! `slic3r-model` to invent on its own.

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
//
// FIDELITY(port): ObjectID.hpp:33 (`operator<=`) is `port/reports/order-comparators.md`'s
// `non_strict` row for this header, whose prescribed port's choice is "the strict form of the
// same relation". `#[derive(PartialOrd, Ord)]` derives exactly that: one total order over `id`,
// with `<=`/`>=` given by the derived `PartialOrd` default methods in terms of it, rather than a
// separately transcribed `operator<=` body -- there is no second, independently-stated
// non-strict relation here for the derive to diverge from.
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

    /// `timestamp()` (`:74`): `ObjectBase`'s own default always returns `0`. See
    /// [`ObjectBaseDispatch::timestamp`] for the trait a type overriding this dispatches
    /// through.
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
    /// See [`ObjectBaseDispatch::assign_new_unique_ids_recursive`] for the trait a type
    /// overriding this dispatches through.
    pub fn assign_new_unique_ids_recursive(&mut self) {
        self.set_new_unique_id();
    }
}

/// `ObjectBase()` (`:79`) is a `protected` constructor in the vendored source -- callable only
/// by a derived class. Rust has no inheritance and no `protected`, and [`ObjectBase::new`] is
/// already `pub` for exactly that reason (see this module's own doc comment); this `Default`
/// impl adds no capability beyond that already-`pub`, already side-effecting constructor. It is
/// recorded here because `Default::default()` can also be reached *implicitly* (`mem::take`,
/// `Vec::resize_with`, a derived `Default` on a containing struct), a path the vendored
/// `protected` constructor has no equivalent of -- see this task's pull request body.
impl Default for ObjectBase {
    fn default() -> Self {
        ObjectBase::new()
    }
}

/// Dispatch mechanism for `ObjectBase`'s two virtual methods (`ObjectID.hpp:74`, `:95`), chosen
/// here per the idiom map's rule that a hierarchy's dispatch mechanism is the foundation
/// design's own call, not a fan-out task's one to invent (see this module's own doc comment for
/// the overriding sites outside this crate). A type that embeds an [`ObjectBase`] implements
/// this trait; `object_base`/`object_base_mut` are also the one path a type whose own base field
/// is private (like [`ObjectWithTimestamp`] below) exposes to that base's own mutators.
pub trait ObjectBaseDispatch {
    /// The embedded base (`self`, for [`ObjectBase`] itself).
    fn object_base(&self) -> &ObjectBase;
    /// The embedded base, mutably.
    fn object_base_mut(&mut self) -> &mut ObjectBase;

    /// `timestamp()` (`:74`): `ObjectBase`'s own default always returns `0`.
    fn timestamp(&self) -> Timestamp {
        0
    }

    /// `assign_new_unique_ids_recursive()` (`:95`): `ObjectBase`'s own body only reassigns its
    /// own id; an implementor that owns other dispatch-trait instances overrides this to recurse
    /// into them.
    fn assign_new_unique_ids_recursive(&mut self) {
        self.object_base_mut().set_new_unique_id();
    }
}

impl ObjectBaseDispatch for ObjectBase {
    fn object_base(&self) -> &ObjectBase {
        self
    }
    fn object_base_mut(&mut self) -> &mut ObjectBase {
        self
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

    /// `set_new_unique_id()` (inherited from [`ObjectBase::set_new_unique_id`], `:89`): the
    /// embedded `base` field is private, so this forwarder is the only path a caller outside
    /// this module has to it -- the vendored method is `protected`, reached by the two in-scope
    /// derived types (`LayerHeightProfile`, `FacetsAnnotation`, `Model.hpp:218,758`) through
    /// inheritance and, for `ModelObject`/`ModelVolume`, friendship (`Model.hpp:253`); see this
    /// task's pull request body for the opposite-direction widening this forwarding makes `pub`
    /// rather than `protected`-equivalent.
    pub fn set_new_unique_id(&mut self) {
        self.base.set_new_unique_id();
    }

    /// `set_invalid_id()` (inherited from [`ObjectBase::set_invalid_id`], `:90`); see
    /// [`Self::set_new_unique_id`]'s own doc comment for why this is a forwarder.
    pub fn set_invalid_id(&mut self) {
        self.base.set_invalid_id();
    }

    /// `copy_id(const ObjectBase &rhs)` (inherited from [`ObjectBase::copy_id`], `:92`); see
    /// [`Self::set_new_unique_id`]'s own doc comment for why this is a forwarder.
    pub fn copy_id(&mut self, rhs: &ObjectWithTimestamp) {
        self.base.copy_id(&rhs.base);
    }

    /// `timestamp()` (`:132`): overrides [`ObjectBase::timestamp`]'s always-`0` default. See
    /// [`ObjectBaseDispatch::timestamp`] for the trait this override dispatches through.
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

/// `ObjectWithTimestamp()` (`:117`) is `protected`; see [`ObjectBase`]'s own `Default` impl doc
/// comment for why recording that gap here, rather than removing this impl, is this module's
/// choice -- [`ObjectWithTimestamp::new`] is already `pub` and already side-effecting.
impl Default for ObjectWithTimestamp {
    fn default() -> Self {
        ObjectWithTimestamp::new()
    }
}

impl ObjectBaseDispatch for ObjectWithTimestamp {
    fn object_base(&self) -> &ObjectBase {
        &self.base
    }
    fn object_base_mut(&mut self) -> &mut ObjectBase {
        &mut self.base
    }

    /// `timestamp()` (`:132`): overrides the trait's always-`0` default.
    fn timestamp(&self) -> Timestamp {
        self.m_timestamp
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

impl ObjectBaseDispatch for CutObjectBase {
    fn object_base(&self) -> &ObjectBase {
        &self.base
    }
    fn object_base_mut(&mut self) -> &mut ObjectBase {
        &mut self.base
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

#[cfg(test)]
mod tests {
    use super::*;
    use slic3r_proof::dumps;

    // The oracle for the `ObjectID.hpp` row:
    // `crates/slic3r-ffi/cxx/proof_units/object_id.cpp`'s scripted sequence over the vendored
    // header's own inline methods, compiled from the pin onto the FFI crate's shared
    // `cxx_build::Build` and reached here through `slic3r-proof`, which re-exports the FFI
    // crate's per-unit proof bridges -- the one sanctioned route (`docs/PORTING.md`, "Decisions
    // recorded on 2026-11-25"). Every value the shim records is a delta or a relation, never an
    // absolute id or timestamp, so this comparison does not depend on where either side's
    // process-global counter stood when it ran and cannot become order-dependent on what else
    // this test binary did first. The absolute first-id values belong to the `ObjectID.cpp` row
    // and are proven in `tests/object_id_proof_path_1.rs`, which is a separate binary for
    // exactly that reason.
    use slic3r_proof::proof_units::object_id::path2_header_observations;

    /// Every observation `path2_header_observations` records, in its own order,
    /// named so a failure names the vendored behaviour that diverged rather than an index.
    const OBSERVATION_NAMES: &[&str] = &[
        "consecutive ObjectBase ids differ by (ObjectID.hpp:100)",
        "ObjectBase(int)'s id is invalid() (ObjectID.hpp:82, :37)",
        "ObjectBase(int)'s id is valid() (ObjectID.hpp:82, :36)",
        "a freshly minted id is valid() (ObjectID.hpp:79, :36)",
        "set_new_unique_id() advances the id by (ObjectID.hpp:89)",
        "set_invalid_id() leaves an invalid() id (ObjectID.hpp:90)",
        "copy_id() reproduces the source id (ObjectID.hpp:92)",
        "assign_new_unique_ids_recursive() advances the id by (ObjectID.hpp:95)",
        "ObjectBase::timestamp() (ObjectID.hpp:74)",
        "a fresh ObjectWithTimestamp's timestamp is non-zero (ObjectID.hpp:138-139)",
        "consecutive touch()es differ by (ObjectID.hpp:135)",
        "two independently constructed timestamps match (ObjectID.hpp:133)",
        "timestamps match after copy_timestamp() (ObjectID.hpp:125, :133)",
        "object_id_and_timestamp_match() after copy_timestamp() (ObjectID.hpp:134)",
        "ObjectInstanceID's default instance_id is size_t(-1) (ObjectID.hpp:48)",
        "ObjectInstanceID's order is object_id-major (ObjectID.hpp:52-55)",
        "ObjectInstanceID's order breaks ties on instance_id (ObjectID.hpp:52-55)",
        "CutObjectBase()'s id is invalid() (ObjectID.hpp:156)",
        "CutObjectBase()'s check_sum (ObjectID.hpp:150)",
        "CutObjectBase()'s connectors_cnt (ObjectID.hpp:152)",
        "check_sum after increase_check_sum(7) (ObjectID.hpp:197)",
        "connectors_cnt after increase_connectors_cnt(3) (ObjectID.hpp:200)",
        "CutObjectBase's order compares the id alone (ObjectID.hpp:165)",
        "CutObjectBase's equality compares the id alone (ObjectID.hpp:166)",
        "is_equal() after copy() (ObjectID.hpp:168-173, :193)",
        "check_sum after invalidate() (ObjectID.hpp:184-189)",
    ];

    /// The same scripted sequence `slic3r_objectid_shim_header_observations` runs, against this
    /// module's own types instead of the vendored header's, in the same order and recorded as the
    /// same deltas and relations.
    fn ported_header_observations() -> Vec<i64> {
        let first = ObjectBase::new();
        let second = ObjectBase::new();
        let consecutive_ids_differ_by = second.id().id as i64 - first.id().id as i64;

        let invalid = ObjectBase::new_invalid();
        let invalid_is_invalid = i64::from(invalid.id().invalid());
        let invalid_is_not_valid = i64::from(invalid.id().valid());
        let fresh_is_valid = i64::from(first.id().valid());

        let mut mutated = ObjectBase::new();
        let before_mutation = mutated.id().id as i64;
        mutated.set_new_unique_id();
        let set_new_unique_id_advances_by = mutated.id().id as i64 - before_mutation;
        mutated.set_invalid_id();
        let set_invalid_id_gives_invalid = i64::from(mutated.id().invalid());
        mutated.copy_id(&first);
        let copy_id_matches_source = i64::from(mutated.id() == first.id());

        let mut recursive = ObjectBase::new();
        let before_recursive = recursive.id().id as i64;
        recursive.assign_new_unique_ids_recursive();
        let assign_recursive_advances_by = recursive.id().id as i64 - before_recursive;

        let object_base_timestamp = first.timestamp() as i64;
        let mut stamped = ObjectWithTimestamp::new();
        let timestamp_is_nonzero_before_touch = i64::from(stamped.timestamp() != 0);
        stamped.touch();
        let after_first_touch = stamped.timestamp() as i64;
        stamped.touch();
        let consecutive_touches_differ_by = stamped.timestamp() as i64 - after_first_touch;
        let mut other_stamped = ObjectWithTimestamp::new();
        let distinct_timestamps_match = i64::from(stamped.timestamp_matches(&other_stamped));
        other_stamped.copy_timestamp(&stamped);
        let copied_timestamps_match = i64::from(stamped.timestamp_matches(&other_stamped));
        let id_and_timestamp_match_after_copy =
            i64::from(stamped.object_id_and_timestamp_match(&other_stamped));

        let default_instance = ObjectInstanceID::default();
        let default_instance_id_is_size_t_minus_one =
            i64::from(default_instance.instance_id == usize::MAX);
        let low = ObjectInstanceID {
            object_id: ObjectID::new(1),
            instance_id: 1,
        };
        let high_by_object = ObjectInstanceID {
            object_id: ObjectID::new(2),
            instance_id: 0,
        };
        let high_by_instance = ObjectInstanceID {
            object_id: ObjectID::new(1),
            instance_id: 2,
        };
        let instance_order_is_object_major = i64::from(low < high_by_object);
        let instance_order_breaks_ties_on_instance = i64::from(low < high_by_instance);

        let mut cut = CutObjectBase::new();
        let cut_default_id_is_invalid = i64::from(cut.id().invalid());
        let cut_default_check_sum = cut.check_sum() as i64;
        let cut_default_connectors_cnt = cut.connectors_cnt() as i64;
        cut.increase_check_sum(7);
        cut.increase_connectors_cnt(3);
        let cut_check_sum_after_increase = cut.check_sum() as i64;
        let cut_connectors_cnt_after_increase = cut.connectors_cnt() as i64;
        let cut_low = CutObjectBase::with_id(ObjectID::new(1), 0, 0);
        let cut_high = CutObjectBase::with_id(ObjectID::new(2), 99, 99);
        let cut_order_is_id_only = i64::from(cut_low < cut_high);
        let cut_equality_is_id_only =
            i64::from(cut_low == CutObjectBase::with_id(ObjectID::new(1), 42, 42));
        let mut cut_copied = CutObjectBase::new();
        cut_copied.copy(&cut_high);
        let cut_copy_is_equal = i64::from(cut_copied.is_equal(&cut_high));
        cut_copied.invalidate();
        let cut_invalidate_check_sum = cut_copied.check_sum() as i64;

        vec![
            consecutive_ids_differ_by,
            invalid_is_invalid,
            invalid_is_not_valid,
            fresh_is_valid,
            set_new_unique_id_advances_by,
            set_invalid_id_gives_invalid,
            copy_id_matches_source,
            assign_recursive_advances_by,
            object_base_timestamp,
            timestamp_is_nonzero_before_touch,
            consecutive_touches_differ_by,
            distinct_timestamps_match,
            copied_timestamps_match,
            id_and_timestamp_match_after_copy,
            default_instance_id_is_size_t_minus_one,
            instance_order_is_object_major,
            instance_order_breaks_ties_on_instance,
            cut_default_id_is_invalid,
            cut_default_check_sum,
            cut_default_connectors_cnt,
            cut_check_sum_after_increase,
            cut_connectors_cnt_after_increase,
            cut_order_is_id_only,
            cut_equality_is_id_only,
            cut_copy_is_equal,
            cut_invalidate_check_sum,
        ]
    }

    /// Proof path 2 (R19) for the `ObjectID.hpp` manifest row.
    ///
    /// **The declared mechanism, and what the corpus does not carry.** This row's `proof path`
    /// cell is `2`, the committed stage dumps, seeded from its non-`none` pipeline stage
    /// (`posSlice`) by `port/inventory`'s own rule. That dump is read below -- a real committed
    /// dump, whose provenance header `dumps::read_dump_default` checks against the pin and
    /// refuses if it disagrees -- so the declared mechanism genuinely runs. But the corpus
    /// carries no `ObjectID` value anywhere to compare against: the `; object:` key every dump
    /// starts with is a *positional index* into `print.objects()`, written by
    /// `oracle/stage-dump/dump_seams.cpp:109-113`'s own linear scan and never derived from an
    /// `ObjectBase::id()`, and every body field below it is geometry and scalars
    /// (`oracle/stage-dump/README.md`'s "Dump format"). That is a finding for whoever
    /// regenerates `port/inventory`'s seeding rule, recorded in this task's pull request body,
    /// and not something a tolerance or a stand-in may paper over on this branch.
    ///
    /// **What is proven instead.** The vendored header's own inline code, compiled from the pin
    /// and called through this crate's shim, against this module's rendering of it -- a
    /// differential comparison, which is *stronger* than the declared mechanism rather than a
    /// substitute for it, and exact with no tolerance as this row's `exact` tier requires. Never
    /// read this as the row's mechanism being proof path 1: path 1 is the `ObjectID.cpp` row's
    /// cell, proven separately in `tests/object_id_proof_path_1.rs`.
    // PROOF: 2
    #[test]
    fn object_id_header_semantics_match_the_oracle() {
        let dump = dumps::read_dump_default("20mm_cube-base-fff", "posSlice")
            .expect("port/dumps/20mm_cube-base-fff/posSlice.dump must read at the pinned commit");
        assert_eq!(
            dump.seam, "posSlice",
            "the dump read is this row's own pipeline stage"
        );

        let ported = ported_header_observations();
        let oracle = path2_header_observations();
        assert_eq!(
            oracle.len(),
            ported.len(),
            "the oracle records {} observation(s) where this test scripts {}",
            oracle.len(),
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
// source: src/libslic3r/ObjectID.cpp
// source: src/libslic3r/ObjectID.hpp
// confidence: medium
// tier: exact
// oracle: built
