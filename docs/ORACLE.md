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

`Fixtures` names a frozen basename under `fixtures/` (R25), exactly as `ls fixtures/` prints it,
and that exact string is directly usable as `oracle/slice`'s `<fixture>` argument (e.g.
`just oracle-slice frog_legs.stl base-fff`): a separator-free argument ending in `.stl` resolves
against `fixtures/` when a file of that name is committed there. `oracle/slice` also accepts the
bare stem with no path separator and no extension, resolved against that directory by appending
`.stl` (added in `11-fixture-set`).

| Configuration | Machine | Process | Filament(s) | Overrides | Fixtures |
|---|---|---|---|---|---|
| `base-fff` | `machine/Bambu Lab X1 Carbon 0.4 nozzle.json` | `process/0.20mm Standard @BBL X1C.json` | `filament/Bambu PLA Basic @BBL X1C.json` | (none) | `frog_legs.stl` |
| `tree-support` | `machine/Bambu Lab X1 Carbon 0.4 nozzle.json` | `process/0.20mm Standard @BBL X1C.json` | `filament/Bambu PLA Basic @BBL X1C.json` | `enable_support=1`; `support_type=tree(auto)` | `frog_legs.stl` |
| `wipe-tower-multi-filament` | `machine/Bambu Lab X1 Carbon 0.4 nozzle.json` | `process/0.20mm Standard @BBL X1C.json` | `filament/Bambu PLA Basic @BBL X1C.json`; `filament/Bambu PLA Basic @BBL X1C.json` | `enable_prime_tower=1`; `outer_wall_filament_id=2` | `frog_legs.stl` |
| `fuzzy-skin` | `machine/Bambu Lab X1 Carbon 0.4 nozzle.json` | `process/0.20mm Standard @BBL X1C.json` | `filament/Bambu PLA Basic @BBL X1C.json` | `fuzzy_skin=external` | `frog_legs.stl` |

All four rows were attempted directly on the pinned 2.5.2 binary (C12) and all four slice through
`just oracle-slice` with exit 0 and a non-empty `plate_1.gcode`; none is marked "not constructible
through the pinned CLI". This differs from the only attempt on record (a 2.4.2 build, modified
process JSON, exit 239/251/254): the mechanism here is the CLI's own `--<key>=<value>` override
flags (every key named above is CLI-exposed — not `nocli` — in
`vendor/OrcaSlicer/src/libslic3r/PrintConfig.cpp`), not a doctored preset file, and `settings.json`
exported from each run confirms the override landed in the resolved config. All of it was measured
before `11-fixture-set` fixed `frog_legs`'s placement, against the vendored
`tests/data/frog_legs.obj` at its own, untranslated coordinates; the byte counts, filament lengths
and marker counts below are that measurement's record. The filament lengths and marker counts are
unaffected by the model's position on the bed (re-measured against the translated
`fixtures/frog_legs.stl`: identical `; filament used [mm]`, identical `; FEATURE: Prime tower` and
toolchange counts for `wipe-tower-multi-filament`). The byte counts are **not** position-independent
— translating the model changes how many digits each X/Y token takes, so `plate_1.gcode`'s size
shifts with it (`wipe-tower-multi-filament`: 632859 bytes against the untranslated
`tests/data/frog_legs.obj`, 661311 bytes against the translated, committed `fixtures/frog_legs.stl`)
— and none of the byte counts below are asserted to reproduce to the byte against the now-translated
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

## Resolved-configuration goldens

`port/goldens/configs/` holds one resolved configuration per row of the table above — `base-fff`,
`tree-support`, `wipe-tower-multi-filament`, `fuzzy-skin` — each named after its row and regenerated
by `just config-goldens`. Every golden is the JSON `just oracle-slice <fixture> <configuration>
--export-settings` writes on an unarmed run with a fresh, throwaway `--datadir`, with one key added:
`provenance`, the nine-item header `oracle/provenance --oracle release --syntax json` prints for the
same `<fixture>`/`<configuration>` pair. `port/merge-provenance` performs that merge — it runs
`oracle/slice` and `oracle/provenance` as subprocesses, neither of which is modified, and writes the
combined document; `just config-goldens` calls it once per row. Re-running it leaves every committed
golden byte-identical, including `fuzzy-skin`'s: that row's G-code is not byte-reproducible (above),
but its *resolved configuration* carries no entropy, and two consecutive
`just oracle-slice frog_legs.stl fuzzy-skin --export-settings` runs were measured to produce an
identical `settings.json` before this was relied on here.

Every row's `Fixtures` cell above names `frog_legs.stl`, so every golden is sliced against that
fixture, not `3DBenchy.stl`: `04-oracle-slice` chose `frog_legs.stl` for all four rows (`d420266`),
before the fixture set existed, and `11-fixture-set` left that choice as it found it — its own task
restricted it to correcting the column's *spelling* (the bare stem to the literal basename), not
reassigning rows to different frozen fixtures. This task's own task.md names `3DBenchy.stl` as the
base-FFF row's fixture, which this table does not; that line is stale, and this task follows the
table on `main`, which `04-oracle-slice` owns and this task's own "Done when" leaves undisturbed,
over the task description's example.

**The credential fields the throwaway datadir keeps empty.** The exported JSON carries all seven
`printhost_*` fields; the pinned CLI never offers a way to select `--datadir`, so none of them is
ever populated from a real credential store. The five string-valued ones export as `""`
(`printhost_apikey`, `printhost_port`, `printhost_cafile`, `printhost_user`, `printhost_password`);
the `coBool` `printhost_ssl_ignore_revoke` exports its default `"0"` and the `coEnum`
`printhost_authorization_type` its default `"key"` — never `""`, because neither is a string field.

**The measured relationship between the exported JSON and the G-code's `CONFIG_BLOCK`.** Measured by
`port/config-block-diff`, a standalone script beside `port/inventory` (not under `port/goldens/`,
which `oracle/provenance --check` covers in full and which has no oracle or fixture of its own to
carry a header for), against `port/goldens/configs/base-fff.json` and the `plate_1.gcode` from the
same `just oracle-slice frog_legs.stl base-fff --export-settings` run. `plate_1.gcode` is not
committed — it is git-ignored scratch output, like every `oracle/slice` run's — so reproduce it
with `./port/merge-provenance frog_legs.stl base-fff /tmp/base-fff.json --out <a directory>`,
which prints that directory's `plate_1.gcode` path alongside the golden it writes (`just
config-goldens` prints the same "run output: ..." line for all four rows, from a fresh directory
each time):

```
$ just config-block-diff port/goldens/configs/base-fff.json <that run's plate_1.gcode>
json only (52):
  deretract_speed_extruder_change
  farthest_point_timelapse
  filament_deretraction_speed
  filament_extruder_compatibility
  filament_flush_temp_fast
  filament_ironing_flow
  filament_ironing_inset
  filament_ironing_spacing
  filament_ironing_speed
  filament_retract_after_wipe
  filament_retract_before_wipe
  filament_retract_length_toolchange
  filament_retract_lift_above
  filament_retract_lift_below
  filament_retract_lift_enforce
  filament_retract_restart_extra
  filament_retract_restart_extra_toolchange
  filament_retract_when_changing_layer
  filament_retraction_length
  filament_retraction_minimum_travel
  filament_retraction_speed
  filament_wipe
  filament_wipe_distance
  filament_z_hop
  filament_z_hop_types
  flush_multiplier_fast
  from
  imex_carriage_margin
  imex_firmware_managed_zones
  imex_gantry_count
  imex_head_filament_map
  imex_mode_active_tools
  imex_mode_gcodes
  imex_mode_names
  imex_nozzle_clearance_x
  imex_nozzle_clearance_y
  imex_parallel_mode
  imex_tool_layout
  imex_tools_per_gantry
  imex_viz_theme
  is_imex
  name
  prime_volume_mode
  print_host
  print_host_webui
  printhost_apikey
  printhost_cafile
  printhost_password
  printhost_port
  printhost_user
  support_fast_purge_mode
  version
gcode only (2):
  first_layer_bed_temperature
  first_layer_temperature
```

The 52 JSON-only keys are `append_full_config`'s own ban list
(`vendor/OrcaSlicer/src/libslic3r/GCode.cpp:8328-8385`, which explicitly excludes
`printhost_apikey`, `printhost_cafile`, `printhost_user`, `printhost_password` and `printhost_port`
— `print_host`/`print_host_webui` besides — from every G-code it writes) plus the keys
`cfg.option(key)->is_nil()` skips for this configuration (`GCode.cpp:8388-8389`); the comparison
excludes the golden's own `provenance` key, which is this task's bookkeeping, not part of
libslic3r's resolved configuration, and was never a candidate for either set. `printhost_authorization_type`
and `printhost_ssl_ignore_revoke` are not in this list: neither is a string field (above), and both
are written into the config block like any other key (`; printhost_authorization_type = key`,
`; printhost_ssl_ignore_revoke = 0`).

The two G-code-only keys are `first_layer_bed_temperature` and `first_layer_temperature`, written by
`vendor/OrcaSlicer/src/libslic3r/GCode.cpp:4325-4335` directly after `append_full_config` returns —
not through `cfg.keys()`, and so never reachable by `--export-settings` — using
`compat_first_layer_bed_temp`/`compat_first_layer_nozzle_temp`, values the oracle derives at G-code
emission time, not part of the resolved configuration `--export-settings` dumps.

**Emission is not resolution, and stays in scope.** The deduction above only removes preset
resolution (preset files, the `inherits` chain, preset-bundle resolution) from the port's work (R11):
the port still writes the G-code `HEADER_BLOCK` and `CONFIG_BLOCK` from the resolved input, selecting
and formatting the same keys the oracle writes (`GCode.cpp:4255-4335`, the span from
`HEADER_BLOCK_START` through `CONFIG_BLOCK_END` that this section measures), including deriving the
two G-code-only keys above. The print-time estimator whose line the header carries
(`GCode/GCodeProcessor.cpp:1310`) is in scope too, because L1 compares both.
