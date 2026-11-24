# asdlc from its repository, so this runs the same in CI and in a terminal with no plugin
asdlc := "uvx --from git+https://github.com/bruk-io/asdlc asdlc"

# everything a pull request has to pass
check:
    {{asdlc}} document check

# the guard: this branch changes only what its documents on main allow
guard *branch:
    {{asdlc}} branch check {{branch}}

# regenerate port/manifest.tsv's file set and per-file derived columns from the vendored tree
inventory:
    ./port/inventory
# variadic (not two-parameter) because `just` rejects a surplus argument to a fixed-arity recipe,
# and callers across steps 5, 6, 9, 10 and 25-27 each need their own extra flags
compare a b *ARGS:
    cargo run --quiet --release -p oracle-compare -- {{a}} {{b}} {{ARGS}}
