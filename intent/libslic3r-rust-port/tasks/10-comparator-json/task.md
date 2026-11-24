---
kind: task
step: 5
delivers: [R4]
depends: [08-comparator-workspace, 04-oracle-slice]
size: 2
---
# Task: Compare the resolved-config and statistics JSON artifacts

## What to build

This is part 3 of plan step 5, "The comparator": `crates/oracle-compare/src/json.rs` and its
tests. The step is split three ways and `delivers` is the whole step's. Part 1
(`08-comparator-workspace`) adds the Rust workspace, `crates/oracle-compare`, `src/lib.rs`,
`src/levels.rs` with L0 and L1, and the declared G-code normalisation. Part 2
(`09-comparator-gcode-levels`) adds `src/gcode.rs` and L2, L3 and L4. **This task adds the JSON
comparison only and changes no G-code level.**

The comparator has two JSON artifacts to compare, and each has its own rule. Both rules are
exact: no numeric tolerance exists on either artifact, and the comparator takes a tolerance only
from a `port/manifest.tsv` row or a `docs/ORACLE.md` configuration row, neither of which applies
here. Nothing in `src/json.rs` reads or accepts a tolerance.

**The resolved configuration** — the oracle's `--export-settings` output, which `oracle/slice`
writes as `settings.json` — is compared **byte for byte with no normalisation**. There is exactly
one allowance, and it is off by default: when the compared pair includes a run with the slicing
pipeline hook armed, the `slicing_pipeline_plugin` key is allowed to differ, because arming the
hook requires that option to be non-empty
(`vendor/OrcaSlicer/src/libslic3r/Print.cpp:3306-3310`, where `m_pipeline_plugin_active` is set
from a registered hook and a non-empty `ConfigOptionStrings`). The caller switches the allowance
on for an armed/unarmed pair; no other key is ever allowed to differ, and the default path
allows nothing.

**The statistics JSON** — the oracle's `plate_<N>.statistics.json`, whose name
`gcode_statistics_path` derives by replacing the G-code's extension with `.statistics.json`
(`vendor/OrcaSlicer/src/libslic3r/GCode/GCodeStatistics.cpp:185-188`) — is compared with
**exactly one field normalised**, `generator`, which the oracle writes as the static string
`std::string(SLIC3R_APP_NAME) + " " + SoftFever_VERSION`
(`vendor/OrcaSlicer/src/libslic3r/GCode/GCodeStatistics.cpp:177`) and which the port cannot
reproduce without claiming to be OrcaSlicer. Every other key is compared, `version` and the
`summary`, `units`, `layers` and `objects` objects included.

**The reported shape is load-bearing**, because later tasks read the vocabulary rather than
re-deriving it: `27-oracle-determinism` writes a `gate` cell for the `settings-json` and
`statistics-json` rows of `port/reports/determinism.tsv`, and `41-compare-benchy`,
`42-compare-fixtures` and `43-compare-configurations` hold the port to those cells. So reuse the
level type part 1 defines in `src/levels.rs` and report, for a JSON artifact, only:

- `L0` — identical bytes;
- `L1` — identical after that artifact's declared allowance or normalisation;
- `none` — otherwise, with the first difference located.

L2, L3 and L4 are G-code levels and are never reported for a JSON artifact. A `none` result
names the first difference as a JSON pointer plus the two values; where a file cannot be parsed
as JSON at all the result reports that and the first differing byte. A malformed or unreadable
input is a reported failure with a non-zero exit, never a panic: no `unwrap` or `expect` on input
data anywhere in `src/json.rs`.

Write two public entry points, one per artifact, each taking the two paths and returning that
result type — the settings one additionally taking whether the `slicing_pipeline_plugin`
allowance is on. Add `mod json;` to `crates/oracle-compare/src/lib.rs`, and one flag arm to
`crates/oracle-compare/src/main.rs` so each artifact is reachable from the command line as
`just compare --settings <a> <b>` and `just compare --statistics <a> <b>`; steps 6, 10, 25 and 27
of the plan compare all three artifacts through this one comparator, so leaving JSON unreachable
from the binary would leave them nothing to call. Those two command lines pass three arguments, so
they work only because part 1 makes the `compare` recipe variadic (`compare a b *ARGS:`); this task
does not reopen the `justfile`. `src/main.rs` and
`crates/oracle-compare/tests/data/` are shared with parts 1 and 2 — expect an ordinary one-line
merge conflict there and nothing more; that overlap is not a dependency and `depends` stays as
cut.

**The JSON dependency is named here, not chosen by the builder.** `src/json.rs` parses JSON, locates
a first difference as a JSON pointer and compares per key, and part 1 states that
`crates/oracle-compare` takes no third-party dependency — which would leave the builder writing a
JSON parser by hand, and a hand-written parser with pointer locations and no `unwrap` on input data
is not a size 2. So this task adds **`serde_json` at exactly `1.0.117`** to
`crates/oracle-compare/Cargo.toml`, the committed `Cargo.lock` gains it and its own transitive
`serde` and `itoa`/`ryu` entries, and the pull request body gives the reason for each, which is what
part 1's lock-file rule requires. Plan step 5 is one of the steps the plan allows to change
`Cargo.lock`. Part 1's "no third-party dependency" sentence described the binary it built, which
only read `std::env::args`; this is the one addition step 5 makes, and no Phase A or Phase B port
task may add another.

**This task also commits the plan's real-run pair**, which step 5 requires
(`tests/levels.rs` and `tests/json.rs` cover each level "against committed fragments under
`tests/data/`, including a pair taken from two real runs of Step 2") and which parts 1 and 2 both
deferred — part 1 by name to this task, part 2 by saying every fragment it commits is hand-written.
This task depends on `04-oracle-slice` for exactly that: run
`just oracle-slice vendor/OrcaSlicer/tests/data/frog_legs.obj base-fff --export-settings --export-statistics`
twice, and commit, under `crates/oracle-compare/tests/data/real/`, the two runs' `settings.json`,
the two runs' `plate_1.statistics.json` and the two runs' `plate_1.gcode` truncated to a reviewable
excerpt that keeps the header block and one layer. Update `tests/data/README.md`, which part 1
commits saying the pair arrives with this task, to say it has arrived and which runs it came from.
`tests/json.rs` compares the real settings pair and the real statistics pair, and `tests/levels.rs`
compares the real G-code pair, so the level the oracle reaches against itself on a real run is
covered by a test and not only by hand-written fragments.

The rest of the tests go in `crates/oracle-compare/tests/json.rs` against small committed fragments
under `crates/oracle-compare/tests/data/`, needing neither the oracle binary nor a network.
Cover, as separate cases: two identical settings files reach `L0`; two settings files differing
only in `slicing_pipeline_plugin` reach `none` with the allowance off and `L1` with it on; two
settings files differing in any other key reach `none` with that key named as the pointer; two
statistics files differing only in `generator` reach `L1` and not `L0`; two statistics files
differing in a `summary`, `layers` or `objects` number reach `none` with the pointer named; and a
malformed JSON input is reported rather than panicking.

## Done when

- `crates/oracle-compare/src/json.rs` exists, is reached by a `mod json;` line in
  `crates/oracle-compare/src/lib.rs`, and exposes one public comparison entry point per artifact.
- The settings comparison is byte for byte with no normalisation, and the only key it can ever
  allow to differ is `slicing_pipeline_plugin`, with `src/libslic3r/Print.cpp:3306-3310` cited in
  a comment beside the allowance.
- That allowance is off by default: a caller that does not ask for it gets `none` on a pair
  differing only in `slicing_pipeline_plugin`, and `tests/json.rs` asserts both directions.
- The statistics comparison normalises exactly the `generator` field and no other, with
  `src/libslic3r/GCode/GCodeStatistics.cpp:177` cited in a comment beside it.
- A JSON artifact is reported only as `L0`, `L1` or `none`, using the level type `src/levels.rs`
  defines; no code path reports L2, L3 or L4 for a JSON artifact.
- A `none` result names the first difference as a JSON pointer with both values. **A pair that is
  not parseable JSON is a reported `none` with the first differing byte and exit 0, not part 1's
  named-error exit**: a file that exists and is readable but is not JSON is a measurement of the two
  artifacts, and only a usage error or an IO failure exits non-zero.
- A malformed, empty or unreadable input file never panics: a malformed or empty file is a reported
  `none`, an unreadable one is part 1's named-error exit, and the malformed-input case in
  `tests/json.rs` asserts the first of those; no `unwrap` or `expect` in `src/json.rs` is reached by
  file IO or by a value parsed from an input file.
- `just compare --settings <a> <b>` and `just compare --statistics <a> <b>` each print the level
  reached and the first difference, and **exit 0 whenever the comparison ran, including at `none`**,
  which is the exit convention part 1 establishes for this binary: a caller reads the level, not the
  exit code. The command exits non-zero only under `--require <level>` that did not pass, on a usage
  error, or on an unreadable file. `27-oracle-determinism` depends on this: it records a gate of
  `none` for these two artifacts as a measurement and its own command still exits 0.
- No tolerance is read, accepted or applied in `src/json.rs`:
  `grep -in 'toler' crates/oracle-compare/src/json.rs` finds nothing.
- `cargo test -p oracle-compare` passes `tests/json.rs`, which has one case for each of the six
  listed above.
- `crates/oracle-compare/tests/data/real/` holds, from two real `just oracle-slice` runs, two
  `settings.json`, two `plate_1.statistics.json` and two excerpted `plate_1.gcode`;
  `tests/json.rs` compares the first two pairs and `tests/levels.rs` the third; and
  `tests/data/README.md` says which runs they came from and no longer says the pair is owed by a
  later task.
- `crates/oracle-compare/Cargo.toml` names `serde_json` at exactly `1.0.117` and no other
  third-party dependency, `Cargo.lock` carries it and its transitive entries, and the pull request
  body gives the reason for each package in the lock that is not a workspace member.
- `cargo fmt --check` and `just check` pass, and the branch adds no file under `intent/`, and the
  branch does not change the `justfile`.
