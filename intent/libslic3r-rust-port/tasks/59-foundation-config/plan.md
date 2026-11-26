---
kind: plan
---
# Plan: foundation-08 -- `Config.cpp`/`Config.hpp` (the `ConfigOption` hierarchy and `ConfigBase`/`DynamicConfig`/`StaticConfig`)

## Files that change

- `crates/slic3r-config/src/config.rs` (new)
- `crates/slic3r-config/src/lib.rs`
- `crates/slic3r-config/tests/config_proof.rs` (new)
- `crates/slic3r-ffi/cxx/proof_units/config.h` (new)
- `crates/slic3r-ffi/cxx/proof_units/config.cpp` (new)
- `crates/slic3r-ffi/src/proof_units/config.rs` (new)

`crates/slic3r-ffi/cxx/proof_shims.cpp`, `src/proof.rs` and `cxx/include_vendored.cpp`, named by
`task.md`, do not exist on `main` (`docs/PORTING.md`, "Decisions recorded on 2026-11-25", decision
1, which `docs/WORKFLOW.md`'s "Decisions that govern over a task's file list" makes the governing
text over the task body). The three new `proof_units/` files above are that decision's real route.

## The risk this plan exists to answer

**The `ConfigOption` hierarchy's Rust shape.** Settled by one grep before any code was written:

```
grep -rn "ConfigOptionEnum<" vendor/OrcaSlicer/src/libslic3r/*.hpp
```

`ConfigOptionEnum<T>` is instantiated outside `Config.hpp` -- in `PrintConfig.hpp`, `Preset.hpp`
and `IMEXHelpers.hpp`, each with its own enum type unrelated to anything declared in `Config.hpp`.
The set of `T` this template is instantiated with is therefore **open**, not closed inside this
unit, so a closed Rust enum standing in for the whole `ConfigOption` hierarchy cannot be faithful:
a later fan-out task (`foundation-13`, `PrintConfig`, and every config-carrying module after it)
needs to add its own enum type without this crate changing. The decision is a **trait**:

- `pub trait ConfigOption: fmt::Debug` -- the abstract base, one method per virtual member
  (`type_()`, `serialize()`, `deserialize()`, `clone_box()`, `set()`, `get_int()`, `get_float()`,
  `get_bool()`, `set_int()`, `eq_dyn()`, `hash_value()`, `is_scalar()`/`is_vector()` as default
  methods over `type_()`, `nullable()`, `is_nil()`, `overridden_by()`, `apply_override()`),
  object-safe so `Box<dyn ConfigOption>` is `DynamicConfig`'s storage, exactly as
  `std::unique_ptr<ConfigOption>` is the vendored storage.
- `pub trait ConfigOptionVectorBase: ConfigOption` for the vector-valued pure-virtual surface
  (`vserialize`, the eight `set*`/`append`/`resize`/`clear`/`size`/`empty`/`is_nil(idx)` members).
- `ConfigOptionSingle<T>` and `ConfigOptionVector<T>` as **generic Rust structs** (not traits),
  mirroring the vendored class templates directly -- each concrete option type
  (`ConfigOptionFloat`, `ConfigOptionInts`, ...) is then a concrete instantiation or a thin
  newtype over one, exactly as the vendored source is `ConfigOptionSingle<double>` etc.
- `ConfigOptionEnum<T>` stays generic over a new trait, `ConfigOptionEnumValue`, which a later
  task's concrete enum type implements (`get_enum_names()`/`get_enum_values()`, the two static
  methods `Config.hpp:2128-2130` declares but never defines in this file -- they are specialised
  per enum type elsewhere, e.g. in `PrintConfig.cpp`, which this task does not reach). This task
  defines the trait and the generic `ConfigOptionEnum<T: ConfigOptionEnumValue>`; it instantiates
  it with no concrete `T`, because no enum type is in scope here.
- `ConfigOptionEnumGeneric`/`ConfigOptionEnumsGenericTempl<NULLABLE>` (the erased, `keys_map`-based
  siblings `Config.hpp:2145` says a `DynamicConfig` uses in place of the generic form) are ordinary
  concrete types over `port/ownership.tsv`'s borrowed `&'a BTreeMap<String, i32>` rows -- the
  erasure the vendored source itself already chose, not a port invention.

**How `port/ownership.tsv` governs the six raw-pointer fields.** Taken verbatim, by header path
and line, not from this task's own `task.md` table (which happens to agree, but the file is the
authority -- `docs/PORTING.md`'s "Raw-pointer fields" rule): `ConfigSubstitution::opt_def` ->
`&'a ConfigOptionDef`; `ConfigOptionEnumGeneric::keys_map`, `ConfigOptionEnumsGenericTempl::keys_map`
and `ConfigOptionDef::enum_keys_map` -> `&'a BTreeMap<String, i32>`; `ConfigOptionDef::nocli` ->
`&'static str`; `ConfigDef::by_serialization_key_ordinal` -> `BTreeMap<usize, &'a ConfigOptionDef>`.
All six are class `borrowed`; none is `owned`/`shared`/`ffi`/`unknown`, so none needs `Option<_>`
wrapping (decision 6 of "Decisions recorded on 2026-11-26" applies only to `owned` rows). A real
Rust lifetime cannot always reproduce the vendored code's own non-owning-pointer-that-outlives-its-
referent pattern without the borrow checker objecting at a call site a later fan-out task writes;
where that happens, the fan-out task's own finding, not a reason to invent a different type here.

## `docs/PORTING.md`'s "Error sets" rule, applied

`Config.hpp` declares five of its own exception types, all deriving (directly or through
`ConfigurationError`) from `Slic3r::RuntimeError`, which `crates/slic3r-exception` already mirrors
as `Exception::RuntimeError(String)`. Per the rule ("one Rust error enum ... whose variants mirror
this hierarchy one-for-one"), this task adds **one small enum of its own**,
`pub enum ConfigError { Configuration(String), UnknownOption(String), NoDefinition(String),
BadOptionType(String), BadOptionValue(String) }`, mirroring `ConfigurationError`,
`UnknownOptionException`, `NoDefinitionException`, `BadOptionTypeException` and
`BadOptionValueException` one-for-one, rather than reusing `slic3r_exception::Exception` directly
(that enum has no variant for any of these five -- they are `Config.hpp`'s own subclasses, not
`Exception.hpp`'s). Every vendored `throw` of one of the five becomes `Err(ConfigError::_)`; no
function that can fail at the pin returns a bare value, and no function that cannot fail returns
`Result`. An `assert()` the oracle's release build compiles out (`NDEBUG`) becomes `debug_assert!`,
never a live `panic!` -- `docs/PORTING.md`'s "Error sets" rule, applied exactly as
`crates/slic3r-libslic3r/src/libslic3r.rs`'s `lerp`/`fast_round_up` already do for the same
vendored idiom.

## The proof-path-3 shim: measured, not assumed, and one finding it surfaced

`Config.cpp`'s row (and `Config.hpp`'s) both carry `proof path: 3`: `Config.cpp` has `static` free
functions with no exported symbol to call through an `extern "C"`/`cxx` declaration
(`is_whitespace`, `is_end_of_line`, `is_end_of_gcode_line`, `is_end_of_word`, `skip_word`,
`skip_whitespaces`, `normalize_cli_bool_value`, the template `dynamic_config_iterate`), so a test
translation unit that `#include`s the vendored `.cpp` whole is R19's stated route for a file with
no externally-linked surface to call.

**Measured before writing any Rust**, per `docs/WORKFLOW.md`'s review loop (a workaround that
needs a paragraph of justification is wrong code -- so this was checked, not argued):

1. `nm -m` on the oracle's own `Config.cpp.o`
   (`vendor/OrcaSlicer/build/arm64/src/libslic3r/CMakeFiles/libslic3r.dir/Release/Config.cpp.o`),
   demangled, counting strong (non-`weak`) *defined* external `Slic3r::` symbols the way
   `clipper_z_utils.cpp`'s own header comment documents for its file: **461**, far above
   `clipper_z_utils`'s 2. A real risk, so the next two steps measure it rather than wave it away.
2. A minimal spike shim (`cxx/proof_units/config.{h,cpp}`, `src/proof_units/config.rs`, one
   trivial forwarding function) was compiled and linked through `cargo test -p slic3r-proof` --
   the widest exposure, since `slic3r-proof` re-exports every per-unit bridge and every port
   crate's test binary dev-depends on it (`docs/PORTING.md`, decision 1). Result: clean link, zero
   `nm` duplicate-symbol hits, all existing tests still green. Nothing else in the tree's link
   needs any of `Config.cpp`'s 461 symbols today, so the archive member is never extracted and
   there is no duplicate definition -- the same mechanism `clipper_z_utils.cpp`'s comment states,
   confirmed empirically for this file rather than assumed from that precedent.
3. **The spike's first attempt did not link at all, for a reason worth recording as a general
   finding, not just this unit's own fix.** `#include "Config.cpp"` (quotes) inside a shim file
   itself named `config.cpp` self-included: on a case-insensitive filesystem (default macOS APFS),
   a quote-include first searches the *including file's own directory*, and `"Config.cpp"`
   case-insensitively matches that same directory's `config.cpp` -- the shim's own file --
   before the compiler ever consults `-I`/`-isystem`. Measured directly: `#include nested too
   deeply` and clang's own `-Wnonportable-include-path` pointing at `"config.cpp"` on disk.
   `clipper_z_utils` never hit this (its stem, with underscores, does not collide
   case-insensitively with `ClipperZUtils`); `config`/`Config` is the first unit whose manifest
   module stem and vendored file stem are the *same* word differing only in case. **Fix: an
   angle-bracket `#include <Config.cpp>`**, which skips the including file's own directory
   entirely and resolves only against the compiler's include search path -- which already carries
   `vendor/OrcaSlicer/src/libslic3r` as a plain `-I` entry (`crates/slic3r-ffi/build.rs`). Recorded
   in the pull request body as a general rule for any future unit whose stem collides this way.

One `// PROOF: 3` marker, beside a test naming `config`, covers both of this unit's manifest rows
(the task's own stated rule, since both carry the same cell).

## A scope gap this task's rows expose, and how this plan answers it

`Config.cpp`'s `#include` block names `nlohmann/json.hpp` and
`boost::property_tree::ini_parser.hpp`. Neither is one of `docs/PORTING.md`'s "Linked libraries"
(Clipper2, admesh, libnest2d, qhull, Boost.Polygon Voronoi, NLopt) -- the only libraries R17 binds
through `crates/slic3r-ffi` -- and a port task may add no Cargo dependency
(`docs/WORKFLOW.md`'s must-not-write list; `34-foundation-spike` is the only place a workspace
dependency is added). So `ConfigBase::load_from_json` (both overloads), `load_from_ini`,
`load_from_ini_string`, `load_from_ini_string_commented`, `load(ptree)` and `save_to_json` (both
overloads) depend on a third-party parser this intent has no sanctioned route to. This is a
genuine gap in `docs/PORTING.md`, not a translation this task can get right by trying harder, and
is recorded as a finding in the pull request body for the regeneration pass to answer (most likely
either widening R17's linked-library list to cover a JSON/INI parser bound the same way Clipper2
is, or accepting a hand-written minimal parser as its own hand-designed foundation type -- a
product decision this plan does not make unilaterally).

**What this plan does about it, measured rather than guessed:**

- `ConfigBase::load(file)`'s own body (`Config.cpp:848-863`) calls `load_from_ini` from **no** live
  path -- the only call site is commented out (`:861`) -- so of the whole boundary, only
  `load_from_json` is reachable from any in-scope caller at all, and even that caller is `load()`
  itself, never anything outside `Config.cpp`.
- A caller sweep across every in-scope `.cpp`/`.hpp` (`grep -rl` for each name, cross-checked
  against `port/manifest.tsv`'s in/out column) finds every external caller of
  `load_from_json`/`load_from_ini`/`load_from_gcode_file`/`save_to_json`/`read_cli` in
  `Preset.cpp`/`Preset.hpp`/`PresetBundle.cpp`/`PresetBundle.hpp` -- all four `out: The port's
  configuration input is the oracle's resolved config, not preset JSON. C6 carries it.` No in-scope
  file reaches any of the JSON/INI-dependent methods.
- So this task bodies the full method -- signature, dispatch, every branch that does not itself
  need the parser -- and narrows only the literal parser call, each with a `// PORT NOTE:` at that
  exact boundary (`## Don't-translate list`'s "out-of-scope code reached from an in-scope file"
  rule, applied to a library boundary rather than a vendored file boundary) and a non-panicking
  `Err(ConfigError::_)` return rather than a `todo!()`/`unimplemented!()` -- the
  `libslic3r.rs`/`object_id.rs` precedent for "a real body for a gap no rule on `main` covers yet",
  never a live panic over a path the oracle itself takes.
- `load_from_gcode_file`, `load_from_gcode_string_legacy`, `load_string_map`, `save` (the INI-style
  writer -- it calls `opt_serialize`, never `save_to_json`), `read_cli`, `collect_plugin_manifest`,
  `update_plugin_manifest` and `save_plugin_collection` need no third-party parser at all (checked
  by reading each body) and are ported in full.

## The cereal/Undo-Redo boundary

Every `friend class cereal::access` / `template<class Archive> void serialize(Archive&)` pair in
`Config.hpp`, plus `ConfigOptionDef::load_option_from_archive`/`save_option_to_archive` and
`Config.cpp`'s trailing `CEREAL_REGISTER_TYPE(...)` block, serialise the `ConfigOption` hierarchy
for the Undo/Redo stack. `port/manifest.tsv` carries no row for `UndoRedo.hpp`/`.cpp` (out of
scope), exactly the finding `crates/slic3r-objectid/src/object_id.rs` already recorded for
`ObjectID.hpp`'s identical pattern. Same treatment here: one `// PORT NOTE:` at the boundary, the
archive methods not ported.

## Number formatting

`Config.hpp`'s own float/int serialisation (`ConfigOptionFloat::serialize`, the vector forms, the
percent forms) is `std::ostringstream << value`, not `GCodeFormatter::emit_axis`'s digit-count
pipeline -- `docs/PORTING.md`'s "Number formatting" rule is scoped to the G-code formatter, which
this unit does not reach. Rust's `f64`/`i32` `Display` is locale-independent and round-trips the
shortest decimal exactly as `operator<<` does under `CNumericLocalesSetter`'s C-locale guard
(`load_string_map`'s own use of it), so `{}`/`.to_string()` is the direct port of `ss << value`
here. `LocalesUtils.hpp`'s `float_to_string_decimal_point` (used by `ConfigBase::set(double)` and
the `SetDeserializeItem` float/double constructors) is in a crate `34-foundation-spike` cut but no
task has ported yet (`port/manifest.tsv`'s `LocalesUtils.cpp`/`.hpp` rows: `not-started`) -- this
task's own instruction for an unported dependency applies: a narrow local helper reproducing the
same locale-independent decimal-point behaviour via `to_string()`, with a
`// TODO(port): src/libslic3r/LocalesUtils.hpp:48` marker, named in the pull request.

## Order of work

### Step 1: the `ConfigOption` trait hierarchy and every concrete option type
- `ConfigOption`, `ConfigOptionVectorBase`, `ConfigOptionSingle<T>`, `ConfigOptionVector<T>`,
  `ConfigOptionEnumValue`, `ConfigOptionEnum<T>`, and every concrete type `Config.hpp` declares
  (`Float`, `*Templ<NULLABLE>` pairs, `Int`, `String`, `Strings`, `Percent`, `*Templ`,
  `FloatOrPercent`, `*Templ`, `Point`, `Points`, `Point3`, `PointsGroups`, `IntsGroups`, `Bool`,
  `*Templ`, `EnumGeneric`, `EnumsGenericTempl<NULLABLE>`).
- delivers: none (step 20 is groundwork; see `task.md`).

### Step 2: `ConfigOptionDef`, `ConfigDef`, `ConfigSubstitution(Context)`, `ConfigError`
- The GUI-presentation metadata struct, the option registry, the substitution-recording types and
  this task's own error enum (see "Error sets" above).
- delivers: none.

### Step 3: `ConfigOptionResolver`, `ConfigBase`
- Every non-archive, non-JSON/INI method bodied in full; the JSON/INI boundary methods bodied down
  to the parser call, which returns `Err` with a `// PORT NOTE:`.
- delivers: none.

### Step 4: `DynamicConfig`, `StaticConfig`, `parse_capability_ref`/`PluginCapabilityRef`, the free
  string-escaping functions
- delivers: none.

### Step 5: the proof-path-3 shim and the one `// PROOF: 3` test
- `crates/slic3r-ffi/cxx/proof_units/config.{h,cpp}`, `src/proof_units/config.rs`,
  `crates/slic3r-config/tests/config_proof.rs`. Exercises `Config.cpp`'s own internal-linkage
  helpers (the `is_*`/`skip_*` family and `normalize_cli_bool_value`) against the port's
  equivalents, and the escaping/unescaping free functions (external linkage, reached through a
  direct FFI declaration in the same shim) at exact equality.
- delivers: none.

### Step 6: `mod config;` in `crates/slic3r-config/src/lib.rs`, only once steps 1-5 compile with no
  live `todo!()`/`unimplemented!()` and the proof test passes
- delivers: none.

## Risks

**The riskiest step is 5, not 1.** The hierarchy's shape (step 1) is a one-time design choice this
plan already settled by measurement (the `ConfigOptionEnum<` grep). The proof shim is riskier
because it is this task's only attempt at a brand-new failure mode (the case-insensitive
self-include) that no prior unit's shim could have surfaced, and because `slic3r-proof`'s
re-export means a mistake here is linked into every port crate's test binary, not only this one's.
Both halves of that risk were retired by measurement before any module code was written (the `nm`
symbol count, the spike link, the `#include <Config.cpp>` fix) rather than argued from precedent.

**A second risk, carried into the pull request rather than resolved here:** if `cargo build
--workspace`/`cargo test --workspace` do not pass once the full module is written (as opposed to
the minimal spike, which only forwards one trivial call), the module stays undiscovered by any
`mod` line (`drafted`, not `ported`) and the task's own "Done when" line for `ported` status is
reported unmet, with the real compiler/test output, rather than forced green.

## Proof

- `cargo build --workspace`, `cargo test --workspace`, `cargo fmt --check`.
- `just port-check` (includes `just port-status --per-commit` and the per-marker assertions).
- `just port-status`: the unit's two rows both derive `ported`, or this plan's second risk is
  reported instead with the real failure.
- `just inventory-check`, `just ownership-check`, `just check`, `just guard`.
- The pull request body records both adversarial review passes (run sequentially in this
  implementer's own context -- no separate-context reviewer mechanism is available here; stated
  plainly rather than represented as two independent reviewers) against the vendored source, and
  this task's manifest rows.
