// Fixture: a row whose `enclosing:` cell names a unit in a DIFFERENT crate. R19's fallback is
// "proven through its enclosing file's exported surface", and a unit in another crate is not
// that -- so this is a bad-enclosing-unit finding and the row derives "drafted".

mod helper_mod;
