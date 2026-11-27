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

**The true size, measured before any Rust was written -- corrected again here against a prior
miscount.** The unit is 17,247 source lines, but that undercounts the work, and an earlier version
of this plan undercounted it a second way by treating one raw grep count as the registry size.
Measured precisely: `grep -nP '^\s*def\s*=\s*this->add' PrintConfig.cpp | wc -l` finds **981**
`this->add(...)`/`this->add_nullable(...)` call sites in the whole file -- but **848** of them are
on lines before `CLIActionsConfigDef::CLIActionsConfigDef()` at `PrintConfig.cpp:12713`, i.e.
inside `PrintConfigDef`'s own `init_*` functions, and the remaining **133** belong to the file's
other `ConfigDef` subclasses (`CLIActionsConfigDef`, `CLITransformConfigDef`, `CLIMiscConfigDef`
and the slicing-state/placeholder defs from `:12713` on) and never enter `print_config_def` at all.
Inside the `PrintConfigDef` region, not every add site is a plain literal key either: at least
three are commented out (`:4069` `adaptive_layer_height`, `:4474` `spaghetti_detector`, `:6426`
`filament_extruder_id`) and several are generated at run time rather than written as a literal
string -- a `for` loop over `filament_extruder_override_keys` (`:97-122`, 19 entries, added via
`add_nullable` at `:8788`), a loop of three string-concatenated `machine_max_*` prefixes
(`:5388,5406,5424`), a `for` loop of three `*_plugin_config_overrides` keys (`:1305`), and three
`add_belt_remap` calls building `gcode_remap_x/y/z` (`:7785-7791`). The registry's own key count --
measured by `oracle_option_keys()` against the real oracle library, on this branch and
independently reproduced by both review passes below -- is **881** unique keys, which is the
number this plan now uses as the comparison test's denominator everywhere below; 981 is a call-site
count across several different registries, not the size of `print_config_def`, and this plan no
longer uses it as a key-count gate. `PrintConfig.hpp` separately declares, per
`CONFIG_OPTION_ENUM_DEFINE_STATIC_MAPS`'s own 65 invocations in the `.cpp`, **65 enum types** (see
Step 2 below for which are in scope), several with their own helper functions reaching real logic,
not just values, and, through the `PRINT_CONFIG_CLASS_DEFINE`/`PRINT_CONFIG_CLASS_DERIVED_DEFINE`
macro family (`:1213-1276`), **11 `StaticPrintConfig`-derived classes** (`PrintObjectConfig`,
`PrintRegionConfig`, `MachineEnvelopeConfig`, `GCodeConfig`, `PrintConfig`, `FullPrintConfig`,
`SLAPrintConfig`, `SLAPrintObjectConfig`, `SLAMaterialConfig`, `SLAPrinterConfig`,
`SLAFullPrintConfig`) each declaring, as **typed struct fields**, a second, independent list of
the option keys it carries -- a list this plan must transcribe in addition to the 881-row registry,
not instead of it, because the two lists serve different jobs (the registry's metadata vs. a
config object's typed storage) and disagree in membership (`FullPrintConfig` and
`SLAFullPrintConfig` declare **no fields of their own** -- they only aggregate their parents).
So the real transcription surface is on the order of 881 metadata rows plus ~900 struct-field
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
can tell exactly which slice of the 881+900 declarations is in and which is not, and the pull
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
are mechanically checked against the oracle's own `print_config_def`.

**CORRECTED, against what building the shim proved: there is no such split, and the
GUI-presentation fields are mechanically checked too.** An earlier version of this paragraph said
that `label`, `full_label`, `tooltip`, `sidetext`, `category`, `mode`, `height`, `width`,
`gui_type` and `gui_flags` "are not separately exercised by any oracle entry point" and had to be
left to a `// PORT SRC:` marker per value plus the two-pass adversarial review. That is wrong.
`ConfigOptionDef` carries all of them as ordinary members, so a shim that reads the whole struct
rather than six of its fields compares them like any other, and the strings compare **bytewise**
because `L(s)` is the identity marker inside libslic3r (see this plan's own `L(s)` section below,
and `I18N.hpp:31` for the libslic3r-wide definition the `.cpp`'s own `#define` agrees with). The
shim therefore exposes every field of `ConfigOptionDef` that the registry's `init_*` bodies can
set -- 29 of them, less `opt_key` (the lookup key) and `serialization_key_ordinal` (drawn from a
process-wide atomic in `slic3r-config`, so not a property of the transcription).

Correcting this is not a tidying-up. Measured on the first run with the wider shim, the
transcription came back with **23 real defects in fields the six-field version could not see** --
`L_CONTEXT`'s second argument concatenated into 19 `sidetext`/`label` values, and whitespace
collapsed inside three multi-line strings. Every one would have shipped green under the split this
paragraph used to describe. So the risk `task.md` asks this plan to resolve is answered the other
way round from how it was first written: "14,438 unreviewable lines" becomes **881 rows all 29 of
whose fields are mechanically checked**, with the adversarial review left to read what a bytewise
comparison cannot judge -- whether a `// FIDELITY(port):` justification holds, whether a
`// PORT NOTE:` describes the vendored line correctly, and whether a divergence is recorded in the
right place.

**Shim shape**, in `cxx/proof_units/print_config.{h,cpp}` -- as actually built in this task's first
commits, corrected here from an earlier draft of this plan that named five per-field accessors:

- `oracle_option_keys() -> Vec<String>`: the full key set, in `print_config_def.options`' own
  iteration order (a `std::map<std::string, ConfigOptionDef>`, so alphabetical -- itself a
  `// FIDELITY(port):` site, since the Rust port's own table must iterate the same way wherever
  that order reaches output).
- `oracle_option_info(key) -> OracleOptionInfo`: one struct per key, not five separate functions,
  holding `found` (false when the oracle's registry has no such key), `option_type`, `nullable`,
  `cli`, `has_default`/`default_serialized`, `enum_values` and `enum_keys_map` (a `Vec` of
  `(name, value)` pairs, empty for a non-enum option) -- `ConfigOptionDef::default_value->serialize()`,
  or `has_default = false` for a null default.

**One comparison test, over the whole key set, asserted at exact equality -- corrected here from an
earlier version of this plan that described a check that could not fail.** `print_config_proof.rs`
calls `oracle_option_keys()` once and, for every one of the 881 keys, asserts the port's own
`print_config_def()` agrees with `oracle_option_info(key)` at exact equality -- never a subset the
implementer happened to finish, which would be weakening the check rather than proving it, and
never a check that merely *reports* a count while passing regardless, which task.md's R19 ("held to
exact equality with no tolerance") and this task's own honesty rules both forbid: a test that is
green while the table agrees with the oracle on nothing is a false-green waiting to be read as
progress. An earlier draft of this plan said the test would be "passing-against-an-empty-table
(0 of 981) before any option is transcribed" and would run "independent of whether `lib.rs` gains
the `mod print_config;` line" -- both wrong, and corrected together because the second explains why
the first was tempting: a `tests/print_config_proof.rs` integration test can only call
`slic3r_printconfig::print_config::...` once `lib.rs` actually declares `mod print_config;` (Rust
has no way to reach a sibling-crate-visible module that is not `mod`-declared), so the test cannot
compile, let alone run, before that line lands -- which is also the line `task.md`'s own rule says
must not go in over a live `todo!()`. The two rules together mean this test is **written** in Step
1 (so its assertions are settled before any option is transcribed, and a reviewer can read what
"exact equality" means here without waiting for Step 7) but is only ever **compiled and run** once
the module is complete enough to add `mod print_config;` without a live `todo!()`/`unimplemented!()`
-- i.e. at Step 7, atomically with the `mod` line, not before. There is no intermediate state in
which `cargo test --workspace` both includes this test and is allowed to fail: if Step 5's
transcription is incomplete when the branch has to stop, the honest outcome is the one `docs/
WORKFLOW.md` already states -- the `mod` line and this test both stay out of the tree (uncommitted
or `#[ignore]`d, never committed as a passing check against a partial table), and the pull request
reports the fraction some other way (a count the author measured against a throwaway, uncommitted
run, named plainly as such, the way this task's own pull request already does for the 881-key
shim-only probe).

## The class hierarchy and the one raw-pointer row

**`StaticCache<T>`/`StaticPrintConfig` (`:1041-1135`), in scope here because `PrintConfig.hpp`
declares it, not `Config.hpp`.** The vendored mechanism stores every option as a **typed struct
field** on the concrete config class, then maps an option key to that field by a **byte offset**
computed once (`opt_add`, storing `(const char*)&opt - base_ptr`) and read back through
`reinterpret_cast<ConfigOption*>((char*)owner + offset)` -- raw pointer arithmetic over the class's
own memory layout, not a map of `Box<dyn ConfigOption>`. `port/ownership.tsv`'s one row for this
unit (`PrintConfig.hpp:1132`, `StaticCache::m_defaults`, `T *`, class `borrowed`, verbatim type
`&'a mut T`) names exactly this mechanism's cached defaults pointer.

**Corrected here against the built code: the byte-offset translation does not work, and the
offsets are not kept at all.** An earlier version of this section said each concrete class would be
`#[repr(C)]` with `StaticCache<T>` recording each field's byte offset through
`std::mem::offset_of!`, and `optptr` would be one `unsafe fn` doing the vendored pointer
arithmetic behind a `// SAFETY:` comment. That is not implementable. The vendored `optptr` is
`reinterpret_cast<ConfigOption*>((char*)owner + offset)` (`PrintConfig.hpp:1090`), which works only
because every option field is a C++ class with `ConfigOption` as a base, so a byte address already
*is* a valid `ConfigOption*`. Rust's `&dyn ConfigOption` is a **fat** pointer: it carries a vtable
chosen by the field's concrete type, and no byte offset alone can produce one. The concrete type has
to be known where the borrow is formed, which is exactly what a per-field function pair is.

So the translation actually built is: `StaticCacheBase<T>` maps each option key to an
`OptionAccessor<T>`, a pair of `fn(&T) -> &dyn ConfigOption` / `fn(&mut T) -> &mut dyn ConfigOption`
generated beside the field declaration, and `optptr` is the map lookup plus a call. Keeping the
offset beside it would buy one `unsafe` block and a `// SAFETY:` invariant that is wrong for all 11
classes at once if it is wrong at all -- this plan's own named second risk, below -- in exchange for
a number nothing observable reads: the offsets never reach `keys()`, `optptr`'s result, `hash()`,
`operator==` or `apply_to()`, and this unit's proof compares the registry, never a
`StaticPrintConfig` instance. `port/ownership.tsv` has no row for `m_map_name_to_offset` either (it
is a `std::map<std::string, ptrdiff_t>`, not a raw-pointer field), so nothing in the committed
inputs asks for the offset form. **This unit therefore adds no `unsafe` block and no `// SAFETY:`
marker of its own, and the plan's second risk below is retired rather than mitigated.**
`m_defaults` still carries `port/ownership.tsv`'s cell verbatim (`&'a mut T`, no `Option<_>`); the
vendored null-pointer "not yet initialized" state becomes the enclosing `OnceLock`/`thread_local`'s,
so the cache is constructible only through `finalize` and the reference can never be null, while
`initialized()` still reads `! m_keys.is_empty()`.

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

`PrintConfig.cpp:90` -- `#define L(s) (s)` -- is this file's own definition (corrected from `:91`,
which is `#define _(s) Slic3r::I18N::translate(s)`). It does not shadow a gettext-backed `L`:
`I18N.hpp:31` defines `L` for the whole of libslic3r as
`const char* L(const char* s) { return s; }`, the identity extraction marker, and only `_u8L`
(`I18N.hpp:33`) goes through `Slic3r::I18N::translate`. So every `def->label = L("...")`,
`->tooltip = L("...")` etc. in this unit is the literal string with no I18N call either way --
this unit adds no dependency on `crates/slic3r-i18n` and every `L("...")` becomes
`"...".to_string()` directly.

**`L_CONTEXT` is not the same function, and getting that wrong cost 19 rows.**
`L_CONTEXT(s, context)` (`I18N.hpp:32`) is
`const char* L_CONTEXT(const char* s, const char* context) { return s; }` -- it returns its
**first** argument and discards the disambiguation context. `PrintConfig.cpp` uses it 19 times
(`def->sidetext = L_CONTEXT("s", "second")` and the like), and treating it like `L` by
concatenating both arguments transcribes `"ssecond"` where the oracle holds `"s"`. Caught by the
widened shim, not by reading.

## Order of work

Each step names the slice a reviewer can accept or reject on its own, even though the task's
"Done when" gate (`mod print_config;`, `ported` status) is all-or-nothing across every step.

**The numbered order below is the order the steps are described in, not the order they can be
built in -- corrected here against the build.** Steps 3 and 4 cannot come before step 5:
`StaticPrintConfig::def()` (`PrintConfig.hpp:1047`) returns `&print_config_def`, and
`StaticCache::finalize` (`:1105-1129`) reads `defs->options` and each option's `default_value`, so
neither compiles -- let alone can be checked -- before the registry exists. The built order is
therefore 2, 5 (the registry and `init_common_params`), 3, 6, then 4 and the rest of 5. Step 6
moved up with step 3 for the same reason in reverse: `StaticPrintConfig`'s `handle_legacy` override
delegates to `PrintConfigDef::handle_legacy`, so step 3 does not compile without step 6.

### Step 1: the proof shim, and the whole-key-set comparison test's assertions settled (not yet running)
- `cxx/proof_units/print_config.{h,cpp}`, `src/proof_units/print_config.rs` are written and
  compile/link/run on their own (no module to reach). `tests/print_config_proof.rs` is written in
  this step too -- its exact-equality assertions over all 881 oracle keys are settled before any
  option is transcribed, so a reviewer can read what the proof checks without waiting for Step 7 --
  but per "The proof design" above it cannot compile until `lib.rs` carries `mod print_config;`,
  so it is not yet part of `cargo test --workspace`'s run; it is wired in at Step 7, atomically with
  the `mod` line, once it is expected to pass in full.
- delivers: none (step 20 is groundwork).

### Step 2: the ~63 enums and their helper functions
- Larger than first scoped from `PrintConfig.hpp:52-839` alone (~40 enums): `PrintConfig.cpp`'s
  `CONFIG_OPTION_ENUM_DEFINE_STATIC_MAPS` invocations name 65 types. Corrected here from an earlier
  version of this plan, which wrongly said two of them were declared outside this unit's `.hpp`:
  only **one**, `NozzleType` (`vendor/OrcaSlicer/src/libslic3r/CommonDefs.hpp:12`), is actually
  external and needs a `// TODO(port):` at its first use rather than a guess at its home file.
  `SLAMaterialSpeed` is declared inside this unit's own header, at `PrintConfig.hpp:2343`
  (`enum SLAMaterialSpeed { slamsSlow, slamsFast };`) -- outside the `:52-839` range this plan
  first scoped enums from, but still this unit's to port, not a `// TODO(port):` target. Finite
  and fully bounded either way; ported in full, including `is_separable_infill_pattern` and its
  siblings, which are real logic, not plain value lists.
- **`ConfigOptionEnumValue`'s `from_i32` is an open design question, not a mechanical one.**
  `grep -rl "ConfigOptionEnumValue" crates/` (outside `slic3r-config/src/config.rs`, which
  declares the trait) finds no implementer anywhere in the workspace -- this step is the first to
  implement it for a concrete type. The vendored `ConfigOptionEnum<T>::set_int`/`get_int`
  round-trip through a bare `static_cast<T>(int)`, a reinterpretation C++ allows even for an
  out-of-range value; Rust's nearest equivalent, `std::mem::transmute` from the repr type, is
  undefined behaviour the instant the value does not match a declared discriminant, so it is not
  a faithful-and-safe translation of the same operation. Decided: a `match` covering every variant
  with a named fallback (the enum's own zero/default variant) and one `// PORT NOTE:` recording
  the divergence, applied the same way across all enums rather than re-decided per type.
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
- **Blocked on a missing capability in `slic3r-config`, measured not assumed.** The visitor's
  `hash()` and `operator==` are reachable: `ConfigOption` declares `hash_value()` and `eq_dyn()`
  (`crates/slic3r-config/src/config.rs:1218,1227`). Its `operator<`
  (`PRINT_CONFIG_CLASS_COMMON_BODY`, `PrintConfig.hpp:1201-1206`) is **not**: the vendored body
  compares two concrete option fields with `a < b`, and `slic3r-config` gives the `ConfigOption`
  trait no ordering method and its concrete option types no `PartialOrd` -- `grep -c PartialOrd`
  over that module returns 1, the single `impl PartialOrd for FloatOrPercent`. Reimplementing an
  ordering per option type here would be inventing a Rust type's semantics for a sibling's module,
  which `task.md`'s "Write only your own modules" and `docs/PORTING.md`'s banned-substitution rule
  both forbid. So when this step lands, `operator<` carries a
  `// TODO(port): src/libslic3r/PrintConfig.hpp:1201` naming that gap, and the gap itself is a
  finding against `slic3r-config` for the pull request -- not a tolerance or a stand-in invented on
  this branch.
- delivers: none.

### Step 5: the option table, by the vendored `init_*` grouping, each its own reviewable slice
- `init_common_params`, `init_fff_params`, `init_extruder_option_keys`,
  `init_filament_option_keys`, `init_sla_params`, in that order (smallest first). Each option's
  `// PORT SRC:` lines are checked against the vendored source line a scratch extraction script
  records (`oracle/.cache/tmp/`, never committed, per `task.md`'s own scratch-file rule) -- a
  reliability aid for the registry's 881 rows' line numbers, not a generator of the Rust itself: `docs/WORKFLOW.md`'s
  must-not-write list forecloses a committed generator under `port/`, and this script never
  produces committed output on its own.
- delivers: none.

### Step 6: `handle_legacy`/`handle_legacy_composite`
- `PrintConfigDef::handle_legacy`'s and `handle_legacy_composite`'s own renamed/obsoleted-key
  branches, read and ported after Step 5 so every key they rename or drop is already in the
  table they rename or drop it from.
- delivers: none.

### Step 7: wire the module in, only once steps 1-6 compile, hold no live `todo!()`/`unimplemented!()`, and the Step 1 test is fully green over all 881 keys
- Add `mod print_config;` to `crates/slic3r-printconfig/src/lib.rs` at its alphabetical position
  (`pub mod print_config;`, matching every sibling crate's own `lib.rs`).
- **Corrected here: the `mod` line and the Step 1 proof test are two separate gates, not one.**
  This step originally bundled them, and said the `mod` line is "not added at all if any earlier
  step is short". That conflated the rule with its purpose. `docs/WORKFLOW.md`'s rule is **never
  land a `mod` line over a live `todo!()`**, because a `mod`-reached `todo!()` derives
  `not-started` and so hides in the burn-down. It says nothing about completeness, and
  `derive_row_status` reads `drafted`, never `ported`, for a `mod`-reached module whose
  `// PROOF:` marker is absent. So the two gates are:
  - **The `mod` line** lands as soon as the module compiles with no live
    `todo!()`/`unimplemented!()`. Landing it then is strictly better than withholding it: the
    module and its in-module `#[cfg(test)]` oracle comparisons are compiled and run by
    `cargo build --workspace`/`cargo test --workspace` and therefore by CI, so every figure this
    task's pull request reports is reproducible from the tree instead of only from the session
    that measured it. Withholding it leaves thousands of committed lines that no check in
    `.asdlc/checks` ever compiles.
  - **The `// PROOF: 3` test** (`crates/slic3r-printconfig/tests/print_config_proof.rs`,
    asserting exact equality over all 881 keys) lands only when the table is complete, for the
    reason "The proof design" above gives: a version narrowed to the rows that happen to be
    finished is weakening the check, and committing the full one early is committing a failing
    test. Until it lands the rows derive `drafted` and `./port/port-status` reports them in
    `in-scope rows remaining`, which is the honest signal.
  The in-module `#[cfg(test)]` tests are **not** that proof and carry no `// PROOF:` marker. They
  assert only what is true of a partial table and stays true as it fills -- every transcribed key
  exists in the oracle and agrees with it exactly; every key map the registry uses matches byte
  for byte; `handle_legacy` agrees for every case whose surviving key the registry knows -- and
  they were mutation-checked rather than assumed: changing one enum key name, one option default
  and one legacy rename makes them fail, and changing one `def->max` is caught instead by
  `port/port-status`'s `// PORT SRC:` line check, which is the complementary half (`min`, `max`,
  `max_literal` and the GUI fields are not fields the shim exposes).
- delivers: none.

## Risks

**The real risk is Step 5's volume, not Steps 1-4's design.** Steps 1-4 are one-time, bounded design
decisions this plan has already settled by measurement (the `nm` count, the `L(s)` macro, the
`PRINT_CONFIG_CLASS_DEFINE` expansion). Step 5 is 881 rows of transcription that cannot be
reviewed as a block; the Step 1 test (compiled and run only at Step 7, per "The proof design"
above) is what makes each row's default/type/enum/cli checkable on its own instead of trusted. If
Step 5 does not finish inside this task's branch, the honest outcome is: `mod print_config;` and
`tests/print_config_proof.rs` both stay out of the committed tree, `port/port-status` reports the
unit `drafted`, and the pull request states whatever fraction was measured against an uncommitted,
throwaway run at push time -- never a committed test asserting agreement on fewer than all 881
keys, which would weaken the check task.md's R19 requires.

**A second, smaller risk -- retired, not mitigated.** This risk read: "the `StaticCache` translation
(Step 3) is this unit's only `unsafe` design of its own; a wrong `// SAFETY:` invariant there is
wrong for all 11 classes at once." Step 3 as built has no `unsafe` block and no `// SAFETY:` marker
at all (see "The class hierarchy and the one raw-pointer row" above for why the byte-offset design
that would have needed one is not implementable in Rust), so there is no invariant to get wrong.
`./port/port-status --per-commit` confirms it: this unit contributes zero to the `SAFETY` marker
count.

**A third risk this plan did not name, found by building against the real `slic3r-config`.** Three
of this unit's translations are blocked, each by something `59-foundation-config`'s module decided
rather than by anything in this unit, and each is a finding for the pull request rather than a
workaround on this branch: `ConfigOption` carries no `Send`/`Sync` bound, so `print_config_def`
cannot live in a `static` and is leaked once per thread instead; the orphan rule forbids this crate
implementing `ConfigOptionEnumValue` for `PrinterTechnology`,
`ForwardCompatibilitySubstitutionRule` (both `slic3r-config`) or `NozzleType`
(`slic3r-commondefs`), so `printer_technology`'s default is a `ConfigOptionEnumGeneric` over the
same key map -- which serializes to the same `"FFF"`, verified against the oracle, not argued; and
the missing ordering named under Step 4 above.

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
