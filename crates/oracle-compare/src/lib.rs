//! The report shape every part of plan step 5 ("The comparator", R4) returns, and the two byte
//! levels, L0 and L1, that this task evaluates. L2, L3 and L4 are declared here but always report
//! [`LevelOutcome::NotEvaluated`] until task `09-comparator-gcode-levels` fills them in.

pub mod levels;

pub use levels::{
    normalise, FailLocation, LevelOutcome, ATTRIBUTION_PREFIX, MODEL_LABEL_ID_PREFIX,
    OBJECT_LABEL_ID_SUBSTRING, PIPELINE_PLUGIN_SUBSTRING,
};

/// The five levels of sameness, ordered strongest to weakest per R4: L0 is the strictest claim
/// (identical bytes), L4 the loosest (aggregate statistics within tolerance).
///
/// Deliberately not `PartialOrd`/`Ord`: `Level::ALL`'s declaration order is strongest-to-weakest,
/// but `Report::strongest_evaluated_pass` below picks the *highest-indexed* pass (see its doc
/// comment), the opposite of what a reader would expect `max()` on this enum to mean. Picking
/// among levels goes through `index()` and an explicit `max_by_key`/`min_by_key` at the call
/// site, never through a derived comparison, so the direction is visible at every use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    L0,
    L1,
    L2,
    L3,
    L4,
}

impl Level {
    pub const ALL: [Level; 5] = [Level::L0, Level::L1, Level::L2, Level::L3, Level::L4];

    pub const fn name(self) -> &'static str {
        match self {
            Level::L0 => "L0",
            Level::L1 => "L1",
            Level::L2 => "L2",
            Level::L3 => "L3",
            Level::L4 => "L4",
        }
    }

    /// The level one step stricter than `self` in the R4 ordering (`None` for `L0`, the
    /// strictest level there is).
    pub const fn stricter(self) -> Option<Level> {
        match self {
            Level::L0 => None,
            Level::L1 => Some(Level::L0),
            Level::L2 => Some(Level::L1),
            Level::L3 => Some(Level::L2),
            Level::L4 => Some(Level::L3),
        }
    }

    pub const fn index(self) -> usize {
        match self {
            Level::L0 => 0,
            Level::L1 => 1,
            Level::L2 => 2,
            Level::L3 => 3,
            Level::L4 => 4,
        }
    }

    pub fn parse(s: &str) -> Option<Level> {
        match s {
            "L0" | "l0" => Some(Level::L0),
            "L1" | "l1" => Some(Level::L1),
            "L2" | "l2" => Some(Level::L2),
            "L3" | "l3" => Some(Level::L3),
            "L4" | "l4" => Some(Level::L4),
            _ => None,
        }
    }
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

/// The tolerance a run used. Every tolerance in this repository defaults to exact; a looser one
/// is declared only on a manifest row or a `docs/ORACLE.md` configuration row, with a reason
/// (task `09-comparator-gcode-levels` fills in the looser variants). This task only ever
/// produces `Exact`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tolerance {
    Exact,
}

impl std::fmt::Display for Tolerance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tolerance::Exact => f.write_str("exact"),
        }
    }
}

/// The comparator's report: the outcome of every level, which of them is the strongest, where
/// the comparison stops being exact beyond that, and the tolerance the run used.
///
/// **Which level is "strongest" is not R4's strict-to-loose ordering.** Among the levels that
/// passed, this report names the one with the *highest* index (the loosest claim that still held
/// evaluated), not the lowest (the strictest claim, which R4's prose calls "strongest"). This
/// follows the task's own worked example: on a byte-identical pair, both L0 and L1 pass, and the
/// report names **L1** as the strongest evaluated level, because reporting the furthest level a
/// pair was confirmed to hold at is the informative summary once a level has passed — the
/// stricter passes beneath it add no information a reader does not already have. `stricter_failure`
/// below keeps R4's literal ordering (L0 is stricter than L1), because that field exists to show
/// a stricter level that did *not* hold, immediately beyond the one that did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// Outcome of each level, indexed by [`Level::index`].
    pub outcomes: [LevelOutcome; 5],
    /// Among levels with [`LevelOutcome::Pass`], the one with the highest index; `None` if no
    /// evaluated level passed.
    pub strongest_evaluated_pass: Option<Level>,
    /// The level one step stricter than `strongest_evaluated_pass`, when that level was
    /// evaluated and failed. `None` when nothing passed (see `first_differing_byte` instead),
    /// when the strongest pass is already the strictest evaluated level, or when the stricter
    /// level also passed.
    pub stricter_failure: Option<(Level, FailLocation)>,
    /// The first byte at which the raw inputs differ, populated only when no evaluated level
    /// passed.
    pub first_differing_byte: Option<usize>,
    /// The tolerance this run used.
    pub tolerance: Tolerance,
}

impl Report {
    pub fn outcome(&self, level: Level) -> &LevelOutcome {
        &self.outcomes[level.index()]
    }

    /// Levels this report actually evaluated (excludes [`LevelOutcome::NotEvaluated`]).
    pub fn evaluated_levels(&self) -> Vec<Level> {
        Level::ALL
            .into_iter()
            .filter(|l| !matches!(self.outcome(*l), LevelOutcome::NotEvaluated))
            .collect()
    }

    /// Whether `level` passed, for `--require`.
    pub fn passed(&self, level: Level) -> bool {
        self.outcome(level).is_pass()
    }
}

/// Reads both inputs as bytes (never as UTF-8 — a G-code comment may hold bytes that are not
/// valid UTF-8) and evaluates L0 and L1. L2, L3 and L4 are reported as
/// [`LevelOutcome::NotEvaluated`]; task `09-comparator-gcode-levels` fills them in.
pub fn compare(a: &[u8], b: &[u8]) -> Report {
    let l0 = levels::evaluate_l0(a, b);
    let l1 = levels::evaluate_l1(a, b);

    let outcomes = [
        l0.clone(),
        l1.clone(),
        LevelOutcome::NotEvaluated,
        LevelOutcome::NotEvaluated,
        LevelOutcome::NotEvaluated,
    ];

    let strongest_evaluated_pass = Level::ALL
        .into_iter()
        .filter(|l| outcomes[l.index()].is_pass())
        .max_by_key(|l| l.index());

    let stricter_failure = strongest_evaluated_pass.and_then(|strongest| {
        let stricter = strongest.stricter()?;
        match &outcomes[stricter.index()] {
            LevelOutcome::Fail(loc) => Some((stricter, loc.clone())),
            _ => None,
        }
    });

    // Computed straight from the raw inputs rather than recovered from L0's outcome, so this
    // holds regardless of how L0 (or a later task's extra levels) produce their own outcome.
    let first_differing_byte = if strongest_evaluated_pass.is_none() {
        levels::first_difference(a, b)
    } else {
        None
    };

    Report {
        outcomes,
        strongest_evaluated_pass,
        stricter_failure,
        first_differing_byte,
        tolerance: Tolerance::Exact,
    }
}

pub mod format;
