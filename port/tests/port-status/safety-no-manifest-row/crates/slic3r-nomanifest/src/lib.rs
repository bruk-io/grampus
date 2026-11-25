// task 29-ffi-bridge-core regression fixture: a crate with no manifest row of its own (it
// ports no libslic3r source file) and no completion trailer of its own writing, whose only
// `unsafe` carries a safety-justification comment. The marker sniff still pulls this file into
// scan_tree (it is the only reason this file is scanned at all), but the trailer check must not
// run on it -- there is no manifest row and no trailer attempt to judge it against. This must
// exit clean (no missing-trailer finding), which is the whole point of the fixture: see
// port/port-status's check_module and scan_tree for the require_trailer logic this exercises.

// SAFETY: not a real FFI declaration, just enough unsafe to exercise check_safety without
// tripping check_trailer, which this fixture asserts never runs on a file like this one.
unsafe fn noop() {}
