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
//! **Proof path 2 could not be honoured as a real proof (a finding for this pull request, not a
//! tolerance or a stand-in invented on this branch).** `port/manifest.tsv`'s `proof path` cell
//! for this row is `2` (the committed stage dumps, read through `crates/slic3r-proof/src/dumps.rs`),
//! seeded — per task.md's own words — "from a row's own pipeline stage" (`posSlice`), not from a
//! verified dump field. `NozzleType` is a machine/printer property, never written into any
//! `posSlice` (or any other) stage dump: every field name across the committed corpus was
//! checked --
//! `grep -n "^field=" port/dumps/*/*.dump | awk -F'field=' '{print $2}' | awk '{print $1}' | sort -u`
//! -- and lists only `fill_surfaces, fills, height, lslices, object_skirt, perimeters, print_z,
//! raw_slices, skirt, skirt_brim_groups, slice_z, slices, thin_fills, wipe_tower_*`; a further
//! `grep -i "nozzle"` over every dump matches only a preset *filename* in a provenance comment
//! (`machine/Bambu Lab X1 Carbon 0.4 nozzle.json`), never a `NozzleType` value. There is no
//! dump-derived oracle value for this unit to compare against, so writing a `// PROOF: 2` test
//! that reads a dump and asserts something about it, unconnected to `NozzleType`, would be
//! exactly the fake-green "a test that does not exercise the real implementation" the requester's
//! honesty rules forbid. This module therefore carries no `// PROOF:` marker and `port/port-status`
//! derives it as `drafted`, not `ported` -- the status `derive_row_status` defines for "a `mod`
//! declaration reaches the target file ... but ... its proof is not yet present", which this is.

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

    /// Not a `// PROOF:` marker (see this module's doc comment: no dump field holds this
    /// unit's output, so proof path 2 cannot be honoured here) -- a plain correctness check that
    /// this port's discriminants match `CommonDefs.hpp:14-20` one for one.
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
}

// PORT STATUS
// source: src/libslic3r/CommonDefs.hpp
// confidence: high
// tier: exact
// oracle: built
