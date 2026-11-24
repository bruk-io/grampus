//! The report shape every part of plan step 5 ("The comparator", R4) returns, and all five
//! levels: L0 and L1 on raw bytes, L2 on the token stream, L3 on the move model and L4 on its
//! per-layer/per-role aggregates, both parsed by [`gcode::parse`]. The five are evaluated
//! independently and are not assumed nested (R4): a pair can fail L0-L2 while passing L3 and L4,
//! or pass L2 while failing L1, and the report states each outcome on its own rather than
//! stopping at the first pass.

pub mod gcode;
pub mod levels;
pub mod tolerance;

pub use levels::{
    normalise, Aggregate, AggregateGroup, AxisDeviation, FailLocation, L4Aggregates, LevelOutcome,
    Side, ATTRIBUTION_PREFIX, MODEL_LABEL_ID_PREFIX, OBJECT_LABEL_ID_SUBSTRING,
    PIPELINE_PLUGIN_SUBSTRING,
};
pub use tolerance::{ComparisonFunction, Tolerance, ToleranceSource};

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
#[derive(Debug, Clone, PartialEq)]
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
    /// L3's max/RMS deviation per axis, over every compared pair of moves, printed whether or
    /// not L3 passed (R4).
    pub l3_axis_deviations: std::collections::BTreeMap<char, AxisDeviation>,
    /// L4's full aggregates for side `a`.
    pub l4_aggregates_a: L4Aggregates,
    /// L4's full aggregates for side `b`.
    pub l4_aggregates_b: L4Aggregates,
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
/// valid UTF-8) and evaluates all five levels at the repository-wide default tolerance, exact.
/// Equivalent to `compare_with_tolerance(a, b, Tolerance::Exact)`.
pub fn compare(a: &[u8], b: &[u8]) -> Report {
    compare_with_tolerance(a, b, Tolerance::Exact)
}

/// Evaluates all five levels (L0 and L1 on the raw bytes; L2, L3 and L4 on `gcode::parse`'s
/// token stream and move model) at `tolerance`. Every level is evaluated independently and
/// never short-circuits on another level's outcome (R4): L2, L3 and L4 are computed even when
/// L0 passes, and vice versa.
pub fn compare_with_tolerance(a: &[u8], b: &[u8], tolerance: Tolerance) -> Report {
    let l0 = levels::evaluate_l0(a, b);
    let l1 = levels::evaluate_l1(a, b);
    let l2 = levels::evaluate_l2(a, b);

    let parsed_a = gcode::parse(a);
    let parsed_b = gcode::parse(b);
    let tolerance_value = tolerance.value();

    let (l3, l3_axis_deviations) =
        levels::evaluate_l3(&parsed_a.moves, &parsed_b.moves, tolerance_value);
    let (l4, l4_aggregates_a, l4_aggregates_b) = levels::evaluate_l4(
        &parsed_a.moves,
        &parsed_b.moves,
        parsed_a.layer_boundaries,
        parsed_b.layer_boundaries,
        tolerance_value,
    );

    let outcomes = [l0, l1, l2, l3, l4];

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
    // holds regardless of how L0 produces its own outcome.
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
        tolerance,
        l3_axis_deviations,
        l4_aggregates_a,
        l4_aggregates_b,
    }
}

pub mod format;
