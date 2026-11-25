---
kind: plan
---
# Plan: Classify the root subsystem's 120 raw-pointer fields

## Files that change

- port/ownership.tsv (the `class`, `evidence` and `rust type` cells of this batch's 120 rows only)

## Order of work

### Step 1: Decide the three cyclic graphs' back-reference shape, once each
- delivers: R30

`class: back-reference` applies in this batch to the parent/child and sibling-within-owning-
collection pointers of three graphs: (a) `Print`/`PrintObject`/`Layer`/`LayerRegion`, (b)
`Model`/`ModelObject`/`ModelVolume`, and (c) `PrintBase.hpp`'s `m_print` pair (`:376`, `:691`).
The default table conditions the Rust shape on whether the owner holds an `Arc`: `Weak<Pointee>`
if it does, `usize` (an index into the owning collection) otherwise.

Every `owned` row this batch found in graphs (a) and (b) is, without exception, a plain
`Vec<Pointee>` or `HashMap<K, Pointee>` by value (`Print::m_objects` deletes each `PrintObject*`
directly, `Model::materials` deletes each `ModelMaterial*` directly, and so on for `m_layers`,
`m_support_layers`, `Layer::m_regions`, `Model::objects`, `ModelObject::volumes`/`instances` -- see
the per-row evidence in `port/ownership.tsv`). None of those owners is `Arc`. An earlier draft of
this plan then tried to treat graph (c) differently -- giving `PrintBase.hpp`'s pair `Weak<Print>`
on the theory that `PrintBase`'s cancellation machinery (`state_mutex`, `cancel_callback`,
`throw_if_canceled()`) implies the real port wraps `Print` in `Arc` -- while giving
`Model.hpp:189`/`:666` (`ModelMaterial`/`ModelObject`'s own `m_model`, which equally reference a
singular root rather than a Vec element) `usize` regardless. That is the same structural fact
(pointee is the singular root, not a collection element) answered two different ways four rows
apart, which is exactly the "one answer per row instead of one per graph" failure this step exists
to avoid.

This plan resolves that by dropping the `Arc<Print>` guess: nothing at the v2.5.2 pin shows `Print`
or `PrintBase` behind an `Arc` or any other shared-ownership wrapper -- the mutex/cancel-callback
machinery shows the design is thread-*aware*, not that this specific pointer is `Arc`-backed, and
every other row in this batch traces its class to a comment or a `.cpp` site, never to an inference
about a design the headers do not show. **All three graphs therefore resolve to `usize`**,
including the two rows whose pointee is a singular root rather than a Vec element
(`Model.hpp:189`/`:666`'s `m_model`, and `PrintBase.hpp:376`/`:691`'s `m_print`): there being only
one such instance, the "index" is a fixed, trivial handle rather than a position in an array, and
whether the real port stores it at all or instead threads the owner through as an explicit
parameter is an implementation decision this classification pass does not make. The discriminator
that keeps the file consistent is: **a back-reference is `Weak` only where this batch's own
evidence shows its owner holding an `Arc`, and `usize` otherwise** -- not which named graph the row
happens to sit in. At this pin, no row in any of the three graphs meets that bar, so every
back-reference in this batch is `usize`.

### Step 2: Classify every row with cited evidence, applying the per-class defaults
- delivers: R30

Fill `class`, `evidence` and `rust type` for the 120 rows `44-ownership-class-root`'s task.md
names (the `root` subsystem: every in-scope header directly under `src/libslic3r`), reading the
vendored header plus, where the header's own comment does not settle it, the corresponding `.cpp`
in the same tree. `port/ownership.tsv` carries the result; this plan does not duplicate the table.

**Two rows an earlier draft of this plan left `unknown` are not, in fact, unknown; both are fixed
after review found the evidence this draft missed.** `GCodeWriter.hpp:402` `buf_end` is a pointer
into `GCodeFormatter`'s own inline `buf[buflen]` array, always `buf + buflen` (`GCodeWriter.hpp:321`)
-- a decided, non-owning borrow of a sibling field, not a case the evidence fails to settle. It is
reclassified `borrowed`, departing from that class's generic `&'a Pointee` default: `usize` (the
offset), because `&'a char` is meaningless for a cursor that is always a compile-time-constant
distance from `buf`, not an address a porter dereferences. `Layer.hpp:344` `area`
(`AreaGroup::area`) does have a use beyond its own declaration and constructor -- this plan's
earlier claim that it did not was wrong, found by grepping the type name (`AreaGroup`) instead of
the field name (`area_groups`). `Support/TreeSupport.cpp:2562-2566` populates `area_groups` with
pointers into `ts_layer->base_areas`/`roof_areas`/`floor_areas`/`roof_1st_layer`, all `ExPolygons`
members of the same `SupportLayer` the `AreaGroup` lives in; `TreeSupport.cpp:2588-2590` mutates
the pointee through the pointer (`expoly->holes.erase(...)`); nothing ever deletes it. That is a
non-owning, mutating borrow of a sibling field's element, owned by the enclosing object -- the same
shape as this batch's other `borrowed` rows into a sibling collection, not a cycle and not external.
Reclassified `borrowed`, `&'a mut ExPolygon` (the pointee is never `const`).

**Rows this plan leaves `unknown`, and why** (7 rows, all of `ProjectTask.hpp`'s cross-pointer
fields except `subtasks` and `parent_task_`): `subtasks` is owned, confirmed by an explicit
`delete`/`new` pair in `ProjectTask.cpp`, and `parent_task_` is the matching back-reference into
that same owned `subtasks`. The remaining seven -- `profile_` (`BBLSliceInfo` and `BBLTask`),
`project_`, `tasks`, and both `slice_info` maps -- are genuinely unresolved: `ProjectTask.cpp` only
ever assigns these from a constructor parameter or looks them up (`get_slice_info`); nothing under
`src/libslic3r` ever `push_back`s into `tasks`/`profiles` or deletes a `profile_`/`project_`/
`slice_info` pointee. Unlike the `AreaGroup` row above, there genuinely is no use of these seven
anywhere in this header's own evidence path to cite -- the object graph that would settle them (if
one exists) is built in GUI code under `src/slic3r/GUI`, out of scope for this header's own
evidence, so their `evidence` cell cites the field's own declaration in place of a use. All seven
carry the conservative `Arc<Mutex<Pointee>>` (the pointee's own pointer is never `const`, so a
porter must assume concurrent mutation until a later pass resolves it), except the two `slice_info`
maps, whose element type keeps that `Arc<Mutex<Pointee>>` but whose container follows `std::map`'s
own ordering (see the container-rule paragraph below): `BTreeMap<String, Arc<Mutex<BBLSliceInfo>>>`,
not `HashMap`.

**Rows this plan departs from a mechanical default, named individually:**
- `Config.hpp:2636` `nocli`: `static const constexpr char *` is a string-literal constant, not a
  pointer at an object -- `borrowed` with `&'static str`, not the generic `&'a Pointee`.
- `clonable_ptr.hpp:141` `px`: its copy constructor deep-clones (`rhs.px->clone()`) and its
  destructor `delete`s -- `owned`, `Box<T>`.
- `Print.hpp:218` `m_data` (`ConstVectorOfPtrsAdaptor::m_data`): a pointer to a vector of pointers,
  held by a view that owns neither the vector nor its elements -- `borrowed`. Its rust type is not
  a departure after all: it is this batch's own container-of-pointers rule (below) applied to a
  `const std::vector<T*> *`, giving `&'a [&'a T]`, the same mechanical answer as every other
  `borrowed` container row. An earlier draft of this cell, `&'a [T]`, silently flattened away the
  inner pointer and was inconsistent with `ClipperUtils.hpp:315` and `PerimeterGenerator.hpp:88`,
  two plain containers of raw pointers that correctly kept it; all three are now spelled the one
  way the container rule already states.
- `InstanceLock.hpp:55` `m_slot` and `FlushVolPredictor.hpp:48` `predictor`: both point into a
  process-lifetime, never-freed static cache (`InstanceLock.cpp`'s leaked registry map;
  `FlushVolPredictor.cpp`'s static `predictor_instances`) -- `borrowed` with a `'static` lifetime
  rather than a bounded `&'a`. `m_slot` is a second, separate departure from the default
  `borrowed`/non-`const`-pointee table: `slot_for()` (`InstanceLock.cpp:80-90`) hands the *same*
  `Slot&` out of its registry to every `InstanceLock` guarding the same `lock_file_path`, and
  `InstanceLock.cpp:152`'s `++m_slot->depth` under that `Slot`'s own `recursive_mutex`
  (`InstanceLock.cpp:71`) exists precisely because more than one holder can reach it at once. A
  Rust `&'static mut Slot` would be unique by construction and so could not be handed to two
  holders at the same time, which is not what the C++ does; this classification pass keeps
  `&'static Slot` (an earlier intermediate draft briefly changed this to `&'static mut Slot` under
  the general non-`const`-pointee rule and was corrected back, because that rule assumes a unique
  reference and this pointee's own mutex is what makes shared mutation through a shared reference
  sound here -- the interior mutability lives inside `Slot`, not in the reference to it). Its
  evidence cell now also cites the field's actual assignment, `InstanceLock.cpp:128`
  (`m_slot = &slot_for(lock_file_path);`), not only the registry internals inside `slot_for` that
  justify the `'static` lifetime.
- `Print.hpp:1514` `m_print_regions`: the task's own "owning spine" paragraph groups this with
  `m_objects`, but `PrintApply.cpp:2233` populates it with `.get()` from a `unique_ptr` owned by
  `PrintObjectRegions::all_regions`, and `Print.cpp:246` only `.clear()`s it, never deletes its
  elements. The committed headers disagree with that framing, so this batch follows the evidence:
  `borrowed`, `Vec<&'a mut PrintRegion>` (a non-owning aggregate cache across every `PrintObject`'s
  shared regions), not `owned`.
- `Print.hpp:678` `m_shared_regions` is the one `shared` row in this batch: `PrintObject.cpp:170`
  manually reference-counts it (`--m_shared_regions->m_ref_cnt == 0) delete`), and the header's own
  comment says it is "Shared among PrintObjects created for the same ModelObject". The default
  table's own condition for `shared` -- "when the C++ mutates the pointee through the pointer from
  more than one owner" -- is met here: `PrintApply.cpp:2215` hands the same `PrintObjectRegions*`
  to several `PrintObject`s, and `PrintApply.cpp:2230-2234` writes `m_print_region_id` through it
  from that shared pointer. An earlier draft of this cell, `Arc<PrintObjectRegions>`, missed that
  condition; the row is `Arc<Mutex<PrintObjectRegions>>`. This leaves an open tension with
  `Print.hpp:1514`'s `Vec<&'a mut PrintRegion>`, which points at `PrintRegion`s reached through this
  same `PrintObjectRegions`: a `&mut` extracted from behind a `Mutex` cannot be held the way a plain
  reference can. This plan states that tension rather than resolving it -- reconciling the two
  cells' shapes is a design decision for the Rust module that actually holds this graph, not a
  classification this pass can settle from the headers alone.

Container rows (a `Vec`, map or `std::pair` of raw pointers, or a pointer/reference *to* such a
container) take the per-element default extended mechanically to the container, the same way the
table already does for an `owned` container (`Vec<ExtrusionEntity>`, not a container of pointers):
a `borrowed` container of `const T*` becomes `Vec<&'a T>`, or `&'a [&'a T]` where the container
itself is only borrowed (`ClipperUtils.hpp:315`, `PerimeterGenerator.hpp:88`, `Print.hpp:218` --
the inner `&'a T` is kept, never flattened to `&'a [T]`, because the elements are pointers
scattered elsewhere, not a contiguous array of `T` values); a `borrowed` container of non-`const T*`
becomes `Vec<&'a mut T>` / `&'a [&'a mut T]` the same way; and a `back-reference` container (a map
keyed by a pointer into the pointing object's own owning collection, e.g. `Print.hpp:1551`'s
`m_sequential_dynamic_orderings`) becomes `HashMap<usize, V>` or `BTreeMap<usize, V>` by the same
ordering rule as any other map (below). This is one mechanical rule applied to every container row,
not a seventh per-row vocabulary.

A map's Rust container follows the C++ container's own ordering, not a blanket `HashMap`: a
`std::map` is ordered and a porter who walks it, or compares two of it element-by-element in
iteration order, relies on that order, so its row takes `BTreeMap`, never `HashMap`; a
`std::unordered_map` (this batch's only one, `GCode.hpp:825`'s `m_instance_names`) takes `HashMap`.
Picking `HashMap` for every map column 5 happens to spell `std::map` under (`Config.hpp:2152`,
`:2215`, `:2619`'s `t_config_enum_values`, `Model.hpp:1700`'s `ModelMaterialMap`,
`Config.hpp:2659`, `Print.hpp:1551`, `ProjectTask.hpp:278`/`:312`) is the idiom substitution R15's
banned list rejects: `Model.cpp:4417-4425` asserts two models' `materials` maps have equal keys at
matching iteration positions, which only holds for an ordered map.

`const`/non-`const` C++ pointees are kept faithful throughout, scoped to the pointee itself rather
than to the row's Rust type as a whole: a `const Pointee` is never reached through a `mut` or
`Mutex` Rust type, and a non-`const` `Pointee *` row's *pointee* defaults to `&'a mut Pointee`
unless the row departs (named above) or is `unknown`. `GCode.hpp:963` `slot`
(`BeltFloorObjectGuard::slot`, `const PrintObject *&`) is this batch's one row where that still
yields a `mut` in the overall type without breaking the rule: the pointee (`PrintObject`) stays
`const` and un-mutated (`&'b PrintObject`), but the field itself is a *reference to an external
pointer variable* that the guard's constructor and destructor overwrite
(`GCode.hpp:963-966`), so the outer reference is `&'a mut`. `&'a mut Option<&'b PrintObject>`
keeps both facts straight: `mut` on the slot being written through, never on the `PrintObject` it
can point at.

## Risks

The riskiest step is Step 1: a wrong `back-reference` shape for the `Print`/`PrintObject`/`Layer`
graph or the `Model`/`ModelObject`/`ModelVolume` graph is load-bearing for every later porting
task, per the task's own framing ("A wrong `borrowed`-versus-`back-reference` call here does not
stay local"). This plan mitigates that by deriving the shape mechanically from each owning
container's already-decided `owned`/`shared` class and its own default Rust type, rather than from
a fresh per-graph guess, and by keeping all three graphs' answers in one place (this plan) rather
than letting a later task re-derive them.

A secondary risk is the `Print.hpp:1514` `m_print_regions` departure from the task's own "owning
spine" framing: the evidence (`PrintApply.cpp:2233`, `Print.cpp:246`) is clear-cut, but a reviewer
who trusts the task's prose over the vendored source could read this as an error rather than a
deliberate, evidenced correction. It is called out explicitly in Step 2 and in the pull request for
that reason.

## Proof

- `just ownership-check` exits 0 and prints `ownership-check: port/ownership.tsv holds 208 row(s)`.
- `just ownership` leaves `port/ownership.tsv` byte-identical (proven by a direct diff against the
  pre-regeneration file, not just by the check passing).
- A direct comparison of this batch's 120 rows against `main` confirms zero blank `class` cells,
  zero blank `evidence` cells, and zero blank `rust type` cells. A scripted scan for a `const`
  pointee reached through `mut`/`Mutex` finds exactly one hit, `GCode.hpp:963` `slot`, and it is a
  false positive against the scan, not the rule: the `mut` is on the outer reference to the pointer
  variable, never on the `const PrintObject` pointee itself (see the container-rule paragraph in
  Step 2). No other row trips it.
- `git diff main -- port/ownership.tsv` touches exactly 120 lines, and columns 1-5 (`header path`,
  `line`, `declaring type`, `field`, `C++ type`) are byte-identical to `main` on every row.
- Every evidence cell's `path:line` citation was checked against the vendored source directly
  (`sed -n '<line>p' <file>`), not just transcribed while reading -- this is what caught the real
  mistakes this branch's history fixes (a wrong file, a call site cited instead of the delete
  inside it, and two citations thrown off by a `latin-1`-decoding bug in the checking script
  itself: a stray UTF-8 continuation byte inside a Chinese comment decodes as U+0085, which
  Python's `splitlines()` treats as its own line break, desyncing line numbers from what
  `grep`/`sed` report for every line after it in a file with non-ASCII comments -- common in this
  codebase's BBS-authored sections). A sibling batch re-using this method should decode source as
  UTF-8, not `latin-1`, when counting lines for a citation.
