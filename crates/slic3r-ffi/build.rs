//! Compiles the vendored Clipper2 and admesh sources, the vendored (unmodified)
//! `libnest2d.cpp`, this crate's own `cxx` glue (including the Boost.Polygon Voronoi unit), and
//! every per-unit proof shim under `cxx/proof_units/` (plan step 18, R19) onto one shared
//! `cxx_build::Build`, so every unit inherits the same flags (`-ffp-contract=off` above all).
//! The proof units are globbed, never listed -- see `discover_proof_units` and
//! `src/proof_units.rs`. qhull is bound against the
//! superbuild's installed copy instead of a vendored source -- `libqhullcpp.a`/
//! `libqhullstatic_r.a` reach the link through the derived link line below; see the
//! determination in `src/lib.rs`'s crate-doc table.
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

/// Where `just oracle-build` (task 13-build-libslic3r) puts the Ninja Multi-Config build tree.
/// Two things read it: task 31-proof-harness's derived link line for proof paths 1 and 3, and
/// task 30-ffi-bridge-libraries' libnest2d bridge, which links the `libslic3r` built there rather
/// than rebuilding it. Overridable for the same reason `DEPS_PREFIX_VAR` is above -- never
/// hardcode a path that exists on one machine only.
const ORACLE_BUILD_DIR_VAR: &str = "GRAMPUS_ORACLE_BUILD_DIR";
const ORACLE_BUILD_DIR_DEFAULT: &str = "vendor/OrcaSlicer/build/arm64";

/// The `Ninja Multi-Config` config name `oracle/build-libslic3r` builds
/// (`-DCMAKE_BUILD_TYPE=Release`, `cmake --build ... --config Release`), which is the
/// subdirectory its archives land in.
const ORACLE_BUILD_CONFIG: &str = "Release";

/// CMake's "Ninja Multi-Config" generator (plan step 7) splits a build tree into `build.ninja`
/// (default-config `phony` aliases only), `build-<Config>.ninja` (an `include` of the real file)
/// and `CMakeFiles/impl-<Config>.ninja` (the actual compile/link edges, with their `FLAGS`,
/// `INCLUDES`, `LINK_LIBRARIES` and `LINK_FLAGS` variables) -- verified empirically against this
/// pin's CMake 4.1 output (see task 31-proof-harness's pull request body): `build.ninja` and
/// `build-Release.ninja` hold only a `phony` alias for `libslic3r_tests`/`fff_print_tests`, never
/// the edge the task's own text names first. Checked in this order (not a single hardcoded name)
/// because the real edge could in principle live in any of them on a different CMake version,
/// and this still never hardcodes a *library* name -- only this generator's known file layout.
const NINJA_FILES_TO_SEARCH: &[&str] = &[
    "CMakeFiles/impl-Release.ninja",
    "build-Release.ninja",
    "build.ninja",
];

/// The two targets task 31-proof-harness names, in its own order: `libslic3r_tests` first,
/// `fff_print_tests` as the fallback.
const LINK_LINE_TARGETS: &[&str] = &["libslic3r_tests", "fff_print_tests"];

/// A token containing any of these is dropped from the derived link line, for one reason in every
/// case: **this crate compiles the same sources itself**, so linking the build tree's archive too
/// would put two definitions of the same symbols on the link, and the crate's own compilation is
/// the one its bridges' behaviour is measured against. Three of the four are that:
/// `deps_src/clipper2/` (`CLIPPER2_VENDORED_SOURCES`, task 29-ffi-bridge-core),
/// `deps_src/admesh/` (`ADMESH_VENDORED_SOURCES`) and `deps_src/libnest2d/`
/// (`LIBNEST2D_VENDORED_SOURCE`) -- the latter two added by task 30-ffi-bridge-libraries along
/// with the compilations that made them duplicates, the same pruning that task's own earlier,
/// hand-enumerated link table did before this derivation replaced it.
///
/// Catch2's own archives are the exception: they exist only to give the oracle's
/// `libslic3r_tests`/`fff_print_tests` binary a `main()` and its assertion macros, neither of
/// which this crate's bridges or proof shims use.
const DROPPED_LINK_SUBSTRINGS: &[&str] = &[
    "deps_src/clipper2/",
    "deps_src/admesh/",
    "deps_src/libnest2d/",
    "tests/catch2/",
];

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
///
/// **Known divergence from the oracle's own command line for this TU, not fixed here.** The
/// oracle compiles this file with `-DNDEBUG` (and several more defines this crate's command line
/// omits: `-D_HAS_AUTO_PTR_ETC=0`, `-DBOOST_NO_CXX98_FUNCTION_BASE`, `-DUSE_TBB`, ...; see
/// `vendor/OrcaSlicer/build/arm64/CMakeFiles/impl-Release.ninja`'s own `libnest2d.cpp.o` entry),
/// while `cargo build`/`cargo test` add no `-DNDEBUG` under the `dev`/`test` profile. Measured
/// directly against this crate's own build (not assumed, and not the evidence another reviewer
/// already measured): `nm -u` on the compiled object this build produces lists one undefined
/// `___assert_rtn`, where the oracle's own `libnest2d.cpp.o` (built by `oracle/build-libslic3r`,
/// `deps_src/libnest2d/CMakeFiles/libnest2d.dir/Release/src/libnest2d.cpp.o`) has zero among its
/// 150 undefined symbols. So an `assert()` inside `libnest2d.hpp`'s header-only templates (e.g.
/// `nfpplacer.hpp:247`) is live in this crate's object and compiled out of the oracle's
/// `liblibslic3r.a` for the same inline functions -- an ODR mismatch and a behaviour divergence a
/// later task porting against this bridge (`TreeSupport.cpp`) should know about. Not fixed here:
/// adding `-DNDEBUG` crate-wide would also reach Clipper2's and admesh's TUs (task
/// 29-ffi-bridge-core's and this task's own committed smoke-test numbers were measured without
/// it), and no `Done when` line in this task asks for NDEBUG parity -- only the four
/// `LIBNEST2D_*` defines below are named. Left as a recorded hazard, not silently accepted; see
/// the pull request body.
///
/// **What the bridge does do about it.** The divergence above is about the whole header-only
/// body of `libnest2d.hpp`, most of which no safe fn in this crate can reach. The part that is
/// reachable is handled instead of merely recorded: `nfpplacer.hpp:247`'s
/// `assert(distance >= .0 && distance <= 1.0)` sits on `EdgeCache::coords(double)`, which
/// `cxx/libnest2d_glue.cc` binds, so that file range-checks `distance` itself and refuses
/// out-of-range input before reaching the assert. That is the only input-domain precondition
/// assert reachable from this crate's bound surface: `strings` over each of the four
/// `*_glue.o` objects this build produces finds that one condition and, beyond it, only
/// Eigen's and libc++'s own internal invariants (`dst.rows() == dstRows`,
/// `index >= 0 && index < size()`, `s != nullptr || count == 0`), none of which is
/// parameterized by bridge input. So no safe fn in this crate aborts on ordinary input, while
/// NDEBUG parity with the oracle's command line is still absent -- the two statements are
/// about different things and both hold.
const LIBNEST2D_VENDORED_SOURCE: &str = "vendor/OrcaSlicer/deps_src/libnest2d/src/libnest2d.cpp";

/// The four compile definitions the vendored target makes `PUBLIC`
/// (`deps_src/libnest2d/CMakeLists.txt:28`), applied to the whole shared `Build` the same way
/// the oracle build applies them to every target depending on `libnest2d` (they show up on
/// `ClipperUtils.cpp.o`'s own `DEFINES` line in `CMakeFiles/impl-Release.ninja`, inherited
/// through `target_link_libraries(libslic3r PUBLIC ... libnest2d ...)`).
///
/// That justification covers the TUs that, in the oracle build, actually depend on `libnest2d`:
/// `libslic3r`'s own sources -- which includes the proof-path-3 unit
/// `cxx/proof_units/clipper_z_utils.cpp`, since the vendored `.cpp` it includes *is* a
/// `libslic3r` source
/// and the oracle compiles that file with exactly these four defines, so giving them to that TU
/// moves it towards the oracle's command line rather than away from it.
///
/// It does not cover Clipper2's five sources or admesh's six: those are separate CMake targets in
/// the oracle build and link neither `libnest2d` nor `libslic3r`, so the oracle's own command line
/// gives them none of these four defines
/// (`grep -rl LIBNEST2D vendor/OrcaSlicer/deps_src/clipper2 vendor/OrcaSlicer/deps_src/admesh`
/// matches nothing). `cxx_build::Build` has one shared flag/define set for every `.file()` added
/// to it, with no per-file override, so this crate gives all of its TUs the same four defines
/// rather than the oracle's narrower per-target sets. Verified inert at the pin (the grep above),
/// not assumed: neither vendored source tree ever references any `LIBNEST2D_*` identifier, so the
/// defines have nothing to affect in either one.
const LIBNEST2D_DEFINES: &[&str] = &[
    "LIBNEST2D_THREADING_tbb",
    "LIBNEST2D_STATIC",
    "LIBNEST2D_OPTIMIZER_nlopt",
    "LIBNEST2D_GEOMETRIES_libslic3r",
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
    // A directory that does not exist yet cannot be canonicalized, but it would still reach the
    // compiler as a dangling `-isystem` entry (clang ignores a missing `-isystem` dir silently),
    // so this guard must not treat canonicalize failure as "not disallowed" -- that would let a
    // not-yet-created path under one of the disallowed prefixes through unchecked, and the
    // failure would then surface much later as a confusing header error instead of this panic.
    // Fall back to a lexical prefix check on the path as given.
    match path.canonicalize() {
        Ok(canon) => DISALLOWED_INCLUDE_PREFIXES
            .iter()
            .any(|prefix| canon.starts_with(prefix)),
        Err(_) => DISALLOWED_INCLUDE_PREFIXES
            .iter()
            .any(|prefix| path.starts_with(prefix)),
    }
}

/// One per-unit proof bridge (R19, plan step 18): the unit's name (its `port/manifest.tsv`
/// `target module` file stem, which is also its Rust module name), the bridge `.rs` under
/// `src/proof_units/` and the shim `.cpp` under `cxx/proof_units/`.
struct ProofUnit {
    name: String,
    bridge_rs: PathBuf,
    shim_cpp: PathBuf,
    shim_h: PathBuf,
}

/// Every per-unit proof bridge present in the tree, found by globbing `src/proof_units/*.rs`
/// rather than from a list any file names -- the point of the layout (`docs/PORTING.md`,
/// "Decisions recorded on 2026-11-25"): a port task adds its unit by creating three new files,
/// so hundreds of port branches built side by side never append to a shared list, a shared C++
/// file or a shared `pub use` block.
///
/// Sorted by unit name, so the generated module list, the `cxx_build::bridges` argument and the
/// compiled-source order are all deterministic and two builds of the same tree are identical.
/// A bridge `.rs` with no matching `cxx/proof_units/<unit>.h` and `<unit>.cpp` is a named panic,
/// not a silent skip: the bridge would otherwise compile to a generated translation unit calling
/// a C++ function nothing defines, which is the undefined-symbol wall a hand-edited shared
/// header already produced once (see PR #54's own measurement of it).
fn discover_proof_units(crate_dir: &Path) -> Vec<ProofUnit> {
    let bridge_dir = crate_dir.join("src/proof_units");
    let shim_dir = crate_dir.join("cxx/proof_units");
    // Both directories, not only their current contents: cargo re-runs this script when a
    // directory's listing changes, which is what makes a newly added unit build without any
    // other file being touched.
    println!("cargo:rerun-if-changed={}", bridge_dir.display());
    println!("cargo:rerun-if-changed={}", shim_dir.display());

    let mut names: Vec<String> = Vec::new();
    let entries = match fs::read_dir(&bridge_dir) {
        Ok(entries) => entries,
        Err(e) => panic!(
            "slic3r-ffi build.rs: cannot read the per-unit proof bridge directory {:?}: {e}",
            bridge_dir
        ),
    };
    for entry in entries {
        let path = entry
            .unwrap_or_else(|e| panic!("slic3r-ffi build.rs: reading {:?}: {e}", bridge_dir))
            .path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_else(|| panic!("slic3r-ffi build.rs: non-utf-8 bridge file name {:?}", path))
            .to_string();
        // The unit name is a Rust module name and a C++ namespace component, so it is held to
        // the intersection of both: lower-case ASCII, digits and underscores, starting with a
        // letter. A name that is not (`Object-ID.rs`, `mod.rs`) would otherwise fail much later,
        // inside the generated module list or the generated C++, with a confusing error.
        let valid = name.chars().next().is_some_and(|c| c.is_ascii_lowercase())
            && name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
        if !valid {
            panic!(
                "slic3r-ffi build.rs: {:?} is not a valid proof-unit name -- a file under \
                 src/proof_units/ must be named <unit>.rs, where <unit> is the unit's \
                 port/manifest.tsv 'target module' file stem (lower-case ASCII, digits and \
                 underscores, starting with a letter)",
                path
            );
        }
        names.push(name);
    }
    // The shim directory is globbed too, so an orphan there is a named failure rather than a
    // file that is silently never compiled: a port task that writes its two C++ files and
    // forgets the bridge would otherwise get an unresolved-name error from its own test, with
    // nothing pointing at the missing file. The bridge-without-shim direction is checked below.
    let mut shim_names: Vec<String> = Vec::new();
    if let Ok(entries) = fs::read_dir(&shim_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("cpp") {
                continue;
            }
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                shim_names.push(stem.to_string());
            }
        }
    }
    for stem in &shim_names {
        if !names.contains(stem) {
            panic!(
                "slic3r-ffi build.rs: {:?} has no bridge at {:?}. A proof unit is exactly three \
                 files: cxx/proof_units/<unit>.h, cxx/proof_units/<unit>.cpp and \
                 src/proof_units/<unit>.rs -- without the bridge the shim is never compiled and \
                 nothing can call it.",
                shim_dir.join(format!("{stem}.cpp")),
                bridge_dir.join(format!("{stem}.rs"))
            );
        }
    }
    names.sort();

    names
        .into_iter()
        .map(|name| {
            let unit = ProofUnit {
                bridge_rs: bridge_dir.join(format!("{name}.rs")),
                shim_cpp: shim_dir.join(format!("{name}.cpp")),
                shim_h: shim_dir.join(format!("{name}.h")),
                name,
            };
            for required in [&unit.shim_cpp, &unit.shim_h] {
                if !required.is_file() {
                    panic!(
                        "slic3r-ffi build.rs: the proof unit '{}' has a bridge at {:?} but no \
                         {:?}. A proof unit is exactly three files: \
                         cxx/proof_units/<unit>.h, cxx/proof_units/<unit>.cpp and \
                         src/proof_units/<unit>.rs.",
                        unit.name, unit.bridge_rs, required
                    );
                }
            }
            unit
        })
        .collect()
}

/// A vendored `src/libslic3r/` file a shim translation unit `#include`s, so `cargo` rebuilds the
/// shim when the pinned source it reads changes. Read out of the shim's own `#include "..."`
/// lines rather than from a list this script keeps: a proof-path-3 shim `#include`s the vendored
/// `.cpp` whole, and a proof-path-1 shim `#include`s the vendored header its symbols are
/// declared in, so the dependency is always written in the shim itself.
fn vendored_includes_of(shim_cpp: &Path, libslic3r_src_dir: &Path) -> Vec<PathBuf> {
    let Ok(text) = fs::read_to_string(shim_cpp) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim_start();
        let Some(rest) = line.strip_prefix("#include") else {
            continue;
        };
        let rest = rest.trim_start();
        let Some(rest) = rest.strip_prefix('"') else {
            continue;
        };
        let Some(end) = rest.find('"') else {
            continue;
        };
        let included = &rest[..end];
        let candidate = libslic3r_src_dir.join(included);
        if candidate.is_file() {
            out.push(candidate);
        }
    }
    out
}

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
///   the default kind) for `.tbd`; this is how the SDK's own `libz.tbd`/`libiconv.tbd` reach the
///   link without this crate naming either one;
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
    println!("cargo:rerun-if-env-changed={ORACLE_BUILD_DIR_VAR}");
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
        "src/libnest2d.rs",
        "cxx/voronoi_shim.h",
        "cxx/voronoi_glue.cc",
        "src/voronoi.rs",
        // The per-unit proof bridges' own module file. Every unit under the two directories it
        // lists is found by `discover_proof_units` below, not named here.
        "src/proof_units.rs",
    ] {
        println!("cargo:rerun-if-changed={}", crate_dir.join(glue).display());
    }

    let deps_prefix: PathBuf = env::var_os(DEPS_PREFIX_VAR)
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root.join(DEPS_PREFIX_DEFAULT));
    let deps_prefix_lib = deps_prefix.join("lib");

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

    println!(
        "cargo:rustc-link-search=native={}",
        deps_prefix_lib.display()
    );
    println!("cargo:rustc-link-lib=static=tbbmalloc");

    let clipper2_lib_include =
        workspace_root.join("vendor/OrcaSlicer/deps_src/clipper2/Clipper2Lib/include");

    // ---- the build tree `just oracle-build` produces, and the link line derived from it ----
    //
    // Two tasks need it and neither can run without it: task 31-proof-harness's proof paths 1
    // and 3 (R19) link against the archives there, and task 30-ffi-bridge-libraries' libnest2d
    // bridge links the `libslic3r` built there rather than rebuilding it (R17).
    //
    // The derived link line: the generated link command of `libslic3r_tests` (or
    // `fff_print_tests`), read out of the CMake build tree rather than enumerated by name -- the
    // one approach that cannot be wrong about which of the three places (the superbuild prefix,
    // the main project's own `deps_src/` archives, or an Apple framework/`SLVS::slvs`) a given
    // archive is in, and that stays correct when the pin's own dependency list changes. It is
    // also what puts the superbuild's `libqhullcpp.a`/`libqhullstatic_r.a` on the link, which is
    // the qhull determination task 30-ffi-bridge-libraries recorded (see `src/lib.rs`'s table):
    // they appear in this line because the step-7 build itself linked that copy, which is
    // stronger evidence than this file naming them.
    let oracle_build_dir: PathBuf = env::var_os(ORACLE_BUILD_DIR_VAR)
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root.join(ORACLE_BUILD_DIR_DEFAULT));

    if !oracle_build_dir.is_dir() {
        panic!(
            "slic3r-ffi build.rs: no oracle build tree at {:?} (from {ORACLE_BUILD_DIR_VAR}, \
             default {ORACLE_BUILD_DIR_DEFAULT}). Run `just oracle-build` first.",
            oracle_build_dir
        );
    }

    // The built libslic3r itself, checked the same way as tbbmalloc above: against the actual
    // archive, not just the build directory's existence, so a half-populated tree fails here with
    // a named error rather than obscurely at the link step.
    let libslic3r_archive = oracle_build_dir
        .join("src/libslic3r")
        .join(ORACLE_BUILD_CONFIG)
        .join("liblibslic3r.a");
    if !libslic3r_archive.is_file() {
        panic!(
            "slic3r-ffi build.rs: no liblibslic3r.a under {:?}. Run `just oracle-build` first \
             (task 13-build-libslic3r) -- this crate's libnest2d bridge and its two proof paths \
             link step 7's built libslic3r, they do not rebuild it.",
            libslic3r_archive
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
        "slic3r-ffi build.rs: deriving the link line from {link_target}'s link edge in {}",
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

    let admesh_include = workspace_root.join("vendor/OrcaSlicer/deps_src/admesh");
    // `deps_src` itself, the parent, is on the path twice over: `admesh`'s own headers reach
    // their siblings through it (`deps_src/admesh/CMakeLists.txt:14-18`), and `Semver.hpp`
    // (pulled in by `libslic3r.h`, which `ClipperZUtils.hpp`/`ObjectID.hpp` both reach)
    // `#include`s "semver/semver.h" while the vendored file sits flat at
    // `deps_src/semver/semver.h`. The real oracle build's own `INCLUDES` line for the latter
    // names `deps_src/semver` directly and still resolves the same `#include`, but only because
    // that compile also loads a precompiled header built with CMake's own, different include
    // set; this crate compiles with no PCH, so it needs the path that actually resolves this
    // include on its own (verified: see task 31-proof-harness's pull request body).
    let deps_src_include = workspace_root.join("vendor/OrcaSlicer/deps_src");
    let libnest2d_include = workspace_root.join("vendor/OrcaSlicer/deps_src/libnest2d/include");
    let miniz_include = workspace_root.join("vendor/OrcaSlicer/deps_src/miniz");
    let vendor_src_dir = workspace_root.join("vendor/OrcaSlicer/src");
    let libslic3r_src_dir = vendor_src_dir.join("libslic3r");
    let deps_prefix_include = deps_prefix.join("include");
    // Eigen installs itself one level deeper than the rest of the superbuild
    // (`$prefix/include/eigen3/Eigen/...`, not `$prefix/include/Eigen/...` -- see the
    // `-isystem .../include/eigen3` entry on `ClipperUtils.cpp.o`'s own `INCLUDES` line in
    // `CMakeFiles/impl-Release.ninja`), which `admesh/stl.h:31`'s `#include <Eigen/Geometry>`
    // needs. OCCT and OpenCV install themselves the same way, and libslic3r's own headers reach
    // both.
    let deps_prefix_eigen3_include = deps_prefix_include.join("eigen3");
    let deps_prefix_occt_include = deps_prefix_include.join("occt");
    let deps_prefix_opencv4_include = deps_prefix_include.join("opencv4");
    // `AABBTreeIndirect.hpp:25`'s `#include <igl/Hit.h>` is unconditional (every proof shim that
    // includes that header at all needs it resolvable, not only one that calls a ray-intersection
    // function) -- the vendored copy sits at `deps_src/libigl/igl/Hit.h`, one level deeper than
    // `deps_src` itself resolves (task `60-foundation-trees-grids`, measured: compiling without
    // this entry fails with `'igl/Hit.h' file not found`, and `deps_src/libigl` is exactly the
    // directory that makes `<igl/Hit.h>` resolve, the same one-line-deeper pattern
    // `admesh_include` above states for `deps_src/admesh`).
    let libigl_include = workspace_root.join("vendor/OrcaSlicer/deps_src/libigl");
    // `libslic3r.h:4`'s `#include "libslic3r_version.h"` is a CMake-generated file that lands in
    // the build directory, not the source tree (`CMAKE_CURRENT_BINARY_DIR`, the same `-I
    // .../build/arm64/src/libslic3r` entry on `ClipperUtils.cpp.o`'s own `INCLUDES` line) --
    // step 7's own build already generated it, this crate does not.
    let libslic3r_build_include = oracle_build_dir.join("src/libslic3r");

    // The include set for the whole shared `Build`, in two groups.
    //
    // `-isystem`, not plain `-I`, for every vendor/superbuild/dependency directory -- exactly how
    // the oracle build itself puts them on `libnest2d.cpp.o`'s command line
    // (`CMakeFiles/impl-Release.ninja`'s `INCLUDES` line). This is not cosmetic: clang suppresses
    // diagnostics raised *inside* a system header, and without it this exact header chain
    // (`libnest2d.hpp` -> `geometries.hpp` -> `utils/boost_alg.hpp` -> `boost/geometry.hpp` ->
    // `boost/qvm/quat_traits.hpp`) fails to compile under this toolchain
    // (`-Wmissing-template-arg-list-after-template-kw`) when included as an ordinary `-I` --
    // confirmed by reproducing the error with `-I` and clearing it by switching to `-isystem`,
    // not guessed. A directory passed as *both* `-I` and `-isystem` is deduplicated by clang
    // keeping the earlier, non-system `-I` entry, so a dependency directory must appear in this
    // group and nowhere else, or the suppression is silently lost.
    let isystem_include_dirs = [
        clipper2_lib_include.clone(),
        admesh_include.clone(),
        deps_src_include.clone(),
        libnest2d_include.clone(),
        miniz_include.clone(),
        vendor_src_dir.clone(),
        deps_prefix_include.clone(),
        deps_prefix_eigen3_include.clone(),
        deps_prefix_occt_include.clone(),
        deps_prefix_opencv4_include.clone(),
        libigl_include.clone(),
    ];
    // Plain `-I`: this crate's own directory (for `cxx/...` includes), the vendored libslic3r
    // source directory and the build tree's generated-header directory -- the three the oracle
    // build itself passes as ordinary `-I` entries too.
    let plain_include_dirs = [
        crate_dir.clone(),
        libslic3r_src_dir.clone(),
        libslic3r_src_dir.join("TextureToColor"),
        libslic3r_build_include.clone(),
    ];

    // Checkable from this list, which task 30-ffi-bridge-libraries' Done-when condition asks for:
    // every include directory this build adds to the shared `Build` -- and therefore to every
    // unit on it, this task's four bridges and task 31-proof-harness's two proof units alike --
    // printed so `cargo build -p slic3r-ffi -vv`'s output (or this list itself) shows it, and
    // asserted here to carry none of `DISALLOWED_INCLUDE_PREFIXES`. The superbuild prefix's own
    // path ends in `usr/local` but does not *start* with `/usr/local`, so it passes.
    for dir in isystem_include_dirs.iter().chain(&plain_include_dirs) {
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

    // The per-unit proof bridges, globbed rather than listed (see `discover_proof_units`). The
    // `#[path]`-ed `pub mod` line for each one is generated into OUT_DIR and `include!`d by
    // `src/proof_units.rs`, so the Rust module list is derived from the same glob the C++ side
    // is and the two can never disagree about which units exist.
    let proof_units = discover_proof_units(&crate_dir);
    let mut mods_rs = String::from(
        "// Generated by crates/slic3r-ffi/build.rs from the files present in \
         src/proof_units/.\n// Do not edit, and do not commit: `src/proof_units.rs` \
         `include!`s this from OUT_DIR.\n",
    );
    for unit in &proof_units {
        for watched in [&unit.bridge_rs, &unit.shim_cpp, &unit.shim_h] {
            println!("cargo:rerun-if-changed={}", watched.display());
        }
        for vendored in vendored_includes_of(&unit.shim_cpp, &libslic3r_src_dir) {
            println!("cargo:rerun-if-changed={}", vendored.display());
        }
        mods_rs.push_str(&format!(
            "#[path = {:?}]\npub mod {};\n",
            unit.bridge_rs.to_str().expect("utf-8 workspace path"),
            unit.name
        ));
    }
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("cargo always sets OUT_DIR"));
    fs::write(out_dir.join("proof_units_mods.rs"), &mods_rs)
        .expect("writing the generated proof-unit module list into OUT_DIR");
    println!(
        "slic3r-ffi build.rs: {} per-unit proof bridge(s): {}",
        proof_units.len(),
        proof_units
            .iter()
            .map(|u| u.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );

    let mut bridge_paths: Vec<PathBuf> = vec![
        PathBuf::from("src/clipper2.rs"),
        PathBuf::from("src/admesh.rs"),
        PathBuf::from("src/qhull.rs"),
        PathBuf::from("src/libnest2d.rs"),
        PathBuf::from("src/voronoi.rs"),
    ];
    bridge_paths.extend(
        proof_units
            .iter()
            .map(|u| PathBuf::from(format!("src/proof_units/{}.rs", u.name))),
    );
    let mut build = cxx_build::bridges(&bridge_paths);
    // `-ffp-contract=off`: not a preference. The library this crate links and the one
    // `oracle/build-libslic3r` builds are the normative oracle for every per-unit and per-stage
    // proof only when both carry this flag (`just port-check`'s drift check fails the build
    // otherwise, by grepping for this exact call below -- not a bare substring anywhere in this
    // file, which a stray prose mention of the flag could satisfy without the flag reaching the
    // compiler). Applied to the shared `Build` before any `.file()` call, so a unit a later task
    // adds -- or the four bridges and two proof units below -- cannot miss it.
    build.flag("-ffp-contract=off");
    for define in LIBNEST2D_DEFINES {
        build.define(define, None);
    }
    build.std("c++17");
    for dir in &plain_include_dirs {
        build.include(dir);
    }
    for dir in &isystem_include_dirs {
        build.flag("-isystem");
        build.flag(dir.to_str().expect("utf-8 workspace path"));
    }
    build
        .file(crate_dir.join("cxx/clipper2_plain.cc"))
        .file(crate_dir.join("cxx/clipper2_z.cc"))
        .file(crate_dir.join("cxx/admesh_glue.cc"))
        .file(crate_dir.join("cxx/qhull_glue.cc"))
        .file(crate_dir.join("cxx/libnest2d_glue.cc"))
        .file(crate_dir.join("cxx/voronoi_glue.cc"));
    for unit in &proof_units {
        build.file(&unit.shim_cpp);
    }
    for src in CLIPPER2_VENDORED_SOURCES
        .iter()
        .chain(ADMESH_VENDORED_SOURCES)
        .chain(std::iter::once(&LIBNEST2D_VENDORED_SOURCE))
    {
        build.file(workspace_root.join(src));
    }
    // The vendored libslic3r files the proof units reach are not listed here: each unit's
    // `#include "..."` lines are read by `vendored_includes_of` above and turned into
    // `cargo:rerun-if-changed` there, so a proof-path-3 unit that `#include`s a vendored `.cpp`
    // whole, and a proof-path-1 unit that `#include`s the header its symbols are declared in,
    // both rebuild when that pinned source changes without this file naming either one.
    //
    // One archive for every source on the shared `Build`: this crate's own Clipper2/admesh/
    // libnest2d compilations, its four bridge glue units and the proof harness's two shims all
    // need the same flags, so they are one `cc::Build` and one output archive rather than several
    // with a flag that could drift between them.
    build.compile("slic3r-ffi-cxx");
}
