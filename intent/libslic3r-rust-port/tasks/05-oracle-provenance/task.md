---
kind: task
step: 3
delivers: [R7]
depends: [04-oracle-slice]
size: 3
---
# Task: Print and check the provenance header every oracle output carries

## What to build

This is the whole of the plan's step 3, "Record what produced every oracle output". It adds one
script, `oracle/provenance`, a standalone executable with PEP 723 inline metadata and no file
extension, run through `uv`, as every script under `oracle/` and `port/` in this repository is.

The script has two modes. In its printing mode it prints the provenance header that every golden
file, dump and report an oracle produced carries. It exists so the scripts of the later oracle
steps — the resolved-config goldens of `12-config-goldens`, the determinism report of
`27-oracle-determinism`, the stage dumper of `25-stage-dumper` and `26-stage-dump-seams`, and the
parity report of `28-oracle-parity` — call one formatter instead of each writing their own.
**`oracle/slice` is not a caller.** Task `04-oracle-slice` states that it writes no header and
leaves no call site for one, because its outputs are scratch artifacts of a run rather than
committed artifacts; the committed artifact derived from such a run is a golden, and
`12-config-goldens` is what adds the header to it. The plan's step 3 lists step 2 among the callers;
that reading is superseded here so the two tasks do not merge disagreeing, and nothing is lost,
because every committed artifact still gets its header from this one formatter.

**The nine recorded items**, which is the whole list:

1. The pinned commit `f92a7c065c5718b8ef1baac1cd704df2911238f4`, read from
   `git -C vendor/OrcaSlicer rev-parse HEAD` and asserted against that literal.
2. Which oracle produced the output — the pinned release binary or the locally built library — and
   its build flags. **The oracle is a required option, `--oracle release|built`**; the script exits
   non-zero naming the option when it is absent, so no header is ever written without an oracle.
   For `release` the flags field is the literal `released build, no local flags`. For `built` the
   flags are read out of the built-oracle flags section of `docs/ORACLE.md`, which
   `13-build-libslic3r` writes; while that section does not exist the script writes the field as
   `unrecorded` and says so on stderr, rather than failing, because `--oracle built` has no caller
   until that task has landed.
3. The OS, from `sw_vers`.
4. The architecture, from `uname -m`.
5. The compiler version, from `clang --version`.
6. The value of `SCALING_FACTOR`.
7. The full preset set and the configuration's overrides.
8. The fixture's basename.
9. The fixture's resolved instance transform, read from that fixture's record in
   `vendor/README.md`.

Items 8 and 9 come from a `vendor/README.md` fixture record that `11-fixture-set` writes, and the
records do not exist yet. Where a fixture named on the command line has no record, the script exits
non-zero and names the fixture rather than printing an empty field, so no header is ever written
with a blank transform.

**That is why the nine-item proof runs against a scratch record, and this task stays ahead of
`11-fixture-set`.** `11-fixture-set` depends on this task — it is the task whose records this script
reads — so the dependency cannot run the other way, and on this branch no record exists for any
fixture. So the proof of items 8 and 9 is run against a record the builder writes into a scratch
copy of `vendor/README.md` outside the repository, passed to the script with
`--readme <path>` (which defaults to `vendor/README.md`), and the "Done when" line below names that
scratch copy. Nothing in this branch commits a fixture record; the real records arrive with
`11-fixture-set` and the same code path reads them unchanged.

**Three syntaxes**, because the header has to suit the artifact it sits in:

- A `;` comment block, for G-code goldens **and** for the stage dumps of `25-stage-dumper` and
  `26-stage-dump-seams`, which are line-oriented text and carry the same block.
- A `provenance` object, for the JSON artifacts.
- A Markdown table, for the reports an oracle produced.

**The syntax is selected by one option whose spelling is fixed here: `--syntax gcode|json|markdown`,
required, with no default.** The artifact's path never selects the syntax. The spelling is written
into this task rather than left to its pull request because `12-config-goldens`,
`25-stage-dumper`, `26-stage-dump-seams`, `27-oracle-determinism` and `28-oracle-parity` are built
concurrently in separate worktrees from their own task files and each has to write the call without
reading this one's pull request. A run with no `--syntax` exits non-zero naming the option.

**The checking mode.** `oracle/provenance --check <path>...` exits non-zero when a file it covers
has no header, or has one that disagrees with the pin. What it covers is named here and in the
script and nowhere else, because the header's items are undefined for a report no oracle produced:

- **Covered:** everything under `port/goldens/`; every committed stage dump, which is every file
  matching `port/dumps/*/*.dump`; and the two reports an oracle run writes,
  `port/reports/determinism.md` and `port/reports/oracle-parity.md`.
- **Excluded by name:** the four static-analysis reports `port/reports/order-containers.md`,
  `port/reports/order-comparators.md`, `port/reports/divergences.md` and
  `port/reports/perf-backlog.md`. These are grepped out of the vendored tree and the port with no
  oracle and no fixture, so they have no provenance to record, and `--check` **fails** when it is
  pointed at one rather than passing it silently.

**The committed stage dumps have a path, and it is named here because four later tasks have to
agree on it.** A committed dump lives at `port/dumps/<fixture-stem>-<configuration>/<seam>.dump` —
one directory per fixture and configuration pair, one file per fired seam, named by the seam's enum
value. `25-stage-dumper` is the task that writes there; `24-port-check-ci` passes the same glob into
`just port-check`; `32-proof-dumps-fuzz` reads a dump from there as its proof path 2. The plan's
"Files that change" list names only `oracle/stage-dump/`'s sources and does not carry
`port/dumps/`, and `plan.md` is on `main` and is not edited from this branch, so the path is fixed
here instead and this is where a reviewer of those four tasks reads it.

**An absent covered path is not a failure, and a path outside the covered set is skipped.** When
`--check` is given a path, or a glob, that matches nothing, it counts zero files and exits 0; a
count of zero is a pass. When it is given a path that is neither covered nor one of the four
excluded reports, it skips it and counts it as skipped rather than failing. That is what lets
`24-port-check-ci` put the four covered groups into `just port-check` on a tree where none of them
exists yet — which is every tree until `12-config-goldens`, `25-stage-dumper`,
`27-oracle-determinism` and `28-oracle-parity` have landed.

**What this task does not build.** It adds no `just` recipe, no line in `.asdlc/checks` and no CI
step: the plan wires `oracle/provenance --check` into `just port-check` with exactly those covered
paths in `24-port-check-ci`, and that task owns the wiring. It produces no goldens
(`12-config-goldens`), no dumps (`25-stage-dumper`) and no reports, and it does not edit the
scripts that will call it. Nothing under `vendor/OrcaSlicer/` changes and the submodule pointer
stays at `f92a7c065c5718b8ef1baac1cd704df2911238f4`.

**What it waits on.** `04-oracle-slice`, for the configuration table of `docs/ORACLE.md`. Item 7 is
the full preset set and the configuration's overrides, and that table is where a configuration's
presets and overrides live; `01-oracle-fetch` creates `docs/ORACLE.md` with the asset section and
the `/Applications` statement and no configuration table, so `base-fff` cannot be resolved before
`04-oracle-slice` merges. It waits on nothing else: items 1 and 3 to 6 come from the vendored tree
and the machine, item 2 from its own option, and items 8 and 9 from a `vendor/README.md` record,
proven here against a scratch copy as described above. It does **not** wait on `11-fixture-set`,
which waits on this task instead.

## Done when

- `oracle/provenance` exists, has no file extension, carries PEP 723 inline metadata and runs
  through `uv`.
- Run in its printing mode for the base FFF configuration, with `--oracle release`, `--syntax gcode`
  and `--readme` pointing at a scratch copy of `vendor/README.md` that holds one fixture record, it
  prints a header in which a reviewer can point to each of the nine items above, with a non-empty
  transform in item 9.
- It prints the header in all three syntaxes, selected only by `--syntax`: `--syntax gcode` gives
  the `;` comment block, `--syntax json` the `provenance` object, and `--syntax markdown` the
  Markdown table. A run with no `--syntax` exits non-zero and names the option, and the artifact's
  path never changes the syntax chosen.
- A run with no `--oracle` exits non-zero and names the option.
- It reads the pinned commit with `git -C vendor/OrcaSlicer rev-parse HEAD`, asserts it against the
  literal `f92a7c065c5718b8ef1baac1cd704df2911238f4`, and exits non-zero when the two disagree.
- Asked for a header for a fixture that has no record in `vendor/README.md`, it exits non-zero and
  names the fixture, and prints no header.
- `oracle/provenance --check <path>...` exits 0 over a set of files that all carry a current header
  and prints the number of files it checked. No covered file exists on this branch, so the positive
  case is run over scratch copies the builder makes at the covered paths — one golden under
  `port/goldens/` and one `port/dumps/<pair>/<seam>.dump` — each written by this script's own
  printing mode, and the "Done when" is answered against that run.
- `oracle/provenance --check` over a covered path that matches no file exits 0 and prints a count of
  zero, and over a path that is neither covered nor one of the four excluded reports exits 0 and
  reports it as skipped.
- It exits non-zero on a file with no header, and non-zero on a file whose header names a commit
  other than the pin.
- It exits non-zero, naming the file, when it is pointed at any of
  `port/reports/order-containers.md`, `port/reports/order-comparators.md`,
  `port/reports/divergences.md` or `port/reports/perf-backlog.md`.
- The covered set in the script is exactly `port/goldens/`, `port/dumps/*/*.dump`,
  `port/reports/determinism.md` and `port/reports/oracle-parity.md`, and
  `grep -n 'port/dumps' oracle/provenance` finds that glob.
- `grep -n 'slice' oracle/provenance` finds no invocation of `oracle/slice`, and this branch does
  not edit `oracle/slice`.
- The branch adds no `just` recipe, no line in `.asdlc/checks` and no step in
  `.github/workflows/ci.yml`.
- `just check` passes, `git -C vendor/OrcaSlicer rev-parse HEAD` prints
  `f92a7c065c5718b8ef1baac1cd704df2911238f4`, and `git status` reports no change under
  `vendor/OrcaSlicer/`.
