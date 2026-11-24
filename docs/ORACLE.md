# The oracle

The test oracle is the real OrcaSlicer binary, run headlessly, that every port unit is proven
against. This document is added to by each oracle step; `oracle/fetch-release` (step 1) writes
only the two sections below.

## The asset and its hash

`just oracle-fetch` fetches and verifies this release:

| | |
|---|---|
| Release tag | `v2.5.2` |
| Release asset | `OrcaSlicer_Mac_universal_V2.5.2.dmg` |
| Recorded sha256 | `6ea1afcdffb7e9ab011e2d6f6db665cb18463d1c4774591cd5db7ee0225479d9` (`vendor/README.md` is the source of truth; this table quotes it) |
| Bundle written to | `oracle/.cache/app/OrcaSlicer.app` |
| Binary used | `oracle/.cache/app/OrcaSlicer.app/Contents/MacOS/OrcaSlicer` |

The binary is the GUI bundle's executable. It is also the CLI and runs headless with no
`DISPLAY`; there is no separate console executable in the bundle. `oracle/fetch-release` never
writes the recorded sha256 — it only reads it and fails when a download does not match it.

## What is never read

`/Applications/OrcaSlicer.app` (2.4.2) and `/Applications/BambuStudio.app` are never read by any
command in this repository. The oracle is always the pinned binary `just oracle-fetch` stages
under `oracle/.cache/`, never an app already installed on the machine running the command.
