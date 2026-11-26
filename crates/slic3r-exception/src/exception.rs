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
//! **Proof path 2, reached via `slic3r-proof`'s re-export of `slic3r-ffi`'s per-unit proof
//! bridges (`docs/PORTING.md`, "Decisions recorded on 2026-11-25", decisions 1 and 4).** Exactly
//! as `crates/slic3r-commondefs/src/common_defs.rs`'s doc comment records for `CommonDefs.hpp`:
//! this row's `proof path` is seeded `2` from its `pipeline stage` cell (`posSlice`) alone, and
//! no field across the committed stage-dump corpus carries an exception message, an object id or
//! any other value this hierarchy produces. Decision 4 names the per-unit proof bridges, not
//! only the dump corpus, as a "2" row's oracle entry point, so this unit's shim
//! (`crates/slic3r-ffi/cxx/proof_units/exception.{h,cpp}`,
//! `crates/slic3r-ffi/src/proof_units/exception.rs`) constructs each concrete vendored type
//! directly and reads its real, compiled `what()`/`objectId()` -- never throwing or catching a
//! C++ exception across the bridge, which `cxx` aborts on without the `Result` convention the
//! shim does not use.

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
    /// `SlicingErrors(std::string const &msg)`, via the inherited `using Exception::Exception;`
    /// (`:45`): `errors` keeps its in-class default, empty (`:48` has no initialiser, so
    /// `std::vector`'s own default-constructed, empty state).
    ///
    /// **Review finding, fixed here:** the adversarial review of this unit found that exposing
    /// `SlicingErrors { message, errors }`'s fields directly (both `pub`, as every other
    /// variant's payload is) lets a caller build a combination neither of the vendored class's
    /// two constructors can produce -- a non-`"Errors"` message paired with a non-empty
    /// `errors`, or vice versa. These two functions are the two real constructors; a caller
    /// building the enum variant literal directly can still violate the invariant, but every
    /// call site this task itself writes (its own test) now goes through one of these two.
    pub fn new_slicing_errors_with_message(message: impl Into<String>) -> Self {
        Exception::SlicingErrors {
            message: message.into(),
            errors: Vec::new(),
        }
    }

    /// `SlicingErrors(const std::vector<SlicingError> &errors)` (`:46`): `message` is always the
    /// literal `"Errors"` (`:46`'s own `Exception("Errors")`), never a caller-supplied string.
    pub fn new_slicing_errors(errors: Vec<SlicingError>) -> Self {
        Exception::SlicingErrors {
            message: "Errors".to_string(),
            errors,
        }
    }

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

    /// A plain correctness check of the hierarchy's `catch`-set relations (`Exception.hpp:18-28`)
    /// and of `SlicingError`'s two constructors (`:34-35`). Not a `// PROOF:` marker -- see
    /// `exception_what_and_object_id_match_the_oracle` below for that.
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

        let errors = Exception::new_slicing_errors(vec![SlicingError::with_object_id("first", 1)]);
        assert_eq!(errors.message(), "Errors");
        let errors_with_message = Exception::new_slicing_errors_with_message("custom message");
        assert_eq!(errors_with_message.message(), "custom message");
        assert!(!errors.is_critical_exception());
    }

    #[test]
    fn exception_display_is_the_message() {
        let e = Exception::RuntimeError("boom".to_string());
        assert_eq!(e.to_string(), "boom");
    }

    /// Every concrete type in the vendored hierarchy (`Exception.hpp:13-28`), constructed from
    /// the same message on both sides and compared by `what()`/`message()`, plus
    /// `SlicingError`'s `objectId()` (both its default and its two-argument form) and
    /// `SlicingErrors`' always-`"Errors"` message and element count. Each side is driven
    /// independently -- the oracle through `slic3r-proof`'s re-export of `slic3r-ffi`'s
    /// per-unit proof bridge, the port through this module's own types.
    // PROOF: 2
    #[test]
    fn exception_what_and_object_id_match_the_oracle() {
        use slic3r_proof::proof_units::exception::{
            path2_critical_exception_what, path2_exception_what, path2_export_error_what,
            path2_file_io_error_what, path2_hard_crash_what, path2_host_network_error_what,
            path2_invalid_argument_what, path2_io_error_what, path2_logic_error_what,
            path2_out_of_range_what, path2_placeholder_parser_error_what, path2_runtime_error_what,
            path2_slicing_error_default_object_id, path2_slicing_error_object_id,
            path2_slicing_error_what, path2_slicing_errors_from_message_what,
            path2_slicing_errors_from_vector_count, path2_slicing_errors_from_vector_what,
        };

        let msg = "a message this test chose";
        let cases: &[(&str, fn(String) -> Exception)] = &[
            ("Exception", Exception::Exception),
            ("CriticalException", Exception::CriticalException),
            ("RuntimeError", Exception::RuntimeError),
            ("LogicError", Exception::LogicError),
            ("HardCrash", Exception::HardCrash),
            ("InvalidArgument", Exception::InvalidArgument),
            ("OutOfRange", Exception::OutOfRange),
            ("IOError", Exception::IOError),
            ("FileIOError", Exception::FileIOError),
            ("HostNetworkError", Exception::HostNetworkError),
            ("ExportError", Exception::ExportError),
            ("PlaceholderParserError", Exception::PlaceholderParserError),
        ];
        let oracle_fns: &[fn(&str) -> String] = &[
            path2_exception_what,
            path2_critical_exception_what,
            path2_runtime_error_what,
            path2_logic_error_what,
            path2_hard_crash_what,
            path2_invalid_argument_what,
            path2_out_of_range_what,
            path2_io_error_what,
            path2_file_io_error_what,
            path2_host_network_error_what,
            path2_export_error_what,
            path2_placeholder_parser_error_what,
        ];
        for ((name, make), oracle_fn) in cases.iter().zip(oracle_fns.iter()) {
            let oracle_what = oracle_fn(msg);
            let ported_what = make(msg.to_string()).message().to_string();
            assert_eq!(oracle_what, ported_what, "mismatch for {name}");
        }

        let oracle_slicing_error_what = path2_slicing_error_what(msg);
        let ported_slicing_error_what = SlicingError::new(msg).message;
        assert_eq!(oracle_slicing_error_what, ported_slicing_error_what);

        let oracle_default_object_id = path2_slicing_error_default_object_id(msg);
        let ported_default_object_id = SlicingError::new(msg).object_id() as u64;
        assert_eq!(oracle_default_object_id, ported_default_object_id);

        let oracle_object_id = path2_slicing_error_object_id(msg, 42);
        let ported_object_id = SlicingError::with_object_id(msg, 42).object_id() as u64;
        assert_eq!(oracle_object_id, ported_object_id);

        let oracle_from_message = path2_slicing_errors_from_message_what(msg);
        let ported_from_message = Exception::new_slicing_errors_with_message(msg)
            .message()
            .to_string();
        assert_eq!(oracle_from_message, ported_from_message);

        for count in [0u32, 1, 3] {
            let oracle_vector_what = path2_slicing_errors_from_vector_what(count);
            let ported_errors = vec![SlicingError::new("e"); count as usize];
            let ported_vector_what = Exception::new_slicing_errors(ported_errors.clone())
                .message()
                .to_string();
            assert_eq!(oracle_vector_what, ported_vector_what);

            let oracle_vector_count = path2_slicing_errors_from_vector_count(count);
            let ported_vector_count = ported_errors.len() as i64;
            assert_eq!(oracle_vector_count, ported_vector_count);
        }
    }
}

// PORT STATUS
// source: src/libslic3r/Exception.hpp
// confidence: high
// tier: exact
// oracle: built
