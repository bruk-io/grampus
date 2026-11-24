---
kind: task
step: 2
delivers: [R2]
depends: [01-oracle-fetch]
size: 5
---
# Task: Run the pinned oracle headlessly on a named configuration

## What to build

This is the whole of the plan's step 2, "Run the oracle headlessly on a named configuration". It
adds `oracle/slice`, the `just oracle-slice` recipe, and the configuration table of
`docs/ORACLE.md`.

`oracle/slice` is a standalone executable with PEP 723 inline metadata and no file extension, run
through `uv`, as every script under `oracle/` and `port/` in this repository is. The recipe is

```
just oracle-slice <fixture> <configuration> [--export-settings] [--export-statistics] [--out <dir>]
```

and it forwards its arguments to `oracle/slice` unchanged. That is the whole interface, and the
callers the plan's step 2 names are its whole caller set: `12-config-goldens` (step 9, which is the
one documented use of a flag, `--export-settings`), `27-oracle-determinism` (step 6),
`28-oracle-parity` (step 10) and the three comparison commands, `41-compare-benchy`,
`42-compare-fixtures` and `43-compare-configurations` (steps 25, 26 and 27). None of them passes a
flag outside that interface, so no flag is added for one caller's convenience. The stage dumper of
`25-stage-dumper` is **not** a caller: its configuration driver is its own program, and it reads a
resolved-config JSON or applies in-code overrides rather than running this script.

`<fixture>` is either a frozen basename in `fixtures/` or a path to a model file. `fixtures/` does
not exist yet (it is `11-fixture-set`), and that task is where `oracle/slice` learns to resolve a
bare stem such as `20mm_cube` against `fixtures/`; until then a fixture argument is a basename with
its extension or a path. This task's own proof passes a path:
`vendor/OrcaSlicer/tests/data/frog_legs.obj`. That model is the one vendored model that
`--arrange 0` can slice before the fixtures exist, because its own coordinates — 34.4 to 86.5 in X
and 9.1 to 58.9 in Y — are inside the X1C's printable area and clear of that machine's
`bed_exclude_area` of `0x0 18x0 18x28 0x28`. Do not reach for `20mm_cube.obj` instead: it spans 0
to 20 in both X and Y at the pin, which is inside the exclusion area, and with `--arrange 0`
nothing in the oracle moves it out.

`--export-settings` and `--export-statistics` are off by default and switch on the oracle's two
JSON exports, written beside the G-code as `settings.json` and the oracle's own
`plate_1.statistics.json`. `--out` defaults to a fresh directory under the scratch directory.
`--datadir` is never selectable: it is always a fresh directory inside the same scratch directory,
because that is what keeps the seven `printhost_*` credential fields of an exported config empty.

The script resolves `<configuration>` to a row in the configuration table of `docs/ORACLE.md`,
`cd`s into a fresh scratch directory — a failing run writes `00000.log` into `$PWD`, which would
dirty the working tree — and invokes the binary `01-oracle-fetch` leaves at
`oracle/.cache/app/OrcaSlicer.app/Contents/MacOS/OrcaSlicer` as:

```
"$BIN" --datadir "$SCRATCH/datadir" \
  --load-settings "<machine json>;<process json>" \
  --load-filaments "<filament json>" \
  --slice 0 --arrange 0 --outputdir "$OUT" \
  [--export-settings "$OUT/settings.json"] [--export-statistics] <fixture>
```

**Shape facts the script must keep.** Each of these silently corrupts a comparison rather than
failing, which is why they are written out here:

- `--load-settings` takes one `;`-separated string holding the machine and the process preset
  together, not two options.
- `--load-filaments` is always passed explicitly. Omitting it still exits 0 and slices with a
  different filament — 9,910 differing lines against the pinned-filament run.
- The filament path is pinned exactly, because a filament that does not match the machine's nozzle
  is accepted without warning.
- `--outputdir` is mandatory. Omitting it exits 0 and writes no G-code anywhere.
- `--arrange 0` is passed on every invocation (R24).
- There is no `-o` or `--output`. The written file is named `plate_<N>.gcode`.
- `--datadir` is a fresh directory per run.
- The script does not read `result.json`: macOS does not write one.

**Two shapes the oracle will not catch, so the script does.**

- A configuration row that names no filament is rejected by the script, before it invokes the
  binary, naming the row — because the oracle exits 0 on a missing `--load-filaments`. A filament
  path that is named but absent from disk is left to the oracle's own `CLI_CONFIG_FILE_ERROR`.
- The script exits non-zero unless a non-empty `plate_1.gcode` was written.

**Exit codes.** The oracle's negative codes reach the shell modulo 256, so the script converts
`rc - 256 if rc > 127 else rc` and looks the result up in a table transcribed from
`vendor/OrcaSlicer/src/libslic3r/Utils.hpp:42-94`. The table holds exactly these nine entries:
`0` `CLI_SUCCESS`, `-2` `CLI_INVALID_PARAMS`, `-5` `CLI_CONFIG_FILE_ERROR`, `-17`
`CLI_PROCESS_NOT_COMPATIBLE`, `-50` `CLI_NO_SUITABLE_OBJECTS`, `-51` `CLI_VALIDATE_ERROR`, `-60`
`CLI_NO_SUITABLE_OBJECTS_AFTER_SKIP`, `-100` `CLI_SLICING_ERROR`, `-101`
`CLI_GCODE_PATH_CONFLICTS`. An unmapped code is reported as its converted value, not as a success.

**`docs/ORACLE.md` gains the configuration table**, appended below the two sections
`01-oracle-fetch` created — the asset and its hash, and the statement that
`/Applications/OrcaSlicer.app` (2.4.2) and `/Applications/BambuStudio.app` are never read by any
command in this repository. Those two sections are not rewritten here. Every configuration row
names the machine, process and filament preset paths under
`vendor/OrcaSlicer/resources/profiles/BBL/` — the app bundle's own presets are packed into an
opaque `BBL.opc` and its loose `machine/` and `process/` directories are empty, so the vendored
tree is the only source — plus the explicit setting overrides that switch the row's feature on, and
the fixtures it uses. The base FFF row is `machine/Bambu Lab X1 Carbon 0.4 nozzle.json`,
`process/0.20mm Standard @BBL X1C.json`, `filament/Bambu PLA Basic @BBL X1C.json`. Rows for tree
support, wipe tower with multi-filament, and fuzzy skin are added here as overrides over the base
row.

**The three non-base rows' overrides are named here, not chosen by the builder**, because
`27-oracle-determinism`, `41-compare-benchy` and `43-compare-configurations` all gate on these rows
and because a row marked "not constructible through the pinned CLI" has to be a fact about the pin
rather than about one builder's choice of keys. Each option below is read from
`vendor/OrcaSlicer/src/libslic3r/PrintConfig.cpp` at the pin:

- **tree support**: `enable_support` = `1` (`PrintConfig.cpp:7316`, `coBool`) and `support_type` =
  `tree(auto)` (`:7323`, `coEnum`, whose enum values at `:7329-7330` are `normal(auto)` and
  `tree(auto)`). Nothing else.
- **wipe tower with multi-filament**: `enable_prime_tower` = `1` (`:8255`, `coBool`), with
  `--load-filaments` naming `filament/Bambu PLA Basic @BBL X1C.json` twice so the print has two
  filaments, and the single object's filament assignment switching between them. R24 is why the
  second filament goes on the one object rather than on a second object.
- **fuzzy skin**: `fuzzy_skin` = `external` (`:4238`, `coEnum`, whose enum values at `:4243-4247`
  are `none`, `external`, `all` and `allwalls`). Nothing else.

A row whose overrides the pinned CLI refuses is marked as C12's decision says and keeps these same
overrides in its row, so the built-oracle driver in `25-stage-dumper` applies the same keys.

**What this task does not build.** It does not format a provenance header of its own: that is
`oracle/provenance` in `05-oracle-provenance`. `oracle/slice` writes no header and takes no
dependency on that script — it is not called from here, and no call site is left for it. The
outputs that carry a header are the resolved-config goldens of `12-config-goldens`, and that task
is the one that calls `oracle/provenance`. It does not add fixtures (`11-fixture-set`), the gate table
(`27-oracle-determinism`), or any CI line (`24-port-check-ci`). Nothing under
`vendor/OrcaSlicer/` changes and the submodule pointer stays at
`f92a7c065c5718b8ef1baac1cd704df2911238f4`.

**Why this is a 5.** Three things make it one, and the first two are unknowns no amount of reading
the pin answers. First, `--arrange 0` is read from the pin and not yet run: the option is live
(`vendor/OrcaSlicer/src/libslic3r/PrintConfig.cpp:12935-12940`, `coInt`, default `0`) and its
handler sets `need_arrange = false` (`src/OrcaSlicer.cpp:5072-5078`), but the one invocation on
record omitted `--arrange` entirely and let the CLI's arranger place the mesh, so the combination
this repository depends on — a disabled arranger with a mesh nothing moves — is first exercised
here. If the pinned CLI will not slice a mesh without arranging it, that is reported, not worked
around: the release binary then stops being usable as an end-to-end oracle and the work moves to
the built-oracle driver in `25-stage-dumper`. Second, the three non-base configuration rows may not
be constructible through the pinned CLI at all, which the spec carries as C12 — the only attempt on
record, on a 2.4.2 build, exited 239, 251 or 254 with no diagnostic. C12's recorded decision
applies: a row the pinned CLI cannot construct is **marked so in its `docs/ORACLE.md` row and
handed to the built-oracle driver in `25-stage-dumper`**, and marking a row that way is a result of
this task rather than a failure of it. Third, the negative exit codes arrive modulo 256 and have to
be converted before they can be named at all.

**What it waits on.** `01-oracle-fetch`, for the verified pinned binary at
`oracle/.cache/app/OrcaSlicer.app/Contents/MacOS/OrcaSlicer` and the first two sections of
`docs/ORACLE.md`.

## Done when

- `oracle/slice` exists, has no file extension, carries PEP 723 inline metadata, and runs through
  `uv`; `just oracle-slice <fixture> <configuration> [--export-settings] [--export-statistics]
  [--out <dir>]` forwards its arguments to it and adds no other flag.
- `just oracle-slice vendor/OrcaSlicer/tests/data/frog_legs.obj base-fff` exits 0 and prints the
  path and the byte size of a non-empty `plate_1.gcode`.
- Run with a configuration row whose machine and process presets have been removed, so
  `--load-settings` is omitted, the command exits non-zero and prints the named failure
  `CLI_VALIDATE_ERROR` for exit `-51` arriving as 205. **That row is a local edit the builder makes
  and discards, not a committed row**: no negative-test row reaches `docs/ORACLE.md`, and this
  task's pull request body carries the transcript of the run, naming the exit code received.
- Run with a configuration row whose filament path has been removed, the command exits non-zero
  **before the binary is invoked** and prints the name of the row. That row is likewise a discarded
  local edit, with its transcript in the pull request body.
- `grep -n 'arrange' oracle/slice` shows `--arrange 0` passed on every invocation of the binary.
- The exit-code table in `oracle/slice` holds the nine entries above, cites
  `vendor/OrcaSlicer/src/libslic3r/Utils.hpp:42-94`, and converts a received code with
  `rc - 256 if rc > 127 else rc` before looking it up.
- A run with `--export-settings` and `--export-statistics` writes `settings.json` and
  `plate_1.statistics.json` beside the G-code, and a run without them writes neither.
- Every run uses a fresh `--datadir` and a fresh working directory, and `--datadir` cannot be set
  from the command line; after a failing run `git status` shows no `00000.log` and no other new
  file in the working tree.
- `oracle/slice` contains no provenance-header formatting of its own.
- `docs/ORACLE.md` has a configuration table whose base FFF row names
  `machine/Bambu Lab X1 Carbon 0.4 nozzle.json`, `process/0.20mm Standard @BBL X1C.json` and
  `filament/Bambu PLA Basic @BBL X1C.json` under `vendor/OrcaSlicer/resources/profiles/BBL/`, and
  rows for tree support, wipe tower with multi-filament, and fuzzy skin, each as overrides over the
  base row, each naming the fixtures it uses; the two sections `01-oracle-fetch` wrote are
  unchanged.
- Each of the three non-base rows either slices through `just oracle-slice` with exit 0 and a
  non-empty `plate_1.gcode`, or is marked in its row as not constructible through the pinned CLI
  and handed to the built-oracle driver; the pull request says which, and prints the failing exit
  code's name for each row that is marked.
- `just check` passes, `git -C vendor/OrcaSlicer rev-parse HEAD` prints
  `f92a7c065c5718b8ef1baac1cd704df2911238f4`, and `git status` reports no change under
  `vendor/OrcaSlicer/`.
