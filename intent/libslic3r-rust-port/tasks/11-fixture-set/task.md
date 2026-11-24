---
kind: task
step: 4
delivers: [R24, R25]
depends: [01-oracle-fetch, 04-oracle-slice]
size: 5
---
# Task: Freeze the three fixtures and fix their placement

## What to build

The whole of plan step 4, "Freeze the fixture set and fix placement": two scripts, three
committed fixtures, the `vendor/README.md` records that make a fixture a recorded act, and the
`justfile` recipes that drive them. Scripts outside the Rust workspace are standalone
executables with PEP 723 inline metadata and no file extension, run through `uv`, as this
repository's conventions require.

Add `oracle/export-fixture` and `just fixture-export`, which turns a vendored model into a
committed STL that is already positioned on the target plate. It does that in **two moves,
because the pinned CLI has no XY placement at all.**

**Move one, the oracle's export.** With `$BIN` the binary `01-oracle-fetch` leaves at
`oracle/.cache/app/OrcaSlicer.app/Contents/MacOS/OrcaSlicer`:

```
"$BIN" --datadir "$SCRATCH/datadir" --arrange 0 --export-stls "$SCRATCH/stl" <vendored model>
```

`--export-stls` takes a directory (`vendor/OrcaSlicer/src/libslic3r/PrintConfig.cpp:12764`,
`coString`; handler at `vendor/OrcaSlicer/src/OrcaSlicer.cpp:6420-6425`) and writes one file per
object through `CLI::output_filepath`, which names it `obj_<index>_<object name>` with `.stl`
appended and creates the directory (`src/OrcaSlicer.cpp:8461-8516`). **No option sets the output
name**, so the script renames the file written there to the frozen basename — and because
`output_filepath` writes one file per object while every fixture holds exactly one, the script
asserts that exactly one `.stl` landed in `$SCRATCH/stl` and exits non-zero naming the count
otherwise, rather than renaming an arbitrary one. The exported
mesh carries whatever instance transform the load applied, because `store_stl(path, object,
true)` serialises `ModelObject::mesh()` (`src/libslic3r/Format/STL.cpp:53-57`), and for a mesh
input that is the default instance plus the Z-only `ModelObject::ensure_on_bed` the CLI applies
at `src/OrcaSlicer.cpp:2003` — so Z lands on the bed and X and Y are the model's own. The `true`
makes it a **binary** STL: 80 header bytes, which are zeroes because the label is empty, a
`uint32` triangle count, then 50 bytes per facet (`its_write_stl_binary`,
`src/libslic3r/TriangleMesh.cpp:1793-1811`).

**Move two, the script's own XY translation**, applied to the exported STL. It is the script's
work and not the oracle's because the CLI's `align_xy` and `center` transform options **do not
exist at the pin**: each `def = this->add(...)` call is inside a commented-out block in
`CLITransformConfigDef` — `align_xy` in the block at
`vendor/OrcaSlicer/src/libslic3r/PrintConfig.cpp:12903-12911` and `center` in the separate block
at `:12930-12933`. The only registered transforms are `arrange`, `repetitions`, `ensure_on_bed`,
`assemble`, `convert_unit`, `orient`, `repair`, `rotate`, `rotate_x`, `rotate_y`,
`ground_largest_face`, `ground_face_normal`, `ground_face_point` and `scale`. Their *handlers*
are still live code in `src/OrcaSlicer.cpp` — `align_xy` at `:5065-5071` — which is why this is
written down: a reader who finds the handler will think the flag works.

The translation puts the mesh's XY bounding-box centre at the centre of the machine's printable
area, and it moves **X and Y only: Z is never touched**, because move one already applied
`ensure_on_bed` and a Z nudge would make the recorded identity transform false. It rewrites the
file as binary STL, in the same layout move one produced. The script reads both bed polygons from
the machine preset rather than from a figure in a document, and the two come from **different
files**, which is the trap:

- `printable_area` is not in the X1C machine preset at all; it is inherited from
  `vendor/OrcaSlicer/resources/profiles/BBL/machine/fdm_bbl_3dp_001_common.json:123`, where it is
  `0x0 256x0 256x256 0x256` — so the centre is `(128, 128)`.
- `bed_exclude_area` **is** overridden in
  `vendor/OrcaSlicer/resources/profiles/BBL/machine/Bambu Lab X1 Carbon 0.4 nozzle.json:14`,
  where it is `0x0 18x0 18x28 0x28`, replacing the eight-point polygon in the common file at
  `:12`. A script that reads only the common file gets the wrong exclusion area; one that reads
  only the X1C file gets no printable area.

The script computes its offset from the mesh's own bounding box, and **asserts afterwards** that
the translated bounding box lies inside the printable area and does not meet the exclusion area.
The benchy's extent is written down nowhere, because its vendored form is Draco-compressed, and
that assertion is what covers it. It matters that something does this:
`vendor/OrcaSlicer/tests/data/20mm_cube.obj` spans 0 to 20 in both X and Y at the pin, so its own
coordinates lie inside that exclusion area, and with `--arrange 0` nothing in the oracle would
move it out. After the translation each fixture's resolved instance transform is the identity and
`ensure_on_bed` is a no-op, and that identity is the one transform the `vendor/README.md` row
records.

The three fixtures, with **frozen basenames**, because the model's basename — not its path — is
embedded in both the G-code (`; printing object <basename> id:0 copy 0`) and the statistics JSON
(`objects[0].name`), and no absolute path leaks into either, so the fixtures are reproducible
across machines:

| Frozen basename | Produced from |
|---|---|
| `fixtures/3DBenchy.stl` | `vendor/OrcaSlicer/resources/handy_models/3DBenchy.drc` |
| `fixtures/20mm_cube.stl` | `vendor/OrcaSlicer/tests/data/20mm_cube.obj` |
| `fixtures/frog_legs.stl` | `vendor/OrcaSlicer/tests/data/frog_legs.obj` |

`vendor/README.md` gains one row per fixture naming its frozen basename, the vendored path it
came from, the single resolved instance transform, and the license the model's publisher states —
3DBenchy.com's CC0 1.0 for the benchy, with the explicit note that upstream records no license
for `resources/handy_models/`, a directory with no license file. Committing that row is the only
way a fixture is ever added.

Add `oracle/assert-placement` and the recipe `just assert-placement <fixture> <run-dir>`, named here
so the two callers do not each invent a spelling. It reads a fixture's `vendor/README.md` row and
the bounding box the oracle reports for the run in `<run-dir>` and exits non-zero when they
disagree. `41-compare-benchy` and `42-compare-fixtures` call it, so `just compare-benchy` and
`just compare-fixtures` fail on an unexpected placement. It also takes a `--list` mode that prints
one line per fixture record — frozen basename, vendored path, transform, license — which is how
`42-compare-fixtures` enumerates the committed set without writing a Markdown parser of its own
against a table it has never seen. Each fixture holds one object, so each
record names one transform; the multi-filament configuration assigns more than one filament to
that single object rather than adding a second one.

**This task is also where `oracle/slice` learns to resolve a bare fixture stem.**
`04-oracle-slice` takes either a frozen basename in `fixtures/` or a path, and `fixtures/` did not
exist then. Here `oracle/slice` gains the one resolution rule the later tasks all write against: an
argument with no path separator and no extension is resolved against `fixtures/` by appending
`.stl`, so `just oracle-slice 3DBenchy base-fff` means `fixtures/3DBenchy.stl`; an argument with a
separator is a path and is not resolved; an argument that resolves to no committed fixture exits
non-zero naming it. That is the form `25-stage-dumper`, `26-stage-dump-seams`,
`32-proof-dumps-fuzz` and `41-compare-benchy` use, and the one flag-free addition this task makes
to that script's behaviour; the command's interface gains no option.

Two boundaries a builder would otherwise guess at. The fixtures are **deliberately outside**
`oracle/provenance --check`'s covered set, which `05-oracle-provenance` names as everything under
`port/goldens/`, `port/dumps/*/*.dump`, and the two reports an oracle run writes — a binary STL
cannot carry a `;` comment block, and its record is its `vendor/README.md` row. This task therefore
calls `oracle/provenance` nowhere and does not depend on it; the consumption runs the other way, as
`05-oracle-provenance` reads the records this task writes for items 8 and 9 of its header, which is
why that task does not appear in this one's `depends`. And
`docs/ORACLE.md`'s configuration table belongs to `04-oracle-slice`: this task fills or corrects
only the column that names the fixtures each row uses, which is R25's last sentence, and
restructures nothing else in that document.

**Why this is a 5, and why it is not split.** Two moves against a CLI that cannot place a mesh:
an oracle export whose output name is unsettable and has to be renamed, then the script's own
binary-STL translation with assertions against two bed polygons read from two different preset
files. The combination this repository depends on — `--arrange 0` with a mesh the script placed —
is first exercised here, and if the pinned CLI will not slice a mesh without arranging it, this
is where that is found. It is not split from `oracle/assert-placement`, which is a 2 on its own
and reads the same `vendor/README.md` rows this task writes, so splitting would hand one builder
the rows and the other the reader of them.

Nothing under `vendor/OrcaSlicer/` changes and the submodule pointer stays at
`f92a7c065c5718b8ef1baac1cd704df2911238f4`. Do not add a line to `.github/workflows/ci.yml`:
`24-port-check-ci` creates the jobs, and each gate joins in the step that makes it pass.

## Done when

- `ls fixtures/` prints exactly `3DBenchy.stl`, `20mm_cube.stl` and `frog_legs.stl`, each a
  binary STL, and each basename matches the one `docs/ORACLE.md`'s configuration rows name.
- `just fixture-export` produces each of the three from its vendored path in the two moves above,
  and re-running it over an already committed fixture leaves that file byte-identical.
- The export step asserts that exactly one `.stl` was written into its scratch directory before
  renaming it, and exits non-zero naming the count when that is not so.
- The translation moves X and Y only: the Z extent of a regenerated fixture equals the Z extent
  move one exported, and `just oracle-slice <fixture> base-fff` needs no further placement.
- For each fixture the script asserts, and prints, that the translated XY bounding box lies
  inside `printable_area` and does not meet `bed_exclude_area`, reading the first from
  `fdm_bbl_3dp_001_common.json:123` and the second from
  `machine/Bambu Lab X1 Carbon 0.4 nozzle.json:14`.
- That assertion fails loudly rather than silently: run against the untranslated
  `20mm_cube` export, the script exits non-zero and names the exclusion area.
- No line in `oracle/` that invokes the oracle binary carries `--align-xy`, `--align_xy` or
  `--center`: a reviewer reads every `"$BIN"` invocation under `oracle/` and finds none of the
  three. The check is on invocations and not on the whole file, because this task deliberately
  records the `align_xy` trap in a comment in `oracle/export-fixture` — the handler at
  `src/OrcaSlicer.cpp:5065-5071` is live code for an option that does not exist at the pin, and a
  reader who finds the handler will think the flag works — so a recursive grep for the name is
  expected to hit that comment.
- `grep -n 'arrange' oracle/export-fixture` shows `--arrange 0` on the export invocation.
- `just oracle-slice 3DBenchy base-fff`, `just oracle-slice 20mm_cube base-fff` and
  `just oracle-slice frog_legs base-fff` each exit 0 with a non-empty `plate_1.gcode`, which is also
  the proof that the bare-stem resolution this task adds to `oracle/slice` works; and
  `just oracle-slice fixtures/3DBenchy.stl base-fff` and
  `just oracle-slice NotAFixture base-fff` behave as stated above, the first slicing and the second
  exiting non-zero naming the argument.
- `just assert-placement <fixture> <run-dir>` is a recipe in the `justfile` and exits 0 for a run of
  each of the three fixtures, and non-zero for a run whose reported object bounding box the
  fixture's record does not name.
- `oracle/assert-placement --list` prints one line per fixture record with all four items, which is
  the enumeration `42-compare-fixtures` reads.
- `vendor/README.md` has one row per fixture carrying all four items — frozen basename, vendored
  path, resolved instance transform (the identity), license — and states explicitly that upstream
  records no license for `resources/handy_models/`.
- `git status --porcelain vendor/OrcaSlicer` is empty and
  `git -C vendor/OrcaSlicer rev-parse HEAD` prints
  `f92a7c065c5718b8ef1baac1cd704df2911238f4`.
- `just check` passes, the branch adds no file under `intent/`, and it adds no line to
  `.github/workflows/ci.yml`.
