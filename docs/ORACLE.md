# The oracle

The test oracle is the real OrcaSlicer binary, run headlessly, that every port unit is proven
against. This document is added to by each oracle step; `oracle/fetch-release` (step 1) writes
only the two sections below.

## The asset and its hash

`just oracle-fetch` fetches and verifies this release:

| | |
|---|---|
| Release tag | `v2.5.2` |
| Release asset | `OrcaSlicer_Mac_universal_V2.5.2.dmg` |
| Recorded sha256 | `6ea1afcdffb7e9ab011e2d6f6db665cb18463d1c4774591cd5db7ee0225479d9` (`vendor/README.md` is the source of truth; this table quotes it) |
| Bundle written to | `oracle/.cache/app/OrcaSlicer.app` |
| Binary used | `oracle/.cache/app/OrcaSlicer.app/Contents/MacOS/OrcaSlicer` |

The binary is the GUI bundle's executable. It is also the CLI and runs headless with no
`DISPLAY`; there is no separate console executable in the bundle. `oracle/fetch-release` never
writes the recorded sha256 — it only reads it and fails when a download does not match it.

## What is never read

`/Applications/OrcaSlicer.app` (2.4.2) and `/Applications/BambuStudio.app` are never read by any
command in this repository. The oracle is always the pinned binary `just oracle-fetch` stages
under `oracle/.cache/`, never an app already installed on the machine running the command.

## Configurations

`oracle/slice` (step 2) resolves `just oracle-slice <fixture> <configuration>`'s `<configuration>`
argument to a row below by the `Configuration` column, exactly. Every preset path is relative to
`vendor/OrcaSlicer/resources/profiles/BBL/`. `Filament(s)` holds one or more `;`-separated preset
paths, loaded in order via `--load-filaments`; `Overrides` holds zero or more `;`-separated
`key=value` pairs, passed to the binary as `--<key with _ replaced by ->=<value>` — the CLI's
equals form, which every option type (including a `coBool` one) accepts as a value without the
space-separated form's risk of being parsed as a stray positional file argument instead. A row
naming no filament is rejected by `oracle/slice` itself, before the binary runs.

Until `11-fixture-set` lands, `Fixtures` names a path under `vendor/OrcaSlicer/`, not a frozen
`fixtures/` basename.

| Configuration | Machine | Process | Filament(s) | Overrides | Fixtures |
|---|---|---|---|---|---|
| `base-fff` | `machine/Bambu Lab X1 Carbon 0.4 nozzle.json` | `process/0.20mm Standard @BBL X1C.json` | `filament/Bambu PLA Basic @BBL X1C.json` | (none) | `vendor/OrcaSlicer/tests/data/frog_legs.obj` |
| `tree-support` | `machine/Bambu Lab X1 Carbon 0.4 nozzle.json` | `process/0.20mm Standard @BBL X1C.json` | `filament/Bambu PLA Basic @BBL X1C.json` | `enable_support=1`; `support_type=tree(auto)` | `vendor/OrcaSlicer/tests/data/frog_legs.obj` |
| `wipe-tower-multi-filament` | `machine/Bambu Lab X1 Carbon 0.4 nozzle.json` | `process/0.20mm Standard @BBL X1C.json` | `filament/Bambu PLA Basic @BBL X1C.json`; `filament/Bambu PLA Basic @BBL X1C.json` | `enable_prime_tower=1` | `vendor/OrcaSlicer/tests/data/frog_legs.obj` |
| `fuzzy-skin` | `machine/Bambu Lab X1 Carbon 0.4 nozzle.json` | `process/0.20mm Standard @BBL X1C.json` | `filament/Bambu PLA Basic @BBL X1C.json` | `fuzzy_skin=external` | `vendor/OrcaSlicer/tests/data/frog_legs.obj` |

All four rows were attempted directly on the pinned 2.5.2 binary (C12) and all four slice through
`just oracle-slice` with exit 0 and a non-empty `plate_1.gcode`; none is marked "not constructible
through the pinned CLI". This differs from the only attempt on record (a 2.4.2 build, modified
process JSON, exit 239/251/254): the mechanism here is the CLI's own `--<key>=<value>` override
flags (every key named above is CLI-exposed — not `nocli` — in
`vendor/OrcaSlicer/src/libslic3r/PrintConfig.cpp`), not a doctored preset file, and `settings.json`
exported from each run confirms the override landed in the resolved config.

**`wipe-tower-multi-filament` caveat, measured, not a `docs/ORACLE.md`-level failure.** `frog_legs.obj`
is a single untextured mesh (no `o`/`g` groups), and the pinned CLI exposes no flag to split one
input mesh's filament assignment by region or by height range from the command line (only
`--load-assemble-list`'s project-file format can, which this task does not build). So although the
row slices at exit 0 with `enable_prime_tower` on and two filaments loaded, the G-code's own
`; filament used [mm]` line shows the second filament at `0.00` — the single object never actually
switches. The row is left as is (it is the override set the task fixes, "nothing else"); whichever
of `25-stage-dumper`, `27-oracle-determinism` or `12-config-goldens` first needs an input that
truly exercises per-region or per-height filament switching has to supply one, through the built
driver or a richer fixture.
