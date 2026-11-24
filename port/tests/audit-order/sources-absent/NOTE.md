# Fixture: every source absent

Deliberately empty otherwise: no `crates/`, no `oracle-parity.tsv`, no `determinism.tsv`. Used
by `port/audit-order --self-test` to prove the "an absent source contributes zero rows and is
not an error" rule for sources 2, 3 and 4. This file exists only so git tracks the directory.
