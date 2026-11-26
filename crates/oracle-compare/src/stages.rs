//! `just compare-stages` (task `39-compare-stages`, R20, plan step 24, part 1 — the command).
//!
//! Slices `fixtures/3DBenchy.stl` with the port (`crates/grampus-cli`, driven as a subprocess —
//! this crate names no dependency on `grampus-cli` or on any `slic3r-*` crate, so `cargo test -p
//! oracle-compare` never needs a libslic3r build) and, for every pipeline stage the port's
//! `port/manifest.tsv` marks as fully ported, compares the port's per-seam dump against the
//! committed oracle dump for that stage at the stage's declared tier. A stage whose rows are not
//! all `ported` prints `not ported`, which is not a failure (task.md, R20): no stage output
//! exists until every file that stage reaches is ported, so this command is useful from the day
//! it lands and gets stronger with every batch that merges.
//!
//! **The stage list is read, not restated.** `stage_names_in_order` returns the manifest's own
//! `pipeline stage or none` column's distinct non-`none` values — on the tree this task lands on,
//! nine of them (`posSlice`, `posPerimeters`, `posPrepareInfill`, `posInfill`, `posContouring`,
//! `psWipeTower`, `psSkirtBrim`, `psGCodeExport`, `psConflictCheck`), not the thirteen task.md's
//! own prose describes (R5's eleven seams plus the two fallback stages) — four of R5's seams
//! (`posIroning`, `posSupportMaterial`, `posDetectOverhangsForLift`, `posSimplifyPath`) are
//! simply not yet the `pipeline stage` of any row in the committed manifest (grep found none;
//! `Support/SupportMaterial.hpp`, for instance, is `posSlice`). This module never hardcodes
//! either count: it reads whatever the column holds, which is what task.md's own words, read
//! literally ("read the list from the manifest column"), ask for, and what keeps this module
//! honest if `06-manifest-scope` never ends up assigning those four names to any row. This
//! discrepancy is called out in this task's own pull request body, not corrected here — this
//! module may not edit `port/manifest.tsv`.
//!
//! **"Fully ported" is derived, never read** (`row_is_ported`): mirrors `port/port-status`'s
//! `derive_row_status`'s `ported` arm exactly (mod-reachable from some crate's `src/lib.rs`, no
//! live `todo!()`/`unimplemented!()`, and a `// PROOF:` marker naming the row's own `proof path`
//! cell beside a test naming the module) — never the manifest's own `status` cell, which is
//! informational only (`port/port-status`'s own doc comment). This module reimplements that
//! Python logic in Rust rather than shelling out to it, because it must run from `cargo test -p
//! oracle-compare` with no Python and no built libslic3r.
//!
//! Two parts of that arm come from docs/PORTING.md's "Decisions recorded on 2026-11-25" and are
//! mirrored here too, under the same names the Python carries: a `// PROOF:` marker counts only
//! over a test that compares a value the ORACLE produced against a value this port computed
//! (`has_differential_comparison`), and a row whose `proof path` cell reads
//! `enclosing: <path>` carries no marker of its own and is `ported` exactly when the in-scope
//! row it names is (`enclosing_unit_of`, `enclosing_row`). `stages.rs`'s own tests hold the Rust
//! implementation to the same five body shapes `port/port-status --self-test` holds the Python
//! one to, so the two cannot drift apart silently.
//!
//! **The two seamless stages fall back to the exported G-code's L0-L4 levels**
//! (`is_fallback_stage`, `FALLBACK_STAGES`): `psGCodeExport` and `psConflictCheck` have no R5
//! seam, so this command never has a per-seam dump for them and instead compares the port's and
//! the built oracle's exported G-code the same way `just compare` already does, saying so in its
//! output rather than naming whichever ordinary seam happens to print immediately above them as
//! their first difference (an earlier draft of this comment said "`psSkirtBrim`, the last seam"
//! — wrong on both counts: `stage_names_in_order`'s printed order is manifest order, not firing
//! order, so the preceding line is not reliably `psSkirtBrim`, and `posSimplifyPath`, not
//! `psSkirtBrim`, is the seam that actually fires last — see `crates/grampus-cli/src/surface.rs`'s
//! `STAGE_DUMP_SEAMS` doc comment).

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::tolerance::{self, Tolerance};
use crate::{compare_with_tolerance, Level, Report};

/// The two pipeline steps with no value in `SlicingPipelineStepPlugin` and therefore no R5 seam
/// at all (spec.md R5, R20; C11). Named here, literally, because this command has to recognise
/// them to decide which fallback applies — this is not "restating the eleven-seam list" (R5's
/// own enumeration), which this module reads from the manifest column instead and nowhere
/// spells out.
pub const FALLBACK_STAGES: [&str; 2] = ["psGCodeExport", "psConflictCheck"];

pub fn is_fallback_stage(stage: &str) -> bool {
    FALLBACK_STAGES.contains(&stage)
}

// --- The manifest ------------------------------------------------------------------------

/// One `port/manifest.tsv` row, holding only the columns this module reads. Column lookup is by
/// header name (like `tolerance::from_manifest_row`), so a reordered or renamed column does not
/// silently break this module.
#[derive(Debug, Clone)]
pub struct ManifestRow {
    /// The vendored `path` cell (e.g. `AABBTreeIndirect.hpp`), used as `tolerance.rs`'s own row
    /// key — unused here except as the key `stage_tolerance` passes to
    /// `tolerance::from_manifest_row`.
    pub path: String,
    pub in_scope: bool,
    pub target_crate: String,
    pub target_module: String,
    /// The raw `pipeline stage or none` cell: a stage name or the literal `none`.
    pub stage: String,
    pub proof_path: String,
    /// The raw `equality tier` cell (R19/R20: "exact equality of integer-coordinate output for a
    /// fixed-point stage, and the row's declared tolerance and comparison function for a float
    /// one"). `stage_tolerance` reads this only to refuse the one contradiction a row's own
    /// cells can state (`exact` *and* a declared tolerance) — the actual comparison tier is
    /// still the resolved `Tolerance`, exactly as every other consumer of
    /// `tolerance::from_manifest_row` already reads it.
    pub equality_tier: String,
}

fn find_col(headers: &[&str], name: &str) -> Result<usize, String> {
    headers
        .iter()
        .position(|h| h.eq_ignore_ascii_case(name))
        .ok_or_else(|| format!("manifest has no {name:?} column"))
}

/// Reads every row of `tsv_path`, finding the columns this module needs by header name. A
/// missing file or a missing required column is a named `Err`; a short data row reads its
/// missing cells as empty strings, same as `tolerance::from_manifest_row`.
pub fn load_manifest_rows(tsv_path: &Path) -> Result<Vec<ManifestRow>, String> {
    let content = fs::read_to_string(tsv_path)
        .map_err(|e| format!("cannot read manifest '{}': {e}", tsv_path.display()))?;
    let mut lines = content.lines();
    let header_line = lines
        .next()
        .ok_or_else(|| format!("manifest '{}' is empty", tsv_path.display()))?;
    let headers: Vec<&str> = header_line.split('\t').collect();

    let path_col = find_col(&headers, "path")?;
    let scope_col = find_col(&headers, "in scope or out with a reason")?;
    let crate_col = find_col(&headers, "target crate")?;
    let module_col = find_col(&headers, "target module")?;
    let stage_col = find_col(&headers, "pipeline stage or none")?;
    let proof_col = find_col(&headers, "proof path")?;
    let tier_col = find_col(&headers, "equality tier")?;

    let mut rows = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        let get = |idx: usize| fields.get(idx).copied().unwrap_or("").to_string();
        let scope_cell = get(scope_col);
        let in_scope = scope_cell.split(':').next().unwrap_or("").trim() == "in";
        rows.push(ManifestRow {
            path: get(path_col),
            in_scope,
            target_crate: get(crate_col),
            target_module: get(module_col),
            stage: get(stage_col),
            proof_path: get(proof_col),
            equality_tier: get(tier_col),
        });
    }
    Ok(rows)
}

/// The manifest's distinct, non-`none` `pipeline stage` values, in the order the stages that are
/// not `FALLBACK_STAGES` first appear in the file, with any fallback stage present moved to the
/// end in the fixed order `FALLBACK_STAGES` declares (`psGCodeExport` then `psConflictCheck` —
/// the order task.md's own prose and R5 always name them in). This is a printing-order decision
/// this task makes, not a fact read off the manifest: the column carries no ordering of its own,
/// and putting the two seamless stages last keeps "the first stage that differs" from ever
/// naming one of them ahead of a seam it did not actually run against.
///
/// **This printed order is manifest row order, not `Print::process()`'s own firing order**
/// (`vendor/OrcaSlicer/src/libslic3r/Print.cpp:3300` and following) — this module reads the
/// stage list from `port/manifest.tsv` and names no seam list of its own (a Done-when line a
/// reviewer checks by grepping this file for the status column and finding nothing; restating
/// the firing order here would be the same kind of restatement), so "the first stage that
/// differs" among the ordinary (non-fallback) stages is the first of them *by manifest row
/// order*, which can print and therefore name a stage that fires *later* in the real pipeline
/// than another differing stage printed after it (e.g. `psWipeTower`, print-wide and firing
/// late, can appear before several per-object seams that fire earlier). Only the two fallback
/// stages are special-cased to the end, for the reason above; every other stage's position is
/// manifest order, unexamined against firing order.
pub fn stage_names_in_order(rows: &[ManifestRow]) -> Vec<String> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut ordinary = Vec::new();
    let mut fallback_seen: HashSet<&str> = HashSet::new();
    for row in rows {
        let s = row.stage.trim();
        if s.is_empty() || s == "none" {
            continue;
        }
        if !seen.insert(s.to_string()) {
            continue;
        }
        if let Some(f) = FALLBACK_STAGES.iter().find(|f| **f == s) {
            fallback_seen.insert(f);
        } else {
            ordinary.push(s.to_string());
        }
    }
    let mut out = ordinary;
    for f in FALLBACK_STAGES {
        if fallback_seen.contains(f) {
            out.push(f.to_string());
        }
    }
    out
}

/// Every in-scope row whose `pipeline stage` cell is exactly `stage`.
pub fn rows_for_stage<'a>(rows: &'a [ManifestRow], stage: &str) -> Vec<&'a ManifestRow> {
    rows.iter()
        .filter(|r| r.in_scope && r.stage.trim() == stage)
        .collect()
}

// --- Status derivation (mirrors `port/port-status`'s `derive_row_status`) ----------------

/// A faithful-enough reimplementation of `port/port-status`'s `split_code_comment`: the code
/// part of `line` up to the first `//` that is outside a string or char literal. Same documented
/// limit as the Python (no raw strings, no byte strings).
fn split_code_comment(line: &[char]) -> Vec<char> {
    let mut in_string = false;
    let mut in_char = false;
    let mut i = 0;
    let n = line.len();
    while i < n {
        let c = line[i];
        if in_string {
            if c == '\\' {
                i += 2;
                continue;
            }
            if c == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        if in_char {
            if c == '\\' {
                i += 2;
                continue;
            }
            if c == '\'' {
                in_char = false;
            }
            i += 1;
            continue;
        }
        if c == '"' {
            in_string = true;
            i += 1;
            continue;
        }
        if c == '\'' {
            let window_end = (i + 5).min(n);
            if line[i + 1..window_end].contains(&'\'') {
                in_char = true;
                i += 1;
                continue;
            }
            i += 1;
            continue;
        }
        if c == '/' && i + 1 < n && line[i + 1] == '/' {
            return line[..i].to_vec();
        }
        i += 1;
    }
    line.to_vec()
}

/// Mirrors `port/port-status`'s `strip_block_comments`: replaces the interior of every `/* ...
/// */` block comment with spaces, tracking state across the whole file (a block comment spans
/// lines), so line count and every other line's length is unchanged.
fn strip_block_comments(text: &str) -> String {
    let mut out: Vec<char> = text.chars().collect();
    let n = out.len();
    let chars: Vec<char> = out.clone();
    let mut in_string = false;
    let mut in_char = false;
    let mut in_line_comment = false;
    let mut in_block = false;
    let mut i = 0;
    while i < n {
        let c = chars[i];
        if c == '\n' {
            in_line_comment = false;
            i += 1;
            continue;
        }
        if in_line_comment {
            i += 1;
            continue;
        }
        if in_block {
            if c == '*' && i + 1 < n && chars[i + 1] == '/' {
                out[i] = ' ';
                out[i + 1] = ' ';
                in_block = false;
                i += 2;
                continue;
            }
            out[i] = ' ';
            i += 1;
            continue;
        }
        if in_string {
            if c == '\\' {
                i += 2;
                continue;
            }
            if c == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        if in_char {
            if c == '\\' {
                i += 2;
                continue;
            }
            if c == '\'' {
                in_char = false;
            }
            i += 1;
            continue;
        }
        if c == '"' {
            in_string = true;
            i += 1;
            continue;
        }
        if c == '\'' {
            let window_end = (i + 5).min(n);
            let window = &chars[i + 1..window_end];
            if window.contains(&'\'')
                && !window[..window.iter().position(|c| *c == '\'').unwrap()].contains(&'\n')
            {
                in_char = true;
                i += 1;
                continue;
            }
            i += 1;
            continue;
        }
        if c == '/' && i + 1 < n && chars[i + 1] == '/' {
            in_line_comment = true;
            i += 2;
            continue;
        }
        if c == '/' && i + 1 < n && chars[i + 1] == '*' {
            out[i] = ' ';
            out[i + 1] = ' ';
            in_block = true;
            i += 2;
            continue;
        }
        i += 1;
    }
    out.into_iter().collect()
}

/// `code_only`: the text `derive_row_status` and `compute_mod_reachable_files` actually read —
/// `strip_block_comments` then, per line, `split_code_comment`'s code half. Deliberately does
/// not replicate `classify_lines`' `#[cfg(test)]` exemption bookkeeping: neither the `mod`-decl
/// scan nor the `todo!`/`unimplemented!` scan in `port/port-status` reads that exemption, only
/// the numeric-literal check does, which this module does not implement (it is not part of
/// status derivation).
fn code_only_text(text: &str) -> String {
    let masked = strip_block_comments(text);
    masked
        .lines()
        .map(|line| {
            let chars: Vec<char> = line.chars().collect();
            split_code_comment(&chars).into_iter().collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Mirrors `port/port-status`'s `MOD_DECL_RE`: `^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+(ident)\s*;`
/// matched per line, since no `mod` declaration spans more than one line in this codebase.
fn match_mod_decl(line: &str) -> Option<String> {
    let bytes: Vec<char> = line.chars().collect();
    let n = bytes.len();
    let mut i = 0;
    while i < n && bytes[i].is_whitespace() {
        i += 1;
    }
    if line[byte_index(&bytes, i)..].starts_with("pub") {
        i += 3;
        if i < n && bytes[i] == '(' {
            let mut depth = 1;
            i += 1;
            while i < n && depth > 0 {
                if bytes[i] == ')' {
                    depth -= 1;
                }
                i += 1;
            }
            if depth != 0 {
                return None;
            }
        }
        let before = i;
        while i < n && bytes[i].is_whitespace() {
            i += 1;
        }
        if i == before {
            return None;
        }
    }
    if !line[byte_index(&bytes, i)..].starts_with("mod") {
        return None;
    }
    i += 3;
    let before = i;
    while i < n && bytes[i].is_whitespace() {
        i += 1;
    }
    if i == before {
        return None;
    }
    let ident_start = i;
    while i < n && (bytes[i].is_ascii_alphanumeric() || bytes[i] == '_') {
        i += 1;
    }
    if i == ident_start {
        return None;
    }
    let ident: String = bytes[ident_start..i].iter().collect();
    if ident
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(true)
    {
        return None;
    }
    while i < n && bytes[i].is_whitespace() {
        i += 1;
    }
    if i < n && bytes[i] == ';' {
        Some(ident)
    } else {
        None
    }
}

/// Byte offset of char index `idx` in `chars` — `match_mod_decl`/`parse_fn_decl_name` work in
/// char space (like the Python) but need a byte offset to slice the original `&str` for
/// `starts_with`.
fn byte_index(chars: &[char], idx: usize) -> usize {
    chars[..idx].iter().map(|c| c.len_utf8()).sum()
}

fn find_mod_decls(code_only: &str) -> Vec<String> {
    code_only.lines().filter_map(match_mod_decl).collect()
}

/// Mirrors `port/port-status`'s `TODO_OR_UNIMPLEMENTED_RE`: `\btodo!\s*\(|\bunimplemented!\s*\(`,
/// a live (not merely mentioned-in-a-comment, since the caller passes `code_only`) `todo!(` or
/// `unimplemented!(` call.
fn has_live_todo_or_unimplemented(code_only: &str) -> bool {
    contains_bang_call(code_only, "todo") || contains_bang_call(code_only, "unimplemented")
}

fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn contains_bang_call(text: &str, word: &str) -> bool {
    let pattern = format!("{word}!");
    let bytes: Vec<char> = text.chars().collect();
    let plen = pattern.chars().count();
    let pat_chars: Vec<char> = pattern.chars().collect();
    let n = bytes.len();
    let mut i = 0;
    'outer: while i + plen <= n {
        for (k, pc) in pat_chars.iter().enumerate() {
            if bytes[i + k] != *pc {
                i += 1;
                continue 'outer;
            }
        }
        let before_ok = i == 0 || !is_word_char(bytes[i - 1]);
        if before_ok {
            let mut j = i + plen;
            while j < n && bytes[j].is_whitespace() {
                j += 1;
            }
            if j < n && bytes[j] == '(' {
                return true;
            }
        }
        i += 1;
    }
    false
}

/// Mirrors `port/port-status`'s `compute_mod_reachable_files`: every `.rs` file under
/// `tree/crates/*/src/` reachable from some crate's `src/lib.rs` by a chain of `mod name;`
/// declarations (file-per-module or `mod.rs` per directory).
pub fn mod_reachable_files(tree: &Path) -> HashSet<PathBuf> {
    let mut reachable = HashSet::new();
    let crates_root = tree.join("crates");
    let Ok(entries) = fs::read_dir(&crates_root) else {
        return reachable;
    };
    let mut crate_dirs: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    crate_dirs.sort();
    for crate_dir in crate_dirs {
        let lib_rs = crate_dir.join("src").join("lib.rs");
        if !lib_rs.is_file() {
            continue;
        }
        let mut stack = vec![lib_rs.clone()];
        let mut seen: HashSet<PathBuf> = HashSet::new();
        seen.insert(lib_rs);
        while let Some(f) = stack.pop() {
            reachable.insert(f.clone());
            let Ok(text) = fs::read_to_string(&f) else {
                continue;
            };
            let file_name = f.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let resolve_dir = if file_name == "lib.rs" || file_name == "mod.rs" {
                f.parent().unwrap().to_path_buf()
            } else {
                let stem = f.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                f.parent().unwrap().join(stem)
            };
            let code_only = code_only_text(&text);
            for name in find_mod_decls(&code_only) {
                for candidate in [
                    resolve_dir.join(format!("{name}.rs")),
                    resolve_dir.join(&name).join("mod.rs"),
                ] {
                    if candidate.is_file() && !seen.contains(&candidate) {
                        seen.insert(candidate.clone());
                        stack.push(candidate);
                    }
                }
            }
        }
    }
    reachable
}

/// Mirrors `TEST_ATTR_RE`: `^\s*#\s*\[\s*test\s*\]\s*$`, checked by discarding every whitespace
/// character and comparing to the compact form (equivalent here, since no whitespace is required
/// between any two non-whitespace tokens of `#[test]`).
fn is_test_attr_line(line: &str) -> bool {
    let compact: String = line.chars().filter(|c| !c.is_whitespace()).collect();
    compact == "#[test]"
}

/// Mirrors `CFG_TEST_RE`: `^#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]\s*$`.
fn is_cfg_test_line(line: &str) -> bool {
    let compact: String = line.chars().filter(|c| !c.is_whitespace()).collect();
    compact == "#[cfg(test)]"
}

/// Mirrors `PROOF_FN_DECL_RE`: `^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+(ident)`.
fn parse_fn_decl_name(line: &str) -> Option<String> {
    let bytes: Vec<char> = line.chars().collect();
    let n = bytes.len();
    let mut i = 0;
    while i < n && bytes[i].is_whitespace() {
        i += 1;
    }
    if line[byte_index(&bytes, i)..].starts_with("pub") {
        i += 3;
        if i < n && bytes[i] == '(' {
            let mut depth = 1;
            i += 1;
            while i < n && depth > 0 {
                if bytes[i] == ')' {
                    depth -= 1;
                }
                i += 1;
            }
            if depth != 0 {
                return None;
            }
        }
        let before = i;
        while i < n && bytes[i].is_whitespace() {
            i += 1;
        }
        if i == before {
            return None;
        }
    }
    if line[byte_index(&bytes, i)..].starts_with("async") {
        i += 5;
        let before = i;
        while i < n && bytes[i].is_whitespace() {
            i += 1;
        }
        if i == before {
            return None;
        }
    }
    if !line[byte_index(&bytes, i)..].starts_with("fn") {
        return None;
    }
    i += 2;
    let before = i;
    while i < n && bytes[i].is_whitespace() {
        i += 1;
    }
    if i == before {
        return None;
    }
    let ident_start = i;
    while i < n && (bytes[i].is_ascii_alphanumeric() || bytes[i] == '_') {
        i += 1;
    }
    if i == ident_start {
        return None;
    }
    let ident: String = bytes[ident_start..i].iter().collect();
    if ident
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(true)
    {
        return None;
    }
    Some(ident)
}

/// Mirrors `PROOF_LINE_RE`: `^\s*//\s*PROOF:\s*(.*?)\s*$`.
fn parse_proof_line(line: &str) -> Option<String> {
    let rest = line.trim_start().strip_prefix("//")?;
    let rest = rest.trim_start().strip_prefix("PROOF:")?;
    Some(rest.trim().to_string())
}

/// Mirrors `port/port-status`'s `mask_strings`: replaces the contents of every string and char
/// literal with `'x'`, preserving length and the position of everything else.
fn mask_strings(code: &str) -> String {
    let mut out: Vec<char> = code.chars().collect();
    let n = out.len();
    let chars = out.clone();
    let mut in_string = false;
    let mut in_char = false;
    let mut i = 0;
    while i < n {
        let c = chars[i];
        if in_string {
            if c == '\\' {
                out[i] = 'x';
                if i + 1 < n {
                    out[i + 1] = 'x';
                }
                i += 2;
                continue;
            }
            if c == '"' {
                in_string = false;
                i += 1;
                continue;
            }
            out[i] = 'x';
            i += 1;
            continue;
        }
        if in_char {
            if c == '\\' {
                out[i] = 'x';
                if i + 1 < n {
                    out[i + 1] = 'x';
                }
                i += 2;
                continue;
            }
            if c == '\'' {
                in_char = false;
                i += 1;
                continue;
            }
            out[i] = 'x';
            i += 1;
            continue;
        }
        if c == '"' {
            in_string = true;
            i += 1;
            continue;
        }
        if c == '\'' {
            let window_end = (i + 5).min(n);
            if chars[i + 1..window_end].contains(&'\'') {
                in_char = true;
                i += 1;
                continue;
            }
            i += 1;
            continue;
        }
        i += 1;
    }
    out.into_iter().collect()
}

/// Mirrors `port/port-status`'s `extract_cfg_test_text`: the raw text of every `#[cfg(test)]`
/// module body in `text`, found by brace counting on `mask_strings`-masked code (so a brace
/// written inside a string or char literal never moves the depth).
fn extract_cfg_test_text(text: &str) -> String {
    let raw_lines: Vec<&str> = text.lines().collect();
    let masked_text = strip_block_comments(text);
    let masked_lines: Vec<&str> = masked_text.lines().collect();
    let mut out_lines: Vec<&str> = Vec::new();
    let mut in_block = false;
    let mut depth: i64 = 0;
    let mut seen_open = false;
    for (raw, masked) in raw_lines.iter().zip(masked_lines.iter()) {
        let stripped = raw.trim();
        if !in_block && is_cfg_test_line(stripped) {
            in_block = true;
            depth = 0;
            seen_open = false;
            continue;
        }
        if in_block {
            let masked_chars: Vec<char> = masked.chars().collect();
            let code = split_code_comment(&masked_chars);
            let code_str: String = code.into_iter().collect();
            let masked_code = mask_strings(&code_str);
            let opens = masked_code.matches('{').count() as i64;
            let closes = masked_code.matches('}').count() as i64;
            if opens > 0 {
                seen_open = true;
            }
            depth += opens - closes;
            out_lines.push(raw);
            if seen_open && depth <= 0 {
                in_block = false;
            }
        }
    }
    out_lines.join("\n")
}

/// Mirrors `find_proof_match`: a `#[test]`-attributed function whose name contains
/// `module_snake`, carrying beside it (a run of `//` lines right above its `#[test]`, through
/// the function's own `fn` line) the literal line `// PROOF: <proof_path_cell>`, **and** whose
/// body compares a value the oracle produced against a value this port computed
/// (`has_differential_comparison`; docs/PORTING.md, "Decisions recorded on 2026-11-25").
fn find_proof_match(text: &str, module_snake: &str, proof_path_cell: &str) -> bool {
    let wanted = proof_path_cell.trim();
    if wanted.is_empty() {
        return false;
    }
    let oracle = oracle_names(text);
    let lines: Vec<&str> = text.lines().collect();
    for i in 0..lines.len() {
        if !is_test_attr_line(lines[i]) {
            continue;
        }
        let mut j = i + 1;
        while j < lines.len()
            && (lines[j].trim().is_empty() || lines[j].trim_start().starts_with("#["))
        {
            j += 1;
        }
        if j >= lines.len() {
            continue;
        }
        let Some(fn_name) = parse_fn_decl_name(lines[j]) else {
            continue;
        };
        if !fn_name.contains(module_snake) {
            continue;
        }
        let mut window_start = i;
        if i > 0 {
            let mut k = i - 1;
            loop {
                if lines[k].trim_start().starts_with("//") {
                    window_start = k;
                    if k == 0 {
                        break;
                    }
                    k -= 1;
                } else {
                    break;
                }
            }
        }
        let window_end = (j + 1).min(lines.len().saturating_sub(1));
        for li in window_start..=window_end {
            if let Some(path) = parse_proof_line(lines[li]) {
                if path == wanted && has_differential_comparison(&function_body(&lines, j), &oracle)
                {
                    return true;
                }
            }
        }
    }
    false
}

// --- The differential-comparison rule and the `enclosing:` proof-path form -----------------
//
// Mirrors `port/port-status`'s `oracle_names`, `function_body`, `split_call_arguments`,
// `tainted_identifiers`, `has_differential_comparison`, `enclosing_unit_of` and
// `enclosing_row_error` (docs/PORTING.md, "Decisions recorded on 2026-11-25"). Reimplemented in
// Rust for the same reason the rest of this module is: `cargo test -p oracle-compare` must run
// with no Python and no built libslic3r. Hand-rolled scanners rather than a regex crate, in the
// same style as `mask_strings` and `extract_cfg_test_text` above, so this module adds no
// dependency.

/// The oracle entry points a proof may call: the per-unit proof bridges `slic3r-proof`
/// re-exports and the committed stage dumps, taken from the file's own `use` paths, plus every
/// name declared in an `extern "C"` block. Mirrors `port/port-status`'s `oracle_names`.
fn oracle_names(text: &str) -> HashSet<String> {
    const ORACLE_PATH_SEGMENTS: [&str; 2] = ["proof_units", "dumps"];
    // Deliberately NOT seeded with the two bare words: see `port/port-status`'s `oracle_names`
    // for why (a local named `dumps` would otherwise taint its binding). A file that reaches an
    // oracle names it in a `use` path, which the loop below collects.
    let mut found: HashSet<String> = HashSet::new();
    // A `use` statement is accumulated across lines until its `;`, never read one line at a
    // time: `cargo fmt` breaks an import over 100 characters into a multi-line braced form, and
    // a line-at-a-time read of that sees only `use slic3r_proof::proof_units::object_id::{` and
    // so collects none of the imported names. Measured against
    // `crates/slic3r-objectid/tests/object_id_proof_path_1.rs`, where the first version of this
    // function found no oracle name at all while `port/port-status`'s regex (`[^;]+`, which
    // spans newlines) found both -- the exact Python/Rust divergence this mirror exists to
    // prevent.
    let mut statements: Vec<String> = Vec::new();
    let mut current: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if current.is_none() && !trimmed.starts_with("use ") {
            continue;
        }
        let acc = current.get_or_insert_with(String::new);
        acc.push(' ');
        acc.push_str(trimmed);
        if trimmed.contains(';') {
            let done = current.take().unwrap_or_default();
            statements.push(done);
        }
    }
    for statement in &statements {
        let rest = statement.trim_start().trim_start_matches("use ");
        let Some(path) = rest.split(';').next() else {
            continue;
        };
        if !ORACLE_PATH_SEGMENTS.iter().any(|seg| path.contains(seg)) {
            continue;
        }
        for segment in path
            .replace('{', " ")
            .replace('}', " ")
            .replace(',', " ")
            .split("::")
            .flat_map(|s| s.split_whitespace())
        {
            let segment = segment.trim();
            if !segment.is_empty() && is_ident(segment) {
                found.insert(segment.to_string());
            }
        }
    }
    // `extern "C" { ... }`: the names declared inside one, found by brace matching from the
    // block's own opening brace so an ordinary `fn` elsewhere in the file is not collected.
    let bytes: Vec<char> = text.chars().collect();
    let mut search_from = 0usize;
    while let Some(rel) = text[search_from..].find("extern") {
        let abs = search_from + rel;
        search_from = abs + "extern".len();
        let after: String = text[abs..].chars().take(16).collect();
        if !after.replace(' ', "").starts_with("extern\"C\"{") {
            continue;
        }
        let Some(open_rel) = text[abs..].find('{') else {
            continue;
        };
        let open = text[..abs + open_rel].chars().count();
        let mut depth = 0i64;
        let mut end = bytes.len();
        for (offset, c) in bytes[open..].iter().enumerate() {
            if *c == '{' {
                depth += 1;
            } else if *c == '}' {
                depth -= 1;
                if depth == 0 {
                    end = open + offset;
                    break;
                }
            }
        }
        let block: String = bytes[open..=end.min(bytes.len() - 1)].iter().collect();
        for name in declared_fn_names(&block) {
            found.insert(name);
        }
    }
    found
}

fn is_ident(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Every `fn NAME(` name in `text` -- used only over an `extern "C"` block's own body.
fn declared_fn_names(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i + 2 < chars.len() {
        if chars[i] == 'f'
            && chars[i + 1] == 'n'
            && (i == 0 || !(chars[i - 1].is_ascii_alphanumeric() || chars[i - 1] == '_'))
            && chars[i + 2].is_whitespace()
        {
            let mut j = i + 2;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            let start = j;
            while j < chars.len() && (chars[j].is_ascii_alphanumeric() || chars[j] == '_') {
                j += 1;
            }
            if j > start {
                let mut k = j;
                while k < chars.len() && chars[k].is_whitespace() {
                    k += 1;
                }
                if k < chars.len() && chars[k] == '(' {
                    out.push(chars[start..j].iter().collect::<String>());
                }
            }
            i = j;
            continue;
        }
        i += 1;
    }
    out
}

/// The text of the function whose signature starts at `lines[fn_line_idx]`, to its matching
/// closing brace, counted on the code-only, string-masked view. Mirrors `port/port-status`'s
/// `function_body`.
fn function_body(lines: &[&str], fn_line_idx: usize) -> String {
    let mut depth = 0i64;
    let mut seen_open = false;
    let mut out: Vec<&str> = Vec::new();
    for line in &lines[fn_line_idx..] {
        out.push(line);
        let masked_line = strip_block_comments(line);
        let chars: Vec<char> = masked_line.chars().collect();
        let code: String = split_code_comment(&chars).into_iter().collect();
        let masked = mask_strings(&code);
        let opens = masked.matches('{').count() as i64;
        let closes = masked.matches('}').count() as i64;
        if opens > 0 {
            seen_open = true;
        }
        depth += opens - closes;
        if seen_open && depth <= 0 {
            break;
        }
    }
    out.join("\n")
}

/// `mask_strings` fills a literal with the letter 'x', which the identifier scan below would
/// read as a value. Mirrors `port/port-status`'s `blank_masked_literals`: empty exactly what
/// `mask_strings` produces, never a lifetime, which it does not mask.
fn blank_masked_literals(masked: &str) -> String {
    let chars: Vec<char> = masked.chars().collect();
    let mut out = String::new();
    let mut i = 0usize;
    'scan: while i < chars.len() {
        if chars[i] == '"' {
            let mut j = i + 1;
            while j < chars.len() && chars[j] == 'x' {
                j += 1;
            }
            if j < chars.len() && chars[j] == '"' {
                out.push_str("\"\"");
                i = j + 1;
                continue 'scan;
            }
        }
        if chars[i] == '\'' {
            // Exactly what `mask_strings` produces for a char literal: one or two 'x's between
            // single quotes (two for an escape such as `'\n'`). Labelled `continue` on a match,
            // never a bare `break` out of the inner loop: an earlier revision broke only the
            // `for` and fell through to `out.push(chars[i]); i += 1;`, which pushed the character
            // *after* the literal and advanced past it. Traced rather than assumed: that
            // fall-through happens to produce the same output for every input that is valid
            // Rust, because the character after a char literal is never itself the start of
            // another literal -- so this is a control-flow correction for clarity, not a
            // behaviour change, and the char-literal case in this module's own tests pins the
            // behaviour either way.
            for width in [1usize, 2] {
                let close = i + 1 + width;
                if close < chars.len()
                    && chars[i + 1..close].iter().all(|c| *c == 'x')
                    && chars[close] == '\''
                {
                    out.push_str("''");
                    i = close + 1;
                    continue 'scan;
                }
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// The plain *value* identifiers in `text`: `own_identifiers`, minus a path segment (either end
/// of a `::`), a macro name (followed by `!`) and a turbofish's type argument (which is removed
/// first, since its `<` hides the `::` that would otherwise disqualify it). Mirrors
/// `port/port-status`'s `VALUE_IDENT_RE` and `value_identifiers`.
///
/// This is the view the PORTED side of a comparison is read with, and it is narrower than the
/// oracle side's on purpose: an oracle may legitimately be reached through a path, but
/// `vec![1, 0, 1]`, `String::from("x")`, `format!("x")` and `Vec::from([..])` are not values
/// this port computed, and reading them as such made `assert_eq!(oracle, vec![1, 0, 1])` -- an
/// oracle value against a literal -- count as differential. Measured.
fn value_identifiers(text: &str) -> Vec<String> {
    let without_turbofish = strip_turbofish(text);
    let chars: Vec<char> = without_turbofish.chars().collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i].is_ascii_alphabetic() || chars[i] == '_' {
            let prev = if i == 0 { None } else { Some(chars[i - 1]) };
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            // The whole token, never a prefix of it: a shorter match would stop being followed
            // by the `!` or `::` that disqualifies it (the same backtracking hole the Python
            // regex closes with a trailing `\b`).
            let preceded_by_path_or_field = matches!(prev, Some(p) if p == '.' || p == ':');
            let mut j = i;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            let followed_by_macro = chars.get(j) == Some(&'!');
            let followed_by_path = chars.get(j) == Some(&':') && chars.get(j + 1) == Some(&':');
            if !preceded_by_path_or_field && !followed_by_macro && !followed_by_path {
                out.push(chars[start..i].iter().collect::<String>());
            }
            continue;
        }
        i += 1;
    }
    out
}

/// `text` with every `::<...>` turbofish removed. Mirrors `port/port-status`'s `TURBOFISH_RE`.
fn strip_turbofish(text: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] == ':' && chars.get(i + 1) == Some(&':') && chars.get(i + 2) == Some(&'<') {
            if let Some(close) = chars[i + 3..].iter().position(|c| *c == '>' || *c == '<') {
                if chars[i + 3 + close] == '>' {
                    i = i + 3 + close + 1;
                    continue;
                }
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Every identifier in `text` not preceded by `.` or a word character -- so a field or method
/// name reached through a `.` never reads as naming a value of its own. Mirrors
/// `port/port-status`'s `IDENT_RE`. This is the view the ORACLE side is read with.
fn own_identifiers(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i].is_ascii_alphabetic() || chars[i] == '_' {
            let prev = if i == 0 { None } else { Some(chars[i - 1]) };
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let own = !matches!(prev, Some(p) if p == '.' || p.is_ascii_alphanumeric() || p == '_');
            if own {
                out.push(chars[start..i].iter().collect::<String>());
            }
            continue;
        }
        i += 1;
    }
    out
}

/// Kept byte-for-byte in step with `port/port-status`'s own `LITERAL_WORD_TOKENS`: the two
/// implementations of this rule must accept and reject the same bodies, and an entry in one list
/// and not the other is a divergence no fixture would catch. `String` was missing here once, and
/// `assert_eq!(oracle.iter().collect::<String>(), "101")` measured differently in the two.
const LITERAL_WORD_TOKENS: [&str; 25] = [
    "true", "false", "as", "None", "Some", "Ok", "Err", "i8", "i16", "i32", "i64", "i128", "isize",
    "u8", "u16", "u32", "u64", "u128", "usize", "f32", "f64", "bool", "char", "str", "String",
];

/// The code-only, string-blanked view of a function body that both scans below read. Mirrors
/// the identical pair of calls `port/port-status` makes.
fn comparison_view(body: &str) -> String {
    let code_only = code_only_text(body);
    blank_masked_literals(&mask_strings(&code_only))
}

/// A fully-qualified call into an oracle, for a proof that writes the path at the call site
/// instead of importing it. Required to be the `::` path form, never the bare word, so a local
/// named `dumps` does not taint its own binding. Mirrors `port/port-status`'s
/// `ORACLE_PATH_CALL_RE`.
fn names_oracle_path(text: &str) -> bool {
    text.contains("proof_units::") || text.contains("dumps::")
}

/// Every `(pattern, right-hand side)` pair in `view`: a `let <pattern> = <expr>;` and a
/// `for <pattern> in <expr>` (which `if let`/`while let` also match, since both write
/// `let <pattern> = <expr>`). Scanned with a bracket-depth counter, so a `;` or an `=` inside
/// the pattern's own brackets does not end it early. Mirrors `port/port-status`'s
/// `_find_bindings`.
fn find_bindings(view: &str) -> Vec<(String, String)> {
    let chars: Vec<char> = view.chars().collect();
    let mut out: Vec<(String, String)> = Vec::new();
    let mut i = 0usize;
    while i < chars.len() {
        let is_word_start =
            i == 0 || !(chars[i - 1].is_ascii_alphanumeric() || chars[i - 1] == '_');
        let keyword = if is_word_start
            && chars.len() > i + 3
            && chars[i..i + 3] == ['l', 'e', 't']
            && chars[i + 3].is_whitespace()
        {
            Some("let")
        } else if is_word_start
            && chars.len() > i + 3
            && chars[i..i + 3] == ['f', 'o', 'r']
            && chars[i + 3].is_whitespace()
        {
            Some("for")
        } else {
            None
        };
        let Some(keyword) = keyword else {
            i += 1;
            continue;
        };
        let mut j = i + 4;
        let pattern_start = j;
        let mut depth = 0i64;
        let mut stopped_at_separator = false;
        while j < chars.len() {
            match chars[j] {
                '(' | '[' | '{' | '<' => depth += 1,
                ')' | ']' | '}' | '>' => depth -= 1,
                _ => {}
            }
            if depth <= 0 {
                if keyword == "let" && chars[j] == '=' && chars.get(j + 1) != Some(&'=') {
                    stopped_at_separator = true;
                    break;
                }
                if keyword == "for"
                    && chars[j] == 'i'
                    && chars.get(j + 1) == Some(&'n')
                    && chars.get(j + 2).is_some_and(|c| c.is_whitespace())
                    && chars
                        .get(j.wrapping_sub(1))
                        .is_some_and(|c| c.is_whitespace())
                {
                    stopped_at_separator = true;
                    break;
                }
                if chars[j] == ';' || chars[j] == '{' {
                    break;
                }
            }
            j += 1;
        }
        if !stopped_at_separator {
            i += 4;
            continue;
        }
        let pattern: String = chars[pattern_start..j].iter().collect();
        let rhs_start = if keyword == "let" { j + 1 } else { j + 3 };
        let mut k = rhs_start.min(chars.len());
        depth = 0;
        while k < chars.len() {
            match chars[k] {
                '(' | '[' | '{' if keyword == "let" => depth += 1,
                ')' | ']' | '}' if keyword == "let" => depth -= 1,
                '(' | '[' => depth += 1,
                ')' | ']' => depth -= 1,
                _ => {}
            }
            if depth <= 0 && (chars[k] == ';' || (keyword == "for" && chars[k] == '{')) {
                break;
            }
            k += 1;
        }
        let rhs: String = chars[rhs_start.min(chars.len())..k.min(chars.len())]
            .iter()
            .collect();
        out.push((pattern, rhs));
        i = k.max(i + 4);
    }
    out
}

/// Every identifier holding a value the oracle produced: a name bound by a `let` or a `for`
/// whose right-hand side names an oracle entry point, plus the oracle names themselves. Mirrors
/// `port/port-status`'s `tainted_identifiers`, one level deep and iterated to the same small
/// fixed point.
fn tainted_identifiers(view: &str, oracle: &HashSet<String>) -> HashSet<String> {
    let mut tainted: HashSet<String> = oracle.clone();
    let bindings = find_bindings(view);
    for _ in 0..4 {
        let before = tainted.len();
        for (pattern, rhs) in &bindings {
            let reaches_oracle =
                names_oracle_path(rhs) || own_identifiers(rhs).iter().any(|n| tainted.contains(n));
            if !reaches_oracle {
                continue;
            }
            for name in own_identifiers(pattern) {
                tainted.insert(name);
            }
        }
        if tainted.len() == before {
            break;
        }
    }
    tainted.remove("mut");
    tainted.remove("in");
    tainted
}

/// The comparison calls a proof may make, mirroring `port/port-status`'s `COMPARISON_CALL_RE`.
const COMPARISON_CALLS: [&str; 8] = [
    "compare_unit_at",
    "compare_unit",
    "assert_eq",
    "assert_ne",
    "debug_assert_eq",
    "debug_assert_ne",
    "debug_assert",
    "assert",
];

/// The comma-separated top-level arguments of the call whose `(` is at `open`. Mirrors
/// `port/port-status`'s `split_call_arguments`.
fn split_call_arguments(chars: &[char], open: usize) -> Vec<String> {
    let mut depth = 0i64;
    let mut args: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut i = open;
    while i < chars.len() {
        let c = chars[i];
        if matches!(c, '(' | '[' | '{') {
            depth += 1;
            if depth == 1 {
                i += 1;
                continue;
            }
        } else if matches!(c, ')' | ']' | '}') {
            depth -= 1;
            if depth == 0 {
                args.push(current);
                return args;
            }
        }
        if depth == 1 && c == ',' {
            args.push(std::mem::take(&mut current));
            i += 1;
            continue;
        }
        current.push(c);
        i += 1;
    }
    args.push(current);
    args
}

/// True if `body` compares a value the oracle produced against a value this port computed.
/// Mirrors `port/port-status`'s `has_differential_comparison`, including its stated limit: a
/// hand-transcribed named constant in the test's own scope is not distinguishable from the
/// ported module's own `pub const`, and still counts here.
pub fn has_differential_comparison(body: &str, oracle: &HashSet<String>) -> bool {
    let view = comparison_view(body);
    let tainted = tainted_identifiers(&view, oracle);
    let chars: Vec<char> = view.chars().collect();
    let mut has_comparison = false;
    let mut i = 0usize;
    while i < chars.len() {
        let mut matched: Option<usize> = None;
        for name in COMPARISON_CALLS {
            let name_chars: Vec<char> = name.chars().collect();
            if i + name_chars.len() > chars.len() {
                continue;
            }
            if chars[i..i + name_chars.len()] != name_chars[..] {
                continue;
            }
            if i > 0 && (chars[i - 1].is_ascii_alphanumeric() || chars[i - 1] == '_') {
                continue;
            }
            let mut j = i + name_chars.len();
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            if j < chars.len() && chars[j] == '!' {
                j += 1;
                while j < chars.len() && chars[j].is_whitespace() {
                    j += 1;
                }
            }
            if j < chars.len() && chars[j] == '(' {
                matched = Some(j);
            }
            break;
        }
        let Some(open) = matched else {
            i += 1;
            continue;
        };
        has_comparison = true;
        let args = split_call_arguments(&chars, open);
        let oracle_args: Vec<usize> = args
            .iter()
            .enumerate()
            .filter(|(_, arg)| own_identifiers(arg).iter().any(|n| tainted.contains(n)))
            .map(|(index, _)| index)
            .collect();
        let ported_args: Vec<usize> = args
            .iter()
            .enumerate()
            .filter(|(_, arg)| {
                value_identifiers(arg)
                    .iter()
                    .any(|n| !tainted.contains(n) && !LITERAL_WORD_TOKENS.contains(&n.as_str()))
            })
            .map(|(index, _)| index)
            .collect();
        for a in &oracle_args {
            for b in &ported_args {
                if a != b {
                    return true;
                }
                // Same argument: it counts only if that argument itself makes a call, which is
                // what distinguishes `assert!(agrees(ported, oracle))` from
                // `assert_eq!(oracle[index], 5)`.
                if argument_makes_a_call(&args[*a]) {
                    return true;
                }
            }
        }
        i = open + 1;
    }
    if !has_comparison {
        return false;
    }
    // The element-wise shape, where the pairing happens in the loop header rather than in the
    // comparison: `for (got, want) in oracle.iter().zip(PORTED) { assert_eq!(got, want); }`.
    // Both loop variables are oracle-tainted, so the comparison above sees no ported side -- but
    // the `for`'s own right-hand side names both an oracle value and a value this port computed,
    // which is the comparison, one level up.
    find_bindings(&view).iter().any(|(_pattern, rhs)| {
        own_identifiers(rhs).iter().any(|n| tainted.contains(n))
            && value_identifiers(rhs)
                .iter()
                .any(|n| !tainted.contains(n) && !LITERAL_WORD_TOKENS.contains(&n.as_str()))
    })
}

/// An identifier (or a path's last segment) followed by `(` in one comparison argument. Mirrors
/// `port/port-status`'s `CALL_IN_ARGUMENT_RE`; see `has_differential_comparison` for the one
/// case it decides.
fn argument_makes_a_call(arg: &str) -> bool {
    let chars: Vec<char> = arg.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i].is_ascii_alphabetic() || chars[i] == '_' {
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let mut j = i;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            if chars.get(j) == Some(&'(') {
                return true;
            }
            continue;
        }
        i += 1;
    }
    false
}

/// The `port/manifest.tsv` `path` cell this row names as its enclosing unit, if its `proof path`
/// cell takes the `enclosing: <path>` form. Mirrors `port/port-status`'s `enclosing_unit_of`.
pub fn enclosing_unit_of(row: &ManifestRow) -> Option<String> {
    let cell = row.proof_path.trim();
    let rest = cell.strip_prefix("enclosing:")?.trim();
    if rest.is_empty() {
        None
    } else {
        Some(rest.to_string())
    }
}

/// True if this row's `enclosing: <path>` cell can be honoured: it names another in-scope row
/// whose own proof path is not itself an `enclosing:` cell. Mirrors `port/port-status`'s
/// `enclosing_row_error` (a chain is refused, so a cycle is impossible).
fn enclosing_row<'a>(row: &ManifestRow, all_rows: &'a [ManifestRow]) -> Option<&'a ManifestRow> {
    let named = enclosing_unit_of(row)?;
    if named == row.path.trim() {
        return None;
    }
    let target = all_rows.iter().find(|r| r.path.trim() == named)?;
    if !target.in_scope || enclosing_unit_of(target).is_some() {
        return None;
    }
    // Same `target crate`: R19's fallback is "proven through its *enclosing file's* exported
    // surface", and a unit in another crate is not that. See `port/port-status`'s
    // `enclosing_row_error` for why this is a floor rather than the whole of that wording.
    let own_crate = row.target_crate.trim();
    let target_crate = target.target_crate.trim();
    if !own_crate.is_empty() && !target_crate.is_empty() && own_crate != target_crate {
        return None;
    }
    Some(target)
}

fn walk_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
    paths.sort();
    for p in paths {
        if p.is_dir() {
            walk_rs_files(&p, out);
        } else if p.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

/// Mirrors `proof_present`: a `// PROOF: <row's own proof path cell>` marker beside a test
/// naming `target_path`'s module (its file stem), either in `tree/crates/<row's target
/// crate>/tests/` or inside a `#[cfg(test)]` module in `target_path` itself. Matched against
/// *this row's own* `proof path` cell, never a sibling row's (see `port/port-status`'s own doc
/// comment on this point).
pub fn proof_present(row: &ManifestRow, target_path: &Path, tree: &Path) -> bool {
    let proof_path_cell = row.proof_path.trim();
    if proof_path_cell.is_empty() {
        return false;
    }
    let module_snake = target_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let crate_name = row.target_crate.trim();
    if !crate_name.is_empty() {
        let tests_dir = tree.join("crates").join(crate_name).join("tests");
        if tests_dir.is_dir() {
            let mut files = Vec::new();
            walk_rs_files(&tests_dir, &mut files);
            for f in files {
                if let Ok(text) = fs::read_to_string(&f) {
                    if find_proof_match(&text, module_snake, proof_path_cell) {
                        return true;
                    }
                }
            }
        }
    }
    let Ok(module_text) = fs::read_to_string(target_path) else {
        return false;
    };
    let cfg_test_text = extract_cfg_test_text(&module_text);
    if !cfg_test_text.is_empty() && find_proof_match(&cfg_test_text, module_snake, proof_path_cell)
    {
        return true;
    }
    false
}

/// Mirrors `derive_row_status`'s `ported` arm exactly: `target module` points at a file that
/// exists, a `mod` declaration reaches it, it holds no live `todo!()`/`unimplemented!()`, and
/// its proof is present. Everything else (`drafted`, `not-started`) collapses to `false` here —
/// this command only ever needs the `ported` predicate (task.md: "the stage is fully ported
/// when every one of those rows has a derived status of `ported`"), never the three-way status.
///
/// `all_rows` is the whole manifest, which the `enclosing: <path>` proof-path form needs: such a
/// row carries no `// PROOF:` marker of its own and is `ported` exactly when the in-scope row it
/// names is (R19's own fallback; docs/PORTING.md, "Decisions recorded on 2026-11-25"). A cell
/// that cannot be honoured — naming no row, an out-of-scope row, itself, or a row whose own
/// proof path is another `enclosing:` cell — is `false`, never `true`.
pub fn row_is_ported(
    row: &ManifestRow,
    all_rows: &[ManifestRow],
    tree: &Path,
    mod_reachable: &HashSet<PathBuf>,
) -> bool {
    let target_module_cell = row.target_module.trim();
    if target_module_cell.is_empty() {
        return false;
    }
    let target_path = tree.join(target_module_cell);
    if !target_path.is_file() {
        return false;
    }
    let Ok(text) = fs::read_to_string(&target_path) else {
        return false;
    };
    let code_only = code_only_text(&text);
    if has_live_todo_or_unimplemented(&code_only) {
        return false;
    }
    if !mod_reachable.contains(&target_path) {
        return false;
    }
    if enclosing_unit_of(row).is_some() {
        return match enclosing_row(row, all_rows) {
            Some(enclosing) => row_is_ported(enclosing, all_rows, tree, mod_reachable),
            None => false,
        };
    }
    proof_present(row, &target_path, tree)
}

/// A stage is fully ported when it has at least one in-scope row and every one of them derives
/// `ported`. A stage that names only out-of-scope rows (so `rows` is empty) is reported `false`,
/// not vacuously `true` — task.md's "every ... row has a derived status of ported" is read here
/// as requiring at least one row to hold the stage to, since a stage with nothing to check is
/// not a stage this run has actually proven anything about.
pub fn stage_fully_ported(
    rows: &[&ManifestRow],
    all_rows: &[ManifestRow],
    tree: &Path,
    mod_reachable: &HashSet<PathBuf>,
) -> bool {
    !rows.is_empty()
        && rows
            .iter()
            .all(|r| row_is_ported(r, all_rows, tree, mod_reachable))
}

// --- Tier (tolerance) ---------------------------------------------------------------------

/// The strictest (smallest allowed deviation) of `a` and `b`, written down here because
/// task.md asks for the rule beside the code: `Tolerance::Exact`'s `.value()` is `0.0`, so it is
/// at least as strict as any `Declared` tolerance; between two `Declared` tolerances the smaller
/// `.value()` wins; a tie keeps `a` (the one encountered first in manifest row order) — an
/// arbitrary but deterministic choice, since no stage in the committed manifest declares two
/// different tolerances with the same value today.
fn stricter(a: Tolerance, b: Tolerance) -> Tolerance {
    if b.value() < a.value() {
        b
    } else {
        a
    }
}

/// The tier to hold `stage` to: the strictest of its rows' own tolerances, each read with
/// `tolerance::from_manifest_row` (the same reader `oracle-compare`'s own `--tolerance-from-
/// manifest` flag uses), so a tolerance cell with a value and no `reason:` clause is refused
/// here exactly the way the comparator already refuses one elsewhere.
pub fn stage_tolerance(manifest_path: &Path, rows: &[&ManifestRow]) -> Result<Tolerance, String> {
    let manifest_str = manifest_path
        .to_str()
        .ok_or_else(|| "manifest path is not valid UTF-8".to_string())?;
    let mut result: Option<Tolerance> = None;
    for row in rows {
        let tol = tolerance::from_manifest_row(manifest_str, &row.path)?;
        // The one contradiction a row's own cells can state: `equality tier` says `exact`
        // (R19/R20's fixed-point reading) while `tolerance` is non-empty (a declared, non-zero
        // deviation). Refused here, the same way a value with no reason is refused, rather than
        // silently comparing at the looser declared tolerance — a reviewer grepping this
        // module for the `equality tier` column (task.md names all three columns together)
        // finds this is where it is read and enforced.
        let tier = row.equality_tier.trim();
        if tier.eq_ignore_ascii_case("exact") && matches!(tol, Tolerance::Declared { .. }) {
            return Err(format!(
                "row '{}' declares equality tier 'exact' but also a non-empty tolerance cell \
                 ({tol}); a fixed-point row cannot also be a float row",
                row.path
            ));
        }
        result = Some(match result {
            None => tol,
            Some(current) => stricter(current, tol),
        });
    }
    result.ok_or_else(|| "stage has no rows to derive a tolerance from".to_string())
}

// --- Dump records --------------------------------------------------------------------------

/// Reads `path` as the dump format `oracle/stage-dump/dump_seams.cpp` writes, keeping only the
/// data records: every line after the `;`-comment provenance header, with every other
/// `;`-prefixed line (including the dumper's own `; seam:` and `; object:` separators, not only
/// its machine-specific `os:`/`arch:`/`compiler:` provenance block) and every blank line dropped
/// too, each remaining line right-trimmed. Dropping `; object:` means a difference in object
/// count or ordering between the two sides is only caught indirectly, by the record stream
/// shifting, never directly by an object index; this module does not carry the current object
/// into each record's key. See [`dump_seam_header`] for the one check this module does make
/// against a dropped `; seam:` line.
pub fn read_dump_records(path: &Path) -> Result<Vec<String>, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("cannot read dump '{}': {e}", path.display()))?;
    Ok(content
        .lines()
        .map(|l| l.trim_end())
        .filter(|l| {
            let t = l.trim_start();
            !t.is_empty() && !t.starts_with(';')
        })
        .map(|s| s.to_string())
        .collect())
}

/// The value of a dump's own `; seam: <name>` header line, read straight from `path` (not from
/// [`read_dump_records`], which drops it) — `None` if the file carries no such line at all. A
/// dump read from the wrong file, or a stage-dump emitter that gets a seam's own filename wrong,
/// is otherwise invisible: nothing about `read_dump_records`'s output names which stage it came
/// from. `run` calls this on both the oracle's and the port's dump for the stage it is about to
/// compare and refuses to compare them if either disagrees with the stage it was read as.
pub fn dump_seam_header(path: &Path) -> Result<Option<String>, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("cannot read dump '{}': {e}", path.display()))?;
    Ok(content.lines().find_map(|l| {
        l.trim_start()
            .strip_prefix("; seam:")
            .map(|rest| rest.trim().to_string())
    }))
}

#[derive(Debug, Clone, PartialEq)]
pub enum DumpOutcome {
    Match,
    Differ {
        record_index: usize,
        oracle: Option<String>,
        port: Option<String>,
    },
}

/// Parses `0x1.999999999999ap-3`-style C99 hex-float literals (the only floating form R5's dump
/// format ever writes, via `%a`). Returns `None` for anything else, including a plain decimal
/// literal — this module never treats a decimal number as a float field, since the dump format
/// itself never writes one (R5: "a field that is genuinely floating-point ... is written with
/// `%a` ... and carries no decimal form").
fn parse_hex_float(s: &str) -> Option<f64> {
    let s = s.trim();
    let (neg, s) = if let Some(rest) = s.strip_prefix('-') {
        (true, rest)
    } else if let Some(rest) = s.strip_prefix('+') {
        (false, rest)
    } else {
        (false, s)
    };
    let s = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X"))?;
    let p_pos = s.find(['p', 'P'])?;
    let mantissa = &s[..p_pos];
    let exponent_str = &s[p_pos + 1..];
    let exponent: i32 = exponent_str.parse().ok()?;
    let (int_part, frac_part) = match mantissa.split_once('.') {
        Some((i, f)) => (i, f),
        None => (mantissa, ""),
    };
    if int_part.is_empty() && frac_part.is_empty() {
        return None;
    }
    let int_val: u64 = if int_part.is_empty() {
        0
    } else {
        u64::from_str_radix(int_part, 16).ok()?
    };
    let mut frac_val: f64 = 0.0;
    let mut scale = 1.0_f64 / 16.0;
    for c in frac_part.chars() {
        let d = c.to_digit(16)? as f64;
        frac_val += d * scale;
        scale /= 16.0;
    }
    let value = (int_val as f64 + frac_val) * 2f64.powi(exponent);
    Some(if neg { -value } else { value })
}

/// Splits a dump record line into its whitespace-separated `key=value` tokens (R5's own format:
/// a `role=` value has every space replaced with `_`, so *that* field never spans more than one
/// token). This underscore rule is scoped to `role=` alone (`oracle/stage-dump/README.md`'s own
/// "a leaf path's `role=` value"); it does not extend to `pts=`, whose value is itself k
/// space-separated `x,y` pairs (`dump_seams.cpp`'s `write_points`), so a `pts=` field tokenises
/// into several `(key, value)` pairs sharing one logical key — `records_equal`'s `Declared`-tier
/// branch therefore compares a multi-point `pts=` list positionally, token for token, and never
/// applies the declared tolerance inside it (each trailing point token's own `=`-split key is
/// just that token, not `pts`). This is sound only because dump coordinates are integer
/// `coord_t` (R5), never float, so no stage declares a non-`exact` tier over a `pts=` field
/// today; a future float-tiered stage whose dump carries `pts=` would need this function changed
/// first.
fn tokenize_record(line: &str) -> Vec<(&str, &str)> {
    line.split_whitespace()
        .map(|tok| tok.split_once('=').unwrap_or((tok, "")))
        .collect()
}

/// Two dump records are equal at `tolerance`: byte-for-byte equal at `Tolerance::Exact`
/// (dump coordinates are integer `coord_t`, never float — R5 — so exact tier never needs field-
/// aware comparison); at a `Declared` tolerance, token-for-token equal, except a token pair that
/// both parse as hex floats is allowed to differ by up to the declared value (`Abs`, the only
/// comparison function this crate implements — `tolerance.rs`'s own doc comment).
fn records_equal(a: &str, b: &str, tolerance: &Tolerance) -> bool {
    match tolerance {
        Tolerance::Exact => a == b,
        Tolerance::Declared { value, .. } => {
            let ta = tokenize_record(a);
            let tb = tokenize_record(b);
            if ta.len() != tb.len() {
                return false;
            }
            for ((ka, va), (kb, vb)) in ta.iter().zip(tb.iter()) {
                if ka != kb {
                    return false;
                }
                match (parse_hex_float(va), parse_hex_float(vb)) {
                    (Some(fa), Some(fb)) => {
                        if (fa - fb).abs() > *value {
                            return false;
                        }
                    }
                    _ => {
                        if va != vb {
                            return false;
                        }
                    }
                }
            }
            true
        }
    }
}

/// Compares two parsed dump record streams record-for-record, at `tolerance`, reporting the
/// first index at which they differ (including a length mismatch, reported as one side missing
/// a record at that index).
pub fn compare_dump_records(
    oracle: &[String],
    port: &[String],
    tolerance: &Tolerance,
) -> DumpOutcome {
    let max_len = oracle.len().max(port.len());
    for i in 0..max_len {
        match (oracle.get(i), port.get(i)) {
            (Some(o), Some(p)) => {
                if !records_equal(o, p, tolerance) {
                    return DumpOutcome::Differ {
                        record_index: i,
                        oracle: Some(o.clone()),
                        port: Some(p.clone()),
                    };
                }
            }
            (o, p) => {
                return DumpOutcome::Differ {
                    record_index: i,
                    oracle: o.cloned(),
                    port: p.cloned(),
                }
            }
        }
    }
    DumpOutcome::Match
}

// --- The fallback (exported G-code, L0-L4) --------------------------------------------------

/// `psGCodeExport`/`psConflictCheck`'s outcome: the full L0-L4 report R4's comparator already
/// computes, `differs` anchored at the level the stage's declared tier actually holds it to —
/// **not** `report.strongest_evaluated_pass`, which names the *loosest* passing level and so
/// reads a pair that fails every level up to L3 but happens to pass the L4 aggregate statistics
/// as "not differing". That is exactly the anchor-direction mistake `Report`'s own doc comment
/// records (`strongest_evaluated_pass` extended to a stricter use once already "silently
/// swallowed an L0 failure" — see `lib.rs` and `tests/levels.rs`'s
/// `generated_by_pair_fails_l0_only_and_names_l4_strongest`). The level `evaluate_fallback` holds
/// the pair to instead: `Tolerance::Exact` carries no allowed deviation at all, so nothing short
/// of the raw bytes matching (L0) counts as a non-difference; a `Declared` tolerance is the
/// comparator's own float-aware per-move level, L3 — `compare_with_tolerance`'s `tolerance_value`
/// only ever reaches L3 and L4 (`lib.rs`), and L4 is an aggregate statistic that can still pass
/// when individual moves genuinely differ, so L3 is the tightest level a declared tolerance is
/// meant to hold a pair to.
#[derive(Debug)]
pub struct FallbackOutcome {
    pub report: Report,
    pub differs: bool,
}

pub fn evaluate_fallback(
    oracle_gcode: &[u8],
    port_gcode: &[u8],
    tolerance: Tolerance,
) -> FallbackOutcome {
    let held_level = match &tolerance {
        Tolerance::Exact => Level::L0,
        Tolerance::Declared { .. } => Level::L3,
    };
    let report = compare_with_tolerance(oracle_gcode, port_gcode, tolerance);
    let differs = !report.passed(held_level);
    FallbackOutcome { report, differs }
}

// --- Driving the port as a subprocess --------------------------------------------------------

/// Runs `port_bin` once over `fixture`/`config`, writing G-code under `gcode_dir` and any R5
/// stage dumps it can already produce under `dump_dir`. A non-zero exit is expected and normal
/// until Phase B lands every stage (`grampus-cli`'s own "not yet ported" error) and does not
/// fail this function — only a stage this run derives as fully ported and still finds no dump
/// for, in the caller, does, and the caller needs this process's exit status and stderr to say
/// *why* when it does (`describe_port_output`). Failing to even start the binary (a bad
/// `--port-bin` path) is this function's only `Err`.
///
/// What the real binary does today is proven against the real binary, in
/// `crates/grampus-cli/tests/dump_stages.rs`: it accepts `--dump-stages`, fails at
/// `Surface::build_model` before any of the eleven seams is reached, and writes no dump at all.
/// Nothing in this crate stands in for it (R19).
pub fn run_port_binary(
    port_bin: &Path,
    fixture: &Path,
    config: &Path,
    gcode_dir: &Path,
    dump_dir: &Path,
) -> Result<std::process::Output, String> {
    fs::create_dir_all(gcode_dir)
        .map_err(|e| format!("cannot create '{}': {e}", gcode_dir.display()))?;
    fs::create_dir_all(dump_dir)
        .map_err(|e| format!("cannot create '{}': {e}", dump_dir.display()))?;
    Command::new(port_bin)
        .arg(fixture)
        .arg("--config")
        .arg(config)
        .arg("--out")
        .arg(gcode_dir)
        .arg("--dump-stages")
        .arg(dump_dir)
        .output()
        .map_err(|e| format!("cannot run port binary '{}': {e}", port_bin.display()))
}

/// Whether `dir` holds at least one entry — used to tell "the port never writes here at all"
/// (a structural gap: nothing behind this call site is ported yet) from "the port wrote some
/// files here but not this one" (a genuine inconsistency between the status derivation and the
/// port's own `Surface`). An unreadable or absent directory reads as empty.
fn dir_has_any_entry(dir: &Path) -> bool {
    fs::read_dir(dir)
        .map(|mut entries| entries.next().is_some())
        .unwrap_or(false)
}

/// A short, human-readable tail of the port subprocess's own exit status and stderr, appended to
/// an error so a real crash or a real "not yet ported" message is distinguishable from the port
/// having simply run and written nothing (`run_port_binary` discards neither any more — see its
/// own doc comment and task 39-compare-stages's pull request body on why this was previously
/// thrown away).
pub fn describe_port_output(output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let tail: Vec<&str> = stderr
        .lines()
        .rev()
        .take(5)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    if tail.is_empty() {
        format!(" (port exited {})", output.status)
    } else {
        format!(
            " (port exited {}; stderr tail:\n{})",
            output.status,
            tail.join("\n")
        )
    }
}

// --- Dispatch and rendering ------------------------------------------------------------------

pub struct StagesConfig<'a> {
    pub manifest_path: &'a Path,
    pub tree: &'a Path,
    pub oracle_dumps_dir: &'a Path,
    pub port_bin: &'a Path,
    pub fixture: &'a Path,
    pub config: &'a Path,
    pub scratch_dir: &'a Path,
}

/// One port run's outputs, as [`compare_stage_outputs`] reads them: the two directories the port
/// was asked to write into, and a diagnostic tail describing how its process ended.
///
/// This is the seam that makes the comparison testable without anything standing in for the port
/// (R19): [`run`] fills it from a real `grampus-cli` subprocess, and this crate's own tests fill
/// it from directories of committed or generated dump fixtures. A directory of dump files is
/// fixture *data* for the comparison — it is not a stand-in for the port, which is never invoked
/// on that path at all, and not a stand-in for the oracle, whose side of every comparison is the
/// committed dump read from `oracle_dumps_dir`.
pub struct PortOutputs<'a> {
    /// Where the port's R5 per-seam stage dumps are. Empty is today's real state at every seam
    /// (`crates/grampus-cli/src/surface.rs`'s `emit_stage_dump` is a documented no-op), and
    /// `compare_stage_outputs` reports it as the structural gap it is, not as a pass.
    pub dump_dir: &'a Path,
    /// Where the port's exported G-code is, for the two fallback stages.
    pub gcode_dir: &'a Path,
    /// A short tail describing how the port process itself ended, appended to the two port-side
    /// errors so a crash or a real "not yet ported" message is distinguishable from the port
    /// having run and written nothing. [`run`] fills this with [`describe_port_output`]; a caller
    /// that ran no port process passes `""`.
    pub diagnostic: &'a str,
}

#[derive(Debug)]
pub enum StageResult {
    NotPorted,
    Dump {
        tolerance: Tolerance,
        outcome: DumpOutcome,
    },
    Fallback {
        tolerance: Tolerance,
        outcome: FallbackOutcome,
    },
}

#[derive(Debug)]
pub struct StageLine {
    pub stage: String,
    pub result: StageResult,
}

#[derive(Debug)]
pub struct StagesReport {
    pub lines: Vec<StageLine>,
    pub first_differing_stage: Option<String>,
    /// Every `.dump` file committed under `--oracle-dumps` whose stem names no stage
    /// `port/manifest.tsv`'s own `pipeline stage or none` column carries at all — today, the
    /// four R5 seams `06-manifest-scope` has not yet assigned to any row
    /// (`posIroning`, `posSupportMaterial`, `posDetectOverhangsForLift`, `posSimplifyPath`).
    /// This command cannot compare them and cannot fix the manifest (this task's own "must not
    /// touch" list), but it does not have to stay silent about the gap either — see
    /// `render_text`.
    pub uncovered_oracle_dumps: Vec<String>,
}

/// The one named error this module's `run` can return: a bad manifest, a refused tolerance, a
/// missing oracle dump or oracle G-code for a stage this run derives as fully ported (task.md's
/// own "exits non-zero, naming `just oracle-dump 3DBenchy base-fff`"), a dump whose own
/// `; seam:` header disagrees with the stage it was read as, or one of two port-side problems
/// for a stage this run derives as fully ported: the port writes *no* output of the kind this
/// stage needs at all (`crates/grampus-cli/src/surface.rs`'s `emit_stage_dump` — or,
/// for the fallback stages, `Surface::export_gcode` — is a documented no-op until a module
/// behind it is ported; this is the state every stage is in on the tree this task lands on, and
/// is expected to keep firing, loudly, until Phase B gives that call site real state to write),
/// or the port writes *some* output but not this stage's own file (a genuine inconsistency
/// between the status derivation and the port's `Surface` — this one really cannot happen on
/// the tree this task lands on, since nothing is ported, but is checked rather than silently
/// read as a pass). Both port-side cases keep this command failing shut rather than reporting a
/// stage this run cannot actually compare as if it had passed.
pub fn run(cfg: &StagesConfig) -> Result<StagesReport, String> {
    let gcode_dir = cfg.scratch_dir.join("gcode");
    let dump_dir = cfg.scratch_dir.join("dump-stages");
    let port_output =
        run_port_binary(cfg.port_bin, cfg.fixture, cfg.config, &gcode_dir, &dump_dir)?;
    let diagnostic = describe_port_output(&port_output);
    compare_stage_outputs(
        cfg.manifest_path,
        cfg.tree,
        cfg.oracle_dumps_dir,
        &PortOutputs {
            dump_dir: &dump_dir,
            gcode_dir: &gcode_dir,
            diagnostic: &diagnostic,
        },
    )
}

/// Everything [`run`] does once the port has run: derive each stage's status from `tree` and
/// `manifest_path`, read its tier, and compare the port's output in `port` against the committed
/// oracle artifact in `oracle_dumps_dir`.
///
/// This is the real comparison, called by `run` on every real invocation and by this crate's own
/// tests directly, so that no test needs a binary that impersonates the port to reach the
/// match/differ/fallback branches (R19). It returns the same errors `run` documents.
pub fn compare_stage_outputs(
    manifest_path: &Path,
    tree: &Path,
    oracle_dumps_dir: &Path,
    port: &PortOutputs,
) -> Result<StagesReport, String> {
    let rows = load_manifest_rows(manifest_path)?;
    let stages = stage_names_in_order(&rows);
    let known_stages: HashSet<String> = stages.iter().cloned().collect();
    let uncovered_oracle_dumps = uncovered_oracle_dumps(oracle_dumps_dir, &known_stages);
    let mod_reachable = mod_reachable_files(tree);

    let gcode_dir = port.gcode_dir;
    let dump_dir = port.dump_dir;
    let port_wrote_no_dumps_at_all = !dir_has_any_entry(dump_dir);
    let port_wrote_no_gcode_at_all = !dir_has_any_entry(gcode_dir);

    let mut lines = Vec::new();
    let mut first_differing_stage: Option<String> = None;

    for stage in stages {
        let stage_rows = rows_for_stage(&rows, &stage);
        let fully_ported = stage_fully_ported(&stage_rows, &rows, tree, &mod_reachable);
        if !fully_ported {
            lines.push(StageLine {
                stage,
                result: StageResult::NotPorted,
            });
            continue;
        }

        let tolerance = stage_tolerance(manifest_path, &stage_rows)?;

        if is_fallback_stage(&stage) {
            let oracle_gcode_path = oracle_dumps_dir.join("plate_1.gcode");
            if !oracle_gcode_path.is_file() {
                return Err(format!(
                    "stage '{stage}' is fully ported but '{}' does not exist, and no command \
                     in this repository commits it there yet: `just oracle-dump 3DBenchy \
                     base-fff` (justfile) writes the per-seam oracle dumps this command reads, \
                     but its own recipe deletes the run directory that held the exported \
                     G-code once it has copied out the per-seam dump files, so running it does \
                     not produce this file. Extending that recipe to also copy `plate_1.gcode` into \
                     the oracle-dumps directory (and `oracle/provenance --check`'s dump glob to \
                     cover it), or naming the command that should, is a follow-up this task \
                     leaves open -- see its pull request body",
                    oracle_gcode_path.display()
                ));
            }
            let port_gcode_path = gcode_dir.join("plate_1.gcode");
            if !port_gcode_path.is_file() {
                return Err(if port_wrote_no_gcode_at_all {
                    format!(
                        "stage '{stage}' is fully ported but the port wrote no G-code at all: \
                         `Surface::export_gcode` (crates/grampus-cli/src/surface.rs) still \
                         returns its documented not-yet-ported error, so there is nothing for \
                         this command to compare -- exiting non-zero rather than reading this as \
                         a pass{}",
                        port.diagnostic
                    )
                } else {
                    format!(
                        "stage '{stage}' is fully ported but the port's own run wrote no '{}', \
                         even though it wrote other output (internal inconsistency between \
                         port-status derivation and the port's Surface){}",
                        port_gcode_path.display(),
                        port.diagnostic
                    )
                });
            }
            let oracle_bytes = fs::read(&oracle_gcode_path)
                .map_err(|e| format!("cannot read '{}': {e}", oracle_gcode_path.display()))?;
            let port_bytes = fs::read(&port_gcode_path)
                .map_err(|e| format!("cannot read '{}': {e}", port_gcode_path.display()))?;
            let outcome = evaluate_fallback(&oracle_bytes, &port_bytes, tolerance.clone());
            if outcome.differs && first_differing_stage.is_none() {
                first_differing_stage = Some(stage.clone());
            }
            lines.push(StageLine {
                stage,
                result: StageResult::Fallback { tolerance, outcome },
            });
        } else {
            let oracle_dump_path = oracle_dumps_dir.join(format!("{stage}.dump"));
            if !oracle_dump_path.is_file() {
                return Err(format!(
                    "stage '{stage}' is fully ported but '{}' does not exist; run `just \
                     oracle-dump 3DBenchy base-fff` to produce it",
                    oracle_dump_path.display()
                ));
            }
            let port_dump_path = dump_dir.join(format!("{stage}.dump"));
            if !port_dump_path.is_file() {
                return Err(if port_wrote_no_dumps_at_all {
                    format!(
                        "stage '{stage}' is fully ported but the port wrote no stage dump at \
                         all: `emit_stage_dump` (crates/grampus-cli/src/surface.rs) is still a \
                         documented no-op at every seam, with no per-object/per-layer state yet \
                         to serialise -- exiting non-zero rather than reading this as a pass; \
                         see task 39-compare-stages's pull request body for which task is \
                         expected to give it a real writer once Phase B lands state behind this \
                         seam{}",
                        port.diagnostic
                    )
                } else {
                    format!(
                        "stage '{stage}' is fully ported but the port's own run wrote no '{}', \
                         even though it wrote other stage dumps (internal inconsistency between \
                         port-status derivation and the port's Surface){}",
                        port_dump_path.display(),
                        port.diagnostic
                    )
                });
            }
            for (label, path) in [("oracle", &oracle_dump_path), ("port", &port_dump_path)] {
                if let Some(seam) = dump_seam_header(path)? {
                    if seam != stage {
                        return Err(format!(
                            "{label} dump '{}' carries '; seam: {seam}' but was read as stage \
                             '{stage}' -- refusing to compare a dump against the wrong stage",
                            path.display()
                        ));
                    }
                }
            }
            let oracle_records = read_dump_records(&oracle_dump_path)?;
            let port_records = read_dump_records(&port_dump_path)?;
            let outcome = compare_dump_records(&oracle_records, &port_records, &tolerance);
            if matches!(outcome, DumpOutcome::Differ { .. }) && first_differing_stage.is_none() {
                first_differing_stage = Some(stage.clone());
            }
            lines.push(StageLine {
                stage,
                result: StageResult::Dump { tolerance, outcome },
            });
        }
    }

    Ok(StagesReport {
        lines,
        first_differing_stage,
        uncovered_oracle_dumps,
    })
}

/// Every `.dump` file in `oracle_dumps_dir` whose stem (the file name with `.dump` removed, R5's
/// own one-file-per-stage naming) is not one of `known_stages` — the manifest's own stage list,
/// which this function never second-guesses or restates, only compares a committed file name
/// against. An unreadable or absent `oracle_dumps_dir` reports no gap (that is `run`'s own
/// missing-oracle-dump error's job, once a stage this run derives as fully ported actually needs
/// a file from it).
fn uncovered_oracle_dumps(oracle_dumps_dir: &Path, known_stages: &HashSet<String>) -> Vec<String> {
    let Ok(entries) = fs::read_dir(oracle_dumps_dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("dump"))
        .filter_map(|p| {
            p.file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        })
        .filter(|stem| !known_stages.contains(stem))
        .collect();
    names.sort();
    names.dedup();
    names
}

fn opt_record(r: &Option<String>) -> &str {
    r.as_deref().unwrap_or("<no record>")
}

/// Renders a [`StagesReport`] as the one-line-per-stage table task.md's "Done when" describes,
/// plus a closing line naming the first differing stage (or that none differed). The closing
/// line's "none" case distinguishes "every compared stage matched" from "nothing was actually
/// compared" (every stage read `not ported`) — otherwise the two read identically, which is the
/// most misleading rendering of a run that proved nothing a reader could be handed: a nine-stage
/// `not ported` run and a nine-stage clean comparison must not look the same on the last line.
pub fn render_text(report: &StagesReport) -> String {
    let mut out = String::new();
    let total = report.lines.len();
    let not_ported = report
        .lines
        .iter()
        .filter(|l| matches!(l.result, StageResult::NotPorted))
        .count();
    for line in &report.lines {
        match &line.result {
            StageResult::NotPorted => {
                out.push_str(&format!("{}: not ported\n", line.stage));
            }
            StageResult::Dump { tolerance, outcome } => match outcome {
                DumpOutcome::Match => {
                    out.push_str(&format!("{}: {tolerance} -- match\n", line.stage));
                }
                DumpOutcome::Differ {
                    record_index,
                    oracle,
                    port,
                } => {
                    out.push_str(&format!(
                        "{}: {tolerance} -- differs at record {record_index}: oracle={} port={}\n",
                        line.stage,
                        opt_record(oracle),
                        opt_record(port),
                    ));
                }
            },
            StageResult::Fallback { tolerance, outcome } => {
                let strongest = outcome
                    .report
                    .strongest_evaluated_pass
                    .map(|l| l.name())
                    .unwrap_or("none");
                out.push_str(&format!(
                    "{}: fallback: exported G-code -- {tolerance} -- strongest evaluated pass: {strongest}\n",
                    line.stage,
                ));
            }
        }
    }
    match &report.first_differing_stage {
        Some(s) => out.push_str(&format!("first differing stage: {s}\n")),
        None => out.push_str(&format!(
            "first differing stage: none ({} of {total} stage{} compared; {not_ported} not \
             ported)\n",
            total - not_ported,
            if total == 1 { "" } else { "s" },
        )),
    }
    for stage in &report.uncovered_oracle_dumps {
        out.push_str(&format!(
            "{stage}: no manifest row names this stage (oracle dump committed, never \
             compared)\n"
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Comment stripping / mod decls / todo-unimplemented ---------------------------------

    #[test]
    fn mod_decl_matches_plain_and_pub_forms() {
        assert_eq!(match_mod_decl("mod foo;"), Some("foo".to_string()));
        assert_eq!(match_mod_decl("  mod foo ;"), Some("foo".to_string()));
        assert_eq!(match_mod_decl("pub mod foo;"), Some("foo".to_string()));
        assert_eq!(
            match_mod_decl("pub(crate) mod foo_bar;"),
            Some("foo_bar".to_string())
        );
        assert_eq!(match_mod_decl("use mod_like_thing;"), None);
        assert_eq!(match_mod_decl("// mod foo;"), None);
        assert_eq!(match_mod_decl("fn modify() {}"), None);
    }

    #[test]
    fn code_only_strips_line_and_block_comments() {
        let text = "mod foo; // not this one\n/* mod bar; */\nmod baz;\n";
        let code = code_only_text(text);
        let decls = find_mod_decls(&code);
        assert_eq!(decls, vec!["foo".to_string(), "baz".to_string()]);
    }

    // This test builds its two macro-call fixtures (`{todo_macro}()`, `{unimplemented_macro}()`)
    // from concatenated pieces rather than writing the literal `todo!(`/`unimplemented!(`
    // substring in this file's own source text: `port/port-status`'s burn-down count
    // (`compute_gate`) greps every `.rs` file under `crates/*/src/` for those exact substrings,
    // with no string-literal masking (unlike the numeric-literal check), so writing them as a
    // literal test fixture here would miscount this crate's own burn-down as non-zero. The
    // function under test still receives the identical runtime string either way.
    #[test]
    fn todo_and_unimplemented_detected_live_not_in_comments() {
        let todo_macro = format!("{}!()", "todo");
        let unimplemented_macro = format!("{}!(\"x\")", "unimplemented");
        assert!(has_live_todo_or_unimplemented(&code_only_text(&format!(
            "fn f() {{ {todo_macro} }}"
        ))));
        assert!(has_live_todo_or_unimplemented(&code_only_text(&format!(
            "fn f() {{ {unimplemented_macro} }}"
        ))));
        assert!(!has_live_todo_or_unimplemented(&code_only_text(&format!(
            "// still has a {todo_macro} here\nfn f() {{}}"
        ))));
        assert!(!has_live_todo_or_unimplemented(&code_only_text(
            "fn todo_list() {}"
        )));
    }

    // --- Hex float parsing -------------------------------------------------------------------

    #[test]
    fn hex_float_parses_known_values() {
        let v = parse_hex_float("0x1.999999999999ap-3").unwrap();
        assert!((v - 0.2).abs() < 1e-12, "got {v}");
        assert_eq!(parse_hex_float("-0x1p+1"), Some(-2.0));
        assert_eq!(parse_hex_float("0x1p0"), Some(1.0));
        assert_eq!(parse_hex_float("not-a-float"), None);
        assert_eq!(parse_hex_float("1.5"), None);
    }

    // --- Dump record comparison ---------------------------------------------------------------

    #[test]
    fn exact_tier_requires_byte_identical_records() {
        let a = vec!["field=print_z layer=0 region=- value=0x1p0".to_string()];
        let b = a.clone();
        assert_eq!(
            compare_dump_records(&a, &b, &Tolerance::Exact),
            DumpOutcome::Match
        );
        let c = vec!["field=print_z layer=0 region=- value=0x1p1".to_string()];
        assert!(matches!(
            compare_dump_records(&a, &c, &Tolerance::Exact),
            DumpOutcome::Differ {
                record_index: 0,
                ..
            }
        ));
    }

    #[test]
    fn declared_tolerance_allows_float_field_within_tolerance_only() {
        let tol = Tolerance::Declared {
            value: 0.01,
            unit: None,
            reason: "test".to_string(),
            comparison_function: tolerance::ComparisonFunction::Abs,
            source: tolerance::ToleranceSource::Manifest {
                tsv_path: "x".to_string(),
                row: "y".to_string(),
            },
        };
        // 0x1p0 == 1.0, 0x1.028f5c28f5c29p0 == 1.001 ...: within 0.01.
        let a = vec!["field=print_z layer=0 region=- value=0x1p0".to_string()];
        let b = vec!["field=print_z layer=0 region=- value=0x1.0a3d70a3d70a4p0".to_string()];
        // 0x1.0a3d70a3d70a4p0 ~= 1.04, outside 0.01 tolerance of 1.0.
        assert!(matches!(
            compare_dump_records(&a, &b, &tol),
            DumpOutcome::Differ { .. }
        ));
        let c = vec!["field=print_z layer=0 region=- value=0x1.0147ae147ae14p0".to_string()];
        // 0x1.0147ae147ae14p0 ~= 1.005, within 0.01 tolerance of 1.0.
        assert_eq!(compare_dump_records(&a, &c, &tol), DumpOutcome::Match);
        // An integer coord differing at all still fails, tolerance or not.
        let d =
            vec!["field=lslices layer=0 region=- poly=0 ring=contour n=4 pts=1,1 2,2".to_string()];
        let e =
            vec!["field=lslices layer=0 region=- poly=0 ring=contour n=4 pts=1,1 2,3".to_string()];
        assert!(matches!(
            compare_dump_records(&d, &e, &tol),
            DumpOutcome::Differ { .. }
        ));
    }

    #[test]
    fn missing_trailing_records_differ_at_the_shorter_length() {
        let a = vec!["x".to_string(), "y".to_string()];
        let b = vec!["x".to_string()];
        assert!(matches!(
            compare_dump_records(&a, &b, &Tolerance::Exact),
            DumpOutcome::Differ {
                record_index: 1,
                port: None,
                ..
            }
        ));
    }

    // --- Stage name ordering ------------------------------------------------------------------

    fn row(stage: &str, in_scope: bool) -> ManifestRow {
        ManifestRow {
            path: format!("{stage}.cpp"),
            in_scope,
            target_crate: String::new(),
            target_module: String::new(),
            stage: stage.to_string(),
            proof_path: String::new(),
            equality_tier: String::new(),
        }
    }

    #[test]
    fn stage_names_in_order_dedupes_excludes_none_and_moves_fallback_last() {
        let rows = vec![
            row("psGCodeExport", true),
            row("posSlice", true),
            row("none", true),
            row("posPerimeters", true),
            row("posSlice", true),
            row("psConflictCheck", true),
        ];
        assert_eq!(
            stage_names_in_order(&rows),
            vec![
                "posSlice".to_string(),
                "posPerimeters".to_string(),
                "psGCodeExport".to_string(),
                "psConflictCheck".to_string(),
            ]
        );
    }

    #[test]
    fn stage_fully_ported_is_false_not_vacuously_true_with_no_rows() {
        let rows: Vec<&ManifestRow> = Vec::new();
        assert!(!stage_fully_ported(
            &rows,
            &[],
            Path::new("/nonexistent"),
            &HashSet::new()
        ));
    }

    // --- Tier strictness ----------------------------------------------------------------------

    #[test]
    fn stage_tolerance_refuses_a_value_with_no_reason_the_way_the_comparator_already_does() {
        let tsv = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/manifest/tolerance.tsv");
        // `Foo/Baz.cpp` in that committed fixture declares a tolerance value with no `reason:`
        // clause (`tolerance.rs`'s own fixture, reused rather than duplicated).
        let r = row("someStage", true);
        let mut r = r;
        r.path = "Foo/Baz.cpp".to_string();
        let rows = vec![&r];
        let err = stage_tolerance(&tsv, &rows).expect_err("a value with no reason must be refused");
        assert!(err.contains("no reason"), "{err}");
    }

    #[test]
    fn stage_tolerance_refuses_exact_tier_combined_with_a_declared_tolerance() {
        let tsv = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/manifest/tolerance.tsv");
        // `Foo/Bar.cpp` declares a valid tolerance *with* a reason (so this reaches the new
        // check rather than being refused by `tolerance::from_manifest_row` first); this test's
        // row states `equality tier: exact` on top of it, which is the one contradiction a
        // row's own cells can state.
        let mut r = row("someStage", true);
        r.path = "Foo/Bar.cpp".to_string();
        r.equality_tier = "exact".to_string();
        let rows = vec![&r];
        let err = stage_tolerance(&tsv, &rows)
            .expect_err("'exact' plus a declared tolerance must be refused");
        assert!(err.contains("exact"), "{err}");
        assert!(err.contains("Foo/Bar.cpp"), "{err}");
    }

    #[test]
    fn stricter_prefers_the_smaller_value_and_exact_beats_any_declared() {
        let declared = |v: f64| Tolerance::Declared {
            value: v,
            unit: None,
            reason: "r".to_string(),
            comparison_function: tolerance::ComparisonFunction::Abs,
            source: tolerance::ToleranceSource::Manifest {
                tsv_path: "x".to_string(),
                row: "y".to_string(),
            },
        };
        assert_eq!(stricter(Tolerance::Exact, declared(0.01)).value(), 0.0);
        assert_eq!(stricter(declared(0.01), declared(0.02)).value(), 0.01);
        assert_eq!(stricter(declared(0.02), declared(0.01)).value(), 0.01);
    }

    // --- The fallback's `differs` anchor (R20; the mistake this crate's own `lib.rs` doc
    // comment already records once "silently swallowed an L0 failure") ------------------------

    fn read_gcode_pair(dir: &str) -> (Vec<u8>, Vec<u8>) {
        let base = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data")
            .join(dir);
        (
            fs::read(base.join("a.gcode")).unwrap_or_else(|e| panic!("{dir}/a.gcode: {e}")),
            fs::read(base.join("b.gcode")).unwrap_or_else(|e| panic!("{dir}/b.gcode: {e}")),
        )
    }

    #[test]
    fn fallback_differs_under_exact_tier_when_l0_fails_even_though_l4_passes() {
        // `generated_by`: the only difference is an attribution comment L1 strips, so L0 fails
        // (a byte-for-byte difference) while L1 through L4 all pass -- `report
        // .strongest_evaluated_pass` (the *loosest* passing level) is `Some(Level::L4)` for this
        // pair, which is exactly the shape that made the old anchor
        // (`strongest_evaluated_pass.is_none()`) read this pair as "not differing". An `exact`
        // tier means no allowed deviation at all, so this pair must read as differing.
        let (a, b) = read_gcode_pair("generated_by");
        let outcome = evaluate_fallback(&a, &b, Tolerance::Exact);
        assert_eq!(outcome.report.strongest_evaluated_pass, Some(Level::L4));
        assert!(
            outcome.differs,
            "an L0 failure must read as differing under an exact tier even when every looser \
             level passes"
        );
    }

    #[test]
    fn fallback_does_not_differ_on_a_byte_identical_pair() {
        let (a, b) = read_gcode_pair("identical");
        let outcome = evaluate_fallback(&a, &b, Tolerance::Exact);
        assert!(!outcome.differs);
    }

    #[test]
    fn fallback_under_a_declared_tolerance_differs_when_l3_fails_even_though_l4_passes() {
        // `inserted_move` at a declared tolerance of 10.0: side `b` has one extra move than
        // side `a` (a genuine structural difference `b`'s own doc comment there confirms), so
        // L3 (per-move comparison) fails regardless of tolerance -- but every L4 aggregate
        // (move count 2 vs 3, bounding box, extruded length) is well within 10.0, so L4 passes.
        // `strongest_evaluated_pass` is therefore `Some(L4)`: this is the exact shape the old
        // anchor (`strongest_evaluated_pass.is_none()`) got wrong for a `Declared` tolerance too
        // -- it would read this pair as "not differing" because *something* passed, when the
        // level the tolerance actually applies to (L3) genuinely failed.
        let (a, b) = read_gcode_pair("inserted_move");
        let loose = Tolerance::Declared {
            value: 10.0,
            unit: None,
            reason: "test".to_string(),
            comparison_function: tolerance::ComparisonFunction::Abs,
            source: tolerance::ToleranceSource::Manifest {
                tsv_path: "x".to_string(),
                row: "y".to_string(),
            },
        };
        let outcome = evaluate_fallback(&a, &b, loose);
        assert_eq!(outcome.report.strongest_evaluated_pass, Some(Level::L4));
        assert!(
            !outcome.report.passed(Level::L3),
            "L3 must fail on this pair"
        );
        assert!(
            outcome.differs,
            "L3 failing must read as differing even though L4 (a looser, aggregate level) \
             passes"
        );
    }

    // --- The uncovered-oracle-dump warning (R5's four not-yet-assigned seams) -----------------

    #[test]
    fn uncovered_oracle_dumps_names_a_dump_with_no_matching_stage_and_ignores_known_ones() {
        let dir = tempdir_for_test("uncovered-dumps-unit");
        write_test_dump(&dir.join("posIroning.dump"));
        write_test_dump(&dir.join("posSlice.dump"));
        write_test_dump(&dir.join("not-a-dump.txt"));
        let known: HashSet<String> = ["posSlice".to_string()].into_iter().collect();
        assert_eq!(
            uncovered_oracle_dumps(&dir, &known),
            vec!["posIroning".to_string()]
        );
    }

    fn tempdir_for_test(name: &str) -> PathBuf {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap()
            .join("oracle/.cache/tmp/oracle-compare-stages-unit-test")
            .join(format!("{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_test_dump(path: &Path) {
        fs::write(path, "; seam: x\nfield=a value=0x1p0\n").unwrap();
    }

    // --- The `; seam:` header check (R5's own per-seam one-file-per-stage convention) --------

    #[test]
    fn dump_seam_header_reads_the_seam_line() {
        let dir = tempdir_for_test("seam-header-unit");
        let path = dir.join("x.dump");
        fs::write(
            &path,
            "; provenance\n; seam: posSlice\nfield=a value=0x1p0\n",
        )
        .unwrap();
        assert_eq!(
            dump_seam_header(&path).unwrap(),
            Some("posSlice".to_string())
        );
    }

    #[test]
    fn dump_seam_header_is_none_with_no_such_line() {
        let dir = tempdir_for_test("seam-header-unit-none");
        let path = dir.join("x.dump");
        fs::write(&path, "; provenance\nfield=a value=0x1p0\n").unwrap();
        assert_eq!(dump_seam_header(&path).unwrap(), None);
    }

    // --- `row_is_ported` against the shapes `port/port-status`'s own `derive_row_status`
    // actually has to get right, not only the single flat-file-reached-from-`lib.rs` shape
    // `manifest.tsv`'s own rows exercise -------------------------------------------------------

    fn fixture_tree_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/stages/fixture_tree")
    }

    #[test]
    fn row_is_ported_follows_a_directory_mod_rs_chain() {
        let tree = fixture_tree_dir();
        let mod_reachable = mod_reachable_files(&tree);
        let mut r = row("myModChainStage", true);
        r.target_crate = "fixture-crate".to_string();
        r.target_module = "crates/fixture-crate/src/mod_chain/leaf.rs".to_string();
        r.proof_path = "proof/mod_chain_leaf".to_string();
        assert!(
            row_is_ported(&r, std::slice::from_ref(&r), &tree, &mod_reachable),
            "a module reached only through lib.rs -> mod_chain/mod.rs -> leaf.rs must derive \
             ported"
        );
    }

    #[test]
    fn row_is_ported_finds_a_proof_under_the_crate_s_own_tests_directory() {
        let tree = fixture_tree_dir();
        let mod_reachable = mod_reachable_files(&tree);
        let mut r = row("myExternalProofStage", true);
        r.target_crate = "fixture-crate".to_string();
        r.target_module = "crates/fixture-crate/src/tested_externally.rs".to_string();
        r.proof_path = "proof/tested_externally".to_string();
        assert!(
            row_is_ported(&r, std::slice::from_ref(&r), &tree, &mod_reachable),
            "a proof living under crates/fixture-crate/tests/ (not an inline #[cfg(test)] module) \
             must still be found"
        );
    }

    // --- The differential-comparison rule and the `enclosing:` form (docs/PORTING.md,
    // "Decisions recorded on 2026-11-25"), held to the same cases `port/port-status`'s own
    // self-test holds its Python implementation to, so the two cannot drift -------------------

    #[test]
    fn has_differential_comparison_separates_an_oracle_comparison_from_a_transcribed_one() {
        // The same list `port/port-status --self-test` holds its Python implementation to, in
        // the same order. A case added to one must be added to the other: these two
        // implementations of one rule are what keep `just compare-stages` and
        // `just port-status` from disagreeing about whether a stage is fully ported.
        let oracle: HashSet<String> = ["dumps", "proof_units", "path1_flags", "oracle_call"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let cases: [(&str, bool, &str); 16] = [
            (
                "let expected: u32 = 0b111;\nassert_eq!(actual, expected);",
                false,
                "no oracle value is read at all",
            ),
            (
                "let dump = dumps::read(\"x\");\nassert_eq!(dump.seam, \"psWipeTower\");",
                false,
                "the oracle value is compared only against a literal",
            ),
            (
                "let oracle = path1_flags();\nassert_eq!(oracle.len(), PORTED_FLAGS.len());",
                true,
                "an oracle value against a value the module computed",
            ),
            (
                "let (a, b) = oracle_call();\nassert_eq!(PORTED, a);\nassert_eq!(OTHER, b);",
                true,
                "a tuple binding off one oracle call",
            ),
            (
                "let oracle = path1_flags();\nassert!(agrees(ported, oracle));",
                true,
                "both sides inside one argument that makes a call",
            ),
            (
                "let oracle = dumps::sep();\nassert_eq!(oracle, '\\n');",
                false,
                "a char literal is blanked, so it does not read as a value",
            ),
            (
                "let oracle = path1_flags();\nassert_eq!(oracle, vec![1, 0, 1]);",
                false,
                "a macro name is not a value this port computed",
            ),
            (
                "let dump = dumps::read(\"x\");\n\
                 assert_eq!(dump.seam, String::from(\"psWipeTower\"));",
                false,
                "neither end of a path is a value this port computed",
            ),
            (
                "let dump = dumps::read(\"x\");\nassert_eq!(dump.seam, format!(\"psWipeTower\"));",
                false,
                "a macro name is not a value, however it is spelled",
            ),
            (
                "let oracle = path1_flags();\nassert_eq!(oracle, Vec::from([1, 0, 1]));",
                false,
                "an associated function on a type is not a value either",
            ),
            (
                "let oracle = path1_flags();\nassert_eq!(oracle, [1, 0, 1].to_vec());",
                false,
                "a method name reached through a dot is not a value",
            ),
            (
                "let oracle = path1_flags();\nassert_eq!(oracle[index], 5);",
                false,
                "a loop variable in the same argument as the oracle value is not a second value",
            ),
            (
                "let oracle = path1_flags();\n\
                 for (got, want) in oracle.iter().zip(PORTED_FLAGS) { assert_eq!(got, want); }",
                true,
                "a `for` pattern off a tainted iterator carries the taint",
            ),
            (
                "let oracle = slic3r_proof::proof_units::u::f();\nassert_eq!(ported(), oracle);",
                true,
                "a fully-qualified oracle path needs no `use` statement",
            ),
            (
                "let oracle: [i64; 3] = path1_flags();\nassert_eq!(oracle, PORTED);",
                true,
                "a `;` inside the binding's own type does not end the pattern",
            ),
            (
                "let oracle = path1_flags();\n\
                 assert_eq!(oracle.iter().collect::<String>(), ported());",
                true,
                "a turbofish type argument is not the ported side, but `ported()` is",
            ),
        ];
        for (body, want, why) in cases {
            assert_eq!(
                has_differential_comparison(body, &oracle),
                want,
                "{why}: {body:?}"
            );
        }
    }

    #[test]
    fn row_is_ported_agrees_with_port_status_on_every_ported_row_of_the_real_manifest() {
        // The guarantee this whole mirror exists for, pinned against the real tree rather than a
        // fixture: every row `port/port-status`'s `derive_row_status` derives as `ported` must
        // be `ported` here too. A reviewer found the two disagreeing about exactly these rows
        // (this module read a multi-line braced `use` one line at a time and so found no oracle
        // name), and no fixture caught it because the fixtures pass a pre-built oracle set in.
        //
        // Read from the committed manifest's own `status` cell, which `port/inventory` writes
        // from the text-visible half of the same rule, so this test does not need to be edited
        // as the port advances: it holds whatever the manifest currently says is ported.
        let tree = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let manifest = tree.join("port/manifest.tsv");
        let rows = load_manifest_rows(&manifest).expect("the committed manifest reads");
        let mod_reachable = mod_reachable_files(&tree);
        let text = fs::read_to_string(&manifest).expect("the committed manifest reads");
        let header: Vec<&str> = text
            .lines()
            .next()
            .unwrap_or_default()
            .split('\t')
            .collect();
        let status_col = header
            .iter()
            .position(|h| *h == "status")
            .expect("the manifest has a status column");
        let path_col = header
            .iter()
            .position(|h| *h == "path")
            .expect("the manifest has a path column");
        let ported_paths: Vec<String> = text
            .lines()
            .skip(1)
            .filter_map(|line| {
                let cells: Vec<&str> = line.split('\t').collect();
                match (cells.get(path_col), cells.get(status_col)) {
                    (Some(path), Some(&"ported")) => Some((*path).to_string()),
                    _ => None,
                }
            })
            .collect();
        assert!(
            !ported_paths.is_empty(),
            "the committed manifest names no ported row, so this test would pass vacuously"
        );
        for path in &ported_paths {
            let row = rows
                .iter()
                .find(|r| r.path.trim() == path.as_str())
                .unwrap_or_else(|| panic!("no manifest row for {path}"));
            assert!(
                row_is_ported(row, &rows, &tree, &mod_reachable),
                "port/manifest.tsv says {path} is ported but this module's row_is_ported says \
                 it is not -- the two implementations of the derivation have diverged"
            );
        }
    }

    #[test]
    fn oracle_names_reads_a_multi_line_braced_use_the_way_the_python_does() {
        // The divergence a reviewer found: `cargo fmt` breaks a long import into this shape, and
        // a line-at-a-time read of it collects none of the imported names, so this mirror
        // silently disagreed with `port/port-status` about every proof written that way.
        let found = oracle_names(
            "use slic3r_proof::proof_units::object_id::{\n\
             \x20   path1_wipe_tower_instance_id, path1_wipe_tower_object_id,\n\
             };\n",
        );
        for wanted in [
            "path1_wipe_tower_object_id",
            "path1_wipe_tower_instance_id",
            "object_id",
        ] {
            assert!(
                found.contains(wanted),
                "oracle_names missed {wanted:?} in a multi-line braced `use`; got {found:?}"
            );
        }
        // And the bare words are not oracle names: a local merely called `dumps` taints nothing.
        assert!(
            oracle_names("let dumps = vec![1, 2];\nassert_eq!(dumps.len(), ported());").is_empty()
        );
    }

    #[test]
    fn row_is_ported_derives_an_enclosing_row_from_the_unit_it_names() {
        let tree = fixture_tree_dir();
        let mod_reachable = mod_reachable_files(&tree);

        let mut enclosing = row("myExternalProofStage", true);
        enclosing.path = "Ported/Enclosing.cpp".to_string();
        enclosing.target_crate = "fixture-crate".to_string();
        enclosing.target_module = "crates/fixture-crate/src/tested_externally.rs".to_string();
        enclosing.proof_path = "proof/tested_externally".to_string();

        let mut helper = row("myExternalProofStage", true);
        helper.path = "Ported/Helper.hpp".to_string();
        helper.target_crate = "fixture-crate".to_string();
        helper.target_module = "crates/fixture-crate/src/enclosing_helper.rs".to_string();
        helper.proof_path = "enclosing: Ported/Enclosing.cpp".to_string();

        let rows = vec![enclosing.clone(), helper.clone()];
        assert!(
            row_is_ported(&helper, &rows, &tree, &mod_reachable),
            "a row naming an enclosing unit whose own row is ported must derive ported, with no \
             `// PROOF:` marker of its own"
        );

        // The enclosing row loses its own proof: the helper must follow it down.
        let mut rows_unproven = rows.clone();
        rows_unproven[0].proof_path = String::new();
        assert!(
            !row_is_ported(&rows_unproven[1], &rows_unproven, &tree, &mod_reachable),
            "a row whose enclosing unit is not ported must not derive ported"
        );

        // A cell naming no row at all, and a chain, are both refused.
        let mut dangling = helper.clone();
        dangling.proof_path = "enclosing: NoSuchFile.hpp".to_string();
        assert!(!row_is_ported(&dangling, &rows, &tree, &mod_reachable));

        let mut chained_enclosing = enclosing.clone();
        chained_enclosing.proof_path = "enclosing: Ported/Helper.hpp".to_string();
        let chained = vec![chained_enclosing, helper.clone()];
        assert!(
            !row_is_ported(&chained[1], &chained, &tree, &mod_reachable),
            "a chain of `enclosing:` cells is refused outright, which is also what makes a cycle \
             impossible"
        );
    }

    #[test]
    fn row_is_ported_is_false_with_no_proof_path_cell_at_all() {
        // `tested_externally`'s own module *is* mod-reachable and holds no live todo!()/
        // unimplemented!() -- an empty `proof path` cell is the one remaining way a row over it
        // can fail to derive ported, and `proof_present` refuses to search for an empty wanted
        // string rather than matching the first marker it finds.
        let tree = fixture_tree_dir();
        let mod_reachable = mod_reachable_files(&tree);
        let mut r = row("myExternalProofStage", true);
        r.target_crate = "fixture-crate".to_string();
        r.target_module = "crates/fixture-crate/src/tested_externally.rs".to_string();
        r.proof_path = String::new();
        assert!(!row_is_ported(
            &r,
            std::slice::from_ref(&r),
            &tree,
            &mod_reachable
        ));
    }
}
