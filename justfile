# asdlc from its repository, so this runs the same in CI and in a terminal with no plugin
asdlc := "uvx --from git+https://github.com/bruk-io/asdlc asdlc"

# everything a pull request has to pass
check:
    {{asdlc}} document check

# the guard: this branch changes only what its documents on main allow
guard *branch:
    {{asdlc}} branch check {{branch}}

# build the vendored OrcaSlicer tree on this platform
oracle-build:
    ./oracle/build-libslic3r
