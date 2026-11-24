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
| `justfile` | `just check` and `just guard`. |

## How work happens

This repository uses [asdlc](https://github.com/bruk-io/asdlc). Work bigger than a small fix
starts as an intent, then a spec, a plan and tasks: Markdown files under `intent/<slug>/`, each
added by a pull request and accepted when a person merges it.

- **Plugin:** `.claude/settings.json` enables the asdlc Claude Code plugin for this project. It
  lints a document when Claude writes it and guards a branch before Claude pushes it.
- **Checks:** `.asdlc/checks` lists what a task has to pass. Today that is `just check`, which
  lints the asdlc documents. Code checks get added to the `check` recipe as code arrives.
- **CI:** `.github/workflows/ci.yml` runs `just check` on every push and pull request, and the
  guard on pull requests.
- **Merging:** pull requests are merged with a merge commit. asdlc reads task state from merge
  commits, so squash and rebase merges are turned off on GitHub.

## Getting started

```sh
git submodule update --init --depth 1
```

## License

AGPL-3.0. See [LICENSE](LICENSE). grampus is derived from OrcaSlicer, which is AGPL-3.0, so it
carries the same license.
