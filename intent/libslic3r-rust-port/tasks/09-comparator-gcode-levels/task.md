---
kind: task
step: 5
delivers: [R4]
depends: [08-comparator-workspace]
size: 5
---
# Task: Compare G-code at the token, per-move and aggregate levels

## What to build

This is part 2 of plan step 5, "The comparator", which delivers R4. It adds
`crates/oracle-compare/src/gcode.rs` — the G-code parser, the move model and the localisation — and
the L2, L3 and L4 evaluations in `crates/oracle-compare/src/levels.rs`, together with the tolerance
input every level above L1 needs. It waits on `08-comparator-workspace` for the workspace, the
crate, the report shape and L0 and L1.

**Why this is a 5.** Several moving parts sit over one parsed model: a token stream, per-axis
tolerances that report max and RMS deviation, per-layer and per-role aggregates of six quantities,
and the localisation that plan step 24 reuses for its G-code fallback. It is not split further
because L3 and L4 both read the move model the parser builds, so a split would hand two tasks the
same file and the same model and make neither reviewable on its own.

Like task 08, this task runs no oracle: it depends on neither `01-oracle-fetch` nor
`04-oracle-slice`, so the oracle binary need not exist in its worktree, and every fragment it
commits under `crates/oracle-compare/tests/data/` is hand-written. Keep `tests/data/README.md`
saying so. It creates neither `port/manifest.tsv` nor `docs/ORACLE.md`, so the readers below are
tested against committed fixture files.

**The parser and the move model (`src/gcode.rs`).** Byte-oriented, as the normalisation is: split
on `\n`, no UTF-8 validation. Split each line into its command word (`G0`, `G1`, `G2`, `G3`,
`G92`, `M83`, `T0`, …) and its parameter words, each a letter plus the literal text of its value;
everything from the first `;` is the comment. Keep both the literal text and the parsed `f64` of
each value, because L2 compares text and L3 compares numbers.

The move model is one entry per `G0`, `G1`, `G2` or `G3` line, carrying the command, each axis word
present (X, Y, Z, E, F, I, J, R) with its literal and its value, whether the word was present at
all — an absent word is not zero — the active tool from the last `T<n>`, the current E mode from
`M82` and `M83` with its origin reset by `G92 E<v>`, and the localisation below.

**The localisation, which plan step 24 reuses.** The current layer index and Z, the current
extrusion role, and the current object label, each tracked from the comments the emitter writes:

- On a Bambu printer — which every configuration in `docs/ORACLE.md` is — the layer-change and role
  tags are emitted as `";" + GCodeProcessor::reserved_tag(…)`, so the lines are `; CHANGE_LAYER` and
  `; FEATURE: <role>`. Those two tag strings come from the `Reserved_Tags` array at
  `vendor/OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:103-134`, and their non-Bambu spellings
  `;LAYER_CHANGE` and `;TYPE:<role>` from `Reserved_Tags_compatible` (`:136-167`). The layer tag is
  emitted at `src/libslic3r/GCode.cpp:4610`, `:6231` and `:6698`, and the role tag at `:9881` for a
  sliced extrusion and at `:4405` and `:4857` for custom G-code, with the role names coming from
  `ExtrusionEntity::role_to_string` rather than from the tag.
- **The Z tag is in neither array.** It is a hardcoded pair at `src/libslic3r/GCode.cpp:6701`,
  `sprintf(buf, print.is_BBL_printer() ? "; Z_HEIGHT: %g\n" : ";Z:%g\n", print_z)`, selected by
  `is_BBL_printer()` and with no `ETags` entry at all. Do not look for it in `Reserved_Tags`: what
  that array holds nearby is `" LAYER_HEIGHT: "` (and `"HEIGHT:"` in the compatible array), which is
  a different tag and would silently localise by layer height rather than by Z.
- Accept both spellings of all three tags and record which set the file used, so a comparison never
  silently loses its layers because the printer flag differed.
- Object labels are `; printing object <name> id:<n> copy <n>`,
  `; start printing object, unique label id: <n>` and the matching stop lines
  (`src/libslic3r/GCode.cpp:7657`, `:7665`, `:7812`, `:7823`). Note that the `unique label id`
  lines are dropped by the declared normalisation (R4) but not by this parser: the parser reads the
  raw file.

Expose the localisation on the public API, with a way to ask for the level reached within one
layer, one role or one object. Task `39-compare-stages` needs exactly that: for the stages with no
hook seam — G-code emission and conflict check — `just compare-stages` falls back to these levels
on the exported G-code, localised by these comments, and says it is in fallback.

**L2: identical token stream, command plus axis words, comments dropped.** Compare the sequence of
(command word, parameter words as letter plus literal value) over every command line, not only
moves. A difference reports the token index, the line number on each side, and both lines.

**L3: identical move count and order, numbers within the declared per-axis tolerance.** Compare
the move models pairwise. A difference in move count, or in the command sequence of the moves,
fails L3 and is reported as the first differing move index with its layer, role and object from
the localisation. An axis word present on one side and absent on the other fails regardless of
tolerance. Report max and RMS deviation per axis over every compared pair of moves, and print them
whether or not the level passed, because that number is what a reader uses to judge how far apart
two runs are.

**L4: per-layer and per-role aggregates within the declared aggregate tolerance.** The six
quantities R4 names — move count, extruded length, E delta, bounding box, travel distance, layer
count — grouped per layer, per role, per (layer, role), and for the whole file. Write each
definition into the module's doc comment in terms a reviewer can recompute by hand on a committed
fragment, and keep these:

- layer count is the number of layer boundaries the localisation found;
- E delta is the sum of positive E advances in the group, which means the difference from the
  previous E in absolute mode and the E word itself in relative mode, with `G92 E<v>` resetting the
  origin;
- extruded length is the Euclidean XY length of each move whose E advances, and travel distance the
  XY length of each move whose E does not;
- bounding box is the min and max X and Y over the group's moves;
- every sum runs in ascending move index, in a fixed order, so the result is reproducible — R18's
  rule applies to the comparator's own arithmetic too.

**The tolerance input.** Every tolerance in this repository defaults to exact: zero per-axis
deviation and zero aggregate deviation. A looser one exists only as a `tolerance` and
`comparison function` value on a `port/manifest.tsv` row or a `docs/ORACLE.md` configuration row,
and only with a reason in that row (R4). So add `--tolerance-from-manifest <tsv> --row <path>`,
naming the libslic3r path whose row to read, and `--tolerance-from-config <md> --configuration
<name>`; with neither flag the comparator is exact. Find the columns by header name. Accept a
tolerance cell of the shape `<value>[ <unit>]; reason: <text>` and refuse — exit non-zero with a
named error, having compared nothing — a cell that carries a value with no reason, or a
`comparison function` the comparator does not implement. Print the tolerance used and where it came
from on every run, in both output formats. A missing file, a `--row` or `--configuration` that names
no row, or a table with no `tolerance` or `comparison function` column each exit non-zero with a
named error, having compared nothing.

Task `14-manifest-columns` owns the manifest's column
semantics; if it lands a different cell shape the reader is corrected there, and the fixture rows
this task commits under `tests/data/` are what it is held to meanwhile. The same hedge applies to
the other side: the configuration table `04-oracle-slice` commits in `docs/ORACLE.md` carries
preset paths, setting overrides and fixtures and **no** `tolerance` or `comparison function`
column, because no row needs one yet. `04-oracle-slice` owns that table, so a row that later needs
a tolerance gains those two columns there; until then `--tolerance-from-config` is exercised against
a fixture table this task commits under `tests/data/`, and against the real document it exits
non-zero with the named missing-column error rather than comparing at a silent default.

**The report.** Fill the L2, L3 and L4 outcomes task 08 left as not-evaluated, so the strongest
level is now over all five, `none` is plain `none` with the first differing byte, and task 08's
"strongest evaluated" and "none of L0, L1" qualifiers are removed. Each level is still evaluated
independently and the levels are not assumed nested: L4 passing where L3 fails, and L2 passing
where L1 fails, are both real outcomes the report has to be able to state. Use no `todo!()` or
`unimplemented!()`, which `just port-status` counts across `crates/`.

**The tests.** Extend `crates/oracle-compare/tests/levels.rs` with hand-written fragment pairs
covering each new level and each failure report: a comment-only difference, where L2 passes and L0
fails; `X10.0` against `X10`, where L2 fails and L3 passes at exact tolerance; one coordinate moved
by 0.001, where L3 fails at exact tolerance and passes against a committed fixture manifest row
whose tolerance cell allows more with a reason; a fixture row carrying a tolerance and no reason,
where the command exits non-zero and names the row; an inserted move, where L3 reports the move
count difference with the first differing move's index, layer, role and object; a pair whose moves
differ but whose per-layer and per-role aggregates are inside the aggregate tolerance, where L4
passes and L3 fails; a pair with a different layer count, where L4 fails naming the layer; and a
fragment in the non-Bambu tag spellings, to prove both tag sets parse.

**What this task does not touch.** `src/json.rs` is task `10-comparator-json` and `src/stages.rs`
is task `39-compare-stages`; leave both uncreated. Leave the declared normalisation and
`tests/normalisation.rs` exactly as task 08 committed them — the four line kinds are upstream's
list and this task adds nothing to it. Add no CI line and no `.asdlc/checks` line (plan step 16,
task `24-port-check-ci`). Nothing under `vendor/OrcaSlicer/` changes and the submodule pointer
stays at `f92a7c065c5718b8ef1baac1cd704df2911238f4`.

## Done when

- `cargo test -p oracle-compare` passes, including the extended `tests/levels.rs`, with
  `tests/normalisation.rs` unchanged from task 08.
- `cargo fmt --check` exits 0 and `grep -rn 'todo!\|unimplemented!' crates/` finds nothing.
- `just compare` on the committed pairs prints every level that passes, the strongest of them and
  the first failing location at the next stronger level, and prints `none` with the first differing
  byte on the unrelated pair; no line of any output says `not evaluated`, `strongest evaluated` or
  `none of L0, L1`.
- An L3 line prints max and RMS deviation for every axis compared, and an L4 line prints the six
  aggregates per layer and per role together with the layer count.
- Every run prints the tolerance it used and its source; with no tolerance flag it prints `exact`;
  against the fixture row whose tolerance has no reason it exits non-zero and names the row.
- The one-coordinate-moved pair reports L3 failing at exact tolerance and passing when pointed at
  the fixture row that allows the deviation with a reason.
- The aggregates-equal pair reports L4 passing while L3 fails, which shows the levels are evaluated
  independently rather than nested.
- A test asserts the layer index, role and object of a named move in a committed fragment through
  the crate's public API, and a second fragment in the `;TYPE:` / `;LAYER_CHANGE` / `;Z:` spellings
  parses to the same model shape.
- The only files this task changed are under `crates/oracle-compare/`;
  `crates/oracle-compare/src/json.rs` and `src/stages.rs` do not exist;
  `git -C vendor/OrcaSlicer status --porcelain` prints nothing and the submodule pointer is
  unmoved.
