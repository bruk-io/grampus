// task 29-ffi-bridge-core, recorded-decision fixture (companion to safety-no-manifest-row):
// the *other* class of file that check_module's `require_trailer` exemption covers, pinned here
// so it is a decision and not an accident.
//
// This file looks like a genuinely ported module, but port/manifest.tsv's only row spells its
// target module `crates/slic3r-typo/src/circl.rs` -- a near miss of this file's real path -- so
// no manifest row names it, and its completion trailer has been deleted. Its one `// SAFETY:`
// marker is the only reason the sniff in scan_tree reads it at all. (This comment deliberately
// never spells the trailer's header line: that spelling anywhere in the text is itself what
// scan_tree's has_trailer_header sniff looks for, and writing it here would make the fixture
// exercise the opposite branch.)
//
// Measured against main's port-status on the same tree, main reports one finding here
// ("missing-trailer: no line reading exactly ... found") and this branch reports none. That is
// deliberate, not an oversight: the manifest row is the authority a trailer requirement rests
// on -- check_trailer judges a trailer's `source:` and `tier:` lines against that row -- so a
// dropped or misspelled row is a manifest defect, which the manifest's own checks and R14's
// status derivation answer, not a trailer defect for this check to invent. The alternative
// considered and not taken was a crate-level guard: require a trailer when any manifest row
// names this file's *crate*, which would close this gap while still exempting slic3r-ffi, no
// row naming that crate. It was rejected here because it would make the trailer requirement
// depend on a sibling file's row rather than on this file's own, a wider change than this task
// needs; a later task is free to take it.
//
// Every other per-module check still runs on a file in this class: in the same shape with a
// `// PORT SRC:` marker over an uncovered literal instead, main reports three findings and this
// branch two -- only the missing-trailer finding disappears, and the numeric-constant and
// unmatched-marker findings still fire.

// SAFETY: not a real FFI declaration -- just enough `unsafe` to put this file in the marker
// sniff's reach, which is what makes the exemption observable at all.
unsafe fn noop() {}
