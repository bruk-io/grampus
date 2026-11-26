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

/// `Slic3r::MaterialTypeInfo` (`MaterialType.hpp:8-16`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaterialTypeInfo {
    pub name: &'static str,
    pub min_temp: i32,
    pub max_temp: i32,
    pub chamber_min_temp: i32,
    pub chamber_max_temp: i32,
    pub adhesion_coefficient: f64,
    pub yield_strength: f64,
    pub thermal_length: f64,
}

// PORT SRC: MaterialType.cpp:9
const DEFAULT_MIN_TEMP: i32 = 190;
// PORT SRC: MaterialType.cpp:10
const DEFAULT_MAX_TEMP: i32 = 300;
// PORT SRC: MaterialType.cpp:12
const DEFAULT_CHAMBER_MAX_TEMP: i32 = 100;
// PORT SRC: MaterialType.cpp:14
const DEFAULT_YIELD_STRENGTH: f64 = 0.02;
// PORT SRC: MaterialType.cpp:15
const DEFAULT_THERMAL_LENGTH: f64 = 200f64;
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
static MATERIAL_TYPES: [MaterialTypeInfo; 75] = [
    // PORT SRC: MaterialType.cpp:21
    MaterialTypeInfo { name: "ABS", min_temp: 190, max_temp: 300, chamber_min_temp: 50, chamber_max_temp: 65, adhesion_coefficient: 1f64, yield_strength: 0.1, thermal_length: 100f64 },
    // PORT SRC: MaterialType.cpp:22
    MaterialTypeInfo { name: "ABS-CF", min_temp: 220, max_temp: 300, chamber_min_temp: 50, chamber_max_temp: 65, adhesion_coefficient: 1f64, yield_strength: 0.1, thermal_length: 100f64 },
    // PORT SRC: MaterialType.cpp:23
    MaterialTypeInfo { name: "ABS-GF", min_temp: 240, max_temp: 280, chamber_min_temp: 50, chamber_max_temp: 65, adhesion_coefficient: 1f64, yield_strength: 0.1, thermal_length: 100f64 },
    // PORT SRC: MaterialType.cpp:24
    MaterialTypeInfo { name: "ASA", min_temp: 220, max_temp: 300, chamber_min_temp: 50, chamber_max_temp: 65, adhesion_coefficient: 1f64, yield_strength: 0.1, thermal_length: 100f64 },
    // PORT SRC: MaterialType.cpp:25
    MaterialTypeInfo { name: "ASA-CF", min_temp: 230, max_temp: 300, chamber_min_temp: 50, chamber_max_temp: 65, adhesion_coefficient: 1f64, yield_strength: 0.1, thermal_length: 100f64 },
    // PORT SRC: MaterialType.cpp:26
    MaterialTypeInfo { name: "ASA-GF", min_temp: 240, max_temp: 300, chamber_min_temp: 50, chamber_max_temp: 65, adhesion_coefficient: 1f64, yield_strength: 0.1, thermal_length: 100f64 },
    // PORT SRC: MaterialType.cpp:27
    MaterialTypeInfo { name: "ASA-AERO", min_temp: 240, max_temp: 280, chamber_min_temp: 50, chamber_max_temp: 65, adhesion_coefficient: 1f64, yield_strength: 0.1, thermal_length: 100f64 },
    // PORT SRC: MaterialType.cpp:28
    MaterialTypeInfo { name: "BVOH", min_temp: 190, max_temp: 240, chamber_min_temp: 0, chamber_max_temp: 70, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:29
    MaterialTypeInfo { name: "CoPE", min_temp: 190, max_temp: 240, chamber_min_temp: 0, chamber_max_temp: 45, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:30
    MaterialTypeInfo { name: "EVA", min_temp: 175, max_temp: 220, chamber_min_temp: 0, chamber_max_temp: 50, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:31
    MaterialTypeInfo { name: "FLEX", min_temp: 210, max_temp: 230, chamber_min_temp: 0, chamber_max_temp: 50, adhesion_coefficient: 0.5, yield_strength: 0.02, thermal_length: 1000f64 },
    // PORT SRC: MaterialType.cpp:32
    MaterialTypeInfo { name: "HIPS", min_temp: 220, max_temp: 270, chamber_min_temp: 50, chamber_max_temp: 60, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:33
    MaterialTypeInfo { name: "PA", min_temp: 235, max_temp: 280, chamber_min_temp: 50, chamber_max_temp: 60, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 100f64 },
    // PORT SRC: MaterialType.cpp:34
    MaterialTypeInfo { name: "PA-CF", min_temp: 240, max_temp: 315, chamber_min_temp: 50, chamber_max_temp: 60, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 100f64 },
    // PORT SRC: MaterialType.cpp:35
    MaterialTypeInfo { name: "PA-GF", min_temp: 240, max_temp: 290, chamber_min_temp: 50, chamber_max_temp: 60, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 100f64 },
    // PORT SRC: MaterialType.cpp:36
    MaterialTypeInfo { name: "PA6", min_temp: 260, max_temp: 300, chamber_min_temp: 50, chamber_max_temp: 60, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 100f64 },
    // PORT SRC: MaterialType.cpp:37
    MaterialTypeInfo { name: "PA6-CF", min_temp: 230, max_temp: 300, chamber_min_temp: 50, chamber_max_temp: 60, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 100f64 },
    // PORT SRC: MaterialType.cpp:38
    MaterialTypeInfo { name: "PA6-GF", min_temp: 260, max_temp: 300, chamber_min_temp: 50, chamber_max_temp: 60, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 100f64 },
    // PORT SRC: MaterialType.cpp:39
    MaterialTypeInfo { name: "PA11", min_temp: 275, max_temp: 295, chamber_min_temp: 50, chamber_max_temp: 60, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 100f64 },
    // PORT SRC: MaterialType.cpp:40
    MaterialTypeInfo { name: "PA11-CF", min_temp: 275, max_temp: 295, chamber_min_temp: 50, chamber_max_temp: 60, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 100f64 },
    // PORT SRC: MaterialType.cpp:41
    MaterialTypeInfo { name: "PA11-GF", min_temp: 275, max_temp: 295, chamber_min_temp: 50, chamber_max_temp: 60, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 100f64 },
    // PORT SRC: MaterialType.cpp:42
    MaterialTypeInfo { name: "PA12", min_temp: 250, max_temp: 270, chamber_min_temp: 50, chamber_max_temp: 60, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 100f64 },
    // PORT SRC: MaterialType.cpp:43
    MaterialTypeInfo { name: "PA12-CF", min_temp: 250, max_temp: 300, chamber_min_temp: 50, chamber_max_temp: 60, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 100f64 },
    // PORT SRC: MaterialType.cpp:44
    MaterialTypeInfo { name: "PA12-GF", min_temp: 255, max_temp: 270, chamber_min_temp: 50, chamber_max_temp: 60, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 100f64 },
    // PORT SRC: MaterialType.cpp:45
    MaterialTypeInfo { name: "PAHT", min_temp: 260, max_temp: 310, chamber_min_temp: 55, chamber_max_temp: 65, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:46
    MaterialTypeInfo { name: "PAHT-CF", min_temp: 270, max_temp: 310, chamber_min_temp: 55, chamber_max_temp: 65, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:47
    MaterialTypeInfo { name: "PAHT-GF", min_temp: 270, max_temp: 310, chamber_min_temp: 55, chamber_max_temp: 65, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:48
    MaterialTypeInfo { name: "PC", min_temp: 240, max_temp: 300, chamber_min_temp: 60, chamber_max_temp: 70, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 40f64 },
    // PORT SRC: MaterialType.cpp:49
    MaterialTypeInfo { name: "PC-ABS", min_temp: 230, max_temp: 270, chamber_min_temp: 60, chamber_max_temp: 70, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 80f64 },
    // PORT SRC: MaterialType.cpp:50
    MaterialTypeInfo { name: "PC-CF", min_temp: 270, max_temp: 295, chamber_min_temp: 60, chamber_max_temp: 70, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 80f64 },
    // PORT SRC: MaterialType.cpp:51
    MaterialTypeInfo { name: "PC-PBT", min_temp: 260, max_temp: 300, chamber_min_temp: 60, chamber_max_temp: 70, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 40f64 },
    // PORT SRC: MaterialType.cpp:52
    MaterialTypeInfo { name: "PCL", min_temp: 130, max_temp: 170, chamber_min_temp: 0, chamber_max_temp: 45, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:53
    MaterialTypeInfo { name: "PCTG", min_temp: 220, max_temp: 300, chamber_min_temp: 0, chamber_max_temp: 55, adhesion_coefficient: 2f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:54
    MaterialTypeInfo { name: "PE", min_temp: 175, max_temp: 260, chamber_min_temp: 45, chamber_max_temp: 60, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:55
    MaterialTypeInfo { name: "PE-CF", min_temp: 175, max_temp: 260, chamber_min_temp: 45, chamber_max_temp: 60, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:56
    MaterialTypeInfo { name: "PE-GF", min_temp: 230, max_temp: 270, chamber_min_temp: 45, chamber_max_temp: 60, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:57
    MaterialTypeInfo { name: "PEI-1010", min_temp: 370, max_temp: 430, chamber_min_temp: 80, chamber_max_temp: 100, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:58
    MaterialTypeInfo { name: "PEI-1010-CF", min_temp: 380, max_temp: 430, chamber_min_temp: 80, chamber_max_temp: 100, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:59
    MaterialTypeInfo { name: "PEI-1010-GF", min_temp: 380, max_temp: 430, chamber_min_temp: 80, chamber_max_temp: 100, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:60
    MaterialTypeInfo { name: "PEI-9085", min_temp: 350, max_temp: 390, chamber_min_temp: 80, chamber_max_temp: 100, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:61
    MaterialTypeInfo { name: "PEI-9085-CF", min_temp: 365, max_temp: 390, chamber_min_temp: 80, chamber_max_temp: 100, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:62
    MaterialTypeInfo { name: "PEI-9085-GF", min_temp: 370, max_temp: 390, chamber_min_temp: 80, chamber_max_temp: 100, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:63
    MaterialTypeInfo { name: "PEEK", min_temp: 350, max_temp: 460, chamber_min_temp: 80, chamber_max_temp: 100, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:64
    MaterialTypeInfo { name: "PEEK-CF", min_temp: 380, max_temp: 410, chamber_min_temp: 80, chamber_max_temp: 100, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:65
    MaterialTypeInfo { name: "PEEK-GF", min_temp: 375, max_temp: 410, chamber_min_temp: 80, chamber_max_temp: 100, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:66
    MaterialTypeInfo { name: "PEKK", min_temp: 325, max_temp: 400, chamber_min_temp: 80, chamber_max_temp: 100, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:67
    MaterialTypeInfo { name: "PEKK-CF", min_temp: 360, max_temp: 400, chamber_min_temp: 80, chamber_max_temp: 100, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:68
    MaterialTypeInfo { name: "PES", min_temp: 340, max_temp: 390, chamber_min_temp: 80, chamber_max_temp: 100, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:69
    MaterialTypeInfo { name: "PET", min_temp: 200, max_temp: 290, chamber_min_temp: 0, chamber_max_temp: 55, adhesion_coefficient: 2f64, yield_strength: 0.3, thermal_length: 100f64 },
    // PORT SRC: MaterialType.cpp:70
    MaterialTypeInfo { name: "PET-CF", min_temp: 240, max_temp: 320, chamber_min_temp: 0, chamber_max_temp: 55, adhesion_coefficient: 2f64, yield_strength: 0.3, thermal_length: 100f64 },
    // PORT SRC: MaterialType.cpp:71
    MaterialTypeInfo { name: "PET-GF", min_temp: 280, max_temp: 320, chamber_min_temp: 0, chamber_max_temp: 55, adhesion_coefficient: 2f64, yield_strength: 0.3, thermal_length: 100f64 },
    // PORT SRC: MaterialType.cpp:72
    MaterialTypeInfo { name: "PETG", min_temp: 190, max_temp: 260, chamber_min_temp: 0, chamber_max_temp: 55, adhesion_coefficient: 2f64, yield_strength: 0.3, thermal_length: 100f64 },
    // PORT SRC: MaterialType.cpp:73
    MaterialTypeInfo { name: "PETG-CF", min_temp: 230, max_temp: 290, chamber_min_temp: 0, chamber_max_temp: 55, adhesion_coefficient: 1f64, yield_strength: 0.3, thermal_length: 100f64 },
    // PORT SRC: MaterialType.cpp:74
    MaterialTypeInfo { name: "PETG-GF", min_temp: 210, max_temp: 270, chamber_min_temp: 0, chamber_max_temp: 55, adhesion_coefficient: 1f64, yield_strength: 0.3, thermal_length: 100f64 },
    // PORT SRC: MaterialType.cpp:75
    MaterialTypeInfo { name: "PHA", min_temp: 190, max_temp: 250, chamber_min_temp: 0, chamber_max_temp: 55, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:76
    MaterialTypeInfo { name: "PI", min_temp: 390, max_temp: 410, chamber_min_temp: 90, chamber_max_temp: 100, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:77
    MaterialTypeInfo { name: "PLA", min_temp: 180, max_temp: 240, chamber_min_temp: 0, chamber_max_temp: 45, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:78
    MaterialTypeInfo { name: "PLA-AERO", min_temp: 220, max_temp: 270, chamber_min_temp: 0, chamber_max_temp: 55, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:79
    MaterialTypeInfo { name: "PLA-CF", min_temp: 190, max_temp: 250, chamber_min_temp: 0, chamber_max_temp: 50, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:80
    MaterialTypeInfo { name: "POM", min_temp: 210, max_temp: 250, chamber_min_temp: 50, chamber_max_temp: 65, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:81
    MaterialTypeInfo { name: "PP", min_temp: 200, max_temp: 240, chamber_min_temp: 45, chamber_max_temp: 60, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:82
    MaterialTypeInfo { name: "PP-CF", min_temp: 210, max_temp: 250, chamber_min_temp: 45, chamber_max_temp: 60, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:83
    MaterialTypeInfo { name: "PP-GF", min_temp: 220, max_temp: 260, chamber_min_temp: 45, chamber_max_temp: 60, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:84
    MaterialTypeInfo { name: "PPA-CF", min_temp: 260, max_temp: 300, chamber_min_temp: 55, chamber_max_temp: 70, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:85
    MaterialTypeInfo { name: "PPA-GF", min_temp: 260, max_temp: 290, chamber_min_temp: 55, chamber_max_temp: 70, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:86
    MaterialTypeInfo { name: "PPS", min_temp: 300, max_temp: 345, chamber_min_temp: 90, chamber_max_temp: 100, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:87
    MaterialTypeInfo { name: "PPS-CF", min_temp: 295, max_temp: 350, chamber_min_temp: 90, chamber_max_temp: 100, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:88
    MaterialTypeInfo { name: "PPSU", min_temp: 360, max_temp: 420, chamber_min_temp: 90, chamber_max_temp: 100, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:89
    MaterialTypeInfo { name: "PSU", min_temp: 350, max_temp: 380, chamber_min_temp: 90, chamber_max_temp: 100, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:90
    MaterialTypeInfo { name: "PVA", min_temp: 185, max_temp: 250, chamber_min_temp: 0, chamber_max_temp: 60, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:91
    MaterialTypeInfo { name: "PVB", min_temp: 190, max_temp: 250, chamber_min_temp: 0, chamber_max_temp: 55, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:92
    MaterialTypeInfo { name: "PVDF", min_temp: 245, max_temp: 265, chamber_min_temp: 40, chamber_max_temp: 60, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:93
    MaterialTypeInfo { name: "SBS", min_temp: 195, max_temp: 250, chamber_min_temp: 0, chamber_max_temp: 55, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:94
    MaterialTypeInfo { name: "TPI", min_temp: 420, max_temp: 445, chamber_min_temp: 90, chamber_max_temp: 100, adhesion_coefficient: 1f64, yield_strength: 0.02, thermal_length: 200f64 },
    // PORT SRC: MaterialType.cpp:95
    MaterialTypeInfo { name: "TPU", min_temp: 175, max_temp: 260, chamber_min_temp: 0, chamber_max_temp: 50, adhesion_coefficient: 0.5, yield_strength: 0.02, thermal_length: 1000f64 },
];

/// `MaterialType::find()` (`MaterialType.cpp:101-106`).
fn find(name: &str) -> Option<&'static MaterialTypeInfo> {
    MATERIAL_TYPES.iter().find(|info| info.name == name)
}

/// `MaterialType::get_temperature_range()` (`MaterialType.cpp:108-120`): returns
/// `(min_temp, max_temp, found)` -- the two out-params plus the bool, always set to the default
/// pair first and then overwritten on a hit, exactly as the vendored function's out-params are.
pub fn get_temperature_range(material_type: &str) -> (i32, i32, bool) {
    match find(material_type) {
        Some(info) => (info.min_temp, info.max_temp, true),
        None => (DEFAULT_MIN_TEMP, DEFAULT_MAX_TEMP, false),
    }
}

/// `MaterialType::get_chamber_temperature_range()` (`MaterialType.cpp:122-134`).
pub fn get_chamber_temperature_range(material_type: &str) -> (i32, i32, bool) {
    match find(material_type) {
        Some(info) => (info.chamber_min_temp, info.chamber_max_temp, true),
        None => (DEFAULT_CHAMBER_MIN_TEMP, DEFAULT_CHAMBER_MAX_TEMP, false),
    }
}

/// `MaterialType::get_adhesion_coefficient()` (`MaterialType.cpp:136-146`).
pub fn get_adhesion_coefficient(material_type: &str) -> (f64, bool) {
    match find(material_type) {
        Some(info) => (info.adhesion_coefficient, true),
        None => (DEFAULT_ADHESION_COEFFICIENT, false),
    }
}

/// `MaterialType::get_yield_strength()` (`MaterialType.cpp:148-158`).
pub fn get_yield_strength(material_type: &str) -> (f64, bool) {
    match find(material_type) {
        Some(info) => (info.yield_strength, true),
        None => (DEFAULT_YIELD_STRENGTH, false),
    }
}

/// `MaterialType::get_thermal_length()` (`MaterialType.cpp:160-170`).
pub fn get_thermal_length(material_type: &str) -> (f64, bool) {
    match find(material_type) {
        Some(info) => (info.thermal_length, true),
        None => (DEFAULT_THERMAL_LENGTH, false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slic3r_proof::proof_units::material_type as oracle;
    use slic3r_proof::{compare_unit, Oracle};

    /// A few real names, a prefix-only near-miss and a wholly unknown name, so both the hit and
    /// the miss (default-filled, `found: false`) path run against the oracle.
    const SAMPLE_MATERIALS: &[&str] = &["ABS", "PLA", "PETG-CF", "TPU", "PEI-9085-GF", "AB", "Unobtainium"];

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
            let oracle_temp = oracle::path_temperature_range(name);
            let (min_temp, max_temp, found) = get_temperature_range(name);
            let ported_temp = vec![min_temp as i64, max_temp as i64, i64::from(found)];
            compare_unit("MaterialType.cpp", Oracle::Built, ported_temp.clone(), oracle_temp.clone())
                .unwrap_or_else(|e| panic!("{name}: get_temperature_range: {e}"));
            compare_unit("MaterialType.hpp", Oracle::Built, ported_temp, oracle_temp)
                .unwrap_or_else(|e| panic!("{name}: get_temperature_range: {e}"));

            let oracle_chamber = oracle::path_chamber_temperature_range(name);
            let (chamber_min, chamber_max, chamber_found) = get_chamber_temperature_range(name);
            let ported_chamber = vec![chamber_min as i64, chamber_max as i64, i64::from(chamber_found)];
            compare_unit(
                "MaterialType.cpp",
                Oracle::Built,
                ported_chamber.clone(),
                oracle_chamber.clone(),
            )
            .unwrap_or_else(|e| panic!("{name}: get_chamber_temperature_range: {e}"));
            compare_unit("MaterialType.hpp", Oracle::Built, ported_chamber, oracle_chamber)
                .unwrap_or_else(|e| panic!("{name}: get_chamber_temperature_range: {e}"));

            let oracle_adhesion = oracle::path_adhesion_coefficient(name);
            let (adhesion, adhesion_found) = get_adhesion_coefficient(name);
            let ported_adhesion = vec![adhesion, if adhesion_found { 1.0 } else { 0.0 }];
            compare_unit(
                "MaterialType.cpp",
                Oracle::Built,
                ported_adhesion.clone(),
                oracle_adhesion.clone(),
            )
            .unwrap_or_else(|e| panic!("{name}: get_adhesion_coefficient: {e}"));
            compare_unit("MaterialType.hpp", Oracle::Built, ported_adhesion, oracle_adhesion)
                .unwrap_or_else(|e| panic!("{name}: get_adhesion_coefficient: {e}"));

            let oracle_yield = oracle::path_yield_strength(name);
            let (yield_strength, yield_found) = get_yield_strength(name);
            let ported_yield = vec![yield_strength, if yield_found { 1.0 } else { 0.0 }];
            compare_unit("MaterialType.cpp", Oracle::Built, ported_yield.clone(), oracle_yield.clone())
                .unwrap_or_else(|e| panic!("{name}: get_yield_strength: {e}"));
            compare_unit("MaterialType.hpp", Oracle::Built, ported_yield, oracle_yield)
                .unwrap_or_else(|e| panic!("{name}: get_yield_strength: {e}"));

            let oracle_thermal = oracle::path_thermal_length(name);
            let (thermal_length, thermal_found) = get_thermal_length(name);
            let ported_thermal = vec![thermal_length, if thermal_found { 1.0 } else { 0.0 }];
            compare_unit(
                "MaterialType.cpp",
                Oracle::Built,
                ported_thermal.clone(),
                oracle_thermal.clone(),
            )
            .unwrap_or_else(|e| panic!("{name}: get_thermal_length: {e}"));
            compare_unit("MaterialType.hpp", Oracle::Built, ported_thermal, oracle_thermal)
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
