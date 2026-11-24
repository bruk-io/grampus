---
kind: task
step: 8
delivers: [R5]
depends: [05-oracle-provenance, 08-comparator-workspace, 11-fixture-set, 13-build-libslic3r]
size: 5
---
# Task: Dump the pipeline's state through the slicing hook

## What to build

This is part 1 of plan step 8, "Dump the pipeline's intermediate state", in
`intent/libslic3r-rust-port/plan.md`. It delivers R5 together with part 2,
`26-stage-dump-seams`; read R5 in `intent/libslic3r-rust-port/spec.md` before starting, because it
is where the seam list lives and no other document restates it. This task builds the program,
registers and arms and resets the hook, proves the armed build's exported G-code is identical at L1
to the unarmed build's, and gives the program its configuration driver. Part 2 fills
`dump_seams.cpp` with the state each seam leaves behind, so this task's dumps carry the provenance
header and the seam's identity and nothing more.

Files this task adds or changes:

- `oracle/stage-dump/CMakeLists.txt`
- `oracle/stage-dump/dump_main.cpp`
- `oracle/stage-dump/dump_seams.hpp`
- `oracle/stage-dump/dump_seams.cpp` — created here, holding the seam entry point and no per-seam
  state reading. It exists in this task because the program has to build and run for the L1 check
  below to mean anything; part 2 fills its body.
- `oracle/stage-dump/README.md` — how to build the program and how to run it. Part 2 adds the dump
  format section.
- `justfile` — gains `just oracle-dump <fixture> <configuration> *ARGS`, variadic so the two
  driver modes' flags reach the program.

Files this task leaves alone: `vendor/README.md`, whose "Pipeline steps" table part 2 corrects;
`.github/workflows/ci.yml`, because step 8 adds no CI line — the steps that each add one are 7, 10,
21, 23, 24, 25, 26 and 27 of the plan; and everything under `vendor/OrcaSlicer/`, which stays at
the pinned commit `f92a7c065c5718b8ef1baac1cd704df2911238f4`.

### the sequence the program drives

Mirror the sequence `vendor/OrcaSlicer/tests/fff_print/test_helpers.hpp` uses, by reading it rather
than linking it: `test_helpers` belongs to the `fff_print_tests` target, not to `libslic3r`. The
calls, from `tests/fff_print/test_helpers.cpp:253-286` (`init_print`) and `:348-357` (`gcode`):

```
Model model = Model::read_from_file(<fixture>);
for (ModelObject *mo : model.objects) { mo->ensure_on_bed(); print.auto_assign_extruders(mo); }
print.apply(model, config);
print.validate();
print.process();
print.export_gcode(<out>/plate_1.gcode, &result, nullptr);
```

Two things `init_print` does that this program must not do, because each would change the artifact
a later comparison reads:

- it calls `arrange_objects(model, InfiniteBed{}, ...)` when its `arrange` parameter is true
  (`test_helpers.cpp:276-277`, defaulted `true` at `test_helpers.hpp:88`). R24 and R10 put placement
  in the fixture and run no arranger on either side of any comparison, so this program never calls
  an arranger.
- it forces `gcode_comments` to true (`test_helpers.cpp:258`). This program applies the
  configuration it is given and sets no key the configuration does not name, except the arming key
  below. Forcing a key makes this oracle disagree with the release binary for a reason that is not
  the library's.

### the hook

Register the hook with `Print::set_slicing_pipeline_hook_fn`
(`vendor/OrcaSlicer/src/libslic3r/Print.hpp:1064`, whose `SlicingPipelineHookFn` alias is at
`:1061`). Its signature is `void(Print&, const PrintObject*, SlicingPipelineStepPlugin)`: it carries
no layer, so it fires once per step and the dumper walks the object's layers itself, and the
print-wide seams pass a null object.

It is armed only when a hook is registered *and* the `slicing_pipeline_plugin` config option is
non-empty (`Print.cpp:3308`), so before `print.apply` the program sets
`config.set_key_value("slicing_pipeline_plugin", new ConfigOptionStrings({"probe"}))`, as the
upstream test does at `tests/fff_print/test_slicing_pipeline_hook.cpp:50`, and afterwards resets
with `set_slicing_pipeline_hook_fn(nullptr)`. The setter holds a static, and upstream asserts that a
nullptr reset is legal (`test_slicing_pipeline_hook.cpp:30-36`); leaving it registered would leak
the hook into anything else linked in the same process.

The hook body calls the one entry point `dump_seams.hpp` declares and writes one file per fired seam
into the run's output directory, named after the seam and the object. **The committed location is
`port/dumps/<fixture-stem>-<configuration>/<seam>.dump`**, the path task `05-oracle-provenance`
fixes and covers with `oracle/provenance --check`, which `24-port-check-ci` passes into
`just port-check` and which `32-proof-dumps-fuzz` reads as its proof path 2. A run writes into its
output directory, and `just oracle-dump` copies the dumps from there into that committed location,
so the four tasks that name the path all name the same one. The plan's "Files that change" list
carries only `oracle/stage-dump/`'s sources and not `port/dumps/`; `plan.md` is on `main` and is not
edited from this branch, so the path is declared in `05-oracle-provenance` and written here.

In this task each file holds
the provenance header plus the seam's name and the object's identity — its index in the print's
object list, or `print` for the null object a print-wide seam passes — and nothing else.

Every dump carries the provenance header by calling `oracle/provenance` (task
`05-oracle-provenance`) rather than formatting one here. A dump is line-oriented text, so it takes
the `;` comment-block form, and both of that script's required options are passed explicitly:
`oracle/provenance --oracle built --syntax gcode`, with the fixture and the configuration. `built`
is right because the dumper is the locally built library; a run with either option missing exits
non-zero naming it, so neither can be left off.

### the L1 identity check

`just oracle-dump` runs the program twice over the same fixture and configuration — once armed as
above, once with the hook never registered and the option left empty — and compares the two exported
G-code files with the comparator task `08-comparator-workspace` builds, through
`just compare <armed> <unarmed> --require L1`. **The `--require L1` is what makes the check a
check**: that task specifies that without `--require` the command exits 0 at every level including
`none`, so a caller reads the level and not the exit code, and a recipe gated on the bare exit code
would be an identity check that can never fail. With the flag the comparator exits non-zero unless
L1 passed, and `just oracle-dump` propagates that.

L1 and not byte for byte, because arming the hook writes a `slicing_pipeline_plugin` line into the
exported config block, which is the fourth of the four line kinds the declared normalisation strips.
It is the same comparison upstream makes at `test_slicing_pipeline_hook.cpp:98-125`, and that file
is also where the normalisation itself comes from (`:83-96`).

### the configuration driver

The program takes its configuration two ways, which is C12's decision in the spec: a configuration
the pinned CLI refuses still has an oracle.

- `--config <path>`: the oracle's resolved configuration, the JSON an unarmed `--export-settings`
  run writes and task `12-config-goldens` commits under `port/goldens/configs/`. Apply it and change
  nothing but the arming key.
- `--set <key>=<value>`, repeatable, together with the overrides of the `docs/ORACLE.md` row that
  `<configuration>` names: start from `DynamicPrintConfig::full_print_config()` and apply them with
  `set_key_value` or `set_deserialize_strict`. This mode resolves no preset and no `inherits` chain —
  R11 puts preset resolution out of scope — and it is how tree support, the wipe tower with
  multi-filament and fuzzy skin get constructed when the pinned CLI will not construct them.

So that task `27-oracle-determinism` can measure the same three artifacts a release-binary run
produces, a run writes all three into its output directory, each through libslic3r's own call:

- the G-code, from the `export_gcode` above;
- the resolved configuration, through `ConfigBase::save_to_json(file, name, from, version)`
  (`src/libslic3r/Config.hpp:2877`) on the config the program applied, with the same four arguments
  the CLI's `--export-settings` passes — `"project_settings"`, `"project"` and `SoftFever_VERSION`
  (`vendor/OrcaSlicer/src/OrcaSlicer.cpp:6345-6349`). `SoftFever_VERSION` comes from the generated
  `libslic3r_version.h`, which the main project's CMake configures into its build tree
  (`src/libslic3r/CMakeLists.txt:10`), so the program includes
  `vendor/OrcaSlicer/build/arm64/src/libslic3r/libslic3r_version.h` and never hard-codes the string;
- the statistics, with `export_gcode_statistics(result, gcode_statistics_path(<gcode path>))` from
  `src/libslic3r/GCode/GCodeStatistics.hpp`, which is the call the CLI's `--export-statistics` makes
  (`src/OrcaSlicer.cpp:7231`).

`<fixture>` is a frozen stem that `oracle/slice` resolves against `fixtures/`, as task
`11-fixture-set` defines it. **This task waits on `11-fixture-set`**, and the proof runs
`just oracle-dump 20mm_cube base-fff`. The reason is the provenance header: items 8 and 9 of it are
the fixture's basename and its resolved instance transform, read from that fixture's
`vendor/README.md` record, and task `05-oracle-provenance` exits non-zero and names the fixture
when there is no record — so a run over the vendored `frog_legs.obj`, which has no record, would
write no header at all and the dumps could not pass `oracle/provenance --check`. The dependency is
acyclic: `11-fixture-set` waits on `01-oracle-fetch` and `04-oracle-slice` and not on this task, and
it brings `04-oracle-slice` with it, which is what gives `base-fff` a row in the `docs/ORACLE.md`
configuration table for the `--set` mode to read overrides from.

**The two driver modes are reached through the recipe, which is variadic**:
`just oracle-dump <fixture> <configuration> *ARGS`, forwarding `{{ARGS}}` to the built program, so
`just oracle-dump 20mm_cube base-fff --config port/goldens/configs/base-fff.json` and
`just oracle-dump 20mm_cube base-fff --set enable_support=1` both work. The binary itself lives in
the gitignored build tree described below, and the recipe is the only documented way to reach it.

### building it

`CMakeLists.txt` links the `libslic3r` task `13-build-libslic3r` builds: the dependency prefix
`vendor/OrcaSlicer/deps/build/arm64/OrcaSlicer_dep/usr/local` and the main build tree
`vendor/OrcaSlicer/build/arm64`, whose `src/libslic3r/` directory is also on the include path,
because that is where the generated `libslic3r_version.h` lives. Compile with `-ffp-contract=off`,
which C2's decision makes normative — a dumper compiled without it is not comparable to the library
it links, and that library was built with it.

Configure and build into a gitignored directory outside the vendored tree:
`oracle/.cache/stage-dump/arm64`, which the `oracle/.cache/` line in `.gitignore` covers; add that
line if task `01-oracle-fetch` has not landed it yet. Write nothing into `vendor/OrcaSlicer/`.

Task `13-build-libslic3r` is a precondition and not something to work around. C7's decision is that
a failure of that build is reported and the work behind it stops.

### what makes this a 5

A C++ program linked against a library built once on this platform: register and arm the hook, reset
it, prove the armed build's G-code is identical at L1 to the unarmed build's, and take a
configuration either as resolved JSON or as in-code overrides so step 6 can reach the rows the CLI
refuses.

It is not split further. The arming, the reset and the L1 identity check are one claim — that a
dumping run's G-code is still the oracle's — and the two driver modes are what the program takes as
input for that claim to be worth making; a program split from its own harmlessness proof could not
be reviewed. The step's serialisation is already split out as part 2.

## Done when

- `oracle/stage-dump/` holds `CMakeLists.txt`, `dump_main.cpp`, `dump_seams.hpp`, `dump_seams.cpp`
  and `README.md`, and `just oracle-dump 20mm_cube base-fff` exits 0.
- That run writes one file per fired seam and prints their count, and copies them to
  `port/dumps/20mm_cube-base-fff/<seam>.dump`.
- `grep -c '^; '` on one of those files is non-zero, and
  `oracle/provenance --check 'port/dumps/*/*.dump'` exits 0 and reports a non-zero count.
- The recipe calls `just compare <armed> <unarmed> --require L1`, prints `L1`, and exits non-zero
  when the comparator reports any weaker level; `grep -n 'require' justfile` shows the flag on that
  call.
- `grep -n 'set_slicing_pipeline_hook_fn' oracle/stage-dump/dump_main.cpp` shows both the
  registration and the `nullptr` reset, and the program sets `slicing_pipeline_plugin` non-empty
  before `print.apply`.
- `oracle/stage-dump/*.cpp` contains no call to an arranger and no assignment of `gcode_comments`.
- A run in each driver mode, reached through the variadic recipe —
  `just oracle-dump 20mm_cube base-fff --config port/goldens/configs/base-fff.json` and
  `just oracle-dump 20mm_cube base-fff --set <key>=<value>` — writes the G-code, the
  resolved-configuration JSON and the statistics JSON into its output directory.
- `-ffp-contract=off` appears in `oracle/stage-dump/CMakeLists.txt`.
- `dump_seams.cpp` holds the seam entry point `dump_seams.hpp` declares and reads no per-seam state;
  its header comment says part 2 fills it.
- The branch changes nothing under `vendor/OrcaSlicer/`, leaves `vendor/README.md` and
  `.github/workflows/ci.yml` untouched, and adds no build output to the index:
  `git status --porcelain vendor/OrcaSlicer` prints nothing.
