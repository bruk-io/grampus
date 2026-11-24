---
kind: task
step: 13
delivers: [R18]
depends: [14-manifest-columns]
size: 5
---
# Task: Audit the unordered containers and the non-total-order comparators

## What to build

This is part 1 of step 13 of `intent/libslic3r-rust-port/plan.md`, "The order-dependence audits",
which is why its `delivers` is the whole step's. Part 2 is `18-divergences-regeneration`, which
extends the same script with `port/reports/divergences.md`, adds `port/regenerate` and
`just audit-check`, and is the only task that may write those. **This task writes the two
static-analysis reports and nothing else.**

**What it waits for.** `14-manifest-columns`, for a committed `port/manifest.tsv`. The audit's file
set is the rows that manifest marks in scope, read from the file at run time and never listed in a
document.

**Add `port/audit-order`**, a standalone executable with PEP 723 inline metadata run through `uv`,
like every other `port/*` script without an extension, and the `justfile` recipe `just audit-order`,
which regenerates two reports over the in-scope files:

- `port/reports/order-containers.md` — every unordered container that is **iterated** rather than
  only looked up.
- `port/reports/order-comparators.md` — every comparator that is **not a total order**.

**Every cell of both reports is derived, and none is hand-edited.** That is not a style preference:
`just audit-check`, which `18-divergences-regeneration` adds, regenerates both into a scratch
directory and fails when a committed one differs, so a hand-written cell would make that check
impossible to pass. Both reports are therefore byte-stable: rows sorted by path then line, paths
relative to the repository root, no timestamp and no machine-specific path anywhere in them.

**The container surface and the rule that classifies it.** The types the script recognises are
`std::unordered_map`, `std::unordered_set`, `std::unordered_multimap`, `std::unordered_multiset`,
`boost::unordered_*`, `ankerl::unordered_dense::map`, `ankerl::unordered_dense::set`,
`tbb::concurrent_unordered_map` and `tbb::concurrent_unordered_set`. That is the recogniser's set,
not the set present at the pin: the measurement below finds no `boost::unordered_*` and no
`std::unordered_multiset` anywhere in the tree, and those two names are kept only so a later
`#include` cannot introduce one unseen. For orientation, this is the surface a builder starts from,
measured at the pin and asserted against nothing:

```
cd vendor/OrcaSlicer/src/libslic3r && grep -rhoE \
  "std::unordered_(map|set|multimap|multiset)|boost::unordered_[a-z_]+|ankerl::[a-z_:]+|tbb::concurrent_unordered_[a-z]+" \
  . | sort | uniq -c | sort -rn
```

which prints 212 `std::unordered_map`, 101 `std::unordered_set`, 8 `std::unordered_multimap`, 4
`tbb::concurrent_unordered_map`, 4 `ankerl::unordered_dense::map`, 3
`ankerl::unordered_dense::set` and 1 `tbb::concurrent_unordered_set`. The script derives its own set
from the in-scope rows; it does not assert against those figures, because a number written in a
document is the thing that drifts.

The iterated-versus-looked-up rule is **written into the script**, not left to a reviewer, for the
whole-file-diff reason above. A container counts as iterated when the script finds, in the scope
that declares it or in a function it is passed to by reference, a range-`for` over it, a call to
`begin()`, `end()`, `cbegin()` or `cend()` on it, or the container handed to an algorithm or a
reduction. It counts as only looked up when every use is `find`, `count`, `contains`, `at`,
`operator[]`, `insert`, `emplace`, `erase` by key, `size`, `empty` or `clear`. **Where the script
cannot decide, the row is reported as iterated and its reason cell says the classification is
undecided.** That is the conservative direction and the reason it is written down: a false positive
is a row somebody reads and dismisses, while a false negative is a nondeterminism that reaches
G-code with no record of it.

Each row of `order-containers.md` carries **seven cells**: the `path:line` of the declaration, the
container type, the declaring scope (the function or the type), the `path:line` of the iteration that
classified it, a **reason** cell, the port's choice, and whether that choice diverges from the
oracle. The reason cell is where the undecided note goes, and it is a cell of its own rather than
text folded into the `path:line` cell, because the file is diffed whole and a reader has to be able
to select the undecided rows.

**The comparator surface and the rule that classifies it.** Comparators reach the audit from three
places: `operator<`, `operator<=` and `operator>` definitions in the in-scope files; the `Compare`
template argument of `std::map`, `std::set`, `std::multimap`, `std::multiset` and
`std::priority_queue`; and the comparator passed at an ordering-algorithm call site. The surface at
the pin:

```
cd vendor/OrcaSlicer/src/libslic3r && grep -rnE \
  "std::(sort|stable_sort|partial_sort|nth_element|lower_bound|upper_bound|binary_search|priority_queue|max_element|min_element)" . | wc -l
cd vendor/OrcaSlicer/src/libslic3r && grep -rnE "bool operator<\(" . | wc -l
```

437 ordering-algorithm call sites and 41 `operator<` definitions, again for orientation and
asserted against nothing.

A comparator is reported as not a total order when it matches one of three shapes the script looks
for: it compares two or more fields as a disjunction of `<` over different fields rather than as a
nested lexicographic chain, so two distinct elements can each compare less than the other; its
top-level return uses `<=` or `>=`, so equal elements compare true and the strict weak ordering a
standard algorithm requires is broken; or it compares floating-point values through a tolerance or
an `EPSILON`, which makes the relation non-transitive. **A comparator the script cannot decide is
reported as not a total order**, with its reason cell saying so, for the same conservative reason as
the containers.

Each row of `order-comparators.md` carries **seven cells**: the `path:line` of the comparator, how it
reaches an ordering (definition, container `Compare`, or call site, with the `path:line` of the use),
which of the three shapes matched, a **reason** cell holding the undecided note where the
classification is undecided, the port's choice, and whether that choice diverges from the oracle.

**The port's choice and the diverges cell are derived, by these two rules**, which is what makes
"every cell of both reports is derived, and none is hand-edited" true of them as well. They are
written here because they are the two cells no pattern reads off the source, and two builders would
otherwise choose differently.

*The port's choice* is a fixed mapping from what the row matched, with no judgement in it:

- an iterated `std::unordered_map`, `std::unordered_multimap`, `ankerl::unordered_dense::map` or
  `tbb::concurrent_unordered_map` → an insertion-ordered map;
- an iterated `std::unordered_set`, `std::unordered_multiset`, `ankerl::unordered_dense::set` or
  `tbb::concurrent_unordered_set` → an insertion-ordered set;
- a container classified as only looked up → the same container kind with no ordering guarantee,
  because its order never reaches output;
- a comparator matching the disjunction shape → a nested lexicographic chain over the same fields
  in the same order;
- a comparator matching the `<=`/`>=` shape → the strict form of the same relation;
- a comparator matching the tolerance shape → the same comparison with the tolerance removed and
  the row's tier deciding the equality, which is the one case that may need a manifest tolerance;
- an undecided comparator → a total-order comparator.

*The diverges cell* is `yes` whenever the port's choice above is not the same relation or the same
iteration order the oracle has — which, for every mapping above except "only looked up" and the
strict form of an already-strict relation, it is not. **Where the script cannot decide whether the
site's order reaches the compared output, the cell is `yes` and the reason cell says the
reaches-output question is undecided.** That is the same conservative direction as the two
classification rules, and for the same reason: a row a reader dismisses costs a minute, while a
`no` on a site that does reach output is a silent divergence with nothing in the repository
recording it.

The port's choices these rules encode, which `19-porting-contract` then carries into
`docs/PORTING.md`: sum in a fixed order, use total-order comparators, and use insertion-ordered maps
wherever iteration order reaches output. The port is therefore deterministic where the oracle is
not. Rows whose diverges cell is `yes` reach `port/reports/divergences.md` through part 2's
regeneration — this task writes no row into that file.

**What makes this a 5.** The surface is large and the risk is asymmetric. Roughly 420 in-scope files
hold over 300 unordered-container declarations and 437 ordering-algorithm call sites, and for each
one the audit has to answer a question no single pattern answers: whether the container is iterated
somewhere that reaches output, and whether a comparator is a strict weak ordering. Judgement that a
script has to hold rather than a reviewer is what the work is, and a missed site is a
nondeterminism that silently diverges from the oracle later with nothing in the repository recording
it — which is exactly what R18 exists to prevent. It is not split into one task per report because
both read the same in-scope rows, walk the same files, and share the one conservative
undecided-means-flagged rule; two scripts each half-parsing the same tree would be more work and two
places for that rule to drift.

**What this task does not do.** It does not write `port/reports/divergences.md`,
`port/reports/perf-backlog.md`, `port/regenerate` or `just audit-check` — those are
`18-divergences-regeneration` and `23-perf-backlog`. It adds no line to `.asdlc/checks` and none to
`just port-check`: `just audit-check` is explicitly not a check a port task passes, and
`24-port-check-ci` owns that recipe. Neither report carries a provenance header — step 3 of the plan
excludes both by name from what `oracle/provenance --check` covers, and that check fails if it is
pointed at one, so do not add a header and do not add either path to the provenance check's list. It
does not edit `port/manifest.tsv` and changes nothing under `vendor/OrcaSlicer/`.

## Done when

- `port/audit-order` exists as a `uv`-run standalone script with PEP 723 inline metadata, and
  `just audit-order` is a recipe in the `justfile`.
- `just audit-order` reads the in-scope rows of `port/manifest.tsv` at run time and derives its file
  set from them, with no file list written into the script or into a document.
- `just audit-order` prints the row count of `port/reports/order-containers.md` and the row count of
  `port/reports/order-comparators.md` on every run, which is half of the plan's recorded proof for
  R18 — part 2, `18-divergences-regeneration`, prints the third report's count and the counts of its
  four sources.
- `port/reports/order-containers.md` is committed and holds one row per unordered container the
  script finds iterated across those files, each row carrying all seven cells: the declaration's
  `path:line`, the container type, the declaring scope, the iteration's `path:line`, the reason cell,
  the port's choice, and whether it diverges from the oracle.
- `port/reports/order-comparators.md` is committed and holds one row per comparator the script finds
  not to be a total order, each row carrying all seven cells: the comparator's `path:line`, how it
  reaches an ordering with the use's `path:line`, which shape matched, the reason cell, the port's
  choice, and whether it diverges from the oracle.
- The iterated-versus-looked-up rule, the three not-a-total-order shapes, the port's-choice mapping
  and the diverges rule are all implemented in `port/audit-order`, and in all four cases a site the
  script cannot decide is reported conservatively rather than dropped — a reviewer can point at the
  branch in the script that does it, including the branch that writes `diverges: yes` with the
  reaches-output question undecided.
- Running `just audit-order` twice leaves both committed reports byte-identical, and a fresh clone
  at this commit regenerates both of them byte for byte, so no cell of either is hand-written.
- Neither report carries a provenance header, and neither path is added to what
  `oracle/provenance --check` covers.
- The branch writes no `port/reports/divergences.md`, no `port/reports/perf-backlog.md`, no
  `port/regenerate` and no `just audit-check` recipe.
- The branch adds no line to `.asdlc/checks` and none to `just port-check`.
- `port/manifest.tsv` is unchanged, nothing under `vendor/OrcaSlicer/` changed, and the submodule
  pointer is still `f92a7c065c5718b8ef1baac1cd704df2911238f4`.
- The branch changes no file under `intent/`, so `just guard` passes, and `just check` passes.
