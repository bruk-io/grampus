---
kind: plan
---
# Plan: foundation-13 -- `PrintConfig.cpp`/`PrintConfig.hpp` (the print configuration option set)

## Files that change

- `crates/slic3r-printconfig/src/print_config.rs` (new)
- `crates/slic3r-printconfig/src/lib.rs`
- `crates/slic3r-printconfig/tests/print_config_proof.rs` (new)
- `crates/slic3r-ffi/cxx/proof_units/print_config.h` (new)
- `crates/slic3r-ffi/cxx/proof_units/print_config.cpp` (new)
- `crates/slic3r-ffi/src/proof_units/print_config.rs` (new)

`crates/slic3r-ffi/cxx/proof_shims.cpp`, `src/proof.rs` and `cxx/include_vendored.cpp`, named by
`task.md`, do not exist on `main`: `docs/PORTING.md`'s "Decisions recorded on 2026-11-25" (decision
1) replaced them with one `cxx/proof_units/<unit>.{h,cpp}` + `src/proof_units/<unit>.rs` triple per
unit, and `docs/WORKFLOW.md`'s "Decisions that govern over a task's file list" makes that the
governing text over `task.md`'s line. This plan's three new `proof_units/` files are that route.
`crates/slic3r-ffi/src/proof_units.rs` is **not** touched -- `build.rs` globs it from the files on
disk, so it is not a file this task writes at all (see that file's own header comment).

## The risk this plan exists to answer, measured rather than argued

**The true size, measured before any Rust was written.** The unit is 17,247 source lines, but that
undercounts the work: `grep -c '^\s*def\s*=\s*this->add' PrintConfig.cpp` finds **981** option
definitions in the `.cpp`, each a `ConfigOptionDef` built by direct field assignment
(`def->label = L("...")`, `def->min = 1`, ...) plus one `set_default_value(new ConfigOptionX{...})`
call -- and `PrintConfig.hpp` separately declares **~40 enums** (several with their own helper
functions reaching real logic, not just values) and, through the `PRINT_CONFIG_CLASS_DEFINE`/
`PRINT_CONFIG_CLASS_DERIVED_DEFINE` macro family (`:1213-1276`), **11 `StaticPrintConfig`-derived
classes** (`PrintObjectConfig`, `PrintRegionConfig`, `MachineEnvelopeConfig`, `GCodeConfig`,
`PrintConfig`, `FullPrintConfig`, `SLAPrintConfig`, `SLAPrintObjectConfig`, `SLAMaterialConfig`,
`SLAPrinterConfig`, `SLAFullPrintConfig`) each declaring, as **typed struct fields**, a second,
independent list of the option keys it carries -- a list this plan must transcribe in addition to
the 981 `.add()` calls, not instead of them, because the two lists serve different jobs (the
registry's metadata vs. a config object's typed storage) and disagree in membership (`FullPrintConfig`
and `SLAFullPrintConfig` declare **no fields of their own** -- they only aggregate their parents).
So the real transcription surface is on the order of 981 metadata blocks plus ~900 struct-field
declarations plus `handle_legacy`/`handle_legacy_composite` (hundreds of their own renamed-key
branches) plus the per-class `apply_to()` bodies `PrintConfig.cpp` defines for each of the 11
classes. This is, measured this way, the largest single unit in the whole port by a wide margin --
consistent with `task.md`'s own "largest row ... by a factor of two," which undercounts it further
still.

**Decided: this plan does not promise to finish.** `docs/WORKFLOW.md`'s own rule governs the
fallback explicitly: never land a `mod` line over a live `todo!()`; an unreached module is
`drafted`, which is an honest status, and a `mod`-reached `todo!()` is `not-started`, which is
worse. This unit cannot be split (`task.md`'s own "Why this is an 8" section: one `target module`
path, so two tasks writing it would be one conflict, and the spike's batch caps already apply to
this row). So the order of work below is chosen so that, whatever fraction completes, a reviewer
can tell exactly which slice of the 981+900 declarations is in and which is not, and the pull
request records the true fraction measured at push time rather than a rounded claim.

## The proof design -- the risk `task.md` actually names

**Route.** Measured, not assumed, the same way `docs/PORTING.md`'s "Config.cpp" precedent and
`cxx/proof_units/utils.cpp`'s header comment both record their own measurements:

```
nm -m vendor/OrcaSlicer/build/arm64/src/libslic3r/CMakeFiles/libslic3r.dir/Release/PrintConfig.cpp.o \
  | grep -E '^\S+ \(.*\) external ' | grep -v weak | c++filt | sort -u | wc -l
-> 340
```

340 strong external `Slic3r::` symbols, far past the 1-4 range `clipper_z_utils.cpp` found viable
and past the 99 `utils.cpp` already refused. `#include`-ing `PrintConfig.cpp` whole is a
duplicate-symbol collision across most of `liblibslic3r.a`'s own archive member for this file, so
path 3 as a literal `#include` is **not attempted**. `extern const PrintConfigDef print_config_def`
(`:880`) is externally linked, so the shim is shaped like proof path 1 against that global --
`utils.cpp`'s own precedent for "the manifest cell still reads 3, matched verbatim by
`proof_present`; the route actually taken is recorded here and in the pull request body" -- and this
plan does the same.

**What the shim proves, and what it does not.** `task.md`'s own risk statement draws the line:
"which defaults, enum key maps and CLI spellings are proven against the oracle through proof path 3
and which are proven through the resolved-config goldens plan step 9 committed." Read plainly: the
behaviourally load-bearing fields -- the default value, the enum name/value map, the CLI spelling --
are mechanically checked against the oracle's own `print_config_def`; the GUI-presentation fields
(`label`, `full_label`, `tooltip`, `sidetext`, `category`, `mode`, `height`, `width`, `gui_type`,
`gui_flags`, ...) are not slicing behaviour, are not separately exercised by any oracle entry point,
and are instead caught the way every transcribed literal is caught: a `// PORT SRC:` marker per
value and the two-pass adversarial review reading the diff against the vendored source. This is the
split that turns "14,438 unreviewable lines" into "981 mechanically-checked rows plus N
reviewed-by-reading rows," which is the actual risk `task.md` asks this plan to resolve.

**Shim shape**, in `cxx/proof_units/print_config.{h,cpp}`:

- `oracle_option_keys() -> Vec<String>`: the full key set, in `print_config_def.options`' own
  iteration order (a `std::map<std::string, ConfigOptionDef>`, so alphabetical -- itself a
  `// FIDELITY(port):` site, since the Rust port's own table must iterate the same way wherever
  that order reaches output).
- `oracle_option_type(key) -> i32`, `oracle_option_cli(key) -> String`,
  `oracle_option_default_serialized(key) -> String` (`ConfigOptionDef::default_value->serialize()`,
  or a sentinel for a null default -- measured against `Config.hpp`'s `default_value` type before
  this is written), `oracle_option_enum_values(key) -> Vec<String>`,
  `oracle_option_enum_keys_map(key) -> Vec<(String, i32)>` (empty for a non-enum option).

**One comparison test, over the whole key set from the start.** `print_config_proof.rs` calls
`oracle_option_keys()` once and, for every key, compares the port's own `print_config_def()`
against the five functions above at exact equality -- never a subset the implementer happened to
finish, which would be weakening the check rather than proving it. Its failure count is this task's
own honest coverage meter: "N of 981 keys agree" is a number this plan's author measures and reports
in the pull request, not one it asserts. The test runs regardless of whether the module is `mod`-
reached; `cargo test -p slic3r-printconfig` sees it once the crate compiles, independent of whether
`lib.rs` gains the `mod print_config;` line.

## The class hierarchy and the one raw-pointer row

**`StaticCache<T>`/`StaticPrintConfig` (`:1041-1135`), in scope here because `PrintConfig.hpp`
declares it, not `Config.hpp`.** The vendored mechanism stores every option as a **typed struct
field** on the concrete config class, then maps an option key to that field by a **byte offset**
computed once (`opt_add`, storing `(const char*)&opt - base_ptr`) and read back through
`reinterpret_cast<ConfigOption*>((char*)owner + offset)` -- raw pointer arithmetic over the class's
own memory layout, not a map of `Box<dyn ConfigOption>`. `port/ownership.tsv`'s one row for this
unit (`PrintConfig.hpp:1132`, `StaticCache::m_defaults`, `T *`, class `borrowed`, verbatim type
`&'a mut T`) names exactly this mechanism's cached defaults pointer.

The simplest translation that keeps this meaning (`docs/WORKFLOW.md`'s "rejection rule" --
not the nicest one) is the same technique in Rust: each concrete class is `#[repr(C)]` with its
option fields in the same declared order as the vendored `PARAMETER_DEFINITION_SEQ`, and
`StaticCache<T>` records each field's byte offset with `std::mem::offset_of!` (computed at compile
time, not re-derived from a live pointer subtraction the way the C++ constructor does it, because
Rust has no "take the address of a field on a not-yet-constructed object" step to mirror -- a
deliberate, named divergence, not an invented shortcut: the vendored code computes the same compile-
time-constant number at run time once, this port computes it at compile time, and both read the same
field through the same offset from then on). `optptr` becomes one `unsafe fn` doing the pointer
arithmetic the vendored function does, each occurrence carrying its own `// SAFETY:` naming the
invariant (`owner` is a valid `&(mut) T`, the offset came from `offset_of!` on that same `T`, the
field at that offset really has the layout the caller's turbofish claims).

**The 11 classes' field lists** are transcribed from the `PRINT_CONFIG_CLASS_DEFINE`/
`_DERIVED_DEFINE` call sites (`:1282-2392`) as plain Rust struct definitions in declaration order
(the order `offset_of!`/the macro's `for_each_option_pair` both depend on for determinism, so
reordering here would be a `// FIDELITY(port):` divergence from the vendored field order, not a
harmless tidy-up). `FullPrintConfig`/`SLAFullPrintConfig` (`_DERIVED_DEFINE0`, no own fields) become
Rust structs that simply compose their parents' fields, matching `PRINT_CONFIG_CLASS_DERIVED_CLASS_LIST`'s
multiple-inheritance-as-field-composition exactly (Rust has no multiple inheritance to reach for).
`hash()`/`operator==`/`operator<`/`initialize()`/`apply_to()` are the one macro-generated visitor
(`PRINT_CONFIG_CLASS_COMMON_BODY`, `:1187-1211`) over each class's own field list; this plan ports it
once, generically over "the list of (key, offset) pairs `StaticCache` already built," rather than by
hand per class, because the vendored macro itself is one body instantiated 11 times, not 11 hand-
written bodies -- porting it as 11 separate copies would be inventing divergence the source does not
have.

## `L(s)` is the identity function in this file

`PrintConfig.cpp:91` -- `#define L(s) (s)` -- shadows the gettext-backed `L` every other `.cpp` in
the tree defines (`Slic3r::I18N::translate(s)`). Every `def->label = L("...")`,
`->tooltip = L("...")` etc. in this unit is therefore the literal string with no I18N call, verified
by that one line rather than assumed from the macro's name elsewhere in the tree -- so this unit adds
no dependency on `crates/slic3r-i18n` and every `L("...")` becomes `"...".to_string()` directly.

## Order of work

Each step names the slice a reviewer can accept or reject on its own, even though the task's
"Done when" gate (`mod print_config;`, `ported` status) is all-or-nothing across every step.

### Step 1: the proof shim and the whole-key-set comparison test
- `cxx/proof_units/print_config.{h,cpp}`, `src/proof_units/print_config.rs`,
  `tests/print_config_proof.rs`. Written and passing-against-an-empty-table (0 of 981) before any
  option is transcribed, so every later step's progress is a number this test prints, not a claim.
- delivers: none (step 20 is groundwork).

### Step 2: the ~40 enums and their helper functions (`PrintConfig.hpp:52-839`)
- Finite and fully bounded; ported in full, including `is_separable_infill_pattern` and its
  siblings, which are real logic, not plain value lists.
- delivers: none.

### Step 3: `StaticCache<T>`/`StaticPrintConfig` and the `PrintConfigDef`/`StaticPrintConfig::StaticCache` raw-pointer mechanism (`:1041-1211`)
- The one `port/ownership.tsv` row lands here, verbatim.
- delivers: none.

### Step 4: the 11 classes' field lists and the one generic visitor body
- The `PrintObjectConfig`/`PrintRegionConfig`/`MachineEnvelopeConfig`/`GCodeConfig`/`PrintConfig`/
  `FullPrintConfig`/`SLAPrintConfig`/`SLAPrintObjectConfig`/`SLAMaterialConfig`/`SLAPrinterConfig`/
  `SLAFullPrintConfig` structs, each field list transcribed in declaration order from its
  `PRINT_CONFIG_CLASS_DEFINE`/`_DERIVED_DEFINE`/`_DERIVED_DEFINE0` call site, plus the one generic
  `hash`/`eq`/`lt`/`initialize` visitor Step 3's `StaticCache` drives (see "The class hierarchy and
  the one raw-pointer row" above).
- delivers: none.

### Step 5: the option table, by the vendored `init_*` grouping, each its own reviewable slice
- `init_common_params`, `init_fff_params`, `init_extruder_option_keys`,
  `init_filament_option_keys`, `init_sla_params`, in that order (smallest first). Each option's
  `// PORT SRC:` lines are checked against the vendored source line a scratch extraction script
  records (`oracle/.cache/tmp/`, never committed, per `task.md`'s own scratch-file rule) -- a
  reliability aid for 981 line numbers, not a generator of the Rust itself: `docs/WORKFLOW.md`'s
  must-not-write list forecloses a committed generator under `port/`, and this script never
  produces committed output on its own.
- delivers: none.

### Step 6: `handle_legacy`/`handle_legacy_composite`
- `PrintConfigDef::handle_legacy`'s and `handle_legacy_composite`'s own renamed/obsoleted-key
  branches, read and ported after Step 5 so every key they rename or drop is already in the
  table they rename or drop it from.
- delivers: none.

### Step 7: wire the module in, only once steps 1-6 compile, hold no live `todo!()`/`unimplemented!()`, and the Step 1 test is fully green over all 981 keys
- Add `mod print_config;` to `crates/slic3r-printconfig/src/lib.rs` at its alphabetical position.
  Not added at all if any earlier step is short; see "Risks" below.
- delivers: none.

## Risks

**The real risk is Step 5's volume, not Steps 1-4's design.** Steps 1-4 are one-time, bounded design
decisions this plan has already settled by measurement (the `nm` count, the `L(s)` macro, the
`PRINT_CONFIG_CLASS_DEFINE` expansion). Step 5 is 981 rows of transcription that cannot be
reviewed as a block; the Step 1 test is what makes each row's default/type/enum/cli checkable on
its own instead of trusted. If Step 5 does not finish inside this task's branch, the honest outcome
is: `mod print_config;` stays out of `lib.rs`, `port/port-status` reports the unit `drafted`, and the
pull request states the exact count the Step 1 test passed against the exact count of 981 --
measured at push time, not estimated here.

**A second, smaller risk:** the `StaticCache` translation (Step 3) is this unit's only `unsafe` design
of its own; a wrong `// SAFETY:` invariant there is wrong for all 11 classes at once, which is why it
is ported once, generically, rather than copied per class.

## Proof

- `cargo build --workspace`, `cargo test --workspace`, `cargo fmt --check`.
- `just port-check` (includes `just port-status --per-commit` and the per-marker assertions).
- `just port-status`: the unit's two rows both derive `ported`, or this plan's first risk is
  reported instead with the real count the comparison test passed.
- `just inventory-check`, `just ownership-check`, `just check`, `just guard`.
- The pull request body records both adversarial review passes (run sequentially in this
  implementer's own context -- no separate-context reviewer mechanism is available here, stated
  plainly rather than represented as two independent reviewers, following
  `docs/WORKFLOW.md`'s "Trial-run record" precedent for the same limitation) against the vendored
  source, and this task's manifest rows.
