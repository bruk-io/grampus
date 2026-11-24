//! Pins the comparator's declared normalisation to the vendored source it must match byte for
//! byte: `strip_nondeterministic_gcode_lines` in
//! `vendor/OrcaSlicer/tests/fff_print/test_slicing_pipeline_hook.cpp`. This test never skips —
//! a missing vendored file is a failure naming the path, because a skipped test is how this
//! check would quietly stop holding.

use std::collections::HashSet;
use std::path::PathBuf;

const RELATIVE_PATH: &str = "vendor/OrcaSlicer/tests/fff_print/test_slicing_pipeline_hook.cpp";
const FUNCTION_NAME: &str = "strip_nondeterministic_gcode_lines";

fn vendored_file_path() -> PathBuf {
    // CARGO_MANIFEST_DIR is crates/oracle-compare; the repository root is two levels up.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .join(RELATIVE_PATH)
}

/// Strips `//`-to-end-of-line comments and `/* ... */` block comments, without touching `//` or
/// `/*` that appear inside a string or character literal. Keeps newlines so line-based reasoning
/// downstream (none here, but harmless) stays intact.
fn strip_comments(src: &str) -> String {
    #[derive(Clone, Copy, PartialEq)]
    enum State {
        Normal,
        InString,
        InChar,
        InLineComment,
        InBlockComment,
    }

    let chars: Vec<char> = src.chars().collect();
    let mut out = String::with_capacity(src.len());
    let mut state = State::Normal;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        match state {
            State::Normal => match (c, next) {
                ('/', Some('/')) => {
                    state = State::InLineComment;
                    i += 2;
                    continue;
                }
                ('/', Some('*')) => {
                    state = State::InBlockComment;
                    i += 2;
                    continue;
                }
                ('"', _) => {
                    state = State::InString;
                    out.push(c);
                }
                ('\'', _) => {
                    state = State::InChar;
                    out.push(c);
                }
                _ => out.push(c),
            },
            State::InString => {
                out.push(c);
                if c == '\\' {
                    if let Some(escaped) = next {
                        out.push(escaped);
                        i += 2;
                        continue;
                    }
                } else if c == '"' {
                    state = State::Normal;
                }
            }
            State::InChar => {
                out.push(c);
                if c == '\\' {
                    if let Some(escaped) = next {
                        out.push(escaped);
                        i += 2;
                        continue;
                    }
                } else if c == '\'' {
                    state = State::Normal;
                }
            }
            State::InLineComment => {
                if c == '\n' {
                    state = State::Normal;
                    out.push(c);
                }
            }
            State::InBlockComment => {
                if c == '*' && next == Some('/') {
                    state = State::Normal;
                    i += 2;
                    continue;
                }
            }
        }
        i += 1;
    }
    out
}

/// Finds `fn_name`'s definition and returns the text strictly between its outermost `{` and the
/// matching `}`, counting braces only outside string/char literals and comments.
fn extract_function_body(src: &str, fn_name: &str) -> String {
    let name_idx = src
        .find(fn_name)
        .unwrap_or_else(|| panic!("{fn_name} not found in {}", vendored_file_path().display()));

    let after_name = &src[name_idx..];
    let open_offset = after_name
        .find('{')
        .unwrap_or_else(|| panic!("no opening brace found after {fn_name}"));
    let body_start = name_idx + open_offset + 1;

    let chars: Vec<char> = src.chars().collect();
    // Map byte index body_start to a char index for straightforward walking.
    let char_start = src[..body_start].chars().count();

    #[derive(Clone, Copy, PartialEq)]
    enum State {
        Normal,
        InString,
        InChar,
        InLineComment,
        InBlockComment,
    }

    let mut state = State::Normal;
    let mut depth: i32 = 1; // we are already inside the opening brace
    let mut i = char_start;
    let mut end_char = chars.len();
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        match state {
            State::Normal => match (c, next) {
                ('/', Some('/')) => {
                    state = State::InLineComment;
                    i += 2;
                    continue;
                }
                ('/', Some('*')) => {
                    state = State::InBlockComment;
                    i += 2;
                    continue;
                }
                ('"', _) => state = State::InString,
                ('\'', _) => state = State::InChar,
                ('{', _) => depth += 1,
                ('}', _) => {
                    depth -= 1;
                    if depth == 0 {
                        end_char = i;
                        break;
                    }
                }
                _ => {}
            },
            State::InString => {
                if c == '\\' {
                    i += 2;
                    continue;
                } else if c == '"' {
                    state = State::Normal;
                }
            }
            State::InChar => {
                if c == '\\' {
                    i += 2;
                    continue;
                } else if c == '\'' {
                    state = State::Normal;
                }
            }
            State::InLineComment => {
                if c == '\n' {
                    state = State::Normal;
                }
            }
            State::InBlockComment => {
                if c == '*' && next == Some('/') {
                    state = State::Normal;
                    i += 2;
                    continue;
                }
            }
        }
        i += 1;
    }
    assert!(
        depth == 0,
        "unbalanced braces while scanning {fn_name}'s body"
    );
    chars[char_start..end_char].iter().collect()
}

/// Extracts every `"..."` string literal's content (minimally unescaping `\"` and `\\`) from
/// already comment-stripped text.
fn extract_string_literals(src: &str) -> Vec<String> {
    let chars: Vec<char> = src.chars().collect();
    let mut literals = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '"' {
            let mut lit = String::new();
            i += 1;
            while i < chars.len() && chars[i] != '"' {
                if chars[i] == '\\' && i + 1 < chars.len() {
                    match chars[i + 1] {
                        'n' => lit.push('\n'),
                        't' => lit.push('\t'),
                        other => lit.push(other),
                    }
                    i += 2;
                } else {
                    lit.push(chars[i]);
                    i += 1;
                }
            }
            literals.push(lit);
            i += 1; // closing quote
        } else {
            i += 1;
        }
    }
    literals
}

/// Finds every `N` for a `compare(0, N, "<literal>")` call in `body`, paired with that literal.
fn extract_compare_prefix_calls(body: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut search_from = 0;
    while let Some(rel) = body[search_from..].find("compare(0,") {
        let start = search_from + rel + "compare(0,".len();
        let rest = &body[start..];
        let comma = rest
            .find(',')
            .unwrap_or_else(|| panic!("malformed compare(0, N, ...) call: {rest}"));
        let n: usize = rest[..comma]
            .trim()
            .parse()
            .unwrap_or_else(|e| panic!("compare(0, N, ...)'s N is not a number: {e}"));
        let after_comma = &rest[comma + 1..];
        let quote_start = after_comma
            .find('"')
            .unwrap_or_else(|| panic!("compare(0, {n}, ...) has no string literal"));
        let literal_text = &after_comma[quote_start..];
        let literal = extract_string_literals(literal_text)
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("could not parse compare(0, {n}, ...)'s literal"));
        out.push((n, literal));
        search_from = start + comma + 1;
    }
    out
}

/// Finds every `<literal>` for a `find("<literal>")` call in `body`.
fn extract_find_calls(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut search_from = 0;
    while let Some(rel) = body[search_from..].find("find(\"") {
        let start = search_from + rel + "find(".len();
        let literal_text = &body[start..];
        let literal = extract_string_literals(literal_text)
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("could not parse find(...)'s literal"));
        out.push(literal);
        search_from = start + 1;
    }
    out
}

#[test]
fn normalisation_matches_the_vendored_function() {
    let path = vendored_file_path();
    let src = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("reading vendored file {}: {e}", path.display()));

    let raw_body = extract_function_body(&src, FUNCTION_NAME);
    let body = strip_comments(&raw_body);

    let literals = extract_string_literals(&body);
    assert_eq!(
        literals.len(),
        4,
        "expected exactly four string literals in {FUNCTION_NAME}'s body (comments stripped), \
         found {}: {literals:?}",
        literals.len()
    );

    let expected: HashSet<String> = [
        oracle_compare::ATTRIBUTION_PREFIX,
        oracle_compare::MODEL_LABEL_ID_PREFIX,
        oracle_compare::OBJECT_LABEL_ID_SUBSTRING,
        oracle_compare::PIPELINE_PLUGIN_SUBSTRING,
    ]
    .iter()
    .map(|b| std::str::from_utf8(b).unwrap().to_string())
    .collect();
    let found: HashSet<String> = literals.iter().cloned().collect();
    assert_eq!(
        found,
        expected,
        "the comparator's four normalisation literals no longer match \
         {FUNCTION_NAME} in {}",
        path.display()
    );

    let prefix_calls = extract_compare_prefix_calls(&body);
    let prefix_literals: HashSet<String> = prefix_calls.iter().map(|(_, l)| l.clone()).collect();
    let expected_prefixes: HashSet<String> = [
        oracle_compare::ATTRIBUTION_PREFIX,
        oracle_compare::MODEL_LABEL_ID_PREFIX,
    ]
    .iter()
    .map(|b| std::str::from_utf8(b).unwrap().to_string())
    .collect();
    assert_eq!(
        prefix_literals, expected_prefixes,
        "the two prefix (compare(0, N, ...)) literals changed upstream"
    );
    for (n, literal) in &prefix_calls {
        assert_eq!(
            *n,
            literal.len(),
            "upstream's N ({n}) no longer equals the byte length of {literal:?} ({})",
            literal.len()
        );
    }

    let find_literals: HashSet<String> = extract_find_calls(&body).into_iter().collect();
    let expected_substrings: HashSet<String> = [
        oracle_compare::OBJECT_LABEL_ID_SUBSTRING,
        oracle_compare::PIPELINE_PLUGIN_SUBSTRING,
    ]
    .iter()
    .map(|b| std::str::from_utf8(b).unwrap().to_string())
    .collect();
    assert_eq!(
        find_literals, expected_substrings,
        "the two substring (find(...)) literals changed upstream"
    );
}
