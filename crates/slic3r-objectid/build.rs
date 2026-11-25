//! Compiles `shim/object_id_shim.cpp` and the vendored `src/libslic3r/ObjectID.cpp` itself,
//! unmodified, into a small static archive this crate's own proofs link against (task
//! 54-foundation-objectid-clonable, R19, proof paths 1 and 2).
//!
//! Deliberately dependency-free, the same shape task `53-foundation-int128-queue`'s
//! `crates/slic3r-int128/build.rs` uses: no `cc`/`cxx` crate, no `[build-dependencies]` line and
//! no `[dependencies]` on `crates/slic3r-ffi` -- every `crates/<crate>/Cargo.toml` other than the
//! fuzz crate's is on this task's must-not-write list, and Cargo picks a package-root `build.rs`
//! up with no manifest key, so this file adds nothing to that manifest. It shells out to the
//! system C++ compiler with `std::process::Command`, which is what `cc`/`cxx-build` do
//! internally.
//!
//! Why a per-crate shim rather than `crates/slic3r-ffi/cxx/proof_shims.cpp` (which this task also
//! appends to, per task.md's shared-file convention): that file's bridge is reachable only from a
//! crate that depends on `crates/slic3r-ffi`, and `slic3r-objectid` does not and may not. See this
//! task's pull request body.
//!
//! **Why `ObjectID.cpp` is compiled here rather than the built archive linked.** `ObjectID.cpp`'s
//! three definitions -- `ObjectBase::s_last_id`, `ObjectWithTimestamp::s_last_timestamp` and the
//! two `wipe_tower_*_id()` functions -- have external linkage and do live in
//! `vendor/OrcaSlicer/build/arm64/src/libslic3r/Release/liblibslic3r.a`, but that archive's link
//! line is the whole of libslic3r's dependency set (Boost, TBB, CGAL, qhull and the rest), which
//! `crates/slic3r-ffi/build.rs` derives from the CMake build tree for exactly that reason.
//! Compiling the pin's own `ObjectID.cpp` here, unmodified and with the same
//! `-ffp-contract=off`, gives the same code from the same commit without duplicating that
//! derivation in a crate that needs four symbols from it. Recorded in this task's pull request
//! body as a deviation from "the library built from the pin", because it is one: the source is
//! the pin, the object file is this build's.
//!
//! `vendor/OrcaSlicer/` is only ever read: both the header and `ObjectID.cpp` are compiled where
//! they lie, and nothing is written into that tree.

use std::env;
use std::path::PathBuf;
use std::process::Command;

/// The dependency superbuild's install prefix, whose `include/cereal` `ObjectID.hpp:4-5` needs
/// (`<cereal/access.hpp>`, `<cereal/types/base_class.hpp>`). Same environment variable and same
/// default as `crates/slic3r-ffi/build.rs`'s own `GRAMPUS_DEPS_PREFIX`, so the two cannot
/// disagree and neither hardcodes a path that exists on one machine only. CI restores this tree
/// alongside `vendor/OrcaSlicer/build/arm64` (`.github/workflows/ci.yml`'s "Restore the
/// dependency superbuild" step), which is what makes `cargo test --workspace` able to run any
/// proof at all there.
const DEPS_PREFIX_VAR: &str = "GRAMPUS_DEPS_PREFIX";
const DEPS_PREFIX_DEFAULT: &str = "vendor/OrcaSlicer/deps/build/arm64/OrcaSlicer_dep/usr/local";

fn main() {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = crate_dir.join("../..");
    let libslic3r_src_dir = workspace_root.join("vendor/OrcaSlicer/src/libslic3r");
    let vendored_object_id_cpp = libslic3r_src_dir.join("ObjectID.cpp");
    let shim_src = crate_dir.join("shim/object_id_shim.cpp");

    let deps_prefix = match env::var_os(DEPS_PREFIX_VAR) {
        Some(p) => PathBuf::from(p),
        None => workspace_root.join(DEPS_PREFIX_DEFAULT),
    };
    let deps_include = deps_prefix.join("include");

    println!("cargo:rerun-if-env-changed={DEPS_PREFIX_VAR}");
    println!("cargo:rerun-if-changed={}", shim_src.display());
    println!(
        "cargo:rerun-if-changed={}",
        vendored_object_id_cpp.display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        libslic3r_src_dir.join("ObjectID.hpp").display()
    );

    if !deps_include.join("cereal").is_dir() {
        panic!(
            "slic3r-objectid build.rs: no cereal/ under {deps_include:?} -- ObjectID.hpp:4-5 \
             includes <cereal/access.hpp> and <cereal/types/base_class.hpp>. Run the dependency \
             superbuild (see oracle/build-libslic3r) or set {DEPS_PREFIX_VAR}."
        );
    }

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("cargo always sets OUT_DIR"));
    let archive_path = out_dir.join("libobject_id_shim.a");

    let cxx = env::var("CXX").unwrap_or_else(|_| "c++".to_string());
    let ar = env::var("AR").unwrap_or_else(|_| "ar".to_string());

    let mut objects: Vec<PathBuf> = Vec::new();
    for (source, object_name) in [
        (&shim_src, "object_id_shim.o"),
        (&vendored_object_id_cpp, "vendored_object_id.o"),
    ] {
        let obj_path = out_dir.join(object_name);
        // `-ffp-contract=off`: the same non-negotiable flag `crates/slic3r-ffi/build.rs` carries
        // (R17's normative-oracle requirement). Neither translation unit does floating-point
        // arithmetic, so the flag changes nothing here; it is applied uniformly to every per-unit
        // proof compilation in this port rather than judged absent per unit.
        let status = Command::new(&cxx)
            .arg("-std=c++17")
            .arg("-ffp-contract=off")
            .arg("-I")
            .arg(&libslic3r_src_dir)
            .arg("-I")
            .arg(&deps_include)
            .arg("-c")
            .arg(source)
            .arg("-o")
            .arg(&obj_path)
            .status()
            .unwrap_or_else(|e| {
                panic!("slic3r-objectid build.rs: failed to run '{cxx} -c {source:?}': {e}")
            });
        if !status.success() {
            panic!(
                "slic3r-objectid build.rs: '{cxx}' failed to compile {source:?} (status {status})"
            );
        }
        objects.push(obj_path);
    }

    let status = Command::new(&ar)
        .arg("rcs")
        .arg(&archive_path)
        .args(&objects)
        .status()
        .unwrap_or_else(|e| panic!("slic3r-objectid build.rs: failed to run '{ar} rcs': {e}"));
    if !status.success() {
        panic!(
            "slic3r-objectid build.rs: '{ar} rcs' failed to archive {objects:?} (status {status})"
        );
    }

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=object_id_shim");
    // The C++ runtime. Unlike `Int128.hpp`'s header-only arithmetic (task
    // 53-foundation-int128-queue's shim, which needs none), this archive carries virtual
    // destructors, their `typeinfo`, `operator delete` and the `__cxa_guard_*` calls that
    // `ObjectID.cpp:11`'s function-local `static ObjectBase mine` compiles into -- all of which
    // live in the standard library. Chosen from the target rather than hardcoded: Apple and the
    // BSDs ship libc++, the rest of the world libstdc++.
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let cxx_runtime = match target_os.as_str() {
        "macos" | "ios" | "freebsd" | "openbsd" => "c++",
        _ => "stdc++",
    };
    println!("cargo:rustc-link-lib=dylib={cxx_runtime}");
}
