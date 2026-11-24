# Fixture configuration table

This is a hand-written stand-in for the `docs/ORACLE.md` configuration table `04-oracle-slice`
will add; that table does not exist yet (`docs/ORACLE.md` today has no `tolerance` or
`comparison function` column at all), so `--tolerance-from-config` is exercised against this
fixture instead, per task `09-comparator-gcode-levels`.

| Configuration | Preset path | Tolerance | Comparison function |
|---|---|---|---|
| basic_pla | presets/basic_pla.json | 0.01 mm; reason: oracle float jitter observed on this fixture | abs |
| no_reason | presets/no_reason.json | 0.01 mm | |
