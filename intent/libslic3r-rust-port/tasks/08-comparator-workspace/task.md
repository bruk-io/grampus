---
kind: task
step: 5
delivers: [R4]
size: 3
---
# Task: Create the Rust workspace and the comparator's byte levels

## What to build

This is part 1 of plan step 5, "The comparator", which delivers R4. It builds the Rust workspace
the whole repository's Rust code lives in, the `oracle-compare` crate with its binary and its
report shape, the two byte levels L0 and L1, and the declared normalisation together with the test
that pins it to the vendored source. Task `09-comparator-gcode-levels` adds the G-code parser and
L2, L3 and L4 to the same crate, and task `10-comparator-json` adds `src/json.rs`; both wait on
this task. Leave `src/gcode.rs`, `src/json.rs` and `src/stages.rs` uncreated.

This task waits on nothing. It must run on a machine with no oracle binary and no libslic3r build,
so it never invokes the oracle and commits no file produced by a run of it.

**The workspace, which every later crate depends on.** Add:

- `Cargo.toml` at the repository root, whose `[workspace]` is `members = ["crates/*"]` and nothing
  else. That glob is the point: no later step and no port task ever edits the root manifest to add
  a crate, because the directory appearing under `crates/` is enough.
- `rust-toolchain.toml`, pinning an exact three-part stable version — the one
  `rustc --version` reports in the environment you build in, written out in full and never
  `stable` — with `components = ["rustfmt", "clippy"]` and `profile = "minimal"`. Task
  `24-port-check-ci` installs exactly that version in CI, and `cargo fmt --check` runs there.
- `Cargo.lock`, committed, because the oracle comparison has to be reproducible on the one
  platform its goldens are valid for.
- `/target/` in `.gitignore`.

`crates/oracle-compare` takes no third-party dependency: the binary reads its arguments from
`std::env::args`, so the committed lock file holds workspace members only and later crates never
contend on it. If you add a dependency, pin it exactly and give the reason in the pull request.

Files this task creates under the crate: `crates/oracle-compare/Cargo.toml`, `src/lib.rs`,
`src/levels.rs`, `src/main.rs`, `tests/levels.rs`, `tests/normalisation.rs` and the fragments under
`tests/data/`. Add `just compare` to the `justfile`, wired to the binary and **variadic** —
`compare a b *ARGS:` running `cargo run --quiet --release -p oracle-compare -- {{a}} {{b}} {{ARGS}}`.
It is variadic rather than two-parameter because `just` rejects a surplus argument to a fixed-arity
recipe, and the flag runs this task's own "Done when" requires (`--format json`, `--require L1`), the
four flags task `09-comparator-gcode-levels` adds (`--tolerance-from-manifest`, `--row`,
`--tolerance-from-config`, `--configuration`) and the two JSON arms task `10-comparator-json` adds
would each otherwise need the recipe reopened — and task 09's own "Done when" forbids it changing
anything outside `crates/oracle-compare/`.

**The report shape, which the later parts fill.** `src/lib.rs` defines what every part of step 5
returns, because R4 is one comparator that evaluates each level independently and does not assume
the levels are nested:

- a `Level` enum with all five variants, `L0` to `L4`;
- a per-level outcome of pass, fail with the first failing location, or not-evaluated;
- a report holding the outcome of each level, the strongest level that passed, the first failing
  location at the next stronger level, the first differing byte when no level passed, and the
  tolerance the run used (always `exact` in this task; task 09 fills it).

This task evaluates L0 and L1 and returns not-evaluated for L2, L3 and L4. Use no `todo!()` and no
`unimplemented!()` anywhere: `just port-status` counts both as scaffolding across `crates/`
(plan step 14), so an honest `NotEvaluated` variant is the way to leave the three levels open.
While they are not evaluated the binary labels the strongest level as the strongest *evaluated*
level, and prints `none of L0, L1` rather than a bare `none`, so no output claims more than was
measured; task 09 removes both qualifiers.

Read both files as bytes (`Vec<u8>`) and never validate UTF-8: a G-code comment may hold bytes that
are not valid UTF-8, and a comparator that refuses such a file cannot compare the oracle's output.

**L0 and L1.** L0 is identical bytes. L1 is identical bytes after the declared normalisation. The
first failing location at L0 is the zero-based byte offset into each file; at L1 it is the
one-based line number in each side's normalised stream, with both lines printed.

**The declared normalisation.** It is exactly the four line kinds upstream itself strips in
`vendor/OrcaSlicer/tests/fff_print/test_slicing_pipeline_hook.cpp:83-96`, in
`strip_nondeterministic_gcode_lines`, and nothing else in the G-code is normalised. Reproduce that
function's semantics, which are load-bearing byte for byte:

- split the input on `\n`, so a `\r` before the newline is kept — there is no line-ending
  normalisation;
- drop a line whose first 15 bytes are `; generated by `;
- drop a line whose first 18 bytes are `; model label id: `;
- drop any line containing `printing object, unique label id: `;
- drop any line containing `slicing_pipeline_plugin`;
- re-emit every kept line followed by `\n`, so a file whose last line has no newline gains one and
  empty input stays empty.

Define the four literals in exactly one place in the Rust tree. The first two are prefix matches
and the last two are substring matches, as upstream has them.

**`tests/normalisation.rs` pins the list to the vendored file**, so neither side can drift. It
reads `vendor/OrcaSlicer/tests/fff_print/test_slicing_pipeline_hook.cpp` (resolve it from
`CARGO_MANIFEST_DIR`), finds the body of `strip_nondeterministic_gcode_lines`, extracts every
string literal in it, and asserts that the set is exactly the four literals the comparator uses,
that there are exactly four, and that the two prefix literals are the ones upstream passes to
`compare(0, N, …)` and the two substring literals the ones it passes to `find`. Strip `//`
comments before collecting literals: the function body carries a comment at line 90 holding a
quoted example line, so an extractor that keeps comments finds five literals and the count
assertion fails on a correct implementation. Assert as well that each `N` in `compare(0, N, …)`
equals the byte length of the literal beside it, which is the second way upstream's list is tied
to this one. The test fails,
with a message naming the path, when the vendored file is absent — it never skips, because a
skipped test is how this check would quietly stop holding.

**The binary.** `oracle-compare <a> <b> [--require <level>] [--format text|json]`:

- text output prints one line per level with its outcome, the strongest level that passed, the
  first failing location at the next stronger level, the tolerance used, and, when no evaluated
  level passed, the first differing byte;
- `--format json` prints one object with the same items under stable key names, because the
  oracle scripts of steps 6, 10 and 25 to 27 are standalone Python executables that call this
  binary and must not parse prose;
- `--require <level>` exits non-zero unless that level passed, which is how a caller gates on a
  level it read from `port/reports/determinism.tsv`;
- without `--require` the command exits 0 whenever the comparison ran, including at `none`, so a
  caller reads the level and not the exit code. A usage error or an unreadable file exits non-zero
  with a named error.

**The committed fragments** under `crates/oracle-compare/tests/data/` are hand-written G-code
excerpts, each small enough that a reviewer can read it, covering at least: an identical pair; a
pair differing only in a `; generated by ` line; a pair where one side carries a
`slicing_pipeline_plugin` line in its config block and the other does not, which is the armed
against unarmed shape step 8 compares; a pair differing in a `; model label id: ` line and in a
`; start printing object, unique label id: N` line; a pair differing in one `G1` coordinate; and
two unrelated files. Commit `tests/data/README.md` saying that these are hand-written because this
task cannot run the oracle, and that the plan's "pair taken from two real runs of step 2" is
committed by **task `10-comparator-json`**, the last part of step 5, which depends on
`04-oracle-slice` and so has the oracle available. Name that task in the README rather than
deferring to "a later task", so the plan item is not lost between the three parts of step 5.

**What this task does not touch.** `.asdlc/checks` and `.github/workflows/ci.yml` belong to plan
step 16, which is task `24-port-check-ci`; it adds the `just port-check` line and the macOS job
that runs `cargo test --workspace`, so add no CI line and no check line here. Do not touch
`README.md`'s checks or CI bullets, which the same step corrects; a one-row addition to its layout
table for `crates/` is fine. Nothing under `vendor/OrcaSlicer/` changes and the submodule pointer
stays at `f92a7c065c5718b8ef1baac1cd704df2911238f4`.

## Done when

- `cargo build --workspace` and `cargo test --workspace` pass from a clean checkout on a machine
  with no oracle binary and no libslic3r build.
- `cargo fmt --check` exits 0.
- The root `Cargo.toml` has `members = ["crates/*"]` and no other member list;
  `rust-toolchain.toml` names a three-part version and the `rustfmt` component; `Cargo.lock` is
  committed; `.gitignore` holds `/target/`.
- The committed `Cargo.lock` lists no package that is not a workspace member, or the pull request
  gives the reason for each one it does.
- `just compare` on the identical fragment pair prints L0 and L1 as passing and names L1 as the
  strongest evaluated level; on the `; generated by ` pair it prints L0 failing with the byte
  offset and L1 passing; on the `slicing_pipeline_plugin` pair it prints L1 passing; on the
  one-coordinate pair it prints both failing with the line; on the unrelated pair it prints
  `none of L0, L1` with the first differing byte.
- Each of those runs with `--format json` prints the documented object, and `--require L1` exits 0
  on the `; generated by ` pair and non-zero on the one-coordinate pair.
- `cargo test -p oracle-compare --test normalisation` passes; changing any one of the four literals
  in the Rust tree makes it fail, and so does moving the vendored file out of the way.
- `grep -rn 'generated by' crates/*/src/` finds the literal in exactly one place. The grep is
  scoped to the source tree because the committed fragment pair under
  `crates/oracle-compare/tests/data/` differs precisely in a `; generated by ` line and so carries
  the literal twice more by design.
- `grep -rn 'todo!\|unimplemented!' crates/` finds nothing, and L2, L3 and L4 report
  `not evaluated`.
- `crates/oracle-compare/src/gcode.rs`, `src/json.rs` and `src/stages.rs` do not exist.
- `.asdlc/checks`, `.github/workflows/ci.yml` and `README.md`'s checks and CI bullets are
  unchanged; `git -C vendor/OrcaSlicer status --porcelain` prints nothing and the submodule pointer
  is unmoved.
