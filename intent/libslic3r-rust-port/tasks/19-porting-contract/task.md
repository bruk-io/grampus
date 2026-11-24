---
kind: task
step: 15
delivers: [R12, R13]
depends: [07-crate-cut, 14-manifest-columns, 16-ownership-spike, 17-order-audits]
size: 5
---
# Task: Write the per-file translation contract

## What to build

Plan step 15 writes two documents and is split in two. This is **part 1: `docs/PORTING.md`**, the
per-file translation contract (R12). Part 2 is `20-workflow-document`, which writes
`docs/WORKFLOW.md`; the frontmatter's `delivers` is the whole step's, so do not write
`docs/WORKFLOW.md` here and do not edit it.

Create `docs/PORTING.md` and nothing else; create `docs/` if plan step 1 has not already created
it for `docs/ORACLE.md`. Write no code: the command that checks this document's sections
(`just port-status`, R29) is written by `22-port-status-gates`, and the trial run that corrects
this document is `35-trial-run`.

Every unit of Phase A and Phase B is held to this document, and a porter is given its task, this
repository and nothing else. Write it so a porter never has to guess and never has to ask.

### What it must contain, and under which headings

`22-port-status-gates` writes a check that reads this document's headings, so the headings are a
contract between the two tasks. Each one below is spelled as R12 and R29 name that section, so a
check written from the requirement's own words finds it. That task's builder reads the headings
from `docs/PORTING.md` as it stands on `main`, not from a list typed into its own body, so if a
heading here and the check ever disagree, this document is the authority. Use exactly these, at
exactly these levels, and say in the document's opening paragraph that `just port-status` reads
them and that renaming one breaks the check:

- `## Crate map`
- `## Type map`
- `## Idiom map`
- `## Don't-translate list`
- `## Banned-substitution list`
- `## Rules`, holding the six rules as `###` subsections, in this order:
  `### Scaling conversions`, `### Float widths`, `### Rounding modes`, `### Error sets`,
  `### Number formatting`, `### Linked libraries`.

**Crate map.** Derived from `port/cut.tsv` and `port/manifest.tsv`, never authored: one row per
crate the cut names, with its crate-level level, and the subsystems it holds. Read the artifacts'
header rows for the column spellings rather than assuming them — the idiom is
`awk -F'\t' 'NR==1{for(i=1;i<=NF;i++)c[$i]=i} NR>1 {print $c["target crate"]}' port/manifest.tsv`.
The section states that the artifacts are the authority, names the command a reader runs to
re-derive the table, and states that an agent may not invent a crate, add one, or move a file
between crates: the crate map changes only through a pull request that regenerates the manifest,
which is C10's recorded decision. Do not list libslic3r source files one by one, and do not
restate the cut's edges or the per-file scope decisions: those live in the artifacts, and R8 is
explicit that no later document restates them.

**Type map.** One row per C++ type a porter meets, with the Rust type and the authority for it.
Fix here, with citations: `coord_t` is `int64_t` at the pin — the `int32_t` branch at
`src/libslic3r/libslic3r.h:45` is inside an `#if 0`, and the live definition is `:48` — so
`coord_t` is `i64` and nothing in the port makes it `i32`; `coordf_t` is `double`
(`libslic3r.h:51`), so `f64`; `float` is `f32` and `double` is `f64` (see `### Float widths`);
`size_t` is `usize`; `std::string` is `String`; `std::optional` is `Option`; `std::variant` is an
enum; `std::map` is a key-ordered map and `std::vector` is `Vec`. An `enum class` keeps the
original's discriminant values, because they reach serialised output. For every raw-pointer field
the row says: take the Rust type from `port/ownership.tsv` verbatim, by header path and line, and
never choose one — that file is R30's artifact and this instruction is what makes it binding.
For the foundation types (`Point`, the Eigen vector aliases at
`src/libslic3r/Point.hpp:52-69`, `Polygon`, `ExPolygon`, the configuration types) the row names
the crate and module from the crate map that defines them and states that the Rust spelling is the
one that module exports: those modules are hand-designed by plan step 20, after this document, and
a porter reads the module rather than inventing a second spelling.

For a `std::unordered_map`, `std::unordered_set` or hash-keyed container whose iteration order
reaches output, the row gives **one** insertion-ordered spelling for the whole port and names the
crate that provides it. A port task may not add a dependency (`Cargo.lock` is one of the seven
paths it may not write), so if that spelling needs a third-party crate, say so and name the
crate-skeleton task of plan step 20, `34-foundation-spike`, as the task that adds it; if it is
hand-rolled, name the
foundation crate that holds it. `17-order-audits` has already written
`port/reports/order-containers.md`; read it and make sure the spelling you fix covers the shapes
that report actually lists.

**Idiom map.** One row per C++ construct, with the single rendering the whole port uses, because a
porter who chooses differently changes every call site. Cover at least: a method on a class; a
free function in an anonymous namespace or a `static` free function (which is proof path 3, so the
row says the test translation unit is how it is proven, not that it should be made public); a
non-const reference out-parameter; a destructor against `Drop`; virtual dispatch over a class
hierarchy such as `ExtrusionEntity`; a template; a `#define` numeric constant (which becomes a
`const` carrying `// PORT SRC:`); `std::sort` against `std::stable_sort`; `tbb::parallel_for` and
`tbb::parallel_reduce`; iterator and pointer arithmetic against slices and indices; and
`operator<<` or other serialisation. For the TBB rows the rule is R18's and R15's together: keep
the order the original's result depends on, sum in a fixed order, and where the port is serial
because the original's reduction order is load-bearing, say that the module carries a
`// PERF(port):` marker and nothing else — that marker is the only place a deferred optimization
may live.

**Don't-translate list.** What a porter stops at rather than ports: out-of-scope code reached from an
in-scope file, with the manifest row's reason as the authority and a `// PORT NOTE:` at the
boundary; the libraries R17 binds, which are called through `crates/slic3r-ffi` (see
`### Linked libraries`); upstream code that is commented out, naming
`src/libslic3r/Point.hpp:676` and `src/libslic3r/libslic3r.h:105` as the two examples this
contract depends on — neither is ported and neither is revived; debug-only blocks and SVG dumps
behind `SLIC3R_DEBUG*` guards; the GUI and plate layer (R10); the preset layer and the
project-format readers (R11 and the manifest's deductions); the SLA implementations, with
`SLA/IndexedMesh` named as the one `SLA/` unit that is in scope (R9); arrange and orient (C14's
decision); the two Objective-C++ files, which the manifest deducts, so no porting rule has to
cover Objective-C++; and an `assert` body, which the oracle's release build compiles out (see
`### Error sets`).

**Banned-substitution list.** Each entry says what is banned and names the check that catches it where
one exists. At least: rounding the scaling conversions; a hash-ordered map where iteration order
reaches output; a parallel reduction where the original's order is load-bearing; replacing one of
R17's libraries with a Rust crate (`just port-check`'s substitute check fails it); widening a
`float` to `f64` or narrowing an `f64`; any fused multiply-add, including `f64::mul_add`, because
the normative oracle is built with `-ffp-contract=off` and never fuses; multiplying by the
reciprocal of `SCALING_FACTOR` instead of dividing by it; locale-dependent or `format!`-based
number formatting in emitted G-code; returning `Result` from a function that cannot fail, and a
live `panic!` where the oracle compiles the assertion out; and a comparator that is not a total
order. Say for the last one that the port's total-order comparator is a deliberate divergence
where the oracle's is not a total order, that the site carries a `// FIDELITY(port):` marker, and
that the marker is what puts it in `port/reports/divergences.md`;
`17-order-audits` has written `port/reports/order-comparators.md`, so read it and make the entry
match the sites it lists.

### The six rules, with their citations

Transcribe each with a `vendor/OrcaSlicer/`-relative `path:line` citation. These are the lines
this task is a 5 for; get them exactly right.

**Scaling conversions — stated per API, because the five entry points do not behave alike.**
`scaled<coord_t>()`, the integral overload (`src/libslic3r/Point.hpp:669-678`, body at `:677`),
is `Tout(v / Tin(SCALING_FACTOR))` with an integral `Tout`: a division converted to an
integer, so it truncates toward zero. Port it as truncating division. Adding rounding is banned,
and the reason is in the original: the `std::round` at `Point.hpp:676` is commented out, under the
`TODO: is the rounding necessary?` comment at `:670-672` that reads as an invitation. The Eigen
overload (`Point.hpp:680-690`, body at `:689`) is `(v / SCALING_FACTOR).template cast<Tout>()`,
which truncates per element the same way. The floating-output `scaled<Tout>()`
(`Point.hpp:660-667`, body at `:666`) does not truncate.

**Each of these divides at the input's precision, and the document says so**, because a porter who
widens first diverges in the low bits at exactly the sites the rule exists to protect. The global is
cast to `Tin`, the *input's* type, at both `Point.hpp:666` and `:677`, so `scaled<coord_t>(0.1f)`
divides in `f32` and only then converts; the Eigen overload at `:689` divides by a bare
`SCALING_FACTOR`, which Eigen's scalar `operator/` coerces to the matrix's own `Scalar`, so a
`Vec2f` likewise divides in `f32`. So the port divides a `f32` input in `f32` and a `f64` input in
`f64` and **never widens the input to `f64` before dividing**. The two macros are different again:
`scale_()` (`libslic3r.h:106`) and `unscale_()` (`:107`) carry no cast at all, so their precision is
whatever the usual arithmetic conversions give at the call site, and the document says to read the
call site rather than the macro. The `scale_()` macro
(`libslic3r.h:106`) is a plain `double` division and truncates nothing — the rounding variant
above it at `:105` is commented out — so where the original converts the result of either of
these to an integer type, the port reproduces that truncation **at the call site** and not inside
the API. `unscale_()` (`libslic3r.h:107`), `unscale<T>()` (`libslic3r.h:137-138`), `unscaled<T>()`
(`Point.hpp:692-700`, body at `:699`) and its Eigen overload (`Point.hpp:702-714`, body at `:713`)
are multiplications by the global and never truncate. In every case the port divides by the
global and never multiplies by its reciprocal.

State the global itself, because it is not a constant: `SCALING_FACTOR` is
`extern double` (`libslic3r.h:80`), initialised to `SCALING_FACTOR_INTERNAL`, `0.000001`
(`libslic3r.h:65`), at `src/libslic3r/libslic3r.cpp:3`, and the only assignments anywhere in the
tree are `src/slic3r/GUI/Plater.cpp:15015` and `:15017`, which set the large-printer value
`0.00001` (`libslic3r.h:66`) above `LARGE_BED_THRESHOLD` `2147` (`:67`) and which R10 puts out of
scope. So the port's surface sees `0.000001`. Fix one spelling for it in the document — a module
does not write the literal inline — and say that R7 records its value with every oracle output.

**Float widths.** Mirror `f32` and `f64` exactly where the original uses `float` and `double`;
widening a `float` field changes every emitted E value.

**Rounding modes. There is no universal rule, and the document must not state one** — the spec's own
wording scopes ties-away to two places and no further: **ties away from zero in the G-code formatter
and in Clipper2 point construction, ties to even in Clipper2 intersections, and every remaining site
ported individually.** Write that, and then enumerate the integer-conversion idioms that actually
occur in in-scope files, each with its citation, because a sentence saying every rounding site is
`std::round` would send every porter to `f64::round` and be wrong at several sites that reach the
emitted G-code.

`std::round` is ties away from zero, which is what Rust's `f64::round` does, and it is the idiom at
the two formatter sites that reach the compared artifact: `int64_t(std::round(v * pow_10[digits]))`
in `GCodeFormatter::emit_axis` (`src/libslic3r/GCodeWriter.cpp:1696`) and in
`GCodeFormatter::quantize` (`src/libslic3r/GCodeWriter.hpp:341`). Do not cite
`GCodeWriter.cpp:782`, which is flavour-gated to MakerWare and Sailfish, and do not cite the
`TextureDisplacement` or `SLA/` sites, which are out of scope.

The other idioms, which the document names and which a porter transcribes as written:

- **`floor(x + 0.5)` is round-half-up, not round-half-away, and is not `f64::round`.** They differ
  at an exact negative half — `floor(-0.5 + 0.5)` is `0` while `(-0.5f64).round()` is `-1.0` — and
  at `0.49999999999999994`. Transcribe it as `(x + 0.5).floor()`. The sites that reach the emitted
  G-code are `src/libslic3r/GCode.cpp:9467`, `(unsigned int)floor(acceleration + 0.5)`, the M204
  and M201 acceleration value; `src/libslic3r/GCode/CoolingBuffer.cpp:995`,
  `int(floor(60. * line->feedrate + 0.5))`, a rewritten `F` word;
  `src/libslic3r/GCode/WipeTower.cpp:1616`, `int(floor(f + 0.5f))`, another `F` word — note the
  `0.5f`, so that one rounds in `f32`; `src/libslic3r/Config.hpp:1023`,
  `int(floor(_value + 0.5))` in `ConfigOptionInt`; and `src/libslic3r/Fill/FillLine.cpp:78`.
- **`lround` and `llround` are ties away from zero**, like `std::round`, and appear where a cast
  would truncate: `src/libslic3r/BeltBrim.cpp:39-40` uses `std::llround` under a comment saying
  exactly that.
- **`fast_round_up<I>()` (`src/libslic3r/libslic3r.h:427-433`) is its own thing** and its body is
  the proof that `floor(a + 0.5)` is not round-half-away:
  `a == 0.49999999999999994 ? I(0) : I(floor(a + 0.5))`. Port it as that expression, special case
  included, and never as `f64::round`.

Ties to even
lives in Clipper2, which is bound and not ported: its point construction uses `std::round`
(`vendor/OrcaSlicer/deps_src/clipper2/Clipper2Lib/include/clipper2/clipper.core.h:132-133` and
`:186-187`) while its intersection arithmetic uses `nearbyint`
(`clipper.core.h:930-931`, `:1066-1067`,
`vendor/OrcaSlicer/deps_src/clipper2/Clipper2Lib/src/clipper.engine.cpp:157`), which in the
default rounding mode is ties to even. Say in the document that those sites are behind the FFI
bridge, that a port of any site which does its own ties-to-even conversion writes it out
explicitly because Rust has no ties-to-even rounding method, and that a C++ `double`-to-integer
conversion is a truncation, which `as i64` matches.

**Error sets.** Never widen or narrow one. A function that cannot fail does not return `Result`.
An `assert` is compiled out of the oracle's release build, so it becomes `debug_assert!` and never
a live panic. Name libslic3r's own exception hierarchy from `src/libslic3r/Exception.hpp:9-49` —
`Exception` and under it `CriticalException`, `RuntimeError`, `LogicError`, `HardCrash`,
`InvalidArgument`, `OutOfRange`, `IOError`, `FileIOError`, `HostNetworkError`, `ExportError`,
`PlaceholderParserError`, `SlicingError` with its `objectId` (`:31-40`) and `SlicingErrors`
(`:42-49`) — and give the one mapping the port uses, so a `throw` site and its `catch` keep the
same set.

**Number formatting.** Format locale-independently and transcribe `GCodeFormatter::emit_axis`
(`src/libslic3r/GCodeWriter.cpp:1690-1747`, whose live body ends at `:1731` and whose
`#if 0` debug block at `:1733-1746` is dead code the port does not carry) verbatim: the digit
counts from `src/libslic3r/GCodeWriter.hpp:334-336` — `xyz_digits` 3, or 4 with high precision;
`e_digits` 5, or 6; `F_EXPORT_DIGITS` 3 — then the zero padding when the integer has fewer digits than the
decimal count, the decimal-point insertion, the trailing-zero stripping, and the fixups that put
back a leading `0` for an empty integer part or a bare `-`. Say that the `__APPLE__` branch at
`GCodeWriter.cpp:1698-1704` chooses `boost::spirit::karma::generate` over `std::to_chars` for the
same `int64_t` and therefore changes nothing the port has to reproduce.

**Linked libraries.** Never reimplement one in this intent (R17): Clipper2 — whose vendored copy
is patched, so no published crate matches it — admesh, libnest2d, qhull, Boost.Polygon's Voronoi
builder, and NLopt only if the manifest marks an in-scope caller. They are called through
`crates/slic3r-ffi`, `just port-check` fails a crate manifest that depends on a named substitute,
and C4's decision leaves replacing them to a later intent.

### What this task waits on, and what it must not touch

It waits on `07-crate-cut` for `port/cut.tsv` and the crate-level levels, on `14-manifest-columns`
for the manifest's column semantics and which columns are judgements carried forward, on
`17-order-audits` for `port/reports/order-containers.md` and `port/reports/order-comparators.md`,
and on `16-ownership-spike` for `port/ownership.tsv` — which it needs only for the file's
existence and its columns, not for its classifications. The classification tasks cut from
`port/ownership-batches.tsv` in a later pass do **not** block this task, so the type map says what
a porter does with such a row — and **the two cases are different, because only one of them has a
Rust type to take**:

- **An `unknown` row**: take the conservative Rust type the row names. `just ownership-check` fails
  an `unknown` row that carries no conservative Rust type, so that cell is guaranteed present.
- **A row whose `class` cell is still blank**: there is no Rust type yet. `16-ownership-spike` fills
  in no `class` cell and no check requires a blank-class row's Rust-type cell to be filled, so the
  cell is guaranteed *empty* for the whole window in which step 12's classification tasks are open —
  which is the window the fan-out drafts are written in. So the porter does not choose one: leave
  the field with a `// TODO(port):` marker naming that row's header path and line, and say in the
  document that the field is filled in by the step-12 classification task cut from
  `port/ownership-batches.tsv` for that row's batch. **No module may invent a Rust type for a
  blank-class row**, which is the whole point of R30 coming before fan-out.

Touch no file but `docs/PORTING.md`. In particular, write nothing in `port/`, nothing in
`crates/`, no `justfile` recipe, no line in `.asdlc/checks` and no line in
`.github/workflows/ci.yml`: `24-port-check-ci` owns the checks and the CI jobs. Expect this
document to be corrected by `35-trial-run`, which ports three units through the full loop and
fixes whatever the three show the contract gets wrong; that correction is that task's work and not
a reason to leave anything vague here.

**Why this is a 5.** Not for its volume. Every later port unit — hundreds of modules across
roughly 420 in-scope files — is held to this document, so a wrong line here is wrong in hundreds
of modules at once, and the scaling rule alone has to be right about five entry points that do not
behave alike, two of which have their rounding commented out in the original. It was not split
further than part 1 and part 2 of the step because the crate map, the type map, the idiom map and
the six rules are one argument about one codebase: splitting them across branches would let two
agents fix two different spellings for the same type.

## Done when

- `docs/PORTING.md` exists and holds, at exactly the levels named above, the headings
  `## Crate map`, `## Type map`, `## Idiom map`, `## Don't-translate list`,
  `## Banned-substitution list`
  and `## Rules`, with `### Scaling conversions`, `### Float widths`, `### Rounding modes`,
  `### Error sets`, `### Number formatting` and `### Linked libraries` under `## Rules`.
- The opening paragraph says that `just port-status` reads those headings and that renaming one
  breaks the check.
- The crate map's rows agree with `port/cut.tsv` and the `target crate` column of
  `port/manifest.tsv` as both stand on `main`, it names the command that re-derives them, and it
  says an agent may not invent, add or move a crate.
- `docs/PORTING.md` lists no libslic3r source file individually, restates no cut edge and restates
  no per-file scope decision; each of those points at the artifact instead.
- The scaling rule states a separate behaviour for each of `scaled<coord_t>()`, the Eigen
  `scaled()`, the floating-output `scaled<Tout>()`, `scale_()`, and the `unscale`/`unscaled`
  family, each with a `vendor/OrcaSlicer/`-relative `path:line` citation, and bans both adding
  rounding and multiplying by the reciprocal.
- The scaling rule states, for each of the five entry points, the precision the division happens
  at: `Point.hpp:666` and `:677` cast the global to `Tin` so the division is at the input's
  precision, `Point.hpp:689` divides in the matrix's `Scalar`, and `scale_()`/`unscale_()` carry no
  cast so the call site decides; and it says the port never widens the input to `f64` first.
- The rounding rule names every rounding idiom that occurs in in-scope files and not only
  `std::round`: `floor(x + 0.5)` stated as round-half-up and distinguished from `f64::round` at an
  exact negative half and at `0.49999999999999994`, `lround`/`llround`, and
  `fast_round_up` at `libslic3r.h:427-433` with its `0.49999999999999994` special case; and it
  states no universal claim that every rounding site is `std::round`.
- The rounding rule's scoping is the spec's: ties away from zero in the G-code formatter and in
  Clipper2 point construction, ties to even in Clipper2 intersections, and each remaining site
  ported individually.
- Every citation in the document resolves: for each `path:line`, the named line of the named file
  at the pin holds what the document says it holds.
- The type map tells porters to take the Rust type of every raw-pointer field from
  `port/ownership.tsv` verbatim, and states the two unclassified cases separately: an `unknown` row's
  conservative Rust type is taken from the row, while a **blank-class** row has no Rust type yet, so
  the field carries a `// TODO(port):` marker naming that row's header path and line and is filled
  in by the step-12 classification task for that row's batch — and no module invents a type for it.
- The type map fixes one insertion-ordered map spelling for the whole port and names the crate that
  provides it and the task that adds it, given that no port task may add a dependency.
- The banned-substitution list names, at a minimum, rounding the scaling conversions, a
  hash-ordered map where iteration order reaches output, a parallel reduction where the original's
  order is load-bearing, substituting any of R17's six libraries, changing a float width, a fused
  multiply-add, reciprocal multiplication, locale-dependent number formatting, a widened or
  narrowed error set, and a comparator that is not a total order — and names the check that catches
  each one where a check exists.
- The don't-translate list names the commented-out `Point.hpp:676` and `libslic3r.h:105` lines, the
  out-of-scope boundary rule with `// PORT NOTE:`, the FFI libraries, the debug-only blocks, and
  the deductions the manifest records.
- The rounding rule says which rounding sites are inside libslic3r and ties away from zero, and
  which are behind the FFI bridge and ties to even, with citations for both.
- The pull request changes `docs/PORTING.md` and no other file.
