//! `PrintConfig.cpp`/`PrintConfig.hpp` (`port/manifest.tsv`'s two rows for this module, task
//! `64-foundation-print-config`, `port/foundation.tsv`'s `foundation-13`): the print
//! configuration option set.
//!
//! **Status: incomplete, and deliberately not reached by a `mod` line.** This module is being
//! built in the order
//! `intent/libslic3r-rust-port/tasks/64-foundation-print-config/plan.md` sets out (Step 2 the
//! enums, Step 3 `StaticCache`/`StaticPrintConfig`, Step 4 the 11 derived classes, Step 5 the
//! option table, Step 6 `handle_legacy`). Until every step is in and the whole-key-set
//! comparison test is green over all 881 oracle keys, `crates/slic3r-printconfig/src/lib.rs`
//! carries **no** `mod print_config;` line, so `port/port-status` derives `drafted` for this
//! unit rather than a `ported` status that is not true. `docs/WORKFLOW.md`'s rule is explicit:
//! an unreached module is `drafted`, a `mod`-reached module over a live `todo!()` is
//! `not-started`, and the first is the honest state of partial work. Which slices are in is
//! recorded in this task's pull request body, measured rather than estimated.
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
use slic3r_config::config::ConfigOptionEnumGeneric;
use slic3r_config::config::ConfigOptionFloat;
use slic3r_config::config::ConfigOptionFloatOrPercent;
use slic3r_config::config::ConfigOptionFloats;
use slic3r_config::config::ConfigOptionFloatsNullable;
use slic3r_config::config::ConfigOptionInt;
use slic3r_config::config::ConfigOptionInts;
use slic3r_config::config::ConfigOptionPercent;
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
        // <<<STEP5-FFF>>>
    }

    /// `PrintConfigDef::init_extruder_option_keys()` (`PrintConfig.cpp:8992-9054`).
    fn init_extruder_option_keys(&mut self) {
        // <<<STEP5-EXTRUDER>>>
    }

    /// `PrintConfigDef::init_filament_option_keys()` (`PrintConfig.cpp:9055-9109`).
    #[allow(dead_code)]
    fn init_filament_option_keys(&mut self) {
        // <<<STEP5-FILAMENT>>>
    }

    /// `PrintConfigDef::init_sla_params()` (`PrintConfig.cpp:9110-9747`).
    fn init_sla_params(&mut self) {
        // <<<STEP5-SLA>>>
    }
}

// <<<APPEND>>>

// PORT STATUS
// source: src/libslic3r/PrintConfig.cpp
// source: src/libslic3r/PrintConfig.hpp
// confidence: low
// tier: exact
// oracle: built
