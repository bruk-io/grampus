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

This task (`25-stage-dumper`) builds the program, the hook, the identity check and the
configuration driver, and gives `dump_seams.cpp` its seam entry point -- but that entry point
reads no per-seam state yet. Every dump this task writes carries only the provenance header, the
seam's name and the object's identity. Part 2, task `26-stage-dump-seams`, fills `dump_seams.cpp`
with the state each seam leaves behind (`Layer::lslices`, `LayerRegion::slices`, `raw_slices`,
`perimeters`, `thin_fills`, `fill_surfaces`, `fills`, `PrintObject::support_layers()`,
`object_skirt()`), per R5 and this same plan step.

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

`oracle/stage-dump/CMakeLists.txt`'s compiler defines, flags and link-library list are copied
verbatim from `vendor/OrcaSlicer/build/arm64`'s own generated `build.ninja`, specifically from
the `fff_print_tests` target's compile and link edges -- the one upstream target that already
links `libslic3r` with the GUI off on this platform. This keeps the dumper ABI-compatible with
the prebuilt archives (same preprocessor macros deciding struct layout, same C++ standard, same
target triple) without re-deriving a `find_package` graph this standalone project has no CMake
cache to run. `oracle/stage-dump/nanosvg_impl.cpp` exists because two members of `liblibslic3r.a`
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

`--set` is the default: with neither flag given (the bare first form), a run still takes the
`--set` driver described below with zero extra overrides of its own -- just
`full_print_config()` plus `<configuration>`'s row's own `Overrides` cell, which for `base-fff`
is itself `(none)`.

`<fixture>` is a frozen stem resolved against `fixtures/` the same way `oracle/slice` resolves
it (task `11-fixture-set`): a bare name with no path separator and no extension gets `.stl`
appended, `20mm_cube` meaning `fixtures/20mm_cube.stl`. `<configuration>` names a row of
`docs/ORACLE.md`'s configuration table.

**`--config <path>`** applies the oracle's resolved configuration -- the JSON an unarmed
`--export-settings` run writes, which task `12-config-goldens` commits under
`port/goldens/configs/` -- over `DynamicPrintConfig::full_print_config()`, and changes nothing
but the arming key. The committed goldens carry a `"provenance"` object alongside their config
keys; the library's own JSON loader logs one line to stderr for that one unrecognised key
(`invalid json type for provenance`) and otherwise ignores it, which is expected and harmless --
it is not this program's doing and not worth suppressing by hand-rolling a second JSON reader.

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
  or "object: print"), not in its name.
