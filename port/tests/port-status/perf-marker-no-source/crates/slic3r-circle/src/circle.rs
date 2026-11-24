// Fixture: a module whose `PORT STATUS` trailer names no `source:` line at all, so a
// PERF(port) marker inside it has no vendored source path to put in its row --
// task 23-perf-backlog's well-formedness rule.

fn identity(x: f64) -> f64 {
    x
}

// PERF(port): no vendored source: line exists on this module's trailer, so there is nothing to
// cite as this row's vendored source path.

// PORT STATUS
// confidence: high
// tier: exact
// oracle: built
