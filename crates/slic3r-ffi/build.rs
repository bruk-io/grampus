//! Compiles the vendored Clipper2 sources (task 29-ffi-bridge-core, plan step 17, R17) and this
//! crate's own `cxx` glue, plus the per-unit proof harness's two shims (task 31-proof-harness,
//! plan step 18, R19: `cxx/proof_shims.cpp` and `cxx/include_vendored.cpp`), onto one shared
//! `cxx_build::Build`, so every unit lands on the same `Build` and inherits the same flags
//! (`-ffp-contract=off` above all).
//!
//! `vendor/OrcaSlicer/` is never written to: every path here is read-only, and the pin
//! (`f92a7c065c5718b8ef1baac1cd704df2911238f4`) is never edited, generated into or patched.

use std::collections::{BTreeSet, HashMap};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

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

/// task 31-proof-harness: where `just oracle-build` (task 13-build-libslic3r) puts the Ninja
/// Multi-Config build tree proof paths 1 and 3 link against. Overridable for the same reason
/// `DEPS_PREFIX_VAR` is above -- never hardcode a path that exists on one machine only.
const ORACLE_BUILD_DIR_VAR: &str = "GRAMPUS_ORACLE_BUILD_DIR";
const ORACLE_BUILD_DIR_DEFAULT: &str = "vendor/OrcaSlicer/build/arm64";

/// CMake's "Ninja Multi-Config" generator (plan step 7) splits a build tree into `build.ninja`
/// (default-config `phony` aliases only), `build-<Config>.ninja` (an `include` of the real file)
/// and `CMakeFiles/impl-<Config>.ninja` (the actual compile/link edges, with their `FLAGS`,
/// `INCLUDES`, `LINK_LIBRARIES` and `LINK_FLAGS` variables) -- verified empirically against this
/// pin's CMake 4.1 output (see this task's pull request body): `build.ninja` and
/// `build-Release.ninja` hold only a `phony` alias for `libslic3r_tests`/`fff_print_tests`, never
/// the edge the task's own text names first. Checked in this order (not a single hardcoded name)
/// because the real edge could in principle live in any of them on a different CMake version,
/// and this still never hardcodes a *library* name -- only this generator's known file layout.
const NINJA_FILES_TO_SEARCH: &[&str] = &[
    "CMakeFiles/impl-Release.ninja",
    "build-Release.ninja",
    "build.ninja",
];

/// The two targets the task names, in its own order: `libslic3r_tests` first,
/// `fff_print_tests` as the fallback.
const LINK_LINE_TARGETS: &[&str] = &["libslic3r_tests", "fff_print_tests"];

/// A token containing either of these is dropped from the derived link line. The Clipper2
/// archive: this crate already compiles the same five sources itself (above), so linking the
/// build tree's archive too would be a duplicate symbol -- the crate's own compilation wins.
/// Catch2's own archives: they exist only to give the oracle's `libslic3r_tests`/
/// `fff_print_tests` binary a `main()` and its assertion macros, neither of which this crate's
/// proof shims use.
const DROPPED_LINK_SUBSTRINGS: &[&str] = &["deps_src/clipper2/", "tests/catch2/"];

/// Finds every line in `content` containing `needle` and, for each, collects the following
/// `  NAME = value` lines -- CMake Ninja's own indentation for a build edge's variables -- up to
/// the first line that is not indented that way (the next build edge, or a blank line). A CMake
/// that emits the rule declaration (`rule CXX_EXECUTABLE_LINKER__<target>_Release`, no
/// `LINK_LIBRARIES`) into the same per-config file as the build edge (which does carry
/// `LINK_LIBRARIES`) would otherwise make the first match -- the rule block -- look like the
/// whole story, so every match is collected and the caller picks the one that actually has the
/// variable it needs. Purely textual: this crate already parses `oracle/build-libslic3r`'s and
/// its own `CXX_FLAGS`/`build.flag(...)` lines the same way (`just port-check`'s drift check),
/// never a CMake or Ninja grammar.
fn ninja_edge_vars_all(content: &str, needle: &str) -> Vec<HashMap<String, String>> {
    let lines: Vec<&str> = content.lines().collect();
    let mut blocks = Vec::new();
    for start in lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.contains(needle))
        .map(|(i, _)| i)
    {
        let mut vars = HashMap::new();
        for line in &lines[start + 1..] {
            if !line.starts_with("  ") {
                break;
            }
            if let Some(eq) = line.find(" = ") {
                let name = line[2..eq].trim().to_string();
                let value = line[eq + 3..].to_string();
                vars.insert(name, value);
            }
        }
        blocks.push(vars);
    }
    blocks
}

/// Searches `NINJA_FILES_TO_SEARCH` under `build_dir` for the Release-config link edge of
/// `libslic3r_tests`, falling back to `fff_print_tests`. Returns the target name found, its
/// `LINK_LIBRARIES` value, and the file it was read from (for `cargo:rerun-if-changed` and the
/// build-time note naming where this derivation actually read its link line from). Keeps the
/// first block, among every occurrence of the needle in the file, that actually carries a
/// `LINK_LIBRARIES` variable -- not just the first occurrence of the needle itself, which may be
/// the `rule` declaration rather than the build edge.
fn find_link_libraries(build_dir: &Path) -> Option<(&'static str, String, PathBuf)> {
    for rel in NINJA_FILES_TO_SEARCH {
        let path = build_dir.join(rel);
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        for target in LINK_LINE_TARGETS {
            let needle = format!("CXX_EXECUTABLE_LINKER__{target}_Release");
            for vars in ninja_edge_vars_all(&content, &needle) {
                if let Some(libs) = vars.get("LINK_LIBRARIES") {
                    return Some((target, libs.clone(), path));
                }
            }
        }
    }
    None
}

/// Emits the `cargo:rustc-link-search`/`cargo:rustc-link-lib` directives the derived
/// `LINK_LIBRARIES` string implies, token by token:
/// - a static archive (`.a`) or Apple's text-based dylib stub (`.tbd`) becomes a search path
///   (its parent directory, deduplicated) plus a named lib -- `static=` for `.a`, bare (dylib,
///   the default kind) for `.tbd`;
/// - `-framework X` becomes `cargo:rustc-link-lib=framework=X`, never a raw
///   `cargo:rustc-link-arg=-framework`: a plain `rustc-link-arg` only reaches this package's own
///   binary targets, which this library crate has none of, while `rustc-link-lib` (every kind,
///   `framework` included) propagates to a dependent crate's test binary the way linking a C
///   library from a build script always has;
/// - a bare `-lNAME` becomes `cargo:rustc-link-lib=NAME`, which is how `-lc++` here carries the
///   C++ standard library onto the link without this crate hardcoding "c++" vs. "stdc++" itself;
/// - anything else is warned about rather than silently dropped: this list is derived, not
///   enumerated, so an unrecognised shape is exactly what a future libslic3r dependency change
///   would produce, and the build must not go quietly wrong.
///
/// `LINK_FLAGS` (`-Wl,-search_paths_first`, `-Wl,-headerpad_max_install_names`) is deliberately
/// not derived: both are policy flags for ld64's library search order and later
/// `install_name_tool` edits, neither of which this crate's own, purely static link needs, and
/// `cargo:rustc-link-arg` would not propagate them past this package's own (non-existent) binary
/// targets regardless.
fn emit_link_directives(build_dir: &Path, link_libraries: &str) {
    let mut search_dirs: BTreeSet<PathBuf> = BTreeSet::new();
    let tokens: Vec<&str> = link_libraries.split_whitespace().collect();
    let mut i = 0;
    while i < tokens.len() {
        let tok = tokens[i];
        if DROPPED_LINK_SUBSTRINGS.iter().any(|s| tok.contains(s)) {
            i += 1;
            continue;
        }
        if tok == "-framework" {
            let name = tokens
                .get(i + 1)
                .unwrap_or_else(|| panic!("slic3r-ffi build.rs: '-framework' with no name"));
            println!("cargo:rustc-link-lib=framework={name}");
            i += 2;
            continue;
        }
        if let Some(name) = tok.strip_prefix("-l") {
            println!("cargo:rustc-link-lib={name}");
            i += 1;
            continue;
        }
        if tok.ends_with(".a") || tok.ends_with(".tbd") {
            let path = if tok.starts_with('/') {
                PathBuf::from(tok)
            } else {
                build_dir.join(tok)
            };
            let dir = path.parent().unwrap_or(build_dir).to_path_buf();
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let stem = file_name
                .strip_prefix("lib")
                .unwrap_or(file_name)
                .trim_end_matches(".a")
                .trim_end_matches(".tbd");
            if search_dirs.insert(dir.clone()) {
                println!("cargo:rustc-link-search=native={}", dir.display());
            }
            if tok.ends_with(".a") {
                println!("cargo:rustc-link-lib=static={stem}");
            } else {
                println!("cargo:rustc-link-lib={stem}");
            }
            i += 1;
            continue;
        }
        println!(
            "cargo:warning=slic3r-ffi build.rs: unrecognised LINK_LIBRARIES token '{tok}', ignored"
        );
        i += 1;
    }
}

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

    // ---- task 31-proof-harness: proof paths 1 and 3 (R19) ----
    //
    // The derived link line: the generated link command of `libslic3r_tests` (or
    // `fff_print_tests`), read out of the CMake build tree rather than enumerated by name -- the
    // one approach that cannot be wrong about which of the three places (the superbuild prefix,
    // the main project's own `deps_src/` archives, or an Apple framework/`SLVS::slvs`) a given
    // archive is in, and that stays correct when the pin's own dependency list changes.
    let oracle_build_dir: PathBuf = env::var_os(ORACLE_BUILD_DIR_VAR)
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root.join(ORACLE_BUILD_DIR_DEFAULT));
    println!("cargo:rerun-if-env-changed={ORACLE_BUILD_DIR_VAR}");

    if !oracle_build_dir.is_dir() {
        panic!(
            "slic3r-ffi build.rs: no oracle build tree at {:?} (from {ORACLE_BUILD_DIR_VAR}, \
             default {ORACLE_BUILD_DIR_DEFAULT}). Run `just oracle-build` first.",
            oracle_build_dir
        );
    }

    let (link_target, link_libraries, link_ninja_file) = find_link_libraries(&oracle_build_dir)
        .unwrap_or_else(|| {
            panic!(
                "slic3r-ffi build.rs: no 'libslic3r_tests' or 'fff_print_tests' Release link \
                 edge found under {:?}. Run `just oracle-build` first -- its STAGE2_TARGETS \
                 builds both with BUILD_TESTS=ON.",
                oracle_build_dir
            )
        });
    // Plain stdout, not a `cargo:warning=` -- this is a normal, expected build step on every
    // build of this crate (by every downstream task, forever), not something to flag. Visible
    // with `cargo build -vv` for anyone who needs to know which file the link line actually
    // came from.
    println!(
        "slic3r-ffi build.rs: deriving the proof-harness link line from {link_target}'s link \
         edge in {}",
        link_ninja_file.display()
    );
    println!("cargo:rerun-if-changed={}", link_ninja_file.display());
    // NOTE: `link_libraries`'s own tokens can carry CMake's *absolute* paths, baked in at the
    // configure time of whichever checkout built `oracle_build_dir` -- normally this one, but a
    // build tree cloned copy-on-write from another checkout (for example a worktree created from
    // a checkout that already ran `just oracle-build`) keeps that other checkout's paths. So the
    // `tbbmalloc`/TBB-header guard above, which resolves strictly from `DEPS_PREFIX_VAR`/
    // `deps_prefix`, can pass while some of the archives `emit_link_directives` below actually
    // points the linker at live under a different prefix than the one just validated. Harmless
    // when the two coincide (an ordinary clone, or CI); the fix, if this ever needs one, is
    // deriving the guard's own path from a token of `link_libraries` instead of from
    // `DEPS_PREFIX_VAR` -- not weakening the guard, which still correctly proves *some* tbbmalloc
    // is present.
    emit_link_directives(&oracle_build_dir, &link_libraries);

    // Include paths for `cxx/proof_shims.cpp` and `cxx/include_vendored.cpp`: the same set
    // `oracle/build-libslic3r` compiles `src/libslic3r/*.cpp` with -- read off this same build
    // tree's `ClipperZUtils.cpp.o` compile edge in `CMakeFiles/impl-Release.ninja`'s `INCLUDES`
    // line (see this task's pull request body) -- but without libslic3r's own `DEFINES` (Boost,
    // wx and OpenVDB static-link macros neither of this crate's two files branches on -- checked:
    // neither contains `#if`/`#ifdef` on anything but its own include guard) or its precompiled
    // header (built by CMake with CMake's own flags, not `cxx_build`'s). Unlike the link line,
    // this is an explicit, hand-picked set rather than a derivation: the task asks only the link
    // line to be read off the build tree, and feeding a PCH-oriented `DEFINES` list into a
    // different compiler driver (`cc`'s, not CMake's) is a correctness risk this crate does not
    // need to take for two files with no conditional compilation of their own.
    let vendor_src_dir = workspace_root.join("vendor/OrcaSlicer/src");
    let libslic3r_src_dir = vendor_src_dir.join("libslic3r");
    let libslic3r_build_dir = oracle_build_dir.join("src/libslic3r");
    let deps_include = deps_prefix.join("include");

    let mut build = cxx_build::bridges(["src/clipper2.rs", "src/proof.rs"]);
    // `-ffp-contract=off`: not a preference. The library this crate links and the one
    // `oracle/build-libslic3r` builds are the normative oracle for every per-unit and per-stage
    // proof only when both carry this flag (`just port-check`'s drift check fails the build
    // otherwise, by grepping for this exact call below -- not a bare substring anywhere in this
    // file, which a stray prose mention of the flag could satisfy without the flag reaching the
    // compiler). Applied to the shared `Build` before any `.file()` call, so a unit a later task
    // adds -- or this task's own two proof files below -- cannot miss it.
    build.flag("-ffp-contract=off");
    build
        .std("c++17")
        .include(&crate_dir)
        .include(&clipper2_lib_include)
        .include(&deps_include)
        .include(&libslic3r_src_dir)
        .include(libslic3r_src_dir.join("TextureToColor"))
        .include(&libslic3r_build_dir)
        .flag("-isystem")
        .flag(vendor_src_dir.to_str().expect("utf-8 workspace path"))
        .flag("-isystem")
        .flag(deps_include.to_str().expect("utf-8 workspace path"))
        .flag("-isystem")
        .flag(
            deps_include
                .join("occt")
                .to_str()
                .expect("utf-8 workspace path"),
        )
        .flag("-isystem")
        .flag(
            deps_include
                .join("eigen3")
                .to_str()
                .expect("utf-8 workspace path"),
        )
        .flag("-isystem")
        .flag(
            workspace_root
                .join("vendor/OrcaSlicer/deps_src/admesh")
                .to_str()
                .expect("utf-8 workspace path"),
        )
        .flag("-isystem")
        .flag(
            workspace_root
                .join("vendor/OrcaSlicer/deps_src/libnest2d/include")
                .to_str()
                .expect("utf-8 workspace path"),
        )
        .flag("-isystem")
        .flag(
            workspace_root
                .join("vendor/OrcaSlicer/deps_src/miniz")
                .to_str()
                .expect("utf-8 workspace path"),
        )
        .flag("-isystem")
        .flag(
            deps_include
                .join("opencv4")
                .to_str()
                .expect("utf-8 workspace path"),
        )
        .flag("-isystem")
        .flag(
            // `Semver.hpp` (pulled in by `libslic3r.h`, which `ClipperZUtils.hpp`/`ObjectID.hpp`
            // both reach) `#include`s "semver/semver.h", and the vendored file sits flat at
            // `deps_src/semver/semver.h` -- so the include path is `deps_src` itself (the
            // parent), not `deps_src/semver`. The real oracle build's own `INCLUDES` line for
            // this file names `deps_src/semver` directly and still resolves the same
            // `#include`, but only because that compile also loads a precompiled header built
            // with CMake's own, different include set; this crate compiles with no PCH, so it
            // needs the path that actually resolves this include on its own (verified: see this
            // task's pull request body).
            workspace_root
                .join("vendor/OrcaSlicer/deps_src")
                .to_str()
                .expect("utf-8 workspace path"),
        )
        .file(crate_dir.join("cxx/clipper2_plain.cc"))
        .file(crate_dir.join("cxx/clipper2_z.cc"))
        .file(crate_dir.join("cxx/proof_shims.cpp"))
        .file(crate_dir.join("cxx/include_vendored.cpp"));
    for src in CLIPPER2_VENDORED_SOURCES {
        build.file(workspace_root.join(src));
    }
    for rerun in [
        crate_dir.join("cxx/proof_shims.h"),
        crate_dir.join("cxx/proof_shims.cpp"),
        crate_dir.join("cxx/include_vendored.cpp"),
        crate_dir.join("src/proof.rs"),
        libslic3r_src_dir.join("ObjectID.hpp"),
        libslic3r_src_dir.join("ObjectID.cpp"),
        libslic3r_src_dir.join("ClipperZUtils.hpp"),
        libslic3r_src_dir.join("ClipperZUtils.cpp"),
    ] {
        println!("cargo:rerun-if-changed={}", rerun.display());
    }
    // One archive for every source on the shared `Build`: this crate's own Clipper2 compilation
    // and the proof harness's two shims all need the same flags, so they are one `cc::Build` and
    // one output archive rather than two with a flag that could drift between them.
    build.compile("slic3r-ffi-cxx");
}
