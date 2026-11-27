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
use slic3r_config::config::ConfigOptionEnum;
use slic3r_config::config::ConfigOptionEnumGeneric;
use slic3r_config::config::ConfigOptionFloat;
use slic3r_config::config::ConfigOptionFloatOrPercent;
use slic3r_config::config::ConfigOptionFloatsNullable;
use slic3r_config::config::ConfigOptionInt;
use slic3r_config::config::ConfigOptionInts;
use slic3r_config::config::ConfigOptionPercent;
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
        // `const int max_temp = 1500` (`PrintConfig.cpp:1322`) is a local of the vendored body,
        // used as the `max` of every nozzle-temperature option. It lands with the first of those
        // options; declaring it now would be a binding nothing reads.

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
        // <<<STEP5-FFF>>>
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
        // FIDELITY(port): braced-initialiser order, as in `init_extruder_option_keys`.
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
        // <<<STEP5-SLA>>>
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
            if d.cli != info.cli {
                why.push(format!("cli mine={:?} oracle={:?}", d.cli, info.cli));
            }
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
}

// PORT STATUS
// source: src/libslic3r/PrintConfig.cpp
// source: src/libslic3r/PrintConfig.hpp
// confidence: low
// tier: exact
// oracle: built
