---
kind: task
step: 1
delivers: [R1]
size: 3
---
# Task: Fetch and verify the pinned OrcaSlicer 2.5.2 release binary

## What to build

The whole of the plan's step 1, "Fetch and verify the pinned oracle binary". It waits on nothing
and starts immediately, alongside step 7 and step 11. Every later oracle step runs the binary this
task produces.

**Commit the recorded hash first.** `vendor/README.md` gains the release asset's name,
`OrcaSlicer_Mac_universal_V2.5.2.dmg`, and its recorded sha256,
`6ea1afcdffb7e9ab011e2d6f6db665cb18463d1c4774591cd5db7ee0225479d9`. That value was measured
during this intent's research and must be committed before the first fetch is run, so the check is
not circular. The script reads it and never writes it.

**Add `oracle/fetch-release`** and the `just oracle-fetch` recipe in `justfile` that runs it. Like
every script outside the Rust workspace it is a standalone executable with PEP 723 inline metadata
and no file extension, run through `uv` (`#!/usr/bin/env -S uv run`), as the repository's
conventions require.

What the script does, in order:

1. Downloads the asset with
   `gh release download v2.5.2 -R OrcaSlicer/OrcaSlicer -p 'OrcaSlicer_Mac_universal_V2.5.2.dmg'`
   into `oracle/.cache/download/`, a directory of its own outside `/Applications`.
2. Reads the recorded sha256 out of `vendor/README.md` and compares it against the measured digest
   of the download. On a mismatch it exits non-zero, prints both the expected and the measured
   value, and **never rewrites the recorded hash**.
3. Mounts the image read-only with `hdiutil attach -readonly -nobrowse`.
4. Copies the bundle to `oracle/.cache/app/OrcaSlicer.app`.
5. Detaches the image with `hdiutil detach`, on the failure paths as well as the success path, so
   no run leaves an image attached.
6. Verifies that
   `/usr/libexec/PlistBuddy -c "Print :CFBundleShortVersionString" oracle/.cache/app/OrcaSlicer.app/Contents/Info.plist`
   prints `2.5.2`, and exits non-zero otherwise.
7. On success prints the asset's sha256, `2.5.2`, and the path of the binary the later steps use.

The binary the later steps use is
`oracle/.cache/app/OrcaSlicer.app/Contents/MacOS/OrcaSlicer`. That is the GUI binary, which is also
the CLI and runs headless with no `DISPLAY`; there is no separate console executable in the bundle.

Add `oracle/.cache/` to `.gitignore`, so neither the download nor the copied bundle can be
committed.

**Create `docs/ORACLE.md` with its first two sections and nothing else:** the asset and its hash —
the release tag, the asset name, the recorded sha256, and the path the fetch writes the bundle and
the binary to — and the statement that `/Applications/OrcaSlicer.app` (which is 2.4.2) and
`/Applications/BambuStudio.app` are never read by any command in this repository. Later steps add
that document's other sections and this task writes none of them: the configuration table (step 2),
the gate table (step 6), the built oracle's flags (step 7) and the resolved-config to G-code key
relationship (step 9).

The real work in this task is the failure paths and the document, not the happy path: a corrupt or
superseded download, a `gh` that is not authenticated or finds no such asset, an image that will not
attach, and a bundle that reports a version other than `2.5.2`. Each exits non-zero with a named
reason, leaves no image attached, and leaves `vendor/README.md` untouched. That is what makes this
a size 3: the approach is settled and there are no open questions, but there is real work in
building and testing those paths.

**A second run over a populated cache is a success, not a failure, and the command is idempotent.**
A run that finds `oracle/.cache/download/` already holding the asset re-verifies the recorded
sha256 against the cached download, and re-verifies the copied bundle's
`CFBundleShortVersionString`; where both match it skips the download, the mount and the copy,
prints the same three values as a first run and exits 0. Where either disagrees it re-fetches from
step 1 of the list above. This matters beyond convenience: R1's command is also a step of the macOS
job step 16 creates, and a command that refused a populated cache would turn that job red on every
pull request whose path filter matches.

**What this task must not touch.** Nothing under `vendor/OrcaSlicer/` changes and the submodule
pointer stays at `f92a7c065c5718b8ef1baac1cd704df2911238f4`. No command in this repository reads
`/Applications`. The task adds nothing to `.asdlc/checks` and nothing to
`.github/workflows/ci.yml`: step 16 creates the macOS job and puts `just oracle-fetch` in it.

## Done when

- `just oracle-fetch` exits 0 and prints the asset's sha256 and `2.5.2`.
- `oracle/.cache/app/OrcaSlicer.app/Contents/MacOS/OrcaSlicer` exists after that run, and
  `/usr/libexec/PlistBuddy -c "Print :CFBundleShortVersionString" oracle/.cache/app/OrcaSlicer.app/Contents/Info.plist`
  prints `2.5.2`.
- `vendor/README.md` records the asset name `OrcaSlicer_Mac_universal_V2.5.2.dmg` and the sha256
  `6ea1afcdffb7e9ab011e2d6f6db665cb18463d1c4774591cd5db7ee0225479d9`, in a commit on this branch
  that precedes the first commit that runs the fetch.
- Run with the recorded value in `vendor/README.md` altered by one character, `just oracle-fetch`
  exits non-zero and prints both the expected and the measured value, and `git diff vendor/README.md`
  afterwards shows only the one-character edit the test made, with the measured digest nowhere in
  the file.
- `just oracle-fetch` run twice in a row exits 0 both times and prints the sha256 and `2.5.2` both
  times, and the second run performs no `gh release download` and no `hdiutil attach`.
- `grep -r '/Applications/' justfile oracle` finds nothing. R1's proof names a third path,
  `port`, which does not exist until step 11's first task lands, so it joins the list then. `docs`
  is deliberately not in the list at all: `docs/ORACLE.md`'s second section names those two bundles
  as the ones never read, so an unscoped grep would fail this repository's own check.
- `hdiutil info` shows no image from this repository attached, both after a successful run and
  after a run that was made to fail at the version check.
- `git status --porcelain` is clean after a run, because `.gitignore` holds `oracle/.cache/`.
- `oracle/fetch-release` has no file extension, starts with the `uv` shebang and carries a PEP 723
  inline metadata block.
- `docs/ORACLE.md` exists and holds exactly the two sections named above — no configuration table,
  no gate table, no build flags.
- `git -C vendor/OrcaSlicer rev-parse HEAD` prints `f92a7c065c5718b8ef1baac1cd704df2911238f4` and
  `git status --porcelain vendor/OrcaSlicer` is empty.
