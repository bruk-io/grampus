# Vendored sources

## OrcaSlicer

`vendor/OrcaSlicer` is a git submodule of the upstream OrcaSlicer repository. It is the
reference implementation grampus ports from, and the source the test oracle is built from.
Nothing in it is modified.

| | |
|---|---|
| Upstream | https://github.com/OrcaSlicer/OrcaSlicer (formerly `SoftFever/OrcaSlicer`) |
| Pinned tag | `v2.5.2` (released 2026-11-21) |
| Pinned commit | `f92a7c065c5718b8ef1baac1cd704df2911238f4` |
| License | AGPL-3.0 (`vendor/OrcaSlicer/LICENSE.txt`) |

### What "libslic3r version" means here

libslic3r is not a separately released library. It lives in-tree at
`vendor/OrcaSlicer/src/libslic3r` and has no version of its own, so the version grampus
targets is **the OrcaSlicer commit above**. Two version strings exist in
`vendor/OrcaSlicer/version.inc`:

- `SoftFever_VERSION "2.5.2"`: the OrcaSlicer release.
- `SLIC3R_VERSION "02.08.01.55"`: inherited from the Bambu Studio lineage. It is written into
  project files but does not identify a libslic3r release.

Any golden file or oracle output must record the pinned commit, not either string.

### Why the whole repository, not just `src/libslic3r`

libslic3r cannot be built on its own. Its CMake target needs the bundled libraries in
`deps_src/`, the dependency superbuild in `deps/`, and the top-level `CMakeLists.txt` and
`cmake/` modules. The upstream test suites and test models are also outside `src/libslic3r`.

### Getting it

The submodule is marked shallow, so a fresh clone only fetches the pinned commit:

```sh
git clone --recurse-submodules --shallow-submodules <grampus-url>
# or, in an existing clone:
git submodule update --init --depth 1
```

The checkout is about 570 MB (130 MB of git data, the rest working tree).

### Layout

Paths are relative to `vendor/OrcaSlicer`.

| Path | Contents |
|---|---|
| `src/libslic3r/` | The slicing engine. The port target. |
| `src/slic3r/` | wxWidgets GUI. Out of scope. |
| `src/libvgcode/` | G-code viewer library used by the GUI. Out of scope. |
| `src/OrcaSlicer.cpp` | Application and CLI entry point. |
| `deps_src/` | Small libraries built in-tree: `clipper2`, `admesh`, `libnest2d`, `libigl`, `mcut`, `qhull`, `miniz`, `nlohmann`, and others. |
| `deps/` | Superbuild recipes for large external libraries: Boost, TBB, CGAL, OpenVDB, OCCT, OpenCV, Eigen, and others. |
| `tests/libslic3r/`, `tests/fff_print/` | Upstream Catch2 suites that drive libslic3r directly. The template for the oracle. |
| `tests/data/` | Small test models. |
| `resources/profiles/` | Printer, filament and process profiles (JSON). |
| `version.inc` | Version strings. |

### libslic3r at this pin

- **Size:** 605 C/C++ source files, about 340,000 lines.
- **Subsystems** (subdirectories): `Algorithm`, `Arachne`, `CAD`, `CSGMesh`, `Execution`,
  `Feature`, `Fill`, `Format`, `GCode`, `Geometry`, `Optimize`, `SLA`, `Support`,
  `TextureBake`, `TextureToColor`.
- **Coordinates:** `coord_t` is `int64_t` (`libslic3r.h`). The scaling factor is a runtime
  global, `SCALING_FACTOR`: `1e-6` mm per unit normally, `1e-5` for large printers.
- **Polygon clipping:** Clipper2 (`deps_src/clipper2`), wrapped by `ClipperUtils.hpp`.
- **Build targets:** `libslic3r` and `libslic3r_cgal`, both static libraries, defined in
  `src/libslic3r/CMakeLists.txt`.
- **Linked libraries:** Eigen, admesh, libigl, libnest2d, miniz, OpenCV, assimp, Boost, cereal,
  Clipper2, Draco, glu-libtess, JPEG, PNG, CGAL, mcut, libnoise, qhull, qoi, semver, TBB, zlib,
  OpenSSL, OCCT, EXPAT, OpenVDB, SLVS.

### Pipeline steps

From `src/libslic3r/Print.hpp`. These are the stage boundaries the oracle dumps at.

Per object (`PrintObjectStep`), in order: `posSlice`, `posPerimeters`,
`posEstimateCurledExtrusions`, `posPrepareInfill`, `posInfill`, `posIroning`, `posContouring`,
`posSupportMaterial`, `posSimplifyPath`, `posSimplifySupportPath`,
`posDetectOverhangsForLift`, `posSimplifyWall`, `posSimplifyInfill`.

Per print (`PrintStep`): `psWipeTower` (also tool ordering), `psSkirtBrim`, `psGCodeExport`,
`psConflictCheck`.

### Relevant upstream build options

From the top-level `CMakeLists.txt`:

| Option | Default | Meaning |
|---|---|---|
| `SLIC3R_GUI` | on | Build the GUI. Whether libslic3r builds cleanly with this off is untested here. |
| `SLIC3R_CAD` | on | Parametric CAD tab; pulls in more of OCCT. |
| `BUILD_TESTS` | off | Build the Catch2 suites under `tests/`. |
| `ORCA_TOOLS` | off | Build upstream developer tools. |

`build_release_macos.sh` is the upstream macOS build script.

### Changing the pin

Moving the pin changes what "correct" means for every oracle comparison, so treat it as a
deliberate migration:

1. `git -C vendor/OrcaSlicer fetch --depth 1 origin tag <new-tag>`
2. `git -C vendor/OrcaSlicer checkout <new-tag>`
3. Update the tag, date and commit in the table above, and re-check the "libslic3r at this
   pin" and "Pipeline steps" sections against the new tree.
4. Regenerate all oracle outputs and review the differences.
5. Commit the submodule pointer and this file together.

### Oracle release asset

`oracle/fetch-release` (R1) does not build OrcaSlicer from this submodule. It downloads the
matching v2.5.2 release binary and verifies it against the value recorded here before using it
as the test oracle, so the check is not circular:

| | |
|---|---|
| Release tag | `v2.5.2` |
| Release asset | `OrcaSlicer_Mac_universal_V2.5.2.dmg` |
| Recorded sha256 | `6ea1afcdffb7e9ab011e2d6f6db665cb18463d1c4774591cd5db7ee0225479d9` |

That sha256 was measured on a download of that asset during this intent's research. It is the
one value `oracle/fetch-release` reads; the script never rewrites it, and a mismatch fails the
command instead. See `docs/ORACLE.md` for how the fetched binary is used.

### Fixtures

`fixtures/` holds three comparison models, each exported from this submodule by the pinned
oracle and then translated in X/Y to the centre of the Bambu Lab X1 Carbon's printable area by
`oracle/export-fixture` (`just fixture-export`), so both the port and the oracle slice the same
committed file at the same, already-resolved placement (R24, R25). A fixture is added only by
committing its row below, alongside its STL; `oracle/assert-placement --list` enumerates exactly
these rows.

| Frozen basename | Vendored path | Resolved instance transform | License |
|---|---|---|---|
| `3DBenchy.stl` | `vendor/OrcaSlicer/resources/handy_models/3DBenchy.drc` | identity | CC0 1.0, per 3DBenchy.com, the model's publisher. Upstream records no license of its own for `resources/handy_models/`: that directory has no license file. |
| `20mm_cube.stl` | `vendor/OrcaSlicer/tests/data/20mm_cube.obj` | identity | No separate license is stated for this file; it ships as upstream OrcaSlicer test data and is covered by this submodule's own AGPL-3.0 (see "Licensing" below). |
| `frog_legs.stl` | `vendor/OrcaSlicer/tests/data/frog_legs.obj` | identity | No separate license is stated for this file; it ships as upstream OrcaSlicer test data and is covered by this submodule's own AGPL-3.0 (see "Licensing" below). |

"identity" means the committed STL already sits at its final plate position: the oracle's only
remaining placement step, the Z-only `ModelObject::ensure_on_bed`, is a no-op on it, and no
instance transform is applied on top (R24).

### Licensing

OrcaSlicer is AGPL-3.0. Code in grampus written by reading or translating this source is a
derivative work and has to be AGPL-3.0 as well.
