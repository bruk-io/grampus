# asdlc from its repository, so this runs the same in CI and in a terminal with no plugin
asdlc := "uvx --from git+https://github.com/bruk-io/asdlc asdlc"

# everything a pull request has to pass
check:
    {{asdlc}} document check

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
