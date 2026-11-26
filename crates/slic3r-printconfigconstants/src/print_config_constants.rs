//! Port of `PrintConfigConstants.hpp`
//! (`vendor/OrcaSlicer/src/libslic3r/PrintConfigConstants.hpp`): three `#define`s the upstream
//! header itself says are "here just for unit testing" (`:3`). Each macro's call site
//! (`PrintConfig.cpp`) pins its type: `ConfigOptionFloat` wraps `double`, `ConfigOptionInt`
//! wraps `int`, `ConfigOptionBool` wraps `bool` (`Config.hpp:824,1018,1907`) -- the type map's
//! `double -> f64` and the platform's 32-bit `int -> i32`.

/// `INITIAL_LAYER_HEIGHT` (`PrintConfigConstants.hpp:5`, `#define INITIAL_LAYER_HEIGHT 0.2`).
// PORT SRC: src/libslic3r/PrintConfigConstants.hpp:5
pub const INITIAL_LAYER_HEIGHT: f64 = 0.2;

/// `INITIAL_RAFT_LAYERS` (`:6`, `#define INITIAL_RAFT_LAYERS 0`). `0` is in the numeric-literal
/// rule's exempt set, so no `// PORT SRC:` marker is required for it, but one is recorded anyway
/// (the Done-when line's own wording: "every numeric constant transcribed ... carries" one).
// PORT SRC: src/libslic3r/PrintConfigConstants.hpp:6
pub const INITIAL_RAFT_LAYERS: i32 = 0;

/// `INITIAL_REDUCE_CROSSING_WALL` (`:7`, `#define INITIAL_REDUCE_CROSSING_WALL false`). Not a
/// numeric literal, so the marker rule never applies to it; cited here for completeness.
pub const INITIAL_REDUCE_CROSSING_WALL: bool = false;

#[cfg(test)]
mod tests {
    use super::*;

    // The proof-path-1 oracle (R19):
    // `crates/slic3r-ffi/cxx/proof_units/print_config_constants.cpp`, which `#include`s the
    // vendored `PrintConfigConstants.hpp` unmodified and is compiled from the pin onto the FFI
    // crate's shared `cxx_build::Build` with `-ffp-contract=off`. Reached here through
    // `slic3r-proof`, which re-exports the FFI crate's per-unit proof bridges -- the one
    // sanctioned route (`docs/PORTING.md`, "Decisions recorded on 2026-11-25"), needing no
    // dependency of this crate's own beyond the dev-dependency on `slic3r-proof` it already has.
    use slic3r_proof::proof_units::print_config_constants::{
        path1_initial_layer_height, path1_initial_raft_layers, path1_initial_reduce_crossing_wall,
    };

    // PROOF: 1
    #[test]
    fn print_config_constants_match_the_oracle() {
        let oracle_layer_height = path1_initial_layer_height();
        let oracle_raft_layers = path1_initial_raft_layers();
        let oracle_reduce_crossing_wall = path1_initial_reduce_crossing_wall();

        // Exact equality with no tolerance, as this row's `exact` equality tier requires:
        // compared on the `f64` bit pattern, so a value that merely prints the same cannot pass.
        assert_eq!(
            INITIAL_LAYER_HEIGHT.to_bits(),
            oracle_layer_height.to_bits(),
            "INITIAL_LAYER_HEIGHT: ported {INITIAL_LAYER_HEIGHT:?} against oracle \
             {oracle_layer_height:?}"
        );
        assert_eq!(
            INITIAL_RAFT_LAYERS, oracle_raft_layers,
            "INITIAL_RAFT_LAYERS"
        );
        assert_eq!(
            INITIAL_REDUCE_CROSSING_WALL, oracle_reduce_crossing_wall,
            "INITIAL_REDUCE_CROSSING_WALL"
        );
    }
}

// PORT STATUS
// source: src/libslic3r/PrintConfigConstants.hpp
// confidence: high
// tier: exact
// oracle: built
