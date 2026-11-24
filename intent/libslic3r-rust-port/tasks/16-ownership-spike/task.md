---
kind: task
step: 12
delivers: [R30]
depends: [14-manifest-columns, 15-task-cuts]
size: 5
---
# Task: Spike: list every in-scope raw-pointer field and batch the classification

## What to build

This is the spike of step 12 of `intent/libslic3r-rust-port/plan.md`, "The ownership manifest". It
is the only task cut from that step so far, which is why its `delivers` is the whole step's. It
classifies nothing.

**The question.** How many raw-pointer fields — and fields that are containers of raw pointers —
does the in-scope header surface declare, and how do those fields batch into classification tasks?
Nothing on `main` answers it. The only figures on record are the spec's note under its ownership
concern that the headers carry 61 raw-pointer-container *signatures*, and that scope is roughly
420 files; neither is a field count, and neither says how the fields distribute across subsystems.
Until the field list exists no classification task can be sized, so this task commits the list and
the batching and stops there. **The rest of step 12 is cut in a later pass once this spike is
answered**: one task per row of `port/ownership-batches.tsv`, each classifying its own rows with
cited evidence, plus the last task of the step, which removes the blank-class allowance described
below.

**What it waits for.** `14-manifest-columns`, for a committed `port/manifest.tsv` whose
`in scope or out with a reason`, `subsystem` and `crate-level level` columns this task reads — the
first of those is the committed header text `03-manifest-file-set` writes, and there is no column
spelled `in scope`; and `15-task-cuts`, for `port/cut-tasks`
and `just task-cuts <phase>`. `just task-cuts ownership` reads `port/ownership.tsv` for its field
list and exits non-zero rather than batching nothing, so the order inside this task is: write
`port/ownership`, run `just ownership` to produce the field list, then run `just task-cuts ownership`.

**Add `port/ownership`**, a standalone executable with PEP 723 inline metadata run through `uv`,
like every other `port/*` script without an extension, plus two `justfile` recipes:

- `just ownership` rewrites the committed `port/ownership.tsv`. It is for a person regenerating
  deliberately.
- `just ownership-check` regenerates the field list into a scratch directory and fails when the
  committed file disagrees with the headers at the pin. This is the variant that becomes part of
  `just port-check` — added there by `24-port-check-ci`, not by this task.

**Commit `port/ownership.tsv`**: one row per raw-pointer field, and per field that is a container of
raw pointers, declared in a header that `port/manifest.tsv` marks in scope. **Its header row is
this, literally**, because two other tasks read it by column name — `15-task-cuts`'s `ownership`
phase joins back to `port/manifest.tsv` on `header path` to recover a field's `subsystem` and
`crate-level level`, which this file does not carry, and `22-port-status-gates` reads `class`:

```
header path	line	declaring type	field	C++ type	class	evidence	rust type
```

`class` is one of `owned`, `shared`, `borrowed`, `back-reference`, `ffi` or `unknown`; `evidence`
holds the `path:line` of the evidence for that class; `rust type` holds the Rust type porters use.

**The trap that decides whether the row count is right.** A field whose declared type is an alias
of a pointer container shows no `*` at its declaration site, so a pattern over field declarations
that looks for `*` misses it. At the pin these aliases include
`ExtrusionEntitiesPtr` (`src/libslic3r/ExtrusionEntity.hpp:172`), `LayerPtrs` and `LayerRegionPtrs`
(`Layer.hpp:27`, `:29`), `LayerPtrs` and `SupportLayerPtrs` again in `Print.hpp:221`, `:228`,
`PrintObjectPtrs` and `PrintRegionPtrs` (`Print.hpp:1008`, `:1015`), `ModelObjectPtrs`,
`ModelVolumePtrs`, `ModelInstancePtrs` (`Model.hpp:138-140`, and `ModelObjectPtrs` again in
`CutUtils.hpp:13`, `ModelInstancePtrs` again in `ModelArrange.hpp:15`), `PointPtrs`
(`Point.hpp:74`), `PolygonPtrs` (`Polygon.hpp:23`), `SupportGeneratorLayersPtr`
(`Support/SupportLayer.hpp:146`), `PlateDataPtrs` (`Format/bbs_3mf.hpp:212`) and the class-nested
`NodeSet` (`Arachne/SkeletalTrapezoidation.hpp:88`, an `ankerl::unordered_dense::set<node_t*>`).
The script builds that alias set from the in-scope headers themselves — every `typedef` or `using`
whose right-hand side holds a pointer, including one declared inside a class — and then treats a
field of an alias type as a container-of-raw-pointers row. It does not read the list above as data;
the list is here so a builder knows the case exists and can check the script's answer against it.

**What `just ownership` preserves.** Like the manifest's judgement columns, `class`, the evidence
cell and the Rust type are carried forward verbatim from the committed file: a rewrite recomputes
header path, line, declaring type, field and C++ type from the vendored headers, keeps those three
cells exactly as it finds them, and writes a new row with a blank `class` for a field that has
none. Running it twice leaves the file byte-identical. Output is deterministic and byte-stable —
rows sorted by header path then line, paths relative to the repository root, no timestamp and no
machine-specific path — because `just ownership-check` diffs the regenerated list.

**What `just ownership-check` fails on.** A field in the headers with no row; a row naming a field
that does not exist at the pin; a row whose `class` is filled in, is not `unknown`, and has no
evidence; and an `unknown` row missing either the `path:line` of the use that leaves the class
unclear or the conservative Rust type a porter must use until it is resolved.

**The blank-class allowance, which this task implements and must not remove.** A row whose `class`
cell is blank is accepted while step 12's batch tasks are open, because this spike commits a row
per field before any field is classified. Step 12 is done when no in-scope row's class is blank,
and the step's *last* task — cut later, not this one — removes the allowance from `port/ownership`
so that a blank class is a failure from then on. The kind of evidence the classification tasks will
cite is comments over raw pointers rather than types: `ExtrusionEntityCollection.hpp:39`
`ExtrusionEntitiesPtr entities;     // we own these entities` against `:19`
`// Removed elements are NOT being deleted.`, and `Model.hpp:189` `Model *m_model;` under
`// Parent, owning this material.` as a back-reference.

**Then run `just task-cuts ownership`** and commit `port/ownership-batches.tsv`, which batches the
fields by subsystem into rows of at most 120 fields each, in the manifest's crate-level order. That
file is the spike's answer: its row count is how many classification tasks step 12 gets, and each
row is one of them.

**What makes this a 5.** The unknown is the size and the shape of the answer — how many rows the
raw-pointer surface of roughly 420 in-scope headers has and how they batch — and that is what makes
it a spike rather than a sized task. The work beside the unknown is real: listing raw-pointer fields
and containers of them out of C++ headers means resolving the alias set above, finding fields
declared inside nested and templated classes, and not counting a pointer in a function signature or
a local variable as a field. It is not split further because the field list, the check and the
batching are one script's output and splitting them would leave a check with nothing to check.

**`just ownership-check` prints the row count of `port/ownership.tsv` on success**, which is the
plan's recorded proof line for R30 and which no other task builds: this is the only task that
creates `port/ownership` and that recipe.

**What this task does not do.** It fills in no `class` cell. It adds no line to `.asdlc/checks` and
none to `just port-check` — `24-port-check-ci` puts `just ownership-check` there. It does not print
the `unknown` count; that is `just port-status`, built by `21-port-status-markers` and
`22-port-status-gates`. It does not edit `port/manifest.tsv`, `docs/PORTING.md` (which
`19-porting-contract` writes, including the instruction to take the Rust type from this file
verbatim), or anything under `vendor/OrcaSlicer/`. `port/ownership.tsv` is not an oracle output and
carries no provenance header.

## Done when

- `port/ownership` exists as a `uv`-run standalone script with PEP 723 inline metadata, and
  `just ownership` and `just ownership-check` are recipes in the `justfile`.
- `port/ownership.tsv`'s first line is exactly the header row written above —
  `header path`, `line`, `declaring type`, `field`, `C++ type`, `class`, `evidence`, `rust type`,
  tab-separated — which is what `port/cut-tasks` joins on and what `port/port-status` reads `class`
  from.
- `port/ownership.tsv` is committed with the eight columns named above and holds one row per
  raw-pointer field and per container-of-raw-pointers field declared in every header
  `port/manifest.tsv` marks in scope, the alias-typed fields included.
- `just ownership-check` exits 0 against the committed tree and prints the row count of
  `port/ownership.tsv`.
- `just ownership-check` exits non-zero in each of these four cases, and the pull request shows each
  failure: a row deleted; a row added for a field that does not exist at the pin; a row whose class
  is filled in, is not `unknown`, and whose evidence cell is empty; an `unknown` row with no
  evidence `path:line` or no Rust type.
- Running `just ownership` twice leaves `port/ownership.tsv` byte-identical, and running it after a
  `class`, evidence or Rust-type cell has been filled in by hand leaves that cell unchanged.
- No row's `class` cell is filled in by this task, and `port/ownership` still accepts a blank class.
- `port/ownership-batches.tsv` is committed, was written by `just task-cuts ownership`, batches by
  subsystem in crate-level order with at most 120 fields in a row, and every field in
  `port/ownership.tsv` appears in exactly one of its rows.
- The spike's answer is readable from those two committed files, both outside `intent/`: the field
  count is the row count of `port/ownership.tsv` and the number of classification tasks step 12 gets
  is the row count of `port/ownership-batches.tsv`.
- `port/ownership.tsv` and `port/ownership-batches.tsv` carry no provenance header.
- `port/manifest.tsv`, `.asdlc/checks`, `just port-check` and everything under `docs/` are unchanged.
- Nothing under `vendor/OrcaSlicer/` changed and the submodule pointer is still
  `f92a7c065c5718b8ef1baac1cd704df2911238f4`.
- The branch changes no file under `intent/`, so `just guard` passes, and `just check` passes.
