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
if it does, `usize` (an index into the owning collection) otherwise. This batch's `owned` rows in
graphs (a) and (b) are, without exception, plain `Vec<Pointee>` or `HashMap<K, Pointee>` by value
(`Print::m_objects` deletes each `PrintObject*` directly, `Model::materials` deletes each
`ModelMaterial*` directly, and so on for `m_layers`, `m_support_layers`, `Layer::m_regions`,
`Model::objects`, `ModelObject::volumes`/`instances` -- see the per-row evidence in
`port/ownership.tsv`). None of those owners is `Arc`, so every back-reference in graphs (a) and (b)
resolves to **`usize`**: `LayerRegion::m_layer` is an index into `PrintObject::m_layers`,
`Layer::m_object` an index into `Print::m_objects`, `Layer::upper_layer`/`lower_layer` and
`Print.hpp:718`'s `m_shared_object` are indices into the same Vec the pointing object itself lives
in, `ModelObject::m_model`/`ModelMaterial::m_model` are indices into... no single Vec (`Model` is
the singular root, not a Vec element) -- but graph (b)'s own pointees that *are* Vec elements
(`ModelVolume::object`, `ModelInstance::object`, both indices into `Model::objects`) get the same
`usize` shape, and this plan applies that one shape across the whole graph rather than splitting it
by pointee, per the task's "one answer per graph, not per row."

Graph (c), the `PrintBase.hpp` pair, is different: both `PrintTryCancel::m_print` and
`PrintObjectBaseWithState::m_print` reference the single `Print`/`PrintBase` instance driving the
whole slice, not an element of any Vec this batch classifies -- there is no "owning collection" to
index into. `PrintBase`'s own cancellation machinery (`state_mutex`, `cancel_callback`,
`throw_if_canceled()`) exists specifically so a worker thread and a cancelling caller can both
reach the same `Print` across a thread boundary, which is exactly the shape `Arc<Print>` is for in
the port. Graph (c) therefore resolves to **`Weak<Pointee>`** (`Weak<PrintBase>`,
`Weak<PrintType>`), on the assumption that whatever owns the live `Print` during slicing holds an
`Arc<Print>`. This is the one graph where this batch departs from "index," named here because a
later task should not have to re-derive it from a different sibling batch's guess.

### Step 2: Classify every row with cited evidence, applying the per-class defaults
- delivers: R30

Fill `class`, `evidence` and `rust type` for the 120 rows `44-ownership-class-root`'s task.md
names (the `root` subsystem: every in-scope header directly under `src/libslic3r`), reading the
vendored header plus, where the header's own comment does not settle it, the corresponding `.cpp`
in the same tree. `port/ownership.tsv` carries the result; this plan does not duplicate the table.

**Rows this plan leaves `unknown`, and why** (9 rows): `GCodeWriter.hpp:402` `buf_end` is a pointer
into `GCodeFormatter`'s own inline `buf[buflen]` array -- always `buf + buflen`, a self-referential
cursor the vocabulary's six classes do not describe cleanly (not freed, not external, not a
parent/child cycle). `Layer.hpp:344` `area` (`AreaGroup::area`) has no use anywhere under
`vendor/OrcaSlicer/src/libslic3r` beyond its own declaration and constructor, so nothing settles
its ownership. The remaining seven are all of `ProjectTask.hpp`'s cross-pointer fields except
`subtasks` (owned, confirmed by an explicit `delete`/`new` pair in `ProjectTask.cpp`) and
`parent_task_` (back-reference into that same owned `subtasks`): `profile_` (`BBLSliceInfo` and
`BBLTask`), `project_`, `tasks`, and both `slice_info` maps. `ProjectTask.cpp` only ever assigns
these from a constructor parameter or looks them up (`get_slice_info`); nothing under
`src/libslic3r` ever `push_back`s into `tasks`/`profiles` or deletes a `profile_`/`project_`/
`slice_info` pointee -- the object graph that would settle it (if one exists) is built in GUI code
under `src/slic3r/GUI`, which is out of scope for this header's own evidence. Each carries a
conservative `Arc<Mutex<Pointee>>` (the pointee's own pointer is never `const`, so a porter must
assume concurrent mutation until a later pass resolves it).

**Rows this plan departs from a mechanical default, named individually:**
- `Config.hpp:2636` `nocli`: `static const constexpr char *` is a string-literal constant, not a
  pointer at an object -- `borrowed` with `&'static str`, not the generic `&'a Pointee`.
- `clonable_ptr.hpp:141` `px`: its copy constructor deep-clones (`rhs.px->clone()`) and its
  destructor `delete`s -- `owned`, `Box<T>`.
- `Print.hpp:218` `m_data` (`ConstVectorOfPtrsAdaptor::m_data`): a pointer to a vector of pointers,
  held by a view that owns neither the vector nor its elements -- `borrowed`, `&'a [T]`.
- `InstanceLock.hpp:55` `m_slot` and `FlushVolPredictor.hpp:48` `predictor`: both point into a
  process-lifetime, never-freed static cache (`InstanceLock.cpp`'s leaked registry map;
  `FlushVolPredictor.cpp`'s static `predictor_instances`) -- `borrowed` with `&'static Pointee`
  rather than a bounded `&'a`.
- `Print.hpp:1514` `m_print_regions`: the task's own "owning spine" paragraph groups this with
  `m_objects`, but `PrintApply.cpp:2233` populates it with `.get()` from a `unique_ptr` owned by
  `PrintObjectRegions::all_regions`, and `Print.cpp:246` only `.clear()`s it, never deletes its
  elements. The committed headers disagree with that framing, so this batch follows the evidence:
  `borrowed`, `Vec<&'a mut PrintRegion>` (a non-owning aggregate cache across every `PrintObject`'s
  shared regions), not `owned`.
- `Print.hpp:678` `m_shared_regions` is the one `shared` row in this batch: `PrintObject.cpp:170`
  manually reference-counts it (`--m_shared_regions->m_ref_cnt == 0) delete`), and the header's own
  comment says it is "Shared among PrintObjects created for the same ModelObject" --
  `Arc<PrintObjectRegions>`.

Container rows (a `Vec`, `HashMap` or `std::pair` of raw pointers) take the per-element default
extended mechanically to the container, the same way the table already does for an `owned`
container (`Vec<ExtrusionEntity>`, not a container of pointers): a `borrowed` container of
`const T*` becomes `Vec<&'a T>` / `HashMap<K, &'a T>`, a `borrowed` container of non-`const T*`
becomes `Vec<&'a mut T>`, and a `back-reference` container (a map keyed by a pointer into the
pointing object's own owning collection, e.g. `Print.hpp:1551`'s `m_sequential_dynamic_orderings`)
becomes `HashMap<usize, V>`. This is one mechanical rule applied to every container row, not a
seventh per-row vocabulary.

`const`/non-`const` C++ pointees are kept faithful throughout: a `const Pointee *` row never takes
a `mut` or `Mutex` Rust type, and a non-`const` `Pointee *` row defaults to `&'a mut Pointee` unless
the row departs (named above) or is `unknown`.

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
  zero blank `evidence` cells, zero blank `rust type` cells, and zero rows where a `const` C++
  pointee's Rust type carries `mut` or `Mutex`.
- `git diff main -- port/ownership.tsv` touches exactly 120 lines, and columns 1-5 (`header path`,
  `line`, `declaring type`, `field`, `C++ type`) are byte-identical to `main` on every row.
