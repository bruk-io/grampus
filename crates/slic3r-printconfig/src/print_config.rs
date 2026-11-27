//! `PrintConfig.cpp`/`PrintConfig.hpp` (`port/manifest.tsv`'s two rows for this module, task
//! `64-foundation-print-config`, `port/foundation.tsv`'s `foundation-13`): the print
//! configuration option set.
//!
//! **Status: incomplete, and `drafted` rather than `ported`.** This module is being built in
//! the order `intent/libslic3r-rust-port/tasks/64-foundation-print-config/plan.md` sets out
//! (Step 2 the enums, Step 3 `StaticCache`/`StaticPrintConfig`, Step 4 the 11 derived classes,
//! Step 5 the option table, Step 6 `handle_legacy`). Steps 2, 3 and 6 are in; Step 5 is
//! partial -- `init_common_params`, `init_extruder_option_keys` and
//! `init_filament_option_keys` in full, plus the first options of `init_fff_params`, with the
//! rest of `init_fff_params` and all of `init_sla_params` still placeholders -- and Step 4 is
//! not started.
//!
//! `crates/slic3r-printconfig/src/lib.rs` **does** carry `pub mod print_config;`, landed in
//! `d6e3f55`. That is `docs/WORKFLOW.md`'s rule satisfied, not a completeness claim: the rule
//! is never to land a `mod` line over a live `todo!()`/`unimplemented!()`, because a
//! `mod`-reached `todo!()` derives `not-started` and hides in the burn-down, and this module
//! holds neither. What keeps the unit's two manifest rows at `drafted` is the **absent**
//! `// PROOF: 3` marker: `derive_row_status` reads `ported` only for a `mod`-reached row whose
//! marker is present, so until the whole-key-set comparison test is green over all 881 oracle
//! keys and lands with that marker, both rows stay in `in-scope rows remaining`. Which slices
//! are in is recorded in this task's pull request body, measured rather than estimated.
//!
//! **`L(s)` is the identity function in this translation unit.** `PrintConfig.cpp:91` is
//! `#define L(s) (s)`, shadowing the gettext-backed `L` every other `.cpp` in the vendored tree
//! defines (`Slic3r::I18N::translate(s)`). Every `def->label = L("...")`, `->tooltip = L("...")`
//! and so on in this unit is therefore the literal string with no I18N call at all, so this
//! module reaches `crates/slic3r-i18n` for nothing and each `L("...")` becomes a plain
//! `.to_string()`.
//!
//! **Proof path 3**, through `crates/slic3r-ffi`'s per-unit shim
//! (`cxx/proof_units/print_config.{h,cpp}`, `src/proof_units/print_config.rs`), re-exported by
//! `slic3r-proof`. That shim is shaped like proof path 1 against the externally linked
//! `Slic3r::print_config_def` rather than `#include`ing `PrintConfig.cpp` whole: measured,
//! `PrintConfig.cpp.o` carries 340 strong external `Slic3r::` symbols, so an `#include`-whole
//! would redefine all 340 against `liblibslic3r.a`'s own archive member. The route is recorded
//! in that shim's header comment and in the pull request body; the manifest cell, and so the
//! `// PROOF: 3` marker, still read `3`.
//!
//! **One exception is genuinely path 3 in the literal sense**, and is why the cell reads `3` at
//! all: `enum_names_from_keys_map` (`PrintConfig.cpp:176`) is a file-scope `static` free
//! function with internal linkage and no symbol to call from outside its translation unit.

#![allow(non_camel_case_types)]

use slic3r_config::config::TConfigEnumNames;
use slic3r_config::config::TConfigEnumValues;

// --------------------------------------------------------------------------------------------
// Step 2: the enums `PrintConfig.hpp` declares, their key maps and their helper functions.
// --------------------------------------------------------------------------------------------

/// `enum_names_from_keys_map` (`PrintConfig.cpp:176-187`), a file-scope `static` free function
/// with internal linkage -- the one item of this unit a literal proof path 3 would have to
/// `#include` the vendored `.cpp` to reach.
///
/// Builds the name-by-value vector `ConfigOptionEnum<T>::get_enum_names()` returns: sized to
/// `max(value) + 1`, filled with the empty string, then each name written at its own value's
/// index. A value with two names in the map keeps whichever the map iterates last, and a gap in
/// the value range stays empty -- both reproduced exactly rather than tidied, because
/// `ConfigOptionEnum<T>::serialize()` indexes straight into this vector.
pub fn enum_names_from_keys_map(enum_keys_map: &TConfigEnumValues) -> TConfigEnumNames {
    let mut cnt: i32 = 0;
    for (_, v) in enum_keys_map.iter() {
        cnt = std::cmp::max(cnt, *v);
    }
    cnt += 1;
    let mut names: TConfigEnumNames = vec![String::new(); cnt as usize];
    // FIDELITY(port): the vendored loop iterates a `std::map<std::string, int>`, i.e. in
    // bytewise key order, and writes `names[kvp.second] = kvp.first`. `TConfigEnumValues` is a
    // `BTreeMap<String, i32>`, which iterates in the same order, so where two names share one
    // value the surviving name is the same one the oracle keeps. This order reaches output
    // through `ConfigOptionEnum<T>::serialize()`.
    for (k, v) in enum_keys_map.iter() {
        names[*v as usize] = k.clone();
    }
    names
}

/// `enum GCodeFlavor : unsigned char` (`PrintConfig.hpp:52-66`). Variant names kept verbatim,
/// the precedent `crates/slic3r-commondefs/src/common_defs.rs` sets for this port: idiom is the
/// following intent's job, and renaming here would make every fan-out call site guess at a
/// second spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GCodeFlavor {
    gcfMarlinLegacy,
    gcfKlipper,
    gcfRepRapFirmware,
    gcfRepetier,
    gcfMarlinFirmware,
    gcfRepRapSprinter,
    gcfTeacup,
    gcfMakerWare,
    gcfSailfish,
    gcfMach3,
    gcfMachinekit,
    gcfSmoothie,
    gcfNoExtrusion,
}
impl Default for GCodeFlavor {
    fn default() -> Self {
        GCodeFlavor::gcfMarlinLegacy
    }
}

/// `enum FilamentUsageType` (`PrintConfig.hpp:70-74`). Declared at the pin and carried here in
/// full although no option in this unit's registry reads it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FilamentUsageType {
    SupportOnly,
    ModelOnly,
    Hybrid,
}
impl Default for FilamentUsageType {
    fn default() -> Self {
        FilamentUsageType::SupportOnly
    }
}

/// `enum class FuzzySkinType` (`PrintConfig.hpp:77-84`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FuzzySkinType {
    None,
    External,
    Hole,
    All,
    AllWalls,
    Disabled_fuzzy,
}
impl Default for FuzzySkinType {
    fn default() -> Self {
        FuzzySkinType::None
    }
}

/// `enum class FuzzySkinMode` (`PrintConfig.hpp:86-90`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FuzzySkinMode {
    Displacement,
    Extrusion,
    Combined,
}
impl Default for FuzzySkinMode {
    fn default() -> Self {
        FuzzySkinMode::Displacement
    }
}

/// `enum class TopSurfaceExpansionDirection` (`PrintConfig.hpp:93-97`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TopSurfaceExpansionDirection {
    InwardAndOutward,
    Inward,
    Outward,
}
impl Default for TopSurfaceExpansionDirection {
    fn default() -> Self {
        TopSurfaceExpansionDirection::InwardAndOutward
    }
}

/// `enum class CenterOfSurfacePattern` (`PrintConfig.hpp:99-103`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CenterOfSurfacePattern {
    Each_Surface,
    Each_Model,
    Each_Assembly,
}
impl Default for CenterOfSurfacePattern {
    fn default() -> Self {
        CenterOfSurfacePattern::Each_Surface
    }
}

/// `enum class NoiseType` (`PrintConfig.hpp:105-112`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NoiseType {
    Classic,
    Perlin,
    Billow,
    RidgedMulti,
    Voronoi,
    Ripple,
}
impl Default for NoiseType {
    fn default() -> Self {
        NoiseType::Classic
    }
}

/// `enum class WipeTowerType` (`PrintConfig.hpp:114-117`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WipeTowerType {
    Type1,
    Type2,
}
impl Default for WipeTowerType {
    fn default() -> Self {
        WipeTowerType::Type1
    }
}

/// `enum PrintHostType` (`PrintConfig.hpp:119-121`). The declaration order is **not** the key
/// map's order (`PrintConfig.cpp:196-214` lists `crealityprint` fifth while the declaration puts
/// `htCrealityPrint` eleventh), so both are transcribed from their own site.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PrintHostType {
    htPrusaLink,
    htPrusaConnect,
    htOctoPrint,
    htDuet,
    htUltiMaker,
    htFlashAir,
    htAstroBox,
    htRepetier,
    htMKS,
    htESP3D,
    htCrealityPrint,
    htObico,
    htFlashforge,
    htSimplyPrint,
    htElegooLink,
    ht3DPrinterOS,
    htMoonraker,
}
impl Default for PrintHostType {
    fn default() -> Self {
        PrintHostType::htPrusaLink
    }
}

/// `enum AuthorizationType` (`PrintConfig.hpp:123-125`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AuthorizationType {
    atKeyPassword,
    atUserPassword,
}
impl Default for AuthorizationType {
    fn default() -> Self {
        AuthorizationType::atKeyPassword
    }
}

/// `enum InfillPattern : int` (`PrintConfig.hpp:127-138`). `ipCount` is the vendored sentinel
/// and is carried as declared; `ipSupportBase`/`ipConcentricInternal` are declared but have no
/// entry in `s_keys_map_InfillPattern` (`PrintConfig.cpp:305-340`), which is why
/// `enum_names_from_keys_map` leaves their indices empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InfillPattern {
    ipMonotonic,
    ipMonotonicLine,
    ipRectilinear,
    ipAlignedRectilinear,
    ipZigZag,
    ipCrossZag,
    ipLockedZag,
    ipLine,
    ipGrid,
    ipTriangles,
    ipStars,
    ipCubic,
    ipAdaptiveCubic,
    ipQuarterCubic,
    ipSupportCubic,
    ipLightning,
    ipHoneycomb,
    ip3DHoneycomb,
    ipLateralHoneycomb,
    ipLateralLattice,
    ipCrossHatch,
    ipTpmsD,
    ipTpmsFK,
    ipGyroid,
    ipConcentric,
    ipSpiralInset,
    ipHilbertCurve,
    ipArchimedeanChords,
    ipOctagramSpiral,
    ipSupportBase,
    ipConcentricInternal,
    ipCount,
}
impl Default for InfillPattern {
    fn default() -> Self {
        InfillPattern::ipMonotonic
    }
}

/// `is_separable_infill_pattern` (`PrintConfig.hpp:141-172`), an `inline` function with real
/// logic rather than a plain value list.
pub fn is_separable_infill_pattern(pattern: InfillPattern) -> bool {
    matches!(
        pattern,
        InfillPattern::ipMonotonic
            | InfillPattern::ipMonotonicLine
            | InfillPattern::ipRectilinear
            | InfillPattern::ipAlignedRectilinear
            | InfillPattern::ipZigZag
            | InfillPattern::ipCrossZag
            | InfillPattern::ipLockedZag
            | InfillPattern::ipLine
            | InfillPattern::ipGrid
            | InfillPattern::ipTriangles
            | InfillPattern::ipStars
            | InfillPattern::ipCubic
            | InfillPattern::ipQuarterCubic
            | InfillPattern::ipHoneycomb
            | InfillPattern::ip3DHoneycomb
            | InfillPattern::ipLateralHoneycomb
            | InfillPattern::ipLateralLattice
            | InfillPattern::ipCrossHatch
            | InfillPattern::ipTpmsD
            | InfillPattern::ipTpmsFK
            | InfillPattern::ipGyroid
            | InfillPattern::ipHilbertCurve
            | InfillPattern::ipArchimedeanChords
            | InfillPattern::ipOctagramSpiral
    )
}

/// `is_octree_infill_pattern` (`PrintConfig.hpp:175`).
pub fn is_octree_infill_pattern(pattern: InfillPattern) -> bool {
    pattern == InfillPattern::ipAdaptiveCubic || pattern == InfillPattern::ipSupportCubic
}

/// `is_adaptive_infill_pattern` (`PrintConfig.hpp:177`).
pub fn is_adaptive_infill_pattern(pattern: InfillPattern) -> bool {
    pattern == InfillPattern::ipGyroid
        || pattern == InfillPattern::ipTpmsD
        || pattern == InfillPattern::ipTpmsFK
}

/// `is_smoothable_infill_pattern` (`PrintConfig.hpp:182-201`). The vendored signature's
/// `int multiline = 1` default argument has no Rust equivalent; the parameter is required here
/// and every call site passes it, which is the simplest translation that keeps the meaning.
pub fn is_smoothable_infill_pattern(pattern: InfillPattern, multiline: i32) -> bool {
    match pattern {
        InfillPattern::ipHilbertCurve
        | InfillPattern::ipOctagramSpiral
        | InfillPattern::ipLightning
        | InfillPattern::ipHoneycomb
        | InfillPattern::ip3DHoneycomb
        | InfillPattern::ipConcentric
        | InfillPattern::ipCrossHatch => true,
        InfillPattern::ipGrid
        | InfillPattern::ipTriangles
        | InfillPattern::ipStars
        | InfillPattern::ipCubic => multiline > 1,
        _ => false,
    }
}

/// `enum class IroningType` (`PrintConfig.hpp:203-209`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IroningType {
    NoIroning,
    TopSurfaces,
    TopmostOnly,
    AllSolid,
    Count,
}
impl Default for IroningType {
    fn default() -> Self {
        IroningType::NoIroning
    }
}

// PORT SRC: src/libslic3r/PrintConfig.hpp:213
/// `constexpr double IRONING_SPACING_MIN` (`PrintConfig.hpp:213`).
pub const IRONING_SPACING_MIN: f64 = 0.05;

/// `enum class WallInfillOrder` (`PrintConfig.hpp:216-223`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WallInfillOrder {
    InnerOuterInfill,
    OuterInnerInfill,
    InfillInnerOuter,
    InfillOuterInner,
    InnerOuterInnerInfill,
    Count,
}
impl Default for WallInfillOrder {
    fn default() -> Self {
        WallInfillOrder::InnerOuterInfill
    }
}

/// `enum class BedTempFormula` (`PrintConfig.hpp:225-229`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BedTempFormula {
    btfFirstFilament,
    btfHighestTemp,
    count,
}
impl Default for BedTempFormula {
    fn default() -> Self {
        BedTempFormula::btfFirstFilament
    }
}

/// `enum class PowerLossRecoveryMode` (`PrintConfig.hpp:232-236`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PowerLossRecoveryMode {
    PrinterConfiguration,
    Enable,
    Disable,
}
impl Default for PowerLossRecoveryMode {
    fn default() -> Self {
        PowerLossRecoveryMode::PrinterConfiguration
    }
}

/// `enum class ArcFittingMode` (`PrintConfig.hpp:239-243`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ArcFittingMode {
    Disabled,
    Enabled,
    SparseInfill,
}
impl Default for ArcFittingMode {
    fn default() -> Self {
        ArcFittingMode::Disabled
    }
}

/// `enum class WallSequence` (`PrintConfig.hpp:246-251`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WallSequence {
    InnerOuter,
    OuterInner,
    InnerOuterInner,
    Count,
}
impl Default for WallSequence {
    fn default() -> Self {
        WallSequence::InnerOuter
    }
}

/// `enum class WallDirection` (`PrintConfig.hpp:254-259`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WallDirection {
    CounterClockwise,
    Clockwise,
    Count,
}
impl Default for WallDirection {
    fn default() -> Self {
        WallDirection::CounterClockwise
    }
}

/// `enum class ImexToolLayout` (`PrintConfig.hpp:263-269`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImexToolLayout {
    FrontLeft,
    FrontRight,
    RearLeft,
    RearRight,
    Count,
}
impl Default for ImexToolLayout {
    fn default() -> Self {
        ImexToolLayout::FrontLeft
    }
}

/// `enum class ImexVizTheme` (`PrintConfig.hpp:273-279`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImexVizTheme {
    Standard,
    Deuteranopia,
    Tritanopia,
    HighContrast,
    Count,
}
impl Default for ImexVizTheme {
    fn default() -> Self {
        ImexVizTheme::Standard
    }
}

/// `enum class SurfaceFillOrder` (`PrintConfig.hpp:283-288`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SurfaceFillOrder {
    Default,
    Outward,
    Inward,
    Count,
}
impl Default for SurfaceFillOrder {
    fn default() -> Self {
        SurfaceFillOrder::Default
    }
}

/// `enum class AdaptiveInfillMode` (`PrintConfig.hpp:291-301`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AdaptiveInfillMode {
    Disabled,
    DistanceWarp,
    SmoothBlend,
    SteppedShells,
    Lobes,
    NormalZ,
    NormalY,
    NormalX,
    Count,
}
impl Default for AdaptiveInfillMode {
    fn default() -> Self {
        AdaptiveInfillMode::Disabled
    }
}

/// `enum class AdaptiveInfillGradient` (`PrintConfig.hpp:304-308`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AdaptiveInfillGradient {
    Linear,
    Quadratic,
    Exponential,
}
impl Default for AdaptiveInfillGradient {
    fn default() -> Self {
        AdaptiveInfillGradient::Linear
    }
}

/// `enum class PrintSequence` (`PrintConfig.hpp:311-316`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PrintSequence {
    ByLayer,
    ByObject,
    ByDefault,
    Count,
}
impl Default for PrintSequence {
    fn default() -> Self {
        PrintSequence::ByLayer
    }
}

/// `enum class PrintOrder` (`PrintConfig.hpp:318-325`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PrintOrder {
    Default,
    AsObjectList,
    BestOfStrategies,
    Snake,
    Count,
}
impl Default for PrintOrder {
    fn default() -> Self {
        PrintOrder::Default
    }
}

/// `enum class SlicingMode` (`PrintConfig.hpp:327-335`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SlicingMode {
    Regular,
    EvenOdd,
    CloseHoles,
}
impl Default for SlicingMode {
    fn default() -> Self {
        SlicingMode::Regular
    }
}

/// `enum class BeltRotationAxis` (`PrintConfig.hpp:341-347`). Every value is written explicitly
/// at the pin and every one of them is in the exempt literal set, so no marker is needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BeltRotationAxis {
    None = 0,
    X = 1,
    Y = 2,
    // PORT SRC: src/libslic3r/PrintConfig.hpp:346
    Z = 3,
}
impl Default for BeltRotationAxis {
    fn default() -> Self {
        BeltRotationAxis::None
    }
}

/// `enum class RemapAxis` (`PrintConfig.hpp:349-354`). The vendored declaration writes all nine
/// values explicitly, three to a line, so the markers below cover them line for line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RemapAxis {
    PosX = 0,
    PosY = 1,
    PosZ = 2,
    // PORT SRC: src/libslic3r/PrintConfig.hpp:352
    NegX = 3,
    // PORT SRC: src/libslic3r/PrintConfig.hpp:352
    NegY = 4,
    // PORT SRC: src/libslic3r/PrintConfig.hpp:352
    NegZ = 5,
    // PORT SRC: src/libslic3r/PrintConfig.hpp:353
    RevX = 6,
    // PORT SRC: src/libslic3r/PrintConfig.hpp:353
    RevY = 7,
    // PORT SRC: src/libslic3r/PrintConfig.hpp:353
    RevZ = 8,
}
impl Default for RemapAxis {
    fn default() -> Self {
        RemapAxis::PosX
    }
}

/// `enum SupportMaterialPattern` (`PrintConfig.hpp:356-361`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SupportMaterialPattern {
    smpDefault,
    smpRectilinear,
    smpRectilinearGrid,
    smpHoneycomb,
    smpLightning,
    smpNone,
}
impl Default for SupportMaterialPattern {
    fn default() -> Self {
        SupportMaterialPattern::smpDefault
    }
}

/// `enum SupportMaterialStyle` (`PrintConfig.hpp:363-365`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SupportMaterialStyle {
    smsDefault,
    smsGrid,
    smsSnug,
    smsTreeOrganic,
    smsTreeSlim,
    smsTreeStrong,
    smsTreeHybrid,
}
impl Default for SupportMaterialStyle {
    fn default() -> Self {
        SupportMaterialStyle::smsDefault
    }
}

/// `enum LongRectrationLevel` (`PrintConfig.hpp:367-372`), vendored spelling of the name kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LongRectrationLevel {
    Disabled = 0,
    EnableMachine,
    EnableFilament,
}
impl Default for LongRectrationLevel {
    fn default() -> Self {
        LongRectrationLevel::Disabled
    }
}

/// `enum SupportMaterialInterfacePattern` (`PrintConfig.hpp:374-376`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SupportMaterialInterfacePattern {
    smipAuto,
    smipRectilinear,
    smipConcentric,
    smipSpiralInset,
    smipRectilinearInterlaced,
    smipGrid,
}
impl Default for SupportMaterialInterfacePattern {
    fn default() -> Self {
        SupportMaterialInterfacePattern::smipAuto
    }
}

/// `enum SupportType` (`PrintConfig.hpp:379-381`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SupportType {
    stNormalAuto,
    stTreeAuto,
    stNormal,
    stTree,
}
impl Default for SupportType {
    fn default() -> Self {
        SupportType::stNormalAuto
    }
}

/// `is_tree` (`PrintConfig.hpp:382-385`). The vendored body builds a two-element
/// `std::set<SupportType>` and counts; the set is a constant of the function, so the membership
/// test is the whole of what it computes.
pub fn is_tree(stype: SupportType) -> bool {
    matches!(stype, SupportType::stTreeAuto | SupportType::stTree)
}

/// `is_tree_slim` (`PrintConfig.hpp:386-389`).
pub fn is_tree_slim(stype: SupportType, style: SupportMaterialStyle) -> bool {
    is_tree(stype) && style == SupportMaterialStyle::smsTreeSlim
}

/// `is_auto` (`PrintConfig.hpp:390-393`).
pub fn is_auto(stype: SupportType) -> bool {
    matches!(stype, SupportType::stNormalAuto | SupportType::stTreeAuto)
}

/// `enum SeamPosition` (`PrintConfig.hpp:395-397`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SeamPosition {
    spNearest,
    spAligned,
    spAlignedBack,
    spRear,
    spRandom,
}
impl Default for SeamPosition {
    fn default() -> Self {
        SeamPosition::spNearest
    }
}

/// `enum class SeamScarfType` (`PrintConfig.hpp:400-404`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SeamScarfType {
    None,
    External,
    All,
}
impl Default for SeamScarfType {
    fn default() -> Self {
        SeamScarfType::None
    }
}

/// `enum EnsureVerticalShellThickness` (`PrintConfig.hpp:407-412`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EnsureVerticalShellThickness {
    evstNone,
    evstCriticalOnly,
    evstModerate,
    evstAll,
}
impl Default for EnsureVerticalShellThickness {
    fn default() -> Self {
        EnsureVerticalShellThickness::evstNone
    }
}

/// `enum InternalBridgeFilter` (`PrintConfig.hpp:415-417`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InternalBridgeFilter {
    ibfDisabled,
    ibfLimited,
    ibfNofilter,
}
impl Default for InternalBridgeFilter {
    fn default() -> Self {
        InternalBridgeFilter::ibfDisabled
    }
}

/// `enum EnableExtraBridgeLayer` (`PrintConfig.hpp:420-422`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EnableExtraBridgeLayer {
    eblDisabled,
    eblExternalBridgeOnly,
    eblInternalBridgeOnly,
    eblApplyToAll,
}
impl Default for EnableExtraBridgeLayer {
    fn default() -> Self {
        EnableExtraBridgeLayer::eblDisabled
    }
}

/// `enum GapFillTarget` (`PrintConfig.hpp:425-427`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GapFillTarget {
    gftEverywhere,
    gftTopBottom,
    gftNowhere,
}
impl Default for GapFillTarget {
    fn default() -> Self {
        GapFillTarget::gftEverywhere
    }
}

/// `enum LiftType` (`PrintConfig.hpp:430-434`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LiftType {
    NormalLift,
    SpiralLift,
    SlopeLift,
}
impl Default for LiftType {
    fn default() -> Self {
        LiftType::NormalLift
    }
}

/// `enum SLAMaterial` (`PrintConfig.hpp:436-442`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SLAMaterial {
    slamTough,
    slamFlex,
    slamCasting,
    slamDental,
    slamHeatResistant,
}
impl Default for SLAMaterial {
    fn default() -> Self {
        SLAMaterial::slamTough
    }
}

/// `enum SLADisplayOrientation` (`PrintConfig.hpp:444-447`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SLADisplayOrientation {
    sladoLandscape,
    sladoPortrait,
}
impl Default for SLADisplayOrientation {
    fn default() -> Self {
        SLADisplayOrientation::sladoLandscape
    }
}

/// `enum SLAPillarConnectionMode` (`PrintConfig.hpp:449-453`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SLAPillarConnectionMode {
    slapcmZigZag,
    slapcmCross,
    slapcmDynamic,
}
impl Default for SLAPillarConnectionMode {
    fn default() -> Self {
        SLAPillarConnectionMode::slapcmZigZag
    }
}

/// `enum BrimType` (`PrintConfig.hpp:455-467`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BrimType {
    btAutoBrim,
    btEar,
    btPainted,
    btOuterOnly,
    btInnerOnly,
    btOuterAndInner,
    btNoBrim,
    btLeadingEdgeOnly,
}
impl Default for BrimType {
    fn default() -> Self {
        BrimType::btAutoBrim
    }
}

/// `enum TimelapseType : int` (`PrintConfig.hpp:469-472`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TimelapseType {
    tlTraditional = 0,
    tlSmooth,
}
impl Default for TimelapseType {
    fn default() -> Self {
        TimelapseType::tlTraditional
    }
}

/// `enum SkirtType` (`PrintConfig.hpp:474-476`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SkirtType {
    stCombined,
    stPerObject,
}
impl Default for SkirtType {
    fn default() -> Self {
        SkirtType::stCombined
    }
}

/// `enum DraftShield` (`PrintConfig.hpp:478-480`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DraftShield {
    dsDisabled,
    dsEnabled,
}
impl Default for DraftShield {
    fn default() -> Self {
        DraftShield::dsDisabled
    }
}

/// `enum class PerimeterGeneratorType` (`PrintConfig.hpp:482-489`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PerimeterGeneratorType {
    Classic,
    Arachne,
}
impl Default for PerimeterGeneratorType {
    fn default() -> Self {
        PerimeterGeneratorType::Classic
    }
}

/// `enum class ToolChangeOrderingType` (`PrintConfig.hpp:491-495`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ToolChangeOrderingType {
    Default,
    Cyclic,
}
impl Default for ToolChangeOrderingType {
    fn default() -> Self {
        ToolChangeOrderingType::Default
    }
}

/// `enum OverhangFanThreshold` (`PrintConfig.hpp:498-505`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OverhangFanThreshold {
    Overhang_threshold_none = 0,
    Overhang_threshold_1_4,
    Overhang_threshold_2_4,
    Overhang_threshold_3_4,
    Overhang_threshold_4_4,
    Overhang_threshold_bridge,
}
impl Default for OverhangFanThreshold {
    fn default() -> Self {
        OverhangFanThreshold::Overhang_threshold_none
    }
}

/// `enum BedType` (`PrintConfig.hpp:508-517`). `btCount` is the vendored sentinel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BedType {
    btDefault = 0,
    btPC,
    btEP,
    btPEI,
    btPTE,
    btPCT,
    btSuperTack,
    btCount,
}
impl Default for BedType {
    fn default() -> Self {
        BedType::btDefault
    }
}

/// `enum class ExtruderOnlyAreaType : unsigned char` (`PrintConfig.hpp:519-524`), vendored
/// spelling of `Engilish` kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExtruderOnlyAreaType {
    btNoArea = 0,
    Engilish,
    Chinese,
    btAreaCount,
}
impl Default for ExtruderOnlyAreaType {
    fn default() -> Self {
        ExtruderOnlyAreaType::btNoArea
    }
}

/// `enum LayerSeq` (`PrintConfig.hpp:527-530`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LayerSeq {
    flsAuto,
    flsCustomize,
}
impl Default for LayerSeq {
    fn default() -> Self {
        LayerSeq::flsAuto
    }
}

/// `enum FanDirection` (`PrintConfig.hpp:532-537`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FanDirection {
    fdUndefine = 0,
    fdLeft,
    fdRight,
    fdBoth,
}
impl Default for FanDirection {
    fn default() -> Self {
        FanDirection::fdUndefine
    }
}

/// `enum PrinterStructure` (`PrintConfig.hpp:558-564`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PrinterStructure {
    psUndefine = 0,
    psCoreXY,
    psI3,
    psHbot,
    psDelta,
}
impl Default for PrinterStructure {
    fn default() -> Self {
        PrinterStructure::psUndefine
    }
}

/// `enum class InputShaperType : unsigned char` (`PrintConfig.hpp:566-580`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InputShaperType {
    Default = 0,
    MZV,
    ZV,
    ZVD,
    ZVDD,
    ZVDDD,
    EI,
    EI2,
    TwoHumpEI,
    EI3,
    ThreeHumpEI,
    DAA,
    Disable,
}
impl Default for InputShaperType {
    fn default() -> Self {
        InputShaperType::Default
    }
}

/// `enum ZHopType` (`PrintConfig.hpp:583-589`). `zhtCount` is the vendored sentinel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ZHopType {
    zhtAuto = 0,
    zhtNormal,
    zhtSlope,
    zhtSpiral,
    zhtCount,
}
impl Default for ZHopType {
    fn default() -> Self {
        ZHopType::zhtAuto
    }
}

/// `enum RetractLiftEnforceType` (`PrintConfig.hpp:591-596`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RetractLiftEnforceType {
    rletAllSurfaces = 0,
    rletTopOnly,
    rletBottomOnly,
    rletTopAndBottom,
}
impl Default for RetractLiftEnforceType {
    fn default() -> Self {
        RetractLiftEnforceType::rletAllSurfaces
    }
}

/// `enum class GCodeThumbnailsFormat` (`PrintConfig.hpp:598-600`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GCodeThumbnailsFormat {
    PNG,
    JPG,
    QOI,
    BTT_TFT,
    ColPic,
}
impl Default for GCodeThumbnailsFormat {
    fn default() -> Self {
        GCodeThumbnailsFormat::PNG
    }
}

/// `enum CounterboreHoleBridgingOption` (`PrintConfig.hpp:602-604`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CounterboreHoleBridgingOption {
    chbNone,
    chbBridges,
    chbFilled,
}
impl Default for CounterboreHoleBridgingOption {
    fn default() -> Self {
        CounterboreHoleBridgingOption::chbNone
    }
}

/// `enum WipeTowerWallType` (`PrintConfig.hpp:606-610`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WipeTowerWallType {
    wtwRectangle = 0,
    wtwCone,
    wtwRib,
}
impl Default for WipeTowerWallType {
    fn default() -> Self {
        WipeTowerWallType::wtwRectangle
    }
}

/// `enum ExtruderType` (`PrintConfig.hpp:613-617`). The vendored declaration's
/// `etMaxExtruderType = etBowden` is an alias of an existing value, which a Rust `enum` cannot
/// declare twice; it is carried as the associated constant below instead -- a named divergence,
/// not a dropped declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExtruderType {
    etDirectDrive = 0,
    etBowden,
}
impl Default for ExtruderType {
    fn default() -> Self {
        ExtruderType::etDirectDrive
    }
}
impl ExtruderType {
    /// `etMaxExtruderType = etBowden` (`PrintConfig.hpp:616`).
    pub const ET_MAX_EXTRUDER_TYPE: ExtruderType = ExtruderType::etBowden;
}

/// `enum NozzleVolumeType` (`PrintConfig.hpp:619-631`). The reserved gap at 4 is real: the
/// vendored comment at `:626` records that E3D High Flow is 5 in BambuStudio's own numbering,
/// and `get_valid_nozzle_volume_type` skips the gap. `nvtMaxNozzleVolumeType` is an alias of
/// `nvtExtraHighFlow` and is carried as an associated constant for the same reason
/// `ExtruderType::ET_MAX_EXTRUDER_TYPE` is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NozzleVolumeType {
    nvtStandard = 0,
    nvtHighFlow,
    nvtHybrid,
    nvtTPUHighFlow,
    // PORT SRC: src/libslic3r/PrintConfig.hpp:627
    nvtE3DHighFlow = 5,
    // PORT SRC: src/libslic3r/PrintConfig.hpp:628
    nvtExtraHighFlow = 6,
}
impl Default for NozzleVolumeType {
    fn default() -> Self {
        NozzleVolumeType::nvtStandard
    }
}
impl NozzleVolumeType {
    /// `nvtMaxNozzleVolumeType = nvtExtraHighFlow` (`PrintConfig.hpp:630`).
    pub const NVT_MAX_NOZZLE_VOLUME_TYPE: NozzleVolumeType = NozzleVolumeType::nvtExtraHighFlow;
}

/// `enum FilamentMapMode` (`PrintConfig.hpp:633-639`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FilamentMapMode {
    fmmAutoForFlush,
    fmmAutoForMatch,
    fmmManual,
    fmmNozzleManual,
    fmmDefault,
}
impl Default for FilamentMapMode {
    fn default() -> Self {
        FilamentMapMode::fmmAutoForFlush
    }
}

/// `is_auto_filament_map_mode` (`PrintConfig.hpp:650-652`): `mode < fmmManual`, which the
/// vendored comment at `:649` records as load-bearing on the declaration order.
pub fn is_auto_filament_map_mode(mode: FilamentMapMode) -> bool {
    (mode as i32) < (FilamentMapMode::fmmManual as i32)
}

/// `enum class BedExcludeVolumeMode` (`PrintConfig.hpp:643-647`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BedExcludeVolumeMode {
    Shared = 0,
    ToolheadOffset,
    PerExtruder,
}
impl Default for BedExcludeVolumeMode {
    fn default() -> Self {
        BedExcludeVolumeMode::Shared
    }
}

/// `enum PrimeVolumeMode` (`PrintConfig.hpp:657-661`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PrimeVolumeMode {
    pvmDefault = 0,
    pvmSaving,
    pvmFast,
}
impl Default for PrimeVolumeMode {
    fn default() -> Self {
        PrimeVolumeMode::pvmDefault
    }
}

/// `enum SLAMaterialSpeed` (`PrintConfig.hpp:2343`), declared inside this unit's own header --
/// outside the `:52-839` block the other enums sit in, but this unit's to port all the same.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SLAMaterialSpeed {
    slamsSlow,
    slamsFast,
}
impl Default for SLAMaterialSpeed {
    fn default() -> Self {
        SLAMaterialSpeed::slamsSlow
    }
}

// --------------------------------------------------------------------------------------------
// Step 2, continued: the 65 `s_keys_map_*` tables and the `CONFIG_OPTION_ENUM_DEFINE_STATIC_MAPS`
// expansion (`PrintConfig.cpp:189-773`).
// --------------------------------------------------------------------------------------------

/// `CONFIG_OPTION_ENUM_DEFINE_STATIC_MAPS(NAME)` (`PrintConfig.cpp:189`), which defines three
/// things per enum type: the key map itself, the name-by-value vector
/// `enum_names_from_keys_map` derives from it, and the two `ConfigOptionEnum<NAME>`
/// specialisations that return them.
///
/// The vendored macro gets file-scope `static` storage initialised before `main`; Rust has no
/// before-`main` phase, so each table is a `OnceLock` initialised on first use. The address is
/// stable once initialised and the content is identical, which is what `ConfigOptionDef`'s
/// `&'static TConfigEnumValues` field and the oracle's `const t_config_enum_values *` both need.
/// Ported once as a macro, exactly as the vendored source writes it once as a macro -- writing
/// 65 copies by hand would invent a divergence the source does not have.
macro_rules! config_option_enum_define_static_maps {
    (
        $keys_fn:ident, $names_fn:ident, $doc:expr,
        [ $( $key:expr => $value:expr ),* $(,)? ]
    ) => {
        #[doc = $doc]
        pub fn $keys_fn() -> &'static TConfigEnumValues {
            static MAP: std::sync::OnceLock<TConfigEnumValues> = std::sync::OnceLock::new();
            MAP.get_or_init(|| {
                let mut m = TConfigEnumValues::new();
                $( m.insert($key.to_string(), $value as i32); )*
                m
            })
        }

        #[doc = $doc]
        #[doc = ""]
        #[doc = "The `enum_names_from_keys_map` half of the same macro expansion."]
        pub fn $names_fn() -> &'static TConfigEnumNames {
            static NAMES: std::sync::OnceLock<TConfigEnumNames> = std::sync::OnceLock::new();
            NAMES.get_or_init(|| enum_names_from_keys_map($keys_fn()))
        }
    };
}

/// The `ConfigOptionEnum<NAME>` specialisation half of
/// `CONFIG_OPTION_ENUM_DEFINE_STATIC_MAPS`, as `slic3r_config::config::ConfigOptionEnumValue`.
///
/// **`// PORT NOTE:` on `from_i32`, the one design question `PrintConfig.hpp` leaves open.**
/// The vendored `ConfigOptionEnum<T>::set_int`/`get_int` round-trip an `int` through a bare
/// `static_cast<T>(int)`, which C++ performs even for a value matching no enumerator. Rust's
/// nearest equivalent, `std::mem::transmute` from the repr type, is undefined behaviour the
/// instant the value is not a declared discriminant, so it is not a faithful-and-safe
/// translation of the same operation. Decided once, and applied identically to every enum
/// rather than re-decided per type: test the value against each declared variant in declaration
/// order and fall back to the type's own default variant (its zero value) when none matches.
/// An in-range value therefore round-trips exactly as the oracle's cast does; an out-of-range
/// value is pinned to the default instead of producing an invalid enum.
macro_rules! impl_config_option_enum_value {
    ( $ty:ty, $keys_fn:ident, $names_fn:ident, [ $( $variant:ident ),+ $(,)? ] ) => {
        impl slic3r_config::config::ConfigOptionEnumValue for $ty {
            fn to_i32(self) -> i32 {
                self as i32
            }
            fn from_i32(v: i32) -> Self {
                $(
                    if v == (<$ty>::$variant as i32) {
                        return <$ty>::$variant;
                    }
                )+
                <$ty as Default>::default()
            }
            fn get_enum_names() -> &'static TConfigEnumNames {
                $names_fn()
            }
            fn get_enum_values() -> &'static TConfigEnumValues {
                $keys_fn()
            }
        }
    };
}

config_option_enum_define_static_maps!(
    keys_map_printer_technology,
    keys_names_printer_technology,
    "`s_keys_map_PrinterTechnology` (`PrintConfig.cpp:191-194`).",
    [
        "FFF" => slic3r_config::config::PrinterTechnology::Fff,
        "SLA" => slic3r_config::config::PrinterTechnology::Sla,
    ]
);

config_option_enum_define_static_maps!(
    keys_map_print_host_type,
    keys_names_print_host_type,
    "`s_keys_map_PrintHostType` (`PrintConfig.cpp:197-215`).",
    [
        "prusalink" => PrintHostType::htPrusaLink,
        "prusaconnect" => PrintHostType::htPrusaConnect,
        "octoprint" => PrintHostType::htOctoPrint,
        "crealityprint" => PrintHostType::htCrealityPrint,
        "duet" => PrintHostType::htDuet,
        "ultimaker" => PrintHostType::htUltiMaker,
        "flashair" => PrintHostType::htFlashAir,
        "astrobox" => PrintHostType::htAstroBox,
        "repetier" => PrintHostType::htRepetier,
        "mks" => PrintHostType::htMKS,
        "esp3d" => PrintHostType::htESP3D,
        "obico" => PrintHostType::htObico,
        "flashforge" => PrintHostType::htFlashforge,
        "simplyprint" => PrintHostType::htSimplyPrint,
        "elegoolink" => PrintHostType::htElegooLink,
        "3dprinteros" => PrintHostType::ht3DPrinterOS,
        "moonraker" => PrintHostType::htMoonraker,
    ]
);
impl_config_option_enum_value!(
    PrintHostType,
    keys_map_print_host_type,
    keys_names_print_host_type,
    [
        htPrusaLink,
        htPrusaConnect,
        htOctoPrint,
        htDuet,
        htUltiMaker,
        htFlashAir,
        htAstroBox,
        htRepetier,
        htMKS,
        htESP3D,
        htCrealityPrint,
        htObico,
        htFlashforge,
        htSimplyPrint,
        htElegooLink,
        ht3DPrinterOS,
        htMoonraker,
    ]
);

config_option_enum_define_static_maps!(
    keys_map_authorization_type,
    keys_names_authorization_type,
    "`s_keys_map_AuthorizationType` (`PrintConfig.cpp:218-221`).",
    [
        "key" => AuthorizationType::atKeyPassword,
        "user" => AuthorizationType::atUserPassword,
    ]
);
impl_config_option_enum_value!(
    AuthorizationType,
    keys_map_authorization_type,
    keys_names_authorization_type,
    [atKeyPassword, atUserPassword]
);

config_option_enum_define_static_maps!(
    keys_map_gcode_flavor,
    keys_names_gcode_flavor,
    "`s_keys_map_GCodeFlavor` (`PrintConfig.cpp:224-238`).",
    [
        "marlin" => GCodeFlavor::gcfMarlinLegacy,
        "klipper" => GCodeFlavor::gcfKlipper,
        "reprapfirmware" => GCodeFlavor::gcfRepRapFirmware,
        "repetier" => GCodeFlavor::gcfRepetier,
        "marlin2" => GCodeFlavor::gcfMarlinFirmware,
        "reprap" => GCodeFlavor::gcfRepRapSprinter,
        "teacup" => GCodeFlavor::gcfTeacup,
        "makerware" => GCodeFlavor::gcfMakerWare,
        "sailfish" => GCodeFlavor::gcfSailfish,
        "smoothie" => GCodeFlavor::gcfSmoothie,
        "mach3" => GCodeFlavor::gcfMach3,
        "machinekit" => GCodeFlavor::gcfMachinekit,
        "no-extrusion" => GCodeFlavor::gcfNoExtrusion,
    ]
);
impl_config_option_enum_value!(
    GCodeFlavor,
    keys_map_gcode_flavor,
    keys_names_gcode_flavor,
    [
        gcfMarlinLegacy,
        gcfKlipper,
        gcfRepRapFirmware,
        gcfRepetier,
        gcfMarlinFirmware,
        gcfRepRapSprinter,
        gcfTeacup,
        gcfMakerWare,
        gcfSailfish,
        gcfMach3,
        gcfMachinekit,
        gcfSmoothie,
        gcfNoExtrusion,
    ]
);

config_option_enum_define_static_maps!(
    keys_map_bed_temp_formula,
    keys_names_bed_temp_formula,
    "`s_keys_map_BedTempFormula` (`PrintConfig.cpp:241-244`).",
    [
        "by_first_filament" => BedTempFormula::btfFirstFilament,
        "by_highest_temp" => BedTempFormula::btfHighestTemp,
    ]
);
impl_config_option_enum_value!(
    BedTempFormula,
    keys_map_bed_temp_formula,
    keys_names_bed_temp_formula,
    [btfFirstFilament, btfHighestTemp, count]
);

config_option_enum_define_static_maps!(
    keys_map_power_loss_recovery_mode,
    keys_names_power_loss_recovery_mode,
    "`s_keys_map_PowerLossRecoveryMode` (`PrintConfig.cpp:248-252`).",
    [
        "printer_configuration" => PowerLossRecoveryMode::PrinterConfiguration,
        "enable" => PowerLossRecoveryMode::Enable,
        "disable" => PowerLossRecoveryMode::Disable,
    ]
);
impl_config_option_enum_value!(
    PowerLossRecoveryMode,
    keys_map_power_loss_recovery_mode,
    keys_names_power_loss_recovery_mode,
    [PrinterConfiguration, Enable, Disable]
);

config_option_enum_define_static_maps!(
    keys_map_arc_fitting_mode,
    keys_names_arc_fitting_mode,
    "`s_keys_map_ArcFittingMode` (`PrintConfig.cpp:256-260`).",
    [
        "disabled" => ArcFittingMode::Disabled,
        "enabled" => ArcFittingMode::Enabled,
        "sparse_infill" => ArcFittingMode::SparseInfill,
    ]
);
impl_config_option_enum_value!(
    ArcFittingMode,
    keys_map_arc_fitting_mode,
    keys_names_arc_fitting_mode,
    [Disabled, Enabled, SparseInfill]
);

config_option_enum_define_static_maps!(
    keys_map_center_of_surface_pattern,
    keys_names_center_of_surface_pattern,
    "`s_keys_map_CenterOfSurfacePattern` (`PrintConfig.cpp:263-266`).",
    [
        "each_surface" => CenterOfSurfacePattern::Each_Surface,
        "each_model" => CenterOfSurfacePattern::Each_Model,
        "each_assembly" => CenterOfSurfacePattern::Each_Assembly,
    ]
);
impl_config_option_enum_value!(
    CenterOfSurfacePattern,
    keys_map_center_of_surface_pattern,
    keys_names_center_of_surface_pattern,
    [Each_Surface, Each_Model, Each_Assembly]
);

config_option_enum_define_static_maps!(
    keys_map_fuzzy_skin_type,
    keys_names_fuzzy_skin_type,
    "`s_keys_map_FuzzySkinType` (`PrintConfig.cpp:269-276`).",
    [
        "none" => FuzzySkinType::None,
        "external" => FuzzySkinType::External,
        "hole" => FuzzySkinType::Hole,
        "all" => FuzzySkinType::All,
        "allwalls" => FuzzySkinType::AllWalls,
        "disabled_fuzzy" => FuzzySkinType::Disabled_fuzzy,
    ]
);
impl_config_option_enum_value!(
    FuzzySkinType,
    keys_map_fuzzy_skin_type,
    keys_names_fuzzy_skin_type,
    [None, External, Hole, All, AllWalls, Disabled_fuzzy]
);

config_option_enum_define_static_maps!(
    keys_map_noise_type,
    keys_names_noise_type,
    "`s_keys_map_NoiseType` (`PrintConfig.cpp:279-286`).",
    [
        "classic" => NoiseType::Classic,
        "perlin" => NoiseType::Perlin,
        "billow" => NoiseType::Billow,
        "ridgedmulti" => NoiseType::RidgedMulti,
        "voronoi" => NoiseType::Voronoi,
        "ripple" => NoiseType::Ripple,
    ]
);
impl_config_option_enum_value!(
    NoiseType,
    keys_map_noise_type,
    keys_names_noise_type,
    [Classic, Perlin, Billow, RidgedMulti, Voronoi, Ripple]
);

config_option_enum_define_static_maps!(
    keys_map_wipe_tower_type,
    keys_names_wipe_tower_type,
    "`s_keys_map_WipeTowerType` (`PrintConfig.cpp:289-292`).",
    [
        "type1" => WipeTowerType::Type1,
        "type2" => WipeTowerType::Type2,
    ]
);
impl_config_option_enum_value!(
    WipeTowerType,
    keys_map_wipe_tower_type,
    keys_names_wipe_tower_type,
    [Type1, Type2]
);

config_option_enum_define_static_maps!(
    keys_map_fuzzy_skin_mode,
    keys_names_fuzzy_skin_mode,
    "`s_keys_map_FuzzySkinMode` (`PrintConfig.cpp:296-300`).",
    [
        "displacement" => FuzzySkinMode::Displacement,
        "extrusion" => FuzzySkinMode::Extrusion,
        "combined" => FuzzySkinMode::Combined,
    ]
);
impl_config_option_enum_value!(
    FuzzySkinMode,
    keys_map_fuzzy_skin_mode,
    keys_names_fuzzy_skin_mode,
    [Displacement, Extrusion, Combined]
);

config_option_enum_define_static_maps!(
    keys_map_top_surface_expansion_direction,
    keys_names_top_surface_expansion_direction,
    "`s_keys_map_TopSurfaceExpansionDirection` (`PrintConfig.cpp:303-307`).",
    [
        "inward_and_outward" => TopSurfaceExpansionDirection::InwardAndOutward,
        "inward" => TopSurfaceExpansionDirection::Inward,
        "outward" => TopSurfaceExpansionDirection::Outward,
    ]
);
impl_config_option_enum_value!(
    TopSurfaceExpansionDirection,
    keys_map_top_surface_expansion_direction,
    keys_names_top_surface_expansion_direction,
    [InwardAndOutward, Inward, Outward]
);

config_option_enum_define_static_maps!(
    keys_map_infill_pattern,
    keys_names_infill_pattern,
    "`s_keys_map_InfillPattern` (`PrintConfig.cpp:310-340`).",
    [
        "monotonic" => InfillPattern::ipMonotonic,
        "monotonicline" => InfillPattern::ipMonotonicLine,
        "rectilinear" => InfillPattern::ipRectilinear,
        "alignedrectilinear" => InfillPattern::ipAlignedRectilinear,
        "zigzag" => InfillPattern::ipZigZag,
        "crosszag" => InfillPattern::ipCrossZag,
        "lockedzag" => InfillPattern::ipLockedZag,
        "line" => InfillPattern::ipLine,
        "grid" => InfillPattern::ipGrid,
        "triangles" => InfillPattern::ipTriangles,
        "tri-hexagon" => InfillPattern::ipStars,
        "cubic" => InfillPattern::ipCubic,
        "adaptivecubic" => InfillPattern::ipAdaptiveCubic,
        "quartercubic" => InfillPattern::ipQuarterCubic,
        "supportcubic" => InfillPattern::ipSupportCubic,
        "lightning" => InfillPattern::ipLightning,
        "honeycomb" => InfillPattern::ipHoneycomb,
        "3dhoneycomb" => InfillPattern::ip3DHoneycomb,
        "lateral-honeycomb" => InfillPattern::ipLateralHoneycomb,
        "lateral-lattice" => InfillPattern::ipLateralLattice,
        "crosshatch" => InfillPattern::ipCrossHatch,
        "tpmsd" => InfillPattern::ipTpmsD,
        "tpmsfk" => InfillPattern::ipTpmsFK,
        "gyroid" => InfillPattern::ipGyroid,
        "concentric" => InfillPattern::ipConcentric,
        "spiralinset" => InfillPattern::ipSpiralInset,
        "hilbertcurve" => InfillPattern::ipHilbertCurve,
        "archimedeanchords" => InfillPattern::ipArchimedeanChords,
        "octagramspiral" => InfillPattern::ipOctagramSpiral,
    ]
);
impl_config_option_enum_value!(
    InfillPattern,
    keys_map_infill_pattern,
    keys_names_infill_pattern,
    [
        ipMonotonic,
        ipMonotonicLine,
        ipRectilinear,
        ipAlignedRectilinear,
        ipZigZag,
        ipCrossZag,
        ipLockedZag,
        ipLine,
        ipGrid,
        ipTriangles,
        ipStars,
        ipCubic,
        ipAdaptiveCubic,
        ipQuarterCubic,
        ipSupportCubic,
        ipLightning,
        ipHoneycomb,
        ip3DHoneycomb,
        ipLateralHoneycomb,
        ipLateralLattice,
        ipCrossHatch,
        ipTpmsD,
        ipTpmsFK,
        ipGyroid,
        ipConcentric,
        ipSpiralInset,
        ipHilbertCurve,
        ipArchimedeanChords,
        ipOctagramSpiral,
        ipSupportBase,
        ipConcentricInternal,
        ipCount,
    ]
);

config_option_enum_define_static_maps!(
    keys_map_ironing_type,
    keys_names_ironing_type,
    "`s_keys_map_IroningType` (`PrintConfig.cpp:343-348`).",
    [
        "no ironing" => IroningType::NoIroning,
        "top" => IroningType::TopSurfaces,
        "topmost" => IroningType::TopmostOnly,
        "solid" => IroningType::AllSolid,
    ]
);
impl_config_option_enum_value!(
    IroningType,
    keys_map_ironing_type,
    keys_names_ironing_type,
    [NoIroning, TopSurfaces, TopmostOnly, AllSolid, Count]
);

config_option_enum_define_static_maps!(
    keys_map_wall_infill_order,
    keys_names_wall_infill_order,
    "`s_keys_map_WallInfillOrder` (`PrintConfig.cpp:352-359`). The vendored table writes \
     `\"inner-outer-inner wall/infill\"` twice, both times for `InnerOuterInnerInfill`; a \
     `std::map` keeps one entry for a repeated key and `TConfigEnumValues` does the same, so \
     the duplicate is transcribed once here exactly as the oracle stores it once.",
    [
        "inner wall/outer wall/infill" => WallInfillOrder::InnerOuterInfill,
        "outer wall/inner wall/infill" => WallInfillOrder::OuterInnerInfill,
        "inner-outer-inner wall/infill" => WallInfillOrder::InnerOuterInnerInfill,
        "infill/inner wall/outer wall" => WallInfillOrder::InfillInnerOuter,
        "infill/outer wall/inner wall" => WallInfillOrder::InfillOuterInner,
    ]
);
impl_config_option_enum_value!(
    WallInfillOrder,
    keys_map_wall_infill_order,
    keys_names_wall_infill_order,
    [
        InnerOuterInfill,
        OuterInnerInfill,
        InfillInnerOuter,
        InfillOuterInner,
        InnerOuterInnerInfill,
        Count,
    ]
);

config_option_enum_define_static_maps!(
    keys_map_wall_sequence,
    keys_names_wall_sequence,
    "`s_keys_map_WallSequence` (`PrintConfig.cpp:363-368`).",
    [
        "inner wall/outer wall" => WallSequence::InnerOuter,
        "outer wall/inner wall" => WallSequence::OuterInner,
        "inner-outer-inner wall" => WallSequence::InnerOuterInner,
    ]
);
impl_config_option_enum_value!(
    WallSequence,
    keys_map_wall_sequence,
    keys_names_wall_sequence,
    [InnerOuter, OuterInner, InnerOuterInner, Count]
);

config_option_enum_define_static_maps!(
    keys_map_wall_direction,
    keys_names_wall_direction,
    "`s_keys_map_WallDirection` (`PrintConfig.cpp:372-375`).",
    [
        "ccw" => WallDirection::CounterClockwise,
        "cw" => WallDirection::Clockwise,
    ]
);
impl_config_option_enum_value!(
    WallDirection,
    keys_map_wall_direction,
    keys_names_wall_direction,
    [CounterClockwise, Clockwise, Count]
);

config_option_enum_define_static_maps!(
    keys_map_surface_fill_order,
    keys_names_surface_fill_order,
    "`s_keys_map_SurfaceFillOrder` (`PrintConfig.cpp:379-383`).",
    [
        "default" => SurfaceFillOrder::Default,
        "outward" => SurfaceFillOrder::Outward,
        "inward" => SurfaceFillOrder::Inward,
    ]
);
impl_config_option_enum_value!(
    SurfaceFillOrder,
    keys_map_surface_fill_order,
    keys_names_surface_fill_order,
    [Default, Outward, Inward, Count]
);

config_option_enum_define_static_maps!(
    keys_map_adaptive_infill_mode,
    keys_names_adaptive_infill_mode,
    "`s_keys_map_AdaptiveInfillMode` (`PrintConfig.cpp:387-396`).",
    [
        "disabled" => AdaptiveInfillMode::Disabled,
        "distance_warp" => AdaptiveInfillMode::DistanceWarp,
        "smooth_blend" => AdaptiveInfillMode::SmoothBlend,
        "stepped_shells" => AdaptiveInfillMode::SteppedShells,
        "lobes" => AdaptiveInfillMode::Lobes,
        "normal_z" => AdaptiveInfillMode::NormalZ,
        "normal_y" => AdaptiveInfillMode::NormalY,
        "normal_x" => AdaptiveInfillMode::NormalX,
    ]
);
impl_config_option_enum_value!(
    AdaptiveInfillMode,
    keys_map_adaptive_infill_mode,
    keys_names_adaptive_infill_mode,
    [
        Disabled,
        DistanceWarp,
        SmoothBlend,
        SteppedShells,
        Lobes,
        NormalZ,
        NormalY,
        NormalX,
        Count,
    ]
);

config_option_enum_define_static_maps!(
    keys_map_adaptive_infill_gradient,
    keys_names_adaptive_infill_gradient,
    "`s_keys_map_AdaptiveInfillGradient` (`PrintConfig.cpp:399-403`).",
    [
        "linear" => AdaptiveInfillGradient::Linear,
        "quadratic" => AdaptiveInfillGradient::Quadratic,
        "exponential" => AdaptiveInfillGradient::Exponential,
    ]
);
impl_config_option_enum_value!(
    AdaptiveInfillGradient,
    keys_map_adaptive_infill_gradient,
    keys_names_adaptive_infill_gradient,
    [Linear, Quadratic, Exponential]
);

config_option_enum_define_static_maps!(
    keys_map_print_sequence,
    keys_names_print_sequence,
    "`s_keys_map_PrintSequence` (`PrintConfig.cpp:407-410`).",
    [
        "by layer" => PrintSequence::ByLayer,
        "by object" => PrintSequence::ByObject,
    ]
);
impl_config_option_enum_value!(
    PrintSequence,
    keys_map_print_sequence,
    keys_names_print_sequence,
    [ByLayer, ByObject, ByDefault, Count]
);

config_option_enum_define_static_maps!(
    keys_map_print_order,
    keys_names_print_order,
    "`s_keys_map_PrintOrder` (`PrintConfig.cpp:413-418`).",
    [
        "default" => PrintOrder::Default,
        "as_obj_list" => PrintOrder::AsObjectList,
        "best_of" => PrintOrder::BestOfStrategies,
        "snake" => PrintOrder::Snake,
    ]
);
impl_config_option_enum_value!(
    PrintOrder,
    keys_map_print_order,
    keys_names_print_order,
    [Default, AsObjectList, BestOfStrategies, Snake, Count]
);

config_option_enum_define_static_maps!(
    keys_map_slicing_mode,
    keys_names_slicing_mode,
    "`s_keys_map_SlicingMode` (`PrintConfig.cpp:421-424`).",
    [
        "regular" => SlicingMode::Regular,
        "even_odd" => SlicingMode::EvenOdd,
        "close_holes" => SlicingMode::CloseHoles,
    ]
);
impl_config_option_enum_value!(
    SlicingMode,
    keys_map_slicing_mode,
    keys_names_slicing_mode,
    [Regular, EvenOdd, CloseHoles]
);

config_option_enum_define_static_maps!(
    keys_map_belt_rotation_axis,
    keys_names_belt_rotation_axis,
    "`s_keys_map_BeltRotationAxis` (`PrintConfig.cpp:428-433`).",
    [
        "none" => BeltRotationAxis::None,
        "x" => BeltRotationAxis::X,
        "y" => BeltRotationAxis::Y,
        "z" => BeltRotationAxis::Z,
    ]
);
impl_config_option_enum_value!(
    BeltRotationAxis,
    keys_map_belt_rotation_axis,
    keys_names_belt_rotation_axis,
    [None, X, Y, Z]
);

config_option_enum_define_static_maps!(
    keys_map_remap_axis,
    keys_names_remap_axis,
    "`s_keys_map_RemapAxis` (`PrintConfig.cpp:436-446`).",
    [
        "pos_x" => RemapAxis::PosX,
        "pos_y" => RemapAxis::PosY,
        "pos_z" => RemapAxis::PosZ,
        "neg_x" => RemapAxis::NegX,
        "neg_y" => RemapAxis::NegY,
        "neg_z" => RemapAxis::NegZ,
        "rev_x" => RemapAxis::RevX,
        "rev_y" => RemapAxis::RevY,
        "rev_z" => RemapAxis::RevZ,
    ]
);
impl_config_option_enum_value!(
    RemapAxis,
    keys_map_remap_axis,
    keys_names_remap_axis,
    [PosX, PosY, PosZ, NegX, NegY, NegZ, RevX, RevY, RevZ]
);

config_option_enum_define_static_maps!(
    keys_map_support_material_pattern,
    keys_names_support_material_pattern,
    "`s_keys_map_SupportMaterialPattern` (`PrintConfig.cpp:449-456`).",
    [
        "rectilinear" => SupportMaterialPattern::smpRectilinear,
        "rectilinear-grid" => SupportMaterialPattern::smpRectilinearGrid,
        "honeycomb" => SupportMaterialPattern::smpHoneycomb,
        "lightning" => SupportMaterialPattern::smpLightning,
        "default" => SupportMaterialPattern::smpDefault,
        "hollow" => SupportMaterialPattern::smpNone,
    ]
);
impl_config_option_enum_value!(
    SupportMaterialPattern,
    keys_map_support_material_pattern,
    keys_names_support_material_pattern,
    [
        smpDefault,
        smpRectilinear,
        smpRectilinearGrid,
        smpHoneycomb,
        smpLightning,
        smpNone,
    ]
);

config_option_enum_define_static_maps!(
    keys_map_support_material_style,
    keys_names_support_material_style,
    "`s_keys_map_SupportMaterialStyle` (`PrintConfig.cpp:459-467`).",
    [
        "default" => SupportMaterialStyle::smsDefault,
        "grid" => SupportMaterialStyle::smsGrid,
        "snug" => SupportMaterialStyle::smsSnug,
        "organic" => SupportMaterialStyle::smsTreeOrganic,
        "tree_slim" => SupportMaterialStyle::smsTreeSlim,
        "tree_strong" => SupportMaterialStyle::smsTreeStrong,
        "tree_hybrid" => SupportMaterialStyle::smsTreeHybrid,
    ]
);
impl_config_option_enum_value!(
    SupportMaterialStyle,
    keys_map_support_material_style,
    keys_names_support_material_style,
    [
        smsDefault,
        smsGrid,
        smsSnug,
        smsTreeOrganic,
        smsTreeSlim,
        smsTreeStrong,
        smsTreeHybrid,
    ]
);

config_option_enum_define_static_maps!(
    keys_map_support_material_interface_pattern,
    keys_names_support_material_interface_pattern,
    "`s_keys_map_SupportMaterialInterfacePattern` (`PrintConfig.cpp:470-477`).",
    [
        "auto" => SupportMaterialInterfacePattern::smipAuto,
        "rectilinear" => SupportMaterialInterfacePattern::smipRectilinear,
        "concentric" => SupportMaterialInterfacePattern::smipConcentric,
        "spiralinset" => SupportMaterialInterfacePattern::smipSpiralInset,
        "rectilinear_interlaced" => SupportMaterialInterfacePattern::smipRectilinearInterlaced,
        "grid" => SupportMaterialInterfacePattern::smipGrid,
    ]
);
impl_config_option_enum_value!(
    SupportMaterialInterfacePattern,
    keys_map_support_material_interface_pattern,
    keys_names_support_material_interface_pattern,
    [
        smipAuto,
        smipRectilinear,
        smipConcentric,
        smipSpiralInset,
        smipRectilinearInterlaced,
        smipGrid,
    ]
);

config_option_enum_define_static_maps!(
    keys_map_support_type,
    keys_names_support_type,
    "`s_keys_map_SupportType` (`PrintConfig.cpp:480-485`).",
    [
        "normal(auto)" => SupportType::stNormalAuto,
        "tree(auto)" => SupportType::stTreeAuto,
        "normal(manual)" => SupportType::stNormal,
        "tree(manual)" => SupportType::stTree,
    ]
);
impl_config_option_enum_value!(
    SupportType,
    keys_map_support_type,
    keys_names_support_type,
    [stNormalAuto, stTreeAuto, stNormal, stTree]
);

config_option_enum_define_static_maps!(
    keys_map_seam_position,
    keys_names_seam_position,
    "`s_keys_map_SeamPosition` (`PrintConfig.cpp:488-494`).",
    [
        "nearest" => SeamPosition::spNearest,
        "aligned" => SeamPosition::spAligned,
        "aligned_back" => SeamPosition::spAlignedBack,
        "back" => SeamPosition::spRear,
        "random" => SeamPosition::spRandom,
    ]
);
impl_config_option_enum_value!(
    SeamPosition,
    keys_map_seam_position,
    keys_names_seam_position,
    [spNearest, spAligned, spAlignedBack, spRear, spRandom]
);

config_option_enum_define_static_maps!(
    keys_map_seam_scarf_type,
    keys_names_seam_scarf_type,
    "`s_keys_map_SeamScarfType` (`PrintConfig.cpp:498-502`).",
    [
        "none" => SeamScarfType::None,
        "external" => SeamScarfType::External,
        "all" => SeamScarfType::All,
    ]
);
impl_config_option_enum_value!(
    SeamScarfType,
    keys_map_seam_scarf_type,
    keys_names_seam_scarf_type,
    [None, External, All]
);

config_option_enum_define_static_maps!(
    keys_map_ensure_vertical_shell_thickness,
    keys_names_ensure_vertical_shell_thickness,
    "`s_keys_map_EnsureVerticalShellThickness` (`PrintConfig.cpp:506-511`).",
    [
        "none" => EnsureVerticalShellThickness::evstNone,
        "ensure_critical_only" => EnsureVerticalShellThickness::evstCriticalOnly,
        "ensure_moderate" => EnsureVerticalShellThickness::evstModerate,
        "ensure_all" => EnsureVerticalShellThickness::evstAll,
    ]
);
impl_config_option_enum_value!(
    EnsureVerticalShellThickness,
    keys_map_ensure_vertical_shell_thickness,
    keys_names_ensure_vertical_shell_thickness,
    [evstNone, evstCriticalOnly, evstModerate, evstAll]
);

config_option_enum_define_static_maps!(
    keys_map_internal_bridge_filter,
    keys_names_internal_bridge_filter,
    "`s_keys_map_InternalBridgeFilter` (`PrintConfig.cpp:515-519`).",
    [
        "disabled" => InternalBridgeFilter::ibfDisabled,
        "limited" => InternalBridgeFilter::ibfLimited,
        "nofilter" => InternalBridgeFilter::ibfNofilter,
    ]
);
impl_config_option_enum_value!(
    InternalBridgeFilter,
    keys_map_internal_bridge_filter,
    keys_names_internal_bridge_filter,
    [ibfDisabled, ibfLimited, ibfNofilter]
);

config_option_enum_define_static_maps!(
    keys_map_enable_extra_bridge_layer,
    keys_names_enable_extra_bridge_layer,
    "`s_keys_map_EnableExtraBridgeLayer` (`PrintConfig.cpp:522-527`).",
    [
        "disabled" => EnableExtraBridgeLayer::eblDisabled,
        "external_bridge_only" => EnableExtraBridgeLayer::eblExternalBridgeOnly,
        "internal_bridge_only" => EnableExtraBridgeLayer::eblInternalBridgeOnly,
        "apply_to_all" => EnableExtraBridgeLayer::eblApplyToAll,
    ]
);
impl_config_option_enum_value!(
    EnableExtraBridgeLayer,
    keys_map_enable_extra_bridge_layer,
    keys_names_enable_extra_bridge_layer,
    [
        eblDisabled,
        eblExternalBridgeOnly,
        eblInternalBridgeOnly,
        eblApplyToAll,
    ]
);

config_option_enum_define_static_maps!(
    keys_map_gap_fill_target,
    keys_names_gap_fill_target,
    "`s_keys_map_GapFillTarget` (`PrintConfig.cpp:531-535`).",
    [
        "everywhere" => GapFillTarget::gftEverywhere,
        "topbottom" => GapFillTarget::gftTopBottom,
        "nowhere" => GapFillTarget::gftNowhere,
    ]
);
impl_config_option_enum_value!(
    GapFillTarget,
    keys_map_gap_fill_target,
    keys_names_gap_fill_target,
    [gftEverywhere, gftTopBottom, gftNowhere]
);

config_option_enum_define_static_maps!(
    keys_map_sla_display_orientation,
    keys_names_sla_display_orientation,
    "`s_keys_map_SLADisplayOrientation` (`PrintConfig.cpp:538-541`).",
    [
        "landscape" => SLADisplayOrientation::sladoLandscape,
        "portrait" => SLADisplayOrientation::sladoPortrait,
    ]
);
impl_config_option_enum_value!(
    SLADisplayOrientation,
    keys_map_sla_display_orientation,
    keys_names_sla_display_orientation,
    [sladoLandscape, sladoPortrait]
);

config_option_enum_define_static_maps!(
    keys_map_sla_pillar_connection_mode,
    keys_names_sla_pillar_connection_mode,
    "`s_keys_map_SLAPillarConnectionMode` (`PrintConfig.cpp:544-548`).",
    [
        "zigzag" => SLAPillarConnectionMode::slapcmZigZag,
        "cross" => SLAPillarConnectionMode::slapcmCross,
        "dynamic" => SLAPillarConnectionMode::slapcmDynamic,
    ]
);
impl_config_option_enum_value!(
    SLAPillarConnectionMode,
    keys_map_sla_pillar_connection_mode,
    keys_names_sla_pillar_connection_mode,
    [slapcmZigZag, slapcmCross, slapcmDynamic]
);

config_option_enum_define_static_maps!(
    keys_map_sla_material_speed,
    keys_names_sla_material_speed,
    "`s_keys_map_SLAMaterialSpeed` (`PrintConfig.cpp:551-554`).",
    [
        "slow" => SLAMaterialSpeed::slamsSlow,
        "fast" => SLAMaterialSpeed::slamsFast,
    ]
);
impl_config_option_enum_value!(
    SLAMaterialSpeed,
    keys_map_sla_material_speed,
    keys_names_sla_material_speed,
    [slamsSlow, slamsFast]
);

config_option_enum_define_static_maps!(
    keys_map_brim_type,
    keys_names_brim_type,
    "`s_keys_map_BrimType` (`PrintConfig.cpp:557-566`).",
    [
        "no_brim" => BrimType::btNoBrim,
        "outer_only" => BrimType::btOuterOnly,
        "inner_only" => BrimType::btInnerOnly,
        "outer_and_inner" => BrimType::btOuterAndInner,
        "auto_brim" => BrimType::btAutoBrim,
        "brim_ears" => BrimType::btEar,
        "painted" => BrimType::btPainted,
        "leading_edge_only" => BrimType::btLeadingEdgeOnly,
    ]
);
impl_config_option_enum_value!(
    BrimType,
    keys_map_brim_type,
    keys_names_brim_type,
    [
        btAutoBrim,
        btEar,
        btPainted,
        btOuterOnly,
        btInnerOnly,
        btOuterAndInner,
        btNoBrim,
        btLeadingEdgeOnly,
    ]
);

config_option_enum_define_static_maps!(
    keys_map_timelapse_type,
    keys_names_timelapse_type,
    "`s_keys_map_TimelapseType` (`PrintConfig.cpp:570-573`). The keys are the strings \"0\" and \
     \"1\", which the vendored comment at `:569` records as deliberate backward compatibility \
     with old files -- they are map keys, not numbers.",
    [
        "0" => TimelapseType::tlTraditional,
        "1" => TimelapseType::tlSmooth,
    ]
);
impl_config_option_enum_value!(
    TimelapseType,
    keys_map_timelapse_type,
    keys_names_timelapse_type,
    [tlTraditional, tlSmooth]
);

config_option_enum_define_static_maps!(
    keys_map_skirt_type,
    keys_names_skirt_type,
    "`s_keys_map_SkirtType` (`PrintConfig.cpp:576-579`).",
    [
        "combined" => SkirtType::stCombined,
        "perobject" => SkirtType::stPerObject,
    ]
);
impl_config_option_enum_value!(
    SkirtType,
    keys_map_skirt_type,
    keys_names_skirt_type,
    [stCombined, stPerObject]
);

config_option_enum_define_static_maps!(
    keys_map_draft_shield,
    keys_names_draft_shield,
    "`s_keys_map_DraftShield` (`PrintConfig.cpp:582-585`).",
    [
        "disabled" => DraftShield::dsDisabled,
        "enabled" => DraftShield::dsEnabled,
    ]
);
impl_config_option_enum_value!(
    DraftShield,
    keys_map_draft_shield,
    keys_names_draft_shield,
    [dsDisabled, dsEnabled]
);

config_option_enum_define_static_maps!(
    keys_map_forward_compatibility_substitution_rule,
    keys_names_forward_compatibility_substitution_rule,
    "`s_keys_map_ForwardCompatibilitySubstitutionRule` (`PrintConfig.cpp:588-592`).",
    [
        "disable" => slic3r_config::config::ForwardCompatibilitySubstitutionRule::Disable,
        "enable" => slic3r_config::config::ForwardCompatibilitySubstitutionRule::Enable,
        "enable_silent" => slic3r_config::config::ForwardCompatibilitySubstitutionRule::EnableSilent,
    ]
);

config_option_enum_define_static_maps!(
    keys_map_overhang_fan_threshold,
    keys_names_overhang_fan_threshold,
    "`s_keys_map_OverhangFanThreshold` (`PrintConfig.cpp:595-602`). The keys are percentage \
     strings, not numbers, and do not run in step with the values.",
    [
        "0%" => OverhangFanThreshold::Overhang_threshold_none,
        "10%" => OverhangFanThreshold::Overhang_threshold_1_4,
        "25%" => OverhangFanThreshold::Overhang_threshold_2_4,
        "50%" => OverhangFanThreshold::Overhang_threshold_3_4,
        "75%" => OverhangFanThreshold::Overhang_threshold_4_4,
        "95%" => OverhangFanThreshold::Overhang_threshold_bridge,
    ]
);
impl_config_option_enum_value!(
    OverhangFanThreshold,
    keys_map_overhang_fan_threshold,
    keys_names_overhang_fan_threshold,
    [
        Overhang_threshold_none,
        Overhang_threshold_1_4,
        Overhang_threshold_2_4,
        Overhang_threshold_3_4,
        Overhang_threshold_4_4,
        Overhang_threshold_bridge,
    ]
);

config_option_enum_define_static_maps!(
    keys_map_bed_type,
    keys_names_bed_type,
    "`s_keys_map_BedType` (`PrintConfig.cpp:606-614`). The table's order is not the \
     declaration's: `Supertack Plate` is listed second while `btSuperTack` is the seventh \
     enumerator, so `enum_names_from_keys_map` sizes the name vector to `btSuperTack + 1`.",
    [
        "Default Plate" => BedType::btDefault,
        "Supertack Plate" => BedType::btSuperTack,
        "Cool Plate" => BedType::btPC,
        "Engineering Plate" => BedType::btEP,
        "High Temp Plate" => BedType::btPEI,
        "Textured PEI Plate" => BedType::btPTE,
        "Textured Cool Plate" => BedType::btPCT,
    ]
);
impl_config_option_enum_value!(
    BedType,
    keys_map_bed_type,
    keys_names_bed_type,
    [
        btDefault,
        btPC,
        btEP,
        btPEI,
        btPTE,
        btPCT,
        btSuperTack,
        btCount,
    ]
);

config_option_enum_define_static_maps!(
    keys_map_layer_seq,
    keys_names_layer_seq,
    "`s_keys_map_LayerSeq` (`PrintConfig.cpp:618-621`).",
    [
        "Auto" => LayerSeq::flsAuto,
        "Customize" => LayerSeq::flsCustomize,
    ]
);
impl_config_option_enum_value!(
    LayerSeq,
    keys_map_layer_seq,
    keys_names_layer_seq,
    [flsAuto, flsCustomize]
);

config_option_enum_define_static_maps!(
    keys_map_nozzle_type,
    keys_names_nozzle_type,
    // The continuation lines of this doc string carry no digits on purpose: `port/port-status`
    // masks string contents line by line, so a digit on the second line of a `\`-continued
    // string literal would be read as a bare numeric literal in code.
    "`s_keys_map_NozzleType` (`PrintConfig.cpp:624-630`). `NozzleType` itself is declared in `CommonDefs.hpp:11-21`, \
     outside this unit, and is already ported by the `foundation-common-defs` task as \
     `slic3r_commondefs::common_defs::NozzleType`; only the key map is this unit's. The table \
     stops at `ntBrass`: `ntE3D` and `ntCount` are declared but have no key, so \
     `enum_names_from_keys_map` sizes the name vector to one past `ntBrass`.",
    [
        "undefine" => slic3r_commondefs::common_defs::NozzleType::ntUndefine,
        "hardened_steel" => slic3r_commondefs::common_defs::NozzleType::ntHardenedSteel,
        "stainless_steel" => slic3r_commondefs::common_defs::NozzleType::ntStainlessSteel,
        "tungsten_carbide" => slic3r_commondefs::common_defs::NozzleType::ntTungstenCarbide,
        "brass" => slic3r_commondefs::common_defs::NozzleType::ntBrass,
    ]
);

config_option_enum_define_static_maps!(
    keys_map_fan_direction,
    keys_names_fan_direction,
    "`s_keys_map_FanDirection` (`PrintConfig.cpp:633-638`).",
    [
        "undefine" => FanDirection::fdUndefine,
        "left" => FanDirection::fdLeft,
        "right" => FanDirection::fdRight,
        "both" => FanDirection::fdBoth,
    ]
);
impl_config_option_enum_value!(
    FanDirection,
    keys_map_fan_direction,
    keys_names_fan_direction,
    [fdUndefine, fdLeft, fdRight, fdBoth]
);

config_option_enum_define_static_maps!(
    keys_map_printer_structure,
    keys_names_printer_structure,
    "`s_keys_map_PrinterStructure` (`PrintConfig.cpp:641-647`).",
    [
        "undefine" => PrinterStructure::psUndefine,
        "corexy" => PrinterStructure::psCoreXY,
        "i3" => PrinterStructure::psI3,
        "hbot" => PrinterStructure::psHbot,
        "delta" => PrinterStructure::psDelta,
    ]
);
impl_config_option_enum_value!(
    PrinterStructure,
    keys_map_printer_structure,
    keys_names_printer_structure,
    [psUndefine, psCoreXY, psI3, psHbot, psDelta]
);

config_option_enum_define_static_maps!(
    keys_map_input_shaper_type,
    keys_names_input_shaper_type,
    "`s_keys_map_InputShaperType` (`PrintConfig.cpp:650-664`).",
    [
        "Default" => InputShaperType::Default,
        "MZV" => InputShaperType::MZV,
        "ZV" => InputShaperType::ZV,
        "ZVD" => InputShaperType::ZVD,
        "ZVDD" => InputShaperType::ZVDD,
        "ZVDDD" => InputShaperType::ZVDDD,
        "EI" => InputShaperType::EI,
        "EI2" => InputShaperType::EI2,
        "2HUMP_EI" => InputShaperType::TwoHumpEI,
        "EI3" => InputShaperType::EI3,
        "3HUMP_EI" => InputShaperType::ThreeHumpEI,
        "DAA" => InputShaperType::DAA,
        "Disable" => InputShaperType::Disable,
    ]
);
impl_config_option_enum_value!(
    InputShaperType,
    keys_map_input_shaper_type,
    keys_names_input_shaper_type,
    [
        Default,
        MZV,
        ZV,
        ZVD,
        ZVDD,
        ZVDDD,
        EI,
        EI2,
        TwoHumpEI,
        EI3,
        ThreeHumpEI,
        DAA,
        Disable,
    ]
);

config_option_enum_define_static_maps!(
    keys_map_perimeter_generator_type,
    keys_names_perimeter_generator_type,
    "`s_keys_map_PerimeterGeneratorType` (`PrintConfig.cpp:667-670`).",
    [
        "classic" => PerimeterGeneratorType::Classic,
        "arachne" => PerimeterGeneratorType::Arachne,
    ]
);
impl_config_option_enum_value!(
    PerimeterGeneratorType,
    keys_map_perimeter_generator_type,
    keys_names_perimeter_generator_type,
    [Classic, Arachne]
);

config_option_enum_define_static_maps!(
    keys_map_tool_change_ordering_type,
    keys_names_tool_change_ordering_type,
    "`s_keys_map_ToolChangeOrderingType` (`PrintConfig.cpp:673-676`).",
    [
        "default" => ToolChangeOrderingType::Default,
        "cyclic" => ToolChangeOrderingType::Cyclic,
    ]
);
impl_config_option_enum_value!(
    ToolChangeOrderingType,
    keys_map_tool_change_ordering_type,
    keys_names_tool_change_ordering_type,
    [Default, Cyclic]
);

config_option_enum_define_static_maps!(
    keys_map_z_hop_type,
    keys_names_z_hop_type,
    "`s_keys_map_ZHopType` (`PrintConfig.cpp:679-684`). `zhtCount` is declared but has no key.",
    [
        "Auto Lift" => ZHopType::zhtAuto,
        "Normal Lift" => ZHopType::zhtNormal,
        "Slope Lift" => ZHopType::zhtSlope,
        "Spiral Lift" => ZHopType::zhtSpiral,
    ]
);
impl_config_option_enum_value!(
    ZHopType,
    keys_map_z_hop_type,
    keys_names_z_hop_type,
    [zhtAuto, zhtNormal, zhtSlope, zhtSpiral, zhtCount]
);

config_option_enum_define_static_maps!(
    keys_map_retract_lift_enforce_type,
    keys_names_retract_lift_enforce_type,
    "`s_keys_map_RetractLiftEnforceType` (`PrintConfig.cpp:687-692`).",
    [
        "All Surfaces" => RetractLiftEnforceType::rletAllSurfaces,
        "Top Only" => RetractLiftEnforceType::rletTopOnly,
        "Bottom Only" => RetractLiftEnforceType::rletBottomOnly,
        "Top and Bottom" => RetractLiftEnforceType::rletTopAndBottom,
    ]
);
impl_config_option_enum_value!(
    RetractLiftEnforceType,
    keys_map_retract_lift_enforce_type,
    keys_names_retract_lift_enforce_type,
    [
        rletAllSurfaces,
        rletTopOnly,
        rletBottomOnly,
        rletTopAndBottom,
    ]
);

config_option_enum_define_static_maps!(
    keys_map_gcode_thumbnails_format,
    keys_names_gcode_thumbnails_format,
    "`s_keys_map_GCodeThumbnailsFormat` (`PrintConfig.cpp:695-701`). The key for \
     `GCodeThumbnailsFormat::ColPic` is the upper-case `\"COLPIC\"`, not the enumerator's own \
     mixed-case spelling.",
    [
        "PNG" => GCodeThumbnailsFormat::PNG,
        "JPG" => GCodeThumbnailsFormat::JPG,
        "QOI" => GCodeThumbnailsFormat::QOI,
        "BTT_TFT" => GCodeThumbnailsFormat::BTT_TFT,
        "COLPIC" => GCodeThumbnailsFormat::ColPic,
    ]
);
impl_config_option_enum_value!(
    GCodeThumbnailsFormat,
    keys_map_gcode_thumbnails_format,
    keys_names_gcode_thumbnails_format,
    [PNG, JPG, QOI, BTT_TFT, ColPic]
);

config_option_enum_define_static_maps!(
    keys_map_counterbore_hole_bridging_option,
    keys_names_counterbore_hole_bridging_option,
    "`s_keys_map_CounterboreHoleBridgingOption` (`PrintConfig.cpp:704-708`).",
    [
        "none" => CounterboreHoleBridgingOption::chbNone,
        "partiallybridge" => CounterboreHoleBridgingOption::chbBridges,
        "sacrificiallayer" => CounterboreHoleBridgingOption::chbFilled,
    ]
);
impl_config_option_enum_value!(
    CounterboreHoleBridgingOption,
    keys_map_counterbore_hole_bridging_option,
    keys_names_counterbore_hole_bridging_option,
    [chbNone, chbBridges, chbFilled]
);

config_option_enum_define_static_maps!(
    keys_map_wipe_tower_wall_type,
    keys_names_wipe_tower_wall_type,
    "`s_keys_map_WipeTowerWallType` (`PrintConfig.cpp:711-715`).",
    [
        "rectangle" => WipeTowerWallType::wtwRectangle,
        "cone" => WipeTowerWallType::wtwCone,
        "rib" => WipeTowerWallType::wtwRib,
    ]
);
impl_config_option_enum_value!(
    WipeTowerWallType,
    keys_map_wipe_tower_wall_type,
    keys_names_wipe_tower_wall_type,
    [wtwRectangle, wtwCone, wtwRib]
);

config_option_enum_define_static_maps!(
    keys_map_extruder_type,
    keys_names_extruder_type,
    "`s_keys_map_ExtruderType` (`PrintConfig.cpp:718-721`).",
    [
        "Direct Drive" => ExtruderType::etDirectDrive,
        "Bowden" => ExtruderType::etBowden,
    ]
);
impl_config_option_enum_value!(
    ExtruderType,
    keys_map_extruder_type,
    keys_names_extruder_type,
    [etDirectDrive, etBowden]
);

config_option_enum_define_static_maps!(
    keys_map_nozzle_volume_type,
    keys_names_nozzle_volume_type,
    "`s_keys_map_NozzleVolumeType` (`PrintConfig.cpp:724-731`). The reserved value 4 has no \
     key, so `enum_names_from_keys_map` leaves that index empty.",
    [
        "Standard" => NozzleVolumeType::nvtStandard,
        "High Flow" => NozzleVolumeType::nvtHighFlow,
        "TPU High Flow" => NozzleVolumeType::nvtTPUHighFlow,
        "Hybrid" => NozzleVolumeType::nvtHybrid,
        "E3D High Flow" => NozzleVolumeType::nvtE3DHighFlow,
        "Extra High Flow" => NozzleVolumeType::nvtExtraHighFlow,
    ]
);
impl_config_option_enum_value!(
    NozzleVolumeType,
    keys_map_nozzle_volume_type,
    keys_names_nozzle_volume_type,
    [
        nvtStandard,
        nvtHighFlow,
        nvtHybrid,
        nvtTPUHighFlow,
        nvtE3DHighFlow,
        nvtExtraHighFlow,
    ]
);

config_option_enum_define_static_maps!(
    keys_map_filament_map_mode,
    keys_names_filament_map_mode,
    "`s_keys_map_FilamentMapMode` (`PrintConfig.cpp:734-739`). `fmmDefault` is declared but \
     has no key.",
    [
        "Auto For Flush" => FilamentMapMode::fmmAutoForFlush,
        "Auto For Match" => FilamentMapMode::fmmAutoForMatch,
        "Manual" => FilamentMapMode::fmmManual,
        "Nozzle Manual" => FilamentMapMode::fmmNozzleManual,
    ]
);
impl_config_option_enum_value!(
    FilamentMapMode,
    keys_map_filament_map_mode,
    keys_names_filament_map_mode,
    [
        fmmAutoForFlush,
        fmmAutoForMatch,
        fmmManual,
        fmmNozzleManual,
        fmmDefault,
    ]
);

config_option_enum_define_static_maps!(
    keys_map_bed_exclude_volume_mode,
    keys_names_bed_exclude_volume_mode,
    "`s_keys_map_BedExcludeVolumeMode` (`PrintConfig.cpp:742-746`).",
    [
        "shared" => BedExcludeVolumeMode::Shared,
        "toolhead_offset" => BedExcludeVolumeMode::ToolheadOffset,
        "per_extruder" => BedExcludeVolumeMode::PerExtruder,
    ]
);
impl_config_option_enum_value!(
    BedExcludeVolumeMode,
    keys_map_bed_exclude_volume_mode,
    keys_names_bed_exclude_volume_mode,
    [Shared, ToolheadOffset, PerExtruder]
);

config_option_enum_define_static_maps!(
    keys_map_imex_tool_layout,
    keys_names_imex_tool_layout,
    "`s_keys_map_ImexToolLayout` (`PrintConfig.cpp:751-756`). The hyphenated keys are kept deliberately, per the vendored comment just above the table \
     (it records that existing presets and 3MFs must keep deserializing unchanged after the \
     `coString` to `coEnum` migration).",
    [
        "front-left" => ImexToolLayout::FrontLeft,
        "front-right" => ImexToolLayout::FrontRight,
        "rear-left" => ImexToolLayout::RearLeft,
        "rear-right" => ImexToolLayout::RearRight,
    ]
);
impl_config_option_enum_value!(
    ImexToolLayout,
    keys_map_imex_tool_layout,
    keys_names_imex_tool_layout,
    [FrontLeft, FrontRight, RearLeft, RearRight, Count]
);

config_option_enum_define_static_maps!(
    keys_map_imex_viz_theme,
    keys_names_imex_viz_theme,
    "`s_keys_map_ImexVizTheme` (`PrintConfig.cpp:759-764`).",
    [
        "standard" => ImexVizTheme::Standard,
        "deuteranopia" => ImexVizTheme::Deuteranopia,
        "tritanopia" => ImexVizTheme::Tritanopia,
        "high_contrast" => ImexVizTheme::HighContrast,
    ]
);
impl_config_option_enum_value!(
    ImexVizTheme,
    keys_map_imex_viz_theme,
    keys_names_imex_viz_theme,
    [Standard, Deuteranopia, Tritanopia, HighContrast, Count]
);

config_option_enum_define_static_maps!(
    keys_map_prime_volume_mode,
    keys_names_prime_volume_mode,
    "`s_keys_map_PrimeVolumeMode` (`PrintConfig.cpp:768-772`).",
    [
        "Default" => PrimeVolumeMode::pvmDefault,
        "Saving" => PrimeVolumeMode::pvmSaving,
        "Fast" => PrimeVolumeMode::pvmFast,
    ]
);
impl_config_option_enum_value!(
    PrimeVolumeMode,
    keys_map_prime_volume_mode,
    keys_names_prime_volume_mode,
    [pvmDefault, pvmSaving, pvmFast]
);

// `PrinterTechnology`, `ForwardCompatibilitySubstitutionRule` (both `slic3r-config`) and
// `NozzleType` (`slic3r-commondefs`) get their key maps above but **no**
// `impl_config_option_enum_value!`: `ConfigOptionEnumValue` is declared in `slic3r-config`, and
// Rust's orphan rule forbids a crate implementing a foreign trait for a foreign type. The
// vendored `CONFIG_OPTION_ENUM_DEFINE_STATIC_MAPS` specialises `ConfigOptionEnum<T>` for all
// three in this translation unit, so this is a real divergence of mechanism and is named in this
// task's pull request body rather than worked around silently. It costs nothing observable here:
// no option in the registry stores a default of type `ConfigOptionEnum<NozzleType>` or
// `ConfigOptionEnum<ForwardCompatibilitySubstitutionRule>` (`nozzle_type` is a `coEnums` whose
// default is a `ConfigOptionEnumsGenericNullable`, `PrintConfig.cpp:4480-4496`), and
// `printer_technology`'s default is carried as a `ConfigOptionEnumGeneric` over the key map
// above, whose `serialize()` returns the same `"FFF"` the oracle's
// `ConfigOptionEnum<PrinterTechnology>` does.

// --------------------------------------------------------------------------------------------
// Step 2, continued: the header's own free functions and lookup tables that live beside the
// enums (`PrintConfig.hpp:539-777`).
// --------------------------------------------------------------------------------------------

/// `NozzleTypeEumnToStr` (`PrintConfig.hpp:539-546`), vendored spelling of the name kept. The
/// vendored object is a file-scope `static std::unordered_map`, i.e. one copy per translation
/// unit that includes the header; a `OnceLock` behind an accessor is the closest Rust gets to
/// the same "initialised once, read many" shape. It carries six entries where
/// `s_keys_map_NozzleType` carries five: `ntE3D` has a string here and no config key there.
pub fn nozzle_type_eumn_to_str(
) -> &'static std::collections::HashMap<slic3r_commondefs::common_defs::NozzleType, String> {
    use slic3r_commondefs::common_defs::NozzleType;
    static MAP: std::sync::OnceLock<std::collections::HashMap<NozzleType, String>> =
        std::sync::OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = std::collections::HashMap::new();
        m.insert(NozzleType::ntUndefine, "undefine".to_string());
        m.insert(NozzleType::ntHardenedSteel, "hardened_steel".to_string());
        m.insert(NozzleType::ntStainlessSteel, "stainless_steel".to_string());
        m.insert(
            NozzleType::ntTungstenCarbide,
            "tungsten_carbide".to_string(),
        );
        m.insert(NozzleType::ntBrass, "brass".to_string());
        m.insert(NozzleType::ntE3D, "E3D".to_string());
        m
    })
}

/// `NozzleTypeStrToEumn` (`PrintConfig.hpp:548-555`), vendored spelling of the name kept: the
/// same six pairs the other way round, transcribed from its own site rather than inverted from
/// the map above, because the vendored source writes both out and a later edit to one need not
/// be mirrored in the other.
pub fn nozzle_type_str_to_eumn(
) -> &'static std::collections::HashMap<String, slic3r_commondefs::common_defs::NozzleType> {
    use slic3r_commondefs::common_defs::NozzleType;
    static MAP: std::sync::OnceLock<std::collections::HashMap<String, NozzleType>> =
        std::sync::OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = std::collections::HashMap::new();
        m.insert("undefine".to_string(), NozzleType::ntUndefine);
        m.insert("hardened_steel".to_string(), NozzleType::ntHardenedSteel);
        m.insert("stainless_steel".to_string(), NozzleType::ntStainlessSteel);
        m.insert(
            "tungsten_carbide".to_string(),
            NozzleType::ntTungstenCarbide,
        );
        m.insert("brass".to_string(), NozzleType::ntBrass);
        m.insert("E3D".to_string(), NozzleType::ntE3D);
        m
    })
}

/// `get_valid_nozzle_volume_type` (`PrintConfig.hpp:680-692`). Walks `0..=nvtMaxNozzleVolumeType`
/// and drops `nvtHybrid` (not a physical variant) and the reserved gap between
/// `nvtTPUHighFlow` and `nvtE3DHighFlow`, both for the reasons the vendored comments give.
///
/// FIDELITY(port): the vendored return type is a `std::set<NozzleVolumeType>`, which iterates in
/// `operator<` order; `BTreeSet` iterates in `Ord` order, and `NozzleVolumeType`'s derived `Ord`
/// follows its discriminants, so the two orders agree. The order reaches output through
/// `get_extruder_variant_string`'s callers.
pub fn get_valid_nozzle_volume_type() -> std::collections::BTreeSet<NozzleVolumeType> {
    let mut type_set = std::collections::BTreeSet::new();
    let mut i: i32 = 0;
    while i <= (NozzleVolumeType::NVT_MAX_NOZZLE_VOLUME_TYPE as i32) {
        let t = <NozzleVolumeType as slic3r_config::config::ConfigOptionEnumValue>::from_i32(i);
        // The vendored loop casts `i` straight to the enum, so the reserved value 4 reaches the
        // two `continue`s below as an out-of-range enum value. `from_i32` pins an unmatched
        // value to the default (`nvtStandard`), so the `i`-based tests below -- not `t`-based,
        // exactly as the vendored source writes them -- are what keep the gap skipped.
        if t == NozzleVolumeType::nvtHybrid && i == (NozzleVolumeType::nvtHybrid as i32) {
            i += 1;
            continue;
        }
        if i > (NozzleVolumeType::nvtTPUHighFlow as i32)
            && i < (NozzleVolumeType::nvtE3DHighFlow as i32)
        {
            i += 1;
            continue;
        }
        type_set.insert(t);
        i += 1;
    }
    type_set
}

/// `bed_type_to_gcode_string` (`PrintConfig.hpp:702-731`). The `default:` arm covers
/// `btDefault` and `btCount` alike, which is why it returns `"unknown"` rather than a per-value
/// string.
pub fn bed_type_to_gcode_string(bed_type: BedType) -> String {
    match bed_type {
        BedType::btSuperTack => "supertack_plate".to_string(),
        BedType::btPC => "cool_plate".to_string(),
        BedType::btPCT => "textured_cool_plate".to_string(),
        BedType::btEP => "eng_plate".to_string(),
        BedType::btPEI => "hot_plate".to_string(),
        BedType::btPTE => "textured_plate".to_string(),
        _ => "unknown".to_string(),
    }
}

/// `get_bed_temp_key` (`PrintConfig.hpp:733-754`). A chain of `if`s at the pin, not a `switch`,
/// falling through to the empty string.
pub fn get_bed_temp_key(bed_type: BedType) -> String {
    if bed_type == BedType::btSuperTack {
        return "supertack_plate_temp".to_string();
    }
    if bed_type == BedType::btPC {
        return "cool_plate_temp".to_string();
    }
    if bed_type == BedType::btPCT {
        return "textured_cool_plate_temp".to_string();
    }
    if bed_type == BedType::btEP {
        return "eng_plate_temp".to_string();
    }
    if bed_type == BedType::btPEI {
        return "hot_plate_temp".to_string();
    }
    if bed_type == BedType::btPTE {
        return "textured_plate_temp".to_string();
    }
    String::new()
}

/// `get_bed_temp_1st_layer_key` (`PrintConfig.hpp:756-777`).
pub fn get_bed_temp_1st_layer_key(bed_type: BedType) -> String {
    if bed_type == BedType::btSuperTack {
        return "supertack_plate_temp_initial_layer".to_string();
    }
    if bed_type == BedType::btPC {
        return "cool_plate_temp_initial_layer".to_string();
    }
    if bed_type == BedType::btPCT {
        return "textured_cool_plate_temp_initial_layer".to_string();
    }
    if bed_type == BedType::btEP {
        return "eng_plate_temp_initial_layer".to_string();
    }
    if bed_type == BedType::btPEI {
        return "hot_plate_temp_initial_layer".to_string();
    }
    if bed_type == BedType::btPTE {
        return "textured_plate_temp_initial_layer".to_string();
    }
    String::new()
}

// --------------------------------------------------------------------------------------------
// Step 5: `PrintConfigDef` and the option registry (`PrintConfig.cpp:991-9747`).
//
// **Order of work, corrected against the plan by building it.** The plan puts `StaticCache`
// (step 3) and the 11 derived classes (step 4) before the option table (step 5). Building it
// that way does not work: `StaticPrintConfig::def()` (`PrintConfig.hpp:1047`) returns
// `&print_config_def`, and `StaticCache::finalize` (`:1105-1129`) reads `defs->options` and
// each option's `default_value`, so steps 3 and 4 cannot compile -- let alone be checked --
// before the registry they are defined against exists. The table is therefore built first and
// the reorder is recorded here and in this task's pull request body, as the plan's own
// "each step names the slice a reviewer can accept or reject on its own" allows.
// --------------------------------------------------------------------------------------------

use slic3r_config::config::ConfigDef;
use slic3r_config::config::ConfigOptionBool;
use slic3r_config::config::ConfigOptionBools;
use slic3r_config::config::ConfigOptionBoolsNullable;
use slic3r_config::config::ConfigOptionEnum;
use slic3r_config::config::ConfigOptionEnumGeneric;
use slic3r_config::config::ConfigOptionEnumsGeneric;
use slic3r_config::config::ConfigOptionEnumsGenericNullable;
use slic3r_config::config::ConfigOptionFloat;
use slic3r_config::config::ConfigOptionFloatOrPercent;
use slic3r_config::config::ConfigOptionFloats;
use slic3r_config::config::ConfigOptionFloatsNullable;
use slic3r_config::config::ConfigOptionFloatsOrPercentsNullable;
use slic3r_config::config::ConfigOptionInt;
use slic3r_config::config::ConfigOptionInts;
use slic3r_config::config::ConfigOptionIntsNullable;
use slic3r_config::config::ConfigOptionPercent;
use slic3r_config::config::ConfigOptionPercents;
use slic3r_config::config::ConfigOptionPercentsNullable;
use slic3r_config::config::ConfigOptionPoint;
use slic3r_config::config::ConfigOptionPoints;
use slic3r_config::config::ConfigOptionPointsGroups;
use slic3r_config::config::ConfigOptionString;
use slic3r_config::config::ConfigOptionStrings;
use slic3r_config::config::ConfigOptionType;
use slic3r_config::config::GuiType;
use slic3r_config::config::PrinterTechnology;
use slic3r_config::config::TOptiondefMap;
use slic3r_config::config::NOCLI;
use slic3r_point::point::Vec2d;

/// `assign_printer_technology_to_unknown` (`PrintConfig.cpp:991-996`), a file-scope `static`
/// free function with internal linkage -- the second item of this unit a literal proof path 3
/// would have to `#include` the vendored `.cpp` to reach.
/// `const std::vector<std::string> filament_extruder_override_keys` (`PrintConfig.cpp:95-121`),
/// declared `extern` at `PrintConfig.hpp:779`. The filament-side override keys whose rows
/// `init_fff_params` generates from the matching printer-side extruder option.
///
/// FIDELITY(port): the vendored `std::vector`'s own order, not sorted. It reaches output: the
/// generating loop at `PrintConfig.cpp:8783` calls `ConfigDef::add_nullable` once per entry in
/// this order, and each call assigns the next `serialization_key_ordinal`, so reordering this
/// list would reorder `by_serialization_key_ordinal`. `PlaceholderParser.cpp:1315` also reads it
/// as a membership test, which the order does not affect.
pub fn filament_extruder_override_keys() -> &'static Vec<String> {
    static KEYS: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    KEYS.get_or_init(|| {
        [
            // floats
            "filament_retraction_length",
            "filament_z_hop",
            "filament_z_hop_types",
            // not in filament_options_with_variant, not used?
            "filament_retract_lift_above",
            // not in filament_options_with_variant, not used?
            "filament_retract_lift_below",
            "filament_retract_lift_enforce",
            "filament_retraction_speed",
            "filament_deretraction_speed",
            // not in filament_options_with_variant, added on 20250816
            "filament_retract_restart_extra",
            "filament_retraction_minimum_travel",
            "filament_retract_length_toolchange",
            "filament_retract_restart_extra_toolchange",
            // BBS: floats
            "filament_wipe_distance",
            // bools
            "filament_retract_when_changing_layer",
            "filament_wipe",
            // percents
            "filament_retract_before_wipe",
            // Orca
            "filament_retract_after_wipe",
            // BBS
            "filament_long_retractions_when_cut",
            "filament_retraction_distances_when_cut",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    })
}

/// `is_filament_extruder_override_key` (`PrintConfig.cpp:128-139`): membership in
/// [`filament_extruder_override_keys`], widened by the five keys defined separately from the
/// generator list. A nil in one of these keys means the override is disabled, so a child preset
/// stores and loads it verbatim instead of treating nil as "inherit".
pub fn is_filament_extruder_override_key(opt_key: &str) -> bool {
    filament_extruder_override_keys()
        .iter()
        .any(|k| k == opt_key)
        || opt_key == "filament_retract_length_nc"
        || opt_key == "filament_ironing_flow"
        || opt_key == "filament_ironing_spacing"
        || opt_key == "filament_ironing_inset"
        || opt_key == "filament_ironing_speed"
}

/// The `switch (def->type)` of the filament-override loop (`PrintConfig.cpp:8806-8812`): the
/// nullable sibling of the source option's type, carrying the source's own value vector.
///
/// The vendored arms `static_cast` the source `ConfigOption*` to its concrete non-nullable type
/// and hand the `values` member to the nullable type's constructor. The `static_cast` becomes a
/// checked `downcast_ref`, because `slic3r-config` gives one `ConfigOptionType` tag to both
/// instantiations of each `…Templ<NULLABLE>` and so cannot distinguish them by tag alone.
///
/// PORT NOTE: the vendored `default: assert(false)` (`:8811`) is `<cassert>`'s, compiled out by
/// `NDEBUG` in the oracle's Release build, where the arm then falls through leaving
/// `default_value` unset. `None` reproduces that, and the caller sets no default for it; a
/// `panic!` here would diverge from the oracle on a type this list never carries.
fn override_default(
    option_type: ConfigOptionType,
    src: Option<&dyn slic3r_config::config::ConfigOption>,
) -> Option<Box<dyn slic3r_config::config::ConfigOption>> {
    let src = src?;
    match option_type {
        ConfigOptionType::Floats => {
            let o = src.as_any().downcast_ref::<ConfigOptionFloats>()?;
            Some(Box::new(ConfigOptionFloatsNullable {
                values: o.values.clone(),
            }))
        }
        ConfigOptionType::Percents => {
            let o = src.as_any().downcast_ref::<ConfigOptionPercents>()?;
            Some(Box::new(ConfigOptionPercentsNullable {
                values: o.values.clone(),
            }))
        }
        ConfigOptionType::Bools => {
            let o = src.as_any().downcast_ref::<ConfigOptionBools>()?;
            Some(Box::new(ConfigOptionBoolsNullable {
                values: o.values.clone(),
            }))
        }
        ConfigOptionType::Enums => {
            let o = src.as_any().downcast_ref::<ConfigOptionEnumsGeneric>()?;
            let mut out = ConfigOptionEnumsGenericNullable::new(o.keys_map);
            out.values = o.values.clone();
            Some(Box::new(out))
        }
        _ => None,
    }
}

fn assign_printer_technology_to_unknown(
    options: &mut TOptiondefMap,
    printer_technology: PrinterTechnology,
) {
    for (_, def) in options.iter_mut() {
        if def.printer_technology == PrinterTechnology::Unknown {
            def.printer_technology = printer_technology;
        }
    }
}

/// `class PrintConfigDef : public ConfigDef` (`PrintConfig.hpp:843-876`). Rust has no
/// inheritance, so the base is a field; every `this->add(...)` in the vendored `init_*` bodies
/// becomes `self.def.add(...)` and every `this->options` becomes `self.def.options`.
pub struct PrintConfigDef {
    /// The `ConfigDef` base.
    pub def: ConfigDef<'static>,
    /// `m_extruder_option_keys` (`PrintConfig.hpp:868`).
    m_extruder_option_keys: Vec<String>,
    /// `m_extruder_retract_keys` (`PrintConfig.hpp:869`).
    m_extruder_retract_keys: Vec<String>,
    /// `m_filament_option_keys` (`PrintConfig.hpp:874`).
    m_filament_option_keys: Vec<String>,
    /// `m_filament_retract_keys` (`PrintConfig.hpp:875`).
    m_filament_retract_keys: Vec<String>,
}

impl PrintConfigDef {
    /// `PrintConfigDef::PrintConfigDef()` (`PrintConfig.cpp:998-1007`).
    pub fn new() -> Self {
        let mut this = PrintConfigDef {
            def: ConfigDef::new(),
            m_extruder_option_keys: Vec::new(),
            m_extruder_retract_keys: Vec::new(),
            m_filament_option_keys: Vec::new(),
            m_filament_retract_keys: Vec::new(),
        };
        this.init_common_params();
        assign_printer_technology_to_unknown(&mut this.def.options, PrinterTechnology::Any);
        this.init_fff_params();
        this.init_extruder_option_keys();
        assign_printer_technology_to_unknown(&mut this.def.options, PrinterTechnology::Fff);
        this.init_sla_params();
        assign_printer_technology_to_unknown(&mut this.def.options, PrinterTechnology::Sla);
        this
    }

    /// `extruder_option_keys()` (`PrintConfig.hpp:852`).
    pub fn extruder_option_keys(&self) -> &Vec<String> {
        &self.m_extruder_option_keys
    }
    /// `extruder_retract_keys()` (`PrintConfig.hpp:856`).
    pub fn extruder_retract_keys(&self) -> &Vec<String> {
        &self.m_extruder_retract_keys
    }
    /// `filament_option_keys()` (`PrintConfig.hpp:859`).
    pub fn filament_option_keys(&self) -> &Vec<String> {
        &self.m_filament_option_keys
    }
    /// `filament_retract_keys()` (`PrintConfig.hpp:860`).
    pub fn filament_retract_keys(&self) -> &Vec<String> {
        &self.m_filament_retract_keys
    }
}

impl Default for PrintConfigDef {
    fn default() -> Self {
        PrintConfigDef::new()
    }
}

/// `extern const PrintConfigDef print_config_def` (`PrintConfig.hpp:880`), the externally linked
/// global this unit's proof shim reads.
///
/// **PORT NOTE: one registry per thread, not one per process, and the reason is not a choice
/// this module made.** The vendored object is a `const` global constructed before `main` and
/// shared by every thread. `ConfigOptionDef::default_value` is a
/// `Option<Box<dyn ConfigOption>>` (`slic3r-config`, task `59-foundation-config`), and
/// `ConfigOption` is declared there with no `Send`/`Sync` bound, so a `PrintConfigDef` cannot be
/// placed in a `static` at all -- `OnceLock<PrintConfigDef>` is rejected for exactly that
/// reason. Widening that trait is `slic3r-config`'s module to change, not this one's
/// (`task.md`: "Write only your own modules"), so this function leaks one registry per thread
/// instead and hands out a `&'static` into it. The registry is read-only after construction and
/// every copy is built by the same code from the same source, so the content a caller observes
/// is identical; what differs is storage, plus `ConfigOptionDef::serialization_key_ordinal`,
/// which `slic3r-config` draws from a process-wide atomic counter and which therefore depends on
/// how many registries have been built. Neither reaches this unit's proof, which compares
/// `option_type`, `nullable`, `cli`, the serialized default, `enum_values` and `enum_keys_map`.
/// Named here and in this task's pull request body as a finding against `slic3r-config`'s trait
/// declaration rather than papered over.
pub fn print_config_def() -> &'static PrintConfigDef {
    thread_local! {
        static DEF: &'static PrintConfigDef = Box::leak(Box::new(PrintConfigDef::new()));
    }
    DEF.with(|d| *d)
}

impl PrintConfigDef {
    /// `PrintConfigDef::init_common_params()` (`PrintConfig.cpp:1009-1316`).
    fn init_common_params(&mut self) {
        {
            let def = self.def.add("printer_technology", ConfigOptionType::Enum);
            def.label = "Printer technology".to_string();
            def.enum_keys_map = keys_map_printer_technology();
            def.enum_values.push("FFF".to_string());
            def.enum_values.push("SLA".to_string());
            // PORT NOTE: the vendored default is
            // `new ConfigOptionEnum<PrinterTechnology>(ptFFF)` (`PrintConfig.cpp:1019`).
            // `PrinterTechnology` and `ConfigOptionEnumValue` both live in `slic3r-config`, so
            // Rust's orphan rule forbids this crate implementing the trait for that type (see
            // the note after the key maps above). `ConfigOptionEnumGeneric` over the same key
            // map is used instead: its `serialize()` returns the name whose value matches, i.e.
            // the same `"FFF"` the oracle's `ConfigOptionEnum<PrinterTechnology>` serializes.
            def.set_default_value(Box::new(ConfigOptionEnumGeneric::with_value(
                keys_map_printer_technology(),
                PrinterTechnology::Fff as i32,
            )));
        }
        {
            let def = self.def.add("printable_area", ConfigOptionType::Points);
            def.label = "Printable area".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionPoints::new();
            v.values.push(Vec2d::new(0_f64, 0_f64));
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1025
            v.values.push(Vec2d::new(200_f64, 0_f64));
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1025
            v.values.push(Vec2d::new(200_f64, 200_f64));
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1025
            v.values.push(Vec2d::new(0_f64, 200_f64));
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("extruder_printable_area", ConfigOptionType::PointsGroups);
            def.label = "Extruder printable area".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.gui_type = GuiType::OneString;
            def.set_default_value(Box::new(ConfigOptionPointsGroups { values: Vec::new() }));
        }
        {
            let def = self
                .def
                .add("support_parallel_printheads", ConfigOptionType::Bool);
            def.label = "Support parallel printheads".to_string();
            def.tooltip =
                "Enable printer settings for machines that can use multiple printheads in parallel."
                    .to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("parallel_printheads_count", ConfigOptionType::Int);
            def.label = "Parallel printheads count".to_string();
            def.tooltip =
                "Set the number of parallel printheads for machines like OrangeStorm Giga printer."
                    .to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.min = 1_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1044
            def.max = 4_f32;
            def.set_default_value(Box::new(ConfigOptionInt::new(1)));
        }
        {
            let def = self.def.add(
                "parallel_printheads_bed_exclude_areas",
                ConfigOptionType::Strings,
            );
            def.label = "Parallel printheads bed exclude areas".to_string();
            def.tooltip = "Ordered list of bed exclude areas by parallel printhead count. Item 1 applies to one printhead, item 2 to two printheads, and so on. Leave an item empty for no excluded area.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionStrings::new()));
        }
        {
            let def = self.def.add("bed_exclude_area", ConfigOptionType::Points);
            def.label = "Excluded bed area".to_string();
            def.tooltip = concat!(
                "Unprintable area in XY plane. For example, X1 Series printers use the front left corner to cut filament during filament change. ",
                "The area is expressed as a polygon in the format \"XxY, XxY, ...\". It constrains material and model placement, ",
                "but allows machine travel. Configure toolhead collision volumes separately for G-code checking and travel avoidance."
            ).to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.gui_type = GuiType::OneString;
            let mut v = ConfigOptionPoints::new();
            v.values.push(Vec2d::new(0_f64, 0_f64));
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("bed_exclude_volumes", ConfigOptionType::String);
            def.label = "Toolhead collision volumes".to_string();
            def.tooltip = concat!(
                "Collision volumes used for model and generated-material exclusion, G-code checking, and travel avoidance. ",
                "Use \"XxY, XxY, ...\" for a full-height polygon or \"ZMIN..ZMAX;XxY, XxY, ...\" for a Z-limited volume. ",
                "Separate multiple volumes with \"|\". These volumes are additional to the excluded bed area. ",
                "Automatic arrangement conservatively uses the bounding box of a non-convex volume."
            ).to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.gui_type = GuiType::OneString;
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
        }
        {
            let def = self
                .def
                .add("bed_exclude_volume_mode", ConfigOptionType::Enum);
            def.label = "Collision volume behavior".to_string();
            def.tooltip = "Choose whether all nozzles use the same collision volumes, derive toolhead-relative volumes from each nozzle's XY offset, or configure independent volumes for every extruder. In individual mode, select each extruder on the Extruder page to configure its volumes.".to_string();
            def.enum_keys_map = keys_map_bed_exclude_volume_mode();
            def.enum_values = vec![
                "shared".to_string(),
                "toolhead_offset".to_string(),
                "per_extruder".to_string(),
            ];
            def.enum_labels = vec![
                "Shared by all nozzles".to_string(),
                "Toolhead-relative (nozzle offsets)".to_string(),
                "Individual per extruder".to_string(),
            ];
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionEnumGeneric::with_value(
                keys_map_bed_exclude_volume_mode(),
                BedExcludeVolumeMode::Shared as i32,
            )));
        }
        {
            let def = self
                .def
                .add("extruder_bed_exclude_volumes", ConfigOptionType::Strings);
            def.label = "Extruder collision volumes".to_string();
            def.tooltip = "Collision volumes used only by this extruder in individual mode. Use the same polygon and optional Z-range format as the shared collision volumes. Leave empty for no collision volumes on this extruder. Automatic arrangement conservatively uses the bounding box of a non-convex volume.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.gui_type = GuiType::OneString;
            let mut v = ConfigOptionStrings::new();
            v.values.push(String::new());
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("bed_custom_texture", ConfigOptionType::String);
            def.label = "Bed custom texture".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.gui_type = GuiType::OneString;
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
        }
        {
            let def = self.def.add("bed_custom_model", ConfigOptionType::String);
            def.label = "Bed custom model".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.gui_type = GuiType::OneString;
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
        }
        {
            let def = self
                .def
                .add("elefant_foot_compensation", ConfigOptionType::Float);
            def.label = "Elephant foot compensation".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "This shrinks the first layer on the build plate to compensate for elephant foot effect.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1109
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.)));
        }
        {
            let def = self
                .def
                .add("elefant_foot_compensation_layers", ConfigOptionType::Int);
            def.label = "Elephant foot compensation layers".to_string();
            def.category = "Quality".to_string();
            def.tooltip = concat!(
                "The number of layers on which the elephant foot compensation will be active. ",
                "The first layer will be shrunk by the elephant foot compensation value, then ",
                "the next layers will be linearly shrunk less, up to the layer indicated by this value."
            ).to_string();
            def.sidetext = "layers".to_string();
            def.min = 1_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionInt::new(1)));
        }
        {
            let def = self
                .def
                .add("elefant_foot_layers_density", ConfigOptionType::Percent);
            def.label = "Elephant foot layers density".to_string();
            def.category = "Quality".to_string();
            def.tooltip = concat!(
                "Density of internal solid infill for Elephant foot layers compensation.\n",
                "The initial value for the second layer is set.\n",
                "Subsequent layers become linearly denser by the height specified in elefant_foot_compensation_layers."
            ).to_string();
            def.sidetext = "%".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1129
            def.min = 50_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1130
            def.max = 100_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Expert;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1132
            def.set_default_value(Box::new(ConfigOptionPercent::new(100_f64)));
        }
        {
            let def = self.def.add("layer_height", ConfigOptionType::Float);
            def.label = "Layer height".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "This is the height for each layer. Smaller layer heights give greater accuracy but longer printing time.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.set_default_value(Box::new(ConfigOptionFloat::new(
                slic3r_printconfigconstants::print_config_constants::INITIAL_LAYER_HEIGHT,
            )));
        }
        {
            let def = self.def.add("printable_height", ConfigOptionType::Float);
            def.label = "Printable height".to_string();
            def.tooltip =
                "This is the maximum printable height which is limited by the height of the build area."
                    .to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1147
            def.max = 214700_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1149
            def.set_default_value(Box::new(ConfigOptionFloat::new(100.0)));
        }
        {
            let def = self
                .def
                .add("extruder_printable_height", ConfigOptionType::Floats);
            def.label = "Extruder printable height".to_string();
            def.tooltip =
                "Maximum printable height of this extruder which is limited by mechanism of printer."
                    .to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1156
            def.max = 1000_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT NOTE: the vendored default is a `ConfigOptionFloatsNullable` although the
            // option itself is registered with plain `coFloats` and not `add_nullable`
            // (`PrintConfig.cpp:1151,1158`), so the oracle's `def->nullable` is false while the
            // default value object is the nullable template. Transcribed exactly as written.
            let mut v = ConfigOptionFloatsNullable::new();
            v.values.push(0_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("preferred_orientation", ConfigOptionType::Float);
            def.label = "Preferred orientation".to_string();
            def.tooltip =
                "Automatically orient STL files on the Z axis upon initial import.".to_string();
            def.sidetext = "\u{b0}".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1164
            def.max = 360_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1165
            def.min = -360_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1167
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.0)));
        }
        {
            // The first of two `add`s of this key: the second, at `PrintConfig.cpp:1276-1280`,
            // keeps every field this one set and overwrites only `opt_key`, `type` and
            // `serialization_key_ordinal` (`ConfigDef::add`, `Config.cpp:363-372`), so the
            // surviving `mode` is the second one's `comAdvanced` and the surviving tooltip is
            // the second one's.
            let def = self.def.add("preset_names", ConfigOptionType::Strings);
            def.label = "Printer preset names".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            def.set_default_value(Box::new(ConfigOptionStrings::new()));
        }
        {
            let def = self.def.add("bbl_use_printhost", ConfigOptionType::Bool);
            def.label = "Use 3rd-party print host".to_string();
            def.tooltip =
                "Allow controlling BambuLab's printer through 3rd party print hosts.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.cli = NOCLI.to_string();
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("use_3mf", ConfigOptionType::Bool);
            def.label = "Use 3MF instead of G-code".to_string();
            def.tooltip = concat!(
                "Enable this if the printer accepts a 3MF file as the print job. When enabled, Orca Slicer ",
                "sends the sliced file as a .gcode.3mf, instead of a plain .gcode file."
            ).to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.cli = NOCLI.to_string();
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("printer_agent", ConfigOptionType::String);
            def.label = "Printer Agent".to_string();
            def.tooltip =
                "Select the network agent implementation for printer communication.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.cli = NOCLI.to_string();
            def.plugin_type = "printer-connection".to_string();
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
        }
        {
            let def = self.def.add("print_host", ConfigOptionType::String);
            def.label = "Hostname, IP or URL".to_string();
            def.tooltip = concat!(
                "Orca Slicer can upload G-code files to a printer host. This field should contain ",
                "the hostname, IP address or URL of the printer host instance. ",
                "Print host behind HAProxy with basic auth enabled can be accessed by putting the user name and password into the URL ",
                "in the following format: https://username:password@your-octopi-address/"
            ).to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.cli = NOCLI.to_string();
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
        }
        {
            let def = self.def.add("print_host_webui", ConfigOptionType::String);
            def.label = "Device UI".to_string();
            def.tooltip =
                "Specify the URL of your device user interface if it's not same as print_host."
                    .to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.cli = NOCLI.to_string();
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
        }
        {
            let def = self.def.add("printhost_apikey", ConfigOptionType::String);
            def.label = "API Key / Password".to_string();
            def.tooltip = concat!(
                "Orca Slicer can upload G-code files to a printer host. This field should contain ",
                "the API Key or the password required for authentication."
            )
            .to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.cli = NOCLI.to_string();
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
        }
        {
            let def = self
                .def
                .add("flashforge_serial_number", ConfigOptionType::String);
            def.label = "Serial Number".to_string();
            def.tooltip = "Flashforge local API requires the printer serial number.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.cli = NOCLI.to_string();
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
        }
        {
            let def = self.def.add("printhost_port", ConfigOptionType::String);
            def.label = "Printer".to_string();
            def.tooltip = "Name of the printer.".to_string();
            def.gui_type = GuiType::SelectOpen;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.cli = NOCLI.to_string();
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
        }
        {
            let def = self.def.add("printhost_cafile", ConfigOptionType::String);
            def.label = "HTTPS CA File".to_string();
            def.tooltip = concat!(
                "Custom CA certificate file can be specified for HTTPS OctoPrint connections, in crt/pem format. ",
                "If left blank, the default OS CA certificate repository is used."
            ).to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.cli = NOCLI.to_string();
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
        }
        {
            let def = self.def.add("printhost_user", ConfigOptionType::String);
            def.label = "User".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.cli = NOCLI.to_string();
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
        }
        {
            let def = self.def.add("printhost_password", ConfigOptionType::String);
            def.label = "Password".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.cli = NOCLI.to_string();
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
        }
        {
            let def = self
                .def
                .add("printhost_ssl_ignore_revoke", ConfigOptionType::Bool);
            def.label = "Ignore HTTPS certificate revocation checks".to_string();
            def.tooltip = "Ignore HTTPS certificate revocation checks in the case of missing or offline distribution points. One may want to enable this option for self signed certificates if connection fails.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.cli = NOCLI.to_string();
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            // The second `add` of `preset_names` (`PrintConfig.cpp:1276-1280`); see the first,
            // above. It overwrites the tooltip and the mode and keeps the label and default.
            let def = self.def.add("preset_names", ConfigOptionType::Strings);
            def.label = "Printer preset names".to_string();
            def.tooltip = "Names of presets related to the physical printer.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionStrings::new()));
        }
        {
            let def = self
                .def
                .add("printhost_authorization_type", ConfigOptionType::Enum);
            def.label = "Authorization Type".to_string();
            def.enum_keys_map = keys_map_authorization_type();
            def.enum_values.push("key".to_string());
            def.enum_values.push("user".to_string());
            def.enum_labels.push("API key".to_string());
            def.enum_labels.push("HTTP digest".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.cli = NOCLI.to_string();
            def.set_default_value(Box::new(ConfigOptionEnumGeneric::with_value(
                keys_map_authorization_type(),
                AuthorizationType::atKeyPassword as i32,
            )));
        }
        {
            // "temporary workaround for compatibility with older Slicer"
            // (`PrintConfig.cpp:1294-1298`): registered with no label, tooltip or mode.
            let def = self.def.add("preset_name", ConfigOptionType::String);
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
        }
        {
            // `PrintConfig.cpp:1304-1314`: three keys generated by a loop over a brace-init list
            // of string literals, one per preset type, all sharing one body.
            for key in [
                "print_plugin_config_overrides",
                "printer_plugin_config_overrides",
                "filament_plugin_config_overrides",
            ] {
                let def = self.def.add(key, ConfigOptionType::String);
                def.label = "Capabilities".to_string();
                def.tooltip = concat!(
                    "Configuration for the plugin capabilities this preset uses, overriding the global ",
                    "Capabilities configuration. Stored as a raw JSON array and edited through the dialog ",
                    "behind the button, never typed in directly."
                ).to_string();
                def.gui_type = GuiType::PluginConfig;
                def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
                def.cli = NOCLI.to_string();
                def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
            }
        }
    }

    /// `PrintConfigDef::init_fff_params()` (`PrintConfig.cpp:1317-8991`).
    fn init_fff_params(&mut self) {
        // `const int max_temp = 1500` (`PrintConfig.cpp:1322`), a local of the vendored body
        // used as the `max` (and, at `:6960`, the negated `min`) of every nozzle-temperature
        // option. `def.min`/`def.max` are `f32`, so it is declared at that width rather than as
        // the vendored `int`: every use is one of those two assignments, and neither takes its
        // integer-ness.
        // PORT SRC: src/libslic3r/PrintConfig.cpp:1322
        let max_temp: f32 = 1500_f32;

        {
            let def = self.def.add("reduce_crossing_wall", ConfigOptionType::Bool);
            def.label = "Avoid crossing walls".to_string();
            def.category = "Quality".to_string();
            def.tooltip =
                "This detours to avoid traveling across walls, which may cause blobs on the surface."
                    .to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(
                slic3r_printconfigconstants::print_config_constants::INITIAL_REDUCE_CROSSING_WALL,
            )));
        }
        {
            let def = self.def.add(
                "max_travel_detour_distance",
                ConfigOptionType::FloatOrPercent,
            );
            def.label = "Avoid crossing walls - Max detour length".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Maximum detour distance for avoiding crossing wall: The printer won't detour if the detour distance is larger than this value. Detour length could be specified either as an absolute value or as percentage (for example 50%) of a direct travel path. A value of 0 will disable this.".to_string();
            def.sidetext = "mm or %".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1337
            def.max_literal = 1000_f64;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1339
            def.set_default_value(Box::new(ConfigOptionFloatOrPercent::new(0., false)));
        }
        // The seven "other layers" bed-temperature options (`PrintConfig.cpp:1342-1447`): one
        // pair per plate type, each pair an "other layers" and a "first layer" option. The
        // sidetext is the degree-Celsius sign written as a UTF-8 escape in the vendored source.
        {
            let def = self.def.add("supertack_plate_temp", ConfigOptionType::Ints);
            def.label = "Other layers".to_string();
            def.tooltip = "This is the bed temperature for layers except for the first one. A value of 0 means the filament does not support printing on the Cool Plate SuperTack.".to_string();
            def.sidetext = "\u{2103}".to_string();
            def.full_label = "Bed temperature".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1348
            def.max = 120_f32;
            let mut v = ConfigOptionInts::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1349
            v.values.push(35);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("cool_plate_temp", ConfigOptionType::Ints);
            def.label = "Other layers".to_string();
            def.tooltip = "This is the bed temperature for layers except for the first one. A value of 0 means the filament does not support printing on the Cool Plate.".to_string();
            def.sidetext = "\u{2103}".to_string();
            def.full_label = "Bed temperature".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1357
            def.max = 300_f32;
            let mut v = ConfigOptionInts::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1358
            v.values.push(35);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("textured_cool_plate_temp", ConfigOptionType::Ints);
            def.label = "Other layers".to_string();
            def.tooltip = "This is the bed temperature for layers except for the first one. A value of 0 means the filament does not support printing on the Textured Cool Plate.".to_string();
            def.sidetext = "\u{2103}".to_string();
            def.full_label = "Bed temperature".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1366
            def.max = 300_f32;
            let mut v = ConfigOptionInts::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1367
            v.values.push(40);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("eng_plate_temp", ConfigOptionType::Ints);
            def.label = "Other layers".to_string();
            def.tooltip = "This is the bed temperature for layers except for the first one. A value of 0 means the filament does not support printing on the Engineering Plate.".to_string();
            def.sidetext = "\u{2103}".to_string();
            def.full_label = "Bed temperature".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1375
            def.max = 300_f32;
            let mut v = ConfigOptionInts::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1376
            v.values.push(45);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("hot_plate_temp", ConfigOptionType::Ints);
            def.label = "Other layers".to_string();
            def.tooltip = "This is the bed temperature for layers except for the first one. A value of 0 means the filament does not support printing on the High Temp Plate.".to_string();
            def.sidetext = "\u{2103}".to_string();
            def.full_label = "Bed temperature".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1384
            def.max = 300_f32;
            let mut v = ConfigOptionInts::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1385
            v.values.push(45);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("textured_plate_temp", ConfigOptionType::Ints);
            def.label = "Other layers".to_string();
            def.tooltip = "This is the bed temperature for layers except for the first one. A value of 0 means the filament does not support printing on the Textured PEI Plate.".to_string();
            def.sidetext = "\u{2103}".to_string();
            def.full_label = "Bed temperature".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1393
            def.max = 300_f32;
            let mut v = ConfigOptionInts::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1394
            v.values.push(45);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("supertack_plate_temp_initial_layer", ConfigOptionType::Ints);
            def.label = "First layer".to_string();
            def.full_label = "First layer bed temperature".to_string();
            def.tooltip = "This is the bed temperature of the first layer. A value of 0 means the filament does not support printing on the Cool Plate SuperTack.".to_string();
            def.sidetext = "\u{2103}".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1402
            def.max = 120_f32;
            let mut v = ConfigOptionInts::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1403
            v.values.push(35);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("cool_plate_temp_initial_layer", ConfigOptionType::Ints);
            def.label = "First layer".to_string();
            def.full_label = "First layer bed temperature".to_string();
            def.tooltip = "This is the bed temperature of the first layer. A value of 0 means the filament does not support printing on the Cool Plate.".to_string();
            def.sidetext = "\u{2103}".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1411
            def.max = 120_f32;
            let mut v = ConfigOptionInts::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1412
            v.values.push(35);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "textured_cool_plate_temp_initial_layer",
                ConfigOptionType::Ints,
            );
            def.label = "First layer".to_string();
            def.full_label = "First layer bed temperature".to_string();
            def.tooltip = "This is the bed temperature of the first layer. A value of 0 means the filament does not support printing on the Textured Cool Plate.".to_string();
            def.sidetext = "\u{2103}".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1420
            def.max = 120_f32;
            let mut v = ConfigOptionInts::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1421
            v.values.push(40);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("eng_plate_temp_initial_layer", ConfigOptionType::Ints);
            def.label = "First layer".to_string();
            def.full_label = "First layer bed temperature".to_string();
            def.tooltip = "This is the bed temperature of the first layer. A value of 0 means the filament does not support printing on the Engineering Plate.".to_string();
            def.sidetext = "\u{2103}".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1429
            def.max = 300_f32;
            let mut v = ConfigOptionInts::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1430
            v.values.push(45);
            def.set_default_value(Box::new(v));
        }
        {
            // `hot_plate_temp_initial_layer` is the one plate option with **no** `min` at the
            // pin (`PrintConfig.cpp:1432-1438`): every sibling sets `def->min = 0` and this one
            // does not, so it keeps `ConfigOptionDef`'s own default. Transcribed as written.
            let def = self
                .def
                .add("hot_plate_temp_initial_layer", ConfigOptionType::Ints);
            def.label = "First layer".to_string();
            def.full_label = "First layer bed temperature".to_string();
            def.tooltip = "This is the bed temperature of the first layer. A value of 0 means the filament does not support printing on the High Temp Plate.".to_string();
            def.sidetext = "\u{2103}".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1437
            def.max = 300_f32;
            let mut v = ConfigOptionInts::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1438
            v.values.push(45);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("textured_plate_temp_initial_layer", ConfigOptionType::Ints);
            def.label = "First layer".to_string();
            def.full_label = "First layer bed temperature".to_string();
            def.tooltip = "This is the bed temperature of the first layer. A value of 0 means the filament does not support printing on the Textured PEI Plate.".to_string();
            def.sidetext = "\u{2103}".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1446
            def.max = 300_f32;
            let mut v = ConfigOptionInts::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1447
            v.values.push(45);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("curr_bed_type", ConfigOptionType::Enum);
            def.label = "Bed type".to_string();
            def.tooltip = "Plate types supported by the printer".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            def.enum_keys_map = keys_map_bed_type();
            def.enum_values.push("Cool Plate".to_string());
            def.enum_values.push("Engineering Plate".to_string());
            def.enum_values.push("High Temp Plate".to_string());
            def.enum_values.push("Textured PEI Plate".to_string());
            def.enum_values.push("Textured Cool Plate".to_string());
            def.enum_values.push("Supertack Plate".to_string());
            def.enum_labels.push("Smooth Cool Plate".to_string());
            def.enum_labels.push("Engineering Plate".to_string());
            def.enum_labels.push("Smooth High Temp Plate".to_string());
            def.enum_labels.push("Textured PEI Plate".to_string());
            def.enum_labels.push("Textured Cool Plate".to_string());
            def.enum_labels.push("Cool Plate (SuperTack)".to_string());
            def.set_default_value(Box::new(ConfigOptionEnum::new(BedType::btPC)));
        }
        {
            let def = self.def.add("default_bed_type", ConfigOptionType::String);
            def.label = "Default bed type".to_string();
            def.tooltip =
                "Default bed type for the printer (supports both numeric and string format)."
                    .to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
        }
        {
            let def = self
                .def
                .add("first_layer_print_sequence", ConfigOptionType::Ints);
            def.label = "First layer print sequence".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1481
            def.max = 16_f32;
            let mut v = ConfigOptionInts::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("other_layers_print_sequence", ConfigOptionType::Ints);
            def.label = "Other layers print sequence".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1487
            def.max = 16_f32;
            let mut v = ConfigOptionInts::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("other_layers_print_sequence_nums", ConfigOptionType::Int);
            def.label = "The number of other layers print sequence".to_string();
            def.set_default_value(Box::new(ConfigOptionInt::new(0)));
        }
        {
            let def = self
                .def
                .add("first_layer_sequence_choice", ConfigOptionType::Enum);
            def.category = "Quality".to_string();
            def.label = "First layer filament sequence".to_string();
            def.enum_keys_map = keys_map_layer_seq();
            def.enum_values.push("Auto".to_string());
            def.enum_values.push("Customize".to_string());
            def.enum_labels.push("Auto".to_string());
            def.enum_labels.push("Customize".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            def.set_default_value(Box::new(ConfigOptionEnum::new(LayerSeq::flsAuto)));
        }
        {
            let def = self
                .def
                .add("other_layers_sequence_choice", ConfigOptionType::Enum);
            def.category = "Quality".to_string();
            def.label = "Other layers filament sequence".to_string();
            def.enum_keys_map = keys_map_layer_seq();
            def.enum_values.push("Auto".to_string());
            def.enum_values.push("Customize".to_string());
            def.enum_labels.push("Auto".to_string());
            def.enum_labels.push("Customize".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            def.set_default_value(Box::new(ConfigOptionEnum::new(LayerSeq::flsAuto)));
        }
        {
            let def = self
                .def
                .add("before_layer_change_gcode", ConfigOptionType::String);
            def.label = "Before layer change G-code".to_string();
            def.tooltip =
                "This G-code is inserted at every layer change before the Z lift.".to_string();
            def.multiline = true;
            def.full_width = true;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1521
            def.height = 5;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
        }
        {
            let def = self.def.add("bottom_shell_layers", ConfigOptionType::Int);
            def.label = "Bottom shell layers".to_string();
            def.category = "Strength".to_string();
            def.sidetext = "layers".to_string();
            def.tooltip = concat!(
                "This is the number of solid layers of bottom shell, including the bottom ",
                "surface layer. When the thickness calculated by this value is thinner ",
                "than bottom shell thickness, the bottom shell layers will be increased."
            )
            .to_string();
            def.full_label = "Bottom shell layers".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1534
            def.set_default_value(Box::new(ConfigOptionInt::new(3)));
        }
        {
            let def = self
                .def
                .add("bottom_shell_thickness", ConfigOptionType::Float);
            def.label = "Bottom shell thickness".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "The number of bottom solid layers is increased when slicing if the thickness calculated by bottom shell layers is thinner than this value. This can avoid having too thin a shell when layer height is small. 0 means that this setting is disabled and the thickness of the bottom shell is determined simply by the number of bottom shell layers.".to_string();
            def.full_label = "Bottom shell thickness".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1543
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.)));
        }
        {
            let def = self
                .def
                .add("bottom_color_penetration_layers", ConfigOptionType::Int);
            def.label = "Bottom paint penetration layers".to_string();
            def.category = "Strength".to_string();
            def.sidetext = "layers".to_string();
            def.tooltip = concat!(
                "The number of layers, including the bottom surface layer, that take the color painted on a bottom surface. ",
                "Set to 0 to use the bottom shell layers. A value larger than the bottom shell also colors the sparse ",
                "infill above the painted area."
            ).to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionInt::new(0)));
        }
        {
            let def = self.def.add("gap_fill_target", ConfigOptionType::Enum);
            def.label = "Apply gap fill".to_string();
            def.category = "Strength".to_string();
            def.tooltip = concat!(
                "Enables gap fill for the selected solid surfaces. The minimum gap length that will be filled can be controlled ",
                "from the filter out tiny gaps option below.\n\n",
                "Options:\n",
                "1. Everywhere: Applies gap fill to top, bottom and internal solid surfaces for maximum strength\n",
                "2. Top and Bottom surfaces: Applies gap fill to top and bottom surfaces only, balancing print speed, ",
                "reducing potential over extrusion in the solid infill and making sure the top and bottom surfaces have ",
                "no pinhole gaps\n",
                "3. Nowhere: Disables gap fill for all solid infill areas\n\n",
                "Note that if using the classic perimeter generator, gap fill may also be generated between perimeters, ",
                "if a full width line cannot fit between them. That perimeter gap fill is not controlled by this setting.\n\n",
                "If you would like all gap fill, including the classic perimeter generated one, removed, ",
                "set the filter out tiny gaps value to a large number, like 999999.\n\n",
                "However this is not advised, as gap fill between perimeters is contributing to the model's strength. ",
                "For models where excessive gap fill is generated between perimeters, a better option would be to ",
                "switch to the arachne wall generator and use this option to control whether the cosmetic top and ",
                "bottom surface gap fill is generated."
            ).to_string();
            def.enum_keys_map = keys_map_gap_fill_target();
            def.enum_values.push("everywhere".to_string());
            def.enum_values.push("topbottom".to_string());
            def.enum_values.push("nowhere".to_string());
            def.enum_labels.push("Everywhere".to_string());
            def.enum_labels.push("Top and bottom surfaces".to_string());
            def.enum_labels.push("Nowhere".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionEnum::new(GapFillTarget::gftNowhere)));
        }
        {
            let def = self
                .def
                .add("enable_overhang_bridge_fan", ConfigOptionType::Bools);
            def.label = "Force cooling for overhangs and bridges".to_string();
            def.tooltip = concat!(
                "Enable this option to allow adjustment of the part cooling fan speed for specifically for overhangs, internal and external ",
                "bridges. Setting the fan speed specifically for these features can improve overall print quality and reduce warping."
            ).to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            let mut v = ConfigOptionBools::new();
            v.values.push(1);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("overhang_fan_speed", ConfigOptionType::Ints);
            def.label = "Overhangs and external bridges fan speed".to_string();
            def.tooltip = concat!(
                "Use this part cooling fan speed when printing bridges or overhang walls with an overhang threshold that exceeds ",
                "the value set in the 'Overhangs cooling threshold' parameter above. Increasing the cooling specifically for overhangs ",
                "and bridges can improve the overall print quality of these features.\n\n",
                "Please note, this fan speed is clamped on the lower end by the minimum fan speed threshold set above. It is also adjusted ",
                "upwards up to the maximum fan speed threshold when the minimum layer time threshold is not met."
            ).to_string();
            def.sidetext = "%".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1601
            def.max = 100_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionInts::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1603
            v.values.push(100);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("overhang_fan_threshold", ConfigOptionType::Enums);
            def.label = "Overhang cooling activation threshold".to_string();
            def.tooltip = concat!(
                "When the overhang exceeds this specified threshold, force the cooling fan to run at the 'Overhang Fan Speed' set below. ",
                "This threshold is expressed as a percentage, indicating the portion of each line's width that is unsupported by the layer ",
                "beneath it. Setting this value to 0% forces the cooling fan to run for all outer walls, regardless of the overhang degree."
            ).to_string();
            def.enum_keys_map = keys_map_overhang_fan_threshold();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            for s in ["0%", "10%", "25%", "50%", "75%", "95%"] {
                def.enum_values.push(s.to_string());
                def.enum_labels.push(s.to_string());
            }
            // PORT NOTE: the vendored default is
            // `new ConfigOptionEnumsGeneric{ (int)Overhang_threshold_bridge }`
            // (`PrintConfig.cpp:1626`), a brace-init that fills the inherited value vector and
            // leaves `keys_map` at its own default, so the oracle's `default_value->serialize()`
            // finds no name for the value and yields the empty string. `ConfigOptionEnumsGeneric::default()`
            // reproduces that exactly (its `keys_map` is `EMPTY_ENUM_VALUES`); giving it this
            // option's key map here would serialize `"95%"` and diverge from the oracle.
            let mut v = slic3r_config::config::ConfigOptionEnumsGeneric::default();
            v.values
                .push(OverhangFanThreshold::Overhang_threshold_bridge as i32);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("bridge_angle", ConfigOptionType::Float);
            def.label = "External bridge infill direction".to_string();
            def.category = "Strength".to_string();
            def.tooltip = concat!(
                "External Bridging angle override.\n",
                "If left to zero, the bridging angle will be calculated automatically for each specific bridge.\n",
                "Otherwise the provided angle will be used according to:\n",
                " - The absolute coordinates\n",
                " - The absolute coordinates + Model rotation: If Align directions to model is enabled\n",
                " - The optimal automatic angle + this value: If 'Relative Bridge Angle' is enabled\n\n",
                "Use 180\u{b0} for zero absolute angle."
            ).to_string();
            def.sidetext = "\u{b0}".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1641
            def.max = 180_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1643
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.)));
        }
        {
            let def = self
                .def
                .add("internal_bridge_angle", ConfigOptionType::Float);
            def.label = "Internal bridge infill direction".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Internal Bridging angle override.\nIf left to zero, the bridging angle will be calculated automatically for each specific bridge.\nOtherwise the provided angle will be used according to:\n - The absolute coordinates\n - The absolute coordinates + Model rotation: If Align directions to model is enabled\n - The optimal automatic angle + this value: If 'Relative Bridge Angle' is enabled\n\nUse 180° for zero absolute angle.".to_string();
            def.sidetext = "°".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1658
            def.max = 180_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1660
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.)));
        }
        {
            let def = self
                .def
                .add("relative_bridge_angle", ConfigOptionType::Bool);
            def.label = "Relative bridge angle".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "When enabled, the bridge angle values are added to the automatically calculated bridge direction instead of overriding it.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("bridge_density", ConfigOptionType::Percent);
            def.label = "External bridge density".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Controls the density (spacing) of external bridge lines.\nTheoretically, 100% means a solid bridge, but due to the tendency of bridge extrusions to sag, 100% may not be sufficient.\n\n- Higher than 100% density (Recommended Max 125%):\n  - Pros: Produces smoother bridge surfaces, as overlapping lines provide additional support during printing.\n  - Cons: Can cause overextrusion, which may reduce lower and upper surface quality and increase the risk of warping.\n\n- Lower than 100% density (Min 10%):\n  - Pros: Can create a string-like first layer. Faster and with better cooling because there is more space for air to circulate around the extruded bridge.\n  - Cons: May lead to sagging and poorer surface finish.".to_string();
            def.sidetext = "%".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1682
            def.min = 10_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1683
            def.max = 125_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1685
            def.set_default_value(Box::new(ConfigOptionPercent::new(100_f64)));
        }
        {
            let def = self
                .def
                .add("internal_bridge_density", ConfigOptionType::Percent);
            def.label = "Internal bridge density".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Controls the density (spacing) of internal bridge lines.\nInternal bridges act as intermediate support between sparse infill and top solid infill and can strongly affect top surface quality.\n\n- Higher than 100% density (Recommended Max 125%):\n  - Pros: Improves internal bridge strength and support under top layers, reducing sagging and improving top-surface finish.\n  - Cons: Increases material use and print time; excessive density may cause overextrusion and internal stresses.\n\n- Lower than 100% density (Min 10%):\n  - Pros: Can reduce pillowing and improve cooling (more airflow through the bridge), and may speed up printing.\n  - Cons: May reduce internal support, increasing the risk of sagging and top surface defects.\n\nThis option works particularly well when combined with the second internal bridge over infill option to improve bridging further before solid infill is extruded.".to_string();
            def.sidetext = "%".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1700
            def.min = 10_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1701
            def.max = 125_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1703
            def.set_default_value(Box::new(ConfigOptionPercent::new(100_f64)));
        }
        {
            let def = self.def.add("bridge_flow", ConfigOptionType::Float);
            def.label = "Bridge flow ratio".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "This value governs the thickness of the external (visible) bridge layer.\nValues above 1.0: Increase the amount of material while maintaining line spacing. This can improve line contact and strength.\nValues below 1.0: Reduce the amount of material while adjusting line spacing to maintain contact. This can improve sagging.\n\nThe actual bridge flow used is calculated by multiplying this value with the filament flow ratio, and if set, the object's flow ratio.".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1713
            def.max = 2.0;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(1_f64)));
        }
        {
            let def = self
                .def
                .add("bridge_line_width", ConfigOptionType::FloatOrPercent);
            def.label = "Bridge".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Line width of the Bridge. If expressed as a %, it will be computed over the nozzle diameter.\nRecommended to use with a higher Bridge density or Bridge flow ratio.\n\nThe maximum value is 100% or the nozzle diameter.\nIf set to 0, the line width will match the Internal solid infill width, or the nozzle diameter for thick bridges.".to_string();
            def.sidetext = "mm or %".to_string();
            def.ratio_over = "nozzle_diameter".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1727
            def.max = 100_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1728
            def.max_literal = 10_f64;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1730
            def.set_default_value(Box::new(ConfigOptionFloatOrPercent::new(100., true)));
        }
        {
            let def = self
                .def
                .add("internal_bridge_flow", ConfigOptionType::Float);
            def.label = "Internal bridge flow ratio".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "This value governs the thickness of the internal bridge layer. This is the first layer over sparse infill so increasing it may increase strength and upper layer quality.\nValues above 1.0: Increase the amount of material while maintaining line spacing. This can improve line contact and strength.\nValues below 1.0: Reduce the amount of material while adjusting line spacing to maintain contact. This can improve sagging.\n\nThe actual bridge flow used is calculated by multiplying this value with the filament flow ratio, and if set, the object's flow ratio.".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1740
            def.max = 2.0;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(1_f64)));
        }
        {
            let def = self
                .def
                .add("top_solid_infill_flow_ratio", ConfigOptionType::Floats);
            def.label = "Top surface flow ratio".to_string();
            def.category = "Advanced".to_string();
            def.tooltip = "This factor affects the amount of material for top solid infill. You can decrease it slightly to have smooth surface finish.\n\nThe actual top surface flow used is calculated by multiplying this value with the filament flow ratio, and if set, the object's flow ratio.".to_string();
            def.min = 0_f32;
            def.max = 2_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            v.values.push(1_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("bottom_solid_infill_flow_ratio", ConfigOptionType::Float);
            def.label = "Bottom surface flow ratio".to_string();
            def.category = "Advanced".to_string();
            def.tooltip = "This factor affects the amount of material for bottom solid infill.\n\nThe actual bottom solid infill flow used is calculated by multiplying this value with the filament flow ratio, and if set, the object's flow ratio.".to_string();
            def.min = 0_f32;
            def.max = 2_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(1_f64)));
        }
        {
            let def = self
                .def
                .add("set_other_flow_ratios", ConfigOptionType::Bool);
            def.label = "Set other flow ratios".to_string();
            def.category = "Advanced".to_string();
            def.tooltip = "Change flow ratios for other extrusion path types.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("first_layer_flow_ratio", ConfigOptionType::Float);
            def.label = "First layer flow ratio".to_string();
            def.category = "Advanced".to_string();
            def.tooltip = "This factor affects the amount of material on the first layer for the extrusion path roles listed in this section.\n\nFor the first layer, the actual flow ratio for each path role (does not affect brims and skirts) will be multiplied by this value.".to_string();
            def.min = 0_f32;
            def.max = 2_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(1_f64)));
        }
        {
            let def = self
                .def
                .add("outer_wall_flow_ratio", ConfigOptionType::Float);
            def.label = "Outer wall flow ratio".to_string();
            def.category = "Advanced".to_string();
            def.tooltip = "This factor affects the amount of material for outer walls.\n\nThe actual outer wall flow used is calculated by multiplying this value by the filament flow ratio, and if set, the object's flow ratio.".to_string();
            def.min = 0_f32;
            def.max = 2_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(1_f64)));
        }
        {
            let def = self
                .def
                .add("inner_wall_flow_ratio", ConfigOptionType::Float);
            def.label = "Inner wall flow ratio".to_string();
            def.category = "Advanced".to_string();
            def.tooltip = "This factor affects the amount of material for inner walls.\n\nThe actual inner wall flow used is calculated by multiplying this value by the filament flow ratio, and if set, the object's flow ratio.".to_string();
            def.min = 0_f32;
            def.max = 2_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(1_f64)));
        }
        {
            let def = self.def.add("overhang_flow_ratio", ConfigOptionType::Float);
            def.label = "Overhang flow ratio".to_string();
            def.category = "Advanced".to_string();
            def.tooltip = "This factor affects the amount of material for overhangs.\n\nThe actual overhang flow used is calculated by multiplying this value by the filament flow ratio, and if set, the object's flow ratio.".to_string();
            def.min = 0_f32;
            def.max = 2_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(1_f64)));
        }
        {
            let def = self
                .def
                .add("sparse_infill_flow_ratio", ConfigOptionType::Float);
            def.label = "Sparse infill flow ratio".to_string();
            def.category = "Advanced".to_string();
            def.tooltip = "This factor affects the amount of material for sparse infill.\n\nThe actual sparse infill flow used is calculated by multiplying this value by the filament flow ratio, and if set, the object's flow ratio.".to_string();
            def.min = 0_f32;
            def.max = 2_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(1_f64)));
        }
        {
            let def = self
                .def
                .add("internal_solid_infill_flow_ratio", ConfigOptionType::Float);
            def.label = "Internal solid infill flow ratio".to_string();
            def.category = "Advanced".to_string();
            def.tooltip = "This factor affects the amount of material for internal solid infill.\n\nThe actual internal solid infill flow used is calculated by multiplying this value by the filament flow ratio, and if set, the object's flow ratio.".to_string();
            def.min = 0_f32;
            def.max = 2_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(1_f64)));
        }
        {
            let def = self.def.add("gap_fill_flow_ratio", ConfigOptionType::Float);
            def.label = "Gap fill flow ratio".to_string();
            def.category = "Advanced".to_string();
            def.tooltip = "This factor affects the amount of material for filling the gaps.\n\nThe actual gap filling flow used is calculated by multiplying this value by the filament flow ratio, and if set, the object's flow ratio.".to_string();
            def.min = 0_f32;
            def.max = 2_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(1_f64)));
        }
        {
            let def = self.def.add("support_flow_ratio", ConfigOptionType::Float);
            def.label = "Support flow ratio".to_string();
            def.category = "Advanced".to_string();
            def.tooltip = "This factor affects the amount of material for support.\n\nThe actual support flow used is calculated by multiplying this value by the filament flow ratio, and if set, the object's flow ratio.".to_string();
            def.min = 0_f32;
            def.max = 2_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(1_f64)));
        }
        {
            let def = self
                .def
                .add("support_interface_flow_ratio", ConfigOptionType::Float);
            def.label = "Support interface flow ratio".to_string();
            def.category = "Advanced".to_string();
            def.tooltip = "This factor affects the amount of material for the support interface.\n\nThe actual support interface flow used is calculated by multiplying this value by the filament flow ratio, and if set, the object's flow ratio.".to_string();
            def.min = 0_f32;
            def.max = 2_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(1_f64)));
        }
        {
            let def = self.def.add("precise_outer_wall", ConfigOptionType::Bool);
            def.label = "Precise wall".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Improve shell precision by adjusting outer wall spacing. This also improves layer consistency.\n\nNote: This option will be ignored for outer-inner or inner-outer-inner wall sequences.".to_string();
            def.set_default_value(Box::new(ConfigOptionBool::new(true)));
        }
        {
            let def = self.def.add("only_one_wall_top", ConfigOptionType::Bool);
            def.label = "Only one wall on top surfaces".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Use only one wall on flat top surfaces, to give more space to the top infill pattern.".to_string();
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("min_width_top_surface", ConfigOptionType::FloatOrPercent);
            def.label = "One wall threshold".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "If a top surface has to be printed and it's partially covered by another layer, it won't be considered at a top layer where its width is below this value. This can be useful to not let the 'one perimeter on top' trigger on surface that should be covered only by perimeters. This value can be a mm or a % of the perimeter extrusion width.\n\nWarning: If enabled, artifacts can be created if you have some thin features on the next layer, like letters. Set this setting to 0 to remove these artifacts.".to_string();
            def.sidetext = "mm or %".to_string();
            def.ratio_over = "inner_wall_line_width".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1888
            def.max_literal = 15_f64;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1890
            def.set_default_value(Box::new(ConfigOptionFloatOrPercent::new(300_f64, true)));
        }
        {
            let def = self
                .def
                .add("only_one_wall_first_layer", ConfigOptionType::Bool);
            def.label = "Only one wall on first layer".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Use only one wall on first layer, to give more space to the bottom infill pattern.".to_string();
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("extra_perimeters_on_overhangs", ConfigOptionType::Bool);
            def.label = "Extra perimeters on overhangs".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Create additional perimeter paths over steep overhangs and areas where bridges cannot be anchored.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("overhang_reverse", ConfigOptionType::Bool);
            def.label = "Reverse on even".to_string();
            def.full_label = "Overhang reversal".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Extrude perimeters that have a part over an overhang in the reverse direction on even layers. This alternating pattern can drastically improve steep overhangs.\n\nThis setting can also help reduce part warping due to the reduction of stresses in the part walls.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("overhang_reverse_internal_only", ConfigOptionType::Bool);
            def.label = "Reverse only internal perimeters".to_string();
            def.full_label = "Reverse only internal perimeters".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Apply the reverse perimeters logic only on internal perimeters.\n\nThis setting greatly reduces part stresses as they are now distributed in alternating directions. This should reduce part warping while also maintaining external wall quality. This feature can be very useful for warp prone material, like ABS/ASA, and also for elastic filaments, like TPU and Silk PLA. It can also help reduce warping on floating regions over supports.\n\nFor this setting to be the most effective, it is recommended to set the Reverse Threshold to 0 so that all internal walls print in alternating directions on even layers irrespective of their overhang degree.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("counterbore_hole_bridging", ConfigOptionType::Enum);
            def.label = "Bridge counterbore holes".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "This option creates bridges for counterbore holes, allowing them to be printed without support. Available modes include:\n1. None: No bridge is created\n2. Partially Bridged: Only a part of the unsupported area will be bridged\n3. Sacrificial Layer: A full sacrificial bridge layer is created".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.enum_keys_map = keys_map_counterbore_hole_bridging_option();
            def.enum_values.push("none".to_string());
            def.enum_values.push("partiallybridge".to_string());
            def.enum_values.push("sacrificiallayer".to_string());
            def.enum_labels.push("None".to_string());
            def.enum_labels.push("Partially bridged".to_string());
            def.enum_labels.push("Sacrificial layer".to_string());
            def.set_default_value(Box::new(ConfigOptionEnum::new(
                CounterboreHoleBridgingOption::chbNone,
            )));
        }
        {
            let def = self.def.add(
                "overhang_reverse_threshold",
                ConfigOptionType::FloatOrPercent,
            );
            def.label = "Reverse threshold".to_string();
            def.full_label = "Overhang reversal threshold".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Number of mm the overhang need to be for the reversal to be considered useful. Can be a % of the perimeter width.\nValue 0 enables reversal on every even layers regardless.\nWhen Detect overhang wall is not enabled, this option is ignored and reversal happens on every even layers regardless.".to_string();
            def.sidetext = "mm or %".to_string();
            def.ratio_over = "line_width".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1955
            def.max_literal = 20_f64;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:1957
            def.set_default_value(Box::new(ConfigOptionFloatOrPercent::new(50_f64, true)));
        }
        {
            let def = self
                .def
                .add("enable_overhang_speed", ConfigOptionType::Bools);
            def.label = "Slow down for overhangs".to_string();
            def.category = "Speed".to_string();
            def.tooltip = "Enable this option to slow down when printing overhangs. The speeds for different overhang percentages are set below.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionBoolsNullable::new();
            v.values.push(1);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("slowdown_for_curled_perimeters", ConfigOptionType::Bools);
            def.label = "Slow down for curled perimeters".to_string();
            def.category = "Speed".to_string();
            def.tooltip = "Enable this option to slow down printing in areas where perimeters may have curled upwards.\nFor example, additional slowdown will be applied when printing overhangs on sharp corners like the front of the Benchy hull, reducing curling which compounds over multiple layers.\n\nIt is generally recommended to have this option switched on unless your printer cooling is powerful enough or the print speed is slow enough that perimeter curling does not happen.\nIf printing with a high external perimeter speed, this parameter may introduce wall artifacts when slowing down, due to the potentially large variance in print speeds causing the extruder to be unable to keep up with the requested flow change.\nRoot cause of these artifacts is most likely PA tuning being slightly off, especially when combined with a high PA smooth time.\n\nRecommendations when enabling this option:\n1. Reduce Pressure Advance smooth time to 0.015 - 0.02 so the extruder reacts quickly to the speed changes.\n2. Increase the minimum print speeds to limit the magnitude of the slowdown and reduce the variance between fast and slow segments.\n3. If artifacts still appear, enable Extrusion Rate Smoothing (ERS) to further smooth the flow transitions.\n\nNote: When this option is enabled, overhang perimeters are treated like overhangs, meaning the overhang speed is applied even if the overhanging perimeter is part of a bridge.\nFor example, when the perimeters are 100% overhanging, with no wall supporting them from underneath, the 75% overhang speed will be applied, limited by the bridge speed.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionBoolsNullable::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("overhang_1_4_speed", ConfigOptionType::FloatsOrPercents);
            def.label = "10%".to_string();
            def.category = "Speed".to_string();
            def.full_label = "10%".to_string();
            def.sidetext = "mm/s or %".to_string();
            def.ratio_over = "outer_wall_speed".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsOrPercentsNullable { values: Vec::new() };
            v.values.push(slic3r_config::config::FloatOrPercent {
                value: 0_f64,
                percent: false,
            });
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("overhang_2_4_speed", ConfigOptionType::FloatsOrPercents);
            def.label = "25%".to_string();
            def.category = "Speed".to_string();
            def.full_label = "25%".to_string();
            def.sidetext = "mm/s or %".to_string();
            def.ratio_over = "outer_wall_speed".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsOrPercentsNullable { values: Vec::new() };
            v.values.push(slic3r_config::config::FloatOrPercent {
                value: 0_f64,
                percent: false,
            });
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("overhang_3_4_speed", ConfigOptionType::FloatsOrPercents);
            def.label = "50%".to_string();
            def.category = "Speed".to_string();
            def.full_label = "50%".to_string();
            def.sidetext = "mm/s or %".to_string();
            def.ratio_over = "outer_wall_speed".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsOrPercentsNullable { values: Vec::new() };
            v.values.push(slic3r_config::config::FloatOrPercent {
                value: 0_f64,
                percent: false,
            });
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("overhang_4_4_speed", ConfigOptionType::FloatsOrPercents);
            def.label = "75%".to_string();
            def.category = "Speed".to_string();
            def.full_label = "75%".to_string();
            def.sidetext = "mm/s or %".to_string();
            def.ratio_over = "outer_wall_speed".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsOrPercentsNullable { values: Vec::new() };
            v.values.push(slic3r_config::config::FloatOrPercent {
                value: 0_f64,
                percent: false,
            });
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("bridge_speed", ConfigOptionType::Floats);
            def.label = "External".to_string();
            def.category = "Speed".to_string();
            def.tooltip = "Speed of the externally visible bridge extrusions.\n\nIn addition, if Slow down for curled perimeters is disabled, it will be the print speed of overhang walls that are supported by less than 13%, whether they are part of a bridge or an overhang.".to_string();
            def.sidetext = "mm/s".to_string();
            def.min = 1_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2054
            v.values.push(25_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("internal_bridge_speed", ConfigOptionType::FloatsOrPercents);
            def.label = "Internal".to_string();
            def.category = "Speed".to_string();
            def.tooltip = "Speed of internal bridges. If the value is expressed as a percentage, it will be calculated based on the bridge_speed. Default value is 150%.".to_string();
            def.sidetext = "mm/s or %".to_string();
            def.ratio_over = "bridge_speed".to_string();
            def.min = 1_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsOrPercentsNullable { values: Vec::new() };
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2065
            v.values.push(slic3r_config::config::FloatOrPercent {
                value: 150_f64,
                percent: true,
            });
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("brim_width", ConfigOptionType::Float);
            def.label = "Brim width".to_string();
            def.category = "Support".to_string();
            def.tooltip =
                "This is the distance from the model to the outermost brim line.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2073
            def.max = 100_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2075
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.)));
        }
        {
            let def = self.def.add("leading_brim_length", ConfigOptionType::Float);
            def.label = "Leading brim length".to_string();
            def.category = "Support".to_string();
            def.tooltip = "Belt printers only. Extends the brim AHEAD of the object along the belt, on every downhill-facing edge of its contact area - both the object's first contact with the belt and any island that lands later. This apron is laid onto the belt before the object reaches it, so the leading edge has something already stuck down to hold on to.\n\nMeasured on the belt surface, and added on top of Brim width: the brim reaches Brim-object gap + Leading brim length + Brim width ahead of the object. Set Brim-object gap to 0, or the apron will not touch the object it is meant to anchor.\n\nOn a tilted belt each layer lays one strip of the brim, so the thickness of the resulting brim sheet is set by flow rather than by layer height. Use Brim flow ratio to tune it.\n\nSet to 0 to disable.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2095
            def.max = 100_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2097
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.)));
        }
        {
            let def = self.def.add("extra_brim_width", ConfigOptionType::Float);
            def.label = "Extra brim width".to_string();
            def.category = "Support".to_string();
            def.tooltip = "Belt printers only. Widens the brim SIDEWAYS, across the belt, without extending it further ahead of or behind the object. Use it when a part needs more grip along its length than Brim width alone gives.\n\nMeasured on the belt surface, and added on top of Brim width: the brim reaches Brim-object gap + Brim width + Extra brim width to either side of the object. To extend the brim ahead of the object instead, use Leading brim length.\n\nSet to 0 to disable.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2112
            def.max = 100_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2114
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.)));
        }
        {
            let def = self.def.add("brim_type", ConfigOptionType::Enum);
            def.label = "Brim type".to_string();
            def.category = "Support".to_string();
            def.tooltip = "This controls the generation of the brim at outer and/or inner side of models. Auto means the brim width is analyzed and calculated automatically.".to_string();
            def.enum_keys_map = keys_map_brim_type();
            def.enum_values.push("auto_brim".to_string());
            def.enum_values.push("brim_ears".to_string());
            def.enum_values.push("painted".to_string());
            def.enum_values.push("outer_only".to_string());
            def.enum_values.push("inner_only".to_string());
            def.enum_values.push("outer_and_inner".to_string());
            def.enum_values.push("no_brim".to_string());
            def.enum_values.push("leading_edge_only".to_string());
            def.enum_labels.push("Auto".to_string());
            def.enum_labels.push("Mouse ear".to_string());
            def.enum_labels.push("Painted".to_string());
            def.enum_labels.push("Outer brim only".to_string());
            def.enum_labels.push("Inner brim only".to_string());
            def.enum_labels.push("Outer and inner brim".to_string());
            def.enum_labels.push("No-brim".to_string());
            def.enum_labels.push("Leading edge only".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            def.set_default_value(Box::new(ConfigOptionEnum::new(BrimType::btAutoBrim)));
        }
        {
            let def = self.def.add("brim_object_gap", ConfigOptionType::Float);
            def.label = "Brim-object gap".to_string();
            def.category = "Support".to_string();
            def.tooltip = "This creates a gap between the innermost brim line and the object and can make the brim easier to remove.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.max = 2_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2149
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.)));
        }
        {
            let def = self.def.add("brim_flow_ratio", ConfigOptionType::Float);
            def.label = "Brim flow ratio".to_string();
            def.category = "Support".to_string();
            def.tooltip = "This factor affects the amount of material for brims.\n\nThe actual brim flow used is calculated by multiplying this value by the filament flow ratio, and if set, the object's flow ratio.\n\nNote: The resulting value will not be affected by the first-layer flow ratio.".to_string();
            def.min = 0_f32;
            def.max = 2_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(1_f64)));
        }
        {
            let def = self.def.add("brim_use_efc_outline", ConfigOptionType::Bool);
            def.label = "Brim follows compensated outline".to_string();
            def.category = "Support".to_string();
            def.tooltip = "When enabled, the brim is aligned with the first-layer perimeter geometry after Elephant Foot Compensation is applied.\nThis option is intended for cases where Elephant Foot Compensation significantly alters the first-layer footprint.\n\nIf your current setup already works well, enabling it may be unnecessary and can cause the brim to fuse with upper layers.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("combine_brims", ConfigOptionType::Bool);
            def.label = "Combine brims".to_string();
            def.category = "Support".to_string();
            def.tooltip = "Combine multiple brims into one when they are close to each other. This can improve brim adhesion.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("brim_ears", ConfigOptionType::Bool);
            def.label = "Brim ears".to_string();
            def.category = "Support".to_string();
            def.tooltip = "Only draw brim over the sharp edges of the model.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("brim_ears_max_angle", ConfigOptionType::Float);
            def.label = "Brim ear max angle".to_string();
            def.category = "Support".to_string();
            def.tooltip = "Maximum angle to let a brim ear appear.\nIf set to 0, no brim will be created.\nIf set to ~180, brim will be created on everything but straight sections.".to_string();
            def.sidetext = "°".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2191
            def.max = 180_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2193
            def.set_default_value(Box::new(ConfigOptionFloat::new(125_f64)));
        }
        {
            let def = self
                .def
                .add("brim_ears_detection_length", ConfigOptionType::Float);
            def.label = "Brim ear detection radius".to_string();
            def.category = "Support".to_string();
            def.tooltip = "The geometry will be decimated before detecting sharp angles. This parameter indicates the minimum length of the deviation for the decimation.\n0 to deactivate.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(1_f64)));
        }
        {
            let def = self.def.add("brim_ears_outer_only", ConfigOptionType::Bool);
            def.label = "Brim ears outer only".to_string();
            def.category = "Support".to_string();
            def.tooltip = "Generate mouse ears only on the outer contour of the model, excluding holes and enclosed sections.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("compatible_printers", ConfigOptionType::Strings);
            def.label = "Select printers".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionStrings::new()));
            def.cli = NOCLI.to_string();
        }
        {
            let def = self
                .def
                .add("upward_compatible_machine", ConfigOptionType::Strings);
            def.label = "upward compatible machine".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionStrings::new()));
            def.cli = NOCLI.to_string();
        }
        {
            let def = self
                .def
                .add("compatible_printers_condition", ConfigOptionType::String);
            def.label = "Condition".to_string();
            def.tooltip = "A Boolean expression using the configuration values of an active printer profile. If this expression evaluates to true, this profile is considered compatible with the active printer profile.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
            def.cli = NOCLI.to_string();
        }
        {
            let def = self.def.add("compatible_prints", ConfigOptionType::Strings);
            def.label = "Select profiles".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionStrings::new()));
            def.cli = NOCLI.to_string();
        }
        {
            let def = self
                .def
                .add("compatible_prints_condition", ConfigOptionType::String);
            def.label = "Condition".to_string();
            def.tooltip = "A Boolean expression using the configuration values of an active print profile. If this expression evaluates to true, this profile is considered compatible with the active print profile.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
            def.cli = NOCLI.to_string();
        }
        {
            let def = self.def.add(
                "compatible_machine_expression_group",
                ConfigOptionType::Strings,
            );
            def.set_default_value(Box::new(ConfigOptionStrings::new()));
            def.cli = NOCLI.to_string();
        }
        {
            let def = self.def.add(
                "compatible_process_expression_group",
                ConfigOptionType::Strings,
            );
            def.set_default_value(Box::new(ConfigOptionStrings::new()));
            def.cli = NOCLI.to_string();
        }
        {
            let def = self
                .def
                .add("different_settings_to_system", ConfigOptionType::Strings);
            def.set_default_value(Box::new(ConfigOptionStrings::new()));
            def.cli = NOCLI.to_string();
        }
        {
            let def = self
                .def
                .add("print_compatible_printers", ConfigOptionType::Strings);
            def.set_default_value(Box::new(ConfigOptionStrings::new()));
            def.cli = NOCLI.to_string();
        }
        {
            let def = self.def.add("print_sequence", ConfigOptionType::Enum);
            def.label = "Print sequence".to_string();
            def.tooltip = "This determines the print sequence, allowing you to print layer-by-layer or object-by-object.".to_string();
            def.enum_keys_map = keys_map_print_sequence();
            def.enum_values.push("by layer".to_string());
            def.enum_values.push("by object".to_string());
            def.enum_labels.push("By layer".to_string());
            def.enum_labels.push("By object".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            def.set_default_value(Box::new(ConfigOptionEnum::new(PrintSequence::ByLayer)));
        }
        {
            let def = self.def.add("print_order", ConfigOptionType::Enum);
            def.label = "Intra-layer order".to_string();
            def.tooltip = "Order in which object instances are visited within a single layer, which controls how much travel is spent moving between them.\n\nDefault: nearest-neighbor chaining, refined with 2-opt and crossing removal. A good general choice.\nAs object list: instances are printed in the same order as the object list, without any path optimization. Use it when you need a predictable, manually controlled order.\nBest of all (shortest path): every strategy is evaluated and the shortest one is used. The object instance order is decided once for the whole print, while the ordering of individual islands is decided per layer, so different layers may end up using different strategies. Slightly slower to slice.\nSnake: serpentine row-by-row traversal, refined with 2-opt. Well suited to regular grids of many small parts.\n\nWith multiple filaments or tools in the same layer, minimizing tool changes takes priority: objects are grouped by filament first and this setting only orders the instances within each filament group, so the overall sequence may not look like the shortest path across the plate.".to_string();
            def.enum_keys_map = keys_map_print_order();
            def.enum_values.push("default".to_string());
            def.enum_values.push("as_obj_list".to_string());
            def.enum_values.push("best_of".to_string());
            def.enum_values.push("snake".to_string());
            def.enum_labels.push("Default".to_string());
            def.enum_labels.push("As object list".to_string());
            def.enum_labels
                .push("Best of all (shortest path)".to_string());
            def.enum_labels.push("Snake".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionEnum::new(PrintOrder::Default)));
        }
        {
            let def = self
                .def
                .add("slow_down_for_layer_cooling", ConfigOptionType::Bools);
            def.label = "Slow printing down for better layer cooling".to_string();
            def.tooltip = "Enable this option to slow printing speed down to ensure that the final layer time is not shorter than the layer time threshold in \"Max fan speed threshold\", so that the layer can be cooled for a longer time. This can improve the quality for small details.".to_string();
            let mut v = ConfigOptionBools::new();
            v.values.push(1);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("default_acceleration", ConfigOptionType::Floats);
            def.label = "Normal printing".to_string();
            def.category = "Speed".to_string();
            def.tooltip = "This is the default acceleration for both normal printing and travel after the first layer.".to_string();
            def.sidetext = "mm/s²".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2317
            v.values.push(500.0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("travel_acceleration", ConfigOptionType::Floats);
            def.label = "Travel".to_string();
            def.category = "Speed".to_string();
            def.tooltip = "Acceleration of travel moves.".to_string();
            def.sidetext = "mm/s²".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2327
            v.values.push(10000.0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "initial_layer_travel_acceleration",
                ConfigOptionType::FloatsOrPercents,
            );
            def.label = "First layer travel".to_string();
            def.tooltip = "Travel acceleration of first layer.\nThe percentage value is relative to Travel Acceleration.".to_string();
            def.sidetext = "mm/s² or %".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.ratio_over = "travel_acceleration".to_string();
            def.nullable = true;
            let mut v = ConfigOptionFloatsOrPercentsNullable { values: Vec::new() };
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2337
            v.values.push(slic3r_config::config::FloatOrPercent {
                value: 100_f64,
                percent: true,
            });
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("bridge_acceleration", ConfigOptionType::FloatsOrPercents);
            def.label = "Bridge".to_string();
            def.category = "Speed".to_string();
            def.tooltip = "Acceleration of bridges. If the value is expressed as a percentage (e.g. 50%), it will be calculated based on the outer wall acceleration.".to_string();
            def.sidetext = "mm/s² or %".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.ratio_over = "outer_wall_acceleration".to_string();
            def.nullable = true;
            let mut v = ConfigOptionFloatsOrPercentsNullable { values: Vec::new() };
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2348
            v.values.push(slic3r_config::config::FloatOrPercent {
                value: 50_f64,
                percent: true,
            });
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("default_filament_profile", ConfigOptionType::Strings);
            def.label = "Default filament profile".to_string();
            def.tooltip =
                "Default filament profile when switching to this machine profile.".to_string();
            def.set_default_value(Box::new(ConfigOptionStrings::new()));
            def.cli = NOCLI.to_string();
        }
        {
            let def = self
                .def
                .add("default_print_profile", ConfigOptionType::String);
            def.label = "Default process profile".to_string();
            def.tooltip =
                "Default process profile when switching to this machine profile.".to_string();
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
            def.cli = NOCLI.to_string();
        }
        {
            let def = self
                .def
                .add("activate_air_filtration", ConfigOptionType::Bools);
            def.label = "Activate air filtration".to_string();
            def.tooltip =
                "Activate for better air filtration.\nG-code command: M106 P3 S(0-255)".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            let mut v = ConfigOptionBools::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "activate_air_filtration_during_print",
                ConfigOptionType::Bools,
            );
            def.full_label = "During print".to_string();
            def.tooltip =
                "Enable this to override the fan speed set in custom G-code during print."
                    .to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            let mut v = ConfigOptionBools::new();
            v.values.push(1);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "activate_air_filtration_on_completion",
                ConfigOptionType::Bools,
            );
            def.full_label = "On completion".to_string();
            def.tooltip = "Enable this to override the fan speed set in custom G-code after print completion.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            let mut v = ConfigOptionBools::new();
            v.values.push(1);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("during_print_exhaust_fan_speed", ConfigOptionType::Ints);
            def.label = "Fan speed".to_string();
            def.tooltip = "Speed of exhaust fan during printing. This speed will override the speed in filament custom G-code.".to_string();
            def.sidetext = "%".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2386
            def.max = 100_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            let mut v = ConfigOptionInts::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2388
            v.values.push(60);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("complete_print_exhaust_fan_speed", ConfigOptionType::Ints);
            def.label = "Fan speed".to_string();
            def.tooltip = "Speed of exhaust fan after printing completes.".to_string();
            def.sidetext = "%".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2395
            def.max = 100_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            let mut v = ConfigOptionInts::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2397
            v.values.push(80);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("close_fan_the_first_x_layers", ConfigOptionType::Ints);
            def.label = "No cooling for the first".to_string();
            def.tooltip = "Turn off all cooling fans for the first few layers. This can be used to improve build plate adhesion.".to_string();
            def.sidetext = "layers".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2405
            def.max = 1000_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            let mut v = ConfigOptionInts::new();
            v.values.push(1);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("bridge_no_support", ConfigOptionType::Bool);
            def.label = "Don't support bridges".to_string();
            def.category = "Support".to_string();
            def.tooltip = "This disables supporting bridges, which decreases the amount of support required. Bridges can usually be printed directly without support over a reasonable distance.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("thick_bridges", ConfigOptionType::Bool);
            def.label = "Thick external bridges".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "If enabled, bridge extrusion uses a line height equal to the nozzle diameter.\nThis increases bridge strength and reliability, allowing longer spans, but may worsen appearance.\nIf disabled, bridges may look better but are generally reliable only for shorter spans.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("thick_internal_bridges", ConfigOptionType::Bool);
            def.label = "Thick internal bridges".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "If enabled, internal bridge extrusion uses a line height equal to the nozzle diameter.\nThis increases internal bridge strength and reliability when printed over sparse infill, but may worsen appearance.\nIf disabled, internal bridges may look better but can be less reliable over sparse infill.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(true)));
        }
        {
            let def = self
                .def
                .add("enable_extra_bridge_layer", ConfigOptionType::Enum);
            def.label = "Extra bridge layers (beta)".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "This option enables the generation of an extra bridge layer over internal and/or external bridges.\n\nExtra bridge layers help improve bridge appearance and reliability, as the solid infill is better supported. This is especially useful in fast printers, where the bridge and solid infill speeds vary greatly. The extra bridge layer results in reduced pillowing on top surfaces, as well as reduced separation of the external bridge layer from its surrounding perimeters.\n\nIt is generally recommended to set this to at least 'External bridge only', unless specific issues with the sliced model are found.\n\nOptions:\n1. Disabled - does not generate second bridge layers. This is the default and is set for compatibility purposes\n2. External bridge only - generates second bridge layers for external-facing bridges only. Please note that small bridges that are shorter or narrower than the set number of perimeters will be skipped as they would not benefit from a second bridge layer. If generated, the second bridge layer will be extruded parallel to the first bridge layer to reinforce the bridge strength\n3. Internal bridge only - generates second bridge layers for internal bridges over sparse infill only. Please note that the internal bridges count towards the top shell layer count of your model. The second internal bridge layer will be extruded as close to perpendicular to the first as possible. If multiple regions in the same island, with varying bridge angles are present, the last region of that island will be selected as the angle reference\n4. Apply to all - generates second bridge layers for both internal and external-facing bridges\n".to_string();
            def.enum_keys_map = keys_map_enable_extra_bridge_layer();
            def.enum_values.push("disabled".to_string());
            def.enum_values.push("external_bridge_only".to_string());
            def.enum_values.push("internal_bridge_only".to_string());
            def.enum_values.push("apply_to_all".to_string());
            def.enum_labels.push("Disabled".to_string());
            def.enum_labels.push("External bridge only".to_string());
            def.enum_labels.push("Internal bridge only".to_string());
            def.enum_labels.push("Apply to all".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionEnum::new(
                EnableExtraBridgeLayer::eblDisabled,
            )));
        }
        {
            let def = self
                .def
                .add("dont_filter_internal_bridges", ConfigOptionType::Enum);
            def.label = "Filter out small internal bridges".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "This option can help reduce pillowing on top surfaces in heavily slanted or curved models.\nBy default, small internal bridges are filtered out and the internal solid infill is printed directly over the sparse infill. This works well in most cases, speeding up printing without too much compromise on top surface quality.\nHowever, in heavily slanted or curved models, especially where too low a sparse infill density is used, this may result in curling of the unsupported solid infill, causing pillowing.\nEnabling limited filtering or no filtering will print internal bridge layer over slightly unsupported internal solid infill. The options below control the sensitivity of the filtering, i.e. they control where internal bridges are created:\n1. Filter - enables this option. This is the default behavior and works well in most cases\n2. Limited filtering - creates internal bridges on heavily slanted surfaces while avoiding unnecessary bridges. This works well for most difficult models\n3. No filtering - creates internal bridges on every potential internal overhang. This option is useful for heavily slanted top surface models; however, in most cases, it creates too many unnecessary bridges.".to_string();
            def.enum_keys_map = keys_map_internal_bridge_filter();
            def.enum_values.push("disabled".to_string());
            def.enum_values.push("limited".to_string());
            def.enum_values.push("nofilter".to_string());
            def.enum_labels.push("Filter".to_string());
            def.enum_labels.push("Limited filtering".to_string());
            def.enum_labels.push("No filtering".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionEnum::new(
                InternalBridgeFilter::ibfDisabled,
            )));
        }
        {
            let def = self.def.add("max_bridge_length", ConfigOptionType::Float);
            def.label = "Max bridge length".to_string();
            def.category = "Support".to_string();
            def.tooltip = "This is the maximum length of bridges that don't need support. Set it to 0 if you want all bridges to be supported, and set it to a very large value if you don't want any bridges to be supported.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2501
            def.set_default_value(Box::new(ConfigOptionFloat::new(10_f64)));
        }
        {
            let def = self.def.add("machine_end_gcode", ConfigOptionType::String);
            def.label = "End G-code".to_string();
            def.tooltip = "Add end G-code when finishing the entire print.".to_string();
            def.multiline = true;
            def.full_width = true;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2508
            def.height = 12;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionString::new(
                "M104 S0 ; turn off temperature\nG28 X0  ; home X axis\nM84     ; disable motors\n"
                    .to_string(),
            )));
        }
        {
            let def = self
                .def
                .add("printing_by_object_gcode", ConfigOptionType::String);
            def.label = "Between Object G-code".to_string();
            def.tooltip = "Insert G-code between objects. This parameter will only come into effect when you print your models object by object.".to_string();
            def.multiline = true;
            def.full_width = true;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2517
            def.height = 12;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
        }
        {
            let def = self
                .def
                .add("filament_end_gcode", ConfigOptionType::Strings);
            def.label = "End G-code".to_string();
            def.tooltip =
                "Add end G-code when finishing the printing of this filament.".to_string();
            def.multiline = true;
            def.full_width = true;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2526
            def.height = 120;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionStrings::new();
            v.values.push(" ".to_string());
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("ensure_vertical_shell_thickness", ConfigOptionType::Enum);
            def.label = "Ensure vertical shell thickness".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Add solid infill near sloping surfaces to guarantee the vertical shell thickness (top+bottom solid layers)\nNone: No solid infill will be added anywhere. Caution: Use this option carefully if your model has sloped surfaces\nCritical Only: Avoid adding solid infill for walls\nModerate: Add solid infill for heavily sloping surfaces only\nAll: Add solid infill for all suitable sloping surfaces\nDefault value is All.".to_string();
            def.enum_keys_map = keys_map_ensure_vertical_shell_thickness();
            def.enum_values.push("none".to_string());
            def.enum_values.push("ensure_critical_only".to_string());
            def.enum_values.push("ensure_moderate".to_string());
            def.enum_values.push("ensure_all".to_string());
            def.enum_labels.push("None".to_string());
            def.enum_labels.push("Critical Only".to_string());
            def.enum_labels.push("Moderate".to_string());
            def.enum_labels.push("All".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionEnum::new(
                EnsureVerticalShellThickness::evstAll,
            )));
        }
        {
            let def = self.def.add("top_surface_pattern", ConfigOptionType::Enum);
            def.label = "Top surface pattern".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "This is the line pattern for top surface infill.".to_string();
            def.enum_keys_map = keys_map_infill_pattern();
            def.enum_values.push("monotonic".to_string());
            def.enum_values.push("monotonicline".to_string());
            def.enum_values.push("rectilinear".to_string());
            def.enum_values.push("alignedrectilinear".to_string());
            def.enum_values.push("concentric".to_string());
            def.enum_values.push("spiralinset".to_string());
            def.enum_values.push("hilbertcurve".to_string());
            def.enum_values.push("archimedeanchords".to_string());
            def.enum_values.push("octagramspiral".to_string());
            def.enum_labels.push("Monotonic".to_string());
            def.enum_labels.push("Monotonic line".to_string());
            def.enum_labels.push("Rectilinear".to_string());
            def.enum_labels.push("Aligned Rectilinear".to_string());
            def.enum_labels.push("Concentric".to_string());
            def.enum_labels.push("Spiral Inset".to_string());
            def.enum_labels.push("Hilbert Curve".to_string());
            def.enum_labels.push("Archimedean Chords".to_string());
            def.enum_labels.push("Octagram Spiral".to_string());
            def.set_default_value(Box::new(ConfigOptionEnum::new(
                InfillPattern::ipMonotonicLine,
            )));
        }
        {
            let def = self
                .def
                .add("top_surface_density", ConfigOptionType::Percent);
            def.label = "Top surface density".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Density of top surface layer. A value of 100% creates a fully solid, smooth top layer. Reducing this value results in a textured top surface, according to the chosen top surface pattern. A value of 0% will result in only the walls on the top layer being created. Intended for aesthetic or functional purposes, not to fix issues such as over-extrusion.".to_string();
            def.sidetext = "%".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2583
            def.max = 100_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2584
            def.set_default_value(Box::new(ConfigOptionPercent::new(100_f64)));
        }
        {
            let def = self
                .def
                .add("top_surface_expansion", ConfigOptionType::Float);
            def.label = "Top surface expansion".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Expands the top surfaces by this distance to connect distinct top surfaces and fill gaps.\nUseful for cases where the top surface is interrupted by a raised feature, such as text on a plane. Expanding it removes the holes beneath these features and creates a continuous path with a better finish for printing on top. The expansion is applied to the original top surface, before any other processing such as bridging or overhang detection.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(0_f64)));
        }
        {
            let def = self
                .def
                .add("top_surface_expansion_margin", ConfigOptionType::Float);
            def.label = "Top expansion wall margin".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Using “Top surface expansion” may cause a surface that did not previously touch the model's outer walls to now do so.\nThis can cause contraction marks (such as the hull line) on the outer walls.\nBy adding a small margin, this contraction will not occur directly on the walls, thereby preventing a visible mark.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2606
            def.max = 10_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(0_f64)));
        }
        {
            let def = self
                .def
                .add("top_surface_expansion_direction", ConfigOptionType::Enum);
            def.label = "Top expansion direction".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Direction in which the top surface expansion grows.\n - Inward grows into the holes and gaps left by features rising from the middle of a top surface.\n - Outward grows the outer edge of the surface, connecting surfaces separated by features that can divide a surface, such as a lattice pattern.\n - Inward and Outward does both.".to_string();
            def.enum_keys_map = keys_map_top_surface_expansion_direction();
            def.enum_values.push("inward_and_outward".to_string());
            def.enum_values.push("inward".to_string());
            def.enum_values.push("outward".to_string());
            def.enum_labels.push("Inward and Outward".to_string());
            def.enum_labels.push("Inward".to_string());
            def.enum_labels.push("Outward".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionEnum::new(
                TopSurfaceExpansionDirection::InwardAndOutward,
            )));
        }
        {
            let def_top_fill_pattern_enum_values =
                self.def.options["top_surface_pattern"].enum_values.clone();
            let def_top_fill_pattern_enum_labels =
                self.def.options["top_surface_pattern"].enum_labels.clone();
            let def = self
                .def
                .add("bottom_surface_pattern", ConfigOptionType::Enum);
            def.label = "Bottom surface pattern".to_string();
            def.category = "Strength".to_string();
            def.tooltip =
                "This is the line pattern of bottom surface infill, not including bridge infill."
                    .to_string();
            def.enum_keys_map = keys_map_infill_pattern();
            def.enum_values = def_top_fill_pattern_enum_values;
            def.enum_labels = def_top_fill_pattern_enum_labels;
            def.set_default_value(Box::new(ConfigOptionEnum::new(InfillPattern::ipMonotonic)));
        }
        {
            let def = self
                .def
                .add("bottom_surface_density", ConfigOptionType::Percent);
            def.label = "Bottom surface density".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Density of the bottom surface layer. Intended for aesthetic or functional purposes, not to fix issues such as over-extrusion.\nWARNING: Lowering this value may negatively affect bed adhesion.".to_string();
            def.sidetext = "%".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2643
            def.min = 10_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2644
            def.max = 100_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2645
            def.set_default_value(Box::new(ConfigOptionPercent::new(100_f64)));
        }
        {
            let def = self
                .def
                .add("top_surface_fill_order", ConfigOptionType::Enum);
            def.label = "Top surface fill order".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Direction in which top surfaces are filled when using a center-based pattern (Concentric, Spiral Inset, Archimedean Chords, Octagram Spiral).\nOutward starts at the center of the surface, so any excess material is pushed towards the edge where it is least visible. Inward starts at the edge and ends with the tight curves at the center.\nDefault uses shortest-path ordering, which may run in either direction.".to_string();
            def.enum_keys_map = keys_map_surface_fill_order();
            def.enum_values.push("default".to_string());
            def.enum_values.push("outward".to_string());
            def.enum_values.push("inward".to_string());
            def.enum_labels.push("Default".to_string());
            def.enum_labels.push("Outward".to_string());
            def.enum_labels.push("Inward".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionEnum::new(SurfaceFillOrder::Default)));
        }
        {
            let def_top_fill_order_enum_values = self.def.options["top_surface_fill_order"]
                .enum_values
                .clone();
            let def_top_fill_order_enum_labels = self.def.options["top_surface_fill_order"]
                .enum_labels
                .clone();
            let def = self
                .def
                .add("bottom_surface_fill_order", ConfigOptionType::Enum);
            def.label = "Bottom surface fill order".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Direction in which bottom surfaces are filled when using a center-based pattern (Concentric, Spiral Inset, Archimedean Chords, Octagram Spiral).\nInward starts each surface with the wider outer curves, which improves first layer adhesion on build plates where the tight curves at the center may not stick. Outward starts at the center, pushing any excess material towards the edge.\nDefault uses shortest-path ordering, which may run in either direction.".to_string();
            def.enum_keys_map = keys_map_surface_fill_order();
            def.enum_values = def_top_fill_order_enum_values;
            def.enum_labels = def_top_fill_order_enum_labels;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionEnum::new(SurfaceFillOrder::Default)));
        }
        {
            let def_top_fill_pattern_enum_values =
                self.def.options["top_surface_pattern"].enum_values.clone();
            let def_top_fill_pattern_enum_labels =
                self.def.options["top_surface_pattern"].enum_labels.clone();
            let def = self
                .def
                .add("internal_solid_infill_pattern", ConfigOptionType::Enum);
            def.label = "Internal solid infill pattern".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Line pattern of internal solid infill. if the detect narrow internal solid infill be enabled, the concentric pattern will be used for the small area.".to_string();
            def.enum_keys_map = keys_map_infill_pattern();
            def.enum_values = def_top_fill_pattern_enum_values;
            def.enum_labels = def_top_fill_pattern_enum_labels;
            def.set_default_value(Box::new(ConfigOptionEnum::new(InfillPattern::ipMonotonic)));
        }
        {
            let def = self
                .def
                .add("outer_wall_line_width", ConfigOptionType::FloatOrPercent);
            def.label = "Outer wall".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Line width of outer wall. If expressed as a %, it will be computed over the nozzle diameter.".to_string();
            def.sidetext = "mm or %".to_string();
            def.ratio_over = "nozzle_diameter".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2697
            def.max = 1000_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2698
            def.max_literal = 10_f64;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2700
            def.set_default_value(Box::new(ConfigOptionFloatOrPercent::new(0., false)));
        }
        {
            let def = self.def.add("outer_wall_speed", ConfigOptionType::Floats);
            def.label = "Outer wall".to_string();
            def.category = "Speed".to_string();
            def.tooltip = "This is the printing speed for the outer walls of parts. These are generally printed slower than inner walls for higher quality.".to_string();
            def.sidetext = "mm/s".to_string();
            def.min = 1_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2710
            v.values.push(60_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("small_perimeter_speed", ConfigOptionType::FloatsOrPercents);
            def.label = "Small perimeters".to_string();
            def.category = "Speed".to_string();
            def.tooltip = "This separate setting will affect the speed of perimeters having radius <= small_perimeter_threshold (usually holes). If expressed as percentage (for example: 80%) it will be calculated on the outer wall speed setting above.".to_string();
            def.sidetext = "mm/s or %".to_string();
            def.ratio_over = "outer_wall_speed".to_string();
            def.min = 1_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsOrPercentsNullable { values: Vec::new() };
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2723
            v.values.push(slic3r_config::config::FloatOrPercent {
                value: 50_f64,
                percent: true,
            });
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("small_perimeter_threshold", ConfigOptionType::Floats);
            def.label = "Small perimeters threshold".to_string();
            def.category = "Speed".to_string();
            def.tooltip =
                "This sets the threshold for small perimeter length. Default threshold is 0mm."
                    .to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            v.values.push(0_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "small_support_perimeter_speed",
                ConfigOptionType::FloatsOrPercents,
            );
            def.label = "Small support perimeters".to_string();
            def.category = "Speed".to_string();
            def.tooltip = "Same as \"Small perimeters\", but for supports. This separate setting will affect the speed of support for areas <= `small_support_perimeter_threshold`. If expressed as a percentage (for example: 80%), it will be calculated on the support or support interface speed setting above.".to_string();
            def.sidetext = "mm/s or %".to_string();
            def.ratio_over = "support_speed".to_string();
            def.min = 1_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloatsOrPercentsNullable { values: Vec::new() };
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2745
            v.values.push(slic3r_config::config::FloatOrPercent {
                value: 50_f64,
                percent: true,
            });
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "small_support_perimeter_threshold",
                ConfigOptionType::Floats,
            );
            def.label = "Small support perimeters threshold".to_string();
            def.category = "Speed".to_string();
            def.tooltip = "This sets the threshold for small support perimeter length. The default threshold is 0mm.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloatsNullable::new();
            v.values.push(0_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("wall_sequence", ConfigOptionType::Enum);
            def.label = "Walls printing order".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Print sequence of the internal (inner) and external (outer) walls.\n\nUse Inner/Outer for best overhangs. This is because the overhanging walls can adhere to a neighbouring perimeter while printing. However, this option results in slightly reduced surface quality as the external perimeter is deformed by being squashed to the internal perimeter.\n\nUse Inner/Outer/Inner for the best external surface finish and dimensional accuracy as the external wall is printed undisturbed from an internal perimeter. However, overhang performance will reduce as there is no internal perimeter to print the external wall against. This option requires a minimum of 3 walls to be effective as it prints the internal walls from the 3rd perimeter onwards first, then the external perimeter and, finally, the first internal perimeter. This option is recommended over the Outer/Inner option in most cases.\n\nUse Outer/Inner for the same external wall quality and dimensional accuracy benefits of Inner/Outer/Inner option. However, the Z seams will appear less consistent as the first extrusion of a new layer starts on a visible surface.".to_string();
            def.enum_keys_map = keys_map_wall_sequence();
            def.enum_values.push("inner wall/outer wall".to_string());
            def.enum_values.push("outer wall/inner wall".to_string());
            def.enum_values.push("inner-outer-inner wall".to_string());
            def.enum_labels.push("Inner/Outer".to_string());
            def.enum_labels.push("Outer/Inner".to_string());
            def.enum_labels.push("Inner/Outer/Inner".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionEnum::new(WallSequence::InnerOuter)));
        }
        {
            let def = self.def.add("is_infill_first", ConfigOptionType::Bool);
            def.label = "Print infill first".to_string();
            def.tooltip = "Order of wall/infill. When the tickbox is unchecked the walls are printed first, which works best in most cases.\n\nPrinting infill first may help with extreme overhangs as the walls have the neighbouring infill to adhere to. However, the infill will slightly push out the printed walls where it is attached to them, resulting in a worse external surface finish. It can also cause the infill to shine through the external surfaces of the part.".to_string();
            def.category = "Quality".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("wall_direction", ConfigOptionType::Enum);
            def.label = "Wall loop direction".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "The direction which the contour wall loops are extruded when looking down from the top.\nHoles are printed in the opposite direction to the contour to maintain alignment with layers whose contour polygons are incomplete and change direction, also partially forming the contour of a hole.\n\nThis option will be disabled if spiral vase mode is enabled.".to_string();
            def.enum_keys_map = keys_map_wall_direction();
            def.enum_values.push("ccw".to_string());
            def.enum_values.push("cw".to_string());
            def.enum_labels.push("Counter clockwise".to_string());
            def.enum_labels.push("Clockwise".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionEnum::new(
                WallDirection::CounterClockwise,
            )));
        }
        {
            let def = self.def.add("extruder", ConfigOptionType::Int);
            def.gui_type = GuiType::IEnumOpen;
            def.label = "Extruder".to_string();
            def.category = "Extruders".to_string();
            def.min = 0_f32;
            def.enum_labels.push("default".to_string());
            def.enum_labels.push("1".to_string());
            def.enum_labels.push("2".to_string());
            def.enum_labels.push("3".to_string());
            def.enum_labels.push("4".to_string());
            def.enum_labels.push("5".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
        }
        {
            let def = self
                .def
                .add("extruder_clearance_dist_to_rod", ConfigOptionType::Float);
            def.label = "Distance to rod".to_string();
            def.tooltip = "Horizontal distance of the nozzle tip to the rod's farther edge. Used for collision avoidance in by-object printing.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2821
            def.set_default_value(Box::new(ConfigOptionFloat::new(40_f64)));
        }
        {
            let def = self
                .def
                .add("extruder_clearance_height_to_rod", ConfigOptionType::Float);
            def.label = "Height to rod".to_string();
            def.tooltip = "Distance from the nozzle tip to the lower rod. Used for collision avoidance in by-object printing.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2829
            def.set_default_value(Box::new(ConfigOptionFloat::new(40_f64)));
        }
        {
            let def = self
                .def
                .add("extruder_clearance_height_to_lid", ConfigOptionType::Float);
            def.label = "Height to lid".to_string();
            def.tooltip = "Distance from the nozzle tip to the lid. Used for collision avoidance in by-object printing.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2838
            def.set_default_value(Box::new(ConfigOptionFloat::new(120_f64)));
        }
        {
            let def = self
                .def
                .add("extruder_clearance_radius", ConfigOptionType::Float);
            def.label = "Radius".to_string();
            def.tooltip = "Clearance radius around extruder: used for collision avoidance in by-object printing.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2846
            def.set_default_value(Box::new(ConfigOptionFloat::new(40_f64)));
        }
        {
            let def = self.def.add("nozzle_height", ConfigOptionType::Float);
            def.label = "Nozzle height".to_string();
            def.tooltip = "The height of nozzle tip.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2854
            def.set_default_value(Box::new(ConfigOptionFloat::new(2.5)));
        }
        {
            let def = self.def.add("bed_mesh_min", ConfigOptionType::Point);
            def.label = "Bed mesh min".to_string();
            def.tooltip = "This option sets the min point for the allowed bed mesh area. Due to the probe's XY offset, most printers are unable to probe the entire bed. To ensure the probe point does not go outside the bed area, the minimum and maximum points of the bed mesh should be set appropriately. OrcaSlicer ensures that adaptive_bed_mesh_min/adaptive_bed_mesh_max values do not exceed these min/max points. This information can usually be obtained from your printer manufacturer.\n\nThe default setting is (-99999, -99999), which means there are no limits, thus allowing probing across the entire bed.".to_string();
            def.sidetext = "mm".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2866
            def.set_default_value(Box::new(ConfigOptionPoint::new(Vec2d::new(
                -99999_f64, -99999_f64,
            ))));
        }
        {
            let def = self.def.add("bed_mesh_max", ConfigOptionType::Point);
            def.label = "Bed mesh max".to_string();
            def.tooltip = "This option sets the max point for the allowed bed mesh area. Due to the probe's XY offset, most printers are unable to probe the entire bed. To ensure the probe point does not go outside the bed area, the minimum and maximum points of the bed mesh should be set appropriately. OrcaSlicer ensures that adaptive_bed_mesh_min/adaptive_bed_mesh_max values do not exceed these min/max points. This information can usually be obtained from your printer manufacturer.\n\nThe default setting is (99999, 99999), which means there are no limits, thus allowing probing across the entire bed.".to_string();
            def.sidetext = "mm".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2878
            def.set_default_value(Box::new(ConfigOptionPoint::new(Vec2d::new(
                99999_f64, 99999_f64,
            ))));
        }
        {
            let def = self
                .def
                .add("bed_mesh_probe_distance", ConfigOptionType::Point);
            def.label = "Probe point distance".to_string();
            def.tooltip = "This option sets the preferred distance between probe points (grid size) for the X and Y directions, with the default being 50mm for both X and Y.".to_string();
            def.min = 0_f32;
            def.sidetext = "mm".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2887
            def.set_default_value(Box::new(ConfigOptionPoint::new(Vec2d::new(50_f64, 50_f64))));
        }
        {
            let def = self
                .def
                .add("adaptive_bed_mesh_margin", ConfigOptionType::Float);
            def.label = "Mesh margin".to_string();
            def.tooltip = "This option determines the additional distance by which the adaptive bed mesh area should be expanded in the XY directions.".to_string();
            def.sidetext = "mm".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(0_f64)));
        }
        {
            let def = self.def.add("grab_length", ConfigOptionType::Floats);
            def.label = "Grab length".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            let mut v = ConfigOptionFloats::new();
            v.values.push(0_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("extruder_colour", ConfigOptionType::Strings);
            def.label = "Extruder Color".to_string();
            def.tooltip = "Only used as a visual help on UI.".to_string();
            def.gui_type = GuiType::Color;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionStrings::new();
            v.values.push("".to_string());
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("extruder_offset", ConfigOptionType::Points);
            def.label = "Extruder offset".to_string();
            def.sidetext = "mm".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionPoints::new();
            v.values.push(Vec2d::new(0_f64, 0_f64));
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_flow_ratio", ConfigOptionType::Floats);
            def.label = "Flow ratio".to_string();
            def.tooltip = "The material may have volumetric change after switching between molten and crystalline states. This setting changes all extrusion flow of this filament in G-code proportionally. The recommended value range is between 0.95 and 1.05. You may be able to tune this value to get a nice flat surface if there is slight overflow or underflow.".to_string();
            def.min = 0_f32;
            def.max = 2_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2931
            v.values.push(1.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("print_flow_ratio", ConfigOptionType::Float);
            def.label = "Flow ratio".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "The material may have volumetric change after switching between molten and crystalline states. This setting changes all extrusion flow of this filament in G-code proportionally. The recommended value range is between 0.95 and 1.05. You may be able to tune this value to get a nice flat surface if there is slight overflow or underflow.\n\nThe final object flow ratio is this value multiplied by the filament flow ratio.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2942
            def.max = 2.;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2943
            def.min = 0.01;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2944
            def.set_default_value(Box::new(ConfigOptionFloat::new(1.)));
        }
        {
            let def = self
                .def
                .add("enable_pressure_advance", ConfigOptionType::Bools);
            def.label = "Enable pressure advance".to_string();
            def.tooltip = "Enable pressure advance, auto calibration result will be overwritten once enabled.".to_string();
            let mut v = ConfigOptionBools::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("pressure_advance", ConfigOptionType::Floats);
            def.label = "Pressure advance".to_string();
            def.tooltip =
                "Pressure advance (Klipper) AKA Linear advance factor (Marlin).".to_string();
            def.max = 2_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2956
            v.values.push(0.02);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("adaptive_pressure_advance", ConfigOptionType::Bools);
            def.label = "Enable adaptive pressure advance (beta)".to_string();
            def.tooltip = "With increasing print speeds (and hence increasing volumetric flow through the nozzle) and increasing accelerations, it has been observed that the effective PA value typically decreases. This means that a single PA value is not always 100% optimal for all features and a compromise value is usually used that does not cause too much bulging on features with lower flow speed and accelerations while also not causing gaps on faster features.\n\nThis feature aims to address this limitation by modeling the response of your printer's extrusion system depending on the volumetric flow speed and acceleration it is printing at. Internally, it generates a fitted model that can extrapolate the needed pressure advance for any given volumetric flow speed and acceleration, which is then emitted to the printer depending on the current print conditions.\n\nWhen enabled, the pressure advance value above is overridden. However, a reasonable default value above is strongly recommended to act as a fallback and for when tool changing.\n\n".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionBools::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("adaptive_pressure_advance_model", ConfigOptionType::Strings);
            def.label = "Adaptive pressure advance measurements (beta)".to_string();
            def.tooltip = "Add sets of pressure advance (PA) values, the volumetric flow speeds and accelerations they were measured at, separated by a comma. One set of values per line. For example\n0.04,3.96,3000\n0.033,3.96,10000\n0.029,7.91,3000\n0.026,7.91,10000\n\nHow to calibrate:\n1. Run the pressure advance test for at least 3 speeds per acceleration value. It is recommended that the test is run for at least the speed of the external perimeters, the speed of the internal perimeters and the fastest feature print speed in your profile (usually its the sparse or solid infill). Then run them for the same speeds for the slowest and fastest print accelerations, and no faster than the recommended maximum acceleration as given by the Klipper input shaper\n2. Take note of the optimal PA value for each volumetric flow speed and acceleration. You can find the flow number by selecting flow from the color scheme drop down and move the horizontal slider over the PA pattern lines. The number should be visible at the bottom of the page. The ideal PA value should be decreasing the higher the volumetric flow is. If it is not, confirm that your extruder is functioning correctly. The slower and with less acceleration you print, the larger the range of acceptable PA values. If no difference is visible, use the PA value from the faster test\n3. Enter the triplets of PA values, Flow and Accelerations in the text box here and save your filament profile.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.multiline = true;
            def.full_width = true;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:2996
            def.height = 15;
            let mut v = ConfigOptionStrings::new();
            v.values.push("0,0,0\n0,0,0".to_string());
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "adaptive_pressure_advance_overhangs",
                ConfigOptionType::Bools,
            );
            def.label = "Enable adaptive pressure advance within features (beta)".to_string();
            def.tooltip = "Enable adaptive PA whenever there are flow changes in a feature, such as line width changes in a corner or overhangs.\n\nNot compatible with Prusa printers as they pause to process PA changes, which causes delays and defects.\n\nThis is an experimental option, as if the PA profile is not set accurately, it will cause uniformity issues.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionBools::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "adaptive_pressure_advance_bridges",
                ConfigOptionType::Floats,
            );
            def.label = "Static pressure advance for bridges".to_string();
            def.tooltip = "Static pressure advance value for bridges. Set to 0 to apply the same pressure advance as equivalent walls (using adaptive settings if enabled).\n\nA lower PA value when printing bridges helps reduce the appearance of slight under-extrusion immediately after bridges. This is caused by the pressure drop in the nozzle when printing in the air and a lower PA helps counteract this.".to_string();
            def.max = 2_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3016
            v.values.push(0.0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("line_width", ConfigOptionType::FloatOrPercent);
            def.label = "Default".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Default line width if other line widths are set to 0. If expressed as a %, it will be computed over the nozzle diameter.".to_string();
            def.sidetext = "mm or %".to_string();
            def.ratio_over = "nozzle_diameter".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3025
            def.max = 1000_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3026
            def.max_literal = 10_f64;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloatOrPercent::new(0_f64, false)));
        }
        {
            let def = self
                .def
                .add("reduce_fan_stop_start_freq", ConfigOptionType::Bools);
            def.label = "Keep fan always on".to_string();
            def.tooltip = "Enabling this setting means that part cooling fan will never stop entirely and will instead run at least at minimum speed to reduce the frequency of starting and stopping.".to_string();
            let mut v = ConfigOptionBools::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("dont_slow_down_outer_wall", ConfigOptionType::Bools);
            def.label = "Don't slow down outer walls".to_string();
            def.tooltip = "If enabled, this setting will ensure external perimeters are not slowed down to meet the minimum layer time. This is particularly helpful in the below scenarios:\n1. To avoid changes in shine when printing glossy filaments\n2. To avoid changes in external wall speed which may create slight wall artifacts that appear like Z banding\n3. To avoid printing at speeds which cause VFAs (fine artifacts) on the external walls".to_string();
            let mut v = ConfigOptionBools::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("fan_cooling_layer_time", ConfigOptionType::Floats);
            def.label = "Layer time".to_string();
            def.tooltip = "The part cooling fan will be enabled for layers where the estimated time is shorter than this value. Fan speed is interpolated between the minimum and maximum fan speeds according to layer printing time.".to_string();
            def.sidetext = "s".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3049
            def.max = 1000_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3051
            v.values.push(60.0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("default_filament_colour", ConfigOptionType::Strings);
            def.label = "Default color".to_string();
            def.tooltip = "Default filament color.\nRight click to reset value to system default."
                .to_string();
            def.gui_type = GuiType::Color;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionStrings::new();
            v.values.push("".to_string());
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("filament_colour", ConfigOptionType::Strings);
            def.label = "Color".to_string();
            def.tooltip = "Only used as a visual help on UI.".to_string();
            def.gui_type = GuiType::Color;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionStrings::new();
            v.values.push("#F2754E".to_string());
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("filament_notes", ConfigOptionType::Strings);
            def.label = "Filament notes".to_string();
            def.tooltip = "You can put your notes regarding the filament here.".to_string();
            def.multiline = true;
            def.full_width = true;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3074
            def.height = 13;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionStrings::new();
            v.values.push("".to_string());
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_multi_colour", ConfigOptionType::Strings);
            let mut v = ConfigOptionStrings::new();
            v.values.push("".to_string());
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_colour_type", ConfigOptionType::Strings);
            let mut v = ConfigOptionStrings::new();
            v.values.push("1".to_string());
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("required_nozzle_HRC", ConfigOptionType::Ints);
            def.label = "Required nozzle HRC".to_string();
            def.tooltip = "Minimum HRC of nozzle required to print the filament. A value of 0 means no checking of the nozzle's HRC.".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3091
            def.max = 500_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            let mut v = ConfigOptionInts::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("filament_map", ConfigOptionType::Ints);
            def.label = "Filament map to extruder".to_string();
            def.tooltip = "Filament map to extruder.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            let mut v = ConfigOptionInts::new();
            v.values.push(1);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("filament_map_2", ConfigOptionType::Ints);
            def.label = "Filament map plus for multi nozzle".to_string();
            def.tooltip = "Filament map to the index identified by extruder and nozzle_volume_type"
                .to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            let mut v = ConfigOptionInts::new();
            v.values.push(1);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("filament_volume_map", ConfigOptionType::Ints);
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            let mut v = ConfigOptionInts::new();
            v.values.push(NozzleVolumeType::nvtStandard as i32);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("physical_extruder_map", ConfigOptionType::Ints);
            def.label = "Map the logical extruder to physical extruder".to_string();
            def.tooltip = "Map the logical extruder to physical extruder.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            let mut v = ConfigOptionInts::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("filament_map_mode", ConfigOptionType::Enum);
            def.label = "filament mapping mode".to_string();
            def.tooltip = "Filament mapping mode used as plate param.".to_string();
            def.enum_keys_map = keys_map_filament_map_mode();
            def.enum_values.push("Auto For Flush".to_string());
            def.enum_values.push("Auto For Match".to_string());
            def.enum_values.push("Manual".to_string());
            def.enum_values.push("Nozzle Manual".to_string());
            def.enum_values.push("Default".to_string());
            def.enum_labels.push("Auto For Flush".to_string());
            def.enum_labels.push("Auto For Match".to_string());
            def.enum_labels.push("Manual".to_string());
            def.enum_labels.push("Nozzle Manual".to_string());
            def.enum_labels.push("Default".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionEnum::new(
                FilamentMapMode::fmmAutoForFlush,
            )));
        }
        {
            let def = self.def.add("filament_nozzle_map", ConfigOptionType::Ints);
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            let mut v = ConfigOptionInts::new();
            v.values.push(1);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("filament_flush_temp", ConfigOptionType::Ints);
            def.label = "Flush temperature".to_string();
            def.tooltip = "Temperature when flushing filament. 0 indicates the upper bound of the recommended nozzle temperature range.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            def.min = 0_f32;
            def.max = max_temp;
            def.sidetext = "\u{2103}".to_string();
            let mut v = ConfigOptionIntsNullable::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_flush_temp_fast", ConfigOptionType::Ints);
            def.label = "Flush temperature".to_string();
            def.tooltip = "Flush temperature used in fast purge mode.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            def.min = 0_f32;
            def.max = max_temp;
            def.sidetext = "\u{2103}".to_string();
            let mut v = ConfigOptionIntsNullable::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_flush_volumetric_speed", ConfigOptionType::Floats);
            def.label = "Flush volumetric speed".to_string();
            def.tooltip =
                "Volumetric speed when flushing filament. 0 indicates the max volumetric speed."
                    .to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3184
            def.max = 200_f32;
            def.sidetext = "mm³/s".to_string();
            let mut v = ConfigOptionFloatsNullable::new();
            v.values.push(0_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_max_volumetric_speed", ConfigOptionType::Floats);
            def.label = "Max volumetric speed".to_string();
            def.tooltip = "This setting is the volume of filament that can be melted and extruded per second. Printing speed is limited by max volumetric speed, in case of too high and unreasonable speed setting. This value cannot be zero.".to_string();
            def.sidetext = "mm³/s".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3194
            v.values.push(2.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("machine_load_filament_time", ConfigOptionType::Float);
            def.label = "Filament load time".to_string();
            def.tooltip = "Time to load new filament when switch filament. It's usually applicable for single-extruder multi-material machines. For tool changers or multi-tool machines, it's typically 0. For statistics only.".to_string();
            def.sidetext = "s".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3203
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.0)));
        }
        {
            let def = self
                .def
                .add("machine_unload_filament_time", ConfigOptionType::Float);
            def.label = "Filament unload time".to_string();
            def.tooltip = "Time to unload old filament when switch filament. It's usually applicable for single-extruder multi-material machines. For tool changers or multi-tool machines, it's typically 0. For statistics only.".to_string();
            def.sidetext = "s".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3212
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.0)));
        }
        {
            let def = self
                .def
                .add("machine_tool_change_time", ConfigOptionType::Float);
            def.label = "Tool change time".to_string();
            def.tooltip = "Time taken to switch tools. It's usually applicable for tool changers or multi-tool machines. For single-extruder multi-material machines, it's typically 0. For statistics only.".to_string();
            def.sidetext = "s".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3221
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.)));
        }
        {
            let def = self
                .def
                .add("support_object_skip_flush", ConfigOptionType::Bool);
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("bed_temperature_formula", ConfigOptionType::Enum);
            def.label = "Bed temperature type".to_string();
            def.tooltip = "This option determines how the bed temperature is set during slicing: based on the temperature of the first filament or the highest temperature of the printed filaments.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.enum_keys_map = keys_map_bed_temp_formula();
            def.enum_values.push("by_first_filament".to_string());
            def.enum_values.push("by_highest_temp".to_string());
            def.enum_labels.push("By First filament".to_string());
            def.enum_labels.push("By Highest Temp".to_string());
            def.set_default_value(Box::new(ConfigOptionEnum::new(
                BedTempFormula::btfHighestTemp,
            )));
        }
        {
            let def = self.def.add("nozzle_flush_dataset", ConfigOptionType::Ints);
            def.nullable = true;
            let mut v = ConfigOptionIntsNullable::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("filament_diameter", ConfigOptionType::Floats);
            def.label = "Diameter".to_string();
            def.tooltip = "Filament diameter is used to calculate extrusion variables in G-code, so it is important that this is accurate and precise.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3247
            v.values.push(1.75);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("pellet_flow_coefficient", ConfigOptionType::Floats);
            def.label = "Pellet flow coefficient".to_string();
            def.tooltip = "Pellet flow coefficient is empirically derived and allows for volume calculation for pellet printers.\n\nInternally it is converted to filament_diameter. All other volume calculations remain the same.\n\nfilament_diameter = sqrt( (4 * pellet_flow_coefficient) / PI )".to_string();
            def.min = 0_f32;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3279
            v.values.push(0.4157);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "filament_adaptive_volumetric_speed",
                ConfigOptionType::Bools,
            );
            def.label = "Adaptive volumetric speed".to_string();
            def.tooltip = "When enabled, the extrusion flow is limited by the smaller of the fitted value (calculated from line width and layer height) and the user-defined maximum flow. When disabled, only the user-defined maximum flow is applied.\n\nNote: Experimental and incomplete feature imported from BBS. Functional for some profiles that already have the variable saved.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            def.nullable = true;
            let mut v = ConfigOptionBoolsNullable::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("volumetric_speed_coefficients", ConfigOptionType::Strings);
            def.label = "Max volumetric speed multinomial coefficients".to_string();
            let mut v = ConfigOptionStrings::new();
            v.values.push("".to_string());
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("filament_shrink", ConfigOptionType::Percents);
            def.label = "Shrinkage (XY)".to_string();
            def.tooltip = "Enter the shrinkage percentage that the filament will get after cooling (94% if you measure 94mm instead of 100mm). The part will be scaled in XY to compensate. For multi-material prints, ensure filament shrinkage matches across all used filaments.\nBe sure to allow enough space between objects, as this compensation is done after the checks.".to_string();
            def.sidetext = "%".to_string();
            def.ratio_over = "".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3303
            def.min = 50_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3304
            def.max = 150_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionPercents { values: Vec::new() };
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3306
            v.values.push(100_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "filament_shrinkage_compensation_z",
                ConfigOptionType::Percents,
            );
            def.label = "Shrinkage (Z)".to_string();
            def.tooltip = "Enter the shrinkage percentage that the filament will get after cooling (94% if you measure 94mm instead of 100mm). The part will be scaled in Z to compensate.".to_string();
            def.sidetext = "%".to_string();
            def.ratio_over = "".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3315
            def.min = 50_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3316
            def.max = 150_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionPercents { values: Vec::new() };
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3318
            v.values.push(100_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_adhesiveness_category", ConfigOptionType::Ints);
            def.label = "Adhesiveness Category".to_string();
            def.tooltip = "Filament category.".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            let mut v = ConfigOptionInts::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_loading_speed", ConfigOptionType::Floats);
            def.label = "Loading speed".to_string();
            def.tooltip = "Speed used for loading the filament on the wipe tower.".to_string();
            def.sidetext = "mm/s".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3333
            v.values.push(28.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_loading_speed_start", ConfigOptionType::Floats);
            def.label = "Loading speed at the start".to_string();
            def.tooltip = "Speed used at the very beginning of loading phase.".to_string();
            def.sidetext = "mm/s".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3341
            v.values.push(3.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_unloading_speed", ConfigOptionType::Floats);
            def.label = "Unloading speed".to_string();
            def.tooltip = "Speed used for unloading the filament on the wipe tower (does not affect initial part of unloading just after ramming).".to_string();
            def.sidetext = "mm/s".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3349
            v.values.push(90.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_unloading_speed_start", ConfigOptionType::Floats);
            def.label = "Unloading speed at the start".to_string();
            def.tooltip =
                "Speed used for unloading the tip of the filament immediately after ramming."
                    .to_string();
            def.sidetext = "mm/s".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3357
            v.values.push(100.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_toolchange_delay", ConfigOptionType::Floats);
            def.label = "Delay after unloading".to_string();
            def.tooltip = "Time to wait after the filament is unloaded. May help to get reliable tool changes with flexible materials that may need more time to shrink to original dimensions.".to_string();
            def.sidetext = "s".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3367
            v.values.push(0.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_cooling_moves", ConfigOptionType::Ints);
            def.label = "Number of cooling moves".to_string();
            def.tooltip = "Filament is cooled by being moved back and forth in the cooling tubes. Specify desired number of these moves.".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3374
            def.max = 20_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionInts::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3376
            v.values.push(4);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_stamping_loading_speed", ConfigOptionType::Floats);
            def.label = "Stamping loading speed".to_string();
            def.tooltip = "Speed used for stamping.".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3383
            v.values.push(0.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_stamping_distance", ConfigOptionType::Floats);
            def.label =
                "Stamping distance measured from the center of the cooling tube".to_string();
            def.tooltip = "If set to non-zero value, filament is moved toward the nozzle between the individual cooling moves (\"stamping\"). This option configures how long this movement should be before the filament is retracted again.".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3391
            v.values.push(0.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_cooling_initial_speed", ConfigOptionType::Floats);
            def.label = "Speed of the first cooling move".to_string();
            def.tooltip =
                "Cooling moves are gradually accelerating beginning at this speed.".to_string();
            def.sidetext = "mm/s".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3399
            v.values.push(2.2);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "filament_minimal_purge_on_wipe_tower",
                ConfigOptionType::Floats,
            );
            def.label = "Minimal purge on wipe tower".to_string();
            def.tooltip = "After a tool change, the exact position of the newly loaded filament inside the nozzle may not be known, and the filament pressure is likely not yet stable. Before purging the print head into an infill or a sacrificial object, Orca Slicer will always prime this amount of material into the wipe tower to produce successive infill or sacrificial object extrusions reliably.".to_string();
            def.sidetext = "mm³".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3410
            v.values.push(15.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_cooling_before_tower", ConfigOptionType::Floats);
            def.label = "Wipe tower cooling".to_string();
            def.tooltip = "Temperature drop before entering the wipe tower.".to_string();
            def.sidetext = "\u{2103}".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3418
            v.values.push(10.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "filament_tower_interface_pre_extrusion_dist",
                ConfigOptionType::Floats,
            );
            def.label = "Interface layer pre-extrusion distance".to_string();
            def.tooltip = "Pre-extrusion distance for prime tower interface layer (where different materials meet).".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3426
            v.values.push(10.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "filament_tower_interface_pre_extrusion_length",
                ConfigOptionType::Floats,
            );
            def.label = "Interface layer pre-extrusion length".to_string();
            def.tooltip = "Pre-extrusion length for prime tower interface layer (where different materials meet).".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3434
            v.values.push(0.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_tower_ironing_area", ConfigOptionType::Floats);
            def.label = "Tower ironing area".to_string();
            def.tooltip =
                "Ironing area for prime tower interface layer (where different materials meet)."
                    .to_string();
            def.sidetext = "mm²".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3442
            v.values.push(4.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "filament_tower_interface_purge_volume",
                ConfigOptionType::Floats,
            );
            def.label = "Interface layer purge length".to_string();
            def.tooltip =
                "Purge length for prime tower interface layer (where different materials meet)."
                    .to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3450
            v.values.push(20.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "filament_tower_interface_print_temp",
                ConfigOptionType::Ints,
            );
            def.label = "Interface layer print temperature".to_string();
            def.tooltip = "Print temperature for prime tower interface layer (where different materials meet). If set to -1, use max recommended nozzle temperature.".to_string();
            def.sidetext = "\u{2103}".to_string();
            def.min = -1_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionInts::new();
            v.values.push(-1);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_cooling_final_speed", ConfigOptionType::Floats);
            def.label = "Speed of the last cooling move".to_string();
            def.tooltip =
                "Cooling moves are gradually accelerating towards this speed.".to_string();
            def.sidetext = "mm/s".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3466
            v.values.push(3.4);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_ramming_parameters", ConfigOptionType::Strings);
            def.label = "Ramming parameters".to_string();
            def.tooltip =
                "This string is edited by RammingDialog and contains ramming specific parameters."
                    .to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionStrings::new();
            v.values.push("120 100 6.6 6.8 7.2 7.6 7.9 8.2 8.7 9.4 9.9 10.0| 0.05 6.6 0.45 6.8 0.95 7.8 1.45 8.3 1.95 9.7 2.45 10 2.95 7.6 3.45 7.6 3.95 7.6 4.45 7.6 4.95 7.6".to_string());
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_multitool_ramming", ConfigOptionType::Bools);
            def.label = "Enable ramming for multi-tool setups".to_string();
            def.tooltip = "Perform ramming when using multi-tool printer (i.e. when the 'Single Extruder Multimaterial' in Printer Settings is unchecked). When checked, a small amount of filament is rapidly extruded on the wipe tower just before the tool change. This option is only used when the wipe tower is enabled.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionBools::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "filament_multitool_ramming_volume",
                ConfigOptionType::Floats,
            );
            def.label = "Multi-tool ramming volume".to_string();
            def.tooltip = "The volume to be rammed before the tool change.".to_string();
            def.sidetext = "mm³".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3489
            v.values.push(10.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_multitool_ramming_flow", ConfigOptionType::Floats);
            def.label = "Multi-tool ramming flow".to_string();
            def.tooltip = "Flow used for ramming the filament before the tool change.".to_string();
            def.sidetext = "mm³/s".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3497
            v.values.push(10.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("filament_density", ConfigOptionType::Floats);
            def.label = "Density".to_string();
            def.tooltip = "Filament density, for statistical purposes only.".to_string();
            def.sidetext = "g/cm³".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3505
            v.values.push(0.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("filament_type", ConfigOptionType::Strings);
            def.label = "Type".to_string();
            def.tooltip = "Filament material type".to_string();
            def.gui_type = GuiType::FEnumOpen;
            def.gui_flags = "show_value".to_string();
            // The enum values come from the shared material-type database rather than from a
            // literal list here (`PrintConfig.cpp:3513-3516`).
            // FIDELITY(port): `MaterialType::all()`'s own table order, which this loop walks
            // and pushes positionally; it reaches output as `enum_values`' order, which the
            // GUI's combo box and this unit's proof both read in order.
            for filament in slic3r_materialtype::material_type::all() {
                // `filament.name` (`MaterialType.hpp:9`) is field 0 of the ported tuple struct.
                def.enum_values.push(filament.0.to_string());
            }
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            let mut v = ConfigOptionStrings::new();
            v.values.push("PLA".to_string());
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("filament_soluble", ConfigOptionType::Bools);
            def.label = "Soluble material".to_string();
            def.tooltip =
                "Soluble material is commonly used to print supports and support interfaces."
                    .to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionBools::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_change_length", ConfigOptionType::Floats);
            def.label = "Filament ramming length".to_string();
            def.tooltip = "When changing the extruder, it is recommended to extrude a certain length of filament from the original extruder. This helps minimize nozzle oozing.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3533
            v.values.push(10_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("filament_is_support", ConfigOptionType::Bools);
            def.label = "Support material".to_string();
            def.tooltip =
                "Support material is commonly used to print supports and support interfaces."
                    .to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionBools::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("filament_is_mixed", ConfigOptionType::Bools);
            def.label = "Is mixed filament".to_string();
            def.tooltip = "Whether this filament slot is a mixed filament composed of multiple physical filaments".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            let mut v = ConfigOptionBools::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_mixed_components", ConfigOptionType::Strings);
            def.label = "Mixed filament components".to_string();
            def.tooltip =
                "Comma-separated 1-based indices of component filaments, e.g. \"1,3\"".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            let mut v = ConfigOptionStrings::new();
            v.values.push("".to_string());
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_mixed_sublayer_ratios", ConfigOptionType::Strings);
            def.label = "Mixed filament sublayer ratios".to_string();
            def.tooltip =
                "Comma-separated ratio values summing to 1.0, e.g. \"0.7,0.3\"".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            let mut v = ConfigOptionStrings::new();
            v.values.push("".to_string());
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_mixed_gradient", ConfigOptionType::Bools);
            def.label = "Mixed filament gradient".to_string();
            def.tooltip = "Enable Z-direction gradient mode for mixed filament sub-layers. When enabled, the sub-layer ratios vary linearly across layers.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            let mut v = ConfigOptionBools::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_mixed_gradient_range", ConfigOptionType::Strings);
            def.label = "Mixed filament gradient range".to_string();
            def.tooltip = "Start and end ratios for the first component in gradient mode. Comma-separated pair, e.g. \"0.10,0.90\" means 10% to 90%.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            let mut v = ConfigOptionStrings::new();
            v.values.push("".to_string());
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_mixed_gradient_curve", ConfigOptionType::Strings);
            def.label = "Mixed filament gradient curve".to_string();
            def.tooltip = "Optional Photoshop-style custom curve mapping Z progress to the first component ratio. Encoded as pipe-separated control points, either \"x,y\" (legacy) or \"x,y,m_in,m_out\" when a tangent override is needed (empty token or \"nan\" means use PCHIP default). x in [0,1]; y is clamped to the configured ratio range, e.g. \"0,0.15|0.5,0.50|1,0.85\". When empty, the linear gradient_range is used instead.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            let mut v = ConfigOptionStrings::new();
            v.values.push("".to_string());
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_mixed_gradient_per_part", ConfigOptionType::Bools);
            def.label = "Mixed filament per-part gradient".to_string();
            def.tooltip = "When gradient mode is enabled, apply the gradient to each part of an assembly independently rather than treating the whole assembly as one Z range.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            let mut v = ConfigOptionBools::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("filament_printable", ConfigOptionType::Ints);
            def.label = "Filament printable".to_string();
            def.tooltip = "The filament is printable in extruder.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            let mut v = ConfigOptionInts::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3605
            v.values.push(3);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_extruder_compatibility", ConfigOptionType::Ints);
            def.label = "Filament-extruder compatibility".to_string();
            def.tooltip = "A single 32-bit int encoding the compatibility level of a filament across all extruders (up to 10). Every 3 bits represent one extruder (bits [3*i, 3*i+2] for extruder i). 0: printable, 1: error, 2: critical warning, 3: warning, 4-7: reserved.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            let mut v = ConfigOptionInts::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("temperature_vitrification", ConfigOptionType::Ints);
            def.label = "Softening temperature".to_string();
            def.tooltip = "The material softens at this temperature, so when the bed temperature is equal to or greater than this, it's highly recommended to open the front door and/or remove the upper glass to avoid clogs.".to_string();
            def.sidetext = "\u{2103}".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            let mut v = ConfigOptionInts::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3625
            v.values.push(100);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("filament_cost", ConfigOptionType::Floats);
            def.label = "Price".to_string();
            def.tooltip = "Filament price, for statistical purposes only.".to_string();
            def.sidetext = "money/kg".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3633
            v.values.push(0.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_settings_id", ConfigOptionType::Strings);
            let mut v = ConfigOptionStrings::new();
            v.values.push("".to_string());
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("filament_ids", ConfigOptionType::Strings);
            def.set_default_value(Box::new(ConfigOptionStrings::new()));
            def.cli = NOCLI.to_string();
        }
        {
            let def = self.def.add("filament_vendor", ConfigOptionType::Strings);
            def.label = "Vendor".to_string();
            def.tooltip = "Vendor of filament. For show only.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionStrings::new();
            v.values.push("(Undefined)".to_string());
            def.set_default_value(Box::new(v));
            def.cli = NOCLI.to_string();
        }
        {
            let def = self.def.add("infill_direction", ConfigOptionType::Float);
            def.label = "Sparse infill direction".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "This is the angle for sparse infill pattern, which controls the start or main direction of lines.".to_string();
            def.sidetext = "°".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3657
            def.max = 360_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3659
            def.set_default_value(Box::new(ConfigOptionFloat::new(45_f64)));
        }
        {
            let def = self
                .def
                .add("solid_infill_direction", ConfigOptionType::Float);
            def.label = "Solid infill direction".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Angle for solid infill pattern, which controls the start or main direction of line.".to_string();
            def.sidetext = "°".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3667
            def.max = 360_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3669
            def.set_default_value(Box::new(ConfigOptionFloat::new(45_f64)));
        }
        {
            let def = self.def.add("top_layer_direction", ConfigOptionType::Float);
            def.label = "Top layer direction".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Fixed angle for the top solid infill and ironing lines.\nSet to -1 to follow the default solid infill direction.".to_string();
            def.sidetext = "°".to_string();
            def.min = -1_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3677
            def.max = 360_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            def.set_default_value(Box::new(ConfigOptionFloat::new(-1_f64)));
        }
        {
            let def = self
                .def
                .add("bottom_layer_direction", ConfigOptionType::Float);
            def.label = "Bottom layer direction".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Fixed angle for the bottom solid infill lines.\nSet to -1 to follow the default solid infill direction.".to_string();
            def.sidetext = "°".to_string();
            def.min = -1_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3687
            def.max = 360_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            def.set_default_value(Box::new(ConfigOptionFloat::new(-1_f64)));
        }
        {
            let def = self
                .def
                .add("sparse_infill_density", ConfigOptionType::Percent);
            def.label = "Sparse infill density".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Density of internal sparse infill, 100% turns all sparse infill into solid infill and internal solid infill pattern will be used.".to_string();
            def.sidetext = "%".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3698
            def.max = 100_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3699
            def.set_default_value(Box::new(ConfigOptionPercent::new(20_f64)));
        }
        {
            let def = self
                .def
                .add("align_infill_direction_to_model", ConfigOptionType::Bool);
            def.label = "Align directions to model".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Aligns infill, bridge, ironing, and top/bottom surface directions to follow the model's orientation on the build plate.\nWhen enabled, these directions rotate together with the model so the printed features keep their intended orientation relative to the part, preserving optimal strength and surface characteristics regardless of how the model is placed.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("extra_solid_infills", ConfigOptionType::String);
            def.label = "Insert solid layers".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Insert solid infill at specific layers. Use N to insert every Nth layer, N#K to insert K consecutive solid layers every N layers (K is optional, e.g. '5#' equals '5#1'), or a comma-separated list (e.g. 1,7,9) to insert at explicit layers. Layers are 1-based.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
        }
        {
            let def = self.def.add("fill_multiline", ConfigOptionType::Int);
            def.label = "Fill Multiline".to_string();
            def.category = "Strength".to_string();
            def.tooltip =
                "Using multiple lines for the infill pattern, if supported by infill pattern."
                    .to_string();
            def.min = 1_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3724
            def.max = 10_f32;
            def.set_default_value(Box::new(ConfigOptionInt::new(1)));
        }
        {
            let def = self.def.add("adaptive_infill_mode", ConfigOptionType::Enum);
            def.label = "Adaptive density (experimental)".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Grades the Gyroid and TPMS infill inside the object: its cells grow from the surface of the object towards its center. The sparse infill density is used at the surface and the interior density at the center.\nDistance warp, Smooth blend and Stepped shells follow the distance to the nearest surface, including the top and bottom, with the interior density at the point farthest from it:\n - Distance warp: one continuous pattern, stretched and sheared where the distance changes across directions, as in plates and long parts.\n - Smooth blend: the patterns of neighbouring densities blended into each other, with small loops where they meet.\n - Stepped shells: shells of the regular pattern at densities about 1.5 times apart, their lines joined along the shell boundaries.\n - Lobes: follows the 3D shape of the object, including its top and bottom. Every lobe, a part joined to the rest by a narrower neck, is graded towards its own center.\n - Normal Z, Y or X: follows the sections of the object normal to that axis, so the density does not change along it.".to_string();
            def.enum_keys_map = keys_map_adaptive_infill_mode();
            def.enum_values.push("disabled".to_string());
            def.enum_values.push("distance_warp".to_string());
            def.enum_values.push("smooth_blend".to_string());
            def.enum_values.push("stepped_shells".to_string());
            def.enum_values.push("lobes".to_string());
            def.enum_values.push("normal_z".to_string());
            def.enum_values.push("normal_y".to_string());
            def.enum_values.push("normal_x".to_string());
            def.enum_labels.push("Disabled".to_string());
            def.enum_labels.push("Distance warp".to_string());
            def.enum_labels.push("Smooth blend".to_string());
            def.enum_labels.push("Stepped shells".to_string());
            def.enum_labels.push("Lobes".to_string());
            def.enum_labels.push("Normal Z".to_string());
            def.enum_labels.push("Normal Y".to_string());
            def.enum_labels.push("Normal X".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionEnum::new(
                AdaptiveInfillMode::Disabled,
            )));
        }
        {
            let def = self.def.add(
                "adaptive_infill_interior_density",
                ConfigOptionType::Percent,
            );
            def.label = "Interior density".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Density of the adaptive infill at the center of the object.".to_string();
            def.sidetext = "%".to_string();
            def.min = 1_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3771
            def.max = 100_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3773
            def.set_default_value(Box::new(ConfigOptionPercent::new(5_f64)));
        }
        {
            let def = self
                .def
                .add("adaptive_infill_gradient", ConfigOptionType::Enum);
            def.label = "Adaptive gradient".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "How the density changes from the surface to the center of the object.\nLinear: the density changes at a constant rate.\nQuadratic: the density stays close to the sparse infill density near the surface and changes faster towards the center.\nExponential: the density changes quickly just below the surface and levels off towards the center.".to_string();
            def.enum_keys_map = keys_map_adaptive_infill_gradient();
            def.enum_values.push("linear".to_string());
            def.enum_values.push("quadratic".to_string());
            def.enum_values.push("exponential".to_string());
            def.enum_labels.push("Linear".to_string());
            def.enum_labels.push("Quadratic".to_string());
            def.enum_labels.push("Exponential".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionEnum::new(
                AdaptiveInfillGradient::Linear,
            )));
        }
        {
            let def = self.def.add("gyroid_optimized", ConfigOptionType::Bool);
            def.label = "Z-buckling bias optimization (experimental)".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Tightens the gyroid wave along the Z (vertical) axis at low infill density to shorten the effective vertical column length and improve Z-axis compression buckling resistance. Filament use is preserved. No effect at ~30% sparse infill density and above. Only applies when Sparse infill pattern is set to Gyroid.".to_string();
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("sparse_infill_pattern", ConfigOptionType::Enum);
            def.label = "Sparse infill pattern".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "This is the line pattern for internal sparse infill.".to_string();
            def.enum_keys_map = keys_map_infill_pattern();
            def.enum_values.push("rectilinear".to_string());
            def.enum_values.push("alignedrectilinear".to_string());
            def.enum_values.push("zigzag".to_string());
            def.enum_values.push("crosszag".to_string());
            def.enum_values.push("lockedzag".to_string());
            def.enum_values.push("line".to_string());
            def.enum_values.push("grid".to_string());
            def.enum_values.push("triangles".to_string());
            def.enum_values.push("tri-hexagon".to_string());
            def.enum_values.push("cubic".to_string());
            def.enum_values.push("adaptivecubic".to_string());
            def.enum_values.push("quartercubic".to_string());
            def.enum_values.push("supportcubic".to_string());
            def.enum_values.push("lightning".to_string());
            def.enum_values.push("honeycomb".to_string());
            def.enum_values.push("3dhoneycomb".to_string());
            def.enum_values.push("lateral-honeycomb".to_string());
            def.enum_values.push("lateral-lattice".to_string());
            def.enum_values.push("crosshatch".to_string());
            def.enum_values.push("tpmsd".to_string());
            def.enum_values.push("tpmsfk".to_string());
            def.enum_values.push("gyroid".to_string());
            def.enum_values.push("concentric".to_string());
            def.enum_values.push("hilbertcurve".to_string());
            def.enum_values.push("archimedeanchords".to_string());
            def.enum_values.push("octagramspiral".to_string());
            def.enum_labels.push("Rectilinear".to_string());
            def.enum_labels.push("Aligned Rectilinear".to_string());
            def.enum_labels.push("Zig Zag".to_string());
            def.enum_labels.push("Cross Zag".to_string());
            def.enum_labels.push("Locked Zag".to_string());
            def.enum_labels.push("Line".to_string());
            def.enum_labels.push("Grid".to_string());
            def.enum_labels.push("Triangles".to_string());
            def.enum_labels.push("Tri-hexagon".to_string());
            def.enum_labels.push("Cubic".to_string());
            def.enum_labels.push("Adaptive Cubic".to_string());
            def.enum_labels.push("Quarter Cubic".to_string());
            def.enum_labels.push("Support Cubic".to_string());
            def.enum_labels.push("Lightning".to_string());
            def.enum_labels.push("Honeycomb".to_string());
            def.enum_labels.push("3D Honeycomb".to_string());
            def.enum_labels.push("Lateral Honeycomb".to_string());
            def.enum_labels.push("Lateral Lattice".to_string());
            def.enum_labels.push("Cross Hatch".to_string());
            def.enum_labels.push("TPMS-D".to_string());
            def.enum_labels.push("TPMS-FK".to_string());
            def.enum_labels.push("Gyroid".to_string());
            def.enum_labels.push("Concentric".to_string());
            def.enum_labels.push("Hilbert Curve".to_string());
            def.enum_labels.push("Archimedean Chords".to_string());
            def.enum_labels.push("Octagram Spiral".to_string());
            def.set_default_value(Box::new(ConfigOptionEnum::new(InfillPattern::ipCrossHatch)));
        }
        {
            let def = self
                .def
                .add("sparse_infill_smooth_factor", ConfigOptionType::Percent);
            def.label = "Sparse infill smooth factor".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Controls how strongly sparse infill corners are rounded. 0% keeps the original sharp path, while 100% produces the largest possible curves between adjacent infill lines.".to_string();
            def.sidetext = "%".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3874
            def.max = 100_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionPercent::new(0_f64)));
        }
        {
            let def = self
                .def
                .add("top_surface_acceleration", ConfigOptionType::Floats);
            def.label = "Top surface".to_string();
            def.category = "Speed".to_string();
            def.tooltip = "This is the acceleration of top surface infill. Using a lower value may improve top surface quality.".to_string();
            def.sidetext = "mm/s²".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3886
            v.values.push(500_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("outer_wall_acceleration", ConfigOptionType::Floats);
            def.label = "Outer wall".to_string();
            def.category = "Speed".to_string();
            def.tooltip =
                "Acceleration of outer wall: using a lower value can improve quality.".to_string();
            def.sidetext = "mm/s²".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3896
            v.values.push(500_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("inner_wall_acceleration", ConfigOptionType::Floats);
            def.label = "Inner wall".to_string();
            def.category = "Speed".to_string();
            def.tooltip = "Acceleration of inner walls.".to_string();
            def.sidetext = "mm/s²".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3906
            v.values.push(10000_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "sparse_infill_acceleration",
                ConfigOptionType::FloatsOrPercents,
            );
            def.label = "Sparse infill".to_string();
            def.category = "Speed".to_string();
            def.tooltip = "Acceleration of sparse infill. If the value is expressed as a percentage (e.g. 100%), it will be calculated based on the default acceleration.".to_string();
            def.sidetext = "mm/s² or %".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.ratio_over = "default_acceleration".to_string();
            def.nullable = true;
            let mut v = ConfigOptionFloatsOrPercentsNullable { values: Vec::new() };
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3917
            v.values.push(slic3r_config::config::FloatOrPercent {
                value: 100_f64,
                percent: true,
            });
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "internal_solid_infill_acceleration",
                ConfigOptionType::FloatsOrPercents,
            );
            def.label = "Internal solid infill".to_string();
            def.category = "Speed".to_string();
            def.tooltip = "Acceleration of internal solid infill. If the value is expressed as a percentage (e.g. 100%), it will be calculated based on the default acceleration.".to_string();
            def.sidetext = "mm/s² or %".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.ratio_over = "default_acceleration".to_string();
            def.nullable = true;
            let mut v = ConfigOptionFloatsOrPercentsNullable { values: Vec::new() };
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3928
            v.values.push(slic3r_config::config::FloatOrPercent {
                value: 100_f64,
                percent: true,
            });
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("initial_layer_acceleration", ConfigOptionType::Floats);
            def.label = "First layer".to_string();
            def.category = "Speed".to_string();
            def.tooltip = "This is the printing acceleration for the first layer. Using limited acceleration can improve build plate adhesion.".to_string();
            def.sidetext = "mm/s²".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3938
            v.values.push(300_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("accel_to_decel_enable", ConfigOptionType::Bool);
            def.label = "Enable accel_to_decel".to_string();
            def.category = "Speed".to_string();
            def.tooltip =
                "Klipper's max_accel_to_decel will be adjusted automatically.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(true)));
        }
        {
            let def = self
                .def
                .add("accel_to_decel_factor", ConfigOptionType::Percent);
            def.label = "accel_to_decel".to_string();
            def.category = "Speed".to_string();
            def.tooltip =
                "Klipper's max_accel_to_decel will be adjusted to this % of acceleration."
                    .to_string();
            def.sidetext = "%".to_string();
            def.min = 1_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3954
            def.max = 100_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3956
            def.set_default_value(Box::new(ConfigOptionPercent::new(50_f64)));
        }
        {
            let def = self.def.add("default_jerk", ConfigOptionType::Floats);
            def.label = "Default".to_string();
            def.category = "Speed".to_string();
            def.tooltip = "Default jerk.".to_string();
            def.sidetext = "mm/s".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            v.values.push(0_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("default_junction_deviation", ConfigOptionType::Floats);
            def.label = "Junction Deviation".to_string();
            def.category = "Speed".to_string();
            def.tooltip =
                "Marlin Firmware Junction Deviation (replaces the traditional XY Jerk setting)."
                    .to_string();
            def.sidetext = "mm".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3973
            def.min = 0.;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3974
            def.max = 0.3;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3977
            v.values.push(0.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("outer_wall_jerk", ConfigOptionType::Floats);
            def.label = "Outer wall".to_string();
            def.category = "Speed".to_string();
            def.tooltip = "Jerk of outer walls.".to_string();
            def.sidetext = "mm/s".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3987
            v.values.push(9_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("inner_wall_jerk", ConfigOptionType::Floats);
            def.label = "Inner wall".to_string();
            def.category = "Speed".to_string();
            def.tooltip = "Jerk of inner walls.".to_string();
            def.sidetext = "mm/s".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:3997
            v.values.push(9_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("top_surface_jerk", ConfigOptionType::Floats);
            def.label = "Top surface".to_string();
            def.category = "Speed".to_string();
            def.nullable = true;
            def.tooltip = "Jerk for top surface.".to_string();
            def.sidetext = "mm/s".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloatsNullable::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4007
            v.values.push(9_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("infill_jerk", ConfigOptionType::Floats);
            def.label = "Infill".to_string();
            def.category = "Speed".to_string();
            def.tooltip = "Jerk for infill.".to_string();
            def.sidetext = "mm/s".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4017
            v.values.push(9_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("initial_layer_jerk", ConfigOptionType::Floats);
            def.label = "First layer".to_string();
            def.category = "Speed".to_string();
            def.tooltip = "Jerk for the first layer.".to_string();
            def.sidetext = "mm/s".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4027
            v.values.push(9_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("travel_jerk", ConfigOptionType::Floats);
            def.label = "Travel".to_string();
            def.category = "Speed".to_string();
            def.tooltip = "Jerk for travel.".to_string();
            def.sidetext = "mm/s".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4037
            v.values.push(12_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "initial_layer_travel_jerk",
                ConfigOptionType::FloatsOrPercents,
            );
            def.label = "First layer travel".to_string();
            def.tooltip =
                "Travel jerk of first layer.\nThe percentage value is relative to Travel Jerk."
                    .to_string();
            def.sidetext = "mm/s or %".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.ratio_over = "travel_jerk".to_string();
            def.nullable = true;
            let mut v = ConfigOptionFloatsOrPercentsNullable { values: Vec::new() };
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4047
            v.values.push(slic3r_config::config::FloatOrPercent {
                value: 100_f64,
                percent: true,
            });
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("initial_layer_line_width", ConfigOptionType::FloatOrPercent);
            def.label = "First layer".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Line width of the first layer. If expressed as a %, it will be computed over the nozzle diameter.".to_string();
            def.sidetext = "mm or %".to_string();
            def.ratio_over = "nozzle_diameter".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4056
            def.max = 1000_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4057
            def.max_literal = 10_f64;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4059
            def.set_default_value(Box::new(ConfigOptionFloatOrPercent::new(0., false)));
        }
        {
            let def = self
                .def
                .add("initial_layer_print_height", ConfigOptionType::Float);
            def.label = "First layer height".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Height of the first layer. Making the first layer height thicker can improve build plate adhesion.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4067
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.2)));
        }
        {
            let def = self
                .def
                .add("initial_layer_speed", ConfigOptionType::Floats);
            def.label = "First layer".to_string();
            def.tooltip =
                "This is the speed for the walls and brim of the first layer.".to_string();
            def.sidetext = "mm/s".to_string();
            def.min = 1_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4084
            v.values.push(30_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("initial_layer_infill_speed", ConfigOptionType::Floats);
            def.label = "First layer infill".to_string();
            def.tooltip = "This is the speed for the first layer except for walls and brim, such as infill, gap fill and support.".to_string();
            def.sidetext = "mm/s".to_string();
            def.min = 1_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4093
            v.values.push(60.0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "initial_layer_travel_speed",
                ConfigOptionType::FloatsOrPercents,
            );
            def.label = "First layer travel speed".to_string();
            def.tooltip = "Travel speed of the first layer.".to_string();
            def.category = "Speed".to_string();
            def.sidetext = "mm/s or %".to_string();
            def.ratio_over = "travel_speed".to_string();
            def.min = 1_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsOrPercentsNullable { values: Vec::new() };
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4104
            v.values.push(slic3r_config::config::FloatOrPercent {
                value: 100_f64,
                percent: true,
            });
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("slow_down_layers", ConfigOptionType::Int);
            def.label = "Number of slow layers".to_string();
            def.tooltip = "The first few layers are printed slower than normal. The speed is gradually increased in a linear fashion over the specified number of layers.".to_string();
            def.category = "Speed".to_string();
            def.sidetext = "layers".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionInt::new(0)));
        }
        {
            let def = self
                .def
                .add("nozzle_temperature_initial_layer", ConfigOptionType::Ints);
            def.label = "First layer".to_string();
            def.full_label = "First layer nozzle temperature".to_string();
            def.tooltip =
                "Nozzle temperature for printing the first layer with this filament".to_string();
            def.sidetext = "\u{2103}".to_string();
            def.min = 0_f32;
            def.max = max_temp;
            let mut v = ConfigOptionInts::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4123
            v.values.push(200);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("full_fan_speed_layer", ConfigOptionType::Ints);
            def.label = "Full fan speed at layer".to_string();
            def.tooltip = "Fan speed will be ramped up linearly from zero at layer \"close_fan_the_first_x_layers\" to maximum at layer \"full_fan_speed_layer\". \"full_fan_speed_layer\" will be ignored if lower than \"close_fan_the_first_x_layers\", in which case the fan will be running at maximum allowed speed at layer \"close_fan_the_first_x_layers\" + 1.".to_string();
            def.sidetext = "layer".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4133
            def.max = 1000_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionInts::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("initial_layer_fan_speed", ConfigOptionType::Ints);
            def.label = "First layer fan speed".to_string();
            def.tooltip = "Sets an exact fan speed for the first layer, overriding all other cooling settings. Useful for protecting 3D-printed toolhead parts (e.g. Voron-style ABS/ASA ducts) from a hot bed. A small amount of airflow cools the ducts down, without using full cooling that may in certain conditions hurt first-layer adhesion.\nFrom the second layer onwards, normal cooling resumes.\nIf \"Full fan speed at layer\" is also set, the fan ramps smoothly from this value on the first layer up to your target by the chosen layer.\nOnly available when \"No cooling for the first\" is 0.\nSet to -1 to disable it.".to_string();
            def.sidetext = "%".to_string();
            def.min = -1_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4151
            def.max = 100_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionInts::new();
            v.values.push(-1);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "support_material_interface_fan_speed",
                ConfigOptionType::Ints,
            );
            def.label = "Support interface fan speed".to_string();
            def.tooltip = "This part cooling fan speed is applied when printing support interfaces. Setting this parameter to a higher than regular speed reduces the layer binding strength between supports and the supported part, making them easier to separate.\nSet to -1 to disable it.\nThis setting is overridden by disable_fan_first_layers.".to_string();
            def.sidetext = "%".to_string();
            def.min = -1_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4163
            def.max = 100_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionInts::new();
            v.values.push(-1);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("internal_bridge_fan_speed", ConfigOptionType::Ints);
            def.label = "Internal bridges fan speed".to_string();
            def.tooltip = "The part cooling fan speed used for all internal bridges. Set to -1 to use the overhang fan speed settings instead.\n\nReducing the internal bridges fan speed, compared to your regular fan speed, can help reduce part warping due to excessive cooling applied over a large surface for a prolonged period of time.".to_string();
            def.sidetext = "%".to_string();
            def.min = -1_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4175
            def.max = 100_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionInts::new();
            v.values.push(-1);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("ironing_fan_speed", ConfigOptionType::Ints);
            def.label = "Ironing fan speed".to_string();
            def.tooltip = "This part cooling fan speed is applied when ironing. Setting this parameter to a lower than regular speed reduces possible nozzle clogging due to the low volumetric flow rate, making the interface smoother.\nSet to -1 to disable it.".to_string();
            def.sidetext = "%".to_string();
            def.min = -1_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4186
            def.max = 100_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionInts::new();
            v.values.push(-1);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_ironing_flow", ConfigOptionType::Percents);
            def.label = "Ironing flow".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Filament-specific override for ironing flow. This allows you to customize the ironing flow for each filament type. Too high value results in overextrusion on the surface.".to_string();
            def.sidetext = "%".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4198
            def.max = 100_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionPercentsNullable { values: Vec::new() };
            v.values.push(ConfigOptionFloatsNullable::nil_value());
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_ironing_spacing", ConfigOptionType::Floats);
            def.label = "Ironing line spacing".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Filament-specific override for ironing line spacing. This allows you to customize the spacing between ironing lines for each filament type.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.max = 1_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            v.values.push(ConfigOptionFloatsNullable::nil_value());
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_ironing_inset", ConfigOptionType::Floats);
            def.label = "Ironing inset".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Filament-specific override for ironing inset. This allows you to customize the distance to keep from the edges when ironing for each filament type.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4222
            def.max = 100_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            v.values.push(ConfigOptionFloatsNullable::nil_value());
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_ironing_speed", ConfigOptionType::Floats);
            def.label = "Ironing speed".to_string();
            def.category = "Speed".to_string();
            def.tooltip = "Filament-specific override for ironing speed. This allows you to customize the print speed of ironing lines for each filament type.".to_string();
            def.sidetext = "mm/s".to_string();
            def.min = 1_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            v.values.push(ConfigOptionFloatsNullable::nil_value());
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("fuzzy_skin", ConfigOptionType::Enum);
            def.label = "Fuzzy skin".to_string();
            def.category = "Others".to_string();
            def.tooltip = "This setting makes the toolhead randomly jitter while printing walls so that the surface has a rough textured look. This setting controls the fuzzy position.".to_string();
            def.enum_keys_map = keys_map_fuzzy_skin_type();
            def.enum_values.push("none".to_string());
            def.enum_values.push("external".to_string());
            def.enum_values.push("hole".to_string());
            def.enum_values.push("all".to_string());
            def.enum_values.push("allwalls".to_string());
            def.enum_values.push("disabled_fuzzy".to_string());
            def.enum_labels.push("Painted only".to_string());
            def.enum_labels.push("Contour".to_string());
            def.enum_labels.push("Hole".to_string());
            def.enum_labels.push("Contour and hole".to_string());
            def.enum_labels.push("All walls".to_string());
            def.enum_labels.push("Disabled".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            def.set_default_value(Box::new(ConfigOptionEnum::new(
                FuzzySkinType::Disabled_fuzzy,
            )));
        }
        {
            let def = self
                .def
                .add("fuzzy_skin_thickness", ConfigOptionType::Float);
            def.label = "Fuzzy skin thickness".to_string();
            def.category = "Others".to_string();
            def.tooltip = "The width of jittering: it\u{2019}s recommended to keep this lower than the outer wall line width.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.max = 2_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4266
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.2)));
        }
        {
            let def = self
                .def
                .add("fuzzy_skin_point_distance", ConfigOptionType::Float);
            def.label = "Fuzzy skin point distance".to_string();
            def.category = "Others".to_string();
            def.tooltip =
                "The average distance between the random points introduced on each line segment."
                    .to_string();
            def.sidetext = "mm".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4273
            def.min = 0.01;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4274
            def.max = 5.;
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4276
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.3)));
        }
        {
            let def = self
                .def
                .add("fuzzy_skin_first_layer", ConfigOptionType::Bool);
            def.label = "Apply fuzzy skin to first layer".to_string();
            def.category = "Others".to_string();
            def.tooltip = "Whether to apply fuzzy skin on the first layer.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("fuzzy_skin_mode", ConfigOptionType::Enum);
            def.label = "Fuzzy skin generator mode".to_string();
            def.category = "Others".to_string();
            def.tooltip = "Fuzzy skin generation mode. Works only with Arachne!\nDisplacement: Сlassic mode when the pattern is formed by shifting the nozzle sideways from the original path.\nExtrusion: The mode when the pattern formed by the amount of extruded plastic. This is the fast and straight algorithm without unnecessary nozzle shake that gives a smooth pattern. But it is more useful for forming loose walls in the entire they array.\nCombined: Joint mode [Displacement] + [Extrusion]. The appearance of the walls is similar to [Displacement] Mode, but it leaves no pores between the perimeters.\n\nAttention! The [Extrusion] and [Combined] modes works only the fuzzy_skin_thickness parameter not more than the thickness of printed loop. At the same time, the width of the extrusion for a particular layer should also not be below a certain level. It is usually equal 15-25% of a layer height. Therefore, the maximum fuzzy skin thickness with a perimeter width of 0.4 mm and a layer height of 0.2 mm will be 0.4-(0.2*0.25)=±0.35mm! If you enter a higher parameter than this, the error Flow::spacing() will displayed, and the model will not be sliced. You can choose this number until this error is repeated.".to_string();
            def.enum_keys_map = keys_map_fuzzy_skin_mode();
            def.enum_values.push("displacement".to_string());
            def.enum_values.push("extrusion".to_string());
            def.enum_values.push("combined".to_string());
            def.enum_labels.push("Displacement".to_string());
            def.enum_labels.push("Extrusion".to_string());
            def.enum_labels.push("Combined".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            def.set_default_value(Box::new(ConfigOptionEnum::new(FuzzySkinMode::Displacement)));
        }
        {
            let def = self
                .def
                .add("fuzzy_skin_noise_type", ConfigOptionType::Enum);
            def.label = "Fuzzy skin noise type".to_string();
            def.category = "Others".to_string();
            def.tooltip = "Noise type to use for fuzzy skin generation:\nClassic: Classic uniform random noise.\nPerlin: Perlin noise, which gives a more consistent texture.\nBillow: Similar to perlin noise, but clumpier.\nRidged Multifractal: Ridged noise with sharp, jagged features. Creates marble-like textures.\nVoronoi: Divides the surface into voronoi cells, and displaces each one by a random amount. Creates a patchwork texture.\nRipple: Uniform ripple pattern that ripples left and right of the original path. Repeating pattern, woven appearance.".to_string();
            def.enum_keys_map = keys_map_noise_type();
            def.enum_values.push("classic".to_string());
            def.enum_values.push("perlin".to_string());
            def.enum_values.push("billow".to_string());
            def.enum_values.push("ridgedmulti".to_string());
            def.enum_values.push("voronoi".to_string());
            def.enum_values.push("ripple".to_string());
            def.enum_labels.push("Classic".to_string());
            def.enum_labels.push("Perlin".to_string());
            def.enum_labels.push("Billow".to_string());
            def.enum_labels.push("Ridged Multifractal".to_string());
            def.enum_labels.push("Voronoi".to_string());
            def.enum_labels.push("Ripple".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            def.set_default_value(Box::new(ConfigOptionEnum::new(NoiseType::Classic)));
        }
        {
            let def = self.def.add("fuzzy_skin_scale", ConfigOptionType::Float);
            def.label = "Fuzzy skin feature size".to_string();
            def.category = "Others".to_string();
            def.tooltip = "The base size of the coherent noise features, in mm. Higher values will result in larger features.".to_string();
            def.sidetext = "mm".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4340
            def.min = 0.1;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4341
            def.max = 500_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4343
            def.set_default_value(Box::new(ConfigOptionFloat::new(1.0)));
        }
        {
            let def = self.def.add("fuzzy_skin_octaves", ConfigOptionType::Int);
            def.label = "Fuzzy Skin Noise Octaves".to_string();
            def.category = "Others".to_string();
            def.tooltip = "The number of octaves of coherent noise to use. Higher values increase the detail of the noise, but also increase computation time.".to_string();
            def.min = 1_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4350
            def.max = 10_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4352
            def.set_default_value(Box::new(ConfigOptionInt::new(4)));
        }
        {
            let def = self
                .def
                .add("fuzzy_skin_persistence", ConfigOptionType::Float);
            def.label = "Fuzzy skin noise persistence".to_string();
            def.category = "Others".to_string();
            def.tooltip = "The decay rate for higher octaves of the coherent noise. Lower values will result in smoother noise.".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4358
            def.min = 0.01;
            def.max = 1_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4361
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.5)));
        }
        {
            let def = self
                .def
                .add("fuzzy_skin_ripples_per_layer", ConfigOptionType::Int);
            def.label = "Number of ripples per layer".to_string();
            def.category = "Others".to_string();
            def.tooltip =
                "Controls how many full cycles of ripples will be added per layer.".to_string();
            def.min = 1_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4369
            def.set_default_value(Box::new(ConfigOptionInt::new(15)));
        }
        {
            let def = self
                .def
                .add("fuzzy_skin_ripple_offset", ConfigOptionType::Percent);
            def.label = "Ripple offset".to_string();
            def.category = "Others".to_string();
            def.tooltip = "Shifts the ripple phase forward along the print path by the specified percentage of a wavelength each layer period.\n- 0% keeps every layer identical.\n- 50% shifts the pattern by half a wavelength, effectively inverting the phase.\n- 100% shifts the pattern by a full wavelength, returning to the original phase.\n\nThe shift is applied once every number of layers set by Layers between ripple offset, so layers within the same group are printed identically.".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4380
            def.max = 100_f32;
            def.sidetext = "%".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4383
            def.set_default_value(Box::new(ConfigOptionPercent::new(50_f64)));
        }
        {
            let def = self.def.add(
                "fuzzy_skin_layers_between_ripple_offset",
                ConfigOptionType::Int,
            );
            def.label = "Layers between ripple offset".to_string();
            def.category = "Others".to_string();
            def.tooltip = "Specifies how many consecutive layers share the same ripple phase before the offset is applied.\nFor example:\n- 1 = Layer 1 is printed with the base ripple pattern, then layer 2 is shifted by the configured offset, then layer 3 returns to the base pattern, and so on.\n- 3 = Layers 1 to 3 are printed with the base ripple pattern, then layers 4 to 6 are shifted by the configured offset, then layers 7 to 9 return to the base pattern, etc.".to_string();
            def.min = 1_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionInt::new(1)));
        }
        {
            let def = self.def.add("filter_out_gap_fill", ConfigOptionType::Float);
            def.label = "Filter out tiny gaps".to_string();
            def.category = "Layers and Perimeters".to_string();
            def.tooltip = "Don't print gap fill with a length is smaller than the threshold specified (in mm). This setting applies to top, bottom and solid infill and, if using the classic perimeter generator, to wall gap fill.".to_string();
            def.sidetext = "mm".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(0_f64)));
        }
        {
            let def = self.def.add("gap_infill_speed", ConfigOptionType::Floats);
            def.label = "Gap infill".to_string();
            def.category = "Speed".to_string();
            def.tooltip = "This is the speed for gap infill. Gaps usually have irregular line width and should be printed more slowly.".to_string();
            def.sidetext = "mm/s".to_string();
            def.min = 1_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4413
            v.values.push(30_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("precise_z_height", ConfigOptionType::Bool);
            def.label = "Precise Z height".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Enable this to get precise Z height of object after slicing. It will get the precise object height by fine-tuning the layer heights of the last few layers. Note that this is an experimental parameter.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("enable_arc_fitting", ConfigOptionType::Enum);
            def.label = "Arc fitting".to_string();
            def.tooltip = "Fit G2 and G3 arc moves to the toolpaths. Walls and surfaces are fitted within the resolution, while sparse infill, supports and the wipe tower use fixed tolerances.\n - Disabled: only straight G1 moves are used.\n - Enabled: arcs are fitted to all toolpaths.\n - Sparse infill only: arcs are fitted to the sparse infill only, which shortens the G-code of curved infill patterns without affecting the walls and surfaces.\n\nNote: For Klipper machines, Disabled is recommended. Klipper splits arcs back into line segments, which lowers the surface quality and can increase the print time.".to_string();
            def.enum_keys_map = keys_map_arc_fitting_mode();
            def.enum_values.push("disabled".to_string());
            def.enum_values.push("enabled".to_string());
            def.enum_values.push("sparse_infill".to_string());
            def.enum_labels.push("Disabled".to_string());
            def.enum_labels.push("Enabled".to_string());
            def.enum_labels.push("Sparse infill only".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionEnum::new(ArcFittingMode::Disabled)));
        }
        {
            let def = self
                .def
                .add("gcode_add_line_number", ConfigOptionType::Bool);
            def.label = "Add line number".to_string();
            def.tooltip =
                "Enable this to add line number(Nx) at the beginning of each G-code line."
                    .to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("scan_first_layer", ConfigOptionType::Bool);
            def.label = "Scan first layer".to_string();
            def.tooltip = "Enable this to allow the camera on the printer to check the quality of the first layer.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("enable_power_loss_recovery", ConfigOptionType::Enum);
            def.label = "Power Loss Recovery".to_string();
            def.tooltip = "Choose how to control power loss recovery. When set to Printer configuration, the slicer will not emit power loss recovery G-code and will leave the printer's configuration unchanged. Applicable to Bambu Lab or Marlin 2 firmware based printers.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.enum_keys_map = keys_map_power_loss_recovery_mode();
            def.enum_values.push("printer_configuration".to_string());
            def.enum_values.push("enable".to_string());
            def.enum_values.push("disable".to_string());
            def.enum_labels.push("Printer configuration".to_string());
            def.enum_labels.push("Enable".to_string());
            def.enum_labels.push("Disable".to_string());
            def.set_default_value(Box::new(ConfigOptionEnum::new(
                PowerLossRecoveryMode::PrinterConfiguration,
            )));
        }
        {
            let def = self.def.add("nozzle_type", ConfigOptionType::Enums);
            def.label = "Nozzle type".to_string();
            def.tooltip = "The metallic material of the nozzle: This determines the abrasive resistance of the nozzle and what kind of filament can be printed.".to_string();
            def.enum_keys_map = keys_map_nozzle_type();
            def.enum_values.push("undefine".to_string());
            def.enum_values.push("hardened_steel".to_string());
            def.enum_values.push("stainless_steel".to_string());
            def.enum_values.push("tungsten_carbide".to_string());
            def.enum_values.push("brass".to_string());
            def.enum_labels.push("Undefined".to_string());
            def.enum_labels.push("Hardened steel".to_string());
            def.enum_labels.push("Stainless steel".to_string());
            def.enum_labels.push("Tungsten carbide".to_string());
            def.enum_labels.push("Brass".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = slic3r_config::config::ConfigOptionEnumsGenericNullable::default();
            v.values
                .push(slic3r_commondefs::common_defs::NozzleType::ntUndefine as i32);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("nozzle_hrc", ConfigOptionType::Int);
            def.label = "Nozzle HRC".to_string();
            def.tooltip =
                "The nozzle's hardness. Zero means no checking for nozzle hardness during slicing."
                    .to_string();
            def.sidetext = "HRC".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4504
            def.max = 500_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            def.set_default_value(Box::new(ConfigOptionInt::new(0)));
        }
        {
            let def = self.def.add("printer_structure", ConfigOptionType::Enum);
            def.label = "Printer structure".to_string();
            def.tooltip =
                "The physical arrangement and components of a printing device.".to_string();
            def.enum_keys_map = keys_map_printer_structure();
            def.enum_values.push("undefine".to_string());
            def.enum_values.push("corexy".to_string());
            def.enum_values.push("i3".to_string());
            def.enum_values.push("hbot".to_string());
            def.enum_values.push("delta".to_string());
            def.enum_labels.push("Undefined".to_string());
            def.enum_labels.push("CoreXY".to_string());
            def.enum_labels.push("I3".to_string());
            def.enum_labels.push("Hbot".to_string());
            def.enum_labels.push("Delta".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            def.set_default_value(Box::new(ConfigOptionEnum::new(
                PrinterStructure::psUndefine,
            )));
        }
        {
            let def = self.def.add("best_object_pos", ConfigOptionType::Point);
            def.label = "Best object position".to_string();
            def.tooltip =
                "Best auto arranging position in range [0,1] w.r.t. bed shape.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4529
            def.set_default_value(Box::new(ConfigOptionPoint::new(Vec2d::new(0.5, 0.5))));
        }
        {
            let def = self.def.add("auxiliary_fan", ConfigOptionType::Bool);
            def.label = "Auxiliary part cooling fan".to_string();
            def.tooltip = "Enable this option if machine has auxiliary part cooling fan.\nG-code command: M106 P2 S(0-255).".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("fan_direction", ConfigOptionType::Enum);
            def.label = "Fan direction".to_string();
            def.tooltip = "Cooling fan direction of the printer".to_string();
            def.enum_keys_map = keys_map_fan_direction();
            def.enum_values.push("undefine".to_string());
            def.enum_values.push("left".to_string());
            def.enum_values.push("right".to_string());
            def.enum_values.push("both".to_string());
            def.enum_labels.push("Undefined".to_string());
            def.enum_labels.push("Left".to_string());
            def.enum_labels.push("Right".to_string());
            def.enum_labels.push("Both".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            def.set_default_value(Box::new(ConfigOptionEnum::new(FanDirection::fdUndefine)));
        }
        {
            let def = self.def.add("fan_speedup_time", ConfigOptionType::Float);
            def.tooltip = "Start the fan this number of seconds earlier than its target start time (you can use fractional seconds). It assumes infinite acceleration for this time estimation, and will only take into account G1 and G0 moves (arc fitting is unsupported).\nIt won't move fan commands from custom G-code (they act as a sort of 'barrier').\nIt won't move fan commands into the start G-code if the 'only custom start G-code' is activated.\nUse 0 to deactivate.".to_string();
            def.sidetext = "s".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(0_f64)));
        }
        {
            let def = self
                .def
                .add("fan_speedup_overhangs", ConfigOptionType::Bool);
            def.label = "Only overhangs".to_string();
            def.tooltip =
                "Will only take into account the delay for the cooling of overhangs.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(true)));
        }
        {
            let def = self.def.add("fan_kickstart", ConfigOptionType::Float);
            def.label = "Fan kick-start time".to_string();
            def.tooltip = "Emit a max fan speed command for this amount of seconds before reducing to target speed to kick-start the cooling fan.\nThis is useful for fans where a low PWM/power may be insufficient to get the fan started spinning from a stop, or to get the fan up to speed faster.\nSet to 0 to deactivate.".to_string();
            def.sidetext = "s".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(0_f64)));
        }
        {
            let def = self
                .def
                .add("part_cooling_fan_min_pwm", ConfigOptionType::Int);
            def.label = "Minimum non-zero part cooling fan speed".to_string();
            def.tooltip = "Some part-cooling fans cannot start spinning when commanded below a certain PWM duty cycle. When set above 0, any non-zero part-cooling fan command will be raised to at least this percentage so the fan reliably starts. A fan command of 0 (fan off) is always honoured exactly. This clamp is applied after every other fan calculation (first-layer ramp, layer-time interpolation, overhang/bridge/support-interface/ironing overrides), so scaling still operates within the range [this value, 100%].\nIf your firmware already disables the fan below a threshold (for example Klipper's [fan] off_below: 0.10 shuts the fan off whenever the commanded duty cycle is below 10%), this option and the firmware threshold should ideally be set to the same value. Matching them (e.g. off_below: 0.10 in Klipper and 10% here) guarantees the slicer never emits a non-zero value that the firmware would silently drop, and the fan never receives a value below the one you know it can actually spool at.\nSet to 0 to deactivate.".to_string();
            def.sidetext = "%".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4600
            def.max = 100_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionInt::new(0)));
        }
        {
            let def = self.def.add("time_cost", ConfigOptionType::Float);
            def.label = "Time cost".to_string();
            def.tooltip = "The printer cost per hour.".to_string();
            def.sidetext = "money/h".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(0_f64)));
        }
        {
            let def = self
                .def
                .add("support_chamber_temp_control", ConfigOptionType::Bool);
            def.label = "Support controlling chamber temperature".to_string();
            def.tooltip = "This option is enabled if the machine supports controlling the chamber temperature\nG-code command: M141 S(0-255)".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(true)));
            def.readonly = false;
        }
        {
            let def = self
                .def
                .add("support_air_filtration", ConfigOptionType::Bool);
            def.label = "Support air filtration".to_string();
            def.tooltip = "Enable this if the printer supports air filtration\nG-code command: M106 P3 S(0-255)".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            def.set_default_value(Box::new(ConfigOptionBool::new(true)));
        }
        {
            let def = self
                .def
                .add("support_cooling_filter", ConfigOptionType::Bool);
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("cooling_filter_enabled", ConfigOptionType::Bool);
            def.label = "Use cooling filter".to_string();
            def.tooltip = "Enable this if the printer supports a cooling filter".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("gcode_flavor", ConfigOptionType::Enum);
            def.label = "G-code flavor".to_string();
            def.tooltip = "What kind of G-code the printer is compatible with.".to_string();
            def.enum_keys_map = keys_map_gcode_flavor();
            def.enum_values.push("marlin".to_string());
            def.enum_values.push("klipper".to_string());
            def.enum_values.push("reprapfirmware".to_string());
            def.enum_values.push("repetier".to_string());
            def.enum_values.push("marlin2".to_string());
            def.enum_labels.push("Marlin (legacy)".to_string());
            def.enum_labels.push("Klipper".to_string());
            def.enum_labels.push("RepRapFirmware".to_string());
            def.enum_labels.push("Repetier".to_string());
            def.enum_labels.push("Marlin 2".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.readonly = false;
            def.set_default_value(Box::new(ConfigOptionEnum::new(
                GCodeFlavor::gcfMarlinLegacy,
            )));
        }
        {
            let def = self
                .def
                .add("gcode_skip_config_block", ConfigOptionType::Bool);
            def.label = "Skip G-code config block".to_string();
            def.tooltip = "Do not write the CONFIG_BLOCK (slicer configuration key/value pairs) into the G-code file. This can help with printers whose firmware crashes when parsing these comment lines (e.g. Anycubic go-klipper).\n\nNote: the G-code file will no longer contain slicer settings, so importing it back into OrcaSlicer will not restore the configuration.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("pellet_modded_printer", ConfigOptionType::Bool);
            def.label = "Pellet Modded Printer".to_string();
            def.tooltip =
                "Enable this option if your printer uses pellets instead of filaments.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("support_multi_bed_types", ConfigOptionType::Bool);
            def.label = "Support multi bed types".to_string();
            def.tooltip = "Enable this option if you want to use multiple bed types.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("gcode_label_objects", ConfigOptionType::Bool);
            def.label = "Label objects".to_string();
            def.tooltip = "Enable this to add comments into the G-code labeling print moves with what object they belong to, which is useful for the Octoprint CancelObject plug-in.\n\nThis setting is NOT compatible with Single Extruder Multi Material setup and Wipe into Object / Wipe into Infill.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(true)));
        }
        {
            let def = self.def.add("exclude_object", ConfigOptionType::Bool);
            def.label = "Exclude objects".to_string();
            def.tooltip = "Enable this option to add EXCLUDE OBJECT command in G-code.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("exclude_object_tolerance", ConfigOptionType::Float);
            def.label = "Outline tolerance".to_string();
            def.tooltip = "Maximum deviation of the object outline written to EXCLUDE_OBJECT_DEFINE from the original outline. Larger values give fewer points, which helps firmware that stalls while parsing G-code with many objects. 0 keeps every point.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4714
            def.max = 5_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Expert;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4716
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.1)));
        }
        {
            let def = self.def.add("gcode_comments", ConfigOptionType::Bool);
            def.label = "Verbose G-code".to_string();
            def.tooltip = "Enable this to get a commented G-code file, with each line explained by a descriptive text.\n\nIf you print from SD card, the additional weight of the file could make your firmware slow down.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("high_precision_gcode", ConfigOptionType::Bool);
            def.label = "High precision G-code (experimental)".to_string();
            def.tooltip = "Write X, Y, Z, I and J with 4 decimal places instead of 3, and E with 6 instead of 5, to reduce rounding errors in the G-code. With relative E distances, the rounding error of each move is also carried over to the next one. Supports are removed after printing, so they keep the standard precision. This may help stabilize extrusion with small nozzles (e.g. 0.2 mm) and thin layers (e.g. 0.08 mm or less), where each move extrudes so little material that rounding errors matter most. The effect depends strongly on how well the rest of the settings are tuned and may be hard to notice unless the print profile is well calibrated.\n\nThe G-code file becomes about 10 percent larger, which may slow down printers that parse G-code slowly.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Expert;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("infill_combination", ConfigOptionType::Bool);
            def.label = "Infill combination".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Automatically combine sparse infill of several layers to print together in order to reduce time. Walls are still printed with original layer height.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("infill_shift_step", ConfigOptionType::Float);
            def.label = "Infill shift step".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "This parameter adds a slight displacement to each layer of infill to create a cross texture.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4753
            def.max = 10_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4755
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.4)));
        }
        {
            let def = self
                .def
                .add("sparse_infill_rotate_template", ConfigOptionType::String);
            def.label = "Sparse infill rotation template".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Rotate the sparse infill direction per layer using a template of angles. Enter comma-separated degrees (e.g., '0,30,60,90'). Angles are applied in order by layer and repeat when the list ends.\n\nAdvanced syntax is supported: '+5' rotates +5° every layer; '+5#5' rotates +5° every 5 layers. See the Wiki for details.\n\nWhen a template is set, the standard infill direction setting is ignored.\n\nNote: some infill patterns (e.g., Gyroid) control rotation themselves; use with care.".to_string();
            def.sidetext = "°".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionString::new("".to_string())));
        }
        {
            let def = self
                .def
                .add("solid_infill_rotate_template", ConfigOptionType::String);
            def.label = "Solid infill rotation template".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "This parameter adds a rotation of solid infill direction to each layer according to the specified template. The template is a comma-separated list of angles in degrees, e.g. '0,90'. The first angle is applied to the first layer, the second angle to the second layer, and so on. If there are more layers than angles, the angles will be repeated.\n\nNote that not all solid infill patterns support rotation.".to_string();
            def.sidetext = "°".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionString::new("".to_string())));
        }
        {
            let def = self
                .def
                .add("skeleton_infill_density", ConfigOptionType::Percent);
            def.label = "Skeleton infill density".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "The remaining part of the model contour after removing a certain depth from the surface is called the skeleton. This parameter is used to adjust the density of this section. When two regions have the same sparse infill settings but different skeleton densities, their skeleton areas will develop overlapping sections. Default is as same as infill density.".to_string();
            def.sidetext = "%".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4792
            def.max = 100_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4794
            def.set_default_value(Box::new(ConfigOptionPercent::new(25_f64)));
        }
        {
            let def = self
                .def
                .add("skin_infill_density", ConfigOptionType::Percent);
            def.label = "Skin infill density".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "The portion of the model's outer surface within a certain depth range is called the skin. This parameter is used to adjust the density of this section. When two regions have the same sparse infill settings but different skin densities, this area will not be split into two separate regions. Default is as same as infill density.".to_string();
            def.sidetext = "%".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4805
            def.max = 100_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4807
            def.set_default_value(Box::new(ConfigOptionPercent::new(25_f64)));
        }
        {
            let def = self.def.add("skin_infill_depth", ConfigOptionType::Float);
            def.label = "Skin infill depth".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "The parameter sets the depth of skin.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4815
            def.max = 100_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4817
            def.set_default_value(Box::new(ConfigOptionFloat::new(2.0)));
        }
        {
            let def = self.def.add("infill_lock_depth", ConfigOptionType::Float);
            def.label = "Infill lock depth".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "The parameter sets the overlapping depth between the interior and skin."
                .to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4825
            def.max = 100_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4827
            def.set_default_value(Box::new(ConfigOptionFloat::new(1.0)));
        }
        {
            let def = self
                .def
                .add("skin_infill_line_width", ConfigOptionType::FloatOrPercent);
            def.label = "Skin line width".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Adjust the line width of the selected skin paths.".to_string();
            def.sidetext = "mm".to_string();
            def.ratio_over = "nozzle_diameter".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4837
            def.set_default_value(Box::new(ConfigOptionFloatOrPercent::new(100_f64, true)));
        }
        {
            let def = self.def.add(
                "skeleton_infill_line_width",
                ConfigOptionType::FloatOrPercent,
            );
            def.label = "Skeleton line width".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Adjust the line width of the selected skeleton paths.".to_string();
            def.sidetext = "mm".to_string();
            def.ratio_over = "nozzle_diameter".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4847
            def.set_default_value(Box::new(ConfigOptionFloatOrPercent::new(100_f64, true)));
        }
        {
            let def = self
                .def
                .add("symmetric_infill_y_axis", ConfigOptionType::Bool);
            def.label = "Symmetric infill Y axis".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "If the model has two parts that are symmetric about the Y axis, and you want these parts to have symmetric textures, please click this option on one of the parts.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("infill_complete_top", ConfigOptionType::Bool);
            def.label = "Fill pattern tops".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Choose this option if you want to completely fill in the tops of the infill pattern".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add(
                "infill_combination_max_layer_height",
                ConfigOptionType::FloatOrPercent,
            );
            def.label = "Infill combination - Max layer height".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Maximum layer height for the combined sparse infill.\n\nSet it to 0 or 100% to use the nozzle diameter (for maximum reduction in print time) or a value of ~80% to maximize sparse infill strength.\n\nThe number of layers over which infill is combined is derived by dividing this value with the layer height and rounded down to the nearest decimal.\n\nUse either absolute mm values (eg. 0.32mm for a 0.4mm nozzle) or % values (eg 80%). This value must not be larger than the nozzle diameter.".to_string();
            def.sidetext = "mm or %".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4876
            def.set_default_value(Box::new(ConfigOptionFloatOrPercent::new(100., true)));
        }
        {
            let def = self
                .def
                .add("enable_wrapping_detection", ConfigOptionType::Bool);
            def.label = "Enable clumping detection".to_string();
            def.tooltip = "Enable clumping detection".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("wrapping_detection_layers", ConfigOptionType::Int);
            def.label = "Clumping detection layers".to_string();
            def.tooltip = "Clumping detection layers.".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4890
            def.set_default_value(Box::new(ConfigOptionInt::new(20)));
        }
        {
            let def = self
                .def
                .add("wrapping_exclude_area", ConfigOptionType::Points);
            def.label = "Probing exclude area of clumping".to_string();
            def.tooltip = "Probing exclude area of clumping.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.gui_type = GuiType::OneString;
            def.set_default_value(Box::new(ConfigOptionPoints::new()));
        }
        {
            let def = self
                .def
                .add("lateral_lattice_angle_1", ConfigOptionType::Float);
            def.label = "Lateral lattice angle 1".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "The angle of the first set of Lateral lattice elements in the Z direction. Zero is vertical.".to_string();
            def.sidetext = "°".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4904
            def.min = -75_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4905
            def.max = 75_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4907
            def.set_default_value(Box::new(ConfigOptionFloat::new(-45_f64)));
        }
        {
            let def = self
                .def
                .add("lateral_lattice_angle_2", ConfigOptionType::Float);
            def.label = "Lateral lattice angle 2".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "The angle of the second set of Lateral lattice elements in the Z direction. Zero is vertical.".to_string();
            def.sidetext = "°".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4914
            def.min = -75_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4915
            def.max = 75_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4917
            def.set_default_value(Box::new(ConfigOptionFloat::new(45_f64)));
        }
        {
            let def = self
                .def
                .add("infill_overhang_angle", ConfigOptionType::Float);
            def.label = "Infill overhang angle".to_string();
            def.category = "Strength".to_string();
            def.tooltip =
                "The angle of the infill angled lines. 60° will result in a pure honeycomb."
                    .to_string();
            def.sidetext = "°".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4924
            def.min = 15_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4925
            def.max = 75_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4927
            def.set_default_value(Box::new(ConfigOptionFloat::new(60_f64)));
        }
        {
            let def = self
                .def
                .add("lightning_overhang_angle", ConfigOptionType::Float);
            def.label = "Lightning overhang angle".to_string();
            def.category = "Strength".to_string();
            def.tooltip =
                "Maximum overhang angle for Lightning infill support propagation.".to_string();
            def.sidetext = "°".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4934
            def.min = 5_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4935
            def.max = 85_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Expert;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4937
            def.set_default_value(Box::new(ConfigOptionFloat::new(45_f64)));
        }
        {
            let def = self
                .def
                .add("lightning_prune_angle", ConfigOptionType::Float);
            def.label = "Prune angle".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Controls how aggressively short or unsupported Lightning branches are pruned.\nThis angle is converted internally to a per-layer distance.".to_string();
            def.sidetext = "°".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4945
            def.min = 5_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4946
            def.max = 85_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Expert;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4948
            def.set_default_value(Box::new(ConfigOptionFloat::new(45_f64)));
        }
        {
            let def = self
                .def
                .add("lightning_straightening_angle", ConfigOptionType::Float);
            def.label = "Straightening angle".to_string();
            def.category = "Strength".to_string();
            def.tooltip =
                "Maximum straightening angle used to simplify Lightning branches.".to_string();
            def.sidetext = "°".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4955
            def.min = 5_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4956
            def.max = 85_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Expert;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4958
            def.set_default_value(Box::new(ConfigOptionFloat::new(45_f64)));
        }
        {
            let def = self
                .def
                .add("infill_anchor", ConfigOptionType::FloatOrPercent);
            def.label = "Sparse infill anchor length".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Connect an infill line to an internal perimeter with a short segment of an additional perimeter. If expressed as percentage (example: 15%) it is calculated over infill extrusion width. Orca Slicer tries to connect two close infill lines to a short perimeter segment. If no such perimeter segment shorter than infill_anchor_max is found, the infill line is connected to a perimeter segment at just one side and the length of the perimeter segment taken is limited to this parameter, but no longer than anchor_length_max.\nSet this parameter to zero to disable anchoring perimeters connected to a single infill line.".to_string();
            def.sidetext = "mm or %".to_string();
            def.ratio_over = "sparse_infill_line_width".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4971
            def.max_literal = 1000_f64;
            def.gui_type = GuiType::FEnumOpen;
            def.enum_values.push("0".to_string());
            def.enum_values.push("1".to_string());
            def.enum_values.push("2".to_string());
            def.enum_values.push("5".to_string());
            def.enum_values.push("10".to_string());
            def.enum_values.push("1000".to_string());
            def.enum_labels.push("0 (no open anchors)".to_string());
            def.enum_labels.push("1 mm".to_string());
            def.enum_labels.push("2 mm".to_string());
            def.enum_labels.push("5 mm".to_string());
            def.enum_labels.push("10 mm".to_string());
            def.enum_labels.push("1000 (unlimited)".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:4986
            def.set_default_value(Box::new(ConfigOptionFloatOrPercent::new(400_f64, true)));
        }
        {
            let def_infill_anchor_min_sidetext = self.def.options["infill_anchor"].sidetext.clone();
            let def_infill_anchor_min_ratio_over =
                self.def.options["infill_anchor"].ratio_over.clone();
            let def_infill_anchor_min_gui_type = self.def.options["infill_anchor"].gui_type;
            let def_infill_anchor_min_enum_values =
                self.def.options["infill_anchor"].enum_values.clone();
            let def_infill_anchor_min_max_literal = self.def.options["infill_anchor"].max_literal;
            let def = self
                .def
                .add("infill_anchor_max", ConfigOptionType::FloatOrPercent);
            def.label = "Maximum length of the infill anchor".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Connect an infill line to an internal perimeter with a short segment of an additional perimeter. If expressed as percentage (example: 15%) it is calculated over infill extrusion width. Orca Slicer tries to connect two close infill lines to a short perimeter segment. If no such perimeter segment shorter than this parameter is found, the infill line is connected to a perimeter segment at just one side and the length of the perimeter segment taken is limited to infill_anchor, but no longer than this parameter.\nIf set to 0, the old algorithm for infill connection will be used, it should create the same result as with 1000 & 0.".to_string();
            def.sidetext = def_infill_anchor_min_sidetext;
            def.ratio_over = def_infill_anchor_min_ratio_over;
            def.gui_type = def_infill_anchor_min_gui_type;
            def.enum_values = def_infill_anchor_min_enum_values;
            def.max_literal = def_infill_anchor_min_max_literal;
            def.enum_labels.push("0 (Simple connect)".to_string());
            def.enum_labels.push("1 mm".to_string());
            def.enum_labels.push("2 mm".to_string());
            def.enum_labels.push("5 mm".to_string());
            def.enum_labels.push("10 mm".to_string());
            def.enum_labels.push("1000 (unlimited)".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5009
            def.set_default_value(Box::new(ConfigOptionFloatOrPercent::new(20_f64, false)));
        }
        {
            let def = self
                .def
                .add("sparse_infill_filament_id", ConfigOptionType::Int);
            def.gui_type = GuiType::IEnumOpen;
            def.label = "Infill".to_string();
            def.category = "Extruders".to_string();
            def.tooltip = "Filament to print internal sparse infill.\n\"Default\" uses the active object/part filament.".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionInt::new(0)));
        }
        {
            let def = self
                .def
                .add("sparse_infill_line_width", ConfigOptionType::FloatOrPercent);
            def.label = "Sparse infill".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Line width of internal sparse infill. If expressed as a %, it will be computed over the nozzle diameter.".to_string();
            def.sidetext = "mm or %".to_string();
            def.ratio_over = "nozzle_diameter".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5027
            def.max = 1000_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5028
            def.max_literal = 10_f64;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5030
            def.set_default_value(Box::new(ConfigOptionFloatOrPercent::new(0., false)));
        }
        {
            let def = self
                .def
                .add("infill_wall_overlap", ConfigOptionType::Percent);
            def.label = "Infill/wall overlap".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "This allows the infill area to be enlarged slightly to overlap with walls for better bonding. The percentage value is relative to the inner wall line width. Set this value to ~10-15% to minimize potential over extrusion and accumulation of material resulting in rough top surfaces.".to_string();
            def.sidetext = "%".to_string();
            def.ratio_over = "inner_wall_line_width".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5040
            def.set_default_value(Box::new(ConfigOptionPercent::new(15_f64)));
        }
        {
            let def = self
                .def
                .add("top_bottom_infill_wall_overlap", ConfigOptionType::Percent);
            def.label = "Top/Bottom solid infill/wall overlap".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Top solid infill area is enlarged slightly to overlap with wall for better bonding and to minimize the appearance of pinholes where the top infill meets the walls. A value of 25-30% is a good starting point, minimizing the appearance of pinholes. The percentage value is relative to the inner wall line width.".to_string();
            def.sidetext = "%".to_string();
            def.ratio_over = "inner_wall_line_width".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5053
            def.set_default_value(Box::new(ConfigOptionPercent::new(25_f64)));
        }
        {
            let def = self
                .def
                .add("sparse_infill_speed", ConfigOptionType::Floats);
            def.label = "Sparse infill".to_string();
            def.category = "Speed".to_string();
            def.tooltip = "This is the speed for internal sparse infill.".to_string();
            def.sidetext = "mm/s".to_string();
            def.min = 1_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5063
            v.values.push(100_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("inherits", ConfigOptionType::String);
            def.label = "Inherits profile".to_string();
            def.tooltip = "Name of parent profile.".to_string();
            def.full_width = true;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5069
            def.height = 5;
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
            def.cli = NOCLI.to_string();
        }
        {
            let def = self.def.add("inherits_group", ConfigOptionType::Strings);
            def.set_default_value(Box::new(ConfigOptionStrings::new()));
            def.cli = NOCLI.to_string();
        }
        {
            let def = self.def.add("interface_shells", ConfigOptionType::Bool);
            def.label = "Interface shells".to_string();
            def.tooltip = "Force the generation of solid shells between adjacent materials/volumes. Useful for multi-extruder prints with translucent materials or manual soluble support material.".to_string();
            def.category = "Quality".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("mmu_segmented_region_max_width", ConfigOptionType::Float);
            def.label = "Maximum width of a segmented region".to_string();
            def.tooltip =
                "Maximum width of a segmented region. A value of 0 disables this feature."
                    .to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.category = "Advanced".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5095
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.)));
        }
        {
            let def = self.def.add(
                "mmu_segmented_region_interlocking_depth",
                ConfigOptionType::Float,
            );
            def.label = "Interlocking depth of a segmented region".to_string();
            def.tooltip = "Interlocking depth of a segmented region. On every other layer, the segmented region is limited to this depth. It will be ignored if \"interlocking_beam\" is enabled. Zero disables this feature.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.category = "Advanced".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5106
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.)));
        }
        {
            let def = self.def.add("interlocking_beam", ConfigOptionType::Bool);
            def.label = "Use beam interlocking".to_string();
            def.tooltip = "Generate interlocking beam structure at the locations where different filaments touch. This improves the adhesion between filaments, especially models printed in different materials.".to_string();
            def.category = "Advanced".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("interlocking_beam_width", ConfigOptionType::Float);
            def.label = "Interlocking beam width".to_string();
            def.tooltip = "The width of the interlocking structure beams.".to_string();
            def.sidetext = "mm".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5119
            def.min = 0.01;
            def.category = "Advanced".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5122
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.8)));
        }
        {
            let def = self
                .def
                .add("interlocking_orientation", ConfigOptionType::Float);
            def.label = "Interlocking direction".to_string();
            def.tooltip = "Orientation of interlock beams.".to_string();
            def.sidetext = "°".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5129
            def.max = 360_f32;
            def.category = "Advanced".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5132
            def.set_default_value(Box::new(ConfigOptionFloat::new(22.5)));
        }
        {
            let def = self
                .def
                .add("interlocking_beam_layer_count", ConfigOptionType::Int);
            def.label = "Interlocking beam layers".to_string();
            def.tooltip = "The height of the beams of the interlocking structure, measured in number of layers. Less layers is stronger, but more prone to defects.".to_string();
            def.min = 1_f32;
            def.category = "Advanced".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionInt::new(2)));
        }
        {
            let def = self.def.add("interlocking_depth", ConfigOptionType::Int);
            def.label = "Interlocking depth".to_string();
            def.tooltip = "The distance from the boundary between filaments to generate interlocking structure, measured in cells. Too few cells will result in poor adhesion.".to_string();
            def.min = 1_f32;
            def.category = "Advanced".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionInt::new(2)));
        }
        {
            let def = self
                .def
                .add("interlocking_boundary_avoidance", ConfigOptionType::Int);
            def.label = "Interlocking boundary avoidance".to_string();
            def.tooltip = "The distance from the outside of a model where interlocking structures will not be generated, measured in cells.".to_string();
            def.min = 0_f32;
            def.category = "Advanced".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionInt::new(2)));
        }
        {
            let def = self.def.add(
                "calib_flowrate_topinfill_special_order",
                ConfigOptionType::Bool,
            );
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("ironing_type", ConfigOptionType::Enum);
            def.label = "Ironing type".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Ironing uses a small flow to print at the same height of a surface to make flat surfaces smoother. This setting controls which layers are being ironed.".to_string();
            def.enum_keys_map = keys_map_ironing_type();
            def.enum_values.push("no ironing".to_string());
            def.enum_values.push("top".to_string());
            def.enum_values.push("topmost".to_string());
            def.enum_values.push("solid".to_string());
            def.enum_labels.push("No ironing".to_string());
            def.enum_labels.push("All top surfaces".to_string());
            def.enum_labels.push("Topmost surface only".to_string());
            def.enum_labels.push("All solid layers".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionEnum::new(IroningType::NoIroning)));
        }
        {
            let def = self.def.add("ironing_pattern", ConfigOptionType::Enum);
            def.label = "Ironing Pattern".to_string();
            def.tooltip = "The pattern that will be used when ironing.".to_string();
            def.category = "Quality".to_string();
            def.enum_keys_map = keys_map_infill_pattern();
            def.enum_values.push("rectilinear".to_string());
            def.enum_values.push("concentric".to_string());
            def.enum_labels.push("Rectilinear".to_string());
            def.enum_labels.push("Concentric".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionEnum::new(
                InfillPattern::ipRectilinear,
            )));
        }
        {
            let def = self.def.add("ironing_flow", ConfigOptionType::Percent);
            def.label = "Ironing flow".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "This is the amount of material to be extruded during ironing. It is relative to the flow of normal layer height. Too high a value will result in overextrusion on the surface.".to_string();
            def.sidetext = "%".to_string();
            def.ratio_over = "layer_height".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5198
            def.max = 100_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5200
            def.set_default_value(Box::new(ConfigOptionPercent::new(10_f64)));
        }
        {
            let def = self.def.add("ironing_spacing", ConfigOptionType::Float);
            def.label = "Ironing line spacing".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "This is the distance between the lines used for ironing.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.max = 1_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5210
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.1)));
        }
        {
            let def = self.def.add("ironing_inset", ConfigOptionType::Float);
            def.label = "Ironing inset".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "The distance to keep from the edges. A value of 0 sets this to half of the nozzle diameter.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5218
            def.max = 100_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(0_f64)));
        }
        {
            let def = self.def.add("ironing_speed", ConfigOptionType::Float);
            def.label = "Ironing speed".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "This is the print speed for ironing lines.".to_string();
            def.sidetext = "mm/s".to_string();
            def.min = 1_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5229
            def.set_default_value(Box::new(ConfigOptionFloat::new(20_f64)));
        }
        {
            let def = self.def.add("ironing_angle", ConfigOptionType::Float);
            def.label = "Ironing angle offset".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "The angle of ironing lines offset from the top surface.".to_string();
            def.sidetext = "°".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5237
            def.max = 359_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(0_f64)));
        }
        {
            let def = self.def.add("ironing_angle_fixed", ConfigOptionType::Bool);
            def.label = "Fixed ironing angle".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Use a fixed absolute angle for ironing.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("ironing_expansion", ConfigOptionType::Float);
            def.label = "Ironing expansion".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Expand or contract the ironing area.".to_string();
            def.sidetext = "mm".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5253
            def.min = -100_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5254
            def.max = 100_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Expert;
            def.set_default_value(Box::new(ConfigOptionFloat::new(0_f64)));
        }
        {
            let def = self.def.add("zaa_enabled", ConfigOptionType::Bool);
            def.label = "Z contouring enabled".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Enable Z-layer contouring (aka Z-layer anti-aliasing).".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Expert;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("zaa_minimize_perimeter_height", ConfigOptionType::Float);
            def.label = "Minimize wall height angle".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Reduce the height of top-surface perimeters to match the model edge height.\nAffects perimeters with a slope greater than this angle (degrees).\nA reasonable value is 35. Set to 0 to disable.".to_string();
            def.sidetext = "°".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5273
            def.max = 90_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Expert;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5275
            def.set_default_value(Box::new(ConfigOptionFloat::new(35_f64)));
        }
        {
            let def = self
                .def
                .add("zaa_dont_alternate_fill_direction", ConfigOptionType::Bool);
            def.label = "Don't alternate fill direction".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Disable alternating fill direction when using Z contouring.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Expert;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("zaa_min_z", ConfigOptionType::Float);
            def.label = "Minimum Z height".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Minimum Z-layer height.\nAlso controls the slicing plane.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5291
            def.max = 100_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Expert;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5293
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.05)));
        }
        {
            let def = self.def.add("layer_change_gcode", ConfigOptionType::String);
            def.label = "Layer change G-code".to_string();
            def.tooltip =
                "This G-code is inserted at every layer change after the Z lift.".to_string();
            def.multiline = true;
            def.full_width = true;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5300
            def.height = 5;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
        }
        {
            let def = self.def.add("time_lapse_gcode", ConfigOptionType::String);
            def.label = "Timelapse G-code".to_string();
            def.multiline = true;
            def.full_width = true;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5308
            def.height = 5;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
        }
        {
            let def = self
                .def
                .add("wrapping_detection_gcode", ConfigOptionType::String);
            def.label = "Clumping detection G-code".to_string();
            def.multiline = true;
            def.full_width = true;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5316
            def.height = 5;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionString::new("".to_string())));
        }
        {
            let def = self
                .def
                .add("emit_machine_limits_to_gcode", ConfigOptionType::Bool);
            def.label = "Emit limits to G-code".to_string();
            def.category = "Machine limits".to_string();
            def.tooltip = "If enabled, the machine limits will be emitted to G-code file.\nThis option will be ignored if the G-code flavor is set to Klipper.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(true)));
        }
        {
            let def = self
                .def
                .add("machine_pause_gcode", ConfigOptionType::String);
            def.label = "Pause G-code".to_string();
            def.tooltip = "This G-code will be used as a code for the pause print. Users can insert pause G-code in the G-code viewer.".to_string();
            def.multiline = true;
            def.full_width = true;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5333
            def.height = 12;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
        }
        {
            let def = self
                .def
                .add("template_custom_gcode", ConfigOptionType::String);
            def.label = "Custom G-code".to_string();
            def.tooltip = "This G-code will be used as a custom code.".to_string();
            def.multiline = true;
            def.full_width = true;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5342
            def.height = 12;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
        }
        {
            let def = self.def.add(
                "small_area_infill_flow_compensation",
                ConfigOptionType::Bool,
            );
            def.label = "Small area flow compensation (beta)".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Enable flow compensation for small infill areas.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add(
                "small_area_infill_flow_compensation_model",
                ConfigOptionType::Strings,
            );
            def.label = "Flow Compensation Model".to_string();
            def.tooltip = "Flow Compensation Model, used to adjust the flow for small infill areas. The model is expressed as a comma separated pair of values for extrusion length and flow correction factor. Each pair is on a separate line, followed by a semicolon, in the following format: \"1.234, 5.678;\"".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.gui_flags = "serialized".to_string();
            def.multiline = true;
            def.full_width = true;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5364
            def.height = 15;
            let mut v = ConfigOptionStrings::new();
            v.values.push("0,0".to_string());
            v.values.push("\n0.2,0.4444".to_string());
            v.values.push("\n0.4,0.6145".to_string());
            v.values.push("\n0.6,0.7059".to_string());
            v.values.push("\n0.8,0.7619".to_string());
            v.values.push("\n1.5,0.8571".to_string());
            v.values.push("\n2,0.8889".to_string());
            v.values.push("\n3,0.9231".to_string());
            v.values.push("\n5,0.9520".to_string());
            v.values.push("\n10,1".to_string());
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("has_scarf_joint_seam", ConfigOptionType::Bool);
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        // The XYZE machine-limit options (`PrintConfig.cpp:5371-5442`): the vendored body
        // declares a local `struct AxisDefault`, a four-element table of them, and then adds
        // three options per axis inside one loop, so twelve registry rows come from one block
        // of source. Ported as the same local table and the same loop rather than as twelve
        // unrolled rows, because the vendored source writes it once.
        //
        // `full_label` and `tooltip` are built with `boost::format` over the upper-cased axis
        // name, which is why each is followed in the vendored body by a run of discarded
        // `(void)L("Maximum speed X")` calls: those exist only so `xgettext` sees the twelve
        // composed strings as extraction candidates, and they assign nothing. The composed
        // string is what the oracle's `ConfigOptionDef` holds, and it is what is built here.
        {
            /// `struct AxisDefault` (`PrintConfig.cpp:5372-5377`), a local of the vendored body.
            struct AxisDefault {
                name: &'static str,
                max_feedrate: [f64; 2],
                max_acceleration: [f64; 2],
                max_jerk: [f64; 2],
            }
            // FIDELITY(port): the vendored `std::vector<AxisDefault>`'s own order
            // (`PrintConfig.cpp:5378-5383`), x then y then z then e. It reaches output: each
            // iteration calls `ConfigDef::add`, which assigns the next
            // `serialization_key_ordinal`, so reordering this table would reorder
            // `by_serialization_key_ordinal`.
            let axes = [
                AxisDefault {
                    name: "x",
                    // PORT SRC: src/libslic3r/PrintConfig.cpp:5380
                    max_feedrate: [500., 200.],
                    // PORT SRC: src/libslic3r/PrintConfig.cpp:5380
                    max_acceleration: [1000., 1000.],
                    // PORT SRC: src/libslic3r/PrintConfig.cpp:5380
                    max_jerk: [10., 10.],
                },
                AxisDefault {
                    name: "y",
                    // PORT SRC: src/libslic3r/PrintConfig.cpp:5381
                    max_feedrate: [500., 200.],
                    // PORT SRC: src/libslic3r/PrintConfig.cpp:5381
                    max_acceleration: [1000., 1000.],
                    // PORT SRC: src/libslic3r/PrintConfig.cpp:5381
                    max_jerk: [10., 10.],
                },
                AxisDefault {
                    name: "z",
                    // PORT SRC: src/libslic3r/PrintConfig.cpp:5382
                    max_feedrate: [12., 12.],
                    // PORT SRC: src/libslic3r/PrintConfig.cpp:5382
                    max_acceleration: [500., 200.],
                    // PORT SRC: src/libslic3r/PrintConfig.cpp:5382
                    max_jerk: [0.2, 0.4],
                },
                AxisDefault {
                    name: "e",
                    // PORT SRC: src/libslic3r/PrintConfig.cpp:5383
                    max_feedrate: [120., 120.],
                    // PORT SRC: src/libslic3r/PrintConfig.cpp:5383
                    max_acceleration: [5000., 5000.],
                    // PORT SRC: src/libslic3r/PrintConfig.cpp:5383
                    max_jerk: [2.5, 2.5],
                },
            ];
            for axis in axes.iter() {
                let axis_upper = axis.name.to_uppercase();
                // The machine feedrate limits for XYZE axes (M203).
                {
                    let def = self.def.add(
                        &format!("machine_max_speed_{}", axis.name),
                        ConfigOptionType::Floats,
                    );
                    def.full_label = format!("Maximum speed {axis_upper}");
                    def.category = "Machine limits".to_string();
                    def.readonly = false;
                    def.tooltip = format!("Maximum speed of the {axis_upper} axis");
                    def.sidetext = "mm/s".to_string();
                    def.min = 0_f32;
                    def.mode = slic3r_config::config::ConfigOptionMode::Simple;
                    let mut v = ConfigOptionFloats::new();
                    v.values.extend(axis.max_feedrate);
                    def.set_default_value(Box::new(v));
                }
                // The machine acceleration limits for XYZE axes (M201).
                {
                    let def = self.def.add(
                        &format!("machine_max_acceleration_{}", axis.name),
                        ConfigOptionType::Floats,
                    );
                    def.full_label = format!("Maximum acceleration {axis_upper}");
                    def.category = "Machine limits".to_string();
                    def.readonly = false;
                    def.tooltip = format!("Maximum acceleration of the {axis_upper} axis");
                    def.sidetext = "mm/s\u{b2}".to_string();
                    def.min = 0_f32;
                    def.mode = slic3r_config::config::ConfigOptionMode::Simple;
                    let mut v = ConfigOptionFloats::new();
                    v.values.extend(axis.max_acceleration);
                    def.set_default_value(Box::new(v));
                }
                // The machine jerk limits for XYZE axes (M205).
                {
                    let def = self.def.add(
                        &format!("machine_max_jerk_{}", axis.name),
                        ConfigOptionType::Floats,
                    );
                    def.full_label = format!("Maximum jerk {axis_upper}");
                    def.category = "Machine limits".to_string();
                    def.readonly = false;
                    def.tooltip = format!("Maximum jerk of the {axis_upper} axis");
                    // PORT NOTE: `PrintConfig.cpp:5438` really is `L("mm/s")` on the *jerk*
                    // option, not `mm/s` with a unit of its own; transcribed as the vendored
                    // line writes it rather than corrected, and the oracle agrees.
                    def.sidetext = "mm/s".to_string();
                    def.min = 0_f32;
                    def.mode = slic3r_config::config::ConfigOptionMode::Simple;
                    let mut v = ConfigOptionFloats::new();
                    v.values.extend(axis.max_jerk);
                    def.set_default_value(Box::new(v));
                }
            }
        }
        {
            let def = self
                .def
                .add("machine_max_junction_deviation", ConfigOptionType::Floats);
            def.full_label = "Maximum Junction Deviation".to_string();
            def.category = "Machine limits".to_string();
            def.tooltip = "Maximum junction deviation (M205 J, only apply if JD > 0 for Marlin Firmware\nIf your Marlin 2 printer uses Classic Jerk set this value to 0.)".to_string();
            def.sidetext = "mm".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5449
            def.min = 0.;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5450
            def.max = 0.3;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5452
            v.values.push(0.01);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("machine_min_extruding_rate", ConfigOptionType::Floats);
            def.full_label = "Minimum speed for extruding".to_string();
            def.category = "Machine limits".to_string();
            def.tooltip = "Minimum speed for extruding (M205 S)".to_string();
            def.sidetext = "mm/s".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5462
            v.values.push(0.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5462
            v.values.push(0.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("machine_min_travel_rate", ConfigOptionType::Floats);
            def.full_label = "Minimum travel speed".to_string();
            def.category = "Machine limits".to_string();
            def.tooltip = "Minimum travel speed (M205 T)".to_string();
            def.sidetext = "mm/s".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5472
            v.values.push(0.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5472
            v.values.push(0.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("machine_max_force_Y", ConfigOptionType::Float);
            def.full_label = "Maximum force of the Y axis".to_string();
            def.category = "Machine limits".to_string();
            def.readonly = false;
            def.tooltip = "The allowed maximum output force of Y axis".to_string();
            def.sidetext = "N".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            def.set_default_value(Box::new(ConfigOptionFloat::new(0_f64)));
        }
        {
            let def = self.def.add("machine_bed_mass_Y", ConfigOptionType::Float);
            def.full_label = "Bed mass of the Y axis".to_string();
            def.category = "Machine limits".to_string();
            def.readonly = false;
            def.tooltip = "The machine bed mass load of Y axis".to_string();
            def.sidetext = "g".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            def.set_default_value(Box::new(ConfigOptionFloat::new(0_f64)));
        }
        {
            let def = self
                .def
                .add("machine_max_printed_mass", ConfigOptionType::Float);
            def.full_label = "The allowed max printed mass".to_string();
            def.category = "Machine limits".to_string();
            def.readonly = false;
            def.tooltip = "The allowed max printed mass on a plate".to_string();
            def.sidetext = "g".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            def.set_default_value(Box::new(ConfigOptionFloat::new(0_f64)));
        }
        {
            let def = self.def.add(
                "machine_max_acceleration_extruding",
                ConfigOptionType::Floats,
            );
            def.full_label = "Maximum acceleration for extruding".to_string();
            def.category = "Machine limits".to_string();
            def.tooltip = "Maximum acceleration for extruding (M204 P)".to_string();
            def.sidetext = "mm/s²".to_string();
            def.min = 0_f32;
            def.readonly = false;
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5518
            v.values.push(1500.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5518
            v.values.push(1250.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "machine_max_acceleration_retracting",
                ConfigOptionType::Floats,
            );
            def.full_label = "Maximum acceleration for retracting".to_string();
            def.category = "Machine limits".to_string();
            def.tooltip = "Maximum acceleration for retracting (M204 R)".to_string();
            def.sidetext = "mm/s²".to_string();
            def.min = 0_f32;
            def.readonly = false;
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5530
            v.values.push(1500.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5530
            v.values.push(1250.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("machine_max_acceleration_travel", ConfigOptionType::Floats);
            def.full_label = "Maximum acceleration for travel".to_string();
            def.category = "Machine limits".to_string();
            def.tooltip = "Maximum acceleration for travel (M204 T), it only applies to Marlin 2."
                .to_string();
            def.sidetext = "mm/s²".to_string();
            def.min = 0_f32;
            def.readonly = false;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5541
            v.values.push(0.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5541
            v.values.push(0.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("resonance_avoidance", ConfigOptionType::Bool);
            def.label = "Resonance avoidance".to_string();
            def.tooltip = "By reducing the speed of the outer wall to avoid the resonance zone of the printer, ringing on the surface of the model are avoided.\nPlease turn this option off when testing ringing.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("min_resonance_avoidance_speed", ConfigOptionType::Float);
            def.label = "Min".to_string();
            def.tooltip = "Minimum speed of resonance avoidance.".to_string();
            def.sidetext = "mm/s".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5558
            def.set_default_value(Box::new(ConfigOptionFloat::new(70_f64)));
        }
        {
            let def = self
                .def
                .add("max_resonance_avoidance_speed", ConfigOptionType::Float);
            def.label = "Max".to_string();
            def.tooltip = "Maximum speed of resonance avoidance.".to_string();
            def.sidetext = "mm/s".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5566
            def.set_default_value(Box::new(ConfigOptionFloat::new(120_f64)));
        }
        {
            let def = self.def.add("input_shaping_emit", ConfigOptionType::Bool);
            def.label = "Emit input shaping".to_string();
            def.tooltip = "Override firmware input shaping settings.\nIf disabled, firmware settings are used.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Expert;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("input_shaping_type", ConfigOptionType::Enum);
            def.label = "Input shaper type".to_string();
            def.tooltip = "Choose the input shaper algorithm.\nDefault uses the firmware default settings.\nDisable turns off input shaping in the firmware.".to_string();
            def.enum_keys_map = keys_map_input_shaper_type();
            def.enum_values.push("Default".to_string());
            def.enum_values.push("MZV".to_string());
            def.enum_values.push("ZV".to_string());
            def.enum_values.push("ZVD".to_string());
            def.enum_values.push("ZVDD".to_string());
            def.enum_values.push("ZVDDD".to_string());
            def.enum_values.push("EI".to_string());
            def.enum_values.push("EI2".to_string());
            def.enum_values.push("2HUMP_EI".to_string());
            def.enum_values.push("EI3".to_string());
            def.enum_values.push("3HUMP_EI".to_string());
            def.enum_values.push("DAA".to_string());
            def.enum_values.push("Disable".to_string());
            def.enum_labels.push("Default".to_string());
            def.enum_labels.push("MZV".to_string());
            def.enum_labels.push("ZV".to_string());
            def.enum_labels.push("ZVD".to_string());
            def.enum_labels.push("ZVDD".to_string());
            def.enum_labels.push("ZVDDD".to_string());
            def.enum_labels.push("EI".to_string());
            def.enum_labels.push("EI2".to_string());
            def.enum_labels.push("2HUMP_EI".to_string());
            def.enum_labels.push("EI3".to_string());
            def.enum_labels.push("3HUMP_EI".to_string());
            def.enum_labels.push("DAA".to_string());
            def.enum_labels.push("Disable".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Expert;
            def.set_default_value(Box::new(ConfigOptionEnum::new(InputShaperType::Default)));
        }
        {
            let def = self
                .def
                .add("input_shaping_freq_x", ConfigOptionType::Float);
            def.label = "X".to_string();
            def.tooltip = "Resonant frequency for the X axis input shaper.\nZero will use the firmware frequency.\nTo disable input shaping, use the Disable type.\nRRF: X and Y values are equal.".to_string();
            def.sidetext = "Hz".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5589
            def.max = 1000_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Expert;
            def.set_default_value(Box::new(ConfigOptionFloat::new(0_f64)));
        }
        {
            let def = self
                .def
                .add("input_shaping_freq_y", ConfigOptionType::Float);
            def.label = "Y".to_string();
            def.tooltip = "Resonant frequency for the Y axis input shaper.\nZero will use the firmware frequency.\nTo disable input shaping, use the Disable type.".to_string();
            def.sidetext = "Hz".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5598
            def.max = 1000_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Expert;
            def.set_default_value(Box::new(ConfigOptionFloat::new(0_f64)));
        }
        {
            let def = self
                .def
                .add("input_shaping_damp_x", ConfigOptionType::Float);
            def.label = "X".to_string();
            def.tooltip = "Damping ratio for the X axis input shaper.\nZero will use the firmware damping ratio.\nTo disable input shaping, use the Disable type.\nRRF: X and Y values are equal.".to_string();
            def.min = 0_f32;
            def.max = 1_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Expert;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5608
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.1)));
        }
        {
            let def = self
                .def
                .add("input_shaping_damp_y", ConfigOptionType::Float);
            def.label = "Y".to_string();
            def.tooltip = "Damping ratio for the Y axis input shaper.\nZero will use the firmware damping ratio.\nTo disable input shaping, use the Disable type.".to_string();
            def.min = 0_f32;
            def.max = 1_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Expert;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5616
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.1)));
        }
        {
            let def = self.def.add("fan_max_speed", ConfigOptionType::Floats);
            def.label = "Fan speed".to_string();
            def.tooltip = "The part cooling fan speed may be increased when auto cooling is enabled. This is the maximum speed for the part cooling fan.".to_string();
            def.sidetext = "%".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5623
            def.max = 100_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5625
            v.values.push(100_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("max_layer_height", ConfigOptionType::Floats);
            def.label = "Max".to_string();
            def.tooltip = "The highest printable layer height for the extruder. Used to limit the maximum layer height when enable adaptive layer height.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5634
            v.values.push(0.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "max_volumetric_extrusion_rate_slope",
                ConfigOptionType::Float,
            );
            def.label = "Extrusion rate smoothing".to_string();
            def.tooltip = "This parameter smooths out sudden extrusion rate changes that happen when the printer transitions from printing a high flow (high speed/larger width) extrusion to a lower flow (lower speed/smaller width) extrusion and vice versa.\n\nIt defines the maximum rate by which the extruded volumetric flow in mm³/s can change over time. Higher values mean higher extrusion rate changes are allowed, resulting in faster speed transitions.\n\nA value of 0 disables the feature.\n\nFor a high speed, high flow direct drive printer (like the Bambu lab or Voron) this value is usually not needed. However it can provide some marginal benefit in certain cases where feature speeds vary greatly. For example, when there are aggressive slowdowns due to overhangs. In these cases a high value of around 300-350 mm³/s² is recommended as this allows for just enough smoothing to assist pressure advance achieve a smoother flow transition.\n\nFor slower printers without pressure advance, the value should be set much lower. A value of 10-15 mm³/s² is a good starting point for direct drive extruders and 5-10 mm³/s² for Bowden style.\n\nThis feature is known as Pressure Equalizer in Prusa slicer.\n\nNote: this parameter disables arc fitting.".to_string();
            def.sidetext = "mm³/s²".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(0_f64)));
        }
        {
            let def = self.def.add(
                "max_volumetric_extrusion_rate_slope_segment_length",
                ConfigOptionType::Float,
            );
            def.label = "Smoothing segment length".to_string();
            def.tooltip = "A lower value results in smoother extrusion rate transitions. However, this results in a significantly larger G-code file and more instructions for the printer to process.\n\nDefault value of 3 works well for most cases. If your printer is stuttering, increase this value to reduce the number of adjustments made.\n\nAllowed values: 0.5-5".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5663
            def.min = 0.5;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5664
            def.max = 5_f32;
            def.sidetext = "mm".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5667
            def.set_default_value(Box::new(ConfigOptionFloat::new(3.0)));
        }
        {
            let def = self.def.add(
                "extrusion_rate_smoothing_external_perimeter_only",
                ConfigOptionType::Bool,
            );
            def.label = "Apply only on external features".to_string();
            def.tooltip = "Applies extrusion rate smoothing only on external perimeters and overhangs. This can help reduce artefacts due to sharp speed transitions on externally visible overhangs without impacting the print speed of features that will not be visible to the user.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("fan_min_speed", ConfigOptionType::Floats);
            def.label = "Fan speed".to_string();
            def.tooltip = "Minimum speed for part cooling fan.".to_string();
            def.sidetext = "%".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5682
            def.max = 100_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5684
            v.values.push(20_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("additional_cooling_fan_speed", ConfigOptionType::Ints);
            def.label = "Fan speed".to_string();
            def.tooltip = "Speed of auxiliary part cooling fan. Auxiliary fan will run at this speed during printing except the first several layers which is defined by no cooling layers.\nPlease enable auxiliary_fan in printer settings to use this feature.\nG-code command: M106 P2 S(0-255)".to_string();
            def.sidetext = "%".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5692
            def.max = 100_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            let mut v = ConfigOptionInts::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "close_additional_fan_first_x_layers",
                ConfigOptionType::Ints,
            );
            def.label = "For the first".to_string();
            def.tooltip =
                "Set special auxiliary cooling fan for the first certain layers.".to_string();
            def.sidetext = "layers".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5701
            def.max = 1000_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            let mut v = ConfigOptionInts::new();
            v.values.push(1);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("additional_fan_full_speed_layer", ConfigOptionType::Ints);
            def.label = "Full fan speed at layer".to_string();
            def.tooltip = "Auxiliary fan speed will be ramped up linearly from layer \"For the first\" to maximum at layer \"Full fan speed at layer\".\n\"Full fan speed at layer\" will be ignored if lower than \"For the first\", in which case the fan will run at maximum allowed speed at layer \"For the first\" + 1.".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5710
            def.max = 1000_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            let mut v = ConfigOptionInts::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("first_x_layer_fan_speed", ConfigOptionType::Floats);
            def.label = "Fan speed".to_string();
            def.tooltip =
                "Special auxiliary cooling fan speed, effective only for the first x layers."
                    .to_string();
            def.sidetext = "%".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5719
            def.max = 100_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            let mut v = ConfigOptionFloats::new();
            v.values.push(0_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("min_layer_height", ConfigOptionType::Floats);
            def.label = "Min".to_string();
            def.tooltip = "The lowest printable layer height for the extruder. Used to limit the minimum layer height when enable adaptive layer height.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5730
            v.values.push(0.07);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("slow_down_min_speed", ConfigOptionType::Floats);
            def.label = "Min print speed".to_string();
            def.tooltip = "The minimum print speed to which the printer slows down to maintain the minimum layer time defined above when the slowdown for better layer cooling is enabled.".to_string();
            def.sidetext = "mm/s".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5739
            v.values.push(10.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("nozzle_diameter", ConfigOptionType::Floats);
            def.label = "Nozzle diameter".to_string();
            def.tooltip = "The diameter of nozzle.".to_string();
            def.sidetext = "mm".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5746
            def.max = 100_f32;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5747
            v.values.push(0.4);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("notes", ConfigOptionType::String);
            def.label = "Configuration notes".to_string();
            def.tooltip = "You can put here your personal notes. This text will be added to the G-code header comments.".to_string();
            def.multiline = true;
            def.full_width = true;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5755
            def.height = 13;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
        }
        {
            let def = self.def.add("host_type", ConfigOptionType::Enum);
            def.label = "Host Type".to_string();
            def.tooltip = "Orca Slicer can upload G-code files to a printer host. This field must contain the kind of the host.".to_string();
            def.enum_keys_map = keys_map_print_host_type();
            def.enum_values.push("prusalink".to_string());
            def.enum_values.push("prusaconnect".to_string());
            def.enum_values.push("octoprint".to_string());
            def.enum_values.push("duet".to_string());
            def.enum_values.push("ultimaker".to_string());
            def.enum_values.push("flashair".to_string());
            def.enum_values.push("astrobox".to_string());
            def.enum_values.push("repetier".to_string());
            def.enum_values.push("mks".to_string());
            def.enum_values.push("esp3d".to_string());
            def.enum_values.push("crealityprint".to_string());
            def.enum_values.push("obico".to_string());
            def.enum_values.push("flashforge".to_string());
            def.enum_values.push("simplyprint".to_string());
            def.enum_values.push("elegoolink".to_string());
            def.enum_values.push("3dprinteros".to_string());
            def.enum_values.push("moonraker".to_string());
            def.enum_labels.push("PrusaLink".to_string());
            def.enum_labels.push("PrusaConnect".to_string());
            def.enum_labels.push("Octo/Klipper".to_string());
            def.enum_labels.push("Duet".to_string());
            def.enum_labels.push("UltiMaker".to_string());
            def.enum_labels.push("FlashAir".to_string());
            def.enum_labels.push("AstroBox".to_string());
            def.enum_labels.push("Repetier".to_string());
            def.enum_labels.push("MKS".to_string());
            def.enum_labels.push("ESP3D".to_string());
            def.enum_labels.push("CrealityPrint".to_string());
            def.enum_labels.push("Obico".to_string());
            def.enum_labels.push("Flashforge".to_string());
            def.enum_labels.push("SimplyPrint".to_string());
            def.enum_labels.push("Elegoo Link".to_string());
            def.enum_labels.push("3DPrinterOS".to_string());
            def.enum_labels.push("Moonraker (Klipper)".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.cli = NOCLI.to_string();
            def.set_default_value(Box::new(ConfigOptionEnum::new(PrintHostType::htOctoPrint)));
        }
        {
            let def = self.def.add("nozzle_volume", ConfigOptionType::Floats);
            def.label = "Nozzle volume".to_string();
            def.tooltip = "Volume of nozzle between the filament cutter and the end of the nozzle"
                .to_string();
            def.sidetext = "mm³".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.readonly = false;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5809
            v.values.push(0.0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("cooling_tube_retraction", ConfigOptionType::Float);
            def.label = "Cooling tube position".to_string();
            def.tooltip = "Distance of the center-point of the cooling tube from the extruder tip."
                .to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5817
            def.set_default_value(Box::new(ConfigOptionFloat::new(91.5)));
        }
        {
            let def = self.def.add("cooling_tube_length", ConfigOptionType::Float);
            def.label = "Cooling tube length".to_string();
            def.tooltip = "Length of the cooling tube to limit space for cooling moves inside it."
                .to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5825
            def.set_default_value(Box::new(ConfigOptionFloat::new(5.)));
        }
        {
            let def = self
                .def
                .add("high_current_on_filament_swap", ConfigOptionType::Bool);
            def.label = "High extruder current on filament swap".to_string();
            def.tooltip = "It may be beneficial to increase the extruder motor current during the filament exchange sequence to allow for rapid ramming feed rates and to overcome resistance when loading a filament with an ugly shaped tip.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("parking_pos_retraction", ConfigOptionType::Float);
            def.label = "Filament parking position".to_string();
            def.tooltip = "Distance of the extruder tip from the position where the filament is parked when unloaded. This should match the value in printer firmware.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5842
            def.set_default_value(Box::new(ConfigOptionFloat::new(92.)));
        }
        {
            let def = self.def.add("extra_loading_move", ConfigOptionType::Float);
            def.label = "Extra loading distance".to_string();
            def.tooltip = "When set to zero, the distance the filament is moved from parking position during load is exactly the same as it was moved back during unload. When positive, it is loaded further, if negative, the loading move is shorter than unloading.".to_string();
            def.sidetext = "mm".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5851
            def.set_default_value(Box::new(ConfigOptionFloat::new(-2.)));
        }
        {
            let def = self.def.add("start_end_points", ConfigOptionType::Points);
            def.label = "Start end points".to_string();
            def.tooltip =
                "The start and end points which are from the cutter area to the excess chute."
                    .to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            def.readonly = true;
            let mut v = ConfigOptionPoints::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5859
            v.values.push(Vec2d::new(30_f64, -3_f64));
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5859
            v.values.push(Vec2d::new(54_f64, 245_f64));
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("reduce_infill_retraction", ConfigOptionType::Bool);
            def.label = "Reduce infill retraction".to_string();
            def.tooltip = "Don't retract when the travel is entirely within an infill area. That means the oozing can't been seen. This can reduce times of retraction for complex model and save printing time, but make slicing and G-code generating slower.\n\nNote that Z-hop is also not performed in areas where retraction is skipped.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("ooze_prevention", ConfigOptionType::Bool);
            def.label = "Enable".to_string();
            def.tooltip = "This option will drop the temperature of the inactive extruders to prevent oozing.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("filename_format", ConfigOptionType::String);
            def.label = "Filename format".to_string();
            def.tooltip = "Users can decide project file names when exporting.".to_string();
            def.full_width = true;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionString::new(
                "{input_filename_base}_{filament_type[initial_tool]}_{print_time}.gcode"
                    .to_string(),
            )));
        }
        {
            let def = self
                .def
                .add("make_overhang_printable", ConfigOptionType::Bool);
            def.label = "Make overhangs printable".to_string();
            def.category = "Quality".to_string();
            def.tooltip =
                "Modify the geometry to print overhangs without support material.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("make_overhang_printable_angle", ConfigOptionType::Float);
            def.label = "Make overhangs printable - Maximum angle".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Maximum angle of overhangs to allow after making more steep overhangs printable.90° will not change the model at all and allow any overhang, while 0 will replace all overhangs with conical material.".to_string();
            def.sidetext = "°".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5895
            def.min = 0.;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5896
            def.max = 90.;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5897
            def.set_default_value(Box::new(ConfigOptionFloat::new(55.)));
        }
        {
            let def = self
                .def
                .add("make_overhang_printable_hole_size", ConfigOptionType::Float);
            def.label = "Make overhangs printable - Hole area".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Maximum area of a hole in the base of the model before it's filled by conical material. A value of 0 will fill all the holes in the model base.".to_string();
            def.sidetext = "mm²".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5906
            def.min = 0.;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5907
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.)));
        }
        {
            let def = self.def.add("detect_overhang_wall", ConfigOptionType::Bool);
            def.label = "Detect overhang walls".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "This detects the overhang percentage relative to line width and uses a different speed to print. For 100% overhang, bridging speed is used.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(true)));
        }
        {
            let def = self
                .def
                .add("unsupported_wall_last", ConfigOptionType::Bool);
            def.label = "Print unsupported walls last".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Wall loops that lie entirely in mid air are printed once something can hold them:\nthey are extruded after the other walls of their island, innermost first, whatever the wall order is.\nA loop that only the bridges of this layer can anchor waits until those bridges are printed, while a loop running alongside a supported wall keeps its place before the infill, which needs it as an anchor.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("outer_wall_filament_id", ConfigOptionType::Int);
            def.gui_type = GuiType::IEnumOpen;
            def.label = "Outer walls".to_string();
            def.category = "Extruders".to_string();
            def.tooltip =
                "Filament to print outer walls.\n\"Default\" uses the active object/part filament."
                    .to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionInt::new(0)));
        }
        {
            let def = self
                .def
                .add("inner_wall_filament_id", ConfigOptionType::Int);
            def.gui_type = GuiType::IEnumOpen;
            def.label = "Inner walls".to_string();
            def.category = "Extruders".to_string();
            def.tooltip =
                "Filament to print inner walls.\n\"Default\" uses the active object/part filament."
                    .to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionInt::new(0)));
        }
        {
            let def = self
                .def
                .add("inner_wall_line_width", ConfigOptionType::FloatOrPercent);
            def.label = "Inner wall".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Line width of inner wall. If expressed as a %, it will be computed over the nozzle diameter.".to_string();
            def.sidetext = "mm or %".to_string();
            def.ratio_over = "nozzle_diameter".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5952
            def.max = 1000_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5953
            def.max_literal = 10_f64;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5955
            def.set_default_value(Box::new(ConfigOptionFloatOrPercent::new(0., false)));
        }
        {
            let def = self.def.add("inner_wall_speed", ConfigOptionType::Floats);
            def.label = "Inner wall".to_string();
            def.category = "Speed".to_string();
            def.tooltip = "This is the speed for inner walls.".to_string();
            def.sidetext = "mm/s".to_string();
            def.aliases.push("perimeter_feed_rate".to_string());
            def.min = 1_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5966
            v.values.push(60_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("wall_loops", ConfigOptionType::Int);
            def.label = "Wall loops".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "This is the number of walls per layer.".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5973
            def.max = 1000_f32;
            def.set_default_value(Box::new(ConfigOptionInt::new(2)));
        }
        {
            let def = self.def.add("alternate_extra_wall", ConfigOptionType::Bool);
            def.label = "Alternate extra wall".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "This setting adds an extra wall to every other layer. This way the infill gets wedged vertically between the walls, resulting in stronger prints.\n\nWhen this option is enabled, the ensure vertical shell thickness option needs to be disabled.\n\nUsing lightning infill together with this option is not recommended as there is limited infill to anchor the extra perimeters to.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("post_process", ConfigOptionType::Strings);
            def.label = "Post-processing Scripts".to_string();
            def.tooltip = "If you want to process the output G-code through custom scripts, just list their absolute paths here. Separate multiple scripts with a semicolon. Scripts will be passed the absolute path to the G-code file as the first argument, and they can access the Orca Slicer config settings by reading environment variables.".to_string();
            def.gui_flags = "serialized".to_string();
            def.multiline = true;
            def.full_width = true;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:5994
            def.height = 6;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionStrings::new()));
        }
        {
            let def = self.def.add(
                "process_change_extrusion_role_gcode",
                ConfigOptionType::String,
            );
            def.label = "Change extrusion role G-code (process)".to_string();
            def.tooltip = "This G-code is inserted when the extrusion role is changed. It runs after the machine and filament extrusion role G-code.".to_string();
            def.multiline = true;
            def.full_width = true;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6003
            def.height = 5;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
        }
        {
            let def = self.def.add("plugins", ConfigOptionType::Strings);
            def.label = "Plugins Used".to_string();
            def.tooltip =
                "Plugin capabilities referenced by this preset, stored as name;uuid;capability."
                    .to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            def.set_default_value(Box::new(ConfigOptionStrings::new()));
        }
        {
            let def = self
                .def
                .add("slicing_pipeline_plugin", ConfigOptionType::Strings);
            def.label = "Slicing Pipeline Plugin".to_string();
            def.tooltip = "Python plugins invoked at each slicing pipeline step to read and modify intermediate slicing data, including a final G-code post-processing step. Research/experimental.".to_string();
            def.gui_type = GuiType::PluginPicker;
            def.plugin_type = "slicing-pipeline".to_string();
            def.full_width = true;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionStrings::new()));
        }
        {
            let def = self.def.add("printer_model", ConfigOptionType::String);
            def.label = "Printer type".to_string();
            def.tooltip = "Type of the printer.".to_string();
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
            def.cli = NOCLI.to_string();
        }
        {
            let def = self.def.add("printer_notes", ConfigOptionType::String);
            def.label = "Printer notes".to_string();
            def.tooltip = "You can put your notes regarding the printer here.".to_string();
            def.multiline = true;
            def.full_width = true;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6034
            def.height = 13;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
        }
        {
            let def = self.def.add("printer_variant", ConfigOptionType::String);
            def.label = "Printer variant".to_string();
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
            def.cli = NOCLI.to_string();
        }
        {
            let def = self.def.add("print_settings_id", ConfigOptionType::String);
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
        }
        {
            let def = self
                .def
                .add("printer_settings_id", ConfigOptionType::String);
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
        }
        {
            let def = self
                .def
                .add("raft_contact_distance", ConfigOptionType::Float);
            def.label = "Raft contact Z distance".to_string();
            def.category = "Support".to_string();
            def.tooltip = "Z gap between raft and object. If Support Top Z Distance is 0, this value is ignored and the object is printed in direct contact with the raft (no gap).".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6061
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.1)));
        }
        {
            let def = self.def.add("raft_expansion", ConfigOptionType::Float);
            def.label = "Raft expansion".to_string();
            def.category = "Support".to_string();
            def.tooltip = "This expands all raft layers in XY plane.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6070
            def.set_default_value(Box::new(ConfigOptionFloat::new(1.5)));
        }
        {
            let def = self
                .def
                .add("raft_first_layer_density", ConfigOptionType::Percent);
            def.label = "First layer density".to_string();
            def.category = "Support".to_string();
            def.tooltip = "This is the density of the first raft or support layer.".to_string();
            def.sidetext = "%".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6077
            def.min = 10_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6078
            def.max = 100_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6080
            def.set_default_value(Box::new(ConfigOptionPercent::new(90_f64)));
        }
        {
            let def = self
                .def
                .add("raft_first_layer_expansion", ConfigOptionType::Float);
            def.label = "First layer expansion".to_string();
            def.category = "Support".to_string();
            def.tooltip =
                "This expands the first raft or support layer to improve bed adhesion.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6090
            def.set_default_value(Box::new(ConfigOptionFloat::new(2.0)));
        }
        {
            let def = self.def.add("raft_layers", ConfigOptionType::Int);
            def.label = "Raft layers".to_string();
            def.category = "Support".to_string();
            def.tooltip = "Object will be raised by this number of support layers. Use this function to avoid warping when printing ABS.".to_string();
            def.sidetext = "layers".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6099
            def.max = 100_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionInt::new(
                slic3r_printconfigconstants::print_config_constants::INITIAL_RAFT_LAYERS,
            )));
        }
        {
            let def = self.def.add("resolution", ConfigOptionType::Float);
            def.label = "Resolution".to_string();
            def.tooltip = "The G-code path is generated after simplifying the contour of models to avoid too many points and G-code lines. Smaller values mean higher resolution and more time required to slice.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6109
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.01)));
        }
        {
            let def = self
                .def
                .add("retraction_minimum_travel", ConfigOptionType::Floats);
            def.label = "Travel distance threshold".to_string();
            def.tooltip =
                "Only trigger retraction when the travel distance is longer than this threshold."
                    .to_string();
            def.sidetext = "mm".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6116
            v.values.push(2.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("retract_before_wipe", ConfigOptionType::Percents);
            def.label = "Retract amount before wipe".to_string();
            def.tooltip = "This is the length of fast retraction before a wipe, relative to retraction length.".to_string();
            def.sidetext = "%".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionPercents { values: Vec::new() };
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6123
            v.values.push(100_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("retract_after_wipe", ConfigOptionType::Percents);
            def.label = "Retract amount after wipe".to_string();
            def.tooltip = "This is the length of fast retraction after wipe, relative to retraction length.\nThe value will be clamped by 100% minus the retract amount before the wipe value.".to_string();
            def.sidetext = "%".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Expert;
            let mut v = ConfigOptionPercents { values: Vec::new() };
            v.values.push(0_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("retract_when_changing_layer", ConfigOptionType::Bools);
            def.label = "Retract on layer change".to_string();
            def.tooltip = "This forces a retraction on layer changes.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionBools::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("retraction_length", ConfigOptionType::Floats);
            def.label = "Length".to_string();
            def.full_label = "Retraction Length".to_string();
            def.tooltip = "Some amount of material in extruder is pulled back to avoid ooze during long travel. Set zero to disable retraction.".to_string();
            def.sidetext = "mm".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6148
            v.values.push(0.8);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("enable_long_retraction_when_cut", ConfigOptionType::Int);
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            def.set_default_value(Box::new(ConfigOptionInt::new(0)));
        }
        {
            let def = self
                .def
                .add("long_retractions_when_cut", ConfigOptionType::Bools);
            def.label = "Long retraction when cut (beta)".to_string();
            def.tooltip = "Experimental feature: Retracting and cutting off the filament at a longer distance during changes to minimize purge. While this reduces flush significantly, it may also raise the risk of nozzle clogs or other printing problems.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            let mut v = ConfigOptionBools::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("retraction_distances_when_cut", ConfigOptionType::Floats);
            def.label = "Retraction distance when cut".to_string();
            def.tooltip = "Experimental feature: Retraction length before cutting off during filament change.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6165
            def.min = 10_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6166
            def.max = 18_f32;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6167
            v.values.push(18_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("long_retractions_when_ec", ConfigOptionType::Bools);
            def.label = "Long retraction when extruder change".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionBoolsNullable::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("retraction_distances_when_ec", ConfigOptionType::Floats);
            def.label = "Retraction distance when extruder change".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6180
            def.max = 10_f32;
            def.sidetext = "mm".to_string();
            let mut v = ConfigOptionFloatsNullable::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6182
            v.values.push(10_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("retract_length_toolchange", ConfigOptionType::Floats);
            def.label = "Retraction Length (Toolchange)".to_string();
            def.tooltip = "When retraction is triggered before changing tool, filament is pulled back by the specified amount (the length is measured on raw filament, before it enters the extruder).".to_string();
            def.sidetext = "mm".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6191
            v.values.push(10.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("z_hop", ConfigOptionType::Floats);
            def.label = "Z-hop height".to_string();
            def.tooltip = "Whenever there is a retraction, the nozzle is lifted a little to create clearance between the nozzle and the print. This prevents the nozzle from hitting the print when traveling more. Using spiral lines to lift Z can prevent stringing.".to_string();
            def.sidetext = "mm".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6199
            def.max = 5_f32;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6200
            v.values.push(0.4);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("retract_lift_above", ConfigOptionType::Floats);
            def.label = "Z-hop lower boundary".to_string();
            def.tooltip = "Z-hop will only come into effect when Z is above this value and is below the parameter: \"Z-hop upper boundary\".".to_string();
            def.sidetext = "mm".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.min = 0_f32;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6208
            v.values.push(0.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("retract_lift_below", ConfigOptionType::Floats);
            def.label = "Z-hop upper boundary".to_string();
            def.tooltip = "If this value is positive, Z-hop will only come into effect when Z is above the parameter: \"Z-hop lower boundary\" and is below this value.".to_string();
            def.sidetext = "mm".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.min = 0_f32;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6216
            v.values.push(0.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("z_hop_types", ConfigOptionType::Enums);
            def.label = "Z-hop type".to_string();
            def.tooltip = "Type of Z-hop.".to_string();
            def.enum_keys_map = keys_map_z_hop_type();
            def.enum_values.push("Auto Lift".to_string());
            def.enum_values.push("Normal Lift".to_string());
            def.enum_values.push("Slope Lift".to_string());
            def.enum_values.push("Spiral Lift".to_string());
            def.enum_labels.push("Auto".to_string());
            def.enum_labels.push("Normal".to_string());
            def.enum_labels.push("Slope".to_string());
            def.enum_labels.push("Spiral".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = slic3r_config::config::ConfigOptionEnumsGeneric::default();
            v.values.push(ZHopType::zhtSlope as i32);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("travel_slope", ConfigOptionType::Floats);
            def.label = "Traveling angle".to_string();
            def.tooltip = "Traveling angle for Slope and Spiral Z-hop type. Setting it to 90° results in Normal Lift.".to_string();
            def.sidetext = "°".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.min = 1_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6239
            def.max = 90_f32;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6240
            v.values.push(3_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("retract_lift_above", ConfigOptionType::Floats);
            def.label = "Only lift Z above".to_string();
            def.tooltip = "If you set this to a positive value, Z lift will only take place above the specified absolute Z.".to_string();
            def.sidetext = "mm".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6247
            v.values.push(0.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("retract_lift_below", ConfigOptionType::Floats);
            def.label = "Only lift Z below".to_string();
            def.tooltip = "If you set this to a positive value, Z lift will only take place below the specified absolute Z.".to_string();
            def.sidetext = "mm".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6254
            v.values.push(0.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("retract_lift_enforce", ConfigOptionType::Enums);
            def.label = "On surfaces".to_string();
            def.tooltip = "Enforce Z-Hop behavior. This setting is impacted by the above settings (Only lift Z above/below).".to_string();
            def.enum_keys_map = keys_map_retract_lift_enforce_type();
            def.enum_values.push("All Surfaces".to_string());
            def.enum_values.push("Top Only".to_string());
            def.enum_values.push("Bottom Only".to_string());
            def.enum_values.push("Top and Bottom".to_string());
            def.enum_labels.push("All Surfaces".to_string());
            def.enum_labels.push("Top Only".to_string());
            def.enum_labels.push("Bottom Only".to_string());
            def.enum_labels.push("Top and Bottom".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = slic3r_config::config::ConfigOptionEnumsGeneric::default();
            v.values
                .push(RetractLiftEnforceType::rletAllSurfaces as i32);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("extruder_type", ConfigOptionType::Enums);
            def.label = "Type".to_string();
            def.tooltip = "This setting is only used for initial value of manual calibration of pressure advance. Bowden extruder usually has larger PA value. This setting doesn't influence normal slicing.".to_string();
            def.enum_keys_map = keys_map_extruder_type();
            def.enum_values.push("Direct Drive".to_string());
            def.enum_values.push("Bowden".to_string());
            def.enum_labels.push("Direct Drive".to_string());
            def.enum_labels.push("Bowden".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = slic3r_config::config::ConfigOptionEnumsGeneric::default();
            v.values.push(ExtruderType::etDirectDrive as i32);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("nozzle_volume_type", ConfigOptionType::Enums);
            def.label = "Nozzle Volume Type".to_string();
            def.tooltip = "Nozzle volume type for extruders.".to_string();
            def.enum_keys_map = keys_map_nozzle_volume_type();
            def.enum_values.push("Standard".to_string());
            def.enum_values.push("High Flow".to_string());
            def.enum_values.push("Hybrid".to_string());
            def.enum_values.push("TPU High Flow".to_string());
            def.enum_values.push("E3D High Flow".to_string());
            def.enum_values.push("Extra High Flow".to_string());
            def.enum_labels.push("Standard".to_string());
            def.enum_labels.push("High Flow".to_string());
            def.enum_labels.push("Hybrid".to_string());
            def.enum_labels.push("TPU High Flow".to_string());
            def.enum_labels.push("E3D High Flow".to_string());
            def.enum_labels.push("Extra High Flow".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            let mut v = slic3r_config::config::ConfigOptionEnumsGeneric::default();
            v.values.push(NozzleVolumeType::nvtStandard as i32);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("default_nozzle_volume_type", ConfigOptionType::Enums);
            def.label = "Default Nozzle Volume Type.".to_string();
            def.tooltip = "Default Nozzle volume type for extruders in this printer.".to_string();
            def.enum_keys_map = keys_map_nozzle_volume_type();
            def.enum_values.push("Standard".to_string());
            def.enum_values.push("High Flow".to_string());
            def.enum_values.push("Hybrid".to_string());
            def.enum_values.push("TPU High Flow".to_string());
            def.enum_values.push("E3D High Flow".to_string());
            def.enum_values.push("Extra High Flow".to_string());
            def.enum_labels.push("Standard".to_string());
            def.enum_labels.push("High Flow".to_string());
            def.enum_labels.push("Hybrid".to_string());
            def.enum_labels.push("TPU High Flow".to_string());
            def.enum_labels.push("E3D High Flow".to_string());
            def.enum_labels.push("Extra High Flow".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            let mut v = slic3r_config::config::ConfigOptionEnumsGeneric::default();
            v.values.push(NozzleVolumeType::nvtStandard as i32);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("extruder_variant_list", ConfigOptionType::Strings);
            def.label = "Extruder variant list".to_string();
            def.tooltip = "Extruder variant list.".to_string();
            let mut v = ConfigOptionStrings::new();
            v.values.push("Direct Drive Standard".to_string());
            def.set_default_value(Box::new(v));
            def.cli = NOCLI.to_string();
        }
        {
            let def = self
                .def
                .add("extruder_ams_count", ConfigOptionType::Strings);
            def.label = "Extruder AMS count".to_string();
            def.tooltip = "AMS counts per extruder.".to_string();
            def.set_default_value(Box::new(ConfigOptionStrings::new()));
        }
        {
            let def = self
                .def
                .add("extruder_nozzle_stats", ConfigOptionType::Strings);
            def.label = "Extruder nozzle stats".to_string();
            def.tooltip =
                "Physical nozzle counts per extruder, keyed by nozzle volume type.".to_string();
            def.set_default_value(Box::new(ConfigOptionStrings::new()));
        }
        {
            let def = self
                .def
                .add("extruder_nozzle_count", ConfigOptionType::Ints);
            def.label = "extruder nozzle count".to_string();
            def.tooltip = "extruder nozzle count".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            let mut v = ConfigOptionInts::new();
            v.values.push(1);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("extruder_nozzle_volume_type", ConfigOptionType::Enums);
            def.label = "Extruder nozzle volume type".to_string();
            def.tooltip = "Nozzle volume type per physical nozzle of an extruder.".to_string();
            def.enum_keys_map = keys_map_nozzle_volume_type();
            def.enum_values.push("Standard".to_string());
            def.enum_values.push("High Flow".to_string());
            def.enum_values.push("TPU High Flow".to_string());
            def.enum_values.push("E3D High Flow".to_string());
            def.enum_values.push("Extra High Flow".to_string());
            def.enum_labels.push("Standard".to_string());
            def.enum_labels.push("High Flow".to_string());
            def.enum_labels.push("TPU High Flow".to_string());
            def.enum_labels.push("E3D High Flow".to_string());
            def.enum_labels.push("Extra High Flow".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            let mut v = slic3r_config::config::ConfigOptionEnumsGeneric::default();
            v.values.push(NozzleVolumeType::nvtStandard as i32);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("enable_filament_dynamic_map", ConfigOptionType::Bool);
            def.label = "Enable filament dynamic map".to_string();
            def.tooltip = "Enable dynamic filament mapping during print.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("has_filament_switcher", ConfigOptionType::Bool);
            def.label = "Has filament switcher".to_string();
            def.tooltip = "Printer has a filament switcher hardware (e.g., AMS).".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("printer_extruder_id", ConfigOptionType::Ints);
            def.label = "Printer extruder id".to_string();
            def.tooltip = "Printer extruder id.".to_string();
            let mut v = ConfigOptionInts::new();
            v.values.push(1);
            def.set_default_value(Box::new(v));
            def.cli = NOCLI.to_string();
        }
        {
            let def = self
                .def
                .add("printer_extruder_variant", ConfigOptionType::Strings);
            def.label = "Printer's extruder variant".to_string();
            def.tooltip = "Printer's extruder variant.".to_string();
            let mut v = ConfigOptionStrings::new();
            v.values.push("Direct Drive Standard".to_string());
            def.set_default_value(Box::new(v));
            def.cli = NOCLI.to_string();
        }
        {
            let def = self.def.add("master_extruder_id", ConfigOptionType::Int);
            def.label = "Master extruder id".to_string();
            def.tooltip = "Default extruder id to place filament.".to_string();
            def.set_default_value(Box::new(ConfigOptionInt::new(1)));
        }
        {
            let def = self.def.add("print_extruder_id", ConfigOptionType::Ints);
            def.label = "Print extruder id".to_string();
            def.tooltip = "Print extruder id.".to_string();
            let mut v = ConfigOptionInts::new();
            v.values.push(1);
            def.set_default_value(Box::new(v));
            def.cli = NOCLI.to_string();
        }
        {
            let def = self
                .def
                .add("print_extruder_variant", ConfigOptionType::Strings);
            def.label = "Print's extruder variant".to_string();
            def.tooltip = "Print's extruder variant.".to_string();
            let mut v = ConfigOptionStrings::new();
            v.values.push("Direct Drive Standard".to_string());
            def.set_default_value(Box::new(v));
            def.cli = NOCLI.to_string();
        }
        {
            let def = self
                .def
                .add("filament_extruder_variant", ConfigOptionType::Strings);
            def.label = "Filament's extruder variant".to_string();
            def.tooltip = "Filament's extruder variant.".to_string();
            let mut v = ConfigOptionStrings::new();
            v.values.push("Direct Drive Standard".to_string());
            def.set_default_value(Box::new(v));
            def.cli = NOCLI.to_string();
        }
        {
            let def = self.def.add("filament_self_index", ConfigOptionType::Ints);
            def.label = "Filament self index".to_string();
            def.tooltip = "Filament self index.".to_string();
            let mut v = ConfigOptionInts::new();
            v.values.push(1);
            def.set_default_value(Box::new(v));
            def.cli = NOCLI.to_string();
        }
        {
            let def = self
                .def
                .add("retract_restart_extra", ConfigOptionType::Floats);
            def.label = "Extra length on restart".to_string();
            def.tooltip = "When the retraction is compensated after the travel move, the extruder will push this additional amount of filament. This setting is rarely needed.".to_string();
            def.sidetext = "mm".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6452
            v.values.push(0.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("retract_restart_extra_toolchange", ConfigOptionType::Floats);
            def.label = "Extra length on restart (Toolchange)".to_string();
            def.tooltip = "When the retraction is compensated after changing tool, the extruder will push this additional amount of filament.".to_string();
            def.sidetext = "mm".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6460
            v.values.push(0.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("retraction_speed", ConfigOptionType::Floats);
            def.label = "Retraction speed".to_string();
            def.full_label = "Retraction speed".to_string();
            def.tooltip = "Speed for retracting filament from the nozzle.".to_string();
            def.sidetext = "mm/s".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6468
            v.values.push(30.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("deretraction_speed", ConfigOptionType::Floats);
            def.label = "Deretraction speed".to_string();
            def.full_label = "Deretraction speed".to_string();
            def.tooltip = "Speed for reloading filament into the nozzle. Zero means same speed of retraction.".to_string();
            def.sidetext = "mm/s".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6476
            v.values.push(0.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("deretract_speed_extruder_change", ConfigOptionType::Floats);
            def.label = "Deretraction speed (extruder change)".to_string();
            def.tooltip =
                "Speed for reloading filament into the nozzle when switching extruder.".to_string();
            def.sidetext = "mm/s".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6489
            v.values.push(0.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("use_firmware_retraction", ConfigOptionType::Bool);
            def.label = "Use firmware retraction".to_string();
            def.tooltip = "This experimental setting uses G10 and G11 commands to have the firmware handle the retraction. This is only supported in recent Marlin.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("bbl_calib_mark_logo", ConfigOptionType::Bool);
            def.label = "Show auto-calibration marks".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(true)));
        }
        {
            let def = self.def.add("disable_m73", ConfigOptionType::Bool);
            def.label = "Disable set remaining print time".to_string();
            def.tooltip =
                "Disable generating of the M73: Set remaining print time in the final G-code."
                    .to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("seam_position", ConfigOptionType::Enum);
            def.label = "Seam position".to_string();
            def.category = "Quality".to_string();
            def.tooltip =
                "This is the starting position for each part of the outer wall.".to_string();
            def.enum_keys_map = keys_map_seam_position();
            def.enum_values.push("nearest".to_string());
            def.enum_values.push("aligned".to_string());
            def.enum_values.push("aligned_back".to_string());
            def.enum_values.push("back".to_string());
            def.enum_values.push("random".to_string());
            def.enum_labels.push("Nearest".to_string());
            def.enum_labels.push("Aligned".to_string());
            def.enum_labels.push("Aligned back".to_string());
            def.enum_labels.push("Back".to_string());
            def.enum_labels.push("Random".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            def.set_default_value(Box::new(ConfigOptionEnum::new(SeamPosition::spAligned)));
        }
        {
            let def = self
                .def
                .add("staggered_inner_seams", ConfigOptionType::Bool);
            def.label = "Staggered inner seams".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "This option causes the inner seams to be shifted backwards based on their depth, forming a zigzag pattern.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("seam_gap", ConfigOptionType::FloatOrPercent);
            def.label = "Seam gap".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "In order to reduce the visibility of the seam in a closed loop extrusion, the loop is interrupted and shortened by a specified amount.\nThis amount can be specified in millimeters or as a percentage of the nozzle diameter. The default value for this parameter is 10%.".to_string();
            def.sidetext = "mm or %".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6543
            def.set_default_value(Box::new(ConfigOptionFloatOrPercent::new(10_f64, true)));
        }
        {
            let def = self.def.add("seam_slope_type", ConfigOptionType::Enum);
            def.label = "Scarf joint seam (beta)".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Use scarf joint to minimize seam visibility and increase seam strength."
                .to_string();
            def.enum_keys_map = keys_map_seam_scarf_type();
            def.enum_values.push("none".to_string());
            def.enum_values.push("external".to_string());
            def.enum_values.push("all".to_string());
            def.enum_labels.push("None".to_string());
            def.enum_labels.push("Contour".to_string());
            def.enum_labels.push("Contour and hole".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionEnum::new(SeamScarfType::None)));
        }
        {
            let def = self
                .def
                .add("seam_slope_conditional", ConfigOptionType::Bool);
            def.label = "Conditional scarf joint".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Apply scarf joints only to smooth perimeters where traditional seams do not conceal the seams at sharp corners effectively.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("scarf_angle_threshold", ConfigOptionType::Int);
            def.label = "Conditional angle threshold".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "This option sets the threshold angle for applying a conditional scarf joint seam.\nIf the maximum angle within the perimeter loop exceeds this value (indicating the absence of sharp corners), a scarf joint seam will be used. The default value is 155°.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.sidetext = "°".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6575
            def.max = 180_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6576
            def.set_default_value(Box::new(ConfigOptionInt::new(155)));
        }
        {
            let def = self
                .def
                .add("scarf_overhang_threshold", ConfigOptionType::Percent);
            def.label = "Conditional overhang threshold".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "This option determines the overhang threshold for the application of scarf joint seams. If the unsupported portion of the perimeter is less than this threshold, scarf joint seams will be applied. The default threshold is set at 40% of the external wall's width. Due to performance considerations, the degree of overhang is estimated.".to_string();
            def.sidetext = "%".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6588
            def.set_default_value(Box::new(ConfigOptionPercent::new(40_f64)));
        }
        {
            let def = self
                .def
                .add("scarf_joint_speed", ConfigOptionType::FloatOrPercent);
            def.label = "Scarf joint speed".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "This option sets the printing speed for scarf joints. It is recommended to print scarf joints at a slow speed (less than 100 mm/s). It's also advisable to enable 'Extrusion rate smoothing' if the set speed varies significantly from the speed of the outer or inner walls.\n\nIf the speed specified here is higher than the speed of the outer or inner walls, the printer will default to the slower of the two speeds. When specified as a percentage (e.g., 80%), the speed is calculated based on the respective outer or inner wall speed. The default value is set to 100%.".to_string();
            def.sidetext = "mm/s or %".to_string();
            def.min = 1_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6602
            def.set_default_value(Box::new(ConfigOptionFloatOrPercent::new(100_f64, true)));
        }
        {
            let def = self
                .def
                .add("scarf_joint_flow_ratio", ConfigOptionType::Float);
            def.label = "Scarf joint flow ratio".to_string();
            def.category = "Quality".to_string();
            def.tooltip =
                "This factor affects the amount of material for scarf joints.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Expert;
            def.min = 0_f32;
            def.max = 2_f32;
            def.set_default_value(Box::new(ConfigOptionFloat::new(1_f64)));
        }
        {
            let def = self
                .def
                .add("seam_slope_start_height", ConfigOptionType::FloatOrPercent);
            def.label = "Scarf start height".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Start height of the scarf.\nThis amount can be specified in millimeters or as a percentage of the current layer height. The default value for this parameter is 0.".to_string();
            def.sidetext = "mm or %".to_string();
            def.ratio_over = "layer_height".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloatOrPercent::new(0_f64, false)));
        }
        {
            let def = self
                .def
                .add("seam_slope_entire_loop", ConfigOptionType::Bool);
            def.label = "Scarf around entire wall".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "The scarf extends to the entire length of the wall.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("seam_slope_min_length", ConfigOptionType::Float);
            def.label = "Scarf length".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Length of the scarf. Setting this parameter to zero effectively disables the scarf.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6638
            def.set_default_value(Box::new(ConfigOptionFloat::new(20_f64)));
        }
        {
            let def = self.def.add("seam_slope_steps", ConfigOptionType::Int);
            def.label = "Scarf steps".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Minimum number of segments of each scarf.".to_string();
            def.min = 1_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6646
            def.set_default_value(Box::new(ConfigOptionInt::new(10)));
        }
        {
            let def = self
                .def
                .add("seam_slope_inner_walls", ConfigOptionType::Bool);
            def.label = "Scarf joint for inner walls".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Use scarf joint for inner walls as well.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("role_based_wipe_speed", ConfigOptionType::Bool);
            def.label = "Role based wipe speed".to_string();
            def.category = "Speed".to_string();
            def.tooltip = "The wipe speed is determined by the speed of the current extrusion role. e.g. if a wipe action is executed immediately following an outer wall extrusion, the speed of the outer wall extrusion will be utilized for the wipe action.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(true)));
        }
        {
            let def = self.def.add("wipe_on_loops", ConfigOptionType::Bool);
            def.label = "Wipe on loops".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "To minimize the visibility of the seam in a closed loop extrusion, a small inward movement is executed before the extruder leaves the loop.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("wipe_inward", ConfigOptionType::Bool);
            def.label = "Wipe inward".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Applies only to external walls, including hole boundaries. Moves the hot nozzle toward the inner walls during wiping to reduce reheating of freshly printed plastic and seam marks.\n\nEspecially useful at layer heights below 0.1 mm, where wipe marks are more visible.\n\nUses the regular wipe where the outer wall has no adjacent inner wall, such as in single-wall areas, or if no supported inward path can be found, for example at tight corners or seam gaps.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Expert;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("wipe_inward_distance", ConfigOptionType::FloatOrPercent);
            def.label = "Wipe inward distance".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "The distance the wipe path is shifted away from the external perimeter, specified in millimeters or as a percentage of the actual outer-wall extrusion width.\n\nFor example, 50% shifts the path by half of the outer-wall width. The effective offset is limited by both the actual outer-wall width and the available spacing to the adjacent wall, so values above 100% or an equivalent absolute distance have no additional effect. Set to 0 to disable the offset.\n\nWith Outer/Inner or Inner/Outer/Inner wall order, the offset is limited to 50% of the actual outer-wall width because the adjacent inner wall is printed after the outer wall. This also applies to equivalent distances specified in millimeters.".to_string();
            def.sidetext = "mm or %".to_string();
            def.ratio_over = "outer_wall_line_width".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6697
            def.max = 100_f32;
            def.max_literal = 2_f64;
            def.mode = slic3r_config::config::ConfigOptionMode::Expert;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6700
            def.set_default_value(Box::new(ConfigOptionFloatOrPercent::new(50_f64, true)));
        }
        {
            let def = self
                .def
                .add("wipe_before_external_loop", ConfigOptionType::Bool);
            def.label = "Wipe before external loop".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "To minimize visibility of potential overextrusion at the start of an external perimeter when printing with Outer/Inner or Inner/Outer/Inner wall print order, the de-retraction is performed slightly on the inside from the start of the external perimeter. That way any potential over extrusion is hidden from the outside surface.\n\nThis is useful when printing with Outer/Inner or Inner/Outer/Inner wall print order as in these modes it is more likely an external perimeter is printed immediately after a de-retraction move.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("wipe_speed", ConfigOptionType::FloatOrPercent);
            def.label = "Wipe speed".to_string();
            def.category = "Speed".to_string();
            def.tooltip = "The wipe speed is determined by the speed setting specified in this configuration. If the value is expressed as a percentage (e.g. 80%), it will be calculated based on the travel speed setting above. The default value for this parameter is 80%.".to_string();
            def.sidetext = "mm/s or %".to_string();
            def.ratio_over = "travel_speed".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6723
            def.set_default_value(Box::new(ConfigOptionFloatOrPercent::new(80_f64, true)));
        }
        {
            let def = self.def.add("skirt_distance", ConfigOptionType::Float);
            def.label = "Skirt distance".to_string();
            def.tooltip =
                "This is the distance from the skirt to the brim or the object.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6730
            def.max = 60_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(2_f64)));
        }
        {
            let def = self.def.add("skirt_start_angle", ConfigOptionType::Float);
            def.label = "Skirt start point".to_string();
            def.category = "Support".to_string();
            def.tooltip = "Angle from the object center to skirt start point. Zero is the most right position, counter clockwise is positive angle.".to_string();
            def.sidetext = "°".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6739
            def.min = -180_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6740
            def.max = 180_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6742
            def.set_default_value(Box::new(ConfigOptionFloat::new(-135_f64)));
        }
        {
            let def = self.def.add("skirt_height", ConfigOptionType::Int);
            def.label = "Skirt height".to_string();
            def.tooltip = "Number of skirt layers: usually only one".to_string();
            def.sidetext = "layers".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6749
            def.max = 10000_f32;
            def.set_default_value(Box::new(ConfigOptionInt::new(1)));
        }
        {
            let def = self
                .def
                .add("single_loop_draft_shield", ConfigOptionType::Bool);
            def.label = "Single loop after first layer".to_string();
            def.tooltip = "Limits the skirt/draft shield loops to one wall after the first layer. This is useful, on occasion, to conserve filament but may cause the draft shield/skirt to warp / crack.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("draft_shield", ConfigOptionType::Enum);
            def.label = "Draft shield".to_string();
            def.tooltip = "A draft shield is useful to protect an ABS or ASA print from warping and detaching from print bed due to wind draft. It is usually needed only with open frame printers, i.e. without an enclosure.\n\nEnabled = skirt is as tall as the highest printed object. Otherwise 'Skirt height' is used.".to_string();
            def.enum_keys_map = keys_map_draft_shield();
            def.enum_values.push("disabled".to_string());
            def.enum_values.push("enabled".to_string());
            def.enum_labels.push("Disabled".to_string());
            def.enum_labels.push("Enabled".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionEnum::new(DraftShield::dsDisabled)));
        }
        {
            let def = self.def.add("skirt_type", ConfigOptionType::Enum);
            def.label = "Skirt type".to_string();
            def.full_label = "Skirt type".to_string();
            def.tooltip =
                "Combined - single skirt for all objects, Per object - individual object skirt."
                    .to_string();
            def.enum_keys_map = keys_map_skirt_type();
            def.enum_values.push("combined".to_string());
            def.enum_values.push("perobject".to_string());
            def.enum_labels.push("Combined".to_string());
            def.enum_labels.push("Per object".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionEnum::new(SkirtType::stCombined)));
        }
        {
            let def = self.def.add("skirt_loops", ConfigOptionType::Int);
            def.label = "Skirt loops".to_string();
            def.full_label = "Skirt loops".to_string();
            def.tooltip =
                "This is the number of loops for the skirt. 0 means the skirt is disabled."
                    .to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6788
            def.max = 10_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            def.set_default_value(Box::new(ConfigOptionInt::new(1)));
        }
        {
            let def = self.def.add("skirt_speed", ConfigOptionType::Float);
            def.label = "Skirt speed".to_string();
            def.full_label = "Skirt speed".to_string();
            def.tooltip = "Speed of skirt, in mm/s. Zero means use default layer extrusion speed."
                .to_string();
            def.min = 0_f32;
            def.sidetext = "mm/s".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6799
            def.set_default_value(Box::new(ConfigOptionFloat::new(50.0)));
        }
        {
            let def = self.def.add("min_skirt_length", ConfigOptionType::Float);
            def.label = "Skirt minimum extrusion length".to_string();
            def.full_label = "Skirt minimum extrusion length".to_string();
            def.tooltip = "Minimum filament extrusion length in mm when printing the skirt. Zero means this feature is disabled.\n\nUsing a non-zero value is useful if the printer is set up to print without a prime line.\nFinal number of loops is not taking into account while arranging or validating objects distance. Increase loop number in such case.".to_string();
            def.min = 0_f32;
            def.sidetext = "mm".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6810
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.0)));
        }
        {
            let def = self
                .def
                .add("slow_down_layer_time", ConfigOptionType::Floats);
            def.label = "Layer time".to_string();
            def.tooltip = "The printing speed in exported G-code will be slowed down when the estimated layer time is shorter than this value in order to get better cooling for these layers.".to_string();
            def.sidetext = "s".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6818
            def.max = 1000_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6820
            v.values.push(5.0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("minimum_sparse_infill_area", ConfigOptionType::Float);
            def.label = "Minimum sparse infill threshold".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Sparse infill areas which are smaller than this threshold value are replaced by internal solid infill.".to_string();
            def.sidetext = "mm²".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6829
            def.set_default_value(Box::new(ConfigOptionFloat::new(15_f64)));
        }
        {
            let def = self
                .def
                .add("internal_solid_filament_id", ConfigOptionType::Int);
            def.gui_type = GuiType::IEnumOpen;
            def.label = "Internal solid infill".to_string();
            def.category = "Extruders".to_string();
            def.tooltip = "Filament to print internal solid infill.\n\"Default\" uses the active object/part filament.".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionInt::new(0)));
        }
        {
            let def = self
                .def
                .add("top_surface_filament_id", ConfigOptionType::Int);
            def.gui_type = GuiType::IEnumOpen;
            def.label = "Top surface".to_string();
            def.category = "Extruders".to_string();
            def.tooltip =
                "Filament to print top surface.\n\"Default\" uses the active object/part filament."
                    .to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionInt::new(0)));
        }
        {
            let def = self
                .def
                .add("bottom_surface_filament_id", ConfigOptionType::Int);
            def.gui_type = GuiType::IEnumOpen;
            def.label = "Bottom surface".to_string();
            def.category = "Extruders".to_string();
            def.tooltip = "Filament to print bottom surface.\n\"Default\" uses the active object/part filament.".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionInt::new(0)));
        }
        {
            let def = self.def.add(
                "internal_solid_infill_line_width",
                ConfigOptionType::FloatOrPercent,
            );
            def.label = "Internal solid infill".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Line width of internal solid infill. If expressed as a %, it will be computed over the nozzle diameter.".to_string();
            def.sidetext = "mm or %".to_string();
            def.ratio_over = "nozzle_diameter".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6865
            def.max = 1000_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6866
            def.max_literal = 10_f64;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6868
            def.set_default_value(Box::new(ConfigOptionFloatOrPercent::new(0., false)));
        }
        {
            let def = self
                .def
                .add("internal_solid_infill_speed", ConfigOptionType::Floats);
            def.label = "Internal solid infill".to_string();
            def.category = "Speed".to_string();
            def.tooltip = "This is the speed for internal solid infill, not including the top or bottom surface.".to_string();
            def.sidetext = "mm/s".to_string();
            def.min = 1_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6878
            v.values.push(100_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("spiral_mode", ConfigOptionType::Bool);
            def.label = "Spiral vase".to_string();
            def.tooltip = "This enables spiraling, which smooths out the Z moves of the outer contour and turns a solid model into a single walled print with solid bottom layers. The final generated model has no seam.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("spiral_mode_smooth", ConfigOptionType::Bool);
            def.label = "Smooth Spiral".to_string();
            def.tooltip = "Smooth Spiral smooths out X and Y moves as well, resulting in no visible seam at all, even in the XY directions on walls that are not vertical.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add(
                "spiral_mode_max_xy_smoothing",
                ConfigOptionType::FloatOrPercent,
            );
            def.label = "Max XY Smoothing".to_string();
            def.tooltip = "Maximum distance to move points in XY to try to achieve a smooth spiral. If expressed as a %, it will be computed over nozzle diameter.".to_string();
            def.sidetext = "mm or %".to_string();
            def.ratio_over = "nozzle_diameter".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6901
            def.max = 1000_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6902
            def.max_literal = 10_f64;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6904
            def.set_default_value(Box::new(ConfigOptionFloatOrPercent::new(200_f64, true)));
        }
        {
            let def = self
                .def
                .add("spiral_starting_flow_ratio", ConfigOptionType::Float);
            def.label = "Spiral starting flow ratio".to_string();
            def.tooltip = "Sets the starting flow ratio while transitioning from the last bottom layer to the spiral. Normally the spiral transition scales the flow ratio from 0% to 100% during the first loop which can in some cases lead to under extrusion at the start of the spiral.".to_string();
            def.min = 0_f32;
            def.max = 1_f32;
            def.set_default_value(Box::new(ConfigOptionFloat::new(0_f64)));
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
        }
        {
            let def = self
                .def
                .add("spiral_finishing_flow_ratio", ConfigOptionType::Float);
            def.label = "Spiral finishing flow ratio".to_string();
            def.tooltip = "Sets the finishing flow ratio while ending the spiral. Normally the spiral transition scales the flow ratio from 100% to 0% during the last loop which can in some cases lead to under extrusion at the end of the spiral.".to_string();
            def.min = 0_f32;
            def.max = 1_f32;
            def.set_default_value(Box::new(ConfigOptionFloat::new(0_f64)));
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
        }
        {
            let def = self.def.add("timelapse_type", ConfigOptionType::Enum);
            def.label = "Timelapse".to_string();
            def.tooltip = "If smooth or traditional mode is selected, a timelapse video will be generated for each print. After each layer is printed, a snapshot is taken with the chamber camera. All of these snapshots are composed into a timelapse video when printing completes.\n\nIf smooth mode is selected, the toolhead will move to the excess chute after each layer is printed and then take a snapshot. Since the melt filament may leak from the nozzle during the process of taking a snapshot, a prime tower is required for smooth mode to wipe the nozzle.".to_string();
            def.enum_keys_map = keys_map_timelapse_type();
            def.enum_values.push("0".to_string());
            def.enum_values.push("1".to_string());
            def.enum_labels.push("Traditional".to_string());
            def.enum_labels.push("Smooth".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            def.set_default_value(Box::new(ConfigOptionEnum::new(
                TimelapseType::tlTraditional,
            )));
        }
        {
            let def = self
                .def
                .add("farthest_point_timelapse", ConfigOptionType::Bool);
            def.label = "Farthest point timelapse".to_string();
            def.tooltip = "When enabled, the timelapse snapshot is taken at the farthest point from camera instead of traveling to the wipe tower or excess chute. Only effective in traditional timelapse mode on non-I3 printers.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("standby_temperature_delta", ConfigOptionType::Int);
            def.label = "Temperature variation".to_string();
            def.tooltip = "Temperature difference to be applied when an extruder is not active. The value is not used when 'idle_temperature' in filament settings is set to non-zero value.".to_string();
            def.sidetext = "\u{2206}\u{2103}".to_string();
            def.min = -max_temp;
            def.max = max_temp;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6963
            def.set_default_value(Box::new(ConfigOptionInt::new(-5)));
        }
        {
            let def = self.def.add("preheat_time", ConfigOptionType::Float);
            def.label = "Preheat time".to_string();
            def.tooltip = "To reduce the waiting time after tool change, Orca can preheat the next tool while the current tool is still in use. This setting specifies the time in seconds to preheat the next tool. Orca will insert a M104 command to preheat the tool in advance.".to_string();
            def.sidetext = "s".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6971
            def.max = 120_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6973
            def.set_default_value(Box::new(ConfigOptionFloat::new(30.0)));
        }
        {
            let def = self.def.add("preheat_steps", ConfigOptionType::Int);
            def.label = "Preheat steps".to_string();
            def.tooltip = "Insert multiple preheat commands (e.g. M104.1). Only useful for Prusa XL. For other printers, please set it to 1.".to_string();
            def.min = 1_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6980
            def.max = 10_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            def.set_default_value(Box::new(ConfigOptionInt::new(1)));
        }
        {
            let def = self.def.add("file_start_gcode", ConfigOptionType::String);
            def.label = "File header G-code".to_string();
            def.tooltip = "G-code written at the very top of the output file, before any other content. Useful for adding metadata that printer firmware reads from the first lines of the file (e.g. estimated print time, filament usage). Supports placeholders like {print_time_total_sec}, {print_time_day}, {print_time_hour}, {print_time_minute}, {print_time_sec} and {used_filament_length}.".to_string();
            def.multiline = true;
            def.full_width = true;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:6993
            def.height = 8;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionString::new("".to_string())));
        }
        {
            let def = self
                .def
                .add("machine_start_gcode", ConfigOptionType::String);
            def.label = "Start G-code".to_string();
            def.tooltip = "G-code added when starting a print.".to_string();
            def.multiline = true;
            def.full_width = true;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7002
            def.height = 12;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionString::new(
                "G28 ; home all axes\nG1 Z5 F5000 ; lift nozzle\n".to_string(),
            )));
        }
        {
            let def = self
                .def
                .add("filament_start_gcode", ConfigOptionType::Strings);
            def.label = "Start G-code".to_string();
            def.tooltip = "G-code added when the printer starts using this filament".to_string();
            def.multiline = true;
            def.full_width = true;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7011
            def.height = 12;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionStrings::new();
            v.values.push(" ".to_string());
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("single_extruder_multi_material", ConfigOptionType::Bool);
            def.label = "Single Extruder Multi Material".to_string();
            def.tooltip = "Use single nozzle to print multi filament.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(true)));
        }
        {
            let def = self.def.add("is_imex", ConfigOptionType::Bool);
            def.label = "IDEX/IQEX Printer".to_string();
            def.tooltip = "Enable parallel printing for printers with multiple independent carriages (IDEX, IQEX, and similar).".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("imex_firmware_managed_zones", ConfigOptionType::Bool);
            def.label = "Firmware-managed zones".to_string();
            def.tooltip = "When enabled, the slicer emits G-code with the primary zone's contents centered on bed origin (0,0). The printer firmware is responsible for placing copies/mirrors at each physical zone position. Enable only on printers whose firmware applies its own zone offsets in copy/mirror mode (e.g. RepRapFirmware IDEX duplication mode). Has no effect in Primary mode.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("imex_gantry_count", ConfigOptionType::Int);
            def.label = "Gantry Count".to_string();
            def.tooltip = "Number of independent Y-axis gantries. 1 for IDEX/single-rail IQEX. 2 for dual-gantry systems (divides bed into rows).".to_string();
            def.min = 1_f32;
            def.max = 2_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionInt::new(1)));
        }
        {
            let def = self.def.add("imex_tools_per_gantry", ConfigOptionType::Int);
            def.label = "Tools per Gantry".to_string();
            def.tooltip =
                "Number of independent toolheads per gantry along X. 2 for IDEX-style.".to_string();
            def.min = 1_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7060
            def.max = 4_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionInt::new(2)));
        }
        {
            let def = self.def.add("imex_tool_layout", ConfigOptionType::Enum);
            def.label = "Tool 0 Position".to_string();
            def.tooltip = "Physical position on the bed where tool T0 (index 0) is located. Determines how tool indices map to bed zones. For single-gantry setups (IDEX) only left/right matters; for dual-gantry setups (IQEX) all four corners are selectable (front = lower Y / near the operator, rear = higher Y / back of machine).".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.enum_keys_map = keys_map_imex_tool_layout();
            def.enum_values.push("front-left".to_string());
            def.enum_values.push("front-right".to_string());
            def.enum_values.push("rear-left".to_string());
            def.enum_values.push("rear-right".to_string());
            def.enum_labels.push("Front-left".to_string());
            def.enum_labels.push("Front-right".to_string());
            def.enum_labels.push("Rear-left".to_string());
            def.enum_labels.push("Rear-right".to_string());
            def.set_default_value(Box::new(ConfigOptionEnum::new(ImexToolLayout::FrontLeft)));
        }
        {
            let def = self
                .def
                .add("imex_nozzle_clearance_x", ConfigOptionType::Float);
            def.label = "Nozzle Clearance X".to_string();
            def.tooltip = "Distance (mm) from the nozzle to the collision-side carriage edge in X. Used to calculate the width of the collision exclusion strip at each X boundary.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7089
            def.set_default_value(Box::new(ConfigOptionFloat::new(30.0)));
        }
        {
            let def = self
                .def
                .add("imex_nozzle_clearance_y", ConfigOptionType::Float);
            def.label = "Nozzle Clearance Y".to_string();
            def.tooltip = "Distance (mm) from the nozzle to the collision-side carriage edge in Y. Used to calculate the width of the collision exclusion strip at each Y boundary.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7097
            def.set_default_value(Box::new(ConfigOptionFloat::new(30.0)));
        }
        {
            let def = self
                .def
                .add("imex_carriage_margin", ConfigOptionType::Float);
            def.label = "Safety Margin".to_string();
            def.tooltip = "Non-blocking advisory clearance strip (mm) drawn inside the primary zone at each carriage boundary. Parts placed within this strip will still slice; it is a visual reminder to leave extra clearance near the zone edge.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7104
            def.max = 200_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7106
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.0)));
        }
        {
            let def = self.def.add("imex_viz_theme", ConfigOptionType::Enum);
            def.label = "Visualization Theme".to_string();
            def.tooltip = "Color theme for IDEX/IQEX bed zone visualization. Choose a colorblind-friendly theme if needed.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.enum_keys_map = keys_map_imex_viz_theme();
            def.enum_values.push("standard".to_string());
            def.enum_values.push("deuteranopia".to_string());
            def.enum_values.push("tritanopia".to_string());
            def.enum_values.push("high_contrast".to_string());
            def.enum_labels.push("Standard".to_string());
            def.enum_labels
                .push("Deuteranopia / Protanopia (red-green)".to_string());
            def.enum_labels.push("Tritanopia (blue-yellow)".to_string());
            def.enum_labels.push("High Contrast".to_string());
            def.set_default_value(Box::new(ConfigOptionEnum::new(ImexVizTheme::Standard)));
        }
        {
            let def = self.def.add("imex_parallel_mode", ConfigOptionType::String);
            def.label = "IDEX/IQEX Print Mode".to_string();
            def.tooltip = "Name of the active IDEX/IQEX parallel print mode, or \"primary\" for single-carriage printing. Requires IDEX/IQEX Printer enabled in the Printer preset (Printer \u{2192} Multimaterial \u{2192} IDEX/IQEX Configuration).".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionString::new("primary".to_string())));
        }
        {
            let def = self
                .def
                .add("imex_head_filament_map", ConfigOptionType::String);
            def.label = "IDEX/IQEX head filament map".to_string();
            def.tooltip = "Per-plate override mapping physical heads to filament slots (1-based). Empty means each head uses the first filament that the printer's physical extruder map routes to it.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            def.set_default_value(Box::new(ConfigOptionString::new("".to_string())));
        }
        {
            let def = self.def.add("imex_mode_names", ConfigOptionType::Strings);
            def.label = "IDEX/IQEX Mode Names".to_string();
            def.tooltip =
                "Display names for each user-defined IDEX/IQEX parallel print mode.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionStrings::new()));
        }
        {
            let def = self
                .def
                .add("imex_mode_active_tools", ConfigOptionType::Strings);
            def.label = "IDEX/IQEX Mode Active Tools".to_string();
            def.tooltip = "Tool role assignments for each mode. Format: \"idx:P,idx:C,idx:M\" where P=Primary, C=Copy, M=Mirror (e.g. \"0:P,1:C,2:M,3:M\"). Managed by the IDEX/IQEX Modes editor in the Printer preset.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionStrings::new()));
        }
        {
            let def = self.def.add("imex_mode_gcodes", ConfigOptionType::Strings);
            def.label = "IDEX/IQEX Mode G-codes".to_string();
            def.tooltip =
                "G-code or macro call to inject at print start for each mode.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionStrings::new()));
        }
        {
            let def = self
                .def
                .add("manual_filament_change", ConfigOptionType::Bool);
            def.label = "Manual Filament Change".to_string();
            def.tooltip = "Enable this option to omit the custom Change filament G-code only at the beginning of the print. The tool change command (e.g., T0) will be skipped throughout the entire print. This is useful for manual multi-material printing, where we use M600/PAUSE to trigger the manual filament change action.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("wipe_tower_type", ConfigOptionType::Enum);
            def.label = "Wipe tower type".to_string();
            def.tooltip = "Choose the wipe tower implementation for multi-material prints. Type 1 is recommended for Bambu and Qidi printers with a filament cutter. Type 2 offers better compatibility with multi-tool and MMU printers and provide overall better compatibility.".to_string();
            def.enum_keys_map = keys_map_wipe_tower_type();
            def.enum_values.push("type1".to_string());
            def.enum_values.push("type2".to_string());
            def.enum_labels.push("Type 1".to_string());
            def.enum_labels.push("Type 2".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionEnum::new(WipeTowerType::Type2)));
        }
        {
            let def = self.def.add("purge_in_prime_tower", ConfigOptionType::Bool);
            def.label = "Purge in prime tower".to_string();
            def.tooltip = "Purge remaining filament into prime tower.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(true)));
        }
        {
            let def = self
                .def
                .add("enable_filament_ramming", ConfigOptionType::Bool);
            def.label = "Enable filament ramming".to_string();
            def.tooltip = "Enable filament ramming".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(true)));
        }
        {
            let def = self
                .def
                .add("tool_change_on_wipe_tower", ConfigOptionType::Bool);
            def.label = "Tool change on wipe tower".to_string();
            def.tooltip = "Force the toolhead to travel to the wipe tower before issuing the tool change command (Tx).\n\nOnly relevant for multi-extruder (multi-toolhead) printers using a Type 2 wipe tower. By default Orca skips the travel on multi-toolhead machines because the firmware handles the head swap, which can result in the Tx command being issued above the printed part. Enable this option if you want the tool change to always be issued above the wipe tower instead.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("wait_for_temp_on_wipe_tower", ConfigOptionType::Bool);
            def.label = "Wait for temperature on wipe tower".to_string();
            def.tooltip = "Pick up the new tool without waiting for it to reach printing temperature, travel to the wipe tower, and wait for the temperature there, right before purging. Ooze from the heat-up lands on the tower instead of the model, and the travel overlaps with the heating.\n\nOnly relevant for multi-extruder (multi-toolhead) printers using a Type 2 wipe tower. The firmware or tool change macro must not wait for the temperature itself. When disabled, the temperature wait is issued right after the tool change command.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("wipe_tower_no_sparse_layers", ConfigOptionType::Bool);
            def.label = "No sparse layers (beta)".to_string();
            def.tooltip = "If enabled, the wipe tower will not be printed on layers with no tool changes. On layers with a tool change, extruder will travel downward to print the wipe tower, so the tower ends up below the model and the toolhead has to reach down to it. Layouts where that would collide with an already printed object are rejected. Has no effect with smooth timelapse or clumping detection, which need a tower on every layer.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add(
                "wipe_tower_sparse_layers_combination",
                ConfigOptionType::Bool,
            );
            def.label = "Combine sparse layers".to_string();
            def.tooltip = "If enabled, consecutive layers on which the prime tower has no filament change are printed as a single thicker tower layer instead of one thin layer each, the same way infill combination merges sparse infill. The merged layer is printed at the top of the run, at the height of everything it covers.\n\nOnly whole layers are merged, and never past the maximum layer height of the nozzle printing the tower (three quarters of the nozzle diameter when that is left at 0). Two or more layers therefore have to fit under that limit before anything changes at all: at a 0.2 mm layer height under a 0.3 mm maximum nothing is merged, while at 0.1 mm three layers become one.\n\nUnlike \"No sparse layers\" the tower keeps following the model, so the toolhead never has to reach down to it. Has no effect with \"No sparse layers\", smooth timelapse or clumping detection, which need a tower on every layer.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add(
                "single_extruder_multi_material_priming",
                ConfigOptionType::Bool,
            );
            def.label = "Prime all printing extruders".to_string();
            def.tooltip = "If enabled, all printing extruders will be primed at the front edge of the print bed at the start of the print.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("toolchange_ordering", ConfigOptionType::Enum);
            def.label = "Toolchange ordering".to_string();
            def.category = "Advanced".to_string();
            def.tooltip = "Determines the order of tool changes on each layer.\n- Default: Starts with the last used extruder to minimize tool changes.\n- Cyclic: Uses a fixed tool sequence each layer. This sacrifices speed for better surface quality, as the extra toolchanges allow layers more time to cool.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.enum_keys_map = keys_map_tool_change_ordering_type();
            def.enum_values.push("default".to_string());
            def.enum_values.push("cyclic".to_string());
            def.enum_labels.push("Default".to_string());
            def.enum_labels.push("Cyclic".to_string());
            def.set_default_value(Box::new(ConfigOptionEnum::new(
                ToolChangeOrderingType::Default,
            )));
        }
        {
            let def = self
                .def
                .add("toolchange_cyclic_order", ConfigOptionType::String);
            def.label = "Cyclic order".to_string();
            def.category = "Advanced".to_string();
            def.tooltip = "Custom filament sequence used by the cyclic toolchange ordering, as filament numbers separated by commas (e.g. \"3,2,1,4\").\nEach layer prints its filaments following this sequence; filaments not listed are printed last, in ascending order.\nLeave empty to cycle through the filaments in ascending order.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Expert;
            def.set_default_value(Box::new(ConfigOptionString::new("".to_string())));
        }
        {
            let def = self
                .def
                .add("toolchange_cyclic_first_layer", ConfigOptionType::Bool);
            def.label = "Apply cyclic order to first layer".to_string();
            def.category = "Advanced".to_string();
            def.tooltip = "Applies the cyclic toolchange order to the first layer as well.\nBy default this is disabled, because the first layer is instead ordered for the best bed adhesion: filaments that print small, fragile first-layer features are printed last, so the following tool changes and travel moves are less likely to knock those weakly anchored parts loose. This first-layer order also honors a custom first layer filament sequence when one is set. The cyclic order's benefit (extra tool changes give each layer more time to cool) does not apply to the first layer, which is printed slowly and hot for adhesion.\nEnable this only if you need the exact same tool sequence on every layer, including the first, at the cost of that adhesion optimization.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Expert;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("slice_closing_radius", ConfigOptionType::Float);
            def.label = "Slice gap closing radius".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Cracks smaller than 2x gap closing radius are being filled during the triangle mesh slicing. The gap closing operation may reduce the final print resolution, therefore it is advisable to keep the value reasonably low.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7290
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.049)));
        }
        {
            let def = self.def.add("slicing_mode", ConfigOptionType::Enum);
            def.label = "Slicing Mode".to_string();
            def.category = "Others".to_string();
            def.tooltip = "Use \"Even-odd\" for 3DLabPrint airplane models. Use \"Close holes\" to close all holes in the model.".to_string();
            def.enum_keys_map = keys_map_slicing_mode();
            def.enum_values.push("regular".to_string());
            def.enum_values.push("even_odd".to_string());
            def.enum_values.push("close_holes".to_string());
            def.enum_labels.push("Regular".to_string());
            def.enum_labels.push("Even-odd".to_string());
            def.enum_labels.push("Close holes".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionEnum::new(SlicingMode::Regular)));
        }
        {
            let def = self.def.add("z_offset", ConfigOptionType::Float);
            def.label = "Z offset".to_string();
            def.tooltip = "This value will be added (or subtracted) from all the Z coordinates in the output G-code. It is used to compensate for bad Z endstop position: for example, if your endstop zero actually leaves the nozzle 0.3mm far from the print bed, set this to -0.3 (or fix your endstop).".to_string();
            def.sidetext = "mm".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(0_f64)));
        }
        {
            let def = self.def.add("enable_support", ConfigOptionType::Bool);
            def.label = "Enable support".to_string();
            def.category = "Support".to_string();
            def.tooltip = "This enables support generation.".to_string();
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("support_type", ConfigOptionType::Enum);
            def.label = "Type".to_string();
            def.category = "Support".to_string();
            def.tooltip = "Normal (auto) and Tree (auto) are used to generate support automatically. If Normal (manual) or Tree (manual) is selected, only support enforcers are generated.".to_string();
            def.enum_keys_map = keys_map_support_type();
            def.enum_values.push("normal(auto)".to_string());
            def.enum_values.push("tree(auto)".to_string());
            def.enum_values.push("normal(manual)".to_string());
            def.enum_values.push("tree(manual)".to_string());
            def.enum_labels.push("Normal (auto)".to_string());
            def.enum_labels.push("Tree (auto)".to_string());
            def.enum_labels.push("Normal (manual)".to_string());
            def.enum_labels.push("Tree (manual)".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            def.set_default_value(Box::new(ConfigOptionEnum::new(SupportType::stNormalAuto)));
        }
        {
            let def = self
                .def
                .add("support_object_xy_distance", ConfigOptionType::Float);
            def.label = "Support/object XY distance".to_string();
            def.category = "Support".to_string();
            def.tooltip =
                "This controls the XY separation between an object and its support.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7346
            def.max = 10_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7349
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.35)));
        }
        {
            let def = self
                .def
                .add("support_object_first_layer_gap", ConfigOptionType::Float);
            def.label = "Support/object first layer gap".to_string();
            def.category = "Support".to_string();
            def.tooltip =
                "XY separation between an object and its support at the first layer.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7357
            def.max = 10_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7360
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.2)));
        }
        {
            let def = self.def.add("support_angle", ConfigOptionType::Float);
            def.label = "Pattern angle".to_string();
            def.category = "Support".to_string();
            def.tooltip = "Use this setting to rotate the support pattern on the horizontal plane."
                .to_string();
            def.sidetext = "°".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7368
            def.max = 359_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(0_f64)));
        }
        {
            let def = self
                .def
                .add("support_on_build_plate_only", ConfigOptionType::Bool);
            def.label = "On build plate only".to_string();
            def.category = "Support".to_string();
            def.tooltip =
                "This setting only generates supports that begin on the build plate.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("support_critical_regions_only", ConfigOptionType::Bool);
            def.label = "Support critical regions only".to_string();
            def.category = "Support".to_string();
            def.tooltip =
                "Only create support for critical regions including sharp tail, cantilever, etc."
                    .to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("support_remove_small_overhang", ConfigOptionType::Bool);
            def.label = "Ignore small overhangs".to_string();
            def.category = "Support".to_string();
            def.tooltip = "Ignore small overhangs that possibly don't require support.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(true)));
        }
        {
            let def = self
                .def
                .add("support_top_z_distance", ConfigOptionType::Float);
            def.label = "Top Z distance".to_string();
            def.min = 0_f32;
            def.category = "Support".to_string();
            def.tooltip = "Z gap between the support's top and object.".to_string();
            def.sidetext = "mm".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7413
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.2)));
        }
        {
            let def = self
                .def
                .add("support_bottom_z_distance", ConfigOptionType::Float);
            def.label = "Bottom Z distance".to_string();
            def.category = "Support".to_string();
            def.tooltip = "Z gap between the object and the support bottom. If Support Top Z Distance is 0 and the bottom has interface layers, this value is ignored and the support is printed in direct contact with the object (no gap).".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7422
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.2)));
        }
        {
            let def = self
                .def
                .add("enforce_support_layers", ConfigOptionType::Int);
            def.category = "Support".to_string();
            def.sidetext = "layers".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7434
            def.max = 5000_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            def.set_default_value(Box::new(ConfigOptionInt::new(0)));
        }
        {
            let def = self.def.add("support_filament", ConfigOptionType::Int);
            def.gui_type = GuiType::IEnumOpen;
            def.label = "Support/raft base".to_string();
            def.category = "Support".to_string();
            def.tooltip = "Filament to print support base and raft.\n\"Default\" means no specific filament for support and current filament is used.".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            def.set_default_value(Box::new(ConfigOptionInt::new(0)));
        }
        {
            let def = self
                .def
                .add("support_interface_not_for_body", ConfigOptionType::Bool);
            def.label = "Avoid interface filament for base".to_string();
            def.category = "Support".to_string();
            def.tooltip =
                "Avoid using support interface filament to print support base if possible."
                    .to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            def.set_default_value(Box::new(ConfigOptionBool::new(true)));
        }
        {
            let def = self
                .def
                .add("support_line_width", ConfigOptionType::FloatOrPercent);
            def.label = "Support".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Line width of support. If expressed as a %, it will be computed over the nozzle diameter.".to_string();
            def.sidetext = "mm or %".to_string();
            def.ratio_over = "nozzle_diameter".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7461
            def.max = 1000_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7462
            def.max_literal = 10_f64;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7464
            def.set_default_value(Box::new(ConfigOptionFloatOrPercent::new(0., false)));
        }
        {
            let def = self
                .def
                .add("support_interface_loop_pattern", ConfigOptionType::Bool);
            def.label = "Loop pattern interface".to_string();
            def.category = "Support".to_string();
            def.tooltip = "This covers the top contact layer of the supports with loops. It is disabled by default.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("support_interface_filament", ConfigOptionType::Int);
            def.gui_type = GuiType::IEnumOpen;
            def.label = "Support/raft interface".to_string();
            def.category = "Support".to_string();
            def.tooltip = "Filament to print support interface.\n\"Default\" means no specific filament for support interface and current filament is used.".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            def.set_default_value(Box::new(ConfigOptionInt::new(0)));
        }
        {
            let def = self
                .def
                .add("support_interface_top_layers", ConfigOptionType::Int);
            def.gui_type = GuiType::IEnumOpen;
            def.label = "Top interface layers".to_string();
            def.category = "Support".to_string();
            def.tooltip = "This is the number of top interface layers.".to_string();
            def.sidetext = "layers".to_string();
            def.min = 0_f32;
            def.enum_values.push("0".to_string());
            def.enum_values.push("1".to_string());
            def.enum_values.push("2".to_string());
            def.enum_values.push("3".to_string());
            def.enum_labels.push("0".to_string());
            def.enum_labels.push("1".to_string());
            def.enum_labels.push("2".to_string());
            def.enum_labels.push("3".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7499
            def.set_default_value(Box::new(ConfigOptionInt::new(3)));
        }
        {
            let support_interface_top_layers_enum_values = self.def.options
                ["support_interface_top_layers"]
                .enum_values
                .clone();
            let support_interface_top_layers_enum_labels = self.def.options
                ["support_interface_top_layers"]
                .enum_labels
                .clone();
            let def = self
                .def
                .add("support_interface_bottom_layers", ConfigOptionType::Int);
            def.gui_type = GuiType::IEnumOpen;
            def.label = "Bottom interface layers".to_string();
            def.category = "Support".to_string();
            def.tooltip = "Number of bottom interface layers.".to_string();
            def.sidetext = "layers".to_string();
            def.min = -1_f32;
            def.enum_values.push("-1".to_string());
            def.enum_values
                .extend(support_interface_top_layers_enum_values);
            def.enum_labels.push("Same as top".to_string());
            def.enum_labels
                .extend(support_interface_top_layers_enum_labels);
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionInt::new(0)));
        }
        {
            let def = self
                .def
                .add("support_interface_spacing", ConfigOptionType::Float);
            def.label = "Top interface spacing".to_string();
            def.category = "Support".to_string();
            def.tooltip = "Spacing of interface lines. Zero means solid interface.\nForce using solid interface when support ironing is enabled.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7523
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.5)));
        }
        {
            let def = self
                .def
                .add("support_bottom_interface_spacing", ConfigOptionType::Float);
            def.label = "Bottom interface spacing".to_string();
            def.category = "Support".to_string();
            def.tooltip = "This is the spacing of bottom interface lines. 0 means solid interface."
                .to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7533
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.5)));
        }
        {
            let def = self
                .def
                .add("support_interface_speed", ConfigOptionType::Floats);
            def.label = "Support interface".to_string();
            def.category = "Speed".to_string();
            def.tooltip = "This is the speed for support interfaces.".to_string();
            def.sidetext = "mm/s".to_string();
            def.min = 1_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7543
            v.values.push(80_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("support_base_pattern", ConfigOptionType::Enum);
            def.label = "Base pattern".to_string();
            def.category = "Support".to_string();
            def.tooltip = "Line pattern of support.\n\nThe Default option for Tree supports is Hollow, which means no base pattern. For other support types, the Default option is the Rectilinear pattern.\n\nNOTE: For Organic supports, the two walls are supported only with the Hollow/Default base pattern. The Lightning base pattern is supported only by Tree Slim/Strong/Hybrid supports. For the other support types, the Rectilinear will be used instead of Lightning.".to_string();
            def.enum_keys_map = keys_map_support_material_pattern();
            def.enum_values.push("default".to_string());
            def.enum_values.push("rectilinear".to_string());
            def.enum_values.push("rectilinear-grid".to_string());
            def.enum_values.push("honeycomb".to_string());
            def.enum_values.push("lightning".to_string());
            def.enum_values.push("hollow".to_string());
            def.enum_labels.push("Default".to_string());
            def.enum_labels.push("Rectilinear".to_string());
            def.enum_labels.push("Rectilinear grid".to_string());
            def.enum_labels.push("Honeycomb".to_string());
            def.enum_labels.push("Lightning".to_string());
            def.enum_labels.push("Hollow".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionEnum::new(
                SupportMaterialPattern::smpDefault,
            )));
        }
        {
            let def = self
                .def
                .add("support_interface_pattern", ConfigOptionType::Enum);
            def.label = "Interface pattern".to_string();
            def.category = "Support".to_string();
            def.tooltip = "This is the line pattern for support interfaces. The default pattern for non-soluble support interfaces is Rectilinear while the default pattern for soluble support interfaces is Concentric.".to_string();
            def.enum_keys_map = keys_map_support_material_interface_pattern();
            def.enum_values.push("auto".to_string());
            def.enum_values.push("rectilinear".to_string());
            def.enum_values.push("concentric".to_string());
            def.enum_values.push("spiralinset".to_string());
            def.enum_values.push("rectilinear_interlaced".to_string());
            def.enum_values.push("grid".to_string());
            def.enum_labels.push("Default".to_string());
            def.enum_labels.push("Rectilinear".to_string());
            def.enum_labels.push("Concentric".to_string());
            def.enum_labels.push("Spiral Inset".to_string());
            def.enum_labels.push("Rectilinear Interlaced".to_string());
            def.enum_labels.push("Grid".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionEnum::new(
                SupportMaterialInterfacePattern::smipAuto,
            )));
        }
        {
            let def = self
                .def
                .add("support_base_pattern_spacing", ConfigOptionType::Float);
            def.label = "Base pattern spacing".to_string();
            def.category = "Support".to_string();
            def.tooltip = "This determines the spacing between support lines.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7597
            def.set_default_value(Box::new(ConfigOptionFloat::new(2.5)));
        }
        {
            let def = self.def.add("support_expansion", ConfigOptionType::Float);
            def.label = "Normal support expansion".to_string();
            def.category = "Support".to_string();
            def.tooltip =
                "Expand (+) or shrink (-) the horizontal span of normal support.".to_string();
            def.sidetext = "mm".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(0_f64)));
        }
        {
            let def = self.def.add("support_speed", ConfigOptionType::Floats);
            def.label = "Support".to_string();
            def.category = "Speed".to_string();
            def.tooltip = "This is the speed for support.".to_string();
            def.sidetext = "mm/s".to_string();
            def.min = 1_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7615
            v.values.push(80_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("support_style", ConfigOptionType::Enum);
            def.label = "Style".to_string();
            def.category = "Support".to_string();
            def.tooltip = "Style and shape of the support.\n\nFor normal support, projecting the supports into a regular grid will create more stable supports (default), while snug support towers will save material and reduce object scarring.\n\nFor tree support, slim and organic style will merge branches more aggressively and save a lot of material (default organic), while hybrid style will create similar structure to normal support under large flat overhangs.".to_string();
            def.enum_keys_map = keys_map_support_material_style();
            def.enum_values.push("default".to_string());
            def.enum_values.push("grid".to_string());
            def.enum_values.push("snug".to_string());
            def.enum_values.push("organic".to_string());
            def.enum_values.push("tree_slim".to_string());
            def.enum_values.push("tree_strong".to_string());
            def.enum_values.push("tree_hybrid".to_string());
            def.enum_labels.push("Default (Grid/Organic)".to_string());
            def.enum_labels.push("Grid".to_string());
            def.enum_labels.push("Snug".to_string());
            def.enum_labels.push("Organic".to_string());
            def.enum_labels.push("Tree Slim".to_string());
            def.enum_labels.push("Tree Strong".to_string());
            def.enum_labels.push("Tree Hybrid".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionEnum::new(
                SupportMaterialStyle::smsDefault,
            )));
        }
        {
            let def = self
                .def
                .add("independent_support_layer_height", ConfigOptionType::Bool);
            def.label = "Independent support layer height".to_string();
            def.category = "Support".to_string();
            def.tooltip = "Support layer uses layer height independent with object layer. This is to support customizing Z-gap and save print time. This option will be invalid when the prime tower is enabled.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(true)));
        }
        {
            let def = self
                .def
                .add("support_threshold_angle", ConfigOptionType::Int);
            def.label = "Threshold angle".to_string();
            def.category = "Support".to_string();
            def.tooltip = "Support will be generated for overhangs whose slope angle is below the threshold. The smaller this value is, the steeper the overhang that can be printed without support.\n\nNote: If set to 0, normal supports use the Threshold overlap instead, while tree supports fall back to a default value of 30.".to_string();
            def.sidetext = "°".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7660
            def.max = 90_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7662
            def.set_default_value(Box::new(ConfigOptionInt::new(30)));
        }
        {
            let def = self.def.add(
                "support_threshold_overlap",
                ConfigOptionType::FloatOrPercent,
            );
            def.label = "Threshold overlap".to_string();
            def.category = "Support".to_string();
            def.tooltip = "If threshold angle is zero, support will be generated for overhangs whose overlap is below the threshold. The smaller this value is, the steeper the overhang that can be printed without support.".to_string();
            def.sidetext = "mm or %".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7670
            def.max = 100_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7671
            def.max_literal = 0.5;
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7673
            def.set_default_value(Box::new(ConfigOptionFloatOrPercent::new(50., true)));
        }
        {
            let def = self.def.add("build_plate_tilt_x", ConfigOptionType::Float);
            def.label = "Build plate tilt X".to_string();
            def.category = "Support".to_string();
            def.tooltip = "Tilt angle of the build plate along the X axis. A positive value tilts the plate so the +X side is higher, shifting gravity toward -X and increasing overhangs on the +X side. A negative value tilts the -X side higher. Set to 0 for no X-axis tilt. In belt printer mode, this is automatically synced to the belt angle.".to_string();
            def.sidetext = "\u{00b0}".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7683
            def.min = -89_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7684
            def.max = 89_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Expert;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7686
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.)));
        }
        {
            let def = self.def.add("build_plate_tilt_y", ConfigOptionType::Float);
            def.label = "Build plate tilt Y".to_string();
            def.category = "Support".to_string();
            def.tooltip = "Tilt angle of the build plate along the Y axis. A positive value tilts the plate so the +Y side is higher, shifting gravity toward -Y and increasing overhangs on the +Y side. A negative value tilts the -Y side higher. Set to 0 for no Y-axis tilt.".to_string();
            def.sidetext = "\u{00b0}".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7695
            def.min = -89_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7696
            def.max = 89_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Expert;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7698
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.)));
        }
        {
            let def = self.def.add("belt_printer", ConfigOptionType::Bool);
            def.label = "Enable belt printing".to_string();
            def.category = "Printable space".to_string();
            def.tooltip = "Enable belt printer mode. Belt printers use a conveyor belt as the build surface, tilted at an angle (typically 45 degrees). The slicer will rotate the slicing plane and transform G-code coordinates for the tilted build surface.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("belt_printer_infinite_y", ConfigOptionType::Bool);
            def.label = "Infinite Y axis".to_string();
            def.category = "Printable space".to_string();
            def.tooltip = "Enable infinite Y axis for belt printers. When enabled, the Y axis build volume limit is effectively removed, allowing objects of any length to be printed along the belt direction.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(true)));
        }
        {
            let def = self.def.add("belt_slice_rotation", ConfigOptionType::Enum);
            def.label = "Belt tilt axis".to_string();
            def.category = "Printable space".to_string();
            def.tooltip = "Axis the mesh is rotated about before slicing. This is the belt printer's tilt: an isometric (no distortion) rotation that also drives bed rendering and support gravity tilt, and that the G-code back-transform inverts before the machine-frame shear/scale and remap. X is the typical gantry tilt (belt travels along Y).".to_string();
            def.enum_keys_map = keys_map_belt_rotation_axis();
            def.enum_values.push("none".to_string());
            def.enum_values.push("x".to_string());
            def.enum_values.push("y".to_string());
            def.enum_values.push("z".to_string());
            def.enum_labels.push("None".to_string());
            def.enum_labels.push("X".to_string());
            def.enum_labels.push("Y".to_string());
            def.enum_labels.push("Z".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            def.set_default_value(Box::new(ConfigOptionEnum::new(BeltRotationAxis::X)));
        }
        {
            let def = self
                .def
                .add("belt_slice_rotation_angle", ConfigOptionType::Float);
            def.label = "Belt tilt angle".to_string();
            def.category = "Printable space".to_string();
            def.tooltip = "Tilt angle of the belt surface, in degrees. Most belt printers use 45°. Positive values rotate counter-clockwise looking down the positive tilt axis; the magnitude is also the physical belt tilt used for bed rendering and support gravity.".to_string();
            def.sidetext = "°".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7743
            def.min = -180.;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7744
            def.max = 180.;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7746
            def.set_default_value(Box::new(ConfigOptionFloat::new(45.)));
        }
        {
            let def = self
                .def
                .add("belt_frame_tilt_decouple", ConfigOptionType::Bool);
            def.label = "Decouple machine-frame tilt".to_string();
            def.category = "Printable space".to_string();
            def.tooltip = "Expert override: set the machine-frame (G-code shear/scale) tilt angle independently of the pre-slice rotation angle. When disabled, the machine-frame transform is derived from the belt tilt angle, so a single angle drives both stages. Enable only to compensate for a machine whose physical gantry tilt differs from the slicing rotation.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Expert;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("belt_frame_tilt_angle", ConfigOptionType::Float);
            def.label = "Machine-frame tilt angle".to_string();
            def.category = "Printable space".to_string();
            def.tooltip = "Tilt angle (degrees) used to derive the machine-frame shear (cot) and scale (1/|sin|) applied to G-code. Only used when 'Decouple machine-frame tilt' is enabled; otherwise the belt tilt angle is used.".to_string();
            def.sidetext = "°".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7766
            def.min = -89.9;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7767
            def.max = 89.9;
            def.mode = slic3r_config::config::ConfigOptionMode::Expert;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7769
            def.set_default_value(Box::new(ConfigOptionFloat::new(45.)));
        }
        // The G-code axis remap with sign (`PrintConfig.cpp:7770-7792`). Each field is its own
        // row in the settings tab. The vendored body writes one `add_belt_remap` lambda and
        // calls it three times, so this is one closure called three times rather than three
        // unrolled rows. The labels and tooltips are literals in `L()` at the call sites so
        // they are extracted for translation.
        {
            let mut add_belt_remap =
                |key: &str,
                 label: &str,
                 tooltip: &str,
                 default_axis: RemapAxis,
                 mode: slic3r_config::config::ConfigOptionMode| {
                    let def = self.def.add(key, ConfigOptionType::Enum);
                    def.label = label.to_string();
                    def.category = "Printable space".to_string();
                    def.tooltip = tooltip.to_string();
                    def.enum_keys_map = keys_map_remap_axis();
                    // FIDELITY(port): the vendored braced-initialiser order of `enum_values` and
                    // `enum_labels` (`PrintConfig.cpp:7779-7780`), which pair positionally and
                    // reach output as the combo box's order.
                    for value in [
                        "pos_x", "pos_y", "pos_z", "neg_x", "neg_y", "neg_z", "rev_x", "rev_y",
                        "rev_z",
                    ] {
                        def.enum_values.push(value.to_string());
                    }
                    for label in [
                        "+X", "+Y", "+Z", "-X", "-Y", "-Z", "Rev X", "Rev Y", "Rev Z",
                    ] {
                        def.enum_labels.push(label.to_string());
                    }
                    // Visibility may also be gated by `toggle_line` in `Tab.cpp`.
                    def.mode = mode;
                    def.set_default_value(Box::new(ConfigOptionEnum::new(default_axis)));
                };
            add_belt_remap(
                "gcode_remap_x",
                "G-code remap X",
                "Which slicing axis maps to machine X in G-code output. Applied AFTER slicing, during G-code generation.",
                RemapAxis::PosX,
                slic3r_config::config::ConfigOptionMode::Develop,
            );
            add_belt_remap(
                "gcode_remap_y",
                "G-code remap Y",
                "Which slicing axis maps to machine Y in G-code output. Applied AFTER slicing, during G-code generation.",
                RemapAxis::PosY,
                slic3r_config::config::ConfigOptionMode::Develop,
            );
            add_belt_remap(
                "gcode_remap_z",
                "G-code remap Z",
                "Which slicing axis maps to machine Z in G-code output. Applied AFTER slicing, during G-code generation.",
                RemapAxis::PosZ,
                slic3r_config::config::ConfigOptionMode::Develop,
            );
        }
        {
            let def = self
                .def
                .add("belt_support_floor_offset", ConfigOptionType::Float);
            def.label = "Support Floor Z offset".to_string();
            def.category = "Printable space".to_string();
            def.tooltip = "Shifts the computed belt floor up or down (mm). Negative values lower the floor, allowing more supports to survive. Use this to diagnose belt floor formula issues.".to_string();
            def.sidetext = "mm".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7805
            def.min = -500_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7806
            def.max = 500_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(0_f64)));
        }
        {
            let def = self
                .def
                .add("enable_belt_purge_tower", ConfigOptionType::Bool);
            def.label = "Enable belt purge tower".to_string();
            def.category = "Multimaterial".to_string();
            def.tooltip = "Belt-printer replacement for the wipe/prime tower. When enabled on a belt printer, a purge prism is automatically generated next to the printed parts and filament-change purging is routed into it (the classic wipe tower cannot be used on belt printers because its G-code bypasses the belt transform). Only available on belt printers.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("belt_purge_tower_width", ConfigOptionType::Float);
            def.label = "Belt purge tower width".to_string();
            def.category = "Printable space".to_string();
            def.tooltip = "Width (machine X, across the belt) of the purge prism that is automatically generated on belt printers when the belt purge tower is enabled and multiple filaments are used. Filament-change purging is routed into this prism's extrusions instead of a classic wipe tower. Its height is computed automatically from the worst-case purge volume per layer: a wider prism results in a shorter one.".to_string();
            def.sidetext = "mm".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7831
            def.min = 1.;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7833
            def.set_default_value(Box::new(ConfigOptionFloat::new(35.)));
        }
        {
            let def = self
                .def
                .add("tree_support_branch_angle", ConfigOptionType::Float);
            def.label = "Tree support branch angle".to_string();
            def.category = "Support".to_string();
            def.tooltip = "This setting determines the maximum overhang angle that the branches of tree support are allowed to make. If the angle is increased, the branches can be printed more horizontally, allowing them to reach farther.".to_string();
            def.sidetext = "°".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7842
            def.max = 60_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7844
            def.set_default_value(Box::new(ConfigOptionFloat::new(40.)));
        }
        {
            let def = self
                .def
                .add("tree_support_branch_angle_organic", ConfigOptionType::Float);
            def.label = "Tree support branch angle".to_string();
            def.category = "Support".to_string();
            def.tooltip = "This setting determines the maximum overhang angle that the branches of tree support are allowed to make. If the angle is increased, the branches can be printed more horizontally, allowing them to reach farther.".to_string();
            def.sidetext = "°".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7853
            def.max = 60_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7855
            def.set_default_value(Box::new(ConfigOptionFloat::new(40.)));
        }
        {
            let def = self
                .def
                .add("tree_support_angle_slow", ConfigOptionType::Float);
            def.label = "Preferred Branch Angle".to_string();
            def.category = "Support".to_string();
            def.tooltip = "The preferred angle of the branches, when they do not have to avoid the model. Use a lower angle to make them more vertical and more stable. Use a higher angle for branches to merge faster.".to_string();
            def.sidetext = "°".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7864
            def.min = 10_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7865
            def.max = 85_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7867
            def.set_default_value(Box::new(ConfigOptionFloat::new(25_f64)));
        }
        {
            let def = self
                .def
                .add("tree_support_branch_distance", ConfigOptionType::Float);
            def.label = "Tree support branch distance".to_string();
            def.category = "Support".to_string();
            def.tooltip =
                "This setting determines the distance between neighboring tree support nodes."
                    .to_string();
            def.sidetext = "mm".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7874
            def.min = 1.0;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7875
            def.max = 10_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7877
            def.set_default_value(Box::new(ConfigOptionFloat::new(5.)));
        }
        {
            let def = self.def.add(
                "tree_support_branch_distance_organic",
                ConfigOptionType::Float,
            );
            def.label = "Tree support branch distance".to_string();
            def.category = "Support".to_string();
            def.tooltip =
                "This setting determines the distance between neighboring tree support nodes."
                    .to_string();
            def.sidetext = "mm".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7884
            def.min = 1.0;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7885
            def.max = 10_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7887
            def.set_default_value(Box::new(ConfigOptionFloat::new(1.)));
        }
        {
            let def = self
                .def
                .add("tree_support_top_rate", ConfigOptionType::Percent);
            def.label = "Branch Density".to_string();
            def.category = "Support".to_string();
            def.tooltip = "Adjusts the density of the support structure used to generate the tips of the branches. A higher value results in better overhangs but the supports are harder to remove, thus it is recommended to enable top support interfaces instead of a high branch density value if dense interfaces are needed.".to_string();
            def.sidetext = "%".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7898
            def.min = 5_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7899
            def.max_literal = 35_f64;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7901
            def.set_default_value(Box::new(ConfigOptionPercent::new(30_f64)));
        }
        {
            let def = self
                .def
                .add("tree_support_auto_brim", ConfigOptionType::Bool);
            def.label = "Auto brim width".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Enabling this option means the width of the brim for tree support will be automatically calculated.".to_string();
            def.set_default_value(Box::new(ConfigOptionBool::new(true)));
        }
        {
            let def = self
                .def
                .add("tree_support_brim_width", ConfigOptionType::Float);
            def.label = "Tree support brim width".to_string();
            def.category = "Quality".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7912
            def.min = 0.0;
            def.tooltip = "Distance from tree branch to the outermost brim line.".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7914
            def.set_default_value(Box::new(ConfigOptionFloat::new(3_f64)));
        }
        {
            let def = self
                .def
                .add("tree_support_tip_diameter", ConfigOptionType::Float);
            def.label = "Tip Diameter".to_string();
            def.category = "Support".to_string();
            def.tooltip = "Branch tip diameter for organic supports.".to_string();
            def.sidetext = "mm".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7922
            def.min = 0.1;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7923
            def.max = 100.;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7925
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.8)));
        }
        {
            let def = self
                .def
                .add("tree_support_branch_diameter", ConfigOptionType::Float);
            def.label = "Tree support branch diameter".to_string();
            def.category = "Support".to_string();
            def.tooltip =
                "This setting determines the initial diameter of support nodes.".to_string();
            def.sidetext = "mm".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7932
            def.min = 1.0;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7933
            def.max = 10_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7935
            def.set_default_value(Box::new(ConfigOptionFloat::new(5.)));
        }
        {
            let def = self.def.add(
                "tree_support_branch_diameter_angle",
                ConfigOptionType::Float,
            );
            def.label = "Branch Diameter Angle".to_string();
            def.category = "Support".to_string();
            def.tooltip = "The angle of the branches' diameter as they gradually become thicker towards the bottom. An angle of 0 will cause the branches to have uniform thickness over their length. A bit of an angle can increase stability of the organic support.".to_string();
            def.sidetext = "°".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7947
            def.max = 15_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7949
            def.set_default_value(Box::new(ConfigOptionFloat::new(5_f64)));
        }
        {
            let def = self.def.add(
                "tree_support_branch_diameter_organic",
                ConfigOptionType::Float,
            );
            def.label = "Tree support branch diameter".to_string();
            def.category = "Support".to_string();
            def.tooltip =
                "This setting determines the initial diameter of support nodes.".to_string();
            def.sidetext = "mm".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7956
            def.min = 1.0;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7957
            def.max = 10_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:7959
            def.set_default_value(Box::new(ConfigOptionFloat::new(2.)));
        }
        {
            let def = self
                .def
                .add("tree_support_wall_count", ConfigOptionType::Int);
            def.label = "Support wall loops".to_string();
            def.category = "Support".to_string();
            def.tooltip = "This setting specifies the count of support walls in the range of [0,2]. 0 means auto.".to_string();
            def.min = 0_f32;
            def.max = 2_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionInt::new(0)));
        }
        {
            let def = self
                .def
                .add("tree_support_with_infill", ConfigOptionType::Bool);
            def.label = "Tree support with infill".to_string();
            def.category = "Support".to_string();
            def.tooltip = "This setting specifies whether to add infill inside large hollows of tree support.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("support_ironing", ConfigOptionType::Bool);
            def.label = "Ironing Support Interface".to_string();
            def.category = "Support".to_string();
            def.tooltip = "Ironing is using small flow to print on same height of support interface again to make it more smooth. This setting controls whether support interface being ironed. When enabled, support interface will be extruded as solid too.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("support_ironing_pattern", ConfigOptionType::Enum);
            def.label = "Support Ironing Pattern".to_string();
            def.tooltip = "The pattern that will be used when ironing.".to_string();
            def.category = "Support".to_string();
            def.enum_keys_map = keys_map_infill_pattern();
            def.enum_values.push("rectilinear".to_string());
            def.enum_values.push("concentric".to_string());
            def.enum_labels.push("Rectilinear".to_string());
            def.enum_labels.push("Concentric".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionEnum::new(
                InfillPattern::ipRectilinear,
            )));
        }
        {
            let def = self
                .def
                .add("support_ironing_flow", ConfigOptionType::Percent);
            def.label = "Support Ironing flow".to_string();
            def.category = "Support".to_string();
            def.tooltip = "The amount of material to extrude during ironing. Relative to flow of normal support interface layer height. Too high value results in overextrusion on the surface.".to_string();
            def.sidetext = "%".to_string();
            def.ratio_over = "layer_height".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8005
            def.max = 100_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8007
            def.set_default_value(Box::new(ConfigOptionPercent::new(10_f64)));
        }
        {
            let def = self
                .def
                .add("support_ironing_spacing", ConfigOptionType::Float);
            def.label = "Support Ironing line spacing".to_string();
            def.category = "Support".to_string();
            def.tooltip = "This is the distance between the lines used for ironing.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.max = 1_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8017
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.1)));
        }
        {
            let def = self
                .def
                .add("activate_chamber_temp_control", ConfigOptionType::Bools);
            def.label = "Activate temperature control".to_string();
            def.tooltip = "Enable this option for automated chamber temperature control. This option activates the emitting of an M191 command before the \"machine_start_gcode\" which sets the chamber temperature and waits until it is reached. In addition, it emits an M141 command at the end of the print to turn off the chamber heater, if present.\n\nThis option relies on the firmware supporting the M191 and M141 commands either via macros or natively and is usually used when an active chamber heater is installed.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            let mut v = ConfigOptionBools::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("chamber_temperature", ConfigOptionType::Ints);
            def.label = "Chamber temperature".to_string();
            def.tooltip = "For high-temperature materials like ABS, ASA, PC, and PA, a higher chamber temperature can help suppress or reduce warping and potentially lead to higher interlayer bonding strength. However, at the same time, a higher chamber temperature will reduce the efficiency of air filtration for ABS and ASA.\n\nFor PLA, PETG, TPU, PVA, and other low-temperature materials, this option should be disabled (set to 0) as the chamber temperature should be low to avoid extruder clogging caused by material softening at the heat break.\n\nIf enabled, this parameter also sets a G-code variable named chamber_temperature, which can be used to pass the desired chamber temperature to your print start macro, or a heat soak macro like this:\nPRINT_START (other variables) CHAMBER_TEMP=[chamber_temperature]\nThis may be useful if your printer does not support M141/M191 commands, or if you desire to handle heat soaking in the print start macro if no active chamber heater is installed.".to_string();
            def.sidetext = "\u{2103}".to_string();
            def.full_label = "Chamber temperature".to_string();
            def.min = 0_f32;
            def.max = max_temp;
            let mut v = ConfigOptionInts::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("chamber_minimal_temperature", ConfigOptionType::Ints);
            def.label = "Minimal".to_string();
            def.tooltip = "This is the chamber temperature at which printing should start, while the chamber continues heating toward the \"Target\" chamber temperature. For example, set the Target to 60 and the Minimal to 50 to begin printing once the chamber reaches 50℃, without waiting for the full 60℃.\n\nIt sets a G-code variable named chamber_minimal_temperature, which can be passed to your print start macro or a heat soak macro, like this:\nPRINT_START (other variables) CHAMBER_MIN_TEMP=[chamber_minimal_temperature]\n\nUnlike the \"Target\" chamber temperature, this option does not emit any M141/M191 commands; it only exposes the value to your custom G-code. It should not exceed the \"Target\" chamber temperature.".to_string();
            def.sidetext = "\u{2103}".to_string();
            def.full_label = "Chamber minimal temperature".to_string();
            def.min = 0_f32;
            def.max = max_temp;
            let mut v = ConfigOptionInts::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("nozzle_temperature", ConfigOptionType::Ints);
            def.label = "Other layers".to_string();
            def.tooltip = "Nozzle temperature after the first layer".to_string();
            def.sidetext = "\u{2103}".to_string();
            def.full_label = "Nozzle temperature".to_string();
            def.min = 0_f32;
            def.max = max_temp;
            let mut v = ConfigOptionInts::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8071
            v.values.push(200);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("nozzle_temperature_range_low", ConfigOptionType::Ints);
            def.label = "Min".to_string();
            def.sidetext = "\u{2103}".to_string();
            def.min = 0_f32;
            def.max = max_temp;
            let mut v = ConfigOptionInts::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8079
            v.values.push(190);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("nozzle_temperature_range_high", ConfigOptionType::Ints);
            def.label = "Max".to_string();
            def.sidetext = "\u{2103}".to_string();
            def.min = 0_f32;
            def.max = max_temp;
            let mut v = ConfigOptionInts::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8087
            v.values.push(240);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("head_wrap_detect_zone", ConfigOptionType::Points);
            def.label = "Head wrap detect zone".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            def.set_default_value(Box::new(ConfigOptionPoints::new()));
        }
        {
            let def = self.def.add("detect_thin_wall", ConfigOptionType::Bool);
            def.label = "Detect thin walls".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "This detects thin walls which can\u{2019}t contain two lines and uses a single line to print. It may not print as well because it\u{2019}s not a closed loop.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("change_filament_gcode", ConfigOptionType::String);
            def.label = "Change filament G-code".to_string();
            def.tooltip = "This G-code is inserted when filament is changed, including T commands to trigger tool change.".to_string();
            def.multiline = true;
            def.full_width = true;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8106
            def.height = 5;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
        }
        {
            let def = self
                .def
                .add("change_extrusion_role_gcode", ConfigOptionType::String);
            def.label = "Change extrusion role G-code".to_string();
            def.tooltip = "This G-code is inserted when the extrusion role is changed.".to_string();
            def.multiline = true;
            def.full_width = true;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8115
            def.height = 5;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
        }
        {
            let def = self.def.add(
                "filament_change_extrusion_role_gcode",
                ConfigOptionType::Strings,
            );
            def.label = "Change extrusion role G-code (filament)".to_string();
            def.tooltip = "This G-code is inserted when the extrusion role is changed for the active filament.".to_string();
            def.multiline = true;
            def.full_width = true;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8124
            def.height = 5;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionStrings::new();
            v.values.push("".to_string());
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("top_surface_line_width", ConfigOptionType::FloatOrPercent);
            def.label = "Top surface".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Line width for top surfaces. If expressed as a %, it will be computed over the nozzle diameter.".to_string();
            def.sidetext = "mm or %".to_string();
            def.ratio_over = "nozzle_diameter".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8135
            def.max = 1000_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8136
            def.max_literal = 10_f64;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8138
            def.set_default_value(Box::new(ConfigOptionFloatOrPercent::new(0., false)));
        }
        {
            let def = self.def.add("top_surface_speed", ConfigOptionType::Floats);
            def.label = "Top surface".to_string();
            def.category = "Speed".to_string();
            def.tooltip = "This is the speed for solid top surface infill.".to_string();
            def.sidetext = "mm/s".to_string();
            def.min = 1_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8148
            v.values.push(100_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("top_shell_layers", ConfigOptionType::Int);
            def.label = "Top shell layers".to_string();
            def.category = "Strength".to_string();
            def.sidetext = "layers".to_string();
            def.tooltip = "This is the number of solid layers of top shell, including the top surface layer. When the thickness calculated by this value is thinner than the top shell thickness, the top shell layers will be increased".to_string();
            def.full_label = "Top solid layers".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8157
            def.set_default_value(Box::new(ConfigOptionInt::new(4)));
        }
        {
            let def = self.def.add("top_shell_thickness", ConfigOptionType::Float);
            def.label = "Top shell thickness".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "The number of top solid layers is increased when slicing if the thickness calculated by top shell layers is thinner than this value. This can avoid having too thin a shell when layer height is small. 0 means that this setting is disabled and thickness of top shell is determined simply by the number of top shell layers.".to_string();
            def.full_label = "Top shell thickness".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8166
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.6)));
        }
        {
            let def = self
                .def
                .add("top_color_penetration_layers", ConfigOptionType::Int);
            def.label = "Top paint penetration layers".to_string();
            def.category = "Strength".to_string();
            def.sidetext = "layers".to_string();
            def.tooltip = "The number of layers, including the top surface layer, that take the color painted on a top surface. Set to 0 to use the top shell layers. A value larger than the top shell also colors the sparse infill below the painted area.".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionInt::new(0)));
        }
        {
            let def = self.def.add("separated_infills", ConfigOptionType::Bool);
            def.label = "Separated infills".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Centers the internal infill of each part on itself, as if it were sliced on its own, instead of on the whole assembly. Parts that touch or overlap are treated as one body and share a center; separate parts (or distinct 3D objects) each get their own.\nUseful when an assembly groups several objects that should each keep a consistent, self-centered infill.\nAdaptive Cubic and Support Cubic always center each part on itself, and Lightning infill is generated for the whole object and is unaffected.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Expert;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("center_of_surface_pattern", ConfigOptionType::Enum);
            def.label = "Center surface pattern on".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "Chooses where the centering point of centered top/bottom surface patterns (Archimedean Chords, Octagram Spiral) is placed.\n - Each Surface: centers the pattern on every individual surface region, so each island is symmetric on its own.\n - Each Model: centers the pattern on each connected body. Parts that touch or overlap share one center; parts detached from the rest each get their own.\n - Each Assembly: uses a single shared center for the whole object or assembly.".to_string();
            def.enum_keys_map = keys_map_center_of_surface_pattern();
            def.enum_values.push("each_surface".to_string());
            def.enum_values.push("each_model".to_string());
            def.enum_values.push("each_assembly".to_string());
            def.enum_labels.push("Each Surface".to_string());
            def.enum_labels.push("Each Model".to_string());
            def.enum_labels.push("Each Assembly".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Expert;
            def.set_default_value(Box::new(ConfigOptionEnum::new(
                CenterOfSurfacePattern::Each_Surface,
            )));
        }
        {
            let def = self.def.add("travel_speed", ConfigOptionType::Floats);
            def.label = "Travel".to_string();
            def.tooltip = "This is the speed at which traveling is done.".to_string();
            def.sidetext = "mm/s".to_string();
            def.min = 1_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8217
            v.values.push(120_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("travel_speed_z", ConfigOptionType::Floats);
            def.sidetext = "mm/s".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8228
            v.values.push(0.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("wipe", ConfigOptionType::Bools);
            def.label = "Wipe while retracting".to_string();
            def.tooltip = "This moves the nozzle along the last extrusion path when retracting to clean any leaked material on the nozzle. This can minimize blobs when printing a new part after traveling.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionBools::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("wipe_distance", ConfigOptionType::Floats);
            def.label = "Wipe distance".to_string();
            def.tooltip = "Describe how long the nozzle will move along the last path when retracting.\n\nDepending on how long the wipe operation lasts, how fast and long the extruder/filament retraction settings are, a retraction move may be needed to retract the remaining filament.\n\nSetting a value in the retract amount before wipe setting below will perform any excess retraction before the wipe, else it will be performed after.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8245
            v.values.push(1.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("enable_mixed_color_sublayer", ConfigOptionType::Bool);
            def.label = "Mixed color sublayer".to_string();
            def.tooltip = "Enable mixed color sublayer splitting. When enabled, layers containing mixed color filaments will be split into sub-layers to achieve color mixing effects.".to_string();
            def.category = "Quality".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("enable_prime_tower", ConfigOptionType::Bool);
            def.label = "Enable".to_string();
            def.tooltip = "The wiping tower can be used to clean up residue on the nozzle and stabilize the chamber pressure inside the nozzle in order to avoid appearance defects when printing objects.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("prime_tower_enable_framework", ConfigOptionType::Bool);
            def.label = "Internal ribs".to_string();
            def.tooltip =
                "Enable internal ribs to increase the stability of the prime tower.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("flush_volumes_vector", ConfigOptionType::Floats);
            def.label = "Purging volumes - load/unload volumes".to_string();
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8275
            v.values.push(140.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8275
            v.values.push(140.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8275
            v.values.push(140.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8275
            v.values.push(140.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8275
            v.values.push(140.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8275
            v.values.push(140.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8275
            v.values.push(140.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8275
            v.values.push(140.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("flush_volumes_matrix", ConfigOptionType::Floats);
            def.label = "Purging volumes".to_string();
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8282
            v.values.push(0.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8282
            v.values.push(280.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8282
            v.values.push(280.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8282
            v.values.push(280.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8282
            v.values.push(280.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8282
            v.values.push(0.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8282
            v.values.push(280.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8282
            v.values.push(280.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8282
            v.values.push(280.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8282
            v.values.push(280.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8282
            v.values.push(0.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8282
            v.values.push(280.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8282
            v.values.push(280.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8282
            v.values.push(280.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8282
            v.values.push(280.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8282
            v.values.push(0.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("flush_multiplier", ConfigOptionType::Floats);
            def.label = "Flush multiplier".to_string();
            def.tooltip = "The actual flushing volumes is equal to the flush multiplier value multiplied by the flushing volumes in the table.".to_string();
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8291
            v.values.push(0.3);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("flush_multiplier_fast", ConfigOptionType::Floats);
            def.label = "Flush multiplier (Fast mode)".to_string();
            def.tooltip = "The flush multiplier used in fast purge mode.".to_string();
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8299
            v.values.push(1.2);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("prime_volume", ConfigOptionType::Float);
            def.label = "Prime volume".to_string();
            def.tooltip = "This is the volume of material to prime the extruder with on the tower."
                .to_string();
            def.sidetext = "mm³".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8306
            def.min = 1.0;
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8308
            def.set_default_value(Box::new(ConfigOptionFloat::new(45.)));
        }
        {
            let def = self.def.add("prime_volume_mode", ConfigOptionType::Enum);
            def.label = "Prime volume mode".to_string();
            def.tooltip = "Selects how the wipe-tower prime and flush volumes are computed on multi-extruder printers.".to_string();
            def.enum_keys_map = keys_map_prime_volume_mode();
            def.enum_values.push("Default".to_string());
            def.enum_values.push("Saving".to_string());
            def.enum_values.push("Fast".to_string());
            def.enum_labels.push("Default".to_string());
            def.enum_labels.push("Saving".to_string());
            def.enum_labels.push("Fast".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            def.set_default_value(Box::new(ConfigOptionEnum::new(PrimeVolumeMode::pvmDefault)));
        }
        {
            let def = self.def.add("wipe_tower_x", ConfigOptionType::Floats);
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8332
            v.values.push(15.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("wipe_tower_y", ConfigOptionType::Floats);
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8340
            v.values.push(220.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add("prime_tower_width", ConfigOptionType::Float);
            def.label = "Width".to_string();
            def.tooltip = "This is the width of prime towers.".to_string();
            def.sidetext = "mm".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8346
            def.min = 2.0;
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8348
            def.set_default_value(Box::new(ConfigOptionFloat::new(60.)));
        }
        {
            let def = self
                .def
                .add("wipe_tower_rotation_angle", ConfigOptionType::Float);
            def.label = "Wipe tower rotation angle".to_string();
            def.tooltip = "Wipe tower rotation angle with respect to X axis.".to_string();
            def.sidetext = "°".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8355
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.)));
        }
        {
            let def = self
                .def
                .add("prime_tower_brim_width", ConfigOptionType::Float);
            def.gui_type = GuiType::FEnumOpen;
            def.label = "Brim width".to_string();
            def.tooltip = "Brim width of prime tower, negative number means auto calculated width based on the height of prime tower.".to_string();
            def.sidetext = "mm".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.min = -1_f32;
            def.enum_values.push("-1".to_string());
            def.enum_labels.push("Auto".to_string());
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8366
            def.set_default_value(Box::new(ConfigOptionFloat::new(3.)));
        }
        {
            let def = self
                .def
                .add("wipe_tower_cone_angle", ConfigOptionType::Float);
            def.label = "Stabilization cone apex angle".to_string();
            def.tooltip = "Angle at the apex of the cone that is used to stabilize the wipe tower. Larger angle means wider base.".to_string();
            def.sidetext = "°".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8374
            def.min = 0.;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8375
            def.max = 90.;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8376
            def.set_default_value(Box::new(ConfigOptionFloat::new(30.0)));
        }
        {
            let def = self
                .def
                .add("wipe_tower_max_purge_speed", ConfigOptionType::Float);
            def.label = "Maximum wipe tower print speed".to_string();
            def.tooltip = "The maximum print speed when purging in the wipe tower and printing the wipe tower sparse layers. When purging, if the sparse infill speed or calculated speed from the filament max volumetric speed is lower, the lowest will be used instead.\n\nWhen printing the sparse layers, if the sparse infill speed or calculated speed from the filament max volumetric speed is lower, the lowest will be used instead.\n\nIncreasing this speed may affect the tower's stability as well as increase the force with which the nozzle collides with any blobs that may have formed on the wipe tower.\n\nBefore increasing this parameter beyond the default of 90 mm/s, make sure your printer can reliably bridge at the increased speeds and that ooze when tool changing is well controlled.\n\nFor the wipe tower external perimeters the internal perimeter speed is used, also limited by this setting.".to_string();
            def.sidetext = "mm/s".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8388
            def.min = 10_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8389
            def.set_default_value(Box::new(ConfigOptionFloat::new(90.)));
        }
        {
            let def = self.def.add("wipe_tower_wall_type", ConfigOptionType::Enum);
            def.label = "Wall type".to_string();
            def.tooltip = "Wipe tower outer wall type.\n1. Rectangle: A rectangle with fixed width and height.\n2. Cone: A cone with a fillet at the bottom to help stabilize the wipe tower.\n3. Rib: The default wall type, which adds four ribs to the tower wall for enhanced stability.".to_string();
            def.enum_keys_map = keys_map_wipe_tower_wall_type();
            def.enum_values.push("rectangle".to_string());
            def.enum_values.push("cone".to_string());
            def.enum_values.push("rib".to_string());
            def.enum_labels.push("Rectangle".to_string());
            def.enum_labels.push("Cone".to_string());
            def.enum_labels.push("Rib".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionEnum::new(WipeTowerWallType::wtwRib)));
        }
        {
            let def = self
                .def
                .add("wipe_tower_extra_rib_length", ConfigOptionType::Float);
            def.label = "Extra rib length".to_string();
            def.tooltip = "Positive values can increase the size of the rib wall, while negative values can reduce the size. However, the size of the rib wall can not be smaller than that determined by the cleaning volume.".to_string();
            def.sidetext = "mm".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8412
            def.max = 300_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(0_f64)));
        }
        {
            let def = self
                .def
                .add("wipe_tower_rib_width", ConfigOptionType::Float);
            def.label = "Rib width".to_string();
            def.tooltip =
                "Rib width is always less than half the prime tower side length.".to_string();
            def.sidetext = "mm".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8422
            def.max = 300_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8423
            def.set_default_value(Box::new(ConfigOptionFloat::new(8_f64)));
        }
        {
            let def = self
                .def
                .add("wipe_tower_fillet_wall", ConfigOptionType::Bool);
            def.label = "Fillet wall".to_string();
            def.tooltip = "The wall of prime tower will fillet.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(true)));
        }
        {
            let def = self.def.add("wipe_tower_filament", ConfigOptionType::Int);
            def.gui_type = GuiType::IEnumOpen;
            def.label = "Wipe tower".to_string();
            def.category = "Extruders".to_string();
            def.tooltip = "The extruder to use when printing perimeter of the wipe tower. Set to 0 to use the one that is available (non-soluble would be preferred).".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionInt::new(0)));
        }
        {
            let def = self
                .def
                .add("wiping_volumes_extruders", ConfigOptionType::Floats);
            def.label = "Purging volumes - load/unload volumes".to_string();
            def.tooltip = "This vector saves required volumes to change from/to each tool used on the wipe tower. These values are used to simplify creation of the full purging volumes below.".to_string();
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8447
            v.values.push(70.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8447
            v.values.push(70.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8447
            v.values.push(70.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8447
            v.values.push(70.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8447
            v.values.push(70.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8447
            v.values.push(70.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8447
            v.values.push(70.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8447
            v.values.push(70.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8447
            v.values.push(70.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8447
            v.values.push(70.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("prime_tower_skip_points", ConfigOptionType::Bool);
            def.label = "Skip points".to_string();
            def.tooltip =
                "The wall of prime tower will skip the start points of wipe path.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(true)));
        }
        {
            let def = self
                .def
                .add("prime_tower_flat_ironing", ConfigOptionType::Bool);
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("enable_tower_interface_features", ConfigOptionType::Bool);
            def.label = "Enable tower interface features".to_string();
            def.tooltip =
                "Enable optimized prime tower interface behavior when different materials meet."
                    .to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add(
                "enable_tower_interface_cooldown_during_tower",
                ConfigOptionType::Bool,
            );
            def.label = "Cool down from interface boost during prime tower".to_string();
            def.tooltip = "When interface-layer temperature boost is active, set the nozzle back to print temperature at the start of the prime tower so it cools down during the tower.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("prime_tower_infill_gap", ConfigOptionType::Percent);
            def.label = "Infill gap".to_string();
            def.tooltip = "Infill gap.".to_string();
            def.sidetext = "%".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8476
            def.min = 100_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8477
            def.set_default_value(Box::new(ConfigOptionPercent::new(150_f64)));
        }
        {
            let def = self.def.add("flush_into_infill", ConfigOptionType::Bool);
            def.category = "Flush options".to_string();
            def.label = "Flush into objects' infill".to_string();
            def.tooltip = "Purging after filament change will be done inside objects' infills. This may lower the amount of waste and decrease the print time. If the walls are printed with transparent filament, the mixed color infill will be visible. It will not take effect unless the prime tower is enabled.".to_string();
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("flush_into_support", ConfigOptionType::Bool);
            def.category = "Flush options".to_string();
            def.label = "Flush into objects' support".to_string();
            def.tooltip = "Purging after filament change will be done inside objects' support. This may lower the amount of waste and decrease the print time. It will not take effect unless a prime tower is enabled.".to_string();
            def.set_default_value(Box::new(ConfigOptionBool::new(true)));
        }
        {
            let def = self.def.add("flush_into_objects", ConfigOptionType::Bool);
            def.category = "Flush options".to_string();
            def.label = "Flush into this object".to_string();
            def.tooltip = "This object will be used to purge the nozzle after a filament change to save filament and decrease the print time. Colors of the objects will be mixed as a result. It will not take effect unless the prime tower is enabled.".to_string();
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("belt_purge_tower_object", ConfigOptionType::Bool);
            def.category = "Flush options".to_string();
            def.label = "Belt purge tower object".to_string();
            def.tooltip = "Marks the auto-generated belt purge prism. Managed automatically; do not set manually.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("wipe_tower_bridging", ConfigOptionType::Float);
            def.label = "Maximal bridging distance".to_string();
            def.tooltip =
                "Maximal distance between supports on sparse infill sections.".to_string();
            def.sidetext = "mm".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8514
            def.set_default_value(Box::new(ConfigOptionFloat::new(10.)));
        }
        {
            let def = self
                .def
                .add("wipe_tower_extra_spacing", ConfigOptionType::Percent);
            def.label = "Wipe tower purge lines spacing".to_string();
            def.tooltip = "Spacing of purge lines on the wipe tower.".to_string();
            def.sidetext = "%".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8521
            def.min = 100.;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8522
            def.max = 300.;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8523
            def.set_default_value(Box::new(ConfigOptionPercent::new(100.)));
        }
        {
            let def = self
                .def
                .add("wipe_tower_extra_flow", ConfigOptionType::Percent);
            def.label = "Extra flow for purging".to_string();
            def.tooltip = "Extra flow used for the purging lines on the wipe tower. This makes the purging lines thicker than they normally would be. The spacing is adjusted automatically.".to_string();
            def.sidetext = "%".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8531
            def.min = 100.;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8532
            def.max = 300.;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8533
            def.set_default_value(Box::new(ConfigOptionPercent::new(100.)));
        }
        {
            let def = self.def.add("idle_temperature", ConfigOptionType::Ints);
            def.label = "Idle temperature".to_string();
            def.tooltip = "Nozzle temperature when the tool is currently not used in multi-tool setups. This is only used when 'Ooze prevention' is active in Print Settings. Set to 0 to disable.".to_string();
            def.sidetext = "\u{2103}".to_string();
            def.min = 0_f32;
            def.max = max_temp;
            let mut v = ConfigOptionInts::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("xy_hole_compensation", ConfigOptionType::Float);
            def.label = "X-Y hole compensation".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Holes in objects will expand or contract in the XY plane by the set value. Positive values make holes bigger and negative values make holes smaller. This function is used to adjust sizes slightly when objects have assembly issues.".to_string();
            def.sidetext = "mm".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(0_f64)));
        }
        {
            let def = self
                .def
                .add("xy_contour_compensation", ConfigOptionType::Float);
            def.label = "X-Y contour compensation".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Contours of objects will expand or contract in the XY plane by the set value. Positive values make contours bigger and negative values make contours smaller. This function is used to adjust sizes slightly when objects have assembly issues.".to_string();
            def.sidetext = "mm".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(0_f64)));
        }
        {
            let def = self.def.add("hole_to_polyhole", ConfigOptionType::Bool);
            def.label = "Convert holes to polyholes".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Search for almost-circular holes that span more than one layer and convert the geometry to polyholes. Use the nozzle size and the (biggest) diameter to compute the polyhole.\nSee http://hydraraptor.blogspot.com/2011/02/polyholes.html".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add(
                "hole_to_polyhole_threshold",
                ConfigOptionType::FloatOrPercent,
            );
            def.label = "Polyhole detection margin".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Maximum defection of a point to the estimated radius of the circle.\nAs cylinders are often exported as triangles of varying size, points may not be on the circle circumference. This setting allows you some leeway to broaden the detection.\nIn mm or in % of the radius.".to_string();
            def.sidetext = "mm or %".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8578
            def.max_literal = 10_f64;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8580
            def.set_default_value(Box::new(ConfigOptionFloatOrPercent::new(0.01, false)));
        }
        {
            let def = self
                .def
                .add("hole_to_polyhole_twisted", ConfigOptionType::Bool);
            def.label = "Polyhole twist".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Rotate the polyhole every layer.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(true)));
        }
        {
            let def = self
                .def
                .add("hole_to_polyhole_max_edges", ConfigOptionType::Int);
            def.label = "Maximum Polyhole edge count".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Maximum number of polyhole edges\nThis setting limits the amount of edges a polyhole can have".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Expert;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8595
            def.min = 3_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8596
            def.set_default_value(Box::new(ConfigOptionInt::new(50)));
        }
        {
            let def = self.def.add("thumbnails", ConfigOptionType::String);
            def.label = "G-code thumbnails".to_string();
            def.tooltip = "Picture sizes to be stored into a .gcode and .sl1 / .sl1s files, in the following format: \"XxY, XxY, ...\"".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.gui_type = GuiType::OneString;
            def.set_default_value(Box::new(ConfigOptionString::new(
                "48x48/PNG,300x300/PNG".to_string(),
            )));
        }
        {
            let def = self.def.add("thumbnails_format", ConfigOptionType::Enum);
            def.label = "Format of G-code thumbnails".to_string();
            def.tooltip = "Format of G-code thumbnails: PNG for best quality, JPG for smallest size, QOI for low memory firmware.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.enum_keys_map = keys_map_gcode_thumbnails_format();
            def.enum_values.push("PNG".to_string());
            def.enum_values.push("JPG".to_string());
            def.enum_values.push("QOI".to_string());
            def.enum_values.push("BTT_TFT".to_string());
            def.enum_values.push("COLPIC".to_string());
            def.enum_labels.push("PNG".to_string());
            def.enum_labels.push("JPG".to_string());
            def.enum_labels.push("QOI".to_string());
            def.enum_labels.push("BTT TT".to_string());
            def.enum_labels.push("ColPic".to_string());
            def.set_default_value(Box::new(ConfigOptionEnum::new(GCodeThumbnailsFormat::PNG)));
        }
        {
            let def = self
                .def
                .add("use_relative_e_distances", ConfigOptionType::Bool);
            def.label = "Use relative E distances".to_string();
            def.tooltip = "Relative extrusion is recommended when using \"label_objects\" option. Some extruders work better with this option unchecked (absolute extrusion mode). Wipe tower is only compatible with relative mode. It is recommended on most printers. Default is checked.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(true)));
        }
        {
            let def = self.def.add("wall_generator", ConfigOptionType::Enum);
            def.label = "Wall generator".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "The classic wall generator produces walls with constant extrusion width and for very thin areas, gap-fill is used. The Arachne engine produces walls with variable extrusion width.".to_string();
            def.enum_keys_map = keys_map_perimeter_generator_type();
            def.enum_values.push("classic".to_string());
            def.enum_values.push("arachne".to_string());
            def.enum_labels.push("Classic".to_string());
            def.enum_labels.push("Arachne".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionEnum::new(
                PerimeterGeneratorType::Arachne,
            )));
        }
        {
            let def = self
                .def
                .add("wall_transition_length", ConfigOptionType::Percent);
            def.label = "Wall transition length".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "When transitioning between different numbers of walls as the part becomes thinner, a certain amount of space is allotted to split or join the wall segments. It's expressed as a percentage over nozzle diameter.".to_string();
            def.sidetext = "%".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8650
            def.set_default_value(Box::new(ConfigOptionPercent::new(100_f64)));
        }
        {
            let def = self.def.add(
                "wall_transition_filter_deviation",
                ConfigOptionType::Percent,
            );
            def.label = "Wall transitioning filter margin".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Prevent transitioning back and forth between one extra wall and one less. This margin extends the range of extrusion widths which follow to [Minimum wall width - margin, 2 * Minimum wall width + margin].\n\nIncreasing this margin reduces the number of transitions, which reduces the number of extrusion starts/stops and travel time. However, large extrusion width variation can lead to under- or overextrusion problems.\n\nIt's expressed as a percentage over nozzle diameter.".to_string();
            def.sidetext = "%".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8665
            def.set_default_value(Box::new(ConfigOptionPercent::new(25_f64)));
        }
        {
            let def = self
                .def
                .add("wall_transition_angle", ConfigOptionType::Float);
            def.label = "Wall transitioning threshold angle".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "When to create transitions between even and odd numbers of walls. A wedge shape with an angle greater than this setting will not have transitions and no walls will be printed in the center to fill the remaining space. Reducing this setting reduces the number and length of these center walls, but may leave gaps or overextrude.".to_string();
            def.sidetext = "°".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8676
            def.min = 1.;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8677
            def.max = 59.;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8678
            def.set_default_value(Box::new(ConfigOptionFloat::new(10.)));
        }
        {
            let def = self
                .def
                .add("wall_distribution_count", ConfigOptionType::Int);
            def.label = "Wall distribution count".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "The number of walls, counted from the center, over which the variation needs to be spread. Lower values mean that the outer walls don't change in width.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.min = 1_f32;
            def.set_default_value(Box::new(ConfigOptionInt::new(1)));
        }
        {
            let def = self.def.add("min_feature_size", ConfigOptionType::Percent);
            def.label = "Minimum feature size".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Minimum thickness of thin features. Model features that are thinner than this value will not be printed, while features thicker than than this value will be widened to the minimum wall width. It's expressed as a percentage over nozzle diameter.".to_string();
            def.sidetext = "%".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8698
            def.set_default_value(Box::new(ConfigOptionPercent::new(25_f64)));
        }
        {
            let def = self.def.add("min_length_factor", ConfigOptionType::Float);
            def.label = "Minimum wall length".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Adjust this value to prevent short, unclosed walls from being printed, which could increase print time. The value is a multiple of the wall width. Higher values remove more and longer walls.\n\nNOTE: Bottom and top surfaces will not be affected by this value to prevent visual gaps on the outside of the model. Adjust 'One wall threshold' in the 'Walls and surfaces' settings below to adjust the sensitivity of what is considered a top-surface. 'One wall threshold' is only visible if this setting is set above the default value of 0.5, or if single-wall top surfaces is enabled.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8709
            def.min = 0.0;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8710
            def.max = 25.0;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8711
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.5)));
        }
        {
            let def = self
                .def
                .add("wall_maximum_resolution", ConfigOptionType::Float);
            def.label = "Maximum wall resolution".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "This value determines the smallest wall line segment length in mm. The smaller you set this value, the more accurate and precise the walls will be.".to_string();
            def.sidetext = "mm".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Expert;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8720
            def.min = 0.005;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8721
            def.max = 0.5;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8722
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.5)));
        }
        {
            let def = self
                .def
                .add("wall_maximum_deviation", ConfigOptionType::Float);
            def.label = "Maximum wall deviation".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "The maximum deviation allowed when reducing the resolution for the 'Maximum wall resolution' setting. If you increase this, the print will be less accurate, but the G-code will be smaller. 'Maximum wall deviation' limits 'Maximum wall resolution', so if the two conflict, 'Maximum wall deviation' takes precedence.".to_string();
            def.sidetext = "mm".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Expert;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8732
            def.min = 0.005;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8733
            def.max = 0.05;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8734
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.025)));
        }
        {
            let def = self
                .def
                .add("initial_layer_min_bead_width", ConfigOptionType::Percent);
            def.label = "First layer minimum wall width".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "The minimum wall width that should be used for the first layer is recommended to be set to the same size as the nozzle. This adjustment is expected to enhance adhesion.".to_string();
            def.sidetext = "%".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8744
            def.set_default_value(Box::new(ConfigOptionPercent::new(85_f64)));
        }
        {
            let def = self.def.add("min_bead_width", ConfigOptionType::Percent);
            def.label = "Minimum wall width".to_string();
            def.category = "Quality".to_string();
            def.tooltip = "Width of the wall that will replace thin features (according to the Minimum feature size) of the model. If the Minimum wall width is thinner than the thickness of the feature, the wall will become as thick as the feature itself. It's expressed as a percentage over nozzle diameter.".to_string();
            def.sidetext = "%".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8756
            def.set_default_value(Box::new(ConfigOptionPercent::new(85_f64)));
        }
        {
            let def = self.def.add(
                "filament_dev_ams_drying_ams_limitations",
                ConfigOptionType::Strings,
            );
            let mut v = ConfigOptionStrings::new();
            v.values.push("".to_string());
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "filament_dev_ams_drying_temperature",
                ConfigOptionType::Floats,
            );
            let mut v = ConfigOptionFloats::new();
            v.values.push(0_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_dev_ams_drying_time", ConfigOptionType::Floats);
            let mut v = ConfigOptionFloats::new();
            v.values.push(0_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "filament_dev_ams_drying_heat_distortion_temperature",
                ConfigOptionType::Floats,
            );
            let mut v = ConfigOptionFloats::new();
            v.values.push(0_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "filament_dev_chamber_drying_bed_temperature",
                ConfigOptionType::Floats,
            );
            let mut v = ConfigOptionFloats::new();
            v.values.push(0_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_dev_chamber_drying_time", ConfigOptionType::Floats);
            let mut v = ConfigOptionFloats::new();
            v.values.push(0_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "filament_dev_drying_softening_temperature",
                ConfigOptionType::Floats,
            );
            let mut v = ConfigOptionFloats::new();
            v.values.push(0_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "filament_dev_drying_cooling_temperature",
                ConfigOptionType::Floats,
            );
            let mut v = ConfigOptionFloats::new();
            v.values.push(0_f64);
            def.set_default_value(Box::new(v));
        }
        // The filament retract overrides (`PrintConfig.cpp:8782-8813`): one nullable row per
        // entry of `filament_extruder_override_keys`, each copying the matching printer-side
        // extruder option's presentation fields and default. Twenty registry rows generated by
        // one loop, so ported as one loop.
        //
        // The vendored body reads `options.find(extruder_raw_key)` and then calls
        // `this->add_nullable(...)` on the same map. In Rust the borrow `add_nullable` returns
        // is live for the rest of the iteration, so the source row is cloned out first. The
        // result cannot differ: the source row is one `init_fff_params` already finished, and
        // nothing in the loop writes to it.
        for opt_key in filament_extruder_override_keys() {
            let filament_prefix = "filament_";
            let extruder_raw_key = opt_key[opt_key
                .find(filament_prefix)
                .expect("every override key carries the filament_ prefix")
                + filament_prefix.len()..]
                .to_string();
            // PORT NOTE: the vendored line is
            // `auto it_opt = options.find(extruder_raw_key); assert(it_opt != options.end());`
            // (`PrintConfig.cpp:8786-8787`) followed by unconditional `it_opt->second` reads.
            // That `assert` is `<cassert>`'s, compiled out by `NDEBUG` in the oracle's Release
            // build, so at the pin a missing key is an unchecked dereference rather than a
            // diagnosed error. There is no value to carry on with either way, so this panics
            // with the key named instead of reproducing the undefined behaviour.
            let src = self
                .def
                .options
                .get(&extruder_raw_key)
                .unwrap_or_else(|| {
                    panic!("{opt_key} overrides {extruder_raw_key}, which is not registered")
                })
                .clone();
            let option_type = src.option_type;
            let simple_mode = matches!(
                opt_key.as_str(),
                "filament_retraction_length"
                    | "filament_z_hop"
                    | "filament_long_retractions_when_cut"
                    | "filament_retraction_distances_when_cut"
            );
            let default_value = override_default(option_type, src.default_value.as_deref());
            let def = self.def.add_nullable(opt_key, option_type);
            def.label = src.label;
            def.full_label = src.full_label;
            def.tooltip = src.tooltip;
            def.sidetext = src.sidetext;
            def.enum_keys_map = src.enum_keys_map;
            def.enum_labels = src.enum_labels;
            def.enum_values = src.enum_values;
            def.min = src.min;
            def.max = src.max;
            // BBS: the specific filament retract settings are shown, because the machine
            // retract settings are hidden behind `comDevelop`.
            def.mode = if simple_mode {
                slic3r_config::config::ConfigOptionMode::Simple
            } else {
                slic3r_config::config::ConfigOptionMode::Advanced
            };
            if let Some(value) = default_value {
                def.set_default_value(value);
            }
        }
        {
            let def = self
                .def
                .add("machine_hotend_change_time", ConfigOptionType::Float);
            def.label = "Hotend change time".to_string();
            def.tooltip = "Time to change hotend.".to_string();
            def.sidetext = "s".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8825
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.0)));
        }
        {
            let def = self.def.add("group_algo_with_time", ConfigOptionType::Bool);
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("machine_prepare_compensation_time", ConfigOptionType::Float);
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8838
            def.set_default_value(Box::new(ConfigOptionFloat::new(260_f64)));
        }
        {
            let def = self.def.add("enable_pre_heating", ConfigOptionType::Bool);
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("handle_hotend_as_extruder", ConfigOptionType::Bool);
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add(
                "filament_max_temperature_drop_when_ec",
                ConfigOptionType::Floats,
            );
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            let mut v = ConfigOptionFloats::new();
            v.values.push(0_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("hotend_cooling_rate", ConfigOptionType::Floats);
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            v.values.push(2_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("hotend_heating_rate", ConfigOptionType::Floats);
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            v.values.push(2_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_change_length_nc", ConfigOptionType::Floats);
            def.label = "Hotend change".to_string();
            def.tooltip = "When changing the hotend, it is recommended to extrude a certain length of filament from the original nozzle. This helps minimize nozzle oozing.".to_string();
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8870
            v.values.push(10_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_ramming_travel_time", ConfigOptionType::Floats);
            def.label = "Extruder change".to_string();
            def.tooltip = "To prevent oozing, the nozzle will perform a reverse travel movement for a certain period after the ramming is complete. The setting define the travel time.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.sidetext = "s".to_string();
            def.min = 0_f32;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            v.values.push(0_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_pre_cooling_temperature", ConfigOptionType::Ints);
            def.label = "Extruder change".to_string();
            def.tooltip = "To prevent oozing, the nozzle temperature will be cooled during ramming. Therefore, the ramming time must be greater than the cooldown time. 0 means disabled.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.sidetext = "\u{2103}".to_string();
            def.min = 0_f32;
            def.nullable = true;
            let mut v = ConfigOptionIntsNullable::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "filament_ramming_volumetric_speed",
                ConfigOptionType::Floats,
            );
            def.label = "Extruder change".to_string();
            def.tooltip = "The maximum volumetric speed for ramming before extruder change, where -1 means using the maximum volumetric speed.".to_string();
            def.sidetext = "mm³/s".to_string();
            def.min = -1_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8896
            def.max = 200_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            v.values.push(-1_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_ramming_travel_time_nc", ConfigOptionType::Floats);
            def.label = "Hotend change".to_string();
            def.tooltip = "To prevent oozing, the nozzle will perform a reverse travel movement for a certain period after the ramming is complete. The setting define the travel time.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.sidetext = "s".to_string();
            def.min = 0_f32;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            v.values.push(0_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "filament_pre_cooling_temperature_nc",
                ConfigOptionType::Ints,
            );
            def.label = "Hotend change".to_string();
            def.tooltip = "To prevent oozing, the nozzle temperature will be cooled during ramming. 0 means disabled.\n\nNote: only a cooldown command and fan activation are triggered, reaching the target temperature is not guaranteed.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.sidetext = "\u{2103}".to_string();
            def.min = 0_f32;
            def.nullable = true;
            let mut v = ConfigOptionIntsNullable::new();
            v.values.push(0);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "filament_ramming_volumetric_speed_nc",
                ConfigOptionType::Floats,
            );
            def.label = "Hotend change".to_string();
            def.tooltip = "The maximum volumetric speed for ramming before a hotend change, where -1 means using the maximum volumetric speed.".to_string();
            def.sidetext = "mm³/s".to_string();
            def.min = -1_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8926
            def.max = 200_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            v.values.push(-1_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_retract_length_nc", ConfigOptionType::Floats);
            def.label = "length when change hotend".to_string();
            def.tooltip = "When this retraction value is modified, it will be used as the amount of filament retracted inside the hotend before changing hotends.".to_string();
            def.sidetext = "mm".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            def.nullable = true;
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8939
            def.max = 18_f32;
            let mut v = ConfigOptionFloatsNullable::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8940
            v.values.push(10.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("extruder_max_nozzle_count", ConfigOptionType::Ints);
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            def.nullable = true;
            let mut v = ConfigOptionIntsNullable::new();
            v.values.push(1);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("support_fast_purge_mode", ConfigOptionType::Bool);
            def.label = "Support fast purge mode".to_string();
            def.tooltip = "Whether this printer supports fast purge mode with optimized temperature and multiplier.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("filament_prime_volume", ConfigOptionType::Floats);
            def.label = "Filament change".to_string();
            def.tooltip = "The volume of material required to prime the extruder on the tower, excluding a hotend change.".to_string();
            def.sidetext = "mm³".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8962
            def.min = 1.0;
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8964
            v.values.push(45.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("filament_prime_volume_nc", ConfigOptionType::Floats);
            def.label = "Hotend change".to_string();
            def.tooltip = "The volume of material required to prime the extruder for a hotend change on the tower.".to_string();
            def.sidetext = "mm³".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8970
            def.min = 1.0;
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:8972
            v.values.push(60.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "filament_preheat_temperature_delta",
                ConfigOptionType::Floats,
            );
            def.label = "Preheat temperature delta".to_string();
            def.tooltip =
                "Temperature delta applied during pre-heating before tool change.".to_string();
            def.sidetext = "\u{2103}".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Develop;
            def.nullable = true;
            let mut v = ConfigOptionFloatsNullable::new();
            v.values.push(0_f64);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self.def.add(
                "detect_narrow_internal_solid_infill",
                ConfigOptionType::Bool,
            );
            def.label = "Detect narrow internal solid infills".to_string();
            def.category = "Strength".to_string();
            def.tooltip = "This option will auto-detect narrow internal solid infill areas. If enabled, the concentric pattern will be used for the area to speed up printing. Otherwise, the rectilinear pattern will be used by default.".to_string();
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(true)));
        }
    }

    /// `PrintConfigDef::init_extruder_option_keys()` (`PrintConfig.cpp:8992-9053`).
    ///
    /// Two braced-initialiser key lists. The vendored comment above the first names the option
    /// types they cover ("ConfigOptionFloats, ConfigOptionPercents, ConfigOptionBools,
    /// ConfigOptionStrings"); the lists themselves are plain string vectors, so this is the
    /// whole of the translation.
    fn init_extruder_option_keys(&mut self) {
        // FIDELITY(port): both lists keep the vendored braced-initialiser order rather than
        // being sorted here, because `extruder_option_keys()`/`extruder_retract_keys()` hand
        // the `std::vector` out whole and callers iterate it (`GCode.cpp`'s per-extruder
        // expansion reads it positionally). The order reaches output, so it is transcribed as
        // written and held to it by `extruder_and_filament_key_lists_match_the_oracle`.
        self.m_extruder_option_keys = [
            "default_filament_profile",
            "default_nozzle_volume_type",
            "deretraction_speed",
            "extruder_colour",
            "extruder_offset",
            "extruder_printable_height",
            "extruder_type",
            "long_retractions_when_cut",
            "max_layer_height",
            "min_layer_height",
            "nozzle_diameter",
            "nozzle_flush_dataset",
            "nozzle_type",
            "nozzle_volume",
            "retract_after_wipe",
            "retract_before_wipe",
            "retract_length_toolchange",
            "retract_lift_above",
            "retract_lift_below",
            "retract_lift_enforce",
            "retract_restart_extra",
            "retract_restart_extra_toolchange",
            "retract_when_changing_layer",
            "retraction_distances_when_cut",
            "retraction_length",
            "retraction_minimum_travel",
            "retraction_speed",
            "travel_slope",
            "wipe",
            "wipe_distance",
            "z_hop",
            "z_hop_types",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        self.m_extruder_retract_keys = [
            "deretraction_speed",
            "long_retractions_when_cut",
            "retract_after_wipe",
            "retract_before_wipe",
            "retract_length_toolchange",
            "retract_lift_above",
            "retract_lift_below",
            "retract_lift_enforce",
            "retract_restart_extra",
            "retract_restart_extra_toolchange",
            "retract_when_changing_layer",
            "retraction_distances_when_cut",
            "retraction_length",
            "retraction_minimum_travel",
            "retraction_speed",
            "travel_slope",
            "wipe",
            "wipe_distance",
            "z_hop",
            "z_hop_types",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        // PORT NOTE: `assert(std::is_sorted(...))` (`PrintConfig.cpp:9052`). The vendored
        // `assert` is `<cassert>`'s, compiled out by `NDEBUG` in the oracle's own Release
        // build, so `debug_assert!` -- not `assert!` -- is the faithful translation: a release
        // Rust build must not keep a check the oracle does not run. The predicate is the
        // vendored one, `is_sorted` over the list just assigned.
        debug_assert!(self.m_extruder_retract_keys.is_sorted());
    }

    /// `PrintConfigDef::init_filament_option_keys()` (`PrintConfig.cpp:9055-9108`).
    ///
    /// **PORT NOTE: dead code at the pin, and kept dead here.** Nothing in the vendored tree
    /// calls this function -- `grep -rn init_filament_option_keys
    /// vendor/OrcaSlicer/src/libslic3r/` finds only its definition (`PrintConfig.cpp:9055`) and
    /// its declaration (`PrintConfig.hpp:872`), and `PrintConfigDef::PrintConfigDef()`
    /// (`:998-1007`) calls `init_common_params`, `init_fff_params`,
    /// `init_extruder_option_keys` and `init_sla_params` and not this one. So
    /// `print_config_def.filament_option_keys()` and `.filament_retract_keys()` are **empty**
    /// on the real oracle, and `PrintConfigDef::new()` here leaves them empty for the same
    /// reason rather than calling this function and diverging. The body is still transcribed,
    /// because the function exists at the pin and a later caller is the fan-out's business, not
    /// this unit's; `#[allow(dead_code)]` records that it is unreachable rather than hiding it.
    /// `extruder_and_filament_key_lists_match_the_oracle` measures the emptiness against the
    /// oracle instead of assuming it.
    #[allow(dead_code)]
    fn init_filament_option_keys(&mut self) {
        // FIDELITY(port): the vendored braced-initialiser order, transcribed as written. Unlike
        // `init_extruder_option_keys`'s pair, this order reaches **no** output at the pin: this
        // function is never called (see the doc comment above), so `m_filament_option_keys` is
        // never built on `print_config_def` and nothing iterates it. The order is kept because
        // it is what the vendored source says and a later caller would depend on it, not
        // because an order choice here is observable today.
        self.m_filament_option_keys = [
            "default_filament_profile",
            "deretraction_speed",
            "filament_colour",
            "filament_diameter",
            "filament_retract_length_nc",
            // The vendored list carries `// "filament_seam_gap",` commented out at
            // `PrintConfig.cpp:9063`; the key is therefore absent, and the comment is kept here
            // so the two lists read line for line.
            "long_retractions_when_cut",
            "max_layer_height",
            "min_layer_height",
            "retract_after_wipe",
            "retract_before_wipe",
            "retract_length_toolchange",
            "retract_lift_above",
            "retract_lift_below",
            "retract_lift_enforce",
            "retract_restart_extra",
            "retract_restart_extra_toolchange",
            "retract_when_changing_layer",
            "retraction_distances_when_cut",
            "retraction_length",
            "retraction_minimum_travel",
            "retraction_speed",
            "volumetric_speed_coefficients",
            "wipe",
            "wipe_distance",
            "z_hop",
            "z_hop_types",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        self.m_filament_retract_keys = [
            "deretraction_speed",
            "filament_retract_length_nc",
            "long_retractions_when_cut",
            "retract_after_wipe",
            "retract_before_wipe",
            "retract_lift_above",
            "retract_lift_below",
            "retract_lift_enforce",
            "retract_restart_extra",
            "retract_when_changing_layer",
            "retraction_distances_when_cut",
            "retraction_length",
            "retraction_minimum_travel",
            "retraction_speed",
            "wipe",
            "wipe_distance",
            "z_hop",
            "z_hop_types",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        // PORT NOTE: `assert(std::is_sorted(...))` (`PrintConfig.cpp:9107`), `debug_assert!`
        // for the same reason as in `init_extruder_option_keys`.
        debug_assert!(self.m_filament_retract_keys.is_sorted());
    }

    /// `PrintConfigDef::init_sla_params()` (`PrintConfig.cpp:9110-9747`).
    fn init_sla_params(&mut self) {
        {
            let def = self.def.add("display_width", ConfigOptionType::Float);
            def.min = 1_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9120
            def.set_default_value(Box::new(ConfigOptionFloat::new(120.)));
        }
        {
            let def = self.def.add("display_height", ConfigOptionType::Float);
            def.min = 1_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9126
            def.set_default_value(Box::new(ConfigOptionFloat::new(68.)));
        }
        {
            let def = self.def.add("display_pixels_x", ConfigOptionType::Int);
            def.label = "X".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9132
            def.min = 100_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9133
            def.set_default_value(Box::new(ConfigOptionInt::new(2560)));
        }
        {
            let def = self.def.add("display_pixels_y", ConfigOptionType::Int);
            def.label = "Y".to_string();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9139
            def.min = 100_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9140
            def.set_default_value(Box::new(ConfigOptionInt::new(1440)));
        }
        {
            let def = self.def.add("display_mirror_x", ConfigOptionType::Bool);
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(true)));
        }
        {
            let def = self.def.add("display_mirror_y", ConfigOptionType::Bool);
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("display_orientation", ConfigOptionType::Enum);
            def.enum_keys_map = keys_map_sla_display_orientation();
            def.enum_values.push("landscape".to_string());
            def.enum_values.push("portrait".to_string());
            def.enum_labels.push(" ".to_string());
            def.enum_labels.push(" ".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionEnum::new(
                SLADisplayOrientation::sladoPortrait,
            )));
        }
        {
            let def = self.def.add("fast_tilt_time", ConfigOptionType::Float);
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9174
            def.set_default_value(Box::new(ConfigOptionFloat::new(5.)));
        }
        {
            let def = self.def.add("slow_tilt_time", ConfigOptionType::Float);
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9183
            def.set_default_value(Box::new(ConfigOptionFloat::new(8.)));
        }
        {
            let def = self.def.add("area_fill", ConfigOptionType::Float);
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9191
            def.set_default_value(Box::new(ConfigOptionFloat::new(50.)));
        }
        {
            let def = self
                .def
                .add("relative_correction", ConfigOptionType::Floats);
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9199
            v.values.push(1.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9199
            v.values.push(1.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("relative_correction_x", ConfigOptionType::Float);
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9207
            def.set_default_value(Box::new(ConfigOptionFloat::new(1.)));
        }
        {
            let def = self
                .def
                .add("relative_correction_y", ConfigOptionType::Float);
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9215
            def.set_default_value(Box::new(ConfigOptionFloat::new(1.)));
        }
        {
            let def = self
                .def
                .add("relative_correction_z", ConfigOptionType::Float);
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9223
            def.set_default_value(Box::new(ConfigOptionFloat::new(1.)));
        }
        {
            let def = self.def.add("absolute_correction", ConfigOptionType::Float);
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9230
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.0)));
        }
        {
            let def = self
                .def
                .add("elefant_foot_min_width", ConfigOptionType::Float);
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9239
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.2)));
        }
        {
            let def = self.def.add("gamma_correction", ConfigOptionType::Float);
            def.min = 0_f32;
            def.max = 1_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9248
            def.set_default_value(Box::new(ConfigOptionFloat::new(1.0)));
        }
        {
            let def = self.def.add("material_colour", ConfigOptionType::String);
            def.gui_type = GuiType::Color;
            def.set_default_value(Box::new(ConfigOptionString::new("#29B2B2".to_string())));
        }
        {
            let def = self.def.add("material_type", ConfigOptionType::String);
            def.gui_type = GuiType::FEnumOpen;
            def.gui_flags = "show_value".to_string();
            def.enum_values.push("Tough".to_string());
            def.enum_values.push("Flexible".to_string());
            def.enum_values.push("Casting".to_string());
            def.enum_values.push("Dental".to_string());
            def.enum_values.push("Heat-resistant".to_string());
            def.set_default_value(Box::new(ConfigOptionString::new("Tough".to_string())));
        }
        {
            let def = self
                .def
                .add("initial_layer_height", ConfigOptionType::Float);
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9276
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.3)));
        }
        {
            let def = self.def.add("bottle_volume", ConfigOptionType::Float);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9282
            def.min = 50_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9283
            def.set_default_value(Box::new(ConfigOptionFloat::new(1000.0)));
        }
        {
            let def = self.def.add("bottle_weight", ConfigOptionType::Float);
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9290
            def.set_default_value(Box::new(ConfigOptionFloat::new(1.0)));
        }
        {
            let def = self.def.add("material_density", ConfigOptionType::Float);
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9297
            def.set_default_value(Box::new(ConfigOptionFloat::new(1.0)));
        }
        {
            let def = self.def.add("bottle_cost", ConfigOptionType::Float);
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9304
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.0)));
        }
        {
            let def = self.def.add("faded_layers", ConfigOptionType::Int);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9309
            def.min = 3_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9310
            def.max = 20_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9312
            def.set_default_value(Box::new(ConfigOptionInt::new(10)));
        }
        {
            let def = self.def.add("min_exposure_time", ConfigOptionType::Float);
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(0_f64)));
        }
        {
            let def = self.def.add("max_exposure_time", ConfigOptionType::Float);
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9328
            def.set_default_value(Box::new(ConfigOptionFloat::new(100_f64)));
        }
        {
            let def = self.def.add("exposure_time", ConfigOptionType::Float);
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9335
            def.set_default_value(Box::new(ConfigOptionFloat::new(10_f64)));
        }
        {
            let def = self
                .def
                .add("min_initial_exposure_time", ConfigOptionType::Float);
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(0_f64)));
        }
        {
            let def = self
                .def
                .add("max_initial_exposure_time", ConfigOptionType::Float);
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9351
            def.set_default_value(Box::new(ConfigOptionFloat::new(150_f64)));
        }
        {
            let def = self
                .def
                .add("initial_exposure_time", ConfigOptionType::Float);
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9358
            def.set_default_value(Box::new(ConfigOptionFloat::new(15_f64)));
        }
        {
            let def = self
                .def
                .add("material_correction", ConfigOptionType::Floats);
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            let mut v = ConfigOptionFloats::new();
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9365
            v.values.push(1.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9365
            v.values.push(1.);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9365
            v.values.push(1.);
            def.set_default_value(Box::new(v));
        }
        {
            let def = self
                .def
                .add("material_correction_x", ConfigOptionType::Float);
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9372
            def.set_default_value(Box::new(ConfigOptionFloat::new(1.)));
        }
        {
            let def = self
                .def
                .add("material_correction_y", ConfigOptionType::Float);
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9379
            def.set_default_value(Box::new(ConfigOptionFloat::new(1.)));
        }
        {
            let def = self
                .def
                .add("material_correction_z", ConfigOptionType::Float);
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9386
            def.set_default_value(Box::new(ConfigOptionFloat::new(1.)));
        }
        {
            let def = self.def.add("material_vendor", ConfigOptionType::String);
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
            def.cli = NOCLI.to_string();
        }
        {
            let def = self
                .def
                .add("default_sla_material_profile", ConfigOptionType::String);
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
            def.cli = NOCLI.to_string();
        }
        {
            let def = self
                .def
                .add("sla_material_settings_id", ConfigOptionType::String);
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
            def.cli = NOCLI.to_string();
        }
        {
            let def = self
                .def
                .add("default_sla_print_profile", ConfigOptionType::String);
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
            def.cli = NOCLI.to_string();
        }
        {
            let def = self
                .def
                .add("sla_print_settings_id", ConfigOptionType::String);
            def.set_default_value(Box::new(ConfigOptionString::new(String::new())));
            def.cli = NOCLI.to_string();
        }
        {
            let def = self.def.add("supports_enable", ConfigOptionType::Bool);
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            def.set_default_value(Box::new(ConfigOptionBool::new(true)));
        }
        {
            let def = self
                .def
                .add("support_head_front_diameter", ConfigOptionType::Float);
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9432
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.4)));
        }
        {
            let def = self
                .def
                .add("support_head_penetration", ConfigOptionType::Float);
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9441
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.2)));
        }
        {
            let def = self.def.add("support_head_width", ConfigOptionType::Float);
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9449
            def.max = 20_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9451
            def.set_default_value(Box::new(ConfigOptionFloat::new(1.0)));
        }
        {
            let def = self
                .def
                .add("support_pillar_diameter", ConfigOptionType::Float);
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9459
            def.max = 15_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9461
            def.set_default_value(Box::new(ConfigOptionFloat::new(1.0)));
        }
        {
            let def = self.def.add(
                "support_small_pillar_diameter_percent",
                ConfigOptionType::Percent,
            );
            def.min = 1_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9469
            def.max = 100_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9471
            def.set_default_value(Box::new(ConfigOptionPercent::new(50_f64)));
        }
        {
            let def = self
                .def
                .add("support_max_bridges_on_pillar", ConfigOptionType::Int);
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9477
            def.max = 50_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9479
            def.set_default_value(Box::new(ConfigOptionInt::new(3)));
        }
        {
            let def = self
                .def
                .add("support_pillar_connection_mode", ConfigOptionType::Enum);
            def.enum_keys_map = keys_map_sla_pillar_connection_mode();
            def.enum_values.push("zigzag".to_string());
            def.enum_values.push("cross".to_string());
            def.enum_values.push("dynamic".to_string());
            def.enum_labels.push(" ".to_string());
            def.enum_labels.push(" ".to_string());
            def.enum_labels.push(" ".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionEnum::new(
                SLAPillarConnectionMode::slapcmDynamic,
            )));
        }
        {
            let def = self
                .def
                .add("support_buildplate_only", ConfigOptionType::Bool);
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("support_pillar_widening_factor", ConfigOptionType::Float);
            def.min = 0_f32;
            def.max = 1_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9508
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.0)));
        }
        {
            let def = self
                .def
                .add("support_base_diameter", ConfigOptionType::Float);
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9516
            def.max = 30_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9518
            def.set_default_value(Box::new(ConfigOptionFloat::new(4.0)));
        }
        {
            let def = self.def.add("support_base_height", ConfigOptionType::Float);
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9527
            def.set_default_value(Box::new(ConfigOptionFloat::new(1.0)));
        }
        {
            let def = self
                .def
                .add("support_base_safety_distance", ConfigOptionType::Float);
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9535
            def.max = 10_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(1_f64)));
        }
        {
            let def = self
                .def
                .add("support_critical_angle", ConfigOptionType::Float);
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9545
            def.max = 90_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9547
            def.set_default_value(Box::new(ConfigOptionFloat::new(45_f64)));
        }
        {
            let def = self
                .def
                .add("support_max_bridge_length", ConfigOptionType::Float);
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9556
            def.set_default_value(Box::new(ConfigOptionFloat::new(15.0)));
        }
        {
            let def = self
                .def
                .add("support_max_pillar_link_distance", ConfigOptionType::Float);
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9565
            def.set_default_value(Box::new(ConfigOptionFloat::new(10.0)));
        }
        {
            let def = self
                .def
                .add("support_object_elevation", ConfigOptionType::Float);
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9573
            def.max = 150_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9575
            def.set_default_value(Box::new(ConfigOptionFloat::new(5.0)));
        }
        {
            let def = self
                .def
                .add("support_points_density_relative", ConfigOptionType::Int);
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9583
            def.set_default_value(Box::new(ConfigOptionInt::new(100)));
        }
        {
            let def = self
                .def
                .add("support_points_minimal_distance", ConfigOptionType::Float);
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9591
            def.set_default_value(Box::new(ConfigOptionFloat::new(1.)));
        }
        {
            let def = self.def.add("pad_enable", ConfigOptionType::Bool);
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            def.set_default_value(Box::new(ConfigOptionBool::new(true)));
        }
        {
            let def = self.def.add("pad_wall_thickness", ConfigOptionType::Float);
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9606
            def.max = 30_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9608
            def.set_default_value(Box::new(ConfigOptionFloat::new(2.0)));
        }
        {
            let def = self.def.add("pad_wall_height", ConfigOptionType::Float);
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9616
            def.max = 30_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9618
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.)));
        }
        {
            let def = self.def.add("pad_brim_size", ConfigOptionType::Float);
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9626
            def.max = 30_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9628
            def.set_default_value(Box::new(ConfigOptionFloat::new(1.6)));
        }
        {
            let def = self
                .def
                .add("pad_max_merge_distance", ConfigOptionType::Float);
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9637
            def.set_default_value(Box::new(ConfigOptionFloat::new(50.0)));
        }
        {
            let def = self.def.add("pad_wall_slope", ConfigOptionType::Float);
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9644
            def.min = 45_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9645
            def.max = 90_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9647
            def.set_default_value(Box::new(ConfigOptionFloat::new(90.0)));
        }
        {
            let def = self.def.add("pad_around_object", ConfigOptionType::Bool);
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("pad_around_object_everywhere", ConfigOptionType::Bool);
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self.def.add("pad_object_gap", ConfigOptionType::Float);
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9669
            def.max = 10_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionFloat::new(1_f64)));
        }
        {
            let def = self
                .def
                .add("pad_object_connector_stride", ConfigOptionType::Float);
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9680
            def.set_default_value(Box::new(ConfigOptionFloat::new(10_f64)));
        }
        {
            let def = self
                .def
                .add("pad_object_connector_width", ConfigOptionType::Float);
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9689
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.5)));
        }
        {
            let def = self
                .def
                .add("pad_object_connector_penetration", ConfigOptionType::Float);
            def.min = 0_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9698
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.3)));
        }
        {
            let def = self.def.add("hollowing_enable", ConfigOptionType::Bool);
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            def.set_default_value(Box::new(ConfigOptionBool::new(false)));
        }
        {
            let def = self
                .def
                .add("hollowing_min_thickness", ConfigOptionType::Float);
            def.min = 1_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9713
            def.max = 10_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Simple;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9715
            def.set_default_value(Box::new(ConfigOptionFloat::new(3.)));
        }
        {
            let def = self.def.add("hollowing_quality", ConfigOptionType::Float);
            def.min = 0_f32;
            def.max = 1_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9724
            def.set_default_value(Box::new(ConfigOptionFloat::new(0.5)));
        }
        {
            let def = self
                .def
                .add("hollowing_closing_distance", ConfigOptionType::Float);
            def.sidetext = "mm".to_string();
            def.min = 0_f32;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9732
            def.max = 10_f32;
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9734
            def.set_default_value(Box::new(ConfigOptionFloat::new(2.0)));
        }
        {
            let def = self.def.add("material_print_speed", ConfigOptionType::Enum);
            def.enum_keys_map = keys_map_sla_material_speed();
            def.enum_values.push("slow".to_string());
            def.enum_values.push("fast".to_string());
            def.enum_labels.push(" ".to_string());
            def.enum_labels.push(" ".to_string());
            def.mode = slic3r_config::config::ConfigOptionMode::Advanced;
            def.set_default_value(Box::new(ConfigOptionEnum::new(SLAMaterialSpeed::slamsFast)));
        }
    }
}

// --------------------------------------------------------------------------------------------
// Step 3: `StaticPrintConfig`, `StaticCacheBase` and `StaticCache<T>`
// (`PrintConfig.hpp:1041-1135`), and the one `port/ownership.tsv` row of this unit.
// --------------------------------------------------------------------------------------------

/// One option field's reachability on its owning config class: the pair of functions that turn
/// a borrow of the owner into a borrow of that field as a `dyn ConfigOption`.
///
/// **PORT NOTE: this stands in for `m_map_name_to_offset`'s `ptrdiff_t`
/// (`PrintConfig.hpp:1069,1073`), and it is a divergence from this task's own plan's Step 3,
/// which specified `std::mem::offset_of!` plus one `unsafe fn` reproducing
/// `optptr`'s pointer arithmetic.** Building it that way does not work. The vendored
/// `optptr` is `reinterpret_cast<ConfigOption*>((char*)owner + offset)` (`:1090`), which relies
/// on every option field being a C++ class with `ConfigOption` as a base, so a byte address is
/// already a valid `ConfigOption*`. Rust's `&dyn ConfigOption` is a *fat* pointer: it carries a
/// vtable chosen by the field's concrete type, and no byte offset alone can produce one. The
/// concrete type therefore has to be known where the borrow is formed, which is exactly what a
/// per-field function pair is. Keeping the offset beside it would add an `unsafe` block, a
/// `// SAFETY:` invariant that is wrong for all 11 classes at once if it is wrong at all (this
/// plan's own named second risk), and a real undefined-behaviour surface, in exchange for a
/// number nothing observable reads: the offsets never reach `keys()`, `optptr`'s result,
/// `hash()`, `operator==` or `apply_to()`, and this unit's proof compares the registry, never a
/// `StaticPrintConfig` instance. `port/ownership.tsv` has no row for
/// `m_map_name_to_offset` either -- it is a `std::map<std::string, ptrdiff_t>`, not a
/// raw-pointer field -- so nothing in the committed inputs requires the offset form. The
/// divergence is named here and in this task's pull request body.
pub struct OptionAccessor<T: 'static> {
    /// `optptr(name, const T *owner) const` (`PrintConfig.hpp:1093-1097`).
    pub get: fn(&T) -> &dyn slic3r_config::config::ConfigOption,
    /// `optptr(name, T *owner) const` (`PrintConfig.hpp:1087-1091`).
    pub get_mut: fn(&mut T) -> &mut dyn slic3r_config::config::ConfigOption,
}
impl<T: 'static> Clone for OptionAccessor<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: 'static> Copy for OptionAccessor<T> {}
impl<T: 'static> std::fmt::Debug for OptionAccessor<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("OptionAccessor")
    }
}

/// `class StaticCacheBase` (`PrintConfig.hpp:1060-1074`), "Internal class for keeping a dynamic
/// map to static options".
#[derive(Debug)]
pub struct StaticCacheBase<T: 'static> {
    /// `m_map_name_to_offset` (`PrintConfig.hpp:1073`), a `std::map<std::string, ptrdiff_t>`.
    /// See [`OptionAccessor`] for why the value is a function pair here rather than a byte
    /// offset. A `BTreeMap` iterates in the same bytewise key order the `std::map` does.
    m_map_name_to_option: std::collections::BTreeMap<String, OptionAccessor<T>>,
}

impl<T: 'static> Default for StaticCacheBase<T> {
    fn default() -> Self {
        StaticCacheBase {
            m_map_name_to_option: std::collections::BTreeMap::new(),
        }
    }
}

impl<T: 'static> StaticCacheBase<T> {
    /// `opt_add(name, base_ptr, opt)` (`PrintConfig.hpp:1065-1070`). The vendored body asserts
    /// the name is not already present and then records the field's offset from `base_ptr`;
    /// there is no `base_ptr` here because there is no offset, and `debug_assert!` carries the
    /// same assertion.
    pub fn opt_add(&mut self, name: &str, accessor: OptionAccessor<T>) {
        debug_assert!(
            !self.m_map_name_to_option.contains_key(name),
            "StaticCacheBase::opt_add: {name} is already registered"
        );
        self.m_map_name_to_option.insert(name.to_string(), accessor);
    }
}

/// `class StaticCache<T> : public StaticCacheBase` (`PrintConfig.hpp:1077-1134`), "Parametrized
/// by the type of the topmost class owning the options".
///
/// `m_defaults` carries `port/ownership.tsv`'s cell for `PrintConfig.hpp:1132`
/// (`StaticCache::m_defaults`, `T *`, class `borrowed`) **verbatim**: `&'a mut T`, with no
/// `Option<_>` wrapper and no owned `T` -- the same way `slic3r-config` takes its own
/// `borrowed` cells verbatim. The vendored constructor sets the pointer to `nullptr` and
/// `initialized()` reports `! m_keys.empty()` (`:1082,1085`), because the cache is a file-scope
/// `static` that exists before anything fills it. A `&'a mut T` cannot be null, so this port
/// makes the cache constructible only through [`StaticCache::finalize`], and the
/// not-yet-initialized state is the enclosing `OnceLock`/`thread_local`'s rather than a null
/// field. `initialized()` is kept and still reads `! m_keys.is_empty()`, so a cache finalized
/// against a `ConfigDef` that resolved no key reports `false` exactly as the vendored one does.
#[derive(Debug)]
pub struct StaticCache<'a, T: 'static> {
    base: StaticCacheBase<T>,
    /// `T *m_defaults` (`PrintConfig.hpp:1132`); `port/ownership.tsv` cell, verbatim.
    m_defaults: &'a mut T,
    /// `std::vector<std::string> m_keys` (`PrintConfig.hpp:1133`).
    m_keys: Vec<String>,
}

impl<'a, T: 'static> StaticCache<'a, T> {
    /// `finalize(T *defaults, const ConfigDef *defs)` (`PrintConfig.hpp:1105-1129`), which is
    /// also the only way to build a cache here; see this struct's own doc comment.
    ///
    /// FIDELITY(port): `m_keys` is filled by walking `defs->options` -- the registry's own
    /// bytewise-by-key order -- and keeping only the keys this `T` actually has a field for, so
    /// `keys()` is in **registry order, not field-declaration order**. That order reaches
    /// output through `ConfigBase::keys()` and everything built on it, so it is transcribed
    /// rather than replaced by the declaration order `StaticCacheBase` was filled in.
    pub fn finalize(base: StaticCacheBase<T>, defaults: &'a mut T, defs: &ConfigDef<'_>) -> Self {
        let mut m_keys: Vec<String> = Vec::with_capacity(base.m_map_name_to_option.len());
        for key in defs.options.keys() {
            let Some(accessor) = base.m_map_name_to_option.get(key).copied() else {
                // "This option is not defined by the ConfigBase of type T."
                continue;
            };
            m_keys.push(key.clone());
            let def = defs.get(key);
            debug_assert!(def.is_some(), "StaticCache::finalize: def is null");
            let Some(def) = def else { continue };
            if let Some(default_value) = def.default_value.as_ref() {
                let opt = (accessor.get_mut)(defaults);
                if default_value.option_type() == ConfigOptionType::Enums {
                    // "Enum list defaults are built without a keys map; set() adopts the one
                    // create_default_option() adds." (`PrintConfig.hpp:1121-1124`).
                    if let Ok(default_option) = def.create_default_option() {
                        let _ = opt.set(default_option.as_ref());
                    }
                } else {
                    let _ = opt.set(default_value.as_ref());
                }
            }
        }
        StaticCache {
            base,
            m_defaults: defaults,
            m_keys,
        }
    }

    /// `initialized()` (`PrintConfig.hpp:1085`).
    pub fn initialized(&self) -> bool {
        !self.m_keys.is_empty()
    }

    /// `const ConfigOption* optptr(name, const T *owner) const`
    /// (`PrintConfig.hpp:1093-1097`).
    pub fn optptr<'o>(
        &self,
        name: &str,
        owner: &'o T,
    ) -> Option<&'o dyn slic3r_config::config::ConfigOption> {
        self.base
            .m_map_name_to_option
            .get(name)
            .map(|a| (a.get)(owner))
    }

    /// `ConfigOption* optptr(name, T *owner) const` (`PrintConfig.hpp:1087-1091`).
    pub fn optptr_mut<'o>(
        &self,
        name: &str,
        owner: &'o mut T,
    ) -> Option<&'o mut dyn slic3r_config::config::ConfigOption> {
        let accessor = *self.base.m_map_name_to_option.get(name)?;
        Some((accessor.get_mut)(owner))
    }

    /// `keys()` (`PrintConfig.hpp:1099`).
    pub fn keys(&self) -> &Vec<String> {
        &self.m_keys
    }

    /// `defaults()` (`PrintConfig.hpp:1100`): the vendored body is `return *m_defaults;`.
    pub fn defaults(&self) -> &T {
        self.m_defaults
    }
}

/// `class StaticPrintConfig : public StaticConfig` (`PrintConfig.hpp:1041-1135`). The vendored
/// class contributes three things to each concrete config class -- `def()` returning
/// `&print_config_def`, the pure-virtual `keys_ref()`, and a `handle_legacy` override
/// delegating to `PrintConfigDef::handle_legacy` -- plus the two nested cache classes, ported
/// above. Rust has no inheritance, so it is a trait the 11 concrete classes implement, with
/// `def()` and `handle_legacy()` provided here so no concrete class repeats them.
pub trait StaticPrintConfig: slic3r_config::config::StaticConfig {
    /// `virtual const t_config_option_keys& keys_ref() const = 0` (`PrintConfig.hpp:1049`).
    fn keys_ref(&self) -> &Vec<String>;

    /// `def()` (`PrintConfig.hpp:1047`): "Overrides ConfigBase::def(). Static configuration
    /// definition."
    fn static_print_config_def() -> &'static ConfigDef<'static>
    where
        Self: Sized,
    {
        &print_config_def().def
    }

    /// `handle_legacy()` (`PrintConfig.hpp:1056-1057`): "Verify whether the opt_key has not
    /// been obsoleted or renamed", delegating to `PrintConfigDef::handle_legacy`.
    fn static_print_config_handle_legacy(opt_key: &mut String, value: &mut String)
    where
        Self: Sized,
    {
        PrintConfigDef::handle_legacy(opt_key, value);
    }
}

// --------------------------------------------------------------------------------------------
// Step 6: the anonymous-namespace string helpers (`PrintConfig.cpp:55-85`) and
// `PrintConfigDef::handle_legacy` (`PrintConfig.cpp:9748-10041`).
//
// `SplitStringAndRemoveDuplicateElement` and `ReplaceString` are defined inside this
// translation unit's own anonymous namespace, so they have internal linkage and no symbol any
// other object file can call -- together with `enum_names_from_keys_map` and
// `assign_printer_technology_to_unknown`, they are what makes this unit's manifest rows read
// `proof path: 3`.
// --------------------------------------------------------------------------------------------

/// `SplitStringAndRemoveDuplicateElement` (`PrintConfig.cpp:56-75`).
///
/// FIDELITY(port): the vendored return type is a `std::set<std::string>`, which iterates in
/// bytewise order and drops duplicates; `BTreeSet<String>` does both the same way. The order
/// reaches output through `handle_legacy`'s `different_settings_to_system` branch, which
/// rewrites the value one split key at a time in this order.
fn split_string_and_remove_duplicate_element(
    s: &str,
    separator: &str,
) -> std::collections::BTreeSet<String> {
    let mut result = std::collections::BTreeSet::new();
    if s.is_empty() {
        return result;
    }
    // The vendored body appends one separator so the final field is terminated, then walks the
    // string byte by byte, taking the text up to the next separator and jumping past it. The
    // `i` index is advanced inside the loop, so this is a `while` here rather than a `for`.
    let strs: Vec<u8> = {
        let mut v = s.as_bytes().to_vec();
        v.extend_from_slice(separator.as_bytes());
        v
    };
    let size = strs.len();
    let sep = separator.as_bytes();
    let mut i: usize = 0;
    while i < size {
        match find_subslice(&strs, sep, i) {
            Some(pos) if pos < size => {
                let sub_str = String::from_utf8_lossy(&strs[i..pos]).into_owned();
                result.insert(sub_str);
                // `i = pos + separator.size() - 1` then the loop's `++i`.
                i = pos + sep.len();
            }
            _ => {
                i += 1;
            }
        }
    }
    result
}

/// `std::string::find(needle, from)` over bytes, the one operation
/// [`split_string_and_remove_duplicate_element`] and [`replace_string`] both need and which
/// Rust's `str` has no direct equivalent of (`str::find` searches from the start only).
fn find_subslice(haystack: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    if needle.is_empty() || from > haystack.len() || needle.len() > haystack.len() - from {
        return None;
    }
    (from..=haystack.len() - needle.len()).find(|&i| &haystack[i..i + needle.len()] == needle)
}

/// `ReplaceString(resource_str, old_str, new_str)` (`PrintConfig.cpp:77-84`): replaces **every**
/// occurrence, advancing past each replacement rather than rescanning it, so a `new_str`
/// containing `old_str` does not loop. `str::replace` has the same every-occurrence,
/// no-rescan behaviour, but it is spelt out here as the vendored loop so the advance is visible.
fn replace_string(resource_str: &mut String, old_str: &str, new_str: &str) {
    if old_str.is_empty() {
        return;
    }
    let mut bytes = resource_str.as_bytes().to_vec();
    let mut pos: usize = 0;
    while let Some(found) = find_subslice(&bytes, old_str.as_bytes(), pos) {
        bytes.splice(found..found + old_str.len(), new_str.bytes());
        // "advance position to continue after replacement"
        pos = found + new_str.len();
    }
    *resource_str = String::from_utf8_lossy(&bytes).into_owned();
}

impl PrintConfigDef {
    /// `PrintConfigDef::handle_legacy(t_config_option_key &opt_key, std::string &value)`
    /// (`PrintConfig.cpp:9748-10041`). "Verify whether the opt_key has not been obsoleted or
    /// renamed. Both opt_key and value may be modified. If the opt_key is no more valid in this
    /// version of Slic3r, opt_key is cleared."
    ///
    /// The whole body is one `else if` chain, so at most one branch runs; it is transcribed as
    /// one chain for the same reason, not reordered into a match on `opt_key` (two branches test
    /// the same key with different values, and one tests a prefix).
    pub fn handle_legacy(opt_key: &mut String, value: &mut String) {
        if opt_key == "curr_bed_type" && value == "SuperTack Plate" {
            *value = "Supertack Plate".to_string();
        } else if opt_key == "enable_wipe_tower" {
            *opt_key = "enable_prime_tower".to_string();
        } else if opt_key == "wipe_tower_width" {
            *opt_key = "prime_tower_width".to_string();
        } else if opt_key == "wiping_volume" {
            *opt_key = "prime_volume".to_string();
        } else if opt_key == "wipe_tower_brim_width" {
            *opt_key = "prime_tower_brim_width".to_string();
        } else if opt_key == "tool_change_gcode" {
            *opt_key = "change_filament_gcode".to_string();
        } else if opt_key == "bridge_fan_speed" {
            *opt_key = "overhang_fan_speed".to_string();
        } else if opt_key == "tpms_adaptive" {
            *opt_key = "adaptive_infill_mode".to_string();
        } else if opt_key == "tpms_interior_density" {
            *opt_key = "adaptive_infill_interior_density".to_string();
        } else if opt_key == "tpms_adaptive_gradient" {
            *opt_key = "adaptive_infill_gradient".to_string();
        } else if opt_key == "infill_extruder" || opt_key == "sparse_infill_filament" {
            // "legacy feature-filament selector. Pre-2.4.0-dev these keys were 1-based and the
            // default value "1" meant "the first/active filament". ... Map the legacy default
            // "1" to "0" (inherit); keep explicit values >1."
            *opt_key = "sparse_infill_filament_id".to_string();
            if value == "1" {
                *value = "0".to_string();
            }
        } else if opt_key == "solid_infill_extruder" || opt_key == "solid_infill_filament" {
            *opt_key = "internal_solid_filament_id".to_string();
            if value == "1" {
                *value = "0".to_string();
            }
        } else if opt_key == "top_solid_infill_filament" {
            *opt_key = "top_surface_filament_id".to_string();
            if value == "1" {
                *value = "0".to_string();
            }
        } else if opt_key == "bottom_solid_infill_filament" {
            *opt_key = "bottom_surface_filament_id".to_string();
            if value == "1" {
                *value = "0".to_string();
            }
        } else if opt_key == "perimeter_extruder"
            || opt_key == "wall_filament"
            || opt_key == "wall_filament_id"
        {
            *opt_key = "outer_wall_filament_id".to_string();
            if value == "1" {
                *value = "0".to_string();
            }
        } else if opt_key == "inner_wall_filament" {
            *opt_key = "inner_wall_filament_id".to_string();
            if value == "1" {
                *value = "0".to_string();
            }
        } else if opt_key == "outer_wall_filament" {
            *opt_key = "outer_wall_filament_id".to_string();
            if value == "1" {
                *value = "0".to_string();
            }
        } else if opt_key == "wipe_tower_extruder" {
            *opt_key = "wipe_tower_filament".to_string();
        } else if opt_key == "support_material_extruder" {
            *opt_key = "support_filament".to_string();
        } else if opt_key == "support_material_interface_extruder" {
            *opt_key = "support_interface_filament".to_string();
        } else if opt_key == "support_material_angle" {
            *opt_key = "support_angle".to_string();
        } else if opt_key == "support_material_enforce_layers" {
            *opt_key = "enforce_support_layers".to_string();
        } else if (opt_key == "initial_layer_print_height"
            || opt_key == "initial_layer_speed"
            || opt_key == "internal_solid_infill_speed"
            || opt_key == "top_surface_speed"
            || opt_key == "support_interface_speed"
            || opt_key == "outer_wall_speed"
            || opt_key == "support_object_xy_distance")
            && value.contains('%')
        {
            // "this is old profile in which value is expressed as percentage. But now these
            // key-value must be absolute value. Reset to default value by erasing these key to
            // avoid parsing error."
            opt_key.clear();
        } else if opt_key == "inherits_cummulative" {
            *opt_key = "inherits_group".to_string();
        } else if opt_key == "compatible_printers_condition_cummulative" {
            *opt_key = "compatible_machine_expression_group".to_string();
        } else if opt_key == "compatible_prints_condition_cummulative" {
            *opt_key = "compatible_process_expression_group".to_string();
        } else if opt_key == "cooling" {
            *opt_key = "slow_down_for_layer_cooling".to_string();
        } else if opt_key == "timelapse_no_toolhead" {
            *opt_key = "timelapse_type".to_string();
        } else if opt_key == "timelapse_type" && value == "2" {
            // "old file "0" is None, "2" is Traditional; new file "0" is Traditional, erase "2""
            *value = "0".to_string();
        } else if opt_key == "support_type" && value == "normal" {
            *value = "normal(manual)".to_string();
        } else if opt_key == "support_type" && value == "tree" {
            *value = "tree(manual)".to_string();
        } else if opt_key == "support_type" && value == "hybrid(auto)" {
            *value = "tree(auto)".to_string();
        } else if opt_key == "support_base_pattern" && value == "none" {
            *value = "hollow".to_string();
        } else if opt_key == "tree_support_wall_count" && value == "-1" {
            *value = "0".to_string();
        } else if opt_key == "different_settings_to_system" {
            // The one recursive branch: each `;`-separated key in the value is run through
            // `handle_legacy` on its own, and where that renames it the whole value has the old
            // spelling replaced by the new one.
            let copy_value: String = value.chars().filter(|c| *c != '"').collect();
            let split_keys = split_string_and_remove_duplicate_element(&copy_value, ";");
            for split_key in split_keys {
                let mut copy_key = split_key.clone();
                let mut copy_inner_value = String::new();
                PrintConfigDef::handle_legacy(&mut copy_key, &mut copy_inner_value);
                if copy_key != split_key {
                    replace_string(value, &split_key, &copy_key);
                }
            }
        } else if opt_key == "overhang_fan_threshold" && value == "5%" {
            *value = "10%".to_string();
        } else if opt_key == "wall_infill_order" {
            if value == "inner wall/outer wall/infill" || value == "infill/inner wall/outer wall" {
                *opt_key = "wall_sequence".to_string();
                *value = "inner wall/outer wall".to_string();
            } else if value == "outer wall/inner wall/infill"
                || value == "infill/outer wall/inner wall"
            {
                *opt_key = "wall_sequence".to_string();
                *value = "outer wall/inner wall".to_string();
            } else if value == "inner-outer-inner wall/infill" {
                *opt_key = "wall_sequence".to_string();
                *value = "inner-outer-inner wall".to_string();
            } else {
                *opt_key = "wall_sequence".to_string();
            }
        } else if opt_key == "nozzle_volume_type"
            || opt_key == "default_nozzle_volume_type"
            || opt_key == "printer_extruder_variant"
            || opt_key == "print_extruder_variant"
            || opt_key == "filament_extruder_variant"
            || opt_key == "extruder_variant_list"
        {
            replace_string(value, "Normal", "Standard");
            replace_string(value, "Big Traffic", "High Flow");
        } else if opt_key == "extruder_type" {
            replace_string(value, "DirectDrive", "Direct Drive");
        } else if opt_key == "enable_power_loss_recovery" {
            // PORT NOTE: `boost::iequals` (`PrintConfig.cpp:9883,9890`) is locale-sensitive in
            // general; both call sites compare against the ASCII literals "true"/"false", for
            // which `eq_ignore_ascii_case` is the same function. Named rather than assumed.
            if value == "1" || value.eq_ignore_ascii_case("true") {
                *value = "enable".to_string();
            } else if value == "0" || value.eq_ignore_ascii_case("false") {
                *value = "disable".to_string();
            }
        } else if opt_key == "enable_arc_fitting" {
            if value == "1" || value.eq_ignore_ascii_case("true") {
                *value = "enabled".to_string();
            } else if value == "0" || value.eq_ignore_ascii_case("false") {
                *value = "disabled".to_string();
            }
        } else if opt_key == "ensure_vertical_shell_thickness" {
            if value == "1" {
                *value = "ensure_all".to_string();
            } else if value == "0" {
                *value = "ensure_moderate".to_string();
            }
        } else if opt_key == "rotate_solid_infill_direction" {
            *opt_key = "solid_infill_rotate_template".to_string();
            if value == "1" {
                *value = "0,90".to_string();
            } else if value == "0" {
                *value = "0".to_string();
            }
        } else if opt_key == "sparse_infill_anchor" {
            *opt_key = "infill_anchor".to_string();
        } else if opt_key == "sparse_infill_anchor_max" {
            *opt_key = "infill_anchor_max".to_string();
        } else if opt_key == "chamber_temperatures" {
            *opt_key = "chamber_temperature".to_string();
        } else if opt_key == "thumbnail_size" {
            *opt_key = "thumbnails".to_string();
        } else if opt_key == "top_one_wall_type" && value != "none" {
            *opt_key = "only_one_wall_top".to_string();
            *value = "1".to_string();
        } else if opt_key == "initial_layer_flow_ratio" {
            *opt_key = "bottom_solid_infill_flow_ratio".to_string();
        } else if opt_key == "ironing_direction" {
            *opt_key = "ironing_angle".to_string();
        } else if opt_key == "ironing_angle" && value.starts_with('-') {
            *value = "0".to_string();
        } else if opt_key == "counterbole_hole_bridging" {
            *opt_key = "counterbore_hole_bridging".to_string();
        } else if opt_key == "draft_shield" && value == "limited" {
            *value = "disabled".to_string();
        } else if (opt_key == "sparse_infill_pattern"
            || opt_key == "top_surface_pattern"
            || opt_key == "bottom_surface_pattern"
            || opt_key == "internal_solid_infill_pattern"
            || opt_key == "ironing_pattern"
            || opt_key == "support_ironing_pattern")
            && value == "zig-zag"
        {
            *value = "rectilinear".to_string();
        } else if opt_key == "filament_map_mode" {
            if value == "Auto" {
                *value = "Auto For Flush".to_string();
            }
        } else if opt_key == "filament_type" {
            // `std::getline(ss, token, ';')` over the value, unquoting each token, renaming
            // "ASA-Aero" to "ASA-AERO", and rebuilding the whole quoted, `;`-joined value only
            // if at least one token was renamed.
            let mut type_list: Vec<String> = Vec::new();
            let mut rebuild_value = false;
            for raw in cpp_getline_split(value, ';') {
                let mut token = raw;
                if token.len() >= 2 && token.starts_with('"') && token.ends_with('"') {
                    token = token[1..token.len() - 1].to_string();
                }
                if token == "ASA-Aero" {
                    token = "ASA-AERO".to_string();
                    rebuild_value = true;
                }
                type_list.push(token);
            }
            if rebuild_value {
                value.clear();
                for (idx, t) in type_list.iter().enumerate() {
                    if idx != 0 {
                        value.push(';');
                    }
                    value.push('"');
                    value.push_str(t);
                    value.push('"');
                }
            }
        } else if opt_key == "prime_tower_rib_wall" {
            if value == "1" {
                *opt_key = "wipe_tower_wall_type".to_string();
                *value = "rib".to_string();
            } else {
                opt_key.clear();
            }
        } else if opt_key == "prime_tower_extra_rib_length" {
            *opt_key = "wipe_tower_extra_rib_length".to_string();
        } else if opt_key == "prime_tower_rib_width" {
            *opt_key = "wipe_tower_rib_width".to_string();
        } else if opt_key == "prime_tower_fillet_wall" {
            *opt_key = "wipe_tower_fillet_wall".to_string();
        } else if opt_key == "extruder_clearance_max_radius" {
            *opt_key = "extruder_clearance_radius".to_string();
        } else if opt_key == "machine_switch_extruder_time" {
            *opt_key = "machine_tool_change_time".to_string();
        } else if opt_key == "wall_direction" && value == "auto" {
            *value = "ccw".to_string();
        } else if opt_key == "is_ixex" {
            // "the IDEX/IQEX parallel printing keys shipped to testers as ixex_* before the
            // feature was renamed IMEX ... is_ixex is the master gate."
            *opt_key = "is_imex".to_string();
        } else if opt_key.starts_with("ixex_") {
            // `opt_key.compare(0, 5, "ixex_") == 0` (`PrintConfig.cpp:9997`): a five-byte
            // prefix test, which `starts_with` reproduces including for a shorter `opt_key`.
            // PORT SRC: src/libslic3r/PrintConfig.cpp:9998
            *opt_key = format!("imex_{}", &opt_key[5..]);
            if opt_key == "imex_carriage_width_x" {
                *opt_key = "imex_nozzle_clearance_x".to_string();
            } else if opt_key == "imex_carriage_width_y" {
                *opt_key = "imex_nozzle_clearance_y".to_string();
            }
        }

        // "Ignore the following obsolete configuration keys" (`PrintConfig.cpp:10005-10030`), a
        // function-scope `static std::set<std::string>`.
        if obsolete_option_keys().contains(opt_key.as_str()) {
            opt_key.clear();
            return;
        }

        if !print_config_def().def.has(opt_key) {
            opt_key.clear();
        }
    }
}

/// `std::getline(ss, token, sep)` over a whole string: the fields between separators, dropping
/// a trailing empty field after a trailing separator (which `str::split` keeps). The same
/// distinction `slic3r-config`'s own `normalize_cli_bool_value` records for its `getline` loop.
fn cpp_getline_split(s: &str, sep: char) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut current = String::new();
    for c in s.chars() {
        if c == sep {
            out.push(std::mem::take(&mut current));
        } else {
            current.push(c);
        }
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

/// The `static std::set<std::string> ignore` of `handle_legacy`
/// (`PrintConfig.cpp:10006-10030`): the obsolete keys whose `opt_key` is cleared on load.
fn obsolete_option_keys() -> &'static std::collections::BTreeSet<&'static str> {
    static KEYS: std::sync::OnceLock<std::collections::BTreeSet<&'static str>> =
        std::sync::OnceLock::new();
    KEYS.get_or_init(|| {
        [
            "acceleration",
            "scale",
            "rotate",
            "duplicate",
            "duplicate_grid",
            "bed_size",
            "print_center",
            "g0",
            "wipe_tower_per_color_wipe",
            "support_sharp_tails",
            "support_remove_small_overhangs",
            "support_with_sheath",
            "tree_support_collision_resolution",
            "tree_support_with_infill",
            "max_volumetric_speed",
            "max_print_speed",
            "support_closing_radius",
            "remove_freq_sweep",
            "remove_bed_leveling",
            "remove_extrusion_calibration",
            "support_transition_line_width",
            "support_transition_speed",
            "bed_temperature",
            "bed_temperature_initial_layer",
            "can_switch_nozzle_type",
            "can_add_auxiliary_fan",
            "extra_flush_volume",
            "spaghetti_detector",
            "adaptive_layer_height",
            "z_hop_type",
            "z_lift_type",
            "bed_temperature_difference",
            "long_retraction_when_cut",
            "retraction_distance_when_cut",
            "internal_bridge_support_thickness",
            "top_area_threshold",
            "reduce_wall_solid_infill",
            "filament_load_time",
            "filament_unload_time",
            "smooth_coefficient",
            "overhang_totally_speed",
            "silent_mode",
            "overhang_speed_classic",
            // "superseded by top_surface_fill_order / bottom_surface_fill_order"
            "anisotropic_surfaces",
            // "Belt printer keys retired before the first release ..."
            "belt_slice_rotation_global",
            "preslice_remap_x",
            "preslice_remap_y",
            "preslice_remap_z",
            "preslice_remap_global",
            "belt_support_z_offset_mode",
            "first_layer_plane",
            "first_layer_plane_offset",
            "belt_preslice_global",
            "gcode_back_transform",
            "belt_support_floor_mode",
            "first_layer_plane_thickness",
        ]
        .into_iter()
        .collect()
    })
}

// <<<APPEND>>>

// --------------------------------------------------------------------------------------------
// Unit tests against the real oracle. **These are not this unit's R19 proof** and carry no
// `// PROOF:` marker: the proof is `crates/slic3r-printconfig/tests/print_config_proof.rs`,
// which asserts exact equality over all 881 registry keys and lands at the plan's Step 7,
// atomically with the `mod print_config;` line, once the table is complete.
//
// What these are for: every assertion below is true of the table as it stands *and* stays true
// as the table fills, so the measurements this task's pull request reports are reproducible
// from the tree rather than only from the session that ran them. `lib.rs` carries
// `pub mod print_config;` (landed in `d6e3f55`), so these five tests are compiled and run by
// `cargo test --workspace` and therefore by CI -- `cargo test -p slic3r-printconfig` reports
// `5 passed`, and CI's oracle-checks job ran them on `2c3b262`. Nothing here weakens into a
// check that cannot fail: each one would fail on a mistranscribed enum value, option type,
// default, CLI spelling or legacy branch.
// --------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use slic3r_proof::proof_units::print_config as ora;

    /// Every key map this module transcribes, by the name of the vendored
    /// `s_keys_map_<NAME>` table it came from.
    fn transcribed_key_maps() -> Vec<(&'static str, &'static TConfigEnumValues)> {
        vec![
            ("PrinterTechnology", keys_map_printer_technology()),
            ("PrintHostType", keys_map_print_host_type()),
            ("AuthorizationType", keys_map_authorization_type()),
            ("GCodeFlavor", keys_map_gcode_flavor()),
            ("BedTempFormula", keys_map_bed_temp_formula()),
            ("PowerLossRecoveryMode", keys_map_power_loss_recovery_mode()),
            ("ArcFittingMode", keys_map_arc_fitting_mode()),
            (
                "CenterOfSurfacePattern",
                keys_map_center_of_surface_pattern(),
            ),
            ("FuzzySkinType", keys_map_fuzzy_skin_type()),
            ("NoiseType", keys_map_noise_type()),
            ("WipeTowerType", keys_map_wipe_tower_type()),
            ("FuzzySkinMode", keys_map_fuzzy_skin_mode()),
            (
                "TopSurfaceExpansionDirection",
                keys_map_top_surface_expansion_direction(),
            ),
            ("InfillPattern", keys_map_infill_pattern()),
            ("IroningType", keys_map_ironing_type()),
            ("WallInfillOrder", keys_map_wall_infill_order()),
            ("WallSequence", keys_map_wall_sequence()),
            ("WallDirection", keys_map_wall_direction()),
            ("SurfaceFillOrder", keys_map_surface_fill_order()),
            ("AdaptiveInfillMode", keys_map_adaptive_infill_mode()),
            (
                "AdaptiveInfillGradient",
                keys_map_adaptive_infill_gradient(),
            ),
            ("PrintSequence", keys_map_print_sequence()),
            ("PrintOrder", keys_map_print_order()),
            ("SlicingMode", keys_map_slicing_mode()),
            ("BeltRotationAxis", keys_map_belt_rotation_axis()),
            ("RemapAxis", keys_map_remap_axis()),
            (
                "SupportMaterialPattern",
                keys_map_support_material_pattern(),
            ),
            ("SupportMaterialStyle", keys_map_support_material_style()),
            (
                "SupportMaterialInterfacePattern",
                keys_map_support_material_interface_pattern(),
            ),
            ("SupportType", keys_map_support_type()),
            ("SeamPosition", keys_map_seam_position()),
            ("SeamScarfType", keys_map_seam_scarf_type()),
            (
                "EnsureVerticalShellThickness",
                keys_map_ensure_vertical_shell_thickness(),
            ),
            ("InternalBridgeFilter", keys_map_internal_bridge_filter()),
            (
                "EnableExtraBridgeLayer",
                keys_map_enable_extra_bridge_layer(),
            ),
            ("GapFillTarget", keys_map_gap_fill_target()),
            ("SLADisplayOrientation", keys_map_sla_display_orientation()),
            (
                "SLAPillarConnectionMode",
                keys_map_sla_pillar_connection_mode(),
            ),
            ("SLAMaterialSpeed", keys_map_sla_material_speed()),
            ("BrimType", keys_map_brim_type()),
            ("TimelapseType", keys_map_timelapse_type()),
            ("SkirtType", keys_map_skirt_type()),
            ("DraftShield", keys_map_draft_shield()),
            (
                "ForwardCompatibilitySubstitutionRule",
                keys_map_forward_compatibility_substitution_rule(),
            ),
            ("OverhangFanThreshold", keys_map_overhang_fan_threshold()),
            ("BedType", keys_map_bed_type()),
            ("LayerSeq", keys_map_layer_seq()),
            ("NozzleType", keys_map_nozzle_type()),
            ("FanDirection", keys_map_fan_direction()),
            ("PrinterStructure", keys_map_printer_structure()),
            ("InputShaperType", keys_map_input_shaper_type()),
            (
                "PerimeterGeneratorType",
                keys_map_perimeter_generator_type(),
            ),
            (
                "ToolChangeOrderingType",
                keys_map_tool_change_ordering_type(),
            ),
            ("ZHopType", keys_map_z_hop_type()),
            (
                "RetractLiftEnforceType",
                keys_map_retract_lift_enforce_type(),
            ),
            ("GCodeThumbnailsFormat", keys_map_gcode_thumbnails_format()),
            (
                "CounterboreHoleBridgingOption",
                keys_map_counterbore_hole_bridging_option(),
            ),
            ("WipeTowerWallType", keys_map_wipe_tower_wall_type()),
            ("ExtruderType", keys_map_extruder_type()),
            ("NozzleVolumeType", keys_map_nozzle_volume_type()),
            ("FilamentMapMode", keys_map_filament_map_mode()),
            ("BedExcludeVolumeMode", keys_map_bed_exclude_volume_mode()),
            ("ImexToolLayout", keys_map_imex_tool_layout()),
            ("ImexVizTheme", keys_map_imex_viz_theme()),
            ("PrimeVolumeMode", keys_map_prime_volume_mode()),
        ]
    }

    /// `enum_names_from_keys_map` (`PrintConfig.cpp:176-187`) on a table with a gap and a value
    /// shared by two names -- the two shapes the registry really contains
    /// (`s_keys_map_NozzleVolumeType` has a gap at 4; `s_keys_map_WallInfillOrder` writes one
    /// key twice).
    #[test]
    fn enum_names_from_keys_map_sizes_to_the_largest_value_and_leaves_gaps_empty() {
        let names = enum_names_from_keys_map(keys_map_nozzle_volume_type());
        assert_eq!(names.len(), 7, "sized to max value + 1");
        assert_eq!(names[0], "Standard");
        assert_eq!(names[4], "", "the reserved gap stays empty");
        assert_eq!(names[6], "Extra High Flow");
    }

    /// Every enum key map this module transcribes, against the real oracle's own
    /// `ConfigOptionDef::enum_keys_map` tables. An enum the registry does not use has no oracle
    /// table to match and is reported, not failed -- but a transcribed table that the registry
    /// *does* use and that differs by one name or one value fails here.
    #[test]
    fn transcribed_enum_key_maps_match_the_oracles() {
        let mut oracle_tables: Vec<std::collections::BTreeMap<String, i32>> = Vec::new();
        for key in ora::oracle_option_keys() {
            let info = ora::oracle_option_info(&key);
            if info.enum_keys_map.is_empty() {
                continue;
            }
            let m: std::collections::BTreeMap<String, i32> = info
                .enum_keys_map
                .iter()
                .map(|e| (e.name.to_string(), e.value))
                .collect();
            if !oracle_tables.contains(&m) {
                oracle_tables.push(m);
            }
        }

        let mine = transcribed_key_maps();
        let mut unused: Vec<&str> = Vec::new();
        for (name, map) in &mine {
            if !oracle_tables.iter().any(|t| t == *map) {
                unused.push(name);
            }
        }
        // Measured at the pin: `wall_infill_order` is not a registry key at all
        // (`handle_legacy` renames it to `wall_sequence`) and the substitution rule is CLI-only,
        // so neither table is reachable through `print_config_def`. Any *other* table failing to
        // match is a mistranscription.
        unused.sort_unstable();
        assert_eq!(
            unused,
            ["ForwardCompatibilitySubstitutionRule", "WallInfillOrder"],
            "every key map the registry uses must match the oracle byte for byte"
        );

        // And the other direction: no oracle table is left unmatched by this module.
        for t in &oracle_tables {
            assert!(
                mine.iter().any(|(_, m)| *m == t),
                "oracle enum_keys_map {t:?} matches no transcribed table"
            );
        }
    }

    /// Every option key this module has transcribed, against the real oracle, on the
    /// behaviourally load-bearing fields. This is the same comparison the Step 7 proof makes;
    /// the difference is that the proof additionally requires the registry to be **complete**,
    /// which it is not yet. So this asserts the two directions that are true today and stay
    /// true as rows are added -- every transcribed key exists in the oracle, and every
    /// transcribed key agrees with it exactly -- and reports the shortfall instead of
    /// asserting it away.
    #[test]
    fn every_transcribed_option_agrees_exactly_with_the_oracle() {
        let oracle_keys: Vec<String> = ora::oracle_option_keys()
            .iter()
            .map(|s| s.to_string())
            .collect();
        let mine = &print_config_def().def.options;

        let mut disagree: Vec<String> = Vec::new();
        for (key, d) in mine.iter() {
            let info = ora::oracle_option_info(key);
            assert!(
                info.found,
                "{key} is in the ported registry and not in the oracle's"
            );
            let mut why: Vec<String> = Vec::new();
            if (d.option_type as i32) != info.option_type {
                why.push(format!(
                    "type mine={} oracle={}",
                    d.option_type as i32, info.option_type
                ));
            }
            if d.nullable != info.nullable {
                why.push(format!(
                    "nullable mine={} oracle={}",
                    d.nullable, info.nullable
                ));
            }
            // Every remaining scalar field of `ConfigOptionDef`, compared the same way. The
            // three enum-valued ones cross the bridge as `i32` casts of the C++ enumerator and
            // are cast the same way here; `slic3r-config` transcribes all three enumerator
            // lists in the vendored declaration order, so the discriminants agree by
            // construction and a future divergence in either would fail here loudly.
            macro_rules! same {
                ($name:literal, $mine:expr, $oracle:expr) => {
                    if $mine != $oracle {
                        why.push(format!(
                            concat!($name, " mine={:?} oracle={:?}"),
                            $mine, $oracle
                        ));
                    }
                };
            }
            same!("gui_type", d.gui_type as i32, info.gui_type);
            same!("gui_flags", d.gui_flags, info.gui_flags);
            same!("plugin_type", d.plugin_type, info.plugin_type);
            same!("label", d.label, info.label);
            same!("full_label", d.full_label, info.full_label);
            same!(
                "printer_technology",
                d.printer_technology as i32,
                info.printer_technology
            );
            same!("category", d.category, info.category);
            same!("tooltip", d.tooltip, info.tooltip);
            same!("sidetext", d.sidetext, info.sidetext);
            same!("cli", d.cli, info.cli);
            same!("cli_params", d.cli_params, info.cli_params);
            same!("ratio_over", d.ratio_over, info.ratio_over);
            same!("multiline", d.multiline, info.multiline);
            same!("full_width", d.full_width, info.full_width);
            same!("is_code", d.is_code, info.is_code);
            same!("readonly", d.readonly, info.readonly);
            same!("height", d.height, info.height);
            same!("width", d.width, info.width);
            same!("min", d.min, info.min);
            same!("max", d.max, info.max);
            same!("max_literal", d.max_literal, info.max_literal);
            same!("mode", d.mode as i32, info.mode);
            let oracle_aliases: Vec<String> = info.aliases.iter().map(|s| s.to_string()).collect();
            same!("aliases", d.aliases, oracle_aliases);
            let oracle_shortcut: Vec<String> =
                info.shortcut.iter().map(|s| s.to_string()).collect();
            same!("shortcut", d.shortcut, oracle_shortcut);
            let oracle_enum_labels: Vec<String> =
                info.enum_labels.iter().map(|s| s.to_string()).collect();
            same!("enum_labels", d.enum_labels, oracle_enum_labels);
            let mine_default = d.default_value.as_ref().map(|v| v.serialize());
            let oracle_default = if info.has_default {
                Some(info.default_serialized.to_string())
            } else {
                None
            };
            if mine_default != oracle_default {
                why.push(format!(
                    "default mine={mine_default:?} oracle={oracle_default:?}"
                ));
            }
            let oracle_values: Vec<String> =
                info.enum_values.iter().map(|s| s.to_string()).collect();
            if d.enum_values != oracle_values {
                why.push(format!(
                    "enum_values mine={:?} oracle={:?}",
                    d.enum_values, oracle_values
                ));
            }
            let oracle_map: Vec<(String, i32)> = info
                .enum_keys_map
                .iter()
                .map(|e| (e.name.to_string(), e.value))
                .collect();
            let mine_map: Vec<(String, i32)> = d
                .enum_keys_map
                .iter()
                .map(|(k, v)| (k.clone(), *v))
                .collect();
            if mine_map != oracle_map {
                why.push(format!(
                    "enum_keys_map mine={mine_map:?} oracle={oracle_map:?}"
                ));
            }
            if !why.is_empty() {
                disagree.push(format!("{key}: {}", why.join("; ")));
            }
        }
        assert!(
            disagree.is_empty(),
            "transcribed options disagreeing with the oracle: {disagree:#?}"
        );
        let missing = oracle_keys
            .iter()
            .filter(|k| !mine.contains_key(*k))
            .count();
        println!(
            "registry agreement: {} of {} oracle keys transcribed and exact; {} not yet transcribed",
            mine.len(),
            oracle_keys.len(),
            missing
        );
    }

    /// `handle_legacy` against the oracle's own, over every registry key plus the legacy,
    /// renamed and obsolete keys the vendored function names, against the value shapes that
    /// reach each of its value-rewriting branches.
    ///
    /// The vendored body's last statement clears `opt_key` when the registry does not know it,
    /// so while the table is incomplete the two sides legitimately differ on a key the oracle
    /// keeps and this registry has not got yet. This asserts the full chain for every case where
    /// the oracle's surviving key is one this registry *does* know, which is exactly the set
    /// where the two sides must agree, and which grows to everything as rows are added.
    ///
    /// **What that deferral excludes today, stated so the case count is not read as more than it
    /// is.** A branch whose key the oracle keeps and this registry has not transcribed is
    /// deferred whatever this module does with it, so the value-rewriting branches of
    /// `filament_type` (the `"ASA-Aero"` rebuild), `support_type`, `extruder_type`,
    /// `enable_power_loss_recovery`, `ensure_vertical_shell_thickness` and `timelapse_type` are
    /// **not** covered here yet, and neither is a rename whose target is untranscribed
    /// (`support_material_angle` -> `support_angle`, for one). What the case count does cover is
    /// the obsolete-key clearing, the `has()`-miss clearing, and every branch whose surviving key
    /// is among the transcribed rows -- `curr_bed_type` -> `"Supertack Plate"` and
    /// `overhang_fan_threshold` -> `"10%"` among them. The wider throwaway probe in
    /// `oracle/.cache/tmp/` is what reaches the rest today, by attributing each difference instead
    /// of deferring it; this test takes over from it as rows land.
    #[test]
    fn handle_legacy_matches_the_oracle() {
        let mut keys: Vec<String> = ora::oracle_option_keys()
            .iter()
            .map(|s| s.to_string())
            .collect();
        for k in [
            "curr_bed_type",
            "enable_wipe_tower",
            "wipe_tower_width",
            "wiping_volume",
            "wipe_tower_brim_width",
            "tool_change_gcode",
            "bridge_fan_speed",
            "tpms_adaptive",
            "tpms_interior_density",
            "tpms_adaptive_gradient",
            "infill_extruder",
            "sparse_infill_filament",
            "solid_infill_extruder",
            "solid_infill_filament",
            "top_solid_infill_filament",
            "bottom_solid_infill_filament",
            "perimeter_extruder",
            "wall_filament",
            "wall_filament_id",
            "inner_wall_filament",
            "outer_wall_filament",
            "wipe_tower_extruder",
            "support_material_extruder",
            "support_material_interface_extruder",
            "support_material_angle",
            "support_material_enforce_layers",
            "initial_layer_print_height",
            "initial_layer_speed",
            "internal_solid_infill_speed",
            "top_surface_speed",
            "support_interface_speed",
            "outer_wall_speed",
            "support_object_xy_distance",
            "inherits_cummulative",
            "compatible_printers_condition_cummulative",
            "compatible_prints_condition_cummulative",
            "cooling",
            "timelapse_no_toolhead",
            "timelapse_type",
            "support_type",
            "support_base_pattern",
            "tree_support_wall_count",
            "different_settings_to_system",
            "overhang_fan_threshold",
            "wall_infill_order",
            "nozzle_volume_type",
            "default_nozzle_volume_type",
            "printer_extruder_variant",
            "print_extruder_variant",
            "filament_extruder_variant",
            "extruder_variant_list",
            "extruder_type",
            "enable_power_loss_recovery",
            "enable_arc_fitting",
            "ensure_vertical_shell_thickness",
            "rotate_solid_infill_direction",
            "sparse_infill_anchor",
            "sparse_infill_anchor_max",
            "chamber_temperatures",
            "thumbnail_size",
            "top_one_wall_type",
            "initial_layer_flow_ratio",
            "ironing_direction",
            "ironing_angle",
            "counterbole_hole_bridging",
            "draft_shield",
            "sparse_infill_pattern",
            "top_surface_pattern",
            "bottom_surface_pattern",
            "internal_solid_infill_pattern",
            "ironing_pattern",
            "support_ironing_pattern",
            "filament_map_mode",
            "filament_type",
            "prime_tower_rib_wall",
            "prime_tower_extra_rib_length",
            "prime_tower_rib_width",
            "prime_tower_fillet_wall",
            "extruder_clearance_max_radius",
            "machine_switch_extruder_time",
            "wall_direction",
            "is_ixex",
            "ixex_carriage_width_x",
            "ixex_carriage_width_y",
            "ixex_primary_col",
            "ixex_tool_layout",
            "ixex_",
            "ixex",
            "acceleration",
            "scale",
            "rotate",
            "duplicate",
            "duplicate_grid",
            "bed_size",
            "print_center",
            "g0",
            "wipe_tower_per_color_wipe",
            "support_sharp_tails",
            "max_volumetric_speed",
            "max_print_speed",
            "silent_mode",
            "anisotropic_surfaces",
            "belt_slice_rotation_global",
            "preslice_remap_x",
            "first_layer_plane_thickness",
            "z_hop_type",
            "nonexistent_key_xyzzy",
            "",
        ] {
            keys.push(k.to_string());
        }
        let values = [
            "",
            "0",
            "1",
            "2",
            "-1",
            "true",
            "TRUE",
            "false",
            "False",
            "none",
            "auto",
            "normal",
            "tree",
            "hybrid(auto)",
            "limited",
            "zig-zag",
            "5%",
            "50%",
            "Auto",
            "SuperTack Plate",
            "inner wall/outer wall/infill",
            "outer wall/inner wall/infill",
            "inner-outer-inner wall/infill",
            "infill/inner wall/outer wall",
            "something else",
            "Normal;Big Traffic",
            "DirectDrive",
            "\"ASA-Aero\";\"PLA\"",
            "\"PLA\"",
            "-30",
            "enable_wipe_tower;cooling;acceleration",
            "\"enable_wipe_tower\";\"wall_infill_order\"",
        ];

        let registry = &print_config_def().def.options;
        let mut checked = 0usize;
        let mut shortfall = 0usize;
        let mut mismatches: Vec<String> = Vec::new();
        for k in &keys {
            for v in values {
                let o = ora::oracle_handle_legacy(k, v);
                let mut mk = k.clone();
                let mut mv = v.to_string();
                PrintConfigDef::handle_legacy(&mut mk, &mut mv);
                if !o.opt_key.is_empty() && !registry.contains_key(o.opt_key.as_str()) {
                    // The oracle kept a key this registry has not transcribed yet: the two
                    // sides must differ here, by the vendored body's own final `has()` check.
                    shortfall += 1;
                    continue;
                }
                checked += 1;
                if mk != o.opt_key || mv != o.value {
                    mismatches.push(format!(
                        "key={k:?} value={v:?} -> mine=({mk:?},{mv:?}) oracle=({:?},{:?})",
                        o.opt_key, o.value
                    ));
                }
            }
        }
        println!(
            "handle_legacy: {checked} case(s) held to exact equality, \
             {shortfall} case(s) deferred to a key not yet transcribed"
        );
        assert!(
            mismatches.is_empty(),
            "handle_legacy divergences: {:#?}",
            &mismatches[..mismatches.len().min(20)]
        );
    }

    /// The `different_settings_to_system` branch's recursion and the two anonymous-namespace
    /// helpers it is built on, against the oracle on the one input shape that exercises all
    /// three at once.
    #[test]
    fn the_recursive_legacy_branch_renames_each_split_key() {
        // The three keys that branch renames to, each present in the oracle's registry.
        for renamed_to in [
            "enable_prime_tower",
            "slow_down_for_layer_cooling",
            "wall_sequence",
        ] {
            assert!(
                ora::oracle_option_info(renamed_to).found,
                "{renamed_to} must be a registry key at the pin"
            );
        }
        for (k, want) in [
            ("enable_wipe_tower", "enable_prime_tower"),
            ("cooling", "slow_down_for_layer_cooling"),
            ("wall_infill_order", "wall_sequence"),
        ] {
            let o = ora::oracle_handle_legacy(k, "");
            assert_eq!(o.opt_key, want, "the oracle's own rename of {k}");
            let mut mk = k.to_string();
            let mut mv = String::new();
            PrintConfigDef::handle_legacy(&mut mk, &mut mv);
            // Once `want` is transcribed this is the full equality; until then the ported
            // chain's own rename is still checked, by asking the helpers directly.
            if print_config_def().def.has(want) {
                assert_eq!(mk, want, "the ported rename of {k}");
            }
        }
        // `SplitStringAndRemoveDuplicateElement` and `ReplaceString` themselves.
        let split = split_string_and_remove_duplicate_element("b;a;b;", ";");
        assert_eq!(
            split.iter().cloned().collect::<Vec<String>>(),
            vec!["".to_string(), "a".to_string(), "b".to_string()],
            "deduplicated and in the std::set's own bytewise order"
        );
        let mut s = "Normal Normal".to_string();
        replace_string(&mut s, "Normal", "Standard");
        assert_eq!(s, "Standard Standard");
        let mut t = "aa".to_string();
        replace_string(&mut t, "a", "aa");
        assert_eq!(
            t, "aaaa",
            "advances past each replacement rather than rescanning"
        );
    }

    /// The four key lists `PrintConfigDef` keeps beside the registry
    /// (`init_extruder_option_keys`, `PrintConfig.cpp:8992-9053`;
    /// `init_filament_option_keys`, `:9055-9108`), held to exact equality against the real
    /// oracle's own accessors -- content *and* order, since both lists are handed out whole
    /// and iterated positionally.
    ///
    /// This covers the two filament lists' **emptiness** as well. `init_filament_option_keys`
    /// is never called at the pin, so `print_config_def.filament_option_keys()` is empty, and
    /// the port leaves its own two members empty for the same reason. That is a divergence a
    /// reader would otherwise have to take on the implementer's word; here it is the oracle
    /// that says so. The transcribed body is still checked, by calling it on a throwaway
    /// `PrintConfigDef` and comparing against the vendored braced initialiser's own content,
    /// so a mistranscribed key in a function the pin never calls still fails a test rather
    /// than sitting unread.
    #[test]
    fn extruder_and_filament_key_lists_match_the_oracle() {
        let def = print_config_def();

        assert_eq!(
            *def.extruder_option_keys(),
            ora::oracle_extruder_option_keys()
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>(),
            "extruder_option_keys, in the vendored vector's own order"
        );
        assert_eq!(
            *def.extruder_retract_keys(),
            ora::oracle_extruder_retract_keys()
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>(),
            "extruder_retract_keys, in the vendored vector's own order"
        );

        // `init_filament_option_keys` is uncalled at the pin, so both of these are empty on
        // the oracle and on the port. Asserted against the oracle rather than against a
        // literal `vec![]`, so this reads as agreement and not as an assumption.
        let oracle_filament_option = ora::oracle_filament_option_keys();
        let oracle_filament_retract = ora::oracle_filament_retract_keys();
        assert!(
            oracle_filament_option.is_empty() && oracle_filament_retract.is_empty(),
            "the pin never calls init_filament_option_keys, so both filament lists are empty \
             on print_config_def; got {} and {} entries",
            oracle_filament_option.len(),
            oracle_filament_retract.len()
        );
        assert!(
            def.filament_option_keys().is_empty() && def.filament_retract_keys().is_empty(),
            "the port must leave them empty too, i.e. must not call \
             init_filament_option_keys from PrintConfigDef::new()"
        );

        // The uncalled function's own body, checked against the vendored braced initialiser.
        let mut throwaway = PrintConfigDef::new();
        throwaway.init_filament_option_keys();
        assert_eq!(
            throwaway.filament_option_keys().len(),
            26,
            "27 braced entries at PrintConfig.cpp:9057-9084 less the one commented out at :9063"
        );
        assert_eq!(
            throwaway.filament_option_keys()[0],
            "default_filament_profile"
        );
        assert_eq!(
            *throwaway.filament_option_keys().last().unwrap(),
            "z_hop_types"
        );
        assert!(
            !throwaway
                .filament_option_keys()
                .contains(&"filament_seam_gap".to_string()),
            "the key commented out at PrintConfig.cpp:9063 must stay absent"
        );
        assert_eq!(throwaway.filament_retract_keys().len(), 18);
        assert!(
            throwaway.filament_retract_keys().is_sorted(),
            "the predicate PrintConfig.cpp:9107's own assert states"
        );
        // Every filament-retract key is a filament-option key, and every extruder-retract key
        // an extruder-option key -- true of the vendored lists and the property the
        // "overridden by the same values defined at the Filament level" comment
        // (`PrintConfig.hpp:853-855`) relies on.
        for k in throwaway.filament_retract_keys() {
            assert!(
                throwaway.filament_option_keys().contains(k),
                "{k} is a filament retract key and not a filament option key"
            );
        }
        for k in def.extruder_retract_keys() {
            assert!(
                def.extruder_option_keys().contains(k),
                "{k} is an extruder retract key and not an extruder option key"
            );
        }
    }

    /// One oracle or ported `ConfigOptionDef` rendered as the lines this unit's proof compares:
    /// `<field>=<value>` for every field of `ConfigOptionDef` (`Config.hpp:2315-2637`) that the
    /// registry's `init_*` bodies can set, in one fixed order. Less `opt_key`, which is the key
    /// this row is filed under, and less `serialization_key_ordinal`, which `slic3r-config`
    /// draws from a process-wide atomic counter rather than from the registry's own
    /// construction order and which is therefore not a property of the transcription (named as
    /// a finding in this task's pull request body).
    ///
    /// Rendering to text rather than comparing field by field is what lets one `compare_unit`
    /// call cover the whole table: `slic3r_proof::ProofValue` is implemented for the numeric
    /// primitives and for `Vec<T>`, and its own doc directs a caller with a structured value to
    /// "flatten it to a `Vec<i64>` (or another primitive) at the call site". The flattening is
    /// this text's bytes, so the comparison is bytewise equality of the two registries.
    fn render_def(fields: &[(&str, String)]) -> Vec<String> {
        fields
            .iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect()
    }

    /// The ported side of one row.
    fn ported_row(d: &slic3r_config::config::ConfigOptionDef) -> Vec<String> {
        render_def(&[
            ("type", (d.option_type as i32).to_string()),
            ("nullable", d.nullable.to_string()),
            (
                "default",
                match d.default_value.as_ref() {
                    Some(v) => format!("some:{}", v.serialize()),
                    None => "none".to_string(),
                },
            ),
            ("gui_type", (d.gui_type as i32).to_string()),
            ("gui_flags", d.gui_flags.clone()),
            ("plugin_type", d.plugin_type.clone()),
            ("label", d.label.clone()),
            ("full_label", d.full_label.clone()),
            (
                "printer_technology",
                (d.printer_technology as i32).to_string(),
            ),
            ("category", d.category.clone()),
            ("tooltip", d.tooltip.clone()),
            ("sidetext", d.sidetext.clone()),
            ("cli", d.cli.clone()),
            ("cli_params", d.cli_params.clone()),
            ("ratio_over", d.ratio_over.clone()),
            ("multiline", d.multiline.to_string()),
            ("full_width", d.full_width.to_string()),
            ("is_code", d.is_code.to_string()),
            ("readonly", d.readonly.to_string()),
            ("height", d.height.to_string()),
            ("width", d.width.to_string()),
            ("min", format!("{:?}", d.min)),
            ("max", format!("{:?}", d.max)),
            ("max_literal", format!("{:?}", d.max_literal)),
            ("mode", (d.mode as i32).to_string()),
            ("aliases", d.aliases.join("\u{1}")),
            ("shortcut", d.shortcut.join("\u{1}")),
            ("enum_values", d.enum_values.join("\u{1}")),
            ("enum_labels", d.enum_labels.join("\u{1}")),
            (
                "enum_keys_map",
                d.enum_keys_map
                    .iter()
                    .map(|(k, v)| format!("{k}:{v}"))
                    .collect::<Vec<_>>()
                    .join("\u{1}"),
            ),
        ])
    }

    /// The oracle side of the same row, from the same fields in the same order.
    fn oracle_row(info: &ora::OracleOptionInfo) -> Vec<String> {
        render_def(&[
            ("type", info.option_type.to_string()),
            ("nullable", info.nullable.to_string()),
            (
                "default",
                if info.has_default {
                    format!("some:{}", info.default_serialized)
                } else {
                    "none".to_string()
                },
            ),
            ("gui_type", info.gui_type.to_string()),
            ("gui_flags", info.gui_flags.to_string()),
            ("plugin_type", info.plugin_type.to_string()),
            ("label", info.label.to_string()),
            ("full_label", info.full_label.to_string()),
            ("printer_technology", info.printer_technology.to_string()),
            ("category", info.category.to_string()),
            ("tooltip", info.tooltip.to_string()),
            ("sidetext", info.sidetext.to_string()),
            ("cli", info.cli.to_string()),
            ("cli_params", info.cli_params.to_string()),
            ("ratio_over", info.ratio_over.to_string()),
            ("multiline", info.multiline.to_string()),
            ("full_width", info.full_width.to_string()),
            ("is_code", info.is_code.to_string()),
            ("readonly", info.readonly.to_string()),
            ("height", info.height.to_string()),
            ("width", info.width.to_string()),
            ("min", format!("{:?}", info.min)),
            ("max", format!("{:?}", info.max)),
            ("max_literal", format!("{:?}", info.max_literal)),
            ("mode", info.mode.to_string()),
            (
                "aliases",
                info.aliases
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
                    .join("\u{1}"),
            ),
            (
                "shortcut",
                info.shortcut
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
                    .join("\u{1}"),
            ),
            (
                "enum_values",
                info.enum_values
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
                    .join("\u{1}"),
            ),
            (
                "enum_labels",
                info.enum_labels
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
                    .join("\u{1}"),
            ),
            (
                "enum_keys_map",
                info.enum_keys_map
                    .iter()
                    .map(|e| format!("{}:{}", e.name, e.value))
                    .collect::<Vec<_>>()
                    .join("\u{1}"),
            ),
        ])
    }

    /// **This unit's R19 proof.** `PrintConfig.cpp` and `PrintConfig.hpp` are the two manifest
    /// rows (442 and 443) that share `crates/slic3r-printconfig/src/print_config.rs`, and both
    /// declare `proof path: 3` and `equality tier: exact` with the `tolerance` and
    /// `comparison function` cells empty. One test carrying one `// PROOF: 3` marker therefore
    /// satisfies `port/port-status`'s `proof_present` for each row independently -- it matches
    /// the marker's text against each row's own cell, and both cells hold the same text. Both
    /// `compare_unit` calls are made, one per row, so each row's own tier is exercised rather
    /// than assumed from the other's.
    ///
    /// **What is held to exact equality, and why it is the whole unit's output.** This unit
    /// produces no integer coordinates, so `task.md`'s equality rule applies in its second
    /// form: "of the unit's own output where it produces none". The unit's own output is the
    /// registry `print_config_def` -- 881 rows, each a full `ConfigOptionDef` -- plus the four
    /// key lists beside it and `handle_legacy`'s rewriting. All three are compared bytewise
    /// against the library built from the pin: the registry here, the key lists in
    /// `extruder_and_filament_key_lists_match_the_oracle`, `handle_legacy` in
    /// `handle_legacy_matches_the_oracle`, and the enum key maps in
    /// `transcribed_enum_key_maps_match_the_oracles`. No tolerance is chosen and none is
    /// representable: `compare_unit` reads the row's `exact` tier and its empty `tolerance`
    /// cell out of `port/manifest.tsv` itself and refuses anything else.
    ///
    /// The assertions run in a deliberate order, so a failure says what went wrong rather than
    /// printing a 881-row byte diff: the key sets first (a row absent from either side), then
    /// the per-row field comparison (a mistranscribed field, named with the key and both
    /// values), then the whole-table `compare_unit` that is the proof of record. The first two
    /// cannot pass while the third fails -- they compare the same text -- so they are
    /// diagnostics on the same equality, not a weaker check standing in for it.
    // PROOF: 3
    #[test]
    fn print_config_def_matches_the_oracle_exactly() {
        let oracle_keys: Vec<String> = ora::oracle_option_keys()
            .iter()
            .map(|s| s.to_string())
            .collect();
        let ported = &print_config_def().def.options;
        let ported_keys: Vec<String> = ported.keys().cloned().collect();

        // Direction 1: a key the oracle has and this registry does not.
        let missing: Vec<&String> = oracle_keys
            .iter()
            .filter(|k| !ported.contains_key(*k))
            .collect();
        assert!(
            missing.is_empty(),
            "{} oracle keys are absent from the ported registry: {missing:#?}",
            missing.len()
        );

        // Direction 2: a key this registry has and the oracle does not -- a key invented here.
        let extra: Vec<&String> = ported_keys
            .iter()
            .filter(|k| !oracle_keys.contains(k))
            .collect();
        assert!(
            extra.is_empty(),
            "{} ported keys are absent from the oracle's registry: {extra:#?}",
            extra.len()
        );

        // FIDELITY(port): both sides are walked in the key order of the vendored
        // `ConfigDef::options`, a `std::map<std::string, ConfigOptionDef>`
        // (`Config.hpp:2658`), which is bytewise-ascending by key; `TOptiondefMap` is a
        // `BTreeMap<String, _>`, whose iteration order is the same. The order reaches the
        // compared output, because the two flattened tables are zipped positionally as well as
        // matched by key.
        assert_eq!(
            ported_keys, oracle_keys,
            "the two registries' key orders differ"
        );

        // Direction 3: every field of every row.
        let mut ported_table: Vec<String> = Vec::new();
        let mut oracle_table: Vec<String> = Vec::new();
        let mut differences: Vec<String> = Vec::new();
        for key in &oracle_keys {
            let info = ora::oracle_option_info(key);
            assert!(info.found, "{key} vanished from the oracle between calls");
            let mine = ported_row(&ported[key]);
            let theirs = oracle_row(&info);
            for (m, t) in mine.iter().zip(theirs.iter()) {
                if m != t {
                    differences.push(format!("{key}: ported {m:?} != oracle {t:?}"));
                }
            }
            ported_table.push(format!("{key}\u{2}{}", mine.join("\u{3}")));
            oracle_table.push(format!("{key}\u{2}{}", theirs.join("\u{3}")));
        }
        assert!(
            differences.is_empty(),
            "{} field(s) disagree with the oracle: {differences:#?}",
            differences.len()
        );

        // The proof of record: the whole table, bytewise, through the harness, once per
        // manifest row so each row's own `exact` tier is read and enforced.
        let ported_bytes: Vec<i64> = ported_table
            .join("\u{4}")
            .bytes()
            .map(|b| i64::from(b))
            .collect();
        let oracle_bytes: Vec<i64> = oracle_table
            .join("\u{4}")
            .bytes()
            .map(|b| i64::from(b))
            .collect();
        slic3r_proof::tiers::compare_unit(
            "PrintConfig.cpp",
            slic3r_proof::tiers::Oracle::Built,
            ported_bytes.clone(),
            oracle_bytes.clone(),
        )
        .expect("PrintConfig.cpp: print_config_def differs from the oracle's");
        slic3r_proof::tiers::compare_unit(
            "PrintConfig.hpp",
            slic3r_proof::tiers::Oracle::Built,
            ported_bytes,
            oracle_bytes,
        )
        .expect("PrintConfig.hpp: print_config_def differs from the oracle's");

        println!(
            "registry proof: {} of {} oracle keys transcribed, every field of every row exact",
            ported_keys.len(),
            oracle_keys.len()
        );
    }
}

// PORT STATUS
// source: src/libslic3r/PrintConfig.cpp
// source: src/libslic3r/PrintConfig.hpp
// confidence: low
// tier: exact
// oracle: built
