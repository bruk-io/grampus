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
| `vendor/OrcaSlicer/` | Upstream OrcaSlicer, unmodified, pinned. |
| `vendor/README.md` | Documentation for the vendored source. |
| `LICENSE` | AGPL-3.0 text. |

## Getting started

```sh
git submodule update --init --depth 1
```

## License

AGPL-3.0. See [LICENSE](LICENSE). grampus is derived from OrcaSlicer, which is AGPL-3.0, so it
carries the same license.
