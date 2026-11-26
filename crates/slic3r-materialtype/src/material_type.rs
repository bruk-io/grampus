//! Port of `MaterialType.cpp`/`MaterialType.hpp`
//! (`vendor/OrcaSlicer/src/libslic3r/MaterialType.{cpp,hpp}`): a static table of per-material
//! printing properties (temperature ranges, adhesion, yield strength, thermal length), looked
//! up by name. No dependency on any other port crate -- the vendored source includes only
//! `<string>`, `<vector>` and `<algorithm>`.
//!
//! **The fixed-point (near-miss) leg of this task's trial** (`intent/libslic3r-rust-port/tasks/35-trial-run/task.md`):
//! an `exact`-tier row whose ported surface returns plain integer output, not a spatial
//! coordinate -- recorded precisely rather than inflated, since `Point` does not exist on `main`
//! yet (see this task's pull request body and its plan's Step 2). It doubles as a second,
//! independent demonstration of the idiom map's "a non-const reference out-parameter -> a
//! return value, or a tuple of return values" rule (`docs/PORTING.md`), alongside the
//! throw/out-param leg (`slic3r-instancelock`), which exercises the same idiom against a
//! `throw` instead of a plain miss.
//!
//! **Idiom:** each `bool get_X(const std::string&, Out&...)` always writes its default into the
//! out-param(s) and then overwrites them on a hit, returning whether it was a hit. The port
//! returns a tuple of `(..out values.., found: bool)` rather than adopting `&mut` parameters,
//! preserving the "always set, even on a miss" behaviour a caller that ignores the bool still
//! observes in the vendored source.

/// `Slic3r::MaterialTypeInfo` (`MaterialType.hpp:8-16`): `(name, min_temp, max_temp,
/// chamber_min_temp, chamber_max_temp, adhesion_coefficient, yield_strength, thermal_length)`,
/// in the vendored struct's own field order. A tuple struct rather than named fields so the
/// 75-row table below stays one row per source line (`cargo fmt` wraps a long named-field
/// literal across several lines, which would separate a row's own `// PORT SRC:` marker -- the
/// one directly above it -- from the later fields R16 holds to the same citation).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaterialTypeInfo(
    pub &'static str,
    pub i32,
    pub i32,
    pub i32,
    pub i32,
    pub f64,
    pub f64,
    pub f64,
);

// PORT SRC: src/libslic3r/MaterialType.cpp:9
const DEFAULT_MIN_TEMP: i32 = 190;
// PORT SRC: src/libslic3r/MaterialType.cpp:10
const DEFAULT_MAX_TEMP: i32 = 300;
// PORT SRC: src/libslic3r/MaterialType.cpp:12
const DEFAULT_CHAMBER_MAX_TEMP: i32 = 100;
// PORT SRC: src/libslic3r/MaterialType.cpp:14
const DEFAULT_YIELD_STRENGTH: f64 = 0.02;
// PORT SRC: src/libslic3r/MaterialType.cpp:15
const DEFAULT_THERMAL_LENGTH: f64 = 200.0;
// `DEFAULT_CHAMBER_MIN_TEMP` (`:11`) and `DEFAULT_ADHESION_COEFFICIENT` (`:13`) are both `0`/`1`
// (R16's exempt literal set) and need no `// PORT SRC:` marker of their own.
const DEFAULT_CHAMBER_MIN_TEMP: i32 = 0;
const DEFAULT_ADHESION_COEFFICIENT: f64 = 1f64;

/// `MaterialType::all()`'s static table (`MaterialType.cpp:18-96`), transcribed row for row --
/// one `// PORT SRC:` marker per row, since every number on a vendored row is held by that one
/// cited line (`port/port-status`'s `check_numeric_constants` matches any literal on a Rust
/// line against any marker covering it, not one marker per literal). A whole-number field that
/// the vendored source also writes as a bare integer (every `min_temp`/`max_temp`/
/// `chamber_min_temp`/`chamber_max_temp`, and an `adhesion_coefficient`/`thermal_length` that
/// happens to be a whole number) is written with an explicit `f64` suffix rather than a decimal
/// point, so its normalised text still matches the vendored line's bare integer digits.
// No explicit element count: `75` is a count of table rows, not a value transcribed from any
// one vendored line, so it is left for the compiler to infer from the initialiser's own length
// rather than given a `// PORT SRC:` marker that would misattribute it to one row among many.
static MATERIAL_TYPES: &[MaterialTypeInfo] = &[
    // PORT SRC: src/libslic3r/MaterialType.cpp:21
    MaterialTypeInfo("ABS", 190, 300, 50, 65, 1f64, 0.1, 100f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:22
    MaterialTypeInfo("ABS-CF", 220, 300, 50, 65, 1f64, 0.1, 100f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:23
    MaterialTypeInfo("ABS-GF", 240, 280, 50, 65, 1f64, 0.1, 100f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:24
    MaterialTypeInfo("ASA", 220, 300, 50, 65, 1f64, 0.1, 100f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:25
    MaterialTypeInfo("ASA-CF", 230, 300, 50, 65, 1f64, 0.1, 100f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:26
    MaterialTypeInfo("ASA-GF", 240, 300, 50, 65, 1f64, 0.1, 100f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:27
    MaterialTypeInfo("ASA-AERO", 240, 280, 50, 65, 1f64, 0.1, 100f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:28
    MaterialTypeInfo("BVOH", 190, 240, 0, 70, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:29
    MaterialTypeInfo("CoPE", 190, 240, 0, 45, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:30
    MaterialTypeInfo("EVA", 175, 220, 0, 50, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:31
    MaterialTypeInfo("FLEX", 210, 230, 0, 50, 0.5, 0.02, 1000f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:32
    MaterialTypeInfo("HIPS", 220, 270, 50, 60, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:33
    MaterialTypeInfo("PA", 235, 280, 50, 60, 1f64, 0.02, 100f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:34
    MaterialTypeInfo("PA-CF", 240, 315, 50, 60, 1f64, 0.02, 100f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:35
    MaterialTypeInfo("PA-GF", 240, 290, 50, 60, 1f64, 0.02, 100f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:36
    MaterialTypeInfo("PA6", 260, 300, 50, 60, 1f64, 0.02, 100f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:37
    MaterialTypeInfo("PA6-CF", 230, 300, 50, 60, 1f64, 0.02, 100f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:38
    MaterialTypeInfo("PA6-GF", 260, 300, 50, 60, 1f64, 0.02, 100f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:39
    MaterialTypeInfo("PA11", 275, 295, 50, 60, 1f64, 0.02, 100f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:40
    MaterialTypeInfo("PA11-CF", 275, 295, 50, 60, 1f64, 0.02, 100f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:41
    MaterialTypeInfo("PA11-GF", 275, 295, 50, 60, 1f64, 0.02, 100f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:42
    MaterialTypeInfo("PA12", 250, 270, 50, 60, 1f64, 0.02, 100f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:43
    MaterialTypeInfo("PA12-CF", 250, 300, 50, 60, 1f64, 0.02, 100f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:44
    MaterialTypeInfo("PA12-GF", 255, 270, 50, 60, 1f64, 0.02, 100f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:45
    MaterialTypeInfo("PAHT", 260, 310, 55, 65, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:46
    MaterialTypeInfo("PAHT-CF", 270, 310, 55, 65, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:47
    MaterialTypeInfo("PAHT-GF", 270, 310, 55, 65, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:48
    MaterialTypeInfo("PC", 240, 300, 60, 70, 1f64, 0.02, 40f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:49
    MaterialTypeInfo("PC-ABS", 230, 270, 60, 70, 1f64, 0.02, 80f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:50
    MaterialTypeInfo("PC-CF", 270, 295, 60, 70, 1f64, 0.02, 80f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:51
    MaterialTypeInfo("PC-PBT", 260, 300, 60, 70, 1f64, 0.02, 40f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:52
    MaterialTypeInfo("PCL", 130, 170, 0, 45, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:53
    MaterialTypeInfo("PCTG", 220, 300, 0, 55, 2f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:54
    MaterialTypeInfo("PE", 175, 260, 45, 60, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:55
    MaterialTypeInfo("PE-CF", 175, 260, 45, 60, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:56
    MaterialTypeInfo("PE-GF", 230, 270, 45, 60, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:57
    MaterialTypeInfo("PEI-1010", 370, 430, 80, 100, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:58
    MaterialTypeInfo("PEI-1010-CF", 380, 430, 80, 100, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:59
    MaterialTypeInfo("PEI-1010-GF", 380, 430, 80, 100, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:60
    MaterialTypeInfo("PEI-9085", 350, 390, 80, 100, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:61
    MaterialTypeInfo("PEI-9085-CF", 365, 390, 80, 100, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:62
    MaterialTypeInfo("PEI-9085-GF", 370, 390, 80, 100, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:63
    MaterialTypeInfo("PEEK", 350, 460, 80, 100, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:64
    MaterialTypeInfo("PEEK-CF", 380, 410, 80, 100, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:65
    MaterialTypeInfo("PEEK-GF", 375, 410, 80, 100, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:66
    MaterialTypeInfo("PEKK", 325, 400, 80, 100, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:67
    MaterialTypeInfo("PEKK-CF", 360, 400, 80, 100, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:68
    MaterialTypeInfo("PES", 340, 390, 80, 100, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:69
    MaterialTypeInfo("PET", 200, 290, 0, 55, 2f64, 0.3, 100f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:70
    MaterialTypeInfo("PET-CF", 240, 320, 0, 55, 2f64, 0.3, 100f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:71
    MaterialTypeInfo("PET-GF", 280, 320, 0, 55, 2f64, 0.3, 100f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:72
    MaterialTypeInfo("PETG", 190, 260, 0, 55, 2f64, 0.3, 100f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:73
    MaterialTypeInfo("PETG-CF", 230, 290, 0, 55, 1f64, 0.3, 100f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:74
    MaterialTypeInfo("PETG-GF", 210, 270, 0, 55, 1f64, 0.3, 100f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:75
    MaterialTypeInfo("PHA", 190, 250, 0, 55, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:76
    MaterialTypeInfo("PI", 390, 410, 90, 100, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:77
    MaterialTypeInfo("PLA", 180, 240, 0, 45, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:78
    MaterialTypeInfo("PLA-AERO", 220, 270, 0, 55, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:79
    MaterialTypeInfo("PLA-CF", 190, 250, 0, 50, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:80
    MaterialTypeInfo("POM", 210, 250, 50, 65, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:81
    MaterialTypeInfo("PP", 200, 240, 45, 60, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:82
    MaterialTypeInfo("PP-CF", 210, 250, 45, 60, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:83
    MaterialTypeInfo("PP-GF", 220, 260, 45, 60, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:84
    MaterialTypeInfo("PPA-CF", 260, 300, 55, 70, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:85
    MaterialTypeInfo("PPA-GF", 260, 290, 55, 70, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:86
    MaterialTypeInfo("PPS", 300, 345, 90, 100, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:87
    MaterialTypeInfo("PPS-CF", 295, 350, 90, 100, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:88
    MaterialTypeInfo("PPSU", 360, 420, 90, 100, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:89
    MaterialTypeInfo("PSU", 350, 380, 90, 100, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:90
    MaterialTypeInfo("PVA", 185, 250, 0, 60, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:91
    MaterialTypeInfo("PVB", 190, 250, 0, 55, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:92
    MaterialTypeInfo("PVDF", 245, 265, 40, 60, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:93
    MaterialTypeInfo("SBS", 195, 250, 0, 55, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:94
    MaterialTypeInfo("TPI", 420, 445, 90, 100, 1f64, 0.02, 200f64),
    // PORT SRC: src/libslic3r/MaterialType.cpp:95
    MaterialTypeInfo("TPU", 175, 260, 0, 50, 0.5, 0.02, 1000f64),
];

/// `MaterialType::find()` (`MaterialType.cpp:101-106`).
fn find(name: &str) -> Option<&'static MaterialTypeInfo> {
    MATERIAL_TYPES.iter().find(|info| info.0 == name)
}

/// `MaterialType::get_temperature_range()` (`MaterialType.cpp:108-120`): returns
/// `(min_temp, max_temp, found)` -- the two out-params plus the bool, always set to the default
/// pair first and then overwritten on a hit, exactly as the vendored function's out-params are.
pub fn get_temperature_range(material_type: &str) -> (i32, i32, bool) {
    match find(material_type) {
        Some(info) => (info.1, info.2, true),
        None => (DEFAULT_MIN_TEMP, DEFAULT_MAX_TEMP, false),
    }
}

/// `MaterialType::get_chamber_temperature_range()` (`MaterialType.cpp:122-134`).
pub fn get_chamber_temperature_range(material_type: &str) -> (i32, i32, bool) {
    match find(material_type) {
        Some(info) => (info.3, info.4, true),
        None => (DEFAULT_CHAMBER_MIN_TEMP, DEFAULT_CHAMBER_MAX_TEMP, false),
    }
}

/// `MaterialType::get_adhesion_coefficient()` (`MaterialType.cpp:136-146`).
pub fn get_adhesion_coefficient(material_type: &str) -> (f64, bool) {
    match find(material_type) {
        Some(info) => (info.5, true),
        None => (DEFAULT_ADHESION_COEFFICIENT, false),
    }
}

/// `MaterialType::get_yield_strength()` (`MaterialType.cpp:148-158`).
pub fn get_yield_strength(material_type: &str) -> (f64, bool) {
    match find(material_type) {
        Some(info) => (info.6, true),
        None => (DEFAULT_YIELD_STRENGTH, false),
    }
}

/// `MaterialType::get_thermal_length()` (`MaterialType.cpp:160-170`).
pub fn get_thermal_length(material_type: &str) -> (f64, bool) {
    match find(material_type) {
        Some(info) => (info.7, true),
        None => (DEFAULT_THERMAL_LENGTH, false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slic3r_proof::proof_units::material_type;
    use slic3r_proof::{compare_unit, Oracle};

    /// A few real names, a prefix-only near-miss and a wholly unknown name, so both the hit and
    /// the miss (default-filled, `found: false`) path run against the oracle.
    const SAMPLE_MATERIALS: &[&str] = &[
        "ABS",
        "PLA",
        "PETG-CF",
        "TPU",
        "PEI-9085-GF",
        "AB",
        "Unobtainium",
    ];

    /// Both the `MaterialType.cpp` and `MaterialType.hpp` manifest rows declare proof path `3`
    /// (see `cxx/proof_units/material_type.cpp`'s header comment for why the mechanism is a
    /// direct external call rather than including the vendored `.cpp` whole), so one test
    /// carrying one `// PROOF: 3` marker satisfies `port/port-status`'s `proof_present` check
    /// for each row independently -- it matches the marker's text against each row's own cell,
    /// and both cells hold the same text. Both `compare_unit` calls below are kept, one per row,
    /// so each row's own tier is genuinely exercised rather than assumed from the other's.
    // PROOF: 3
    #[test]
    fn material_type_get_x_matches_the_oracle() {
        for &name in SAMPLE_MATERIALS {
            let oracle_temp = material_type::path_temperature_range(name);
            let (min_temp, max_temp, found) = get_temperature_range(name);
            let ported_temp = vec![min_temp as i64, max_temp as i64, i64::from(found)];
            compare_unit(
                "MaterialType.cpp",
                Oracle::Built,
                ported_temp.clone(),
                oracle_temp.clone(),
            )
            .unwrap_or_else(|e| panic!("{name}: get_temperature_range: {e}"));
            compare_unit("MaterialType.hpp", Oracle::Built, ported_temp, oracle_temp)
                .unwrap_or_else(|e| panic!("{name}: get_temperature_range: {e}"));

            let oracle_chamber = material_type::path_chamber_temperature_range(name);
            let (chamber_min, chamber_max, chamber_found) = get_chamber_temperature_range(name);
            let ported_chamber = vec![
                chamber_min as i64,
                chamber_max as i64,
                i64::from(chamber_found),
            ];
            compare_unit(
                "MaterialType.cpp",
                Oracle::Built,
                ported_chamber.clone(),
                oracle_chamber.clone(),
            )
            .unwrap_or_else(|e| panic!("{name}: get_chamber_temperature_range: {e}"));
            compare_unit(
                "MaterialType.hpp",
                Oracle::Built,
                ported_chamber,
                oracle_chamber,
            )
            .unwrap_or_else(|e| panic!("{name}: get_chamber_temperature_range: {e}"));

            let oracle_adhesion = material_type::path_adhesion_coefficient(name);
            let (adhesion, adhesion_found) = get_adhesion_coefficient(name);
            let ported_adhesion = vec![adhesion, if adhesion_found { 1.0 } else { 0.0 }];
            compare_unit(
                "MaterialType.cpp",
                Oracle::Built,
                ported_adhesion.clone(),
                oracle_adhesion.clone(),
            )
            .unwrap_or_else(|e| panic!("{name}: get_adhesion_coefficient: {e}"));
            compare_unit(
                "MaterialType.hpp",
                Oracle::Built,
                ported_adhesion,
                oracle_adhesion,
            )
            .unwrap_or_else(|e| panic!("{name}: get_adhesion_coefficient: {e}"));

            let oracle_yield = material_type::path_yield_strength(name);
            let (yield_strength, yield_found) = get_yield_strength(name);
            let ported_yield = vec![yield_strength, if yield_found { 1.0 } else { 0.0 }];
            compare_unit(
                "MaterialType.cpp",
                Oracle::Built,
                ported_yield.clone(),
                oracle_yield.clone(),
            )
            .unwrap_or_else(|e| panic!("{name}: get_yield_strength: {e}"));
            compare_unit(
                "MaterialType.hpp",
                Oracle::Built,
                ported_yield,
                oracle_yield,
            )
            .unwrap_or_else(|e| panic!("{name}: get_yield_strength: {e}"));

            let oracle_thermal = material_type::path_thermal_length(name);
            let (thermal_length, thermal_found) = get_thermal_length(name);
            let ported_thermal = vec![thermal_length, if thermal_found { 1.0 } else { 0.0 }];
            compare_unit(
                "MaterialType.cpp",
                Oracle::Built,
                ported_thermal.clone(),
                oracle_thermal.clone(),
            )
            .unwrap_or_else(|e| panic!("{name}: get_thermal_length: {e}"));
            compare_unit(
                "MaterialType.hpp",
                Oracle::Built,
                ported_thermal,
                oracle_thermal,
            )
            .unwrap_or_else(|e| panic!("{name}: get_thermal_length: {e}"));
        }
    }
}

// PORT STATUS
// source: src/libslic3r/MaterialType.cpp
// source: src/libslic3r/MaterialType.hpp
// confidence: high
// tier: exact
// oracle: built
