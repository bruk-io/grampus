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

**Rows this plan leaves `unknown`, and why** (9 rows): `GCodeWriter.hpp:402` `buf_end` is a pointer
into `GCodeFormatter`'s own inline `buf[buflen]` array -- always `buf + buflen`, a self-referential
cursor the vocabulary's six classes do not describe cleanly (not freed, not external, not a
parent/child cycle). Its rust type departs from the `unknown` default's mechanical
`Arc<Mutex<Pointee>>`: that type is nonsense for a buffer cursor (there is nothing to share or
mutex-guard -- the pointer's whole content is `buf + buflen`, a compile-time constant), so this row
carries `usize` instead, the offset a porter can actually use once `buf` becomes a plain
`[u8; buflen]` array. `Layer.hpp:344` `area` (`AreaGroup::area`) has no use anywhere under
`vendor/OrcaSlicer/src/libslic3r` beyond its own declaration and constructor, so nothing settles
its ownership. The remaining seven are all of `ProjectTask.hpp`'s cross-pointer fields except
`subtasks` (owned, confirmed by an explicit `delete`/`new` pair in `ProjectTask.cpp`) and
`parent_task_` (back-reference into that same owned `subtasks`): `profile_` (`BBLSliceInfo` and
`BBLTask`), `project_`, `tasks`, and both `slice_info` maps. `ProjectTask.cpp` only ever assigns
these from a constructor parameter or looks them up (`get_slice_info`); nothing under
`src/libslic3r` ever `push_back`s into `tasks`/`profiles` or deletes a `profile_`/`project_`/
`slice_info` pointee -- the object graph that would settle it (if one exists) is built in GUI code
under `src/slic3r/GUI`, which is out of scope for this header's own evidence. These seven, plus
`Layer.hpp:344` `area`, carry the conservative `Arc<Mutex<Pointee>>` (the pointee's own pointer is
never `const`, so a porter must assume concurrent mutation until a later pass resolves it); only
`buf_end` departs from that default, for the reason given above.

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
- Every evidence cell's `path:line` citation was checked against the vendored source directly
  (`sed -n '<line>p' <file>`), not just transcribed while reading -- this is what caught the real
  mistakes this branch's history fixes (a wrong file, a call site cited instead of the delete
  inside it, and two citations thrown off by a `latin-1`-decoding bug in the checking script
  itself: a stray UTF-8 continuation byte inside a Chinese comment decodes as U+0085, which
  Python's `splitlines()` treats as its own line break, desyncing line numbers from what
  `grep`/`sed` report for every line after it in a file with non-ASCII comments -- common in this
  codebase's BBS-authored sections). A sibling batch re-using this method should decode source as
  UTF-8, not `latin-1`, when counting lines for a citation.
