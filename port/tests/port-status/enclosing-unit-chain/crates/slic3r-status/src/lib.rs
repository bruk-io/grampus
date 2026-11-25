// Fixture: two rows each naming the other as its enclosing unit. A chain is refused outright
// rather than followed, which is also what makes a cycle impossible: both rows are
// `bad-enclosing-unit` findings and neither derives "ported".

mod helper_mod;
mod ported_mod;
