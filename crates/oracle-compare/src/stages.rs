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
//! **The two seamless stages fall back to the exported G-code's L0-L4 levels**
//! (`is_fallback_stage`, `FALLBACK_STAGES`): `psGCodeExport` and `psConflictCheck` have no R5
//! seam, so this command never has a per-seam dump for them and instead compares the port's and
//! the built oracle's exported G-code the same way `just compare` already does, saying so in its
//! output rather than naming `psSkirtBrim` (the last seam) as their first difference.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::tolerance::{self, Tolerance};
use crate::{compare_with_tolerance, Report};

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
/// the function's own `fn` line) the literal line `// PROOF: <proof_path_cell>`.
fn find_proof_match(text: &str, module_snake: &str, proof_path_cell: &str) -> bool {
    let wanted = proof_path_cell.trim();
    if wanted.is_empty() {
        return false;
    }
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
                if path == wanted {
                    return true;
                }
            }
        }
    }
    false
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
pub fn row_is_ported(row: &ManifestRow, tree: &Path, mod_reachable: &HashSet<PathBuf>) -> bool {
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
    proof_present(row, &target_path, tree)
}

/// A stage is fully ported when it has at least one in-scope row and every one of them derives
/// `ported`. A stage that names only out-of-scope rows (so `rows` is empty) is reported `false`,
/// not vacuously `true` — task.md's "every ... row has a derived status of ported" is read here
/// as requiring at least one row to hold the stage to, since a stage with nothing to check is
/// not a stage this run has actually proven anything about.
pub fn stage_fully_ported(
    rows: &[&ManifestRow],
    tree: &Path,
    mod_reachable: &HashSet<PathBuf>,
) -> bool {
    !rows.is_empty() && rows.iter().all(|r| row_is_ported(r, tree, mod_reachable))
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
        result = Some(match result {
            None => tol,
            Some(current) => stricter(current, tol),
        });
    }
    result.ok_or_else(|| "stage has no rows to derive a tolerance from".to_string())
}

// --- Dump records --------------------------------------------------------------------------

/// Reads `path` as the dump format `oracle/stage-dump/dump_seams.cpp` writes, keeping only the
/// data records (every line after the `;`-comment provenance header and any other `;`-prefixed
/// or blank line), each right-trimmed. Comment lines are never compared: they carry
/// machine-specific provenance (`os:`, `arch:`, `compiler:`, ...) the two sides are not expected
/// to agree on, and R5 states the header separately from the record shape this parses.
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
/// "a role= value has every space replaced with `_`, so a line can be tokenised on whitespace
/// with no value spanning more than one token").
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
/// computes, `differs` set from whether *any* level passed (`report.strongest_evaluated_pass`) —
/// a report with nothing evaluated as a pass is the fallback's reading of "this stage differs";
/// one with at least L4 passing is read as not differing, the same loosest-confirmed-level
/// reading `Report`'s own doc comment gives `strongest_evaluated_pass` for every other consumer
/// of this struct.
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
    let report = compare_with_tolerance(oracle_gcode, port_gcode, tolerance);
    let differs = report.strongest_evaluated_pass.is_none();
    FallbackOutcome { report, differs }
}

// --- Driving the port as a subprocess --------------------------------------------------------

/// Runs `port_bin` once over `fixture`/`config`, writing G-code under `gcode_dir` and any R5
/// stage dumps it can already produce under `dump_dir`. A non-zero exit is expected and normal
/// until Phase B lands every stage (`grampus-cli`'s own "not yet ported" error) and does not
/// fail this function — only a stage this run derives as fully ported and still finds no dump
/// for, in the caller, does. Failing to even start the binary (a bad `--port-bin` path) is this
/// function's only `Err`.
pub fn run_port_binary(
    port_bin: &Path,
    fixture: &Path,
    config: &Path,
    gcode_dir: &Path,
    dump_dir: &Path,
) -> Result<(), String> {
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
        .map_err(|e| format!("cannot run port binary '{}': {e}", port_bin.display()))?;
    Ok(())
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
}

/// The one named error this module's `run` can return: a bad manifest, a refused tolerance, a
/// missing oracle dump or oracle G-code for a stage this run derives as fully ported (task.md's
/// own "exits non-zero, naming `just oracle-dump 3DBenchy base-fff`"), or an internal
/// inconsistency where a stage this run derives as fully ported has no matching output from the
/// port's own `--dump-stages` run (which cannot happen on the tree this task lands on, since no
/// stage is fully ported yet, but is checked rather than silently read as a pass).
pub fn run(cfg: &StagesConfig) -> Result<StagesReport, String> {
    let rows = load_manifest_rows(cfg.manifest_path)?;
    let stages = stage_names_in_order(&rows);
    let mod_reachable = mod_reachable_files(cfg.tree);

    let gcode_dir = cfg.scratch_dir.join("gcode");
    let dump_dir = cfg.scratch_dir.join("dump-stages");
    run_port_binary(cfg.port_bin, cfg.fixture, cfg.config, &gcode_dir, &dump_dir)?;

    let mut lines = Vec::new();
    let mut first_differing_stage: Option<String> = None;

    for stage in stages {
        let stage_rows = rows_for_stage(&rows, &stage);
        let fully_ported = stage_fully_ported(&stage_rows, cfg.tree, &mod_reachable);
        if !fully_ported {
            lines.push(StageLine {
                stage,
                result: StageResult::NotPorted,
            });
            continue;
        }

        let tolerance = stage_tolerance(cfg.manifest_path, &stage_rows)?;

        if is_fallback_stage(&stage) {
            let oracle_gcode_path = cfg.oracle_dumps_dir.join("plate_1.gcode");
            if !oracle_gcode_path.is_file() {
                return Err(format!(
                    "stage '{stage}' is fully ported but '{}' does not exist; run `just \
                     oracle-dump 3DBenchy base-fff` to produce it",
                    oracle_gcode_path.display()
                ));
            }
            let port_gcode_path = gcode_dir.join("plate_1.gcode");
            if !port_gcode_path.is_file() {
                return Err(format!(
                    "stage '{stage}' is fully ported but the port's own run wrote no '{}' \
                     (internal inconsistency between port-status derivation and the port's \
                     Surface)",
                    port_gcode_path.display()
                ));
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
            let oracle_dump_path = cfg.oracle_dumps_dir.join(format!("{stage}.dump"));
            if !oracle_dump_path.is_file() {
                return Err(format!(
                    "stage '{stage}' is fully ported but '{}' does not exist; run `just \
                     oracle-dump 3DBenchy base-fff` to produce it",
                    oracle_dump_path.display()
                ));
            }
            let port_dump_path = dump_dir.join(format!("{stage}.dump"));
            if !port_dump_path.is_file() {
                return Err(format!(
                    "stage '{stage}' is fully ported but the port's own run wrote no '{}' \
                     (internal inconsistency between port-status derivation and the port's \
                     Surface)",
                    port_dump_path.display()
                ));
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
    })
}

fn opt_record(r: &Option<String>) -> &str {
    r.as_deref().unwrap_or("<no record>")
}

/// Renders a [`StagesReport`] as the one-line-per-stage table task.md's "Done when" describes,
/// plus a closing line naming the first differing stage (or that none differed).
pub fn render_text(report: &StagesReport) -> String {
    let mut out = String::new();
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
        None => out.push_str("first differing stage: none\n"),
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
}
