# grampus

An incremental Rust port of OrcaSlicer's slicing engine, libslic3r.

This is a monorepo. Right now it holds only the reference source the port is measured against.

## Target

grampus targets libslic3r as shipped in **OrcaSlicer v2.5.2**
(commit `f92a7c065c5718b8ef1baac1cd704df2911238f4`), vendored as a submodule at
`vendor/OrcaSlicer`. See [vendor/README.md](vendor/README.md) for what is pinned, how the tree
is laid out, and how to change the pin.

## Layout

| Path | Contents |
|---|---|
| `crates/` | The Rust workspace: ported crates and `oracle-compare`, the L0-L4 comparator. |
| `vendor/OrcaSlicer/` | Upstream OrcaSlicer, unmodified, pinned. |
| `vendor/README.md` | Documentation for the vendored source. |
| `LICENSE` | AGPL-3.0 text. |
| `intent/` | asdlc documents: intents, specs, plans and tasks. Created by the first intent. |
| `.asdlc/checks` | The commands a task has to pass. |
| `justfile` | `just check`, `just port-check` and `just guard`. |

## How work happens

This repository uses [asdlc](https://github.com/bruk-io/asdlc). Work bigger than a small fix
starts as an intent, then a spec, a plan and tasks: Markdown files under `intent/<slug>/`, each
added by a pull request and accepted when a person merges it.

- **Plugin:** `.claude/settings.json` enables the asdlc Claude Code plugin for this project. It
  lints a document when Claude writes it and guards a branch before Claude pushes it.
- **Checks:** `.asdlc/checks` lists what a task has to pass: `just check`, which lints the asdlc
  documents, and `just port-check`, the code checks a port task passes that need neither the
  oracle binary nor a built libslic3r. Checks that need either run in CI only, in a separate job.
- **CI:** `.github/workflows/ci.yml` has an ubuntu job that runs `just check` and
  `just port-check` on every push and pull request, with a Rust toolchain step and a shallow
  submodule checkout ahead of them, and the guard on pull requests; and a macOS arm64 job for
  the checks that need the oracle binary or a built libslic3r, which is skipped for a pull
  request that changes only a document and otherwise runs on its own path filter (unconditionally
  on a push to `main`). `.github/workflows/oracle.yml` builds the vendored libslic3r itself, on
  its own narrower path filter, since that build costs on the order of an hour and a half of
  macOS runner time.
- **Merging:** pull requests are merged with a merge commit. asdlc reads task state from merge
  commits, so squash and rebase merges are turned off on GitHub.

## Getting started

```sh
git submodule update --init --depth 1
```

## License

AGPL-3.0. See [LICENSE](LICENSE). grampus is derived from OrcaSlicer, which is AGPL-3.0, so it
carries the same license.

<!-- scratch doc-only filter test, see PR #39 Done-when; will be closed -->
