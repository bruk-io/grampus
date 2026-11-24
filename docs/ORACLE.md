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

`Fixtures` names a frozen basename under `fixtures/` (R25); `oracle/slice` resolves a bare stem
with no path separator and no extension against that directory by appending `.stl` (added in
`11-fixture-set`).

| Configuration | Machine | Process | Filament(s) | Overrides | Fixtures |
|---|---|---|---|---|---|
| `base-fff` | `machine/Bambu Lab X1 Carbon 0.4 nozzle.json` | `process/0.20mm Standard @BBL X1C.json` | `filament/Bambu PLA Basic @BBL X1C.json` | (none) | `frog_legs` |
| `tree-support` | `machine/Bambu Lab X1 Carbon 0.4 nozzle.json` | `process/0.20mm Standard @BBL X1C.json` | `filament/Bambu PLA Basic @BBL X1C.json` | `enable_support=1`; `support_type=tree(auto)` | `frog_legs` |
| `wipe-tower-multi-filament` | `machine/Bambu Lab X1 Carbon 0.4 nozzle.json` | `process/0.20mm Standard @BBL X1C.json` | `filament/Bambu PLA Basic @BBL X1C.json`; `filament/Bambu PLA Basic @BBL X1C.json` | `enable_prime_tower=1`; `outer_wall_filament_id=2` | `frog_legs` |
| `fuzzy-skin` | `machine/Bambu Lab X1 Carbon 0.4 nozzle.json` | `process/0.20mm Standard @BBL X1C.json` | `filament/Bambu PLA Basic @BBL X1C.json` | `fuzzy_skin=external` | `frog_legs` |

All four rows were attempted directly on the pinned 2.5.2 binary (C12) and all four slice through
`just oracle-slice` with exit 0 and a non-empty `plate_1.gcode`; none is marked "not constructible
through the pinned CLI". This differs from the only attempt on record (a 2.4.2 build, modified
process JSON, exit 239/251/254): the mechanism here is the CLI's own `--<key>=<value>` override
flags (every key named above is CLI-exposed — not `nocli` — in
`vendor/OrcaSlicer/src/libslic3r/PrintConfig.cpp`), not a doctored preset file, and `settings.json`
exported from each run confirms the override landed in the resolved config. All of it was measured
before `11-fixture-set` fixed `frog_legs`'s placement, against the vendored
`tests/data/frog_legs.obj` at its own, untranslated coordinates; the byte counts, filament lengths
and marker counts below are that measurement's record and are unaffected by the model's position
on the bed, but are not asserted to reproduce to the byte against the now-translated
`fixtures/frog_legs.stl`.

**`wipe-tower-multi-filament`'s third element, measured.** task.md names three things for this row:
`enable_prime_tower=1`, two `--load-filaments` entries so the print has two filaments, and "the
single object's filament assignment switching between them". `enable_prime_tower=1` alone does not
reach the third element — the row's own `settings.json` shows the key is already `1` in the
inherited process preset, so the override is a no-op by itself, and without a per-region or
per-height assignment the single mesh never draws from the second filament (confirmed: with only
`enable_prime_tower=1`, `; filament used [mm]` reads `589.35, 0.00` and no `; FEATURE: Prime tower`
line appears). `outer_wall_filament_id` (`coInt`, CLI-exposed, not `nocli`,
`vendor/OrcaSlicer/src/libslic3r/PrintConfig.cpp:5927`) assigns the single object's outer walls to
the second filament by region, which is what the row's override cell now adds. Measured on the
pinned 2.5.2 binary through `just oracle-slice`: exit 0, `plate_1.gcode` 632859 bytes,
`; filament used [mm] = 786.64, 555.17`, 20 `; FEATURE: Prime tower` markers, 14 toolchanges — the
single object's filament assignment does switch between the two loaded filaments. `--load-custom-gcodes`'s
JSON `ToolChange` items are a second, separately-measured mechanism for a per-height switch on this
same mesh (print_z below `frog_legs.obj`'s 2 mm height), but this task builds neither
`--load-custom-gcodes` nor `--load-assemble-list` support into `oracle/slice` — the row's override
cell is enough on its own, so no second mechanism is added here.

**`fuzzy-skin`'s G-code is not byte-reproducible run to run.** Three consecutive
`just oracle-slice vendor/OrcaSlicer/tests/data/frog_legs.obj fuzzy-skin` runs measured
`plate_1.gcode` at 535737, 534845 and 532275 bytes; `base-fff` and `tree-support` reproduce
byte-exactly across repeated runs, so the variance is specific to this row, not the harness. The
cause is in the pin: `vendor/OrcaSlicer/src/libslic3r/Feature/FuzzySkin/FuzzySkin.cpp:38-42`
seeds each run's `random_value()` from `std::random_device`/a thread id, with no fixed seed, and
that value is consumed by the perimeter generator at `FuzzySkin.cpp:320,337,375,393`. Every run
still exits 0 with a non-empty `plate_1.gcode`, which is all this row's Done-when line requires;
`27-oracle-determinism` is where a normalisation or tolerance for this row's byte comparison has
to be decided.
