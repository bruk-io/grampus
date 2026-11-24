//! The G-code parser, the move model and the localisation that L2, L3 and L4 (`levels.rs`) read,
//! and that plan step 24 reuses for its G-code fallback.
//!
//! Byte-oriented like the normalisation in `levels.rs`: lines are split on `\n` with no UTF-8
//! validation. Building a command word, a parameter letter or a localisation tag from a line's
//! bytes uses `String::from_utf8_lossy`, which is lossless for the ASCII text every mnemonic,
//! axis letter and reserved tag is written in and never panics on a byte that is not valid UTF-8
//! elsewhere on the line (a free-text object name, in principle). A single trailing `\r` is
//! stripped from each line before it is split into words, so a `\r\n`-terminated file parses the
//! same G-code as its `\n`-terminated twin; this is a parsing convenience only, distinct from
//! L0/L1's byte-for-byte and declared-normalisation comparisons in `levels.rs`, which keep `\r`
//! (R4 says L0 and L1 normalise nothing else, and this crate does not change that — L0 and L1
//! still fail a CRLF-vs-LF pair; only L2 upward, which this stripping feeds, treat the two the
//! same). **Decision (task silent, declared here since R4 does not mention it):** without this,
//! a trailing axis word on a CRLF line would carry a literal `Y0\r` whose `f64` parse fails,
//! silently dropping that axis from L3 and L4 rather than comparing it; `tests/levels.rs`'s
//! `crlf_and_lf_pair_agree_from_l2_up` pins the intended outcome so a later change to this crate
//! cannot alter it unnoticed.
//!
//! **Decision (task silent):** the E register's starting mode, before any `M82`/`M83`, is
//! absolute — RepRap's own default and `GCodeWriter::preamble`'s (`src/libslic3r/GCodeWriter.cpp
//! :298-301` emits `M83` only when `config.use_relative_e_distances` is set, `M82` otherwise,
//! and only for flavors in the `FLAVOR_IS(...)` gate at `:289-296`, which includes
//! `gcfMarlinLegacy`). Every Bambu machine profile this port's oracle configurations use sets
//! `"gcode_flavor": "marlin"` (`resources/profiles/BBL/machine/fdm_machine_common.json:31`,
//! mapped to `gcfMarlinLegacy` at `PrintConfig.cpp:225`), so that gate is always taken and an
//! explicit `M82`/`M83` is always emitted near the top of real oracle output; this default is
//! never actually exercised against it. It only matters for a hand-written fragment with no mode
//! command at all, which this task's fixtures exercise directly.

use std::collections::BTreeMap;

/// Splits `data` into lines the way `std::getline` walks a stream delimited by `\n`: a trailing
/// `\n` ends the final line without a phantom empty line after it, a missing trailing `\n` still
/// yields the last partial line, and empty input yields no lines at all. Shared with `levels.rs`,
/// which defines the canonical copy; `levels.rs`'s is `pub(crate)` so both modules split lines
/// identically.
pub(crate) use crate::levels::getline_split;

/// One letter-plus-literal parameter word from a command line, e.g. `X10.0` → letter `'X'`,
/// literal `"10.0"`. `letter` and `literal` are kept exactly as written (case, sign, trailing
/// zeros and all) because L2 compares them as text; `value` is the literal parsed as `f64`,
/// `None` when it is not a valid float (an L2 token still carries the letter and literal then).
#[derive(Debug, Clone, PartialEq)]
pub struct Word {
    pub letter: char,
    pub literal: String,
    pub value: Option<f64>,
}

/// One token of the L2 stream: either the command word that opens a command line, or one of its
/// parameter words. `levels.rs` flattens every command line's tokens, in order, into one sequence
/// per file.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Command(String),
    Param(Word),
}

/// One token with the 1-based line number (into the raw, un-normalised file) it came from.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenEntry {
    pub line_number: usize,
    pub token: Token,
}

/// The E register's mode, set by `M82` (absolute, the default) or `M83` (relative).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EMode {
    Absolute,
    Relative,
}

/// Which spelling of each localisation tag a file used, recorded independently for the three
/// tag categories because `GCode.cpp:4610`'s apron-band block emits the Bambu layer tag
/// (`; CHANGE_LAYER`) immediately followed by the non-Bambu Z spelling (`;Z:`) regardless of
/// `is_BBL_printer()` — a real file can mix spellings within one category pair, so this is a set
/// of independent flags rather than one enum that assumes a file picks one spelling and keeps it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TagSpellingsSeen {
    pub layer_bambu: bool,
    pub layer_compatible: bool,
    pub role_bambu: bool,
    pub role_compatible: bool,
    pub z_bambu: bool,
    pub z_compatible: bool,
}

/// The localisation state current at a move: the layer the `; CHANGE_LAYER` / `;LAYER_CHANGE`
/// comments count (`None` before the first one), the Z the `; Z_HEIGHT:` / `;Z:` comment last
/// named, the extrusion role the `; FEATURE:` / `;TYPE:` comment last named, and the object
/// label the `; printing object`, `; stop printing object`, `; start printing object, unique
/// label id:` and `; stop printing object, unique label id:` comments last opened or closed.
///
/// Carried on every [`Move`] rather than as a side table, so a reader can ask "what layer, role
/// or object was this move in" without re-walking the file. The "way to ask for the level
/// reached within one layer, one role or one object" plan step 24 and task `39-compare-stages`
/// need is [`crate::levels::evaluate_within`] with a [`crate::levels::Scope`] built from these
/// fields — not a plain filter-the-move-slice-then-call-`evaluate_l4` recipe, which would lose
/// each move's carried-forward position and report a filtered group's extruded/travel length
/// measured from the scope's first move instead of from where the whole file's previous move
/// actually left off.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Localisation {
    pub layer: Option<usize>,
    pub z: Option<f64>,
    pub role: Option<String>,
    pub object: Option<String>,
}

/// One `G0`, `G1`, `G2` or `G3` line. Only the eight axis letters L3 and L4 care about (`X`, `Y`,
/// `Z`, `E`, `F`, `I`, `J`, `R`) are captured in `words`; a letter absent from this map was absent
/// on the line, which R4/L3 treats as a hard mismatch against a side that carries it — not as a
/// zero value.
///
/// `x`, `y` and `e_delta` are derived, not literal G-code words: `x`/`y` are the move's resulting
/// absolute position, carrying forward an axis absent from this line from the previous move (an
/// absent word is not a move to the origin); `e_delta` is the signed change in the E register
/// this move causes, `None` only when the line has no `E` word at all. They are computed in the
/// same pass that tracks `e_mode` and the `G92 E<v>` origin reset, so L4's six aggregates can read
/// them directly instead of re-deriving the same state machine a second time.
#[derive(Debug, Clone, PartialEq)]
pub struct Move {
    pub line_number: usize,
    pub command: String,
    pub words: BTreeMap<char, Word>,
    pub active_tool: Option<u32>,
    pub e_mode: EMode,
    pub localisation: Localisation,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub e_delta: Option<f64>,
}

impl Move {
    /// The literal (letter, text) of an axis word present on this line, for L2-style text
    /// equality. `None` when the axis letter was absent.
    pub fn axis_literal(&self, letter: char) -> Option<&str> {
        self.words.get(&letter).map(|w| w.literal.as_str())
    }

    /// The parsed value of an axis word present on this line. `None` when the axis letter was
    /// absent, distinct from a word present with an unparseable literal (also `None` — L3 is not
    /// reached by a move whose words fail this invariant in real G-code).
    pub fn axis_value(&self, letter: char) -> Option<f64> {
        self.words.get(&letter).and_then(|w| w.value)
    }
}

/// The parse of one G-code file: the flattened L2 token stream and the L3/L4 move model, which
/// spelling(s) of the three localisation tag categories the file used, and the raw count of
/// layer-change tags the localisation found. `layer_boundaries` is kept separately from the
/// per-move `localisation.layer` values because a layer boundary with no move after it (the
/// last layer in a file that ends on a custom-G-code seam, for instance) would otherwise vanish
/// from every per-move-derived count — L4's whole-file "layer count" (R4: "the number of layer
/// boundaries the localisation found") reads this field rather than counting distinct layer
/// indices across moves, so it stays correct in that case.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParsedFile {
    pub tokens: Vec<TokenEntry>,
    pub moves: Vec<Move>,
    pub tag_spellings: TagSpellingsSeen,
    pub layer_boundaries: usize,
}

const ROLE_BAMBU_PREFIX: &str = " FEATURE: ";
const ROLE_COMPATIBLE_PREFIX: &str = "TYPE:";
const LAYER_BAMBU_PREFIX: &str = " CHANGE_LAYER";
const LAYER_COMPATIBLE_PREFIX: &str = "LAYER_CHANGE";
const Z_BAMBU_PREFIX: &str = " Z_HEIGHT: ";
const Z_COMPATIBLE_PREFIX: &str = "Z:";
const OBJECT_START_PREFIX: &str = " printing object ";
const OBJECT_STOP_PREFIX: &str = " stop printing object ";
const OBJECT_START_UNIQUE_PREFIX: &str = " start printing object, unique label id: ";
const OBJECT_STOP_UNIQUE_PREFIX: &str = " stop printing object, unique label id: ";

/// Updates the localisation state and `tag_spellings` from one line's comment text (everything
/// after the first `;`, not including it). A line that matches none of these markers — almost
/// every comment in a real file — leaves the state untouched.
fn apply_localisation_tag(
    comment: &str,
    layer: &mut Option<usize>,
    layer_boundaries: &mut usize,
    z: &mut Option<f64>,
    role: &mut Option<String>,
    object: &mut Option<String>,
    seen: &mut TagSpellingsSeen,
) {
    if let Some(rest) = comment.strip_prefix(ROLE_BAMBU_PREFIX) {
        *role = Some(rest.trim_end().to_string());
        seen.role_bambu = true;
    } else if let Some(rest) = comment.strip_prefix(ROLE_COMPATIBLE_PREFIX) {
        *role = Some(rest.trim_end().to_string());
        seen.role_compatible = true;
    }

    if comment.starts_with(LAYER_BAMBU_PREFIX) {
        *layer = Some(layer.map_or(0, |n| n + 1));
        *layer_boundaries += 1;
        seen.layer_bambu = true;
    } else if comment.starts_with(LAYER_COMPATIBLE_PREFIX) {
        *layer = Some(layer.map_or(0, |n| n + 1));
        *layer_boundaries += 1;
        seen.layer_compatible = true;
    }

    if let Some(rest) = comment.strip_prefix(Z_BAMBU_PREFIX) {
        *z = rest.trim_end().parse::<f64>().ok();
        seen.z_bambu = true;
    } else if let Some(rest) = comment.strip_prefix(Z_COMPATIBLE_PREFIX) {
        *z = rest.trim_end().parse::<f64>().ok();
        seen.z_compatible = true;
    }

    if let Some(rest) = comment.strip_prefix(OBJECT_START_UNIQUE_PREFIX) {
        *object = Some(rest.trim_end().to_string());
    } else if comment.starts_with(OBJECT_STOP_UNIQUE_PREFIX) {
        *object = None;
    } else if let Some(rest) = comment.strip_prefix(OBJECT_START_PREFIX) {
        *object = Some(rest.trim_end().to_string());
    } else if comment.starts_with(OBJECT_STOP_PREFIX) {
        *object = None;
    }
}

/// Splits a line (with any trailing `\r` already removed) into its code part (before the first
/// `;`) and its comment part (after the first `;`, excluding the `;` itself; `None` when the line
/// has no `;` at all).
fn split_comment(line: &[u8]) -> (&[u8], Option<&[u8]>) {
    match line.iter().position(|&b| b == b';') {
        Some(idx) => (&line[..idx], Some(&line[idx + 1..])),
        None => (line, None),
    }
}

/// Splits a code part on runs of ASCII space/tab, dropping empty pieces.
fn split_words(code: &[u8]) -> Vec<&[u8]> {
    code.split(|&b| b == b' ' || b == b'\t')
        .filter(|w| !w.is_empty())
        .collect()
}

fn parse_word(bytes: &[u8]) -> Word {
    let text = String::from_utf8_lossy(bytes).into_owned();
    let mut chars = text.chars();
    let letter = chars.next().unwrap_or('\0');
    let literal: String = chars.collect();
    let value = literal
        .strip_prefix('+')
        .unwrap_or(&literal)
        .parse::<f64>()
        .ok();
    Word {
        letter,
        literal,
        value,
    }
}

/// `Some(n)` when `command` is a tool-change word (`T0`, `T1`, …, case-insensitive), the digits
/// after `T` parsed as `n`.
fn parse_tool_change(command: &str) -> Option<u32> {
    let rest = command
        .strip_prefix('T')
        .or_else(|| command.strip_prefix('t'))?;
    if rest.is_empty() || !rest.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    rest.parse::<u32>().ok()
}

/// `true` when `command` is one of the four move commands, case-insensitive.
fn is_move_command(command: &str) -> bool {
    matches!(
        command.to_ascii_uppercase().as_str(),
        "G0" | "G1" | "G2" | "G3"
    )
}

/// The eight axis letters this parser captures into a move's `words`. Re-exported from
/// `levels.rs`, which owns the canonical copy and uses the same array to decide what L3 checks
/// and reports, so capture and comparison cannot silently drift apart by editing only one of
/// them.
use crate::levels::MOVE_AXES;

/// Parses one G-code file into its L2 token stream and its L3/L4 move model in a single pass:
/// localisation tags update the running layer/Z/role/object state, `T<n>` updates the active
/// tool, `M82`/`M83` update the E mode, and `G92 E<v>` resets the E baseline, all before the next
/// move reads them.
pub fn parse(data: &[u8]) -> ParsedFile {
    let mut tokens = Vec::new();
    let mut moves = Vec::new();
    let mut tag_spellings = TagSpellingsSeen::default();

    let mut active_tool: Option<u32> = None;
    let mut e_mode = EMode::Absolute;
    let mut last_e: f64 = 0.0;
    let mut layer: Option<usize> = None;
    let mut layer_boundaries: usize = 0;
    let mut z: Option<f64> = None;
    let mut role: Option<String> = None;
    let mut object: Option<String> = None;
    let mut last_x: Option<f64> = None;
    let mut last_y: Option<f64> = None;

    for (i, raw_line) in getline_split(data).into_iter().enumerate() {
        let line_number = i + 1;
        let line: &[u8] = if raw_line.last() == Some(&b'\r') {
            &raw_line[..raw_line.len() - 1]
        } else {
            raw_line
        };
        let (code, comment) = split_comment(line);

        if let Some(comment) = comment {
            let comment_text = String::from_utf8_lossy(comment).into_owned();
            apply_localisation_tag(
                &comment_text,
                &mut layer,
                &mut layer_boundaries,
                &mut z,
                &mut role,
                &mut object,
                &mut tag_spellings,
            );
        }

        let raw_words = split_words(code);
        if raw_words.is_empty() {
            continue;
        }

        let command = String::from_utf8_lossy(raw_words[0]).into_owned();
        tokens.push(TokenEntry {
            line_number,
            token: Token::Command(command.clone()),
        });

        let mut parsed_words: Vec<Word> = Vec::with_capacity(raw_words.len().saturating_sub(1));
        for raw_word in &raw_words[1..] {
            let word = parse_word(raw_word);
            tokens.push(TokenEntry {
                line_number,
                token: Token::Param(word.clone()),
            });
            parsed_words.push(word);
        }

        if let Some(n) = parse_tool_change(&command) {
            active_tool = Some(n);
        }
        match command.to_ascii_uppercase().as_str() {
            "M82" => e_mode = EMode::Absolute,
            "M83" => e_mode = EMode::Relative,
            _ => {}
        }
        if command.eq_ignore_ascii_case("G92") {
            if let Some(v) = parsed_words
                .iter()
                .find(|w| w.letter == 'E' || w.letter == 'e')
                .and_then(|w| w.value)
            {
                last_e = v;
            }
        }

        if is_move_command(&command) {
            let mut words: BTreeMap<char, Word> = BTreeMap::new();
            for word in &parsed_words {
                let axis = word.letter.to_ascii_uppercase();
                if MOVE_AXES.contains(&axis) {
                    words.insert(axis, word.clone());
                }
            }

            let x = words.get(&'X').and_then(|w| w.value).or(last_x);
            let y = words.get(&'Y').and_then(|w| w.value).or(last_y);
            let e_delta = words.get(&'E').and_then(|w| w.value).map(|v| match e_mode {
                EMode::Absolute => {
                    let delta = v - last_e;
                    last_e = v;
                    delta
                }
                EMode::Relative => {
                    last_e += v;
                    v
                }
            });

            last_x = x;
            last_y = y;

            moves.push(Move {
                line_number,
                command: command.clone(),
                words,
                active_tool,
                e_mode,
                localisation: Localisation {
                    layer,
                    z,
                    role: role.clone(),
                    object: object.clone(),
                },
                x,
                y,
                e_delta,
            });
        }
    }

    ParsedFile {
        tokens,
        moves,
        tag_spellings,
        layer_boundaries,
    }
}
