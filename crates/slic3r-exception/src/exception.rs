//! Ports `vendor/OrcaSlicer/src/libslic3r/Exception.hpp` (crate-level level 0, hand-designed by
//! plan step 20's task `52-foundation-common-defs`, `port/foundation.tsv` row `foundation-01`):
//! libslic3r's own exception hierarchy. No dependent among this task's own units and no
//! `port/ownership.tsv` row (no raw-pointer field at the pin).
//!
//! **Decision (task was silent): one flat enum, per `docs/PORTING.md`'s `### Error sets`.** That
//! rule states the mapping is "one Rust error enum ... whose variants mirror this hierarchy
//! one-for-one, so a `throw SlicingError(...)` site and the code that would have caught it keep
//! the same set of cases across the port" -- which requires `SlicingError`/`SlicingErrors`
//! (`Exception.hpp:31,42`, siblings of `CriticalException` directly under `Exception`) to be
//! variants of the *same* enum as the `CriticalException` branch, not a second type: a
//! `catch (Exception &)` site in the oracle catches every one of them, and only one enum lets a
//! port site recover that same combined set with a single `match`. [`is_critical_exception`],
//! [`is_runtime_error`], [`is_logic_error`] and [`is_io_error`] below are how a port call site
//! recovers each narrower `catch (X &)` set (`Exception.hpp:18-27`) without a second
//! representation of the hierarchy (an explicit parent/`is_a` relation over one flat enum,
//! rather than nested Rust enums, which cannot express "`IOError` catches `FileIOError` *or*
//! `HostNetworkError`" and "`CriticalException` catches `IOError`" as the same relation at two
//! levels without one level boxing the other).
//!
//! [`SlicingError`] is its own struct, not folded into a variant's bare fields, because
//! `SlicingErrors` holds a `Vec<SlicingError>` (`Exception.hpp:48`) of the exact same shape a
//! standalone `SlicingError` has -- one definition of that shape, reused both as
//! `Exception::SlicingError`'s payload and inside `Exception::SlicingErrors`'s `errors` field,
//! rather than `Vec<Exception>` (which would also, wrongly, accept every other variant).
//!
//! **Proof path 2 could not be honoured as a real proof (a finding for this pull request, not a
//! tolerance or a stand-in invented on this branch).** Exactly as
//! `crates/slic3r-commondefs/src/common_defs.rs`'s doc comment records for `CommonDefs.hpp`:
//! this row's `proof path` is seeded `2` from its `pipeline stage` cell (`posSlice`) alone, and
//! no field across the committed stage-dump corpus (`port/dumps/*/*.dump`) carries an exception
//! message, an object id or any other value this hierarchy produces -- the corpus's full field
//! set (`fill_surfaces, fills, height, lslices, object_skirt, perimeters, print_z, raw_slices,
//! skirt, skirt_brim_groups, slice_z, slices, thin_fills, wipe_tower_*`) holds none of it, and no
//! oracle run that reaches a compared dump throws (a thrown `SlicingError` aborts that slice
//! before any `posSlice` dump is written). This module therefore carries no `// PROOF:` marker;
//! `port/port-status` derives it as `drafted`, not `ported`.

/// `Slic3r::SlicingError` (`Exception.hpp:31-40`): its base's message plus the `objectId`
/// (`size_t` -> `usize`) the two-argument constructor adds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlicingError {
    pub message: String,
    pub object_id: usize,
}

impl SlicingError {
    /// `SlicingError(std::string const &msg)`, via the inherited `using Exception::Exception;`
    /// (`Exception.hpp:34`): `objectId_` keeps its in-class default, `0` (`:39`).
    pub fn new(message: impl Into<String>) -> Self {
        SlicingError {
            message: message.into(),
            object_id: 0,
        }
    }

    /// `SlicingError(std::string const &msg, size_t objectId)` (`Exception.hpp:35`).
    pub fn with_object_id(message: impl Into<String>, object_id: usize) -> Self {
        SlicingError {
            message: message.into(),
            object_id,
        }
    }

    /// `SlicingError::objectId()` (`Exception.hpp:36`).
    pub fn object_id(&self) -> usize {
        self.object_id
    }
}

/// `Slic3r::Exception` and its whole hierarchy (`Exception.hpp:9-49`), one variant per concrete
/// C++ type, each carrying the message every one of them stores (`std::runtime_error`'s own
/// payload) -- see this module's doc comment for why `SlicingError`/`SlicingErrors` are variants
/// here too, and for [`SlicingError`]'s own extra field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Exception {
    /// `Slic3r::Exception` itself (`:13`) -- constructed directly only where the oracle does not
    /// use one of its named subclasses.
    Exception(String),
    /// `CriticalException` (`:18`).
    CriticalException(String),
    /// `RuntimeError` (`:19`, under `CriticalException`).
    RuntimeError(String),
    /// `LogicError` (`:20`, under `CriticalException`).
    LogicError(String),
    /// `HardCrash` (`:21`, under `CriticalException`).
    HardCrash(String),
    /// `InvalidArgument` (`:22`, under `LogicError`).
    InvalidArgument(String),
    /// `OutOfRange` (`:23`, under `LogicError`).
    OutOfRange(String),
    /// `IOError` (`:24`, under `CriticalException`).
    IOError(String),
    /// `FileIOError` (`:25`, under `IOError`).
    FileIOError(String),
    /// `HostNetworkError` (`:26`, under `IOError`).
    HostNetworkError(String),
    /// `ExportError` (`:27`, under `CriticalException`).
    ExportError(String),
    /// `PlaceholderParserError` (`:28`, under `RuntimeError`).
    PlaceholderParserError(String),
    /// `SlicingError` (`:31-40`, directly under `Exception`, a sibling of `CriticalException`).
    SlicingError(SlicingError),
    /// `SlicingErrors` (`:42-49`, directly under `Exception`): `errors` defaults empty when
    /// constructed from a bare message (the inherited `using Exception::Exception;`, `:45`);
    /// the `std::vector<SlicingError>` constructor (`:46`) always sets `message` to the literal
    /// `"Errors"` (`:46`'s own `Exception("Errors")`), never a caller-supplied string.
    SlicingErrors {
        message: String,
        errors: Vec<SlicingError>,
    },
}

impl Exception {
    /// `std::runtime_error::what()`, inherited unchanged by every variant here (`Exception.hpp`
    /// adds no variant that overrides it).
    pub fn message(&self) -> &str {
        match self {
            Exception::Exception(m)
            | Exception::CriticalException(m)
            | Exception::RuntimeError(m)
            | Exception::LogicError(m)
            | Exception::HardCrash(m)
            | Exception::InvalidArgument(m)
            | Exception::OutOfRange(m)
            | Exception::IOError(m)
            | Exception::FileIOError(m)
            | Exception::HostNetworkError(m)
            | Exception::ExportError(m)
            | Exception::PlaceholderParserError(m) => m,
            Exception::SlicingError(e) => &e.message,
            Exception::SlicingErrors { message, .. } => message,
        }
    }

    /// Whether a `catch (CriticalException &)` (`Exception.hpp:18-27`'s whole subtree) would
    /// catch `self`. `SlicingError`/`SlicingErrors` derive from `Exception` directly (`:31,42`),
    /// not from `CriticalException`, so neither variant matches here.
    pub fn is_critical_exception(&self) -> bool {
        matches!(
            self,
            Exception::CriticalException(_)
                | Exception::RuntimeError(_)
                | Exception::LogicError(_)
                | Exception::HardCrash(_)
                | Exception::InvalidArgument(_)
                | Exception::OutOfRange(_)
                | Exception::IOError(_)
                | Exception::FileIOError(_)
                | Exception::HostNetworkError(_)
                | Exception::ExportError(_)
                | Exception::PlaceholderParserError(_)
        )
    }

    /// Whether a `catch (RuntimeError &)` (`:19,28`) would catch `self`.
    pub fn is_runtime_error(&self) -> bool {
        matches!(
            self,
            Exception::RuntimeError(_) | Exception::PlaceholderParserError(_)
        )
    }

    /// Whether a `catch (LogicError &)` (`:20,22-23`) would catch `self`.
    pub fn is_logic_error(&self) -> bool {
        matches!(
            self,
            Exception::LogicError(_) | Exception::InvalidArgument(_) | Exception::OutOfRange(_)
        )
    }

    /// Whether a `catch (IOError &)` (`:24-26`) would catch `self`.
    pub fn is_io_error(&self) -> bool {
        matches!(
            self,
            Exception::IOError(_) | Exception::FileIOError(_) | Exception::HostNetworkError(_)
        )
    }
}

impl std::fmt::Display for Exception {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for Exception {}

#[cfg(test)]
mod tests {
    use super::*;

    /// Not a `// PROOF:` marker (see this module's doc comment: no dump field holds this unit's
    /// output, so proof path 2 cannot be honoured here) -- a plain correctness check of the
    /// hierarchy's `catch`-set relations (`Exception.hpp:18-28`) and of `SlicingError`'s two
    /// constructors (`:34-35`).
    #[test]
    fn exception_catch_sets_match_the_vendored_hierarchy() {
        let hard_crash = Exception::HardCrash("critical".to_string());
        assert!(hard_crash.is_critical_exception());
        assert!(!hard_crash.is_runtime_error());
        assert!(!hard_crash.is_logic_error());
        assert!(!hard_crash.is_io_error());

        let placeholder = Exception::PlaceholderParserError("bad placeholder".to_string());
        assert!(placeholder.is_critical_exception());
        assert!(placeholder.is_runtime_error());
        assert!(!placeholder.is_logic_error());

        let invalid_argument = Exception::InvalidArgument("bad arg".to_string());
        assert!(invalid_argument.is_critical_exception());
        assert!(invalid_argument.is_logic_error());
        assert!(!invalid_argument.is_runtime_error());

        let file_io = Exception::FileIOError("could not open".to_string());
        assert!(file_io.is_critical_exception());
        assert!(file_io.is_io_error());

        // SlicingError/SlicingErrors derive from Exception directly, not from
        // CriticalException (Exception.hpp:31,42) -- neither is a CriticalException.
        let slicing_error = Exception::SlicingError(SlicingError::with_object_id("object 3", 3));
        assert!(!slicing_error.is_critical_exception());
        assert_eq!(slicing_error.message(), "object 3");

        let default_object_id = SlicingError::new("no object id given");
        assert_eq!(default_object_id.object_id(), 0);

        let errors = Exception::SlicingErrors {
            message: "Errors".to_string(),
            errors: vec![SlicingError::with_object_id("first", 1)],
        };
        assert_eq!(errors.message(), "Errors");
        assert!(!errors.is_critical_exception());
    }

    #[test]
    fn exception_display_is_the_message() {
        let e = Exception::RuntimeError("boom".to_string());
        assert_eq!(e.to_string(), "boom");
    }
}

// PORT STATUS
// source: src/libslic3r/Exception.hpp
// confidence: high
// tier: exact
// oracle: built
