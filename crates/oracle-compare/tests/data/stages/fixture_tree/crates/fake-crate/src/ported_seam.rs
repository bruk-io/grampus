// Fixture module for `crates/oracle-compare/tests/stages.rs`: reached by `lib.rs`'s `mod`
// line, holds no `todo!()`/`unimplemented!()`, and carries a `// PROOF:` marker naming
// `proof/ported_seam` beside a `#[test]` function whose name contains this file's own stem
// ("ported_seam") -- `row_is_ported`'s three conditions, all satisfied, so every manifest row
// in the fixture manifest that points at this file derives `ported`.

pub fn stub() {}

#[cfg(test)]
mod tests {
    // PROOF: proof/ported_seam
    #[test]
    fn test_ported_seam() {}
}
