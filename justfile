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

# regenerate port/reports/order-containers.md and port/reports/order-comparators.md (R18, step
# 13 part 1, task 17-order-audits): every unordered container that is iterated rather than only
# looked up, and every comparator that is not a total order, over the manifest's in-scope rows,
# read at run time. Not a check a port task passes (just audit-check, 18-divergences-regeneration,
# is); this recipe only regenerates the two reports.
audit-order:
    ./port/audit-order
