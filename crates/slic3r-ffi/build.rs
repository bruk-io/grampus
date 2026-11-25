//! Compiles the vendored Clipper2 and admesh sources, the vendored (unmodified)
//! `libnest2d.cpp`, and this crate's own `cxx` glue (including the Boost.Polygon Voronoi unit)
//! onto one shared `cxx_build::Build`, so every unit inherits the same flags
//! (`-ffp-contract=off` above all). qhull is bound against the superbuild's installed copy
//! instead of a vendored source -- `qhullcpp`/`qhullstatic_r` in `ORACLE_LINK_LIBS` below; see
//! the determination in `src/lib.rs`'s crate-doc table.
//!
//! `vendor/OrcaSlicer/` is never written to: every path here is read-only, and the pin
//! (`f92a7c065c5718b8ef1baac1cd704df2911238f4`) is never edited, generated into or patched.

use std::collections::BTreeSet;
use std::env;
use std::path::{Path, PathBuf};

/// `oracle/build-libslic3r`'s dependency superbuild install prefix -- the same default that
/// script's stage-two `cmake -DCMAKE_PREFIX_PATH=$DEPS` uses
/// (`intent/libslic3r-rust-port/tasks/13-build-libslic3r/task.md`). Resolved from one environment
/// variable for both the include path (`$prefix/include`, for
/// `<oneapi/tbb/scalable_allocator.h>`) and the link search path (`$prefix/lib`, for
/// `tbbmalloc`), so the two cannot disagree.
const DEPS_PREFIX_VAR: &str = "GRAMPUS_DEPS_PREFIX";
const DEPS_PREFIX_DEFAULT: &str = "vendor/OrcaSlicer/deps/build/arm64/OrcaSlicer_dep/usr/local";

/// Step 7's own build directory, which already holds the built `libslic3r` (`liblibslic3r.a`)
/// the libnest2d bridge links, and the `Ninja Multi-Config` config name
/// `oracle/build-libslic3r` builds (`-DCMAKE_BUILD_TYPE=Release`, `cmake --build ... --config
/// Release`).
const ORACLE_BUILD_DIR: &str = "vendor/OrcaSlicer/build/arm64";
const ORACLE_BUILD_CONFIG: &str = "Release";

/// The five sources `deps_src/clipper2/CMakeLists.txt:26-32` names -- exactly these, unmodified,
/// at the pin. `clipper2_z.cpp` `#define`s `USINGZ` itself and `#include`s the other four
/// (`Clipper2Lib/src/clipper2_z.cpp:1-9`), which is why compiling all five together is not a
/// duplicate-symbol error and why `USINGZ` is never defined on this command line.
const CLIPPER2_VENDORED_SOURCES: &[&str] = &[
    "vendor/OrcaSlicer/deps_src/clipper2/Clipper2Lib/src/clipper.engine.cpp",
    "vendor/OrcaSlicer/deps_src/clipper2/Clipper2Lib/src/clipper.offset.cpp",
    "vendor/OrcaSlicer/deps_src/clipper2/Clipper2Lib/src/clipper.rectclip.cpp",
    "vendor/OrcaSlicer/deps_src/clipper2/Clipper2Lib/src/clipper.triangulation.cpp",
    "vendor/OrcaSlicer/deps_src/clipper2/Clipper2Lib/src/clipper2_z.cpp",
];

/// The six sources `deps_src/admesh/CMakeLists.txt:4-12` names -- exactly these, unmodified.
const ADMESH_VENDORED_SOURCES: &[&str] = &[
    "vendor/OrcaSlicer/deps_src/admesh/connect.cpp",
    "vendor/OrcaSlicer/deps_src/admesh/normals.cpp",
    "vendor/OrcaSlicer/deps_src/admesh/shared.cpp",
    "vendor/OrcaSlicer/deps_src/admesh/stl_io.cpp",
    "vendor/OrcaSlicer/deps_src/admesh/stlinit.cpp",
    "vendor/OrcaSlicer/deps_src/admesh/util.cpp",
];

/// The vendored, unmodified libnest2d translation unit (`deps_src/libnest2d/CMakeLists.txt:21`).
const LIBNEST2D_VENDORED_SOURCE: &str = "vendor/OrcaSlicer/deps_src/libnest2d/src/libnest2d.cpp";

/// The four compile definitions the vendored target makes `PUBLIC`
/// (`deps_src/libnest2d/CMakeLists.txt:28`), applied to the whole shared `Build` the same way
/// the oracle build applies them to every target depending on `libnest2d` (they show up on
/// `ClipperUtils.cpp.o`'s own `DEFINES` line in `CMakeFiles/impl-Release.ninja`, inherited
/// through `target_link_libraries(libslic3r PUBLIC ... libnest2d ...)`).
const LIBNEST2D_DEFINES: &[&str] = &[
    "LIBNEST2D_THREADING_tbb",
    "LIBNEST2D_STATIC",
    "LIBNEST2D_OPTIMIZER_nlopt",
    "LIBNEST2D_GEOMETRIES_libslic3r",
];

/// Where one of [`ORACLE_LINK_LIBS`]'s archives lives.
enum LibLoc {
    /// `$DEPS_PREFIX/lib/lib<name>.a` -- the superbuild's own install.
    DepsPrefixLib,
    /// `$DEPS_PREFIX/lib/<subdir>/lib<name>.a`.
    DepsPrefixLibSub(&'static str),
    /// `$ORACLE_BUILD_DIR/<subdir>/$ORACLE_BUILD_CONFIG/lib<name>.a` -- step 7's own build
    /// output.
    OracleBuild(&'static str),
}

/// Step 7's full link line for an executable linking `libslic3r`
/// (`CXX_EXECUTABLE_LINKER__libslic3r_tests_Release` in
/// `vendor/OrcaSlicer/build/arm64/CMakeFiles/impl-Release.ninja`), in the same order, with three
/// kinds of entry pruned because this crate provides an equivalent itself rather than linking
/// the oracle's copy: Clipper2 (`clipper2.rs`'s own `CLIPPER2_VENDORED_SOURCES`, task
/// 29-ffi-bridge-core), admesh (`ADMESH_VENDORED_SOURCES` above), and libnest2d
/// (`LIBNEST2D_VENDORED_SOURCE` above -- its own compiled object, not `liblibnest2d.a`, is what
/// actually makes it into this crate's archive). Catch2's two archives (the oracle's own test
/// framework) are dropped too; nothing here depends on it. `liblibslic3r.a` is kept exactly where
/// the oracle's own list repeats it.
const ORACLE_LINK_LIBS: &[(LibLoc, &str)] = &[
    (LibLoc::OracleBuild("src/libslic3r"), "libslic3r"),
    (LibLoc::DepsPrefixLib, "draco"),
    (LibLoc::OracleBuild("src/libslic3r"), "libslic3r"),
    (LibLoc::OracleBuild("deps_src/miniz"), "miniz_static"),
    (LibLoc::DepsPrefixLib, "opencv_world"),
    (LibLoc::DepsPrefixLibSub("opencv4/3rdparty"), "tegra_hal"),
    (LibLoc::DepsPrefixLib, "jpeg"),
    (LibLoc::DepsPrefixLib, "png"),
    (LibLoc::DepsPrefixLib, "assimp"),
    (LibLoc::DepsPrefixLib, "expat"),
    (LibLoc::DepsPrefixLibSub("occt"), "TKOffset"),
    (LibLoc::DepsPrefixLibSub("occt"), "TKFillet"),
    (LibLoc::DepsPrefixLibSub("occt"), "TKBool"),
    (LibLoc::DepsPrefixLibSub("occt"), "TKDESTEP"),
    (LibLoc::DepsPrefixLibSub("occt"), "TKDE"),
    (LibLoc::DepsPrefixLibSub("occt"), "TKXCAF"),
    (LibLoc::DepsPrefixLibSub("occt"), "TKXSBase"),
    (LibLoc::DepsPrefixLibSub("occt"), "TKVCAF"),
    (LibLoc::DepsPrefixLibSub("occt"), "TKCAF"),
    (LibLoc::DepsPrefixLibSub("occt"), "TKLCAF"),
    (LibLoc::DepsPrefixLibSub("occt"), "TKCDF"),
    (LibLoc::DepsPrefixLibSub("occt"), "TKV3d"),
    (LibLoc::DepsPrefixLibSub("occt"), "TKService"),
    (LibLoc::DepsPrefixLibSub("occt"), "TKMesh"),
    (LibLoc::DepsPrefixLibSub("occt"), "TKBO"),
    (LibLoc::DepsPrefixLibSub("occt"), "TKPrim"),
    (LibLoc::DepsPrefixLibSub("occt"), "TKHLR"),
    (LibLoc::DepsPrefixLibSub("occt"), "TKShHealing"),
    (LibLoc::DepsPrefixLibSub("occt"), "TKTopAlgo"),
    (LibLoc::DepsPrefixLibSub("occt"), "TKGeomAlgo"),
    (LibLoc::DepsPrefixLibSub("occt"), "TKBRep"),
    (LibLoc::DepsPrefixLibSub("occt"), "TKGeomBase"),
    (LibLoc::DepsPrefixLibSub("occt"), "TKG3d"),
    (LibLoc::DepsPrefixLibSub("occt"), "TKG2d"),
    (LibLoc::DepsPrefixLibSub("occt"), "TKMath"),
    (LibLoc::DepsPrefixLibSub("occt"), "TKernel"),
    (LibLoc::DepsPrefixLib, "draco"),
    (LibLoc::OracleBuild("deps_src/glu-libtess"), "glu-libtess"),
    (LibLoc::DepsPrefixLib, "jpeg"),
    (LibLoc::OracleBuild("src/libslic3r"), "libslic3r_cgal"),
    (LibLoc::DepsPrefixLib, "boost_log_setup"),
    (LibLoc::DepsPrefixLib, "boost_log"),
    (LibLoc::DepsPrefixLib, "boost_filesystem"),
    (LibLoc::DepsPrefixLib, "boost_coroutine"),
    (LibLoc::DepsPrefixLib, "boost_context"),
    (LibLoc::DepsPrefixLib, "boost_serialization"),
    (LibLoc::DepsPrefixLib, "boost_locale"),
    (LibLoc::DepsPrefixLib, "boost_thread"),
    (LibLoc::DepsPrefixLib, "boost_chrono"),
    (LibLoc::DepsPrefixLib, "boost_atomic"),
    (LibLoc::DepsPrefixLib, "boost_date_time"),
    (LibLoc::DepsPrefixLib, "boost_exception"),
    (LibLoc::DepsPrefixLib, "boost_program_options"),
    (LibLoc::DepsPrefixLib, "boost_container"),
    (LibLoc::DepsPrefixLib, "boost_nowide"),
    (LibLoc::DepsPrefixLib, "gmpxx"),
    (LibLoc::DepsPrefixLib, "mpfr"),
    (LibLoc::DepsPrefixLib, "gmp"),
    (LibLoc::OracleBuild("deps_src/mcut"), "mcut"),
    (LibLoc::DepsPrefixLib, "libnoise_static"),
    (LibLoc::DepsPrefixLib, "png"),
    (LibLoc::DepsPrefixLib, "qhullcpp"),
    (LibLoc::DepsPrefixLib, "qhullstatic_r"),
    (LibLoc::OracleBuild("deps_src/qoi"), "qoi"),
    (LibLoc::OracleBuild("lib"), "semver"),
    // libtbbmalloc is linked once already, below, for Clipper2's own tbbmalloc check.
    (LibLoc::DepsPrefixLib, "crypto"),
    (LibLoc::DepsPrefixLib, "openvdb"),
    (LibLoc::DepsPrefixLib, "boost_iostreams"),
    (LibLoc::DepsPrefixLib, "boost_random"),
    (LibLoc::DepsPrefixLib, "Half-2_5"),
    (LibLoc::DepsPrefixLib, "blosc"),
    (LibLoc::DepsPrefixLib, "slvs"),
    (LibLoc::DepsPrefixLib, "tbb"),
    (LibLoc::DepsPrefixLib, "nlopt"),
];

/// A directory `build.rs` must never pass as an include path: the oracle build itself passes
/// `-DCMAKE_IGNORE_PREFIX_PATH=/opt/local;/usr/local;/opt/homebrew` to keep a Homebrew or
/// MacPorts copy of a dependency from being picked up instead of the superbuild's own, so a
/// per-unit or per-stage proof never silently compares against a locally installed library
/// instead of the pinned one. Checked by path-component prefix, not by substring: the
/// superbuild prefix itself is `.../OrcaSlicer_dep/usr/local`, which *contains* the substring
/// "/usr/local" without *starting* with it.
const DISALLOWED_INCLUDE_PREFIXES: &[&str] = &["/usr/local", "/opt/homebrew", "/opt/local"];

fn is_disallowed_include(path: &Path) -> bool {
    let Ok(canon) = path.canonicalize() else {
        return false;
    };
    DISALLOWED_INCLUDE_PREFIXES
        .iter()
        .any(|prefix| canon.starts_with(prefix))
}

fn main() {
    // Cargo runs build scripts with the crate directory as the current directory, not the
    // workspace root, so every path below is built from CARGO_MANIFEST_DIR rather than a bare
    // relative string: `crates/slic3r-ffi/../..` is the workspace root this crate always sits
    // two directories under.
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = crate_dir.join("..").join("..");

    println!("cargo:rerun-if-env-changed={DEPS_PREFIX_VAR}");
    for src in CLIPPER2_VENDORED_SOURCES
        .iter()
        .chain(ADMESH_VENDORED_SOURCES)
        .chain(std::iter::once(&LIBNEST2D_VENDORED_SOURCE))
    {
        println!(
            "cargo:rerun-if-changed={}",
            workspace_root.join(src).display()
        );
    }
    for glue in [
        "cxx/clipper2_shim.h",
        "cxx/clipper2_plain.cc",
        "cxx/clipper2_z.cc",
        "src/clipper2.rs",
        "cxx/admesh_shim.h",
        "cxx/admesh_glue.cc",
        "src/admesh.rs",
        "cxx/qhull_shim.h",
        "cxx/qhull_glue.cc",
        "src/qhull.rs",
        "cxx/libnest2d_shim.h",
        "cxx/libnest2d_glue.cc",
        "src/nest2d.rs",
        "cxx/voronoi_shim.h",
        "cxx/voronoi_glue.cc",
        "src/voronoi.rs",
    ] {
        println!("cargo:rerun-if-changed={}", crate_dir.join(glue).display());
    }

    let deps_prefix: PathBuf = env::var_os(DEPS_PREFIX_VAR)
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root.join(DEPS_PREFIX_DEFAULT));
    let deps_prefix_lib = deps_prefix.join("lib");
    let oracle_build_dir = workspace_root.join(ORACLE_BUILD_DIR);

    // Fail with a named error rather than silently falling back to a system or Homebrew TBB: the
    // oracle build passes -DCMAKE_IGNORE_PREFIX_PATH to keep Homebrew out, so a Homebrew
    // tbbmalloc is not the library the oracle links, and any per-unit proof against it would be a
    // proof against the wrong reference. Checked against the actual library file, not just the
    // prefix directory's existence, since an empty or partial prefix would otherwise pass this
    // check and fail obscurely at the link step instead.
    let tbbmalloc_lib = deps_prefix_lib.join("libtbbmalloc.a");
    if !tbbmalloc_lib.is_file() {
        panic!(
            "slic3r-ffi build.rs: no libtbbmalloc.a under {:?} (from {DEPS_PREFIX_VAR}, default \
             {DEPS_PREFIX_DEFAULT}). Run `just oracle-build` first, or set {DEPS_PREFIX_VAR} to \
             the dependency superbuild's install prefix \
             (deps/build/arm64/OrcaSlicer_dep/usr/local).",
            tbbmalloc_lib
        );
    }
    let tbb_scalable_allocator_header = deps_prefix
        .join("include")
        .join("oneapi")
        .join("tbb")
        .join("scalable_allocator.h");
    if !tbb_scalable_allocator_header.is_file() {
        panic!(
            "slic3r-ffi build.rs: no oneapi/tbb/scalable_allocator.h under {:?} (from \
             {DEPS_PREFIX_VAR}, default {DEPS_PREFIX_DEFAULT}); clipper.engine.cpp:14 needs it.",
            deps_prefix.join("include")
        );
    }

    // The built libslic3r this crate's libnest2d bridge links -- step 7's own output. Checked
    // the same way as tbbmalloc above: against the actual archive, not just the build directory's
    // existence.
    let libslic3r_archive = oracle_build_dir
        .join("src/libslic3r")
        .join(ORACLE_BUILD_CONFIG)
        .join("liblibslic3r.a");
    if !libslic3r_archive.is_file() {
        panic!(
            "slic3r-ffi build.rs: no liblibslic3r.a under {:?}. Run `just oracle-build` first \
             (task 13-build-libslic3r) -- this crate's libnest2d bridge links step 7's built \
             libslic3r, it does not rebuild it.",
            libslic3r_archive
        );
    }

    println!(
        "cargo:rustc-link-search=native={}",
        deps_prefix_lib.display()
    );
    println!("cargo:rustc-link-lib=static=tbbmalloc");
    // Two system dylibs the oracle link line pulls from the macOS SDK sysroot
    // (`libz.tbd`/`libiconv.tbd` under the Xcode SDK, not under any of
    // `DISALLOWED_INCLUDE_PREFIXES` -- those are include-path prefixes, this is a link library),
    // resolved the ordinary way through the linker's default system search path rather than by
    // pointing at the SDK's `.tbd` stub file directly.
    println!("cargo:rustc-link-lib=dylib=z");
    println!("cargo:rustc-link-lib=dylib=iconv");

    let mut search_dirs_emitted: BTreeSet<PathBuf> = BTreeSet::new();
    search_dirs_emitted.insert(deps_prefix_lib.clone());
    for (loc, name) in ORACLE_LINK_LIBS {
        let dir = match loc {
            LibLoc::DepsPrefixLib => deps_prefix_lib.clone(),
            LibLoc::DepsPrefixLibSub(sub) => deps_prefix_lib.join(sub),
            LibLoc::OracleBuild(sub) => oracle_build_dir.join(sub).join(ORACLE_BUILD_CONFIG),
        };
        if search_dirs_emitted.insert(dir.clone()) {
            println!("cargo:rustc-link-search=native={}", dir.display());
        }
        println!("cargo:rustc-link-lib=static={name}");
    }

    let clipper2_lib_include =
        workspace_root.join("vendor/OrcaSlicer/deps_src/clipper2/Clipper2Lib/include");
    let admesh_include = workspace_root.join("vendor/OrcaSlicer/deps_src/admesh");
    let admesh_parent_include = workspace_root.join("vendor/OrcaSlicer/deps_src");
    let libnest2d_include = workspace_root.join("vendor/OrcaSlicer/deps_src/libnest2d/include");
    let libslic3r_src_include = workspace_root.join("vendor/OrcaSlicer/src");
    let deps_prefix_include = deps_prefix.join("include");
    // Eigen installs itself one level deeper than the rest of the superbuild
    // (`$prefix/include/eigen3/Eigen/...`, not `$prefix/include/Eigen/...` -- see the
    // `-isystem .../include/eigen3` entry on `ClipperUtils.cpp.o`'s own `INCLUDES` line in
    // `CMakeFiles/impl-Release.ninja`), which `admesh/stl.h:31`'s `#include <Eigen/Geometry>`
    // needs.
    let deps_prefix_eigen3_include = deps_prefix_include.join("eigen3");
    // `libslic3r.h:4`'s `#include "libslic3r_version.h"` is a CMake-generated file that lands in
    // the build directory, not the source tree (`CMAKE_CURRENT_BINARY_DIR`, the same `-I
    // .../build/arm64/src/libslic3r` entry on `ClipperUtils.cpp.o`'s own `INCLUDES` line) --
    // step 7's own build already generated it, this crate does not.
    let libslic3r_build_include = oracle_build_dir.join("src/libslic3r");

    // Checkable from this list, which the Done-when condition asks for: every include directory
    // this build adds to the shared `Build`, printed so `cargo build -p slic3r-ffi -vv`'s output
    // (or this list itself) shows it, and asserted here to carry none of
    // `DISALLOWED_INCLUDE_PREFIXES` -- the superbuild prefix's own path ends in `usr/local` but
    // does not *start* with `/usr/local`, so it passes.
    let include_dirs = [
        crate_dir.clone(),
        clipper2_lib_include.clone(),
        admesh_include.clone(),
        admesh_parent_include.clone(),
        libnest2d_include.clone(),
        libslic3r_src_include.clone(),
        deps_prefix_include.clone(),
        deps_prefix_eigen3_include.clone(),
        libslic3r_build_include.clone(),
    ];
    for dir in &include_dirs {
        println!(
            "cargo:warning=slic3r-ffi build.rs include path: {}",
            dir.display()
        );
        if is_disallowed_include(dir) {
            panic!(
                "slic3r-ffi build.rs: include directory {:?} is under a prefix the oracle build \
                 ignores (-DCMAKE_IGNORE_PREFIX_PATH=/opt/local;/usr/local;/opt/homebrew) -- a \
                 per-unit proof against it would not be comparing against the pinned library.",
                dir
            );
        }
    }

    let mut build = cxx_build::bridges([
        "src/clipper2.rs",
        "src/admesh.rs",
        "src/qhull.rs",
        "src/nest2d.rs",
        "src/voronoi.rs",
    ]);
    // `-ffp-contract=off`: not a preference. The library this crate links and the one
    // `oracle/build-libslic3r` builds are the normative oracle for every per-unit and per-stage
    // proof only when both carry this flag (`just port-check`'s drift check fails the build
    // otherwise, by grepping for this exact call below -- not a bare substring anywhere in this
    // file, which a stray prose mention of the flag could satisfy without the flag reaching the
    // compiler). Applied to the shared `Build` before any `.file()` call, so a unit a later task
    // adds cannot miss it.
    build.flag("-ffp-contract=off");
    for define in LIBNEST2D_DEFINES {
        build.define(define, None);
    }
    build
        .std("c++17")
        .include(&crate_dir)
        .include(&libslic3r_build_include);
    // Every vendor/superbuild/dependency directory goes on the path as `-isystem`, not plain
    // `-I` -- exactly how the oracle build itself puts them on `libnest2d.cpp.o`'s command line
    // (`CMakeFiles/impl-Release.ninja`'s `INCLUDES` line). This is not cosmetic: clang suppresses
    // diagnostics raised *inside* a system header, and without it this exact header chain
    // (`libnest2d.hpp` -> `geometries.hpp` -> `utils/boost_alg.hpp` -> `boost/geometry.hpp` ->
    // `boost/qvm/quat_traits.hpp`) fails to compile under this toolchain
    // (`-Wmissing-template-arg-list-after-template-kw`) when included as an ordinary `-I` --
    // confirmed by reproducing the error with `-I` and clearing it by switching to `-isystem`,
    // not guessed.
    for dir in [
        &clipper2_lib_include,
        &admesh_include,
        &admesh_parent_include,
        &libnest2d_include,
        &libslic3r_src_include,
        &deps_prefix_include,
        &deps_prefix_eigen3_include,
    ] {
        build.flag(&format!("-isystem{}", dir.display()));
    }
    build
        .file(crate_dir.join("cxx/clipper2_plain.cc"))
        .file(crate_dir.join("cxx/clipper2_z.cc"))
        .file(crate_dir.join("cxx/admesh_glue.cc"))
        .file(crate_dir.join("cxx/qhull_glue.cc"))
        .file(crate_dir.join("cxx/libnest2d_glue.cc"))
        .file(crate_dir.join("cxx/voronoi_glue.cc"));
    for src in CLIPPER2_VENDORED_SOURCES
        .iter()
        .chain(ADMESH_VENDORED_SOURCES)
        .chain(std::iter::once(&LIBNEST2D_VENDORED_SOURCE))
    {
        build.file(workspace_root.join(src));
    }
    build.compile("slic3r-ffi-bridges");
}
