# asdlc from its repository, so this runs the same in CI and in a terminal with no plugin
asdlc := "uvx --from git+https://github.com/bruk-io/asdlc asdlc"

# R17, task 29-ffi-bridge-core: crates.io names known today for a Rust reimplementation of one
# of the six libraries `crates/slic3r-ffi` binds through FFI instead of depending on (C4's
# decision; there is no allowlist). This is a deny-list of names known today, not a proof of
# absence -- a later task should extend it when it finds another, not trust that the absence of
# a name here means no substitute exists. One or more names per library, reviewed in this task's
# pull request: Clipper2 -- geo-clipper, solidmesh-clipper, clipper2 (crates.io's own name for a
# Rust API over stock, unpatched Clipper2 -- not the vendored copy's two offsetter patches at
# Clipper2Lib/src/clipper.offset.cpp:301,377, which is why this is a binding at all),
# i_overlay, geo-booleanop; admesh -- meshrepair; qhull -- qhull_rayon, chull, qhull; libnest2d
# -- nfp; Boost.Polygon's Voronoi builder -- voronoi, spade, voronator; NLopt -- cobyla, bobyqa,
# slsqp, nlopt.
substitute_names := "geo-clipper|solidmesh-clipper|clipper2|i_overlay|geo-booleanop|meshrepair|qhull_rayon|chull|qhull|nfp|voronoi|spade|voronator|cobyla|bobyqa|slsqp|nlopt"

# everything a pull request has to pass
check:
    {{asdlc}} document check

# the code checks that need neither the oracle binary nor a built libslic3r (R23): every one
# of these reads only the vendored source and the repository's own files, so a task that
# touches no code still passes with this recipe alone, and it never waits on a build. Two more
# lines join it later, each added by the task that makes it pass: Step 17's substitute and
# -ffp-contract=off drift checks and Step 18's mock check (task 24-port-check-ci does not add
# either). `just audit-check` and the perf-backlog whole-file diff are deliberately not here --
# see their own recipes' docstrings -- and neither is `just port-status` without
# `--per-commit`, the completion gate that runs in the macOS job instead.
port-check:
    just port-status --per-commit
    just inventory-check
    just ownership-check
    ./oracle/provenance --check port/goldens/ 'port/dumps/*/*.dump' port/reports/determinism.md port/reports/oracle-parity.md
    cargo fmt --check
    # R17, task 29-ffi-bridge-core: -ffp-contract=off cannot drift between the two compilations
    # the per-unit and per-stage proofs compare against -- the oracle libslic3r build and this
    # crate's own build.rs. Anchored to the thing that actually reaches each compiler, not a bare
    # substring: oracle/build-libslic3r's own comment above its CXX_FLAGS line (explaining why
    # the flag is there) contains the string "-ffp-contract=off" too, and build.rs's module doc
    # mentions it in prose as well, so a bare `grep -q` would still pass with the flag deleted
    # from the command line in either file as long as a prose mention survives -- this was found
    # and fixed on the oracle side during this task and, left as a bare substring, remained a
    # proven false negative on the build.rs side (see this task's pull request body). Still not
    # a CMake/syn parse -- the flag is a literal command-line token in both places (the
    # CXX_FLAGS variable, build.rs's cxx_build::Build::flag() call), so this is checking for its
    # presence at the call site, not its binding.
    grep -qE '^CXX_FLAGS = ".*-ffp-contract=off.*"' oracle/build-libslic3r || { echo "port-check: -ffp-contract=off is missing from oracle/build-libslic3r's CXX_FLAGS assignment" >&2; exit 1; }
    grep -qE '^\s*build\.flag\("-ffp-contract=off"\)' crates/slic3r-ffi/build.rs || { echo "port-check: -ffp-contract=off is missing from crates/slic3r-ffi/build.rs's build.flag() call" >&2; exit 1; }
    # R17, task 29-ffi-bridge-core: no crate manifest may depend on a named Rust reimplementation
    # of the six libraries this intent binds through FFI instead -- see substitute_names above.
    # Catches every realistic way a Cargo.toml spells a dependency on one of these names: the
    # inline form (`name = "..."` or `name = { version = ... }`), indented under a `[dependencies]`
    # table (TOML allows leading whitespace on a key); the dotted workspace-inherited form
    # (`name.workspace = true`); a quoted key (`"name" = "..."`); the dependency's own table
    # header (`[dependencies.name]`, `[dev-dependencies.name]`, `[build-dependencies.name]`),
    # indented, with internal spacing, or nested under a target table
    # (`[target.'cfg(unix)'.dependencies.name]`); and the renamed form
    # (`alias = { package = "name", version = "..." }`) -- a bare `^name[[:space:]]*=` anchor
    # alone misses all but the first. Still anchored per form, not a bare substring, so it does
    # not fire on `slic3r-clipperutils`/`slic3r-clipperzutils`, this workspace's own ported
    # crates, or any other crate whose name merely contains one of these as a substring (e.g.
    # `qhull_rayon` naming itself, not `qhull`).
    ! grep -RnE '^[[:space:]]*\[[[:space:]]*.*dependencies\.({{substitute_names}})[[:space:]]*\]|^[[:space:]]*"?({{substitute_names}})"?(\.workspace)?[[:space:]]*=|package[[:space:]]*=[[:space:]]*"({{substitute_names}})"' crates/*/Cargo.toml

# the guard: this branch changes only what its documents on main allow
guard *branch:
    {{asdlc}} branch check {{branch}}

# fetch and verify the pinned oracle release binary (R1)
oracle-fetch:
    ./oracle/fetch-release
# build the vendored OrcaSlicer tree on this platform
oracle-build:
    ./oracle/build-libslic3r
# regenerate port/manifest.tsv's file set and per-file derived columns, and port/cut.tsv, from
# the vendored tree
inventory:
    ./port/inventory
# variadic (not two-parameter) because `just` rejects a surplus argument to a fixed-arity recipe,
# and callers across steps 5, 6, 9, 10 and 25-27 each need their own extra flags
compare a b *ARGS:
    cargo run --quiet --release -p oracle-compare -- {{a}} {{b}} {{ARGS}}

# run the pinned oracle headlessly on a named row of docs/ORACLE.md's configuration table (R2);
# forwards its arguments to oracle/slice unchanged and adds no flag of its own
oracle-slice *ARGS:
    ./oracle/slice {{ARGS}}

# export, place and freeze the committed fixture set from their vendored models (R24, R25);
# with no NAMES, (re)produces all three
fixture-export *NAMES:
    ./oracle/export-fixture {{NAMES}}
# check a run directory's reported object placement against its fixture's vendor/README.md
# record (R24); named here so 41-compare-benchy and 42-compare-fixtures do not each invent a
# spelling for the same call
assert-placement fixture run_dir:
    ./oracle/assert-placement {{fixture}} {{run_dir}}

# regenerate port/manifest.tsv into memory (task 14-manifest-columns); never reads the six
# judgement columns or status, never writes a file, and never compares port/cut.tsv
# fail, with a diff, when a derived manifest column has drifted from the vendored tree
inventory-check:
    ./port/inventory --check
# regenerate every resolved-configuration golden under port/goldens/configs/ (R11): one per row
# of docs/ORACLE.md's configuration table, each sliced through oracle-slice and carrying its
# provenance header via port/merge-provenance; a row added there must be added here too.
# oracle/.cache/config-goldens/ accumulates one scratch directory per invocation (git-ignored,
# left behind for inspection) and nothing prunes it; safe to delete by hand.
config-goldens:
    ./port/merge-provenance frog_legs.stl base-fff port/goldens/configs/base-fff.json
    ./port/merge-provenance frog_legs.stl tree-support port/goldens/configs/tree-support.json
    ./port/merge-provenance frog_legs.stl wipe-tower-multi-filament port/goldens/configs/wipe-tower-multi-filament.json
    ./port/merge-provenance frog_legs.stl fuzzy-skin port/goldens/configs/fuzzy-skin.json

# print the keys a resolved-configuration golden's JSON carries that <gcode>'s G-code
# CONFIG_BLOCK does not, and the keys the config block carries that the JSON does not (R11)
config-block-diff golden gcode:
    ./port/config-block-diff {{golden}} {{gcode}}

# check every ported module's PORT STATUS trailer and port markers (R14, R15, R16 part);
# variadic so `just port-status --per-commit` and the later `--pr-body <file>` (36-pr-body-check)
# both reach it without this recipe being rewritten (task 21-port-status-markers)
port-status *args:
    ./port/port-status {{args}}
# regenerate port/reports/order-containers.md and port/reports/order-comparators.md (R18, step
# 13 part 1, task 17-order-audits): every unordered container that is iterated rather than only
# looked up, and every comparator that is not a total order, over the manifest's in-scope rows,
# read at run time. Not a check a port task passes (just audit-check, 18-divergences-regeneration,
# is); this recipe only regenerates the two reports.
audit-order:
    ./port/audit-order
# batch port/manifest.tsv and port/cut.tsv's rows into one of the four plan-step cut files
# (R8, R9; task 15-task-cuts): ownership, foundation, phase-a or phase-b. Variadic so a caller
# can also pass `--out <dir>` to redirect the write away from the committed port/ tree.
task-cuts phase *ARGS:
    ./port/cut-tasks {{phase}} {{ARGS}}

# regenerate port/ownership.tsv's field list from the vendored headers (R30, task
# 16-ownership-spike): one row per raw-pointer field, and per field that is a container of raw
# pointers, declared in a header port/manifest.tsv marks in scope. Rewrites the committed file --
# for a person regenerating deliberately.
ownership:
    ./port/ownership
# the drift-detecting variant: regenerates into a scratch directory and fails, with a diff and
# every invalid row, when port/ownership.tsv disagrees with the headers at the pin. Prints the
# row count of port/ownership.tsv on success -- the plan's recorded proof line for R30.
ownership-check:
    ./port/ownership --check
# the one command that rewrites every generated report (task 18-divergences-regeneration):
# today, just audit-order; 23-perf-backlog adds the perf-backlog regeneration to this same seam
regenerate:
    ./port/regenerate
# the drift-detecting variant of audit-order: regenerates order-containers.md,
# order-comparators.md and divergences.md into a scratch directory and fails when a committed
# one differs byte for byte (R18, task 18-divergences-regeneration). Not a check a port task
# passes -- see port/audit-order's module docstring and this task's own task.md -- so this
# recipe is deliberately in neither `.asdlc/checks` nor `just port-check`; step 14's
# regeneration pass (23-perf-backlog) is what runs it, on main.
audit-check:
    ./port/audit-order --check

# the drift-detecting variant of the perf-backlog half of regenerate: regenerates
# port/reports/perf-backlog.md in memory and fails, with a diff, when it differs from the
# committed file (R15, task 23-perf-backlog). Not a check a port task passes -- see
# port/regenerate's module docstring and this task's own task.md -- so this recipe is
# deliberately in neither `.asdlc/checks` nor `just port-check`; step 14's regeneration pass is
# what runs it, on main, alongside `just audit-check`.
perf-check:
    ./port/regenerate --check

# create crates/<crate>/{Cargo.toml,src/lib.rs} for every crate row of port/cut.tsv, plus the
# shared slic3r-proof skeleton every generated Cargo.toml dev-depends on (task 34-foundation-spike,
# plan step 20); writes only what is missing, changes no file that already exists
crate-skeletons:
    ./port/crate-skeletons
# build (if needed) and run the per-stage dumper (R5, plan step 8) over <fixture> <configuration>;
# *ARGS is variadic so the two driver modes' flags reach the program --
# `just oracle-dump 20mm_cube base-fff --config port/goldens/configs/base-fff.json` and
# `just oracle-dump 20mm_cube base-fff --set enable_support=1` both work (task 25-stage-dumper).
# Builds oracle/stage-dump/ into the gitignored oracle/.cache/stage-dump/arm64 (never into
# vendor/OrcaSlicer/), runs the program twice over the same fixture, configuration and ARGS --
# once armed (registers and arms the slicing-pipeline hook, writing one dump per fired seam),
# once with --unarmed (hook never registered, option left empty) -- and proves the two runs'
# exported G-code agree at L1 through `just compare ... --require L1`: the flag is what makes
# the check a check, since without --require the comparator exits 0 at every level including
# none. Copies the armed run's dumps to the committed port/dumps/<fixture-stem>-<configuration>/
# (05-oracle-provenance's path) and prints their count.
#
# When ARGS names neither --config nor --set, this recipe (not stage_dump's own CLI default,
# which stays --set per oracle/stage-dump/README.md) resolves <configuration>'s own committed
# golden, port/goldens/configs/<configuration>.json, and passes it as --config when that file
# exists -- so the dumps committed under a configuration's name are actually that
# configuration's resolved presets and overrides, not full_print_config() with none of them
# applied (--set mode resolves no preset at all, R11). C12's accepted fallback tries the pinned
# CLI first and constructs in code only where it cannot; every row this table has today
# constructed through the pinned CLI and therefore has a golden, so this is the common case.
# Falls back to the bare --set driver only for a configuration with no golden yet.
oracle-dump fixture configuration *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail
    build_dir=oracle/.cache/stage-dump/arm64
    cmake -S oracle/stage-dump -B "$build_dir" -G Ninja -DCMAKE_BUILD_TYPE=Release >/dev/null
    cmake --build "$build_dir" --target stage_dump >/dev/null
    bin="$build_dir/stage_dump"
    fixture_stem=$(basename "{{fixture}}" .stl)
    pair_dir="port/dumps/${fixture_stem}-{{configuration}}"
    run_root=oracle/.cache/stage-dump/runs
    mkdir -p "$run_root"
    armed_dir=$(mktemp -d "$run_root/armed-XXXXXX")
    unarmed_dir=$(mktemp -d "$run_root/unarmed-XXXXXX")
    golden_args=""
    case " {{ARGS}} " in
      *" --config "*|*" --set "*) ;;
      *)
        golden="port/goldens/configs/{{configuration}}.json"
        if [ -f "$golden" ]; then
          golden_args="--config $golden"
        fi
        ;;
    esac
    "$bin" "{{fixture}}" "{{configuration}}" --out "$armed_dir" $golden_args {{ARGS}}
    "$bin" "{{fixture}}" "{{configuration}}" --out "$unarmed_dir" --unarmed $golden_args {{ARGS}}
    just compare "$armed_dir/plate_1.gcode" "$unarmed_dir/plate_1.gcode" --require L1
    echo "L1"
    mkdir -p "$pair_dir"
    rm -f "$pair_dir"/*.dump
    cp "$armed_dir"/*.dump "$pair_dir"/
    echo "copied $(ls "$pair_dir"/*.dump | wc -l | tr -d ' ') dump(s) to $pair_dir"
    # Cleaned up only after a successful run reaches here; set -e leaves the two run
    # directories in place on any earlier failure, where they are the evidence.
    rm -rf "$armed_dir" "$unarmed_dir"

# measure the oracle's run-to-run reproducibility and publish the gate table (R3, plan step 6,
# task 27-oracle-determinism): for every row of docs/ORACLE.md's configuration table and every
# fixture it names, runs the oracle at least five times through just oracle-slice, compares every
# pair for the G-code, the --export-settings JSON and the --export-statistics JSON through
# just compare --format json, and regenerates port/reports/determinism.tsv,
# port/reports/determinism.md and docs/ORACLE.md's gate table from the measurement. Every run's
# scratch output lands under oracle/.cache/determinism/ (git-ignored, left behind for inspection).
oracle-determinism:
    ./oracle/determinism

# report whether the two oracles agree (R6, plan step 10, task 28-oracle-parity): for every row
# of docs/ORACLE.md's configuration table and every fixture it names, slices with the pinned
# release binary (oracle/slice) and with the locally built library (oracle/stage-dump's export
# path, unarmed, the hook never registered), compares the G-code and both JSON artifacts through
# just compare ... --format json, and regenerates port/reports/oracle-parity.tsv and
# port/reports/oracle-parity.md from the measurement. Builds oracle/stage-dump fresh into the
# git-ignored oracle/.cache/stage-dump/arm64 against the already-built vendor/OrcaSlicer/build/arm64
# (just oracle-build's own output; never rebuilt here). Scratch output lands under
# oracle/.cache/parity/ (git-ignored, left behind for inspection).
oracle-parity:
    ./oracle/parity
# the one spelling a port unit's pull request is held to (R28, plan step 21, task
# 36-pr-body-check): fails unless <file> (the pull request body) records both review verdicts
# the loop in docs/WORKFLOW.md states and names an existing port/manifest.tsv row, skipped --
# from the diff alone, never from the body -- for a pull request that adds no module. See
# `./port/port-status --help`'s own --pr-body section. `just port-status` (no argument) forwards
# nothing to it.
pr-body-check file *args:
    ./port/port-status --pr-body {{file}} {{args}}

# the command half of R20 / plan step 24 (task 39-compare-stages): slices fixtures/3DBenchy.stl
# with the port in the base FFF configuration and compares, for every pipeline stage
# port/manifest.tsv's own `pipeline stage or none` column marks as fully ported (derived the same
# way just port-status derives it, never read from the status cell), the port's per-seam stage
# dump against that stage's committed oracle dump (port/dumps/3DBenchy-base-fff/*.dump, from
# just oracle-dump 3DBenchy base-fff) at the stage's declared tier; the two stages with no R5
# seam fall back to the exported G-code's L0-L4 levels instead. A stage whose rows are not all
# `ported` prints `not ported`, which is not a failure -- see
# crates/oracle-compare/src/stages.rs's own doc comment for what this reads and does not re-derive.
# This task adds no CI line (task 40-compare-stages-ci owns that). `--port-bin` is passed in
# rather than read from `CARGO_BIN_EXE_grampus-cli`, which is only ever set inside that crate's
# own tests: this is what keeps crates/oracle-compare/Cargo.toml free of a dependency on
# grampus-cli (and, through it, on any slic3r-* crate or a libslic3r build).
compare-stages:
    cargo build --quiet --release -p grampus-cli
    rm -rf oracle/.cache/tmp/compare-stages
    mkdir -p oracle/.cache/tmp/compare-stages
    cargo run --quiet --release -p oracle-compare -- --stages \
      --manifest port/manifest.tsv \
      --tree . \
      --oracle-dumps port/dumps/3DBenchy-base-fff \
      --port-bin target/release/grampus-cli \
      --fixture fixtures/3DBenchy.stl \
      --config port/goldens/configs/base-fff.json \
      --scratch oracle/.cache/tmp/compare-stages
