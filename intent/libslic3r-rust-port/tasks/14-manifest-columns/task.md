---
kind: task
step: 11
delivers: [R8, R9]
depends: [06-manifest-scope, 07-crate-cut]
size: 5
---
# Task: Seed and carry forward the manifest's judgement columns and check for drift

## What to build

This is **part 4 of step 11** of `intent/libslic3r-rust-port/plan.md` ("The scope manifest and the
crate cut"): the column semantics and `just inventory-check`. Parts 1 to 3 are
`03-manifest-file-set` (the file set and the per-file derived columns), `06-manifest-scope` (the
four-pass scope algorithm and the deductions) and `07-crate-cut` (`port/cut.tsv` and the
crate-level level). Part 5 is `15-task-cuts`. This task extends the same `port/inventory` script
those tasks build and adds one recipe to the `justfile`; it computes no scope and no cut of its
own. **It also runs `just inventory` and commits the regenerated `port/manifest.tsv`**, with the
six seeded columns filled, because that committed file is the seed every later run carries forward
and the "Done when" lines below are written against it: parts 1 and 3 commit those cells empty, so
without the regeneration this task would ship the script and leave the file unseeded.

This part is the whole answer to how a generated file can also be the port's progress record.
`port/manifest.tsv`'s columns fall into exactly three categories, and `port/inventory` has to
treat each differently:

**Derived from the vendored tree on every run**, recomputed from the source and overwriting
whatever is committed: path, subsystem, lines, file-level include-DAG level, crate-level level, in
scope or out with its reason, target crate and module, and pipeline stage. Those are parts 1 to 3's
work — `03-manifest-file-set` for path, subsystem, lines and the file-level level,
`06-manifest-scope` for the scope cell **and the `pipeline stage or none` cell**, `07-crate-cut` for
the crate-level level and the target crate and module — and this task changes none of them, and only
has to know which they are, because the drift check below compares these and nothing else.

**Seeded by the script, then carried forward verbatim** from the committed `port/manifest.tsv` on
every later run — six columns: `template-heavy`, `equality tier`, `tolerance`,
`comparison function`, `proof path`, and `fuzz target`. These are judgements a script can only
guess at: they are set by `35-trial-run` and by review in the pull request that first fills a row.
So `port/inventory` writes a stated default into a row it has never seen before, and then keeps
each of those six cells **exactly as it finds it** on every run after that. The defaults are:

- `template-heavy` by the rule below.
- `equality tier`: `exact`.
- `tolerance` and `comparison function`: empty. Every tolerance in this repository defaults to
  exact, and the comparator refuses a tolerance with no reason text in the row.
- `proof path`: the path the row's own columns imply, by a rule written into the script's source
  next to the code that applies it. Write it from columns the manifest already has, mapping onto
  R19's three paths, tested in this order: a row whose `.cpp` declares an anonymous namespace or
  static free functions seeds proof path 3, the test translation unit that `#include`s the
  vendored `.cpp`, since that code has no external linkage to call; otherwise a row whose
  `pipeline stage` column names a stage seeds proof path 2, the committed stage dumps; otherwise
  proof path 1, FFI to the library built from the pin. That second test is reachable because
  `06-manifest-scope` populates `pipeline stage or none` for every in-scope row. An out-of-scope
  row's cell is empty.
- `fuzz target` by the rule below.

**What every row gets, including the negative cases**, so the committed file has no cell whose value
is undefined: `template-heavy` is `yes` where the rule below fires and `no` everywhere else,
in-scope or not; `equality tier` is `exact` for an in-scope row and empty for an out-of-scope one;
`tolerance` and `comparison function` are empty everywhere until something sets them;
`proof path` is one of the three for an in-scope row and empty for an out-of-scope one; and
`fuzz target` is `required` or `n/a` for an in-scope row and `n/a` for an out-of-scope one.

**Derived from the Rust tree, and never stored as a fact anyone edits**: `status`. A row's status
is `drafted` when a file exists at its target path carrying a `PORT STATUS` trailer that names it
and no `mod` declaration reaches that file; `ported` when a `mod` line reaches it and it holds no
`todo!()` or `unimplemented!()` and its proof passes; and `not-started` otherwise.
`21-port-status-markers` and `22-port-status-gates` compute it the same way from the same tree, and
that computation is the authority. `port/inventory` writes the cell for a reader's benefit and
never reads one as a fact.

No proof harness exists at this step, so `port/inventory` evaluates only the text-visible half of
that rule — the target file present with a trailer naming the row, whether a `mod` line reaches it,
and whether it holds `todo!()` or `unimplemented!()` — reading files as lines and never parsing
Rust or running a test. The proof half belongs to `22-port-status-gates`, which is the authority on
a row's status; the manifest's cell is informational, and nothing in this script shells out to
`cargo`.

**The two seed rules, transcribed as written.** `template-heavy` is `yes` for a row that is
type-and-trait design rather than a file-unit, which is the set `34-foundation-spike` hand-designs:
an in-scope header with no same-stem `.cpp` sibling, **or** an in-scope header where `template<`,
`constexpr` and `inline` definitions make up more than a quarter of its non-comment lines. The
column exists because that set cannot be read off the file layout: at the pin `Point.hpp`,
`Config.hpp`, `Model.hpp` and `libslic3r.h` — the four heaviest foundation headers — all **do** have
a same-stem `.cpp`, so the sibling half of the rule misses every one of them, which is why the
second half (template, `constexpr` and `inline` density) exists and why the column is carried
forward. The sibling half is not narrow in general: 77 of the tree's headers have no same-stem
`.cpp` at the pin, and the plan's step 20 counts 76 header-only files holding 14,840 of the
library's 340,112 lines, so it is the four named headers the rule misses and not the set at large.
`fuzz target` is `required` for an
in-scope row whose equality tier is `exact` and whose ported surface takes `Point`, `Points`,
`Polygon`, `Polygons`, `ExPolygon` or `ExPolygons`, read from the signatures in its header, and
`n/a` otherwise.

Both rules are **transcribed as the plan states them and not tuned**. The seed rule for
`template-heavy` will read some rows wrongly, `Model.hpp` the likeliest, and that is why the column
is carried forward rather than recomputed: a wrong seed is corrected in the review of this task's
own pull request and in `34-foundation-spike`, never by special-casing a path inside the script.
The same holds for a `fuzz target` cell the signature reading gets wrong.

**`just inventory-check`.** Add the recipe. It regenerates the manifest into a scratch directory
and fails when the committed file differs — but it **diffs the derived columns only**, and ignores
the six carried-forward columns and the status column entirely. `24-port-check-ci` puts it inside
`just port-check`, which every port task's pull request has to pass, so the contract matters: exit
0 and print nothing on a clean tree; exit non-zero with a diff naming the column when a derived
column has been edited by hand; exit 0 when `tolerance`, `equality tier` or another carried-forward
column has changed, which is how a port task's pull request passes it; and never compare `status`
at all. Plain `just inventory` rewrites the committed file and is for a person regenerating
deliberately.

**A port task never edits `port/manifest.tsv`.** A row's status advances because a task added a
Rust file, not because anybody wrote a cell, which is what lets a hundred task branches run at once
without colliding on one line of one file. Say so in the script's own comment, so the next reader
of `port/inventory` finds it there.

The manifest carries **no coverage column**: nothing gates on coverage-measured execution, and a
column no step owns is a column that drifts.

**What makes this a 5.** It is the design the hundred-branch fan-out rests on, and getting the
three categories wrong breaks it: a judgement column recomputed on every run throws away the
trial's work, a derived column left out of the drift check lets a hand edit stand, and a status
read as a cell turns every port task into an edit of one shared line. Six columns need a stated
default each, two of those defaults are rules over the vendored source rather than constants, the
status derivation has to agree exactly with a script two other tasks build, and the check has to
pass and fail on the right columns in both directions. It is not split further because the three
categories are one decision: the seeding, the carrying forward and the check are the same rule seen
from three sides, and splitting them would leave a check that does not know what it may compare.

## Done when

- `port/inventory` writes all six carried-forward columns with the stated defaults into a row it
  has not seen before, and the rule it uses for `proof path`, `template-heavy` and `fuzz target` is
  written in the script's source beside the code that applies it.
- Setting `equality tier` and `tolerance` by hand on a row and then running `just inventory` leaves
  those two cells byte-identical, and the same holds for `template-heavy`, `comparison function`,
  `proof path` and `fuzz target`.
- `port/inventory` derives every row's `status` by the three rules above from the Rust tree, and
  reads no status cell as an input.
- `just inventory-check` exits 0 and prints nothing on a clean tree.
- `just inventory-check` exits non-zero and prints a diff when a derived column — `subsystem`,
  `lines`, either level, the scope cell and its reason, target crate and module, or
  `pipeline stage` — is edited by hand.
- `just inventory-check` exits 0 when a carried-forward column is changed by hand, and never
  compares the `status` column.
- `port/manifest.tsv` as committed on this branch is the output of `just inventory`, and every
  **in-scope** row has a non-empty `template-heavy`, `equality tier`, `proof path` and
  `fuzz target` cell, with `exact` tier and empty `tolerance` and `comparison function` wherever
  nothing has set them; every **out-of-scope** row has `template-heavy` `no` or `yes`,
  `fuzz target` `n/a`, and an empty `equality tier`, `tolerance`, `comparison function` and
  `proof path`.
- At least one in-scope row's `proof path` cell is path 2, which is only reachable because
  `06-manifest-scope` fills the `pipeline stage` column.
- `port/manifest.tsv` has no coverage column.
- The script carries a comment stating that a port task never edits `port/manifest.tsv` and that
  status is derived from the tree.
