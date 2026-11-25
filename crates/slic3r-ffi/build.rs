//! Compiles the vendored Clipper2 sources (task 29-ffi-bridge-core, plan step 17, R17) and this
//! crate's own `cxx` glue onto one shared `cxx_build::Build`, so that a later task's units land
//! on the same `Build` and inherit the same flags (`-ffp-contract=off` above all).
//!
//! `vendor/OrcaSlicer/` is never written to: every path here is read-only, and the pin
//! (`f92a7c065c5718b8ef1baac1cd704df2911238f4`) is never edited, generated into or patched.

use std::env;
use std::path::PathBuf;

/// `oracle/build-libslic3r`'s dependency superbuild install prefix -- the same default that
/// script's stage-two `cmake -DCMAKE_PREFIX_PATH=$DEPS` uses
/// (`intent/libslic3r-rust-port/tasks/13-build-libslic3r/task.md`). Resolved from one environment
/// variable for both the include path (`$prefix/include`, for
/// `<oneapi/tbb/scalable_allocator.h>`) and the link search path (`$prefix/lib`, for
/// `tbbmalloc`), so the two cannot disagree.
const DEPS_PREFIX_VAR: &str = "GRAMPUS_DEPS_PREFIX";
const DEPS_PREFIX_DEFAULT: &str = "vendor/OrcaSlicer/deps/build/arm64/OrcaSlicer_dep/usr/local";

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

fn main() {
    // Cargo runs build scripts with the crate directory as the current directory, not the
    // workspace root, so every path below is built from CARGO_MANIFEST_DIR rather than a bare
    // relative string: `crates/slic3r-ffi/../..` is the workspace root this crate always sits
    // two directories under.
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = crate_dir.join("..").join("..");

    println!("cargo:rerun-if-env-changed={DEPS_PREFIX_VAR}");
    for src in CLIPPER2_VENDORED_SOURCES {
        println!(
            "cargo:rerun-if-changed={}",
            workspace_root.join(src).display()
        );
    }
    println!(
        "cargo:rerun-if-changed={}",
        crate_dir.join("cxx/clipper2_shim.h").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        crate_dir.join("cxx/clipper2_plain.cc").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        crate_dir.join("cxx/clipper2_z.cc").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        crate_dir.join("src/clipper2.rs").display()
    );

    let deps_prefix: PathBuf = env::var_os(DEPS_PREFIX_VAR)
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root.join(DEPS_PREFIX_DEFAULT));

    // Fail with a named error rather than silently falling back to a system or Homebrew TBB: the
    // oracle build passes -DCMAKE_IGNORE_PREFIX_PATH to keep Homebrew out, so a Homebrew
    // tbbmalloc is not the library the oracle links, and any per-unit proof against it would be a
    // proof against the wrong reference. Checked against the actual library file, not just the
    // prefix directory's existence, since an empty or partial prefix would otherwise pass this
    // check and fail obscurely at the link step instead.
    let tbbmalloc_lib = deps_prefix.join("lib").join("libtbbmalloc.a");
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

    println!(
        "cargo:rustc-link-search=native={}",
        deps_prefix.join("lib").display()
    );
    println!("cargo:rustc-link-lib=static=tbbmalloc");

    let clipper2_lib_include =
        workspace_root.join("vendor/OrcaSlicer/deps_src/clipper2/Clipper2Lib/include");

    let mut build = cxx_build::bridge("src/clipper2.rs");
    // `-ffp-contract=off`: not a preference. The library this crate links and the one
    // `oracle/build-libslic3r` builds are the normative oracle for every per-unit and per-stage
    // proof only when both carry this flag (`just port-check`'s drift check fails the build
    // otherwise, by grepping for this exact call below -- not a bare substring anywhere in this
    // file, which a stray prose mention of the flag could satisfy without the flag reaching the
    // compiler). Applied to the shared `Build` before any `.file()` call, so a unit a later task
    // adds cannot miss it.
    build.flag("-ffp-contract=off");
    build
        .std("c++17")
        .include(&crate_dir)
        .include(clipper2_lib_include)
        .include(deps_prefix.join("include"))
        .file(crate_dir.join("cxx/clipper2_plain.cc"))
        .file(crate_dir.join("cxx/clipper2_z.cc"));
    for src in CLIPPER2_VENDORED_SOURCES {
        build.file(workspace_root.join(src));
    }
    build.compile("slic3r-ffi-clipper2");
}
