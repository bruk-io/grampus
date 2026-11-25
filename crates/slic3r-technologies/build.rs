//! Compiles `shim/technologies_shim.cpp` (task 54-foundation-objectid-clonable, R19,
//! proof path 1) into a small static archive this crate's own `#[cfg(test)]` proof links against.
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
//! crate that depends on `crates/slic3r-ffi`, and `slic3r-technologies` does not and may
//! not. See this task's pull request body.
//!
//! `vendor/OrcaSlicer/` is only ever read: the vendored header is `#include`d unmodified and
//! nothing is written into that tree.

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let libslic3r_src_dir = crate_dir.join("../../vendor/OrcaSlicer/src/libslic3r");
    let shim_src = crate_dir.join("shim/technologies_shim.cpp");

    println!("cargo:rerun-if-changed={}", shim_src.display());
    println!(
        "cargo:rerun-if-changed={}",
        libslic3r_src_dir.join("Technologies.hpp").display()
    );

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("cargo always sets OUT_DIR"));
    let obj_path = out_dir.join("technologies_shim.o");
    let archive_path = out_dir.join("libtechnologies_shim.a");

    let cxx = env::var("CXX").unwrap_or_else(|_| "c++".to_string());
    // `-ffp-contract=off`: the same non-negotiable flag `crates/slic3r-ffi/build.rs` carries (R17's
    // normative-oracle requirement). `Technologies.hpp` is eighteen `0`/`1` `#define`s with no
    // floating-point arithmetic anywhere, so the flag changes nothing here; it is applied
    // uniformly to every per-unit proof compilation in this port rather than judged absent per
    // unit.
    let status = Command::new(&cxx)
        .arg("-std=c++17")
        .arg("-ffp-contract=off")
        .arg("-I")
        .arg(&libslic3r_src_dir)
        .arg("-c")
        .arg(&shim_src)
        .arg("-o")
        .arg(&obj_path)
        .status()
        .unwrap_or_else(|e| {
            panic!("slic3r-technologies build.rs: failed to run '{cxx} -c {shim_src:?}': {e}")
        });
    if !status.success() {
        panic!(
            "slic3r-technologies build.rs: '{cxx}' failed to compile {shim_src:?} \
             (status {status})"
        );
    }

    let ar = env::var("AR").unwrap_or_else(|_| "ar".to_string());
    let status = Command::new(&ar)
        .arg("rcs")
        .arg(&archive_path)
        .arg(&obj_path)
        .status()
        .unwrap_or_else(|e| panic!("slic3r-technologies build.rs: failed to run '{ar} rcs': {e}"));
    if !status.success() {
        panic!(
            "slic3r-technologies build.rs: '{ar} rcs' failed to archive {obj_path:?} \
             (status {status})"
        );
    }

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=technologies_shim");
}
