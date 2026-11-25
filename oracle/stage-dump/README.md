# oracle/stage-dump

The locally built libslic3r's per-stage dumper (R5, `intent/libslic3r-rust-port/plan.md` step 8).
A C++ program, `stage_dump`, linked against the `libslic3r` task `13-build-libslic3r` builds on
this platform, plus the one `just` recipe that builds and runs it: `just oracle-dump`.

## What it proves and what it does not

The program drives the vendored library through the same sequence
`vendor/OrcaSlicer/tests/fff_print/test_helpers.hpp` uses (build a model, apply a config,
validate, process, export G-code), registers a hook with `Print::set_slicing_pipeline_hook_fn`,
and arms it with the `slicing_pipeline_plugin` config key. Registering and arming the hook is
proven harmless by an identity check: the same build, run twice over the same fixture,
configuration and configuration-driver arguments -- once armed, once with `--unarmed` -- must
export G-code that is identical at L1 (`crates/oracle-compare`'s comparator; L1 and not byte for
byte, because arming adds a `slicing_pipeline_plugin` line to the exported config block). `just
oracle-dump` runs both and fails the check with `just compare ... --require L1` if they disagree.

Task `25-stage-dumper` built the program, the hook, the identity check and the configuration
driver, and gave `dump_seams.cpp` its seam entry point, with no per-seam state read yet -- every
dump it wrote carried only the provenance header, the seam's name and the object's identity.
This task, `26-stage-dump-seams`, fills `dump_seams.cpp`'s body with the state each seam leaves
behind (`Layer::lslices`, `LayerRegion::slices`, `raw_slices`, `perimeters`, `thin_fills`,
`fill_surfaces`, `fills`, `PrintObject::support_layers()`, `object_skirt()`), per R5 and this same
plan step. See "Dump format" below for the record shape every dump now carries, and the next
section down from it for a precise list of what this task read.

**The provenance header's "Built oracle flags" field reads `unrecorded` on every dump this task
writes.** `oracle/provenance --oracle built` (task `05-oracle-provenance`, not a file this task
changes) locates a `docs/ORACLE.md` heading matching `Built[- ]oracle flags`; that heading does
not exist yet (the closest is `## The locally built oracle`, prose rather than the committed
flag list), so every `--oracle built` call -- this task is its first caller -- falls back to
`unrecorded` and prints a warning naming the missing section. R6 and R7 want the built oracle's
exact flags, including `-ffp-contract=off`, on record; `oracle/stage-dump/CMakeLists.txt` is
where those flags actually live, but writing them into `docs/ORACLE.md` is outside this task's
own file list. Not fixed here; see the pull request body.

## Dump format

A dump is a line-oriented text file: the provenance header (`oracle/provenance --syntax gcode`,
unchanged from task `25-stage-dumper`), then `; seam: <name>` and `; object: <print object index
or "print">` (also unchanged), then the body this task adds. Every body line is self-describing
-- a reader finds a value by its key, never by counting lines or knowing the writing order.

**Which seams write what.** R5 (`intent/libslic3r-rust-port/spec.md`) is the one place the eleven
seams and the reason each fires are enumerated; this file does not restate it. What follows is
only the field list and the record shape, not the reasons.

The nine per-object seams (`posSlice` through `posSimplifyPath`) all call the same
`dump_object_state`, documented at the top of `dump_seams.cpp`: each of these nine steps only
ever adds to or refines one growing body of state, so writing the same full field set at every
one of them is accurate at each point in the pipeline -- a field a later step has not reached yet
is simply still empty, not fabricated. That state is, per object:

- per layer (`PrintObject::layers()`, 0-based, in that order): `print_z`, `slice_z`, `height`
  (each a `coordf_t` i.e. a C++ `double`), and `lslices` (`ExPolygons`).
- per region of that layer (`Layer::regions()`, 0-based, in that order): `slices`
  (`SurfaceCollection`), `raw_slices` (`ExPolygons`), `thin_fills` (`ExtrusionEntityCollection`),
  `fill_surfaces` (`SurfaceCollection`), `perimeters` (`ExtrusionEntityCollection`), `fills`
  (`ExtrusionEntityCollection`).
- per support layer (`PrintObject::support_layers()`, 0-based, in that order, a separate index
  from the layer index above): `support_islands` (`ExPolygons`), `support_fills`
  (`ExtrusionEntityCollection`).

The two print-wide seams each read a named, narrower subset of their own field:

- `psWipeTower`: `Print::wipe_tower_data()`'s own scalar geometry fields only -- `depth`, `width`,
  `height`, `brim_width` (each a C++ `float`), `bbx` (a `BoundingBoxf`: `defined`, `min`, `max`),
  `rib_offset` (a `Vec2f`) and `number_of_toolchanges` (an `int`). `priming`, `tool_changes` and
  `final_purge` are not read: they carry already-emitted G-code in unscaled millimetres, not
  `coord_t` geometry, and reading them would be a second, incompatible serialisation format
  inside the same file. See "Decisions" below.
- `psSkirtBrim`: `Print::skirt()`, `Print::skirt_brim_groups()` (each group's own `skirt` and
  each of its `brims`' own `brim`, in their stored order) and `PrintObject::object_skirt()` for
  every print object (`Print::objects()` order) -- one `object_skirt` record per object always,
  even when its own collection is empty.

**Record shapes.** Every record's first token is `field=<name>`, then `layer=<L>` and
`region=<R>`, where `L`/`R` is a 0-based index into the collection named above, or the sentinel
`-` where that axis does not apply (every print-wide field; `region` for a field that has no
per-region structure, i.e. `lslices`, `support_islands`, `support_fills`, and the scalar
per-layer fields). A support-layer field carries its own `support_layer=<S>` key alongside
`layer=-`, since it indexes a different collection (`support_layers()`) than `layer` does
(`layers()`) and the two must never be read as the same axis. `psSkirtBrim`'s own
`skirt_brim_group_*` and `object_skirt` records carry an additional `group=<g>` /
`brim=<b> filament_id=<f>` / `object=<o>` key for the same reason -- so a record names everything
needed to place it without depending on which dump or which line it came from.

- A scalar field (`print_z`, `slice_z`, `height`, each `wipe_tower_*` field): one line,
  `field=... layer=<L|-> value=<v>`. A `coordf_t`/`double` or `float` value is written with C's
  `%a` conversion -- a hexadecimal floating-point literal, exact for the value held (a `float` is
  promoted to `double` for the formatter and never narrowed back). This is the one floating-point
  form this program ever writes; a `coord_t` coordinate is never written this way, and no field
  this program writes is ever formatted as a plain decimal float. `wipe_tower_number_of_toolchanges`
  is an `int`, written as a plain decimal integer, same as a coordinate. `wipe_tower_bbx` and
  `wipe_tower_rib_offset` are two-line exceptions with `min=`/`max=`/`value=` holding a
  comma-separated pair of `%a` values, since a bounding box and a 2D offset are each two scalars,
  not one.
- An `ExPolygons` field (`lslices`, `raw_slices`, `support_islands`): one summary line,
  `... count=<n>` (the polygon count), then, for each polygon in its stored order (never
  sorted), one line per ring -- the contour, then each hole in `ExPolygon::holes`' own order --
  `... poly=<p> ring=contour|hole:<h> n=<k> pts=<k space-separated "x,y" pairs>`. `x`/`y` are
  `coord_t` (`int64_t` at this pin), written as plain decimal integers in their stored order,
  never through `unscale`/`unscale_`/`unscaled` and never through a `float` or `double`.
- A `SurfaceCollection` field (`slices`, `fill_surfaces`): the same shape as an `ExPolygons`
  field, except the per-polygon index key is `surface=<s>` (one `Surface`, not a bare polygon)
  and each of its ring lines also carries `type=<SurfaceType>` (e.g. `stTop`, `stInternal`), read
  off the enum by name. A `Surface`'s other fields (`thickness`, `thickness_layers`,
  `bridge_angle`, `extra_perimeters`) are not written; see "Decisions".
- An `ExtrusionEntityCollection` field (`thin_fills`, `perimeters`, `fills`, `support_fills`,
  `skirt`, `skirt_brim_group_skirt`, `skirt_brim_group_brim`, `object_skirt`): one summary line,
  `... count=<n>` (the collection's own total leaf-path count, defined next), then one line per
  leaf path, `... path=<p> role=<ExtrusionRole> loop=<0|1> n=<k> pts=<k space-separated "x,y,z"
  triples>`. The walk that produces this descends into a nested `ExtrusionEntityCollection` in
  place (a grouping collection contributes no record of its own, only its leaves) and, for an
  `ExtrusionLoop` or an `ExtrusionMultiPath`, visits each of its own `paths` in turn -- both hold
  an `ExtrusionPaths` vector and own no geometry beyond it. `path` is a running counter over the
  whole top-level collection, assigned in this same walk order (the library's own stored order,
  never sorted), so it names a path without depending on which container held it. `role` is
  `ExtrusionEntity::role_to_string`'s own name for the leaf path's `ExtrusionRole` (e.g.
  `Inner wall`, `Bottom surface`, `Skirt`, `Brim`). `loop` is `1` when the path's nearest ancestor
  was an `ExtrusionLoop` (closed) and `0` when it was a bare `ExtrusionPath` or an
  `ExtrusionMultiPath` (open) -- a fact about the path's own shape, not a coordinate, so it is
  written as a plain `0`/`1`, never through `%a`. `x`/`y`/`z` are the path's `Polyline3`'s own
  `Point3` coordinates (three `coord_t` per point, `z` usually `0` outside Z-contouring), written
  as plain decimal integers in stored order. A path's own `width`/`height`/`mm3_per_mm`/
  `overhang_degree`/`smooth_speed` are not written; see "Decisions".

**The "did not fire" roster**, separate from any one dump file: `dump_seams.cpp` itself tracks
which of R5's eleven seams, plus `posEstimateCurledExtrusions` (reported as not fired rather than
as missing -- R5's own account of why), actually ran `dump_seam`, and prints `did not fire:
<name>` to stdout once per missing one when the program exits, provided at least one seam did
fire this run (so the `--unarmed` comparison run, which never calls `dump_seam` at all, prints
nothing). See "Decisions" for why this lives in `dump_seams.cpp` rather than `dump_main.cpp`.

## Building it directly

```
cmake -S oracle/stage-dump -B oracle/.cache/stage-dump/arm64 -G Ninja -DCMAKE_BUILD_TYPE=Release
cmake --build oracle/.cache/stage-dump/arm64 --target stage_dump
```

Both commands are also what `just oracle-dump` runs itself, so a `just oracle-dump` invocation
always rebuilds the program first (Ninja resolves an unchanged source to no work). The build
tree is `oracle/.cache/stage-dump/arm64`, gitignored, outside the vendored tree. It links the
`libslic3r` and `libslic3r_cgal` static archives `just oracle-build` (task `13-build-libslic3r`)
already produced at `vendor/OrcaSlicer/build/arm64`, plus the dependency superbuild's install
prefix at `vendor/OrcaSlicer/deps/build/arm64/OrcaSlicer_dep/usr/local` (task
`02-deps-superbuild`). If either is missing, `oracle/stage-dump/CMakeLists.txt`'s configure step
fails loudly, naming `just oracle-build` as the fix, rather than producing a confusing linker
error -- that build is a precondition this CMakeLists.txt does not work around (C7's decision).

`oracle/stage-dump/CMakeLists.txt`'s compiler defines, flags and include paths are copied from the
`libslic3r` target's own `Print.cpp` compile edge in
`vendor/OrcaSlicer/build/arm64/CMakeFiles/impl-Release.ninja` -- the edge whose macros decide the
struct layouts this program shares with the prebuilt archives, since it includes the same headers.
Only the **link-library list** comes from `fff_print_tests`' own link edge, in that same
`impl-Release.ninja`: that is the one upstream target which already links `libslic3r` with the GUI
off on this platform, so it already resolves every transitive library this program also needs.
`fff_print_tests` has no `Print.cpp` compile edge of its own -- it links the already-built
`liblibslic3r.a`, and compiles its own sources with `-fPIE` and test-only defines this program does
not carry -- so taking the defines and flags from it would be wrong. Neither edge lives in the
top-level `build.ninja`, which holds only a phony for that target; re-deriving either list means
reading `CMakeFiles/impl-Release.ninja`. The copy is deliberately not quite verbatim, and
`CMakeLists.txt` names each departure at the point it makes it: the `libz.tbd`/`libiconv.tbd`
absolute Xcode SDK paths substituted for `-lz`/`-liconv` (see its `target_link_libraries`
comment), the dropped `Catch2`/`Catch2Main` entries (this is not a test binary), and the dropped
`-Werror` machinery and precompiled-header flags. This keeps the dumper ABI-compatible with the
prebuilt archives (same preprocessor macros deciding struct layout, same C++ standard, same target
triple) without re-deriving a `find_package` graph this standalone project has no CMake cache to
run.

`oracle/stage-dump/nanosvg_impl.cpp` exists because two members of `liblibslic3r.a`
(`NSVGUtils.cpp.o`, `svg.cpp.o`) are pulled in whole for other symbols this program's call graph
needs, and their `nsvgParse`/`nsvgParseFromFile`/`nsvgDelete` calls are otherwise unresolved: the
one place upstream defines nanosvg's implementation is a GUI file
(`src/slic3r/GUI/BitmapCache.cpp`), excluded with the GUI off. This program never calls into SVG
embossing itself.

## Running it

Through the recipe, which is the only documented way to reach the binary:

```
just oracle-dump <fixture> <configuration>
just oracle-dump <fixture> <configuration> --config <path>
just oracle-dump <fixture> <configuration> --set <key>=<value> [--set <key>=<value> ...]
```

`--set` is `stage_dump`'s own default: with neither flag given, the binary itself still takes the
`--set` driver described below with zero extra overrides of its own -- just
`full_print_config()` plus `<configuration>`'s row's own `Overrides` cell, which for `base-fff`
is itself `(none)`. **The recipe does not pass the bare form through unchanged**: for that same
case (no `--config` and no `--set` anywhere in its own arguments), `just oracle-dump` itself
resolves `port/goldens/configs/<configuration>.json` and passes `--config` with that path when
the file exists, falling back to the binary's bare `--set` default only when a configuration has
no committed golden yet. Every row `docs/ORACLE.md` has today was constructed through the pinned
CLI and therefore has one (C12's accepted fallback tries the CLI first), so in practice the
recipe's bare form runs `--config` mode. This is so the dumps it commits under a configuration's
name are that configuration's actual resolved presets and overrides -- which R7 requires the
header to record -- rather than `full_print_config()` with none of them applied. Before this
recipe-level resolution existed, a bare `just oracle-dump 20mm_cube base-fff` committed dumps
whose provenance header asserted `base-fff`'s three presets (`oracle/provenance` fills that field
from the named row, unconditionally) even though the `--set` driver that actually produced them
had loaded none of those presets -- the header was not false about the pinned commit or the
fixture, but it was false about which presets were applied. `oracle/provenance` itself has no way
to be told which driver mode ran (it is called with only the fixture and the configuration name,
task `05-oracle-provenance`'s own interface, which this task does not change), so a configuration
with no committed golden and no `--config` run still carries that same mismatch; it is an
accepted gap for that case, not fixed here.

`<fixture>` is a frozen stem resolved against `fixtures/` the same way `oracle/slice` resolves
it (task `11-fixture-set`): a bare name with no path separator and no extension gets `.stl`
appended, `20mm_cube` meaning `fixtures/20mm_cube.stl`. `<configuration>` names a row of
`docs/ORACLE.md`'s configuration table.

**`--config <path>`** applies the oracle's resolved configuration -- the JSON an unarmed
`--export-settings` run writes, which task `12-config-goldens` commits under
`port/goldens/configs/` -- over `DynamicPrintConfig::full_print_config()`. This program sets no
key beyond the arming key, but the library's own JSON loader is not a no-op on everything it
reads: `ConfigBase::load_from_json` calls `handle_legacy_composite()`
(`vendor/OrcaSlicer/src/libslic3r/Config.cpp:1152-1153`), which rewrites the `thumbnails` string
by re-joining its comma-separated list with `", "` (`PrintConfig.cpp:10066-10073`), so a golden
whose `thumbnails` value has no space after its comma (the plain `coString` default,
`PrintConfig.cpp:8603`) comes back out of a `--config` run with one added. Measured against
`port/goldens/configs/base-fff.json`: of the keys the two share, only `thumbnails` and
`slicing_pipeline_plugin` (the arming key, which R4 already allows to differ for an armed run)
differ -- a load-time rewrite the library performs, not this program's doing, and not corrected
here; task `27-oracle-determinism` is where that normalisation gets decided. The committed
goldens also carry a `"provenance"` object alongside their config keys; the library's own JSON
loader logs one line to stderr for that one unrecognised key (`invalid json type for
provenance`) and otherwise ignores it, which is expected and harmless -- it is not this
program's doing and not worth suppressing by hand-rolling a second JSON reader.

**`--set <key>=<value>`**, repeatable, applies `DynamicPrintConfig::full_print_config()` plus the
`Overrides` cell of `docs/ORACLE.md`'s row named by `<configuration>` (its own `;`-separated
`key=value` pairs), plus every `--set` given on the command line, in that order -- a command-line
`--set` always wins over the row's own value for the same key. This mode resolves no preset and
no `inherits` chain (R11): it never reads the row's `Machine`, `Process` or `Filament(s)`
columns, which is how it constructs a configuration the pinned CLI cannot (C12) -- tree support,
the wipe tower with multi-filament, fuzzy skin.

`print.validate()`'s relative-extruder-addressing check (`Print.cpp:2794-2800`) is gated on
`!is_BBL_printer()`, a `Print`-level flag the CLI itself sets from outside the config
(`src/OrcaSlicer.cpp:7011-7026`) -- never a config key, so setting it touches neither mode's "the
configuration it is given". `--config` mode derives it from the loaded config's own
`printer_model`, the field the CLI reads. `--set` mode never learns a `printer_model` at all
(R11: it resolves no preset, including no `Machine` column), so this program sets it `true`
directly for that mode: every row of `docs/ORACLE.md`'s configuration table builds a Bambu
printer's print, and `--set` mode constructs the configurations that table's `Machine` column
would otherwise supply, not a different printer family.

**`--set`-mode artifacts are therefore not reproducible from their own `settings.json` alone.**
Its `printer_model` is `""` (never resolved), yet the run it was written from had
`is_BBL_printer() == true`, forced from outside the config. That flag reaches further than the
validate() check it exists for: it feeds `nozzle_diameter_for_filament()`
(`PrintRegion.cpp:60`, `Flow.cpp:246,253,265,276`), bed-type gating (`Print.cpp:2817`),
`GCodeProcessor::s_IsBBLPrinter` (`Print.cpp:5792,5842`, `GCode.cpp:2881`), and one emitted
G-code line's shape (`GCode.cpp:6701`, `; Z_HEIGHT: %g` vs `;Z:%g`). None of that is exercised by
a single-filament, single-nozzle `base-fff` run -- the L1 identity check holds regardless -- but a
reader reconstructing a `--set`-mode run from its `settings.json` and a config loader alone would
get a non-Bambu print, not the one that was actually sliced. Re-deriving `is_BBL_printer()` from
`printer_model` in `--set` mode too, and feeding it a `printer_model` as part of the mode's own
applied overrides, is left to whichever task next needs `--set` mode for a row the pinned CLI
cannot construct (none does yet, C12's fallback not having been needed so far).

`tree-support`'s own override, `support_type=tree(auto)`, is read straight out of
`docs/ORACLE.md`'s table by this program's own file I/O, so `just oracle-dump <fixture>
tree-support` with no `--set` at all applies it correctly with no shell involved. A value typed
at the command line and containing a shell metacharacter is the case that needs care: it has to
survive two rounds of shell parsing to reach the program -- your own shell's, and `just`'s own
re-substitution of its variadic `*ARGS` parameter, which is not re-quoted. Escape it so it
survives both, for example `--set 'support_type=tree\(auto\)'`.

A run writes three artifacts into its own output directory, through libslic3r's own calls:
`plate_1.gcode` (`Print::export_gcode`), `settings.json` (`ConfigBase::save_to_json`, with the
same four arguments the CLI's `--export-settings` passes:
`"project_settings"`, `"project"`, `SoftFever_VERSION`), and `plate_1.statistics.json`
(`export_gcode_statistics`, at the path `gcode_statistics_path` derives from the G-code path) --
so that task `27-oracle-determinism` can measure the same three artifacts a release-binary run
produces. When armed, it also writes one dump per fired seam into that same directory, named by
the seam's enum value (`posSlice.dump`, `psWipeTower.dump`, ...); `just oracle-dump` copies those
into the committed `port/dumps/<fixture-stem>-<configuration>/` (the path task `05-oracle-provenance`
fixes).

Every run, in every driver mode, also prints one more libslic3r error line to stderr:
`get_hrc_by_nozzle_type: parse info/nozzle_info.json got a nlohmann::detail::parse_error, ...`.
That is the library's own resources lookup failing to find a file this standalone project's
working directory has no reason to carry, not this program's doing; it does not affect any
result (exit 0, every dump written, L1 identity held on every run observed), the same way the
`invalid json type for provenance` line above does not.

## Decisions this task made where the task and spec were silent

- **The hook's output directory and the `--unarmed` flag are this program's own options, not
  named anywhere upstream.** The task names the committed destination
  (`port/dumps/<fixture-stem>-<configuration>/`) but not the run's own scratch output directory
  or how the recipe reaches the unarmed comparison run of the same binary; `--out <dir>` and
  `--unarmed` are this task's choices, documented here rather than left to be read out of the
  justfile alone.
- **`is_BBL_printer()` is set immediately after `print.apply()` and before `print.validate()`**,
  described above: from the applied config's `printer_model` in `--config` mode, exactly as the
  CLI does, and `true` directly in `--set` mode, where no `printer_model` is ever resolved. The
  task's own sequence, read from `test_helpers.hpp`/`.cpp`, omits this step because those helpers
  never hit the check it exists for; running the program against a real Bambu configuration
  surfaced it. It is a `Print`-level flag, not a config key, so neither mode's "changes nothing
  but the arming key" / "sets no key the configuration does not name" is affected by setting it.
- **The dump's filename is the seam name alone, with no object-index suffix.** Every committed
  fixture holds exactly one object (task `11-fixture-set`'s own invariant), so the seam name is
  already unique within a run; the object's identity still lives in the dump's body ("object: 0"
  or "object: print"), not in its name. Because that invariant is assumed rather than checked,
  `dump_seam` refuses to overwrite a path that already exists in the run's output directory and
  exits 1 naming it, so a future multi-object fixture fails loudly instead of silently losing a
  dump to the next one that fires the same seam.
- **`just oracle-dump`'s bare form (no `--config`, no `--set`) resolves a configuration's own
  committed golden and passes it as `--config` when one exists**, documented above under
  "Running it". This is a recipe-level decision, not `stage_dump`'s own: the binary's bare
  default stays `--set` either way, and anyone invoking it directly (bypassing the recipe) still
  gets that. The task names the two driver modes but not which one the bare recipe invocation
  reaches; this choice is what keeps the provenance header's `preset-set` field (R7) describing
  presets the run actually applied for the configurations that have a golden.

Task `26-stage-dump-seams`'s own decisions, filling `dump_seams.cpp`'s body:

- **One shared `dump_object_state` for all nine per-object seams**, rather than nine functions
  each reading a different subset of the growing state. Documented at the top of
  `dump_seams.cpp` and under "Dump format" above: every one of these nine steps only ever adds
  to or refines the same state, so a field a later step has not reached yet is simply still
  empty at an earlier seam's dump -- an accurate snapshot, not a fabricated one. Each seam still
  gets its own one-line wrapper function, named in its own comment, so "a reviewer greps the
  file for each seam's enumerator name and finds one" (this task's own "Done when") holds for a
  reviewer who has not read this decision.
- **The "did not fire" roster lives in `dump_seams.cpp`, not `dump_main.cpp` or the justfile.**
  This task's own "Done when" requires `just oracle-dump 20mm_cube base-fff` to name every seam
  that did not fire, but `dump_main.cpp` and `CMakeLists.txt` are frozen by that same "Done
  when" list, and the task's own file list does not include the justfile. `dump_seams.cpp`
  tracks which seam names `dump_seam` was actually called with in a file-local `std::set`, and
  registers an `std::atexit` handler the first time `dump_seam` runs (not unconditionally): `just
  oracle-dump` runs this same binary a second time with `--unarmed` for the L1 identity check,
  and that run never calls `dump_seam` at all, so an unconditional roster would print "every seam
  did not fire" for a run where arming, and so every seam, was deliberately off. Measured:
  `./oracle/.cache/stage-dump/arm64/stage_dump 20mm_cube base-fff --out <dir>` (armed) prints
  `dumps: 10` then `did not fire: posEstimateCurledExtrusions` and `did not fire: posContouring`
  -- the fixture is not a belt printer, so Z-contouring never runs, and
  `posEstimateCurledExtrusions` never fires for a sliced object for the reason R5 gives.
- **`psWipeTower` reads only `WipeTowerData`'s scalar geometry fields**, not `priming`,
  `tool_changes` or `final_purge`. Those three carry `WipeTower::ToolChangeResult`, which holds
  already-emitted G-code text in unscaled millimetres -- a different unit and a different format
  entirely from every other field this program writes, and reading it would mean inventing a
  second serialisation inside the same file for state R5 does not name a reason to read. The
  task's own account of which fields are "genuinely floating-point in the library" names only
  "the wipe tower's depth" as an example from this seam, which this reading matches.
  `wipe_tower_data()` is called with its default `filaments_cnt=0`; read before writing this
  call, `Print::wipe_tower_data` (`Print.cpp`) takes that default argument's early-return branch
  whenever `is_step_done(psWipeTower)` is already true, which it always is by the time this hook
  fires (`Print::process` calls `set_done(psWipeTower)` immediately before firing the hook), so
  this read never takes the branch that estimates and mutates `m_wipe_tower_data` -- the one
  path that could have put the L1 identity check at risk.
- **`object_skirt()` was found to be permanently empty at this pin, for every configuration.**
  The task's own account of this field (and this task's "Done when") describes
  `PrintObject::m_skirt` as filled by skirt generation "when `skirt_type` is `stPerObject`"
  (`Brim.cpp:63`). Measured against the vendored tree at the pinned commit: every reference to
  `m_skirt` (a plain, non-word-boundary-sensitive search) across the whole of
  `src/libslic3r/` is either a read through `object_skirt()`/`skirt()` or a `.clear()` call
  (`Print.cpp:3704`, `Print.cpp:4222`, and the constructor-site clears each group's own skirt
  before rebuilding it); the one place a skirt collection is actually appended to
  (`Print.cpp:4485`, inside the function that builds `skirt_brim_groups()`) appends into
  `Print::m_skirt` -- the print-wide member sharing the name `object_skirt()`'s own class
  declares at `Print.hpp:716` -- never into any `PrintObject::m_skirt`. Confirmed empirically,
  not just by reading: `stage_dump 20mm_cube base-fff --set skirt_loops=1 --set
  skirt_type=perobject` (and separately `--set brim_width=3 --set brim_type=outer_only`, and
  `--set skirt_loops=2` with the default combined skirt type) each produced real, non-empty
  `skirt` and `skirt_brim_group_skirt`/`skirt_brim_group_brim` records -- proving this program's
  own read and serialisation of those fields is correct -- while `object_skirt`'s own record
  stayed `count=0` in every one of those runs. This program still writes one `object_skirt`
  record per print object on every `psSkirtBrim` dump, exactly as the task specifies and exactly
  as it would need to if a future configuration or a future libslic3r version does populate it;
  the finding above is reported here because a dump that always reads `count=0` for a field is
  something a reviewer of this pull request should be able to see was checked, not assumed.
- **A leaf extrusion path's own `width`/`height`/`mm3_per_mm`/`overhang_degree`/`smooth_speed`
  are not written**, only its geometry (`polyline`) and its `role()`. R5 and this task's own
  "the state each seam reads" name the containing collections (`perimeters`, `fills`,
  `thin_fills`, `support_fills`) and never a per-path flow field; this phase's proof is geometry
  (R19's integer-`coord_t` comparison), and adding five more floating-point fields per path -- a
  second, much larger floating-point surface alongside the handful of per-layer and per-tower
  scalars this task's own field list already names -- is out of scope until a task that actually
  compares flow or timing needs it.
- **A `Surface`'s `thickness`/`thickness_layers`/`bridge_angle`/`extra_perimeters` are not
  written**, only its `expolygon` and `surface_type`. The same reasoning as the extrusion-path
  fields above: R5 names `LayerRegion::slices` and `fill_surfaces` as fields to read, not their
  per-surface bookkeeping, and `surface_type` is the one piece of that bookkeeping a reader
  needs to tell a `slices` ring from, say, a `stInternalSolid` one apart from a `stTop` one.
- **A `SkirtBrimGroup`'s and a `Brim`'s own `instances` (`std::vector<ObjectInstanceID>`) are not
  written**, only `filament_id` for a `Brim`. An `ObjectInstanceID` is an internal identity, not
  a coordinate or a count the oracle's own output depends on for comparison, and R5 names
  `Print::skirt_brim_groups()` as a field to read, not its own internal instance bookkeeping.
- **Hexadecimal floating-point (`%a`), not a shortest-round-tripping-decimal library call, for
  every floating-point field.** `%a` is exact for the `double` or `float` it is given and has
  been part of C since C99 and of this toolchain's `libc` for as long as this project has
  targeted it; a `std::to_chars(double)` shortest-decimal path is a newer, less uniformly
  available piece of `libc++` across toolchains, and the task's own wording allows either form
  as long as `README.md` names which: this is that naming.
