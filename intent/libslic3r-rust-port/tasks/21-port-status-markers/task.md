---
kind: task
step: 14
delivers: [R14, R15, R16, R29]
depends: [14-manifest-columns]
size: 5
---
# Task: Check every ported module's trailer, markers and constants

## What to build

Plan step 14 builds `port/port-status`, the one command the porting work is held to, and is split
in three. This is **part 1: the text-based per-module checks**. The frontmatter's `delivers` is the
whole step's, so this task delivers only the per-module half of R14 and R16's placeholder-constant
check. The other two parts are `22-port-status-gates` (deriving each row's status, the burn-down,
the in-scope gate, the `unknown` count and the document-section checks of R29) and
`23-perf-backlog` (`port/reports/perf-backlog.md` and the per-marker report assertion).

Create `port/port-status` and add the `just port-status` recipe. Follow the repository's
convention for a script outside the Rust workspace: a standalone executable with no file
extension, `#!/usr/bin/env -S uv run` and PEP 723 inline metadata, `click` for the command line,
type hints throughout, run through `uv`. Add the recipe as

```
port-status *args:
    ./port/port-status {{args}}
```

so `just port-status`, `just port-status --per-commit` and the later `--pr-body <file>` all reach
it without the recipe being rewritten.

**Every check is text-based.** The script reads files as lines and never parses Rust, because it
has to run over Phase A drafts that are not valid Rust and that `cargo fmt` never sees, since no
`mod` declaration reaches them. No check in this task runs a build, reads the oracle binary or
needs a libslic3r build: `24-port-check-ci` puts this command in the checks a task passes on an
ordinary machine, so it has to pass there.

### What the script scans, and how it resolves paths

A **ported module** is a file under `<tree>/crates/*/src/**/*.rs` that holds a `PORT STATUS`
trailer. Accept `--tree <dir>`, defaulting to the repository root, and resolve both the module
scan and `<tree>/port/manifest.tsv` under it; resolve `vendor/OrcaSlicer/` from the repository
root always, because the pin is the pin. Assert that
`git -C vendor/OrcaSlicer rev-parse HEAD` is `f92a7c065c5718b8ef1baac1cd704df2911238f4` and exit
non-zero otherwise, since R14's promise is that a source path exists **at the pin**.

Read `port/manifest.tsv`'s header row for its column names rather than assuming positions —
`14-manifest-columns` owns the spellings — and index the rows by their `path` column and by their
target crate and module. A vendored path written in a trailer or a marker uses exactly the
spelling the manifest's `path` column uses, and the script resolves it through one function used
by both checks, so the two cannot disagree.

### The trailer, and its agreement with the manifest row

Fix the trailer's form here, because hundreds of modules write it and two tasks read it. Print it
in `--help`. It is the last run of line comments in the file, keys in this order, one per line:

```
// PORT STATUS
// source: src/libslic3r/Geometry/Circle.cpp
// source: src/libslic3r/Geometry/Circle.hpp
// confidence: high
// tier: exact
// oracle: built
```

The checks: the trailer is present and is the file's last non-blank content; `source:` appears at
least once and may appear more than once, which is how a unit of a `.cpp` plus its same-stem
`.hpp` and a header-only unit both fit; every `source:` path exists in the vendored tree at the
pin; every `source:` path has a row in `port/manifest.tsv` whose target crate and module name
**this** file's path, which is the module-to-manifest direction R14 asks for; `confidence:` is one
of `high`, `medium`, `low`; `tier:` equals that row's `equality tier` cell verbatim, compared as
text and never interpreted, because the tier's values belong to the manifest; and `oracle:` is one
of the closed pair `release` and `built`, the two oracles the spec names — the pinned release
binary and the locally built library with `-ffp-contract=off` — spelled as
`port/reports/oracle-parity.tsv`'s `normative oracle` column spells them. Where a module's
`source:` rows disagree about the tier, that is a finding and not something the script picks a
winner for.

### `// SAFETY:` on every `unsafe` block

A line that uses `unsafe` as a word, outside a line comment and outside a string literal, fails
unless a `// SAFETY:` comment is on that same line or on one of the contiguous comment lines
immediately above it. There is no exemption list.

### `// PORT SRC:` beside every numeric constant, and the placeholder check

This is the rule the task has to design, because R14 and R16 say "numeric constant" and a
text-based script needs something closed. Write the rule down in the script's module docstring
**and** in `--help`, in one place that both the check and the message quote, and cover every
branch of it with a fixture below. Use this shape:

- A line of a ported module that contains a numeric literal fails unless every non-exempt literal
  on it is covered by a marker, or every literal on it is exempt. **More than one marker may apply
  to a line**: the markers that cover a line are the `// PORT SRC: <vendored path>:<line>` on that
  line itself together with those on each contiguous comment line immediately above it. A line
  writing two transcribed constants from two different vendored lines therefore carries two markers
  and needs no expression split — and a line writing two non-exempt literals under one marker that
  matches only one of them **fails**, which is the case a looser reading would let through.
- **Every `// PORT SRC:` marker must match at least one literal on the line it covers**, so a marker
  left behind after the constant it named was edited away is a finding rather than silent.
- The exempt literal set is closed and written into the script with its reason: the integers `0`,
  `1` and `2` and their negatives, which are structural indices, arities and strides rather than
  transcribed constants. Every other literal needs the marker, including every float literal.
- The exempt positions are closed too: inside a line or block comment, inside a string or char
  literal, inside an attribute such as `#[...]`, and inside a test file under `crates/*/tests/`
  or a `#[cfg(test)]` module.
- Both sets are closed, and there is **no per-module, per-file or per-line opt-out**: no marker, no
  allowlist file and no comment in a module suppresses this rule, and neither set changes except by
  a pull request against `port/port-status`. Say so in the rule's own words, where the script
  states it and where its failure message quotes it. This is the part of the script most likely to
  be wrong on first contact with real modules, so a reviewer of the first Phase A pull request
  needs to know the right answer to noise is to change the script, not the module.

Then the placeholder check R16 asks for: for each `// PORT SRC: <path>:<line>`, the path exists at
the pin, the file has that line, and that line **contains a literal the marked line writes**; and,
taken over all the markers covering a line, **every** non-exempt literal on that line is matched by
at least one of their cited vendored lines. The two directions together are what make the rule
checkable: the per-marker direction catches a stale marker, and the per-literal direction catches a
placeholder constant hiding behind the easiest literal on the line.
State and implement one normalisation before the search, because a Rust literal is not spelled
exactly as a C++ one: strip Rust digit separators (`1_000` searches for `1000`) and a type suffix
(`0.4f32` searches for `0.4`), and search for the remaining text. Two real pairs to use as
fixtures: `0.0125` against `src/libslic3r/libslic3r.h:86`, which holds
`static constexpr double RESOLUTION = 0.0125;`, and `1e-4` against `src/libslic3r/libslic3r.h:57`,
which holds `static constexpr double EPSILON = 1e-4;`. A marker naming a line that does not hold
the literal is the placeholder-constant failure, and it is a failure in both modes.

### The closed marker whitelist

Every port marker is one of `// TODO(port):`, `// PERF(port):`, `// PORT NOTE:`, `// SAFETY:`,
`// PORT SRC:` and `// FIDELITY(port):`. Write the candidate-marker detector down as a regular
expression in the script — a line comment whose text begins with an all-capitals word, optionally
followed by `(port)`, and then a colon — and fail on any candidate whose prefix is not exactly one
of the six, naming the file, the line and the unrecognised prefix. A near miss such as
`// TODO:`, `// Todo(port):` or `// PORT-SRC:` is a finding; that is what the whitelist is for.

### Output, exit status and the flag this task leaves open

Print one finding per line as `<path>:<line>: <rule>: <message>`, print the count per marker kind
over the whole tree, and exit non-zero if there is any finding and zero otherwise. Accept
`--per-commit` and run the same per-module checks in both modes: every check in this task is one a
port task has to pass, so the two modes do not differ yet. Say so in `--help` and leave the real
difference between the modes to `22-port-status-gates`, which adds the burn-down, the in-scope-row
gate, the `unknown` count and the document-section checks and makes them fail only without the
flag.

### Proving it with no ported module in the tree

At this point in the order of work no crate and no ported module exists, so the checks cannot be
exercised against real code and the negative cases cannot be shown by breaking one. Commit fixture
trees instead, under `port/tests/port-status/`: one `pass/` tree and one tree per rule that fails,
each holding a `crates/<crate>/src/<module>.rs` and a small `port/manifest.tsv` with the rows that
module's trailer names. They sit outside `crates/`, so no `cargo` command ever reads them. Add
`--self-test`, which runs the script over each fixture tree with `--tree` and asserts the expected
exit status and the expected finding for each, so the rules above are checked rather than
described. The fixtures cite real vendored paths and real vendored lines at the pin, so the
`pass/` tree passes for the same reason a real module will.

`port/tests/port-status/` and `--self-test` are additions beyond the plan's "Files that change"
list; they are here because R14's proof is that the command exits non-zero on a module whose
trailer is missing, whose source path does not exist at the pin or disagrees with its manifest
row, on an `unsafe` block with no `// SAFETY:`, on a numeric constant with no `// PORT SRC:` and
on an unrecognised marker, and none of that can be shown without something to run it over. Name
them in the pull request so a reviewer judges them.

### What this task waits on, and what it must not touch

It waits on `14-manifest-columns` for `port/manifest.tsv`'s column semantics and for the rule that
status is derived from the tree rather than read from a cell.

Do not write: `port/reports/perf-backlog.md` or the per-marker report assertion (`23-perf-backlog`);
the status derivation, the burn-down, the in-scope gate, the `port/ownership.tsv` `unknown` count
or the `docs/PORTING.md` and `docs/WORKFLOW.md` section checks (`22-port-status-gates`); the
`--pr-body <file>` mode or `just pr-body-check` (`36-pr-body-check`); `port/manifest.tsv` itself,
any file under `crates/`, the root `Cargo.toml`, `Cargo.lock`, any generated report, any line in
`.asdlc/checks` or any line in `.github/workflows/ci.yml` — `24-port-check-ci` adds
`just port-status --per-commit` to `just port-check` and creates the jobs. No check this task adds
gates on throughput, instruction count or allocation, which R15 forbids anywhere in this
repository.

**Why this is a 5.** Several checks that must all hold at once, all of them text-based because they
run over Phase A drafts that are not valid Rust: a trailer agreeing with a manifest row the script
has to index two ways, source paths resolved against a pinned submodule, `// SAFETY:` on every
`unsafe` block, `// PORT SRC:` beside every numeric constant with the vendored line actually
holding that literal — which needs a closed definition of "numeric constant" and a literal
normalisation that neither R14 nor R16 supplies — and a closed marker whitelist that has to fail
on near misses. Each of the five is cheap alone; what makes the task a 5 is that it is held to
them by hundreds of later modules and has to be proven with no module in the tree yet. It was not
split further because the five checks share one file walk, one manifest index and one path
resolver, and splitting them would mean three branches editing the same script.

## Done when

- `port/port-status` exists as an executable standalone script with a `uv` shebang and PEP 723
  inline metadata, and `just port-status` forwards its arguments to it.
- Running it with no argument and with `--per-commit` both run the per-module checks, print the
  count per marker kind, and exit zero on a tree with no ported module.
- It exits non-zero when `git -C vendor/OrcaSlicer rev-parse HEAD` is not
  `f92a7c065c5718b8ef1baac1cd704df2911238f4`.
- `--help` prints the trailer's exact form and the numeric-constant rule, including the exempt
  literal set, the exempt positions and the statement that there is no per-module, per-file or
  per-line opt-out.
- `port/tests/port-status/` holds a passing fixture tree and **one failing fixture tree for every
  rule this task states**: missing trailer; trailer with a bad `confidence:`; trailer whose `tier:`
  disagrees with its manifest row; trailer whose `oracle:` value is outside the closed pair
  `release` and `built`; trailer that is not the file's last non-blank content; trailer with no
  `source:` line; a two-`source:` module whose rows' manifest tiers disagree; `source:` path absent
  at the pin; `source:` path whose manifest row names another module; `unsafe` block with no
  `// SAFETY:`; numeric constant with no `// PORT SRC:`; `// PORT SRC:` whose vendored line does not
  hold the literal; a line with two non-exempt literals carrying one marker that matches only one of
  them; a `// PORT SRC:` marker that matches no literal on the line it covers; and an unrecognised
  port marker.
- The passing tree additionally holds a line with two non-exempt literals carrying two markers, one
  per literal, which passes.
- `port/port-status --self-test` exits zero, and prints for each fixture tree the expected exit
  status and the finding it matched.
- A reviewer can break any one of those rules in the passing fixture tree and see the command
  exit non-zero naming that file, that line and that rule.
- The script parses `port/manifest.tsv` by its header row and not by column position, and resolves
  a vendored path through one function shared by the trailer check and the `// PORT SRC:` check.
- The marker whitelist rejects `// TODO:`, `// Todo(port):` and `// PORT-SRC:` and accepts exactly
  the six markers.
- `just port-status` completes with no build, no oracle binary and no libslic3r build present.
- The pull request changes only `port/port-status`, `port/tests/port-status/**` and the `justfile`,
  and names the two additions beyond the plan's file list.
