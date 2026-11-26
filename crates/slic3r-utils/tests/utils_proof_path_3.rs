//! Proof for the `Utils.hpp`/`utils.cpp` manifest row (task 57-foundation-utils): `Slic3r::
//! get_utf8_sequence_length(const char*, size_t)` (`utils.cpp:1390-1452`), called through
//! `crates/slic3r-ffi/cxx/proof_units/utils.cpp`'s shim, reached through `slic3r-proof`'s
//! re-export of the FFI crate's per-unit proof bridges -- the one sanctioned route
//! (`docs/PORTING.md`, "Decisions recorded on 2026-11-25").
//!
//! Marked `// PROOF: 3` because the manifest row's `proof path` cell is `3`
//! (`port/port-status`'s `proof_present` matches that row's own cell verbatim); the shim itself
//! is shaped like proof path 1, not path 3 -- see `cxx/proof_units/utils.cpp`'s header comment
//! for the symbol-count measurement that rules the literal `#include` of `utils.cpp` out, and
//! this task's pull request body for the same note.
//!
//! Every input is non-empty: both sides read byte `0` unconditionally with no length check
//! (the vendored function has no guard either, `utils.cpp:1393`), so an empty input is not a
//! meaningful comparison.

use slic3r_proof::proof_units::utils::path1_get_utf8_sequence_length;
use slic3r_utils::utils::get_utf8_sequence_length;

/// `Slic3r::get_utf8_sequence_length` against this port's own [`get_utf8_sequence_length`],
/// across every leading-byte range the function branches on: plain ASCII, a lone continuation
/// byte, each valid multi-byte header length (2 through 6 bytes), a header byte whose declared
/// length runs past the buffer's end (in range and truncated), and the two `0xFE`/`0xFF`
/// "not a UTF-8 sequence" bytes.
// PROOF: 3
#[test]
fn utils_get_utf8_sequence_length_matches_the_oracle() {
    let cases: &[&[u8]] = &[
        b"a",
        b"hello world",
        &[0x80],
        &[0x80, 0x81, 0x82, b'x'],
        &[0xC2, 0xA9],
        &[0xC2],
        &[0xE2, 0x82, 0xAC],
        &[0xE2, 0x82],
        &[0xF0, 0x9F, 0x98, 0x80],
        &[0xF0, 0x9F],
        &[0xF8, 0x80, 0x80, 0x80, 0x80],
        &[0xF8, 0x80],
        &[0xFC, 0x80, 0x80, 0x80, 0x80, 0x80],
        &[0xFC],
        &[0xFE],
        &[0xFF],
        &[0xFE, 0x00],
    ];

    for seq in cases {
        let oracle_len = path1_get_utf8_sequence_length(seq.to_vec());
        let ported_len = get_utf8_sequence_length(seq) as u64;
        assert_eq!(oracle_len, ported_len, "mismatch for {seq:?}");
    }
}
