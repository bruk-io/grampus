//! The port's GUI-free slicing surface (R10): one struct, `Surface`, with five inherent methods
//! in a fixed order, mirroring the sequence
//! `vendor/OrcaSlicer/tests/fff_print/test_helpers.cpp` uses — `init_print` (`:253-282`, which
//! adds an object, adds its volume and adds an instance), `print.apply(model, config)`
//! (`:283`), `print.validate()` (`:284`), `print.process()` (`:352`) and
//! `print.export_gcode(temp.string(), result, nullptr)` (`:353`). `init_print`'s own two
//! per-object calls between building the model and applying the config —
//! `ModelObject::ensure_on_bed()` (`:279`) and `Print::auto_assign_extruders()` (`:280`) —
//! belong to [`Surface::build_model`] rather than being dropped, so the port's sequence matches
//! the oracle's; for a committed fixture `ensure_on_bed` is the identity (R24).
//!
//! **No arranger.** `init_print`'s own `arrange` parameter
//! (`vendor/OrcaSlicer/tests/fff_print/test_helpers.hpp:88`, default `true`) is not part of this
//! surface, and the auto-placement call it guards at `test_helpers.cpp:277` is not reachable
//! from any of the five methods below. R24 fixes placement in the fixture and every oracle
//! invocation passes `--arrange 0`, so no auto-placement and no orientation step runs on either
//! side of any comparison (C14's decision leaves that out-of-scope code to the following
//! intent).
//!
//! **Nothing from the plate layer.** No type, function or constant from the GUI's plate module
//! under `vendor/OrcaSlicer/src/slic3r/GUI/` is referenced or reimplemented anywhere in this
//! crate (checked by a literal grep over `crates/` for that module's own name, which this
//! comment is deliberately written to avoid spelling out).
//!
//! **No preset resolution.** `apply_config` takes a [`ResolvedConfig`](crate::resolved_config),
//! not a preset bundle, so nothing reachable from here reads
//! `vendor/OrcaSlicer/resources/profiles/`, a preset file or an `inherits` chain (R11).
//!
//! **The Bambu-printer flag** is carried on the `ResolvedConfig` the caller builds and passes to
//! `apply_config` — set explicitly by the caller (see
//! `crate::resolved_config::ResolvedConfig::set_bambu_printer`), never derived here from the
//! resolved configuration's own JSON object, because the oracle itself derives it from the
//! printer's vendor preset rather than from resolved settings
//! (`vendor/OrcaSlicer/src/OrcaSlicer.cpp:7026`).
//!
//! **Nothing under `crates/slic3r-*/` is ported yet**, so every method below returns
//! [`SurfaceError::NotYetPorted`] once its ordering precondition holds — never a stub-macro
//! panic of the two kinds `just port-status`'s burn-down counts. The precondition itself
//! *is* enforced now: `Surface` tracks which of the five calls has been reached in its private
//! `stage`, and a call made before the one before it reports
//! [`SurfaceError::OutOfOrder`] instead, before anything about "not yet ported" is reached.
//! This is what makes the five-call order a mechanical check (`tests/surface.rs`) rather than a
//! matter of reading this module.

use std::cell::Cell;
use std::fmt;
use std::path::{Path, PathBuf};

use crate::resolved_config::ResolvedConfig;

/// The R5 per-object and print-wide seams, in `Print.hpp`'s `SlicingPipelineStepPlugin` enum's
/// own **declaration** order (`:133-134`) — named here because task `39-compare-stages` asks
/// this module for "the emission call sites ... at the same eleven seams" — unlike
/// `crates/oracle-compare/src/stages.rs`, which reads the stage list from `port/manifest.tsv`
/// and names none of them, this module *is* where the eleven seams are named, the same way
/// `dump_seams.cpp`'s own `seam_name` names them.
///
/// **This is not the firing order.** `Print::process()` (`Print.cpp:3300`) calls
/// `posSimplifyPath` *last* of the eleven, after `psSkirtBrim` — not ninth, where it sits in
/// this list and in the enum's own declaration. `psSkirtBrim` is marked done at
/// `Print.cpp:3958`, inside `Print::process()`'s main per-object/per-print sequence; the
/// `simplify_extrusion_path()` loop that marks `posSimplifyPath` done (`Print.cpp:3971,
/// 3979-3980`) runs afterwards, as a separate pass over every object. A reader placing a real
/// emission call site "at" `posSimplifyPath` by this list's position would put it in the wrong
/// place; place it after the `psSkirtBrim` call site instead.
pub const STAGE_DUMP_SEAMS: [&str; 11] = [
    "posSlice",
    "posPerimeters",
    "posPrepareInfill",
    "posInfill",
    "posIroning",
    "posContouring",
    "posSupportMaterial",
    "posDetectOverhangsForLift",
    "posSimplifyPath",
    "psWipeTower",
    "psSkirtBrim",
];

/// Which of the five calls `Surface` has reached. Tracked so an out-of-order call is caught
/// mechanically (`require`) rather than relying on every caller reading this module's doc
/// comment.
///
/// PHASE B (plan step 23): the task's own contract is that `OutOfOrder` fires when a call is
/// made before the one before it **has succeeded**, but today every call always returns
/// `Err` (nothing underneath is ported), so gating the advance on `Ok` would make `apply_config`
/// through `export_gcode` all report `OutOfOrder` even when called strictly in order — which
/// would fail this module's own "in order ⇒ `NotYetPorted`, not `OutOfOrder`" test. Until a real
/// implementation can succeed, `stage` advances as soon as a call's ordering precondition holds
/// (advance-on-attempt), not as soon as the call succeeds (advance-on-success). The first commit
/// that gives one of these five methods a real `Ok` body must invert this: move each `stage.set`
/// below onto that method's success path, so a failed call no longer unlocks the next one.
///
/// A second consequence of the same stand-in: `require` compares `stage` for strict equality, so
/// `Surface` is single-use per call — a second `build_model`, a repeated `validate`, and so on
/// all report `OutOfOrder` naming the *first* call's predecessor rather than being idempotent.
/// That is intentional for now (each of the five calls is meant to run exactly once per
/// `Surface`), not an oversight.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    Fresh,
    BuiltModel,
    AppliedConfig,
    Validated,
    Processed,
}

/// The two variants that matter before Phase B (plan step 23) lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceError {
    /// Every one of the five calls returns this once its ordering precondition holds, until
    /// Phase B replaces this module's body with a real port. Carries the method's name, which
    /// `src/main.rs` prints on exit rather than panicking.
    NotYetPorted(&'static str),
    /// A call was made before the one before it in the five-call sequence had been reached.
    /// Carries the method that was called and the method it required to have gone first.
    OutOfOrder {
        called: &'static str,
        expected_after: &'static str,
    },
}

impl fmt::Display for SurfaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SurfaceError::NotYetPorted(call) => {
                write!(f, "not yet ported: Surface::{call}")
            }
            SurfaceError::OutOfOrder {
                called,
                expected_after,
            } => write!(
                f,
                "Surface::{called} called out of order: {expected_after} must go first"
            ),
        }
    }
}

impl std::error::Error for SurfaceError {}

/// The GUI-free slicing surface: build a model, apply a config, validate, process, export
/// G-code. See this module's doc comment for what each call carries and what it deliberately
/// does not expose.
pub struct Surface {
    stage: Cell<Stage>,
    /// Set by [`Surface::set_dump_stages_dir`] (`grampus-cli`'s `--dump-stages <dir>`, task
    /// `39-compare-stages`). `None` by default, so a caller that never asks for stage dumps (the
    /// five-call tests, every binary invocation before this task) sees no change in behaviour.
    dump_stages_dir: Option<PathBuf>,
}

impl Surface {
    // `clippy::new_without_default` fires on a public `new` with no arguments unless either a
    // `Default` impl exists or the lint is silenced; the task's fixed shape is the five methods
    // and `new` and nothing else, so this silences the lint rather than adding a seventh public
    // item.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Surface {
            stage: Cell::new(Stage::Fresh),
            dump_stages_dir: None,
        }
    }

    /// Enables per-seam stage-dump emission: [`Surface::process`] attempts to write the R5 dump
    /// format (`oracle/stage-dump/dump_seams.cpp`'s record shape) to `dir` at each of
    /// [`STAGE_DUMP_SEAMS`] as it reaches them. Does not change the five-call order or any
    /// method's return value — only `process`'s internal emission call sites read this field.
    pub fn set_dump_stages_dir(&mut self, dir: PathBuf) {
        self.dump_stages_dir = Some(dir);
    }

    /// Builds the model: adds an object from `mesh_path`, adds its volume and adds an instance
    /// (`init_print`, `test_helpers.cpp:260-273`), then runs the two per-object calls
    /// `init_print` makes next — `ensure_on_bed()` and `auto_assign_extruders()`
    /// (`:279-280`) — which belong here rather than being dropped (see this module's doc
    /// comment). Not yet ported: nothing under `crates/slic3r-*/` is ported yet.
    pub fn build_model(&mut self, mesh_path: &Path) -> Result<(), SurfaceError> {
        self.require(Stage::Fresh, "build_model", "new")?;
        let _ = mesh_path;
        // PHASE B: advance only on Ok (see this module's `Stage` doc comment).
        self.stage.set(Stage::BuiltModel);
        Err(SurfaceError::NotYetPorted("build_model"))
    }

    /// Applies a resolved configuration to the built model (`print.apply(model, config)`,
    /// `test_helpers.cpp:283`). `config` is the oracle's resolved configuration (R11), never a
    /// preset bundle; the Bambu-printer flag it may carry is read from `config`, never derived
    /// here (see this module's doc comment). Not yet ported.
    pub fn apply_config(&mut self, config: &ResolvedConfig) -> Result<(), SurfaceError> {
        self.require(Stage::BuiltModel, "apply_config", "build_model")?;
        let _ = config;
        // PHASE B: advance only on Ok (see this module's `Stage` doc comment).
        self.stage.set(Stage::AppliedConfig);
        Err(SurfaceError::NotYetPorted("apply_config"))
    }

    /// Validates the applied print (`print.validate()`, `test_helpers.cpp:284`). Takes `&self`,
    /// like the struct shape tasks 39 to 43 are written against fixes: validation does not need
    /// to mutate the surface. Not yet ported.
    pub fn validate(&self) -> Result<(), SurfaceError> {
        self.require(Stage::AppliedConfig, "validate", "apply_config")?;
        // PHASE B: advance only on Ok (see this module's `Stage` doc comment).
        self.stage.set(Stage::Validated);
        Err(SurfaceError::NotYetPorted("validate"))
    }

    /// Processes the print (`print.process()`, `test_helpers.cpp:352`). The R5 seams fire during
    /// this call on the oracle's side (`Print::process()`), which is why the eleven
    /// `emit_stage_dump` call sites below live here and nowhere else in this module. Not yet
    /// ported.
    pub fn process(&mut self) -> Result<(), SurfaceError> {
        self.require(Stage::Validated, "process", "validate")?;
        // PHASE B: advance only on Ok (see this module's `Stage` doc comment).
        self.stage.set(Stage::Processed);
        // The eleven R5 seams, in `STAGE_DUMP_SEAMS`'s own pipeline order. Each call site is
        // placed where the corresponding seam would fire once this method's body is a real
        // port rather than an immediate `NotYetPorted` (below): nothing under `crates/slic3r-*/`
        // is ported yet, so `emit_stage_dump` has no per-object/per-layer state to serialise at
        // any of them today and every call is a documented no-op (see its own doc comment) —
        // this is what task `39-compare-stages`'s "a seam whose modules are not ported yet
        // writes nothing" means in this module. This also means `--dump-stages` changes nothing
        // about this method's control flow or return value: the slicing sequence and the
        // eventual exported G-code are identical with and without a dump directory set.
        for seam in STAGE_DUMP_SEAMS {
            self.emit_stage_dump(seam);
        }
        Err(SurfaceError::NotYetPorted("process"))
    }

    /// Exports G-code to `out` (`print.export_gcode(temp.string(), result, nullptr)`,
    /// `test_helpers.cpp:353`). Takes `&self` for the same reason `validate` does. Not yet
    /// ported.
    pub fn export_gcode(&self, out: &Path) -> Result<(), SurfaceError> {
        self.require(Stage::Processed, "export_gcode", "process")?;
        let _ = out;
        Err(SurfaceError::NotYetPorted("export_gcode"))
    }

    /// The one emission call site every R5 seam in [`STAGE_DUMP_SEAMS`] goes through. A no-op
    /// when [`Surface::set_dump_stages_dir`] was never called (the common case today — no
    /// binary invocation passes `--dump-stages` unless `just compare-stages` is asking for one).
    /// With a dump directory set, this is still a no-op today: there is no per-object/per-layer
    /// state anywhere in this crate to serialise at `seam` (nothing under `crates/slic3r-*/` is
    /// ported), so writing the R5 record format here would be inventing data rather than
    /// reporting it — task `39-compare-stages`'s own rule ("A seam whose modules are not ported
    /// yet writes nothing"). The first Phase B batch that ports a module `seam` depends on is
    /// what turns this into the one place that actually writes
    /// `oracle/stage-dump/dump_seams.cpp`'s record shape under `dir`.
    fn emit_stage_dump(&self, seam: &'static str) {
        let Some(_dir) = self.dump_stages_dir.as_ref() else {
            return;
        };
        let _ = seam;
    }

    /// The mechanical order check: `called` may run only once `stage` has reached `expected`.
    /// `expected_after` is the method whose call is what advances `stage` to `expected` (purely
    /// for the error message; the check itself only compares `stage`).
    fn require(
        &self,
        expected: Stage,
        called: &'static str,
        expected_after: &'static str,
    ) -> Result<(), SurfaceError> {
        if self.stage.get() == expected {
            Ok(())
        } else {
            Err(SurfaceError::OutOfOrder {
                called,
                expected_after,
            })
        }
    }
}
