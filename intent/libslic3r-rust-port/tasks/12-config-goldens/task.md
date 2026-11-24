---
kind: task
step: 9
delivers: [R11]
depends: [04-oracle-slice, 05-oracle-provenance, 11-fixture-set]
size: 3
---
# Task: Commit the resolved-configuration goldens the port consumes

## What to build

The whole of plan step 9, "Resolved-config goldens": the committed goldens that are the port's
configuration input, the recipe that regenerates them, and the measured section of
`docs/ORACLE.md` that says how the exported JSON relates to the G-code's config block.

Add `port/goldens/configs/`, holding **one resolved configuration per configuration row in
`docs/ORACLE.md`**, named after that row, each produced by

```
just oracle-slice <fixture> <configuration> --export-settings
```

on an **unarmed** run — no `slicing_pipeline_plugin` value, so no golden carries that key — and
each carrying `oracle/provenance`'s header in its JSON form, a `provenance` object, which
`05-oracle-provenance` defines.

**`<fixture>` is the fixture that configuration's `docs/ORACLE.md` row names**, and this task waits
on `11-fixture-set` for it: a resolved configuration does not depend on the model, but the run that
produces it needs a model that `--arrange 0` can slice, and `oracle/provenance` refuses to print a
header for a fixture with no `vendor/README.md` record — which is why the dependency is on the
frozen set and not on a vendored path. The base FFF row's fixture is `3DBenchy.stl`, so
`just oracle-slice 3DBenchy base-fff --export-settings` is the command that produces that row's
golden, and `port/config-block-diff` below reads that same run's `plate_1.gcode`.

**How the header gets into the golden.** `oracle/slice` writes no header and `oracle/provenance`
only prints one, so the merge is a step of its own and this task owns it: `just config-goldens` runs
`oracle/slice <fixture> <configuration> --export-settings`, then calls
`oracle/provenance --oracle release --syntax json` for that fixture and configuration, and merges
the printed object into the exported JSON as its `provenance` key. The merge is performed by a
script this task adds, `port/merge-provenance`, which the recipe invokes; it is not inlined in the
`justfile` and it is not a flag on `oracle/slice`, which `04-oracle-slice` forbids. So
`just config-goldens` regenerates every golden through `oracle/slice` and adds **no second slicing
binary** — that is what the "and nothing else" below means — while the header and the merge are its
own work. No golden is hand-edited into existence.

These goldens are the port's configuration input. Preset files, the `inherits` chain and
preset-bundle resolution are out of scope for this intent, so a benchy difference cannot be
caused by preset resolution.

**The throwaway data directory is the care in this task.** `oracle/slice` never lets `--datadir`
be selected and always passes a fresh directory inside its scratch directory, and that is what
keeps the credential fields of an exported configuration empty. The seven `printhost_*` fields
are not alike, so check them as they are: `printhost_apikey`
(`vendor/OrcaSlicer/src/libslic3r/PrintConfig.cpp:1221`), `printhost_port` (`:1236`),
`printhost_cafile` (`:1244`), `printhost_user` (`:1254`) and `printhost_password` (`:1261`) are
`coString` and must export as the empty string; `printhost_ssl_ignore_revoke` (`:1269`) is
`coBool` and `printhost_authorization_type` (`:1282`) is `coEnum`, so they export as their
defaults and never as `""`. No golden is committed until the five string fields are empty.

**The measured section of `docs/ORACLE.md`.** Document, from a committed script-measured
comparison rather than from memory, the relationship between the exported JSON and the G-code's
`CONFIG_BLOCK`. Add that script as `port/config-block-diff`, a standalone PEP 723 executable
beside `port/inventory` and the plan's other committed scripts, with the recipe
`just config-block-diff <golden> <gcode>` — it does
**not** go under `port/goldens/`, because `oracle/provenance --check` covers everything in that
tree and would demand a provenance header on a script that has no oracle and no fixture. It takes
a golden and the `plate_1.gcode` from the same run, and prints the
keys the JSON carries that the config block does not — among them the seven `printhost_*` fields,
which is why the throwaway datadir is mandatory — and the keys the config block carries that the
JSON does not. The second set is exactly two, and the port derives both at emission time:
`first_layer_temperature` and `first_layer_bed_temperature`, which the oracle writes after
`append_full_config` at `vendor/OrcaSlicer/src/libslic3r/GCode.cpp:4325-4335`. The section names
them and cites that line range, and the figures in it come from the script's output.

Say in the same section what stays in scope, because the deduction of preset resolution is easy
to over-read: **emission is not resolution.** The port writes the G-code `HEADER_BLOCK` and
`CONFIG_BLOCK` from the resolved input, selecting and formatting the same keys the oracle writes
(`vendor/OrcaSlicer/src/libslic3r/GCode.cpp:4255-4335`), and the print-time estimator whose line
the header carries (`vendor/OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:1310`) is in scope,
because L1 compares both.

The goldens are byte-stable across runs with no normalisation. This task does not measure that:
`27-oracle-determinism` records it, and this task neither writes nor edits anything under
`port/reports/`.

A configuration row the pinned CLI cannot construct gets no golden here. That is C12's recorded
decision: the row stays marked so in `docs/ORACLE.md`, which `04-oracle-slice` owns, and its
golden arrives through the built-oracle driver in `25-stage-dumper`. It is not worked around, and
no golden is hand-edited into existence.

Two files this task adds are not in the plan's "Files that change" list — `port/config-block-diff`
and `port/merge-provenance` — so a reviewer of this pull request is not surprised by them. The plan
names the measured `docs/ORACLE.md` section and the goldens and leaves the scripts that produce them
unlisted; `plan.md` is on `main` and is not edited from this branch, so the two paths are declared
here instead.

Do not add a line to `.github/workflows/ci.yml`: `24-port-check-ci` creates the jobs, and
`oracle/provenance --check` over these paths joins the checks there. Nothing under
`vendor/OrcaSlicer/` changes.

## Done when

- `port/goldens/configs/` holds one `.json` per configuration row in `docs/ORACLE.md` that the
  pinned CLI can construct, each named after its row, and a row the CLI cannot construct is
  marked so in `docs/ORACLE.md` rather than given a golden.
- `just config-goldens` regenerates every golden through `just oracle-slice ... --export-settings`
  with a fresh throwaway `--datadir`, and re-running it leaves each committed golden
  byte-identical.
- `oracle/provenance --check port/goldens/configs/*.json` exits 0 and reports the number of files
  checked. The files are passed individually, because `05-oracle-provenance` defines the interface
  as `--check <path>...` over files and this task does not extend it to walk a directory.
- Every golden carries a `provenance` key holding the nine items `oracle/provenance` prints, and
  `just config-goldens` produces it by calling
  `oracle/provenance --oracle release --syntax json` and merging through `port/merge-provenance`;
  `grep -n 'provenance' justfile port/merge-provenance` shows that call chain and
  `git diff oracle/slice` is empty.
- `grep -o '"printhost_apikey": *"[^"]*"' port/goldens/configs/*.json` prints the key with an
  empty value for every golden, and the same holds for `printhost_port`, `printhost_cafile`,
  `printhost_user` and `printhost_password`.
- No golden carries a non-empty `slicing_pipeline_plugin` value, which is what makes every run
  that produced one unarmed.
- `just config-block-diff port/goldens/configs/base-fff.json <the same run's plate_1.gcode>` prints
  both key sets, `docs/ORACLE.md` carries its output rather than hand-written figures, and no file
  was added under `port/goldens/` other than the goldens themselves.
- A person reads `docs/ORACLE.md` and finds the two G-code-only keys named —
  `first_layer_temperature` and `first_layer_bed_temperature` — with
  `src/libslic3r/GCode.cpp:4325-4335` cited, and finds the statement that emission stays in scope
  with `GCode.cpp:4255-4335` and `GCode/GCodeProcessor.cpp:1310` cited.
- The branch writes nothing under `port/reports/`, adds no line to
  `.github/workflows/ci.yml`, and leaves `docs/ORACLE.md`'s configuration table otherwise as
  `04-oracle-slice` wrote it.
- `git status --porcelain vendor/OrcaSlicer` is empty, `just check` passes, and the branch adds
  no file under `intent/`.
