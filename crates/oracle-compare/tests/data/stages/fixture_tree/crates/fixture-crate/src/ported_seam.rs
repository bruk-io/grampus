// Fixture module for `crates/oracle-compare/tests/stages.rs`: reached by `lib.rs`'s `mod`
// line, holds no `todo!()`/`unimplemented!()`, and carries a `// PROOF:` marker naming
// `proof/ported_seam` beside a `#[test]` function whose name contains this file's own stem
// ("ported_seam") -- `row_is_ported`'s three conditions, all satisfied, so every manifest row
// in the fixture manifest that points at this file derives `ported`.

pub fn value() -> i64 {
    7
}

#[cfg(test)]
mod tests {
    use super::value;
    use slic3r_proof::proof_units::ported_seam::path1_value;

    // PROOF: proof/ported_seam
    #[test]
    fn test_ported_seam() {
        // The differential comparison the `// PROOF:` marker needs to count (docs/PORTING.md,
        // "Decisions recorded on 2026-11-25"): a value the oracle produced, reached through
        // `slic3r-proof`'s re-export of the per-unit proof bridges, against the value this
        // module computed. This tree is never compiled -- it is read as text by
        // `row_is_ported` -- so the import and the call stand for the shape.
        let oracle = path1_value();
        assert_eq!(value(), oracle);
    }
}
