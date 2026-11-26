//! Ports `vendor/OrcaSlicer/src/libslic3r/CommonDefs.hpp` (crate-level level 0, hand-designed
//! by plan step 20's task `52-foundation-common-defs`, `port/foundation.tsv` row
//! `foundation-01`): one plain (unscoped) C++ `enum`, `Slic3r::NozzleType`, with no dependent
//! among this task's own units and no `port/ownership.tsv` row (no raw-pointer field at the
//! pin).
//!
//! **Decision (task was silent): variant names kept verbatim.** The vendored identifiers
//! (`ntUndefine`, `ntHardenedSteel`, ...) are not Rust's `PascalCase` convention, but
//! `docs/WORKFLOW.md`'s "The rejection rule" and this task's own "Faithfulness, not idiom" rule
//! leave idiom to the following intent; renaming the variants here would make every later
//! fan-out call site that reads this type guess at a second spelling. `#[allow(non_camel_case_types)]`
//! silences the one resulting `cargo` lint; nothing in `.asdlc/checks` or `just check` denies it.
//!
//! **No `// PORT SRC:` marker is needed.** Only `ntUndefine = 0` (`CommonDefs.hpp:14`) writes a
//! literal, and `0` is in `port/port-status`'s exempt set; every other variant is implicit
//! (`CommonDefs.hpp:15-20`), so Rust's own implicit `+1` numbering reproduces the enum exactly
//! with no literal of its own to cite.
//!
//! **Proof path 2, reached via `slic3r-proof`'s re-export of `slic3r-ffi`'s per-unit proof
//! bridges (`docs/PORTING.md`, "Decisions recorded on 2026-11-25", decision 1 and decision 4).**
//! `port/manifest.tsv`'s `proof path` cell for this row is `2`, seeded -- per task.md's own
//! words -- "from a row's own pipeline stage" (`posSlice`), not from a verified dump field:
//! `NozzleType` is a machine/printer property, never written into any `posSlice` (or any other)
//! stage dump (checked with
//! `grep -n "^field=" port/dumps/*/*.dump | awk -F'field=' '{print $2}' | awk '{print $1}' | sort -u`,
//! whose real output -- fixed here by adversarial review, which found the earlier wording had
//! dropped two fields and collapsed seven more -- is: `fill_surfaces, fills, height, lslices,
//! object_skirt, perimeters, print_z, raw_slices, skirt, skirt_brim_group_brim,
//! skirt_brim_group_skirt, skirt_brim_groups, slice_z, slices, thin_fills, wipe_tower_bbx,
//! wipe_tower_brim_width, wipe_tower_depth, wipe_tower_height,
//! wipe_tower_number_of_toolchanges, wipe_tower_rib_offset, wipe_tower_width` -- still no field
//! carrying a `NozzleType` value, so the conclusion the grep supports is unchanged). Decision 4
//! names the per-unit proof bridges, not only the dump corpus, as a "2" row's oracle entry point
//! -- the same answer `ObjectID.hpp`'s `path2_header_observations` is proven by -- so this unit's
//! shim (`crates/slic3r-ffi/cxx/proof_units/common_defs.{h,cpp}`,
//! `crates/slic3r-ffi/src/proof_units/common_defs.rs`) drives the vendored header's own compiled
//! enumerators directly.

#![allow(non_camel_case_types)]

/// `Slic3r::NozzleType` (`CommonDefs.hpp:11-21`), a plain (unscoped) C++ `enum`. The type map's
/// `enum class` row ("keeping the original's discriminant values") is applied the same way to
/// this unscoped enum: one Rust variant per C++ enumerator, in the same order, with the same
/// explicit/implicit discriminants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum NozzleType {
    ntUndefine = 0,
    ntHardenedSteel,
    ntStainlessSteel,
    ntTungstenCarbide,
    ntBrass,
    ntE3D,
    ntCount,
}

#[cfg(test)]
mod tests {
    use super::NozzleType;

    /// A plain correctness check that this port's discriminants match `CommonDefs.hpp:14-20`
    /// one for one. Not a `// PROOF:` marker -- see `common_defs_nozzle_type_matches_the_oracle`
    /// below for that.
    #[test]
    fn common_defs_nozzle_type_discriminants_match_the_vendored_enum() {
        assert_eq!(NozzleType::ntUndefine as i32, 0);
        assert_eq!(NozzleType::ntHardenedSteel as i32, 1);
        assert_eq!(NozzleType::ntStainlessSteel as i32, 2);
        assert_eq!(NozzleType::ntTungstenCarbide as i32, 3);
        assert_eq!(NozzleType::ntBrass as i32, 4);
        assert_eq!(NozzleType::ntE3D as i32, 5);
        assert_eq!(NozzleType::ntCount as i32, 6);
    }

    /// `Slic3r::NozzleType`'s seven enumerators (`CommonDefs.hpp:11-20`), read from the
    /// vendored header's own compiled definition through `slic3r-proof`'s re-export of
    /// `slic3r-ffi`'s per-unit proof bridge (`docs/PORTING.md`, "Decisions recorded on
    /// 2026-11-25", decisions 1 and 4), against this port's own discriminants.
    // PROOF: 2
    #[test]
    fn common_defs_nozzle_type_matches_the_oracle() {
        use slic3r_proof::proof_units::common_defs::path2_nozzle_type_discriminants;
        use slic3r_proof::{compare_unit, Oracle};

        // No `let ... : Vec<i64>` type annotation on either binding below: `port/port-status`'s
        // binding-taint pass marks a `let` pattern's type annotation tokens as oracle names, so
        // an annotated `oracle`/`ported` pair here would taint both sides and lose this row's
        // differential-comparison evidence. Not fixable in `port/port-status` itself from this
        // task (that script is on the must-not-write list), so the annotation stays off rather
        // than reintroduced by a later refactor.
        let oracle = path2_nozzle_type_discriminants();
        let ported = vec![
            NozzleType::ntUndefine as i64,
            NozzleType::ntHardenedSteel as i64,
            NozzleType::ntStainlessSteel as i64,
            NozzleType::ntTungstenCarbide as i64,
            NozzleType::ntBrass as i64,
            NozzleType::ntE3D as i64,
            NozzleType::ntCount as i64,
        ];
        compare_unit("CommonDefs.hpp", Oracle::Built, ported, oracle)
            .expect("NozzleType's seven discriminants against the oracle");
    }
}

// PORT STATUS
// source: src/libslic3r/CommonDefs.hpp
// confidence: high
// tier: exact
// oracle: built
