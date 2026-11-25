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
use std::path::Path;

use crate::resolved_config::ResolvedConfig;

/// Which of the five calls `Surface` has reached. Tracked so an out-of-order call is caught
/// mechanically (`require`) rather than relying on every caller reading this module's doc
/// comment. A stage advances as soon as its call's ordering precondition holds, even though the
/// call then returns [`SurfaceError::NotYetPorted`] — nothing underneath actually runs yet, but
/// the *position* in the sequence is real, which is what the next call's precondition reads.
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
}

// A seventh public item the task's fixed shape (the five methods and `new`) does not name:
// `clippy::new_without_default` fires without it (verified by removing it and re-running
// `cargo clippy -p grampus-cli --all-targets`). It delegates to `new` and adds no behavior of
// its own.
impl Default for Surface {
    fn default() -> Self {
        Self::new()
    }
}

impl Surface {
    pub fn new() -> Self {
        Surface {
            stage: Cell::new(Stage::Fresh),
        }
    }

    /// Builds the model: adds an object from `mesh_path`, adds its volume and adds an instance
    /// (`init_print`, `test_helpers.cpp:260-273`), then runs the two per-object calls
    /// `init_print` makes next — `ensure_on_bed()` and `auto_assign_extruders()`
    /// (`:279-280`) — which belong here rather than being dropped (see this module's doc
    /// comment). Not yet ported: nothing under `crates/slic3r-*/` exists yet.
    pub fn build_model(&mut self, mesh_path: &Path) -> Result<(), SurfaceError> {
        self.require(Stage::Fresh, "build_model", "new")?;
        let _ = mesh_path;
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
        self.stage.set(Stage::AppliedConfig);
        Err(SurfaceError::NotYetPorted("apply_config"))
    }

    /// Validates the applied print (`print.validate()`, `test_helpers.cpp:284`). Takes `&self`,
    /// like the struct shape tasks 39 to 43 are written against fixes: validation does not need
    /// to mutate the surface. Not yet ported.
    pub fn validate(&self) -> Result<(), SurfaceError> {
        self.require(Stage::AppliedConfig, "validate", "apply_config")?;
        self.stage.set(Stage::Validated);
        Err(SurfaceError::NotYetPorted("validate"))
    }

    /// Processes the print (`print.process()`, `test_helpers.cpp:352`). Not yet ported.
    pub fn process(&mut self) -> Result<(), SurfaceError> {
        self.require(Stage::Validated, "process", "validate")?;
        self.stage.set(Stage::Processed);
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
