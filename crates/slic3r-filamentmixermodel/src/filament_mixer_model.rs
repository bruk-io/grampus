//! Ports `vendor/OrcaSlicer/src/libslic3r/FilamentMixerModel.hpp` (crate-level level 0,
//! hand-designed by plan step 20's task `52-foundation-common-defs`, `port/foundation.tsv` row
//! `foundation-01`): a header-only degree-4 polynomial pigment-mixing model. No dependent among
//! this task's own units and no `port/ownership.tsv` row (no raw-pointer field at the pin).
//!
//! **Decision (task was silent): one function per overload pair, not a mirrored out-parameter
//! API.** `FilamentMixerModel.hpp:766-804` (`void lerp(..., unsigned char* out_r, ...)`) and
//! `:809-815` (`RGB lerp(...)`, a convenience wrapper over the first) are one algorithm with two
//! C++ calling shapes. The idiom map's "non-const reference out-parameter" row bans an adopted
//! `&mut` parameter "just to mirror the C++ signature", so this module ports only the shape that
//! rule allows -- a single [`lerp`] returning [`Rgb`], doing the out-parameter version's exact
//! arithmetic internally. Every value either C++ overload can produce, [`lerp`] produces too.
//!
//! **Fidelity notes:**
//! - `compute_poly_features`'s inner exponentiation (`:741-742`) is reproduced as the same
//!   fixed-count multiplication loop, never `f64::powi`, which is not guaranteed bit-identical to
//!   repeated multiplication and the oracle is built `-ffp-contract=off` specifically so no
//!   compiler is free to refactor this kind of arithmetic either.
//! - The dot product's summation order (`:790-794`, by feature index `i`, same order `COEF`'s
//!   own row order) is kept exactly -- floating-point addition is not associative, so a
//!   differently-ordered sum can differ in its low bits; this is this unit's one
//!   `// FIDELITY(port):` site.
//! - `t` (an `f32` parameter, `:768`) is widened to `f64` only because `:783`'s own
//!   `static_cast<double>(t)` does -- the port never widens a float of its own accord
//!   (`### Float widths`), it transcribes the oracle's own explicit cast.
//! - `static_cast<int>(sum)` (`:796`) truncates toward zero for every finite `sum` this model's
//!   trained coefficients produce over a valid `u8`/`u8` input range; Rust's `as i32` saturates
//!   instead of invoking C++'s undefined behaviour on a theoretical out-of-range `sum`, which is
//!   strictly safer and never reachable differently for any input in that range -- the immediate
//!   `[0, 255]` clamp (`:797-798`) makes the two casts agree on every value either can produce.
//!
//! **Proof path 1**: `Slic3r::filament_mixer_lerp` (`src/libslic3r/FilamentMixer.cpp:58-64`), an
//! externally linked symbol in `liblibslic3r.a` that forwards its six channel bytes and mixing
//! ratio straight into `::filament_mixer::lerp` with no transformation of its own -- verified by
//! reading `FilamentMixer.cpp:58-64` directly. `crates/slic3r-ffi/src/proof_units/filament_mixer_model.rs`'s
//! `path1_filament_mixer_lerp` is this task's own proof shim for it, reached through
//! `slic3r-proof`'s re-export (`docs/PORTING.md`, "Decisions recorded on 2026-11-25", decision 1).

/// `filament_mixer::detail` (`FilamentMixerModel.hpp:48-750`): the generated coefficient tables
/// and the polynomial-feature expansion, private to this module exactly as the vendored
/// `detail` namespace is private to its header (the idiom map's "anonymous namespace / `static`
/// free function" row applied to a named-but-internal namespace the same way).
mod detail {
    // BEGIN AUTO-GENERATED COEFFICIENTS (FilamentMixerModel.hpp:50-730): transcribed verbatim,
    // one table row per Rust row, each citing its own vendored line -- see this task's pull
    // request body for the generation and verification method (every row's numbers were
    // independently re-extracted from the cited vendored line and compared, with zero
    // mismatches over all 663 rows).
    // `POLY_DEGREE` is unused here, exactly as it is unused anywhere else in the vendored
    // header beyond its own definition (grep confirms) -- a faithfully ported dead constant.
    #[allow(dead_code)]
    pub(super) const POLY_DEGREE: i32 = 4; // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:55
    pub(super) const N_FEATURES: usize = 330; // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:56
    pub(super) const N_INPUTS: usize = 7; // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:57

    // Decision (task was silent): the oracle has no name for this width -- `COEF`/`INTERCEPT`'s
    // own column count and `lerp`'s per-channel loop bound (`:790`, `for (int c = 0; c < 3;
    // ++c)`) are both the literal `3` with no macro or constant behind it. Naming it here is an
    // idiom choice with no behaviour change (every array it sizes holds the same values either
    // way), so it is not the kind of workaround `docs/WORKFLOW.md`'s "The rejection rule" bans;
    // it exists only so `POWERS`/`COEF`/`INTERCEPT`'s own declaration lines carry no bare literal
    // needing a second `// PORT SRC:` of their own beyond the one already on each data row.
    pub(super) const N_CHANNELS: usize = 3; // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:790

    #[rustfmt::skip]
    pub(super) static POWERS: [[i32; N_INPUTS]; N_FEATURES] = [
        [0, 0, 0, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:60
        [1, 0, 0, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:61
        [0, 1, 0, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:62
        [0, 0, 1, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:63
        [0, 0, 0, 1, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:64
        [0, 0, 0, 0, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:65
        [0, 0, 0, 0, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:66
        [0, 0, 0, 0, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:67
        [2, 0, 0, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:68
        [1, 1, 0, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:69
        [1, 0, 1, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:70
        [1, 0, 0, 1, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:71
        [1, 0, 0, 0, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:72
        [1, 0, 0, 0, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:73
        [1, 0, 0, 0, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:74
        [0, 2, 0, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:75
        [0, 1, 1, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:76
        [0, 1, 0, 1, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:77
        [0, 1, 0, 0, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:78
        [0, 1, 0, 0, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:79
        [0, 1, 0, 0, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:80
        [0, 0, 2, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:81
        [0, 0, 1, 1, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:82
        [0, 0, 1, 0, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:83
        [0, 0, 1, 0, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:84
        [0, 0, 1, 0, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:85
        [0, 0, 0, 2, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:86
        [0, 0, 0, 1, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:87
        [0, 0, 0, 1, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:88
        [0, 0, 0, 1, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:89
        [0, 0, 0, 0, 2, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:90
        [0, 0, 0, 0, 1, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:91
        [0, 0, 0, 0, 1, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:92
        [0, 0, 0, 0, 0, 2, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:93
        [0, 0, 0, 0, 0, 1, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:94
        [0, 0, 0, 0, 0, 0, 2], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:95
        [3, 0, 0, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:96
        [2, 1, 0, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:97
        [2, 0, 1, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:98
        [2, 0, 0, 1, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:99
        [2, 0, 0, 0, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:100
        [2, 0, 0, 0, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:101
        [2, 0, 0, 0, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:102
        [1, 2, 0, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:103
        [1, 1, 1, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:104
        [1, 1, 0, 1, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:105
        [1, 1, 0, 0, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:106
        [1, 1, 0, 0, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:107
        [1, 1, 0, 0, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:108
        [1, 0, 2, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:109
        [1, 0, 1, 1, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:110
        [1, 0, 1, 0, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:111
        [1, 0, 1, 0, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:112
        [1, 0, 1, 0, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:113
        [1, 0, 0, 2, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:114
        [1, 0, 0, 1, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:115
        [1, 0, 0, 1, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:116
        [1, 0, 0, 1, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:117
        [1, 0, 0, 0, 2, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:118
        [1, 0, 0, 0, 1, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:119
        [1, 0, 0, 0, 1, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:120
        [1, 0, 0, 0, 0, 2, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:121
        [1, 0, 0, 0, 0, 1, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:122
        [1, 0, 0, 0, 0, 0, 2], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:123
        [0, 3, 0, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:124
        [0, 2, 1, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:125
        [0, 2, 0, 1, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:126
        [0, 2, 0, 0, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:127
        [0, 2, 0, 0, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:128
        [0, 2, 0, 0, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:129
        [0, 1, 2, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:130
        [0, 1, 1, 1, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:131
        [0, 1, 1, 0, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:132
        [0, 1, 1, 0, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:133
        [0, 1, 1, 0, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:134
        [0, 1, 0, 2, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:135
        [0, 1, 0, 1, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:136
        [0, 1, 0, 1, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:137
        [0, 1, 0, 1, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:138
        [0, 1, 0, 0, 2, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:139
        [0, 1, 0, 0, 1, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:140
        [0, 1, 0, 0, 1, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:141
        [0, 1, 0, 0, 0, 2, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:142
        [0, 1, 0, 0, 0, 1, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:143
        [0, 1, 0, 0, 0, 0, 2], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:144
        [0, 0, 3, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:145
        [0, 0, 2, 1, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:146
        [0, 0, 2, 0, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:147
        [0, 0, 2, 0, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:148
        [0, 0, 2, 0, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:149
        [0, 0, 1, 2, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:150
        [0, 0, 1, 1, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:151
        [0, 0, 1, 1, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:152
        [0, 0, 1, 1, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:153
        [0, 0, 1, 0, 2, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:154
        [0, 0, 1, 0, 1, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:155
        [0, 0, 1, 0, 1, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:156
        [0, 0, 1, 0, 0, 2, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:157
        [0, 0, 1, 0, 0, 1, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:158
        [0, 0, 1, 0, 0, 0, 2], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:159
        [0, 0, 0, 3, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:160
        [0, 0, 0, 2, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:161
        [0, 0, 0, 2, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:162
        [0, 0, 0, 2, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:163
        [0, 0, 0, 1, 2, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:164
        [0, 0, 0, 1, 1, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:165
        [0, 0, 0, 1, 1, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:166
        [0, 0, 0, 1, 0, 2, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:167
        [0, 0, 0, 1, 0, 1, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:168
        [0, 0, 0, 1, 0, 0, 2], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:169
        [0, 0, 0, 0, 3, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:170
        [0, 0, 0, 0, 2, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:171
        [0, 0, 0, 0, 2, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:172
        [0, 0, 0, 0, 1, 2, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:173
        [0, 0, 0, 0, 1, 1, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:174
        [0, 0, 0, 0, 1, 0, 2], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:175
        [0, 0, 0, 0, 0, 3, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:176
        [0, 0, 0, 0, 0, 2, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:177
        [0, 0, 0, 0, 0, 1, 2], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:178
        [0, 0, 0, 0, 0, 0, 3], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:179
        [4, 0, 0, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:180
        [3, 1, 0, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:181
        [3, 0, 1, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:182
        [3, 0, 0, 1, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:183
        [3, 0, 0, 0, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:184
        [3, 0, 0, 0, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:185
        [3, 0, 0, 0, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:186
        [2, 2, 0, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:187
        [2, 1, 1, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:188
        [2, 1, 0, 1, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:189
        [2, 1, 0, 0, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:190
        [2, 1, 0, 0, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:191
        [2, 1, 0, 0, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:192
        [2, 0, 2, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:193
        [2, 0, 1, 1, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:194
        [2, 0, 1, 0, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:195
        [2, 0, 1, 0, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:196
        [2, 0, 1, 0, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:197
        [2, 0, 0, 2, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:198
        [2, 0, 0, 1, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:199
        [2, 0, 0, 1, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:200
        [2, 0, 0, 1, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:201
        [2, 0, 0, 0, 2, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:202
        [2, 0, 0, 0, 1, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:203
        [2, 0, 0, 0, 1, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:204
        [2, 0, 0, 0, 0, 2, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:205
        [2, 0, 0, 0, 0, 1, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:206
        [2, 0, 0, 0, 0, 0, 2], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:207
        [1, 3, 0, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:208
        [1, 2, 1, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:209
        [1, 2, 0, 1, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:210
        [1, 2, 0, 0, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:211
        [1, 2, 0, 0, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:212
        [1, 2, 0, 0, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:213
        [1, 1, 2, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:214
        [1, 1, 1, 1, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:215
        [1, 1, 1, 0, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:216
        [1, 1, 1, 0, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:217
        [1, 1, 1, 0, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:218
        [1, 1, 0, 2, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:219
        [1, 1, 0, 1, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:220
        [1, 1, 0, 1, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:221
        [1, 1, 0, 1, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:222
        [1, 1, 0, 0, 2, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:223
        [1, 1, 0, 0, 1, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:224
        [1, 1, 0, 0, 1, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:225
        [1, 1, 0, 0, 0, 2, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:226
        [1, 1, 0, 0, 0, 1, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:227
        [1, 1, 0, 0, 0, 0, 2], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:228
        [1, 0, 3, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:229
        [1, 0, 2, 1, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:230
        [1, 0, 2, 0, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:231
        [1, 0, 2, 0, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:232
        [1, 0, 2, 0, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:233
        [1, 0, 1, 2, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:234
        [1, 0, 1, 1, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:235
        [1, 0, 1, 1, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:236
        [1, 0, 1, 1, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:237
        [1, 0, 1, 0, 2, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:238
        [1, 0, 1, 0, 1, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:239
        [1, 0, 1, 0, 1, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:240
        [1, 0, 1, 0, 0, 2, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:241
        [1, 0, 1, 0, 0, 1, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:242
        [1, 0, 1, 0, 0, 0, 2], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:243
        [1, 0, 0, 3, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:244
        [1, 0, 0, 2, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:245
        [1, 0, 0, 2, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:246
        [1, 0, 0, 2, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:247
        [1, 0, 0, 1, 2, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:248
        [1, 0, 0, 1, 1, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:249
        [1, 0, 0, 1, 1, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:250
        [1, 0, 0, 1, 0, 2, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:251
        [1, 0, 0, 1, 0, 1, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:252
        [1, 0, 0, 1, 0, 0, 2], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:253
        [1, 0, 0, 0, 3, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:254
        [1, 0, 0, 0, 2, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:255
        [1, 0, 0, 0, 2, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:256
        [1, 0, 0, 0, 1, 2, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:257
        [1, 0, 0, 0, 1, 1, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:258
        [1, 0, 0, 0, 1, 0, 2], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:259
        [1, 0, 0, 0, 0, 3, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:260
        [1, 0, 0, 0, 0, 2, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:261
        [1, 0, 0, 0, 0, 1, 2], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:262
        [1, 0, 0, 0, 0, 0, 3], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:263
        [0, 4, 0, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:264
        [0, 3, 1, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:265
        [0, 3, 0, 1, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:266
        [0, 3, 0, 0, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:267
        [0, 3, 0, 0, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:268
        [0, 3, 0, 0, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:269
        [0, 2, 2, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:270
        [0, 2, 1, 1, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:271
        [0, 2, 1, 0, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:272
        [0, 2, 1, 0, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:273
        [0, 2, 1, 0, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:274
        [0, 2, 0, 2, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:275
        [0, 2, 0, 1, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:276
        [0, 2, 0, 1, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:277
        [0, 2, 0, 1, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:278
        [0, 2, 0, 0, 2, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:279
        [0, 2, 0, 0, 1, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:280
        [0, 2, 0, 0, 1, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:281
        [0, 2, 0, 0, 0, 2, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:282
        [0, 2, 0, 0, 0, 1, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:283
        [0, 2, 0, 0, 0, 0, 2], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:284
        [0, 1, 3, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:285
        [0, 1, 2, 1, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:286
        [0, 1, 2, 0, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:287
        [0, 1, 2, 0, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:288
        [0, 1, 2, 0, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:289
        [0, 1, 1, 2, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:290
        [0, 1, 1, 1, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:291
        [0, 1, 1, 1, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:292
        [0, 1, 1, 1, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:293
        [0, 1, 1, 0, 2, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:294
        [0, 1, 1, 0, 1, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:295
        [0, 1, 1, 0, 1, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:296
        [0, 1, 1, 0, 0, 2, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:297
        [0, 1, 1, 0, 0, 1, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:298
        [0, 1, 1, 0, 0, 0, 2], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:299
        [0, 1, 0, 3, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:300
        [0, 1, 0, 2, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:301
        [0, 1, 0, 2, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:302
        [0, 1, 0, 2, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:303
        [0, 1, 0, 1, 2, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:304
        [0, 1, 0, 1, 1, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:305
        [0, 1, 0, 1, 1, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:306
        [0, 1, 0, 1, 0, 2, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:307
        [0, 1, 0, 1, 0, 1, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:308
        [0, 1, 0, 1, 0, 0, 2], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:309
        [0, 1, 0, 0, 3, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:310
        [0, 1, 0, 0, 2, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:311
        [0, 1, 0, 0, 2, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:312
        [0, 1, 0, 0, 1, 2, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:313
        [0, 1, 0, 0, 1, 1, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:314
        [0, 1, 0, 0, 1, 0, 2], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:315
        [0, 1, 0, 0, 0, 3, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:316
        [0, 1, 0, 0, 0, 2, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:317
        [0, 1, 0, 0, 0, 1, 2], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:318
        [0, 1, 0, 0, 0, 0, 3], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:319
        [0, 0, 4, 0, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:320
        [0, 0, 3, 1, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:321
        [0, 0, 3, 0, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:322
        [0, 0, 3, 0, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:323
        [0, 0, 3, 0, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:324
        [0, 0, 2, 2, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:325
        [0, 0, 2, 1, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:326
        [0, 0, 2, 1, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:327
        [0, 0, 2, 1, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:328
        [0, 0, 2, 0, 2, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:329
        [0, 0, 2, 0, 1, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:330
        [0, 0, 2, 0, 1, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:331
        [0, 0, 2, 0, 0, 2, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:332
        [0, 0, 2, 0, 0, 1, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:333
        [0, 0, 2, 0, 0, 0, 2], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:334
        [0, 0, 1, 3, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:335
        [0, 0, 1, 2, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:336
        [0, 0, 1, 2, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:337
        [0, 0, 1, 2, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:338
        [0, 0, 1, 1, 2, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:339
        [0, 0, 1, 1, 1, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:340
        [0, 0, 1, 1, 1, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:341
        [0, 0, 1, 1, 0, 2, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:342
        [0, 0, 1, 1, 0, 1, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:343
        [0, 0, 1, 1, 0, 0, 2], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:344
        [0, 0, 1, 0, 3, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:345
        [0, 0, 1, 0, 2, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:346
        [0, 0, 1, 0, 2, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:347
        [0, 0, 1, 0, 1, 2, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:348
        [0, 0, 1, 0, 1, 1, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:349
        [0, 0, 1, 0, 1, 0, 2], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:350
        [0, 0, 1, 0, 0, 3, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:351
        [0, 0, 1, 0, 0, 2, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:352
        [0, 0, 1, 0, 0, 1, 2], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:353
        [0, 0, 1, 0, 0, 0, 3], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:354
        [0, 0, 0, 4, 0, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:355
        [0, 0, 0, 3, 1, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:356
        [0, 0, 0, 3, 0, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:357
        [0, 0, 0, 3, 0, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:358
        [0, 0, 0, 2, 2, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:359
        [0, 0, 0, 2, 1, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:360
        [0, 0, 0, 2, 1, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:361
        [0, 0, 0, 2, 0, 2, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:362
        [0, 0, 0, 2, 0, 1, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:363
        [0, 0, 0, 2, 0, 0, 2], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:364
        [0, 0, 0, 1, 3, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:365
        [0, 0, 0, 1, 2, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:366
        [0, 0, 0, 1, 2, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:367
        [0, 0, 0, 1, 1, 2, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:368
        [0, 0, 0, 1, 1, 1, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:369
        [0, 0, 0, 1, 1, 0, 2], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:370
        [0, 0, 0, 1, 0, 3, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:371
        [0, 0, 0, 1, 0, 2, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:372
        [0, 0, 0, 1, 0, 1, 2], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:373
        [0, 0, 0, 1, 0, 0, 3], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:374
        [0, 0, 0, 0, 4, 0, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:375
        [0, 0, 0, 0, 3, 1, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:376
        [0, 0, 0, 0, 3, 0, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:377
        [0, 0, 0, 0, 2, 2, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:378
        [0, 0, 0, 0, 2, 1, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:379
        [0, 0, 0, 0, 2, 0, 2], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:380
        [0, 0, 0, 0, 1, 3, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:381
        [0, 0, 0, 0, 1, 2, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:382
        [0, 0, 0, 0, 1, 1, 2], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:383
        [0, 0, 0, 0, 1, 0, 3], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:384
        [0, 0, 0, 0, 0, 4, 0], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:385
        [0, 0, 0, 0, 0, 3, 1], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:386
        [0, 0, 0, 0, 0, 2, 2], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:387
        [0, 0, 0, 0, 0, 1, 3], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:388
        [0, 0, 0, 0, 0, 0, 4], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:389
    ];

    #[rustfmt::skip]
    pub(super) static COEF: [[f64; N_CHANNELS]; N_FEATURES] = [
        [8.70954844857314666e-12, 1.27926950848359881e-09, -2.06865474316332923e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:393
        [1.05783308354771544e+00, -8.02119209663359686e-03, -7.88705651445470723e-02], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:394
        [1.35905954452774837e-02, 8.71267975138422468e-01, 1.04898760410704936e-01], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:395
        [-4.16452026099768252e-02, 1.75465381596434100e-02, 1.00224594702931546e+00], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:396
        [4.50321316661211821e-02, -7.11409155427628892e-02, 3.91232300778902690e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:397
        [1.76675507851922452e-02, -1.32709276116036640e-01, 6.36935270589509828e-02], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:398
        [-5.23434830565911030e-02, 3.77681739012521722e-02, -2.08691145087504179e-02], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:399
        [-2.33722556520224792e-03, -1.57542611462692145e-03, -3.05158628452478807e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:400
        [-8.87678609044812990e-04, 3.83194388837734693e-04, 1.37779212442523083e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:401
        [-2.11519042076831979e-03, 5.82337362515735358e-04, 2.24055108941204821e-04], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:402
        [4.61545125563611917e-04, 7.72869451707915893e-04, -1.10800630143346882e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:403
        [1.05937484157345879e-03, -3.14448681732842211e-04, -1.75129182446198098e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:404
        [1.49045689016363055e-03, -2.09220860101674106e-04, 5.93100338908187697e-04], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:405
        [-3.51246656293852696e-04, -8.20743017485394289e-04, 5.71854064480802862e-04], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:406
        [-9.18204643629581319e-01, -2.27788122702773155e-01, 6.39980793022790623e-02], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:407
        [9.24243491377523679e-05, 7.32841332381495400e-04, -1.55219718415109450e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:408
        [7.13695056804217989e-04, -8.46467621879685712e-05, 6.50202947442505750e-04], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:409
        [1.66640864747485983e-03, -1.24492362771216523e-04, 2.68236502346156410e-04], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:410
        [-7.20253644860527516e-04, 7.81434220384157334e-04, 1.12661089007361367e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:411
        [-6.83033334365238206e-05, 7.27742627159490762e-04, -1.78048843835204584e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:412
        [-3.13431571993316588e-02, -8.57604034845650287e-01, -2.57225920656276863e-01], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:413
        [-6.47867200595898341e-05, -1.16688982572457655e-03, 1.14174511750260031e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:414
        [-5.00713925613324338e-04, -6.87598082111323477e-04, 6.20598069880440176e-04], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:415
        [-8.56716727659588957e-05, 9.74478786593559361e-04, -1.65892838405139512e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:416
        [6.53468478750158263e-04, 7.51662000672516676e-04, -6.73196326298856570e-04], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:417
        [-4.42539011000103941e-02, -2.01965359697350230e-02, -9.94663493761314355e-01], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:418
        [-7.39107395392403087e-04, 5.28870828612476996e-04, 1.00947183860234540e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:419
        [-2.06577300933763214e-03, 9.60215813758718011e-04, -3.27993888180819421e-04], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:420
        [3.47783280638377555e-04, 8.41824316850705743e-04, -8.87458944147930993e-04], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:421
        [1.20960551709587905e+00, -7.07660818059813873e-02, -8.56332806008946491e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:422
        [2.11116509318935269e-04, 7.68490846994171776e-04, -1.63228995491542417e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:423
        [6.47698075356516103e-04, -4.20589129268072884e-04, 1.18354001300614896e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:424
        [-2.78795945253848716e-02, 1.22199201000304547e+00, -2.07383075858847743e-01], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:425
        [-5.32457386680677347e-05, -9.58027320315790677e-04, 9.89667309649038679e-04], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:426
        [-9.03932426306289782e-02, -4.00969232187064692e-02, 1.26285611182120072e+00], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:427
        [-2.19453630740322871e-03, -1.21893190049422620e-03, -1.92293368093085417e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:428
        [1.72950845415964505e-06, -8.93952511560151819e-09, -6.14874900641340649e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:429
        [8.02644554976326974e-06, -6.42543741723487294e-06, -6.07103419227907060e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:430
        [3.20307552755319525e-06, -4.83533743093466500e-06, 9.13563764113473065e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:431
        [-2.18105804067510178e-06, 6.19595552598436322e-07, 5.21392855381760945e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:432
        [-2.43310123604345563e-06, 2.17201813434465818e-06, 1.94098874242362718e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:433
        [-1.56293672065252465e-06, 3.95256011818110372e-06, 1.68792962079201969e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:434
        [-1.37567295252127852e-03, 3.59746071987262106e-04, 7.38927139000157259e-05], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:435
        [4.27822004137219658e-06, -8.80187479967658548e-07, 2.29453131891411977e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:436
        [7.68758937964332534e-06, 2.40909410585557829e-07, 4.69351234070854509e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:437
        [-2.87166709944317033e-06, 7.60223902901142716e-07, 4.57864913314467992e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:438
        [-4.01295140267654560e-06, 2.65929275888376483e-06, -2.36575067819565221e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:439
        [2.32693030513910805e-07, 2.28814396769890308e-06, 1.83526107699893970e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:440
        [-2.18213927011287265e-03, 1.65013083920367864e-03, 2.31992998847323087e-04], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:441
        [-7.70829764693697905e-06, 4.23888841240673345e-07, 7.30018322002944087e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:442
        [-1.23111329452911533e-06, 1.50076529718910084e-06, -1.91139744928209288e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:443
        [-1.68872756433485760e-06, 1.03254236824697979e-06, -1.72081108163607555e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:444
        [1.64276928199709460e-06, -4.96350219553231067e-07, -1.46349385185670297e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:445
        [1.12731767057843682e-03, 5.03104281148445223e-04, 1.36398977654308994e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:446
        [-1.05449609518089293e-06, -4.06952115309007489e-07, 3.53062441379482783e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:447
        [-1.98745923822574166e-06, 4.98021943693208180e-07, 3.92645061370218429e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:448
        [-1.55569377977005097e-07, -4.00262856484093037e-07, -2.49609122397048688e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:449
        [2.18005022830924673e-03, -4.10275057064835439e-05, -2.59776311836759947e-04], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:450
        [5.41337439827552225e-07, -1.88603932528607146e-06, -2.06428606152470051e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:451
        [-6.03243799807140491e-06, -3.75067864464502022e-06, -3.05702776851046742e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:452
        [2.30038011634901016e-03, -1.32581161861259635e-03, -1.07680096899188406e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:453
        [4.46773877910556887e-06, 1.85008408528524772e-08, -2.72851357570281713e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:454
        [-1.49177636513049289e-03, -1.91426739654176659e-04, -1.71206384332753194e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:455
        [2.31661325589237743e-02, 2.26540538563063554e-01, 5.42330337046266139e-02], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:456
        [-1.40563059963100256e-06, -4.50551806294901061e-06, 8.87542894832671347e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:457
        [-1.66780916452391459e-06, 4.12065434881171526e-06, -3.55865035776836702e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:458
        [2.71536622051954390e-07, -3.08564858926584692e-06, -1.52164363662402047e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:459
        [2.66659632027280158e-06, -1.19436686895073481e-06, -3.25738306279285683e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:460
        [-1.43666282346327501e-06, -2.51923473623639690e-06, 5.21205120344175876e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:461
        [2.82954522469612199e-04, -1.59147454710008968e-03, 1.27685773978167098e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:462
        [-3.99471240294241303e-06, 9.97323772325767188e-08, -5.28196823261495307e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:463
        [-6.39858432699424995e-06, -4.59897864440506933e-06, -2.39736149785715891e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:464
        [2.89457420106498109e-06, -3.10427512149489757e-06, 9.75553221437691631e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:465
        [-8.96518259720091581e-07, -5.53996694461914366e-06, 1.03733964032237669e-05], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:466
        [8.82130497168875905e-04, -2.33618402105562365e-03, 1.35100410641244379e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:467
        [-2.14088521029685841e-06, 2.59005410360388117e-06, -9.78713171504927426e-08], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:468
        [-4.50668337071552516e-06, 3.58808570076458002e-06, -1.56159349007541082e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:469
        [-1.52345101244247272e-06, 2.21066768791959578e-06, -2.19555898547246775e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:470
        [2.07334042074768356e-03, -1.56333498489329517e-03, -5.53762940364141767e-04], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:471
        [2.22151748134440108e-06, -4.74729938900429749e-07, -3.46744150304684889e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:472
        [2.95389009221172505e-06, -2.96312023445686329e-06, -9.00385068308695580e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:473
        [-6.47780848348620771e-04, 2.38772263398574292e-03, -8.93908589731968019e-04], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:474
        [9.69501567645025819e-07, 2.41432205872957328e-06, 5.56908291093893837e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:475
        [-6.33392066185247586e-04, 2.38613844267241120e-03, -1.05383725637261472e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:476
        [6.76250135616376785e-02, -5.57799579151454852e-02, 1.83393652374666566e-01], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:477
        [3.53986894266120067e-06, 5.92996717102502093e-06, -7.32378536156402804e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:478
        [5.69667193362453916e-06, 1.20219201908705218e-06, -4.56663805956276925e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:479
        [7.11494218295222192e-07, 2.93069858359131137e-06, 1.23210839732268429e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:480
        [-3.41917893741799928e-06, -1.47435291776966751e-06, 1.07397354370819542e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:481
        [7.30931882734254710e-04, 1.15433149094644884e-03, -2.40026982569019722e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:482
        [-1.22780859907432871e-06, 2.29287908084027789e-06, 1.84270754640877832e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:483
        [7.71579140080615178e-07, 2.92378122615943208e-06, -1.91800935486416413e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:484
        [-3.76107279903559188e-07, -1.83159743461489867e-06, 8.17089655984204466e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:485
        [-1.10830882430058061e-03, -5.10908079549339251e-04, -1.77835176235151705e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:486
        [-1.26839781743699406e-06, -2.86942252006448415e-06, 4.47464983859263005e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:487
        [-1.44518716284694482e-06, -7.03360635528004451e-06, 1.04898109513258675e-05], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:488
        [-4.98687888007460470e-04, 1.86990180752567262e-03, -1.24341018156770089e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:489
        [-2.90479801332704790e-06, -9.24272269110706229e-07, 7.56354222045119151e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:490
        [-1.16451534008294149e-03, -2.34216801827852273e-03, 4.91479264672447288e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:491
        [-7.70970926241258958e-02, 9.35855573900774423e-02, 1.50623807158846906e-01], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:492
        [1.14039905307547484e-06, -1.80664235182388840e-07, -5.15527441317074897e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:493
        [7.50559587697416375e-06, -6.23982034686780714e-06, -5.01245198064126721e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:494
        [2.37840954889385892e-06, -4.15663063190341991e-06, 1.93118829429697603e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:495
        [-1.54903048110950777e-03, 2.65832194444263125e-04, 5.34401520444913940e-04], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:496
        [4.00040634507183718e-06, -2.43965474694277443e-06, 2.88683251413283937e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:497
        [7.72301916160400559e-06, -9.54300275625495457e-07, 5.50777546561020959e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:498
        [-2.28103126593574368e-03, 1.02658341009706066e-03, 1.22010567464172614e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:499
        [-6.32818026002207601e-06, 9.83088209200334157e-07, 5.24316808343458507e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:500
        [1.37175660779395581e-03, 4.01188715721313943e-04, 7.59370199245276625e-04], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:501
        [-3.33184694847917573e-01, 7.82846225823195241e-02, -9.94270054263078074e-02], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:502
        [-1.70108770909324636e-06, -5.10749831734438279e-06, 9.80267482880020635e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:503
        [-1.79301365419055891e-06, 4.44839673308561508e-06, -3.83837422072638712e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:504
        [1.71911692904483371e-04, -1.56077480341044431e-03, 1.30725115579017584e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:505
        [-3.55763938679129477e-06, 1.20558966207589408e-06, -5.94340114624253291e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:506
        [1.02325453537648178e-03, -1.52640960762801372e-03, 3.10973117856692537e-04], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:507
        [3.81842873295820109e-03, -3.02114884453467680e-01, 2.78264587142456665e-01], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:508
        [3.46123498726202961e-06, 5.05929187103208375e-06, -6.85764673719752027e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:509
        [4.47228353489932293e-04, 9.60672217798415784e-04, -2.19382758010531077e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:510
        [2.22711833124298791e-01, -4.14141995162802465e-02, -4.27998216564745015e-01], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:511
        [-1.78271151817048783e-03, -9.81039111371464307e-04, -1.37513011841553174e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:512
        [3.35305394560947434e-10, -1.26710751613412498e-09, 3.54248685940916630e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:513
        [-9.26917423371698135e-09, 6.21190912597491263e-09, 1.86942252233812667e-08], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:514
        [-1.56687696151180944e-09, -5.44315731376698864e-09, 1.93822974337010123e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:515
        [7.52897716393974292e-10, -3.48923168136394679e-10, -5.94217786087369859e-10], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:516
        [2.52116855170569920e-10, -2.48216903975251313e-09, 1.01699001303634518e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:517
        [3.72215577457146729e-09, 4.51910314724912610e-10, -6.15361639422218332e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:518
        [-2.62088816666700142e-07, 3.23631086683010168e-07, 8.85302852722882894e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:519
        [-1.30537319842360944e-08, 1.46808588619151692e-08, 2.67574040702101001e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:520
        [-1.23991327621864045e-08, 2.61298349069072344e-08, -4.58919307373337193e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:521
        [5.03079244928983371e-09, -6.73783119575777079e-10, -1.13935871848269699e-08], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:522
        [9.09065785148488459e-09, -1.04304054004966673e-08, -3.23123813816827976e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:523
        [9.55627910137479830e-10, -1.41129563591135820e-08, -1.75594400131373618e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:524
        [-1.05549669436946769e-07, 8.47284096194811896e-08, 6.70761880091491625e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:525
        [-5.92079330008488114e-10, 6.31702118392141188e-09, -4.51534448719925763e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:526
        [-1.04033970327321867e-09, 4.67775485013532943e-09, 2.79348504744758586e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:527
        [5.38758108958869997e-09, -9.55380699552144108e-09, 6.16488249338686956e-11], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:528
        [1.12057409185073453e-09, -3.00645183748393663e-09, -2.14940637510707688e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:529
        [-6.27004681934967278e-07, 8.59159786402940127e-07, 2.73192537668387470e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:530
        [7.36784189214745311e-10, -8.12761968838060511e-10, -2.43226564583531868e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:531
        [1.25546123497244366e-09, -6.98609614602219153e-10, -5.29894812750786315e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:532
        [-8.88351475714088679e-10, 1.37132565025677167e-09, 1.92497813869541012e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:533
        [6.10992637326349119e-07, -6.13496367368217277e-07, -2.19901889726877020e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:534
        [-8.59090437677068053e-11, 2.72772732179404898e-09, 1.54554039011323141e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:535
        [-4.58798915525804318e-10, 4.54384851966693759e-09, 3.63189350816028877e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:536
        [9.93115786933340683e-08, 1.63700862245048928e-07, -1.71397937400244449e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:537
        [-1.62985361318312982e-09, -3.10762126448649312e-09, 1.76193495557419588e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:538
        [6.27207737564569601e-07, -1.49343052365004934e-06, 8.16168870109573730e-08], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:539
        [1.42518738380244172e-03, -3.47531891583186285e-04, -2.98661838800559913e-04], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:540
        [8.98157254125564464e-09, -8.24242643235328920e-09, -5.34769730234363472e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:541
        [-2.17776999489327494e-08, -4.47141107473569832e-09, -1.10218517090920898e-08], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:542
        [3.19614509858290319e-09, -3.32861183754973311e-09, 9.92016746526047655e-11], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:543
        [-2.91660393059167689e-09, 5.59829099744391101e-09, 1.70080685646389895e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:544
        [1.22479524179014421e-09, 9.20737683318684219e-09, -1.10618757209746121e-10], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:545
        [7.70594587548882257e-09, -1.33267446898667659e-06, 4.52812675308736368e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:546
        [9.46080642993951670e-09, -1.95483249032513129e-08, -1.23592694620255905e-08], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:547
        [-2.02330094345448686e-09, 1.18198534293512125e-10, 2.34746776184291406e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:548
        [4.00839940406516604e-09, -4.80716730311137042e-09, 5.25802457129742606e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:549
        [-2.53115202408782380e-09, 2.05563177591017165e-10, 5.46003270374129102e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:550
        [3.24841319972028232e-08, -1.24284705839720552e-06, 4.97326549863015555e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:551
        [1.37729661009444726e-09, -1.67903983772088594e-09, -5.62083748989472554e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:552
        [-3.53256937590806785e-10, 4.49320892992322030e-09, -4.02300486673778934e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:553
        [2.48976475547557641e-09, -6.97256366533061112e-09, 1.43185084622299286e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:554
        [-4.38617299338556199e-09, 9.45081248826811111e-08, -2.91197460585562728e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:555
        [3.24429103026879773e-09, -1.71647943601749287e-09, 2.71076100455402980e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:556
        [3.86933235105302309e-09, -2.82628156988984358e-09, 8.24455756442965537e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:557
        [-7.46614068323353530e-07, 1.27696340529665289e-06, 6.88413034833322557e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:558
        [-5.78118683480788320e-09, 1.34319005917760137e-09, -1.15898873831454807e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:559
        [4.42686972671260670e-07, 6.41810588767341775e-07, -1.16058405342719939e-08], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:560
        [2.24399192788231686e-03, -1.35129336477888174e-03, -7.39944244498236844e-04], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:561
        [7.47869199901884940e-09, -2.68762612165573955e-09, -7.41584788022109365e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:562
        [1.80867308283150230e-09, -2.21500551234043996e-09, 1.86995768869380186e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:563
        [-5.05514829302056157e-09, 4.74048706539109688e-09, 2.52998993977016085e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:564
        [1.32441967115592973e-09, 5.70339246663831290e-09, 7.13448300437846683e-10], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:565
        [1.19767475292940212e-06, 6.72445227582811568e-07, -1.97500319605841551e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:566
        [-1.70612399208458498e-09, 1.07145120553653328e-09, 1.73225882249550267e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:567
        [1.15369127445807962e-09, -5.80362996549510513e-09, 9.33515653667171819e-10], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:568
        [3.38692740520230018e-09, 3.72531013675958533e-09, -3.18062756687886861e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:569
        [1.14787653780236421e-06, -1.84917201319622368e-06, -2.44834286920736499e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:570
        [1.45558928799083276e-09, 1.12720083267348059e-09, 9.00940544390493869e-10], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:571
        [2.09654001104286891e-09, 4.92913422578400429e-09, 3.04938074791039071e-10], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:572
        [3.54033623213741155e-07, 1.07259516691213860e-06, -6.03027205987524684e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:573
        [-2.72038239157446071e-09, -1.60070143945256760e-09, 6.03853855807301443e-10], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:574
        [-2.03235662485238069e-06, -1.03151962834260348e-06, 1.99637918628457062e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:575
        [-1.26261175077493210e-03, -4.98503988506484859e-04, -1.03875859619143593e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:576
        [6.43182729298530376e-10, 8.01776645076301975e-10, -1.83589794755523172e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:577
        [4.01805119037978997e-09, -5.63673552278487477e-10, -1.09102650663883693e-08], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:578
        [-1.48648961195707585e-09, 5.01067861508053269e-09, 2.99132781045319263e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:579
        [-8.91404754824534629e-07, 7.49163968581634775e-07, 2.12542215183124383e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:580
        [2.38642574451608525e-09, -3.47605810802065207e-09, 3.86935566920598717e-10], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:581
        [-2.80031986488182838e-09, -4.25160427697246490e-11, 2.24182921879090280e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:582
        [-1.26991357818351247e-07, -1.45348284568834647e-07, 5.68792533226815389e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:583
        [1.39227229745131353e-09, -1.84849578699353145e-09, 2.24967258190267305e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:584
        [-1.15462500328497586e-06, 1.84347590761761086e-06, 3.64918716654494962e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:585
        [-2.09357112083411985e-03, 1.60820400301404873e-05, 2.27418117008655948e-04], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:586
        [-1.04484803378768198e-08, 4.86043558178828050e-09, 2.00996588123336650e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:587
        [1.44040971927772432e-08, 1.42223015309195233e-09, 1.99778974613318283e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:588
        [-1.62414574166394599e-07, -1.31976785339561840e-06, 4.43918084507000099e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:589
        [3.73061943836905385e-09, 1.00036822436866402e-08, -1.05450977117005351e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:590
        [-2.06551932971539565e-07, -9.72167971235462190e-07, 4.28861904300768815e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:591
        [-2.16051814014425313e-03, 1.48780488507118812e-03, 7.79940397419977911e-04], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:592
        [-4.80544204428667854e-09, -1.09870773590259319e-09, 6.58876991984844174e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:593
        [1.31575045692056136e-06, 4.32430764481131318e-07, -1.55255090541518703e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:594
        [1.28823975640215602e-03, 4.04521283440268135e-04, 1.76186984141882253e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:595
        [-1.09767251093991436e-01, -4.94112205838347640e-02, -5.43102978164306804e-02], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:596
        [7.93691223854864347e-10, 1.54639511196208446e-08, -1.71518303448969789e-08], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:597
        [2.56523843833456056e-09, -2.31047392329486456e-09, -4.29758133398648601e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:598
        [-9.87725901069325118e-09, 4.28127375218245732e-09, 2.02888056355376989e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:599
        [3.21762172461603768e-10, -5.82937505211322815e-09, 3.88293127512318037e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:600
        [1.63250610252241302e-09, -7.02161705168347083e-09, 3.46592492032893329e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:601
        [-1.44272117683086343e-07, -4.40408510988914148e-07, 5.92746408872857344e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:602
        [2.71961467235293242e-09, -1.47466668633244868e-08, 2.89637452632884873e-08], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:603
        [1.47637712476396399e-08, 1.16406781783262581e-09, 2.04904540557215853e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:604
        [-5.53709807865621073e-09, 7.05512286092169205e-09, 1.56159114805820565e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:605
        [5.29268649740455288e-09, 2.10616986628942016e-08, -3.03219004488264332e-08], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:606
        [1.79978890693655025e-07, 7.95085399132693105e-07, -4.78366567607801940e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:607
        [-4.03847393894152251e-10, 2.90357085597214848e-09, 1.12992165623992946e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:608
        [2.99031871486832301e-09, -1.37951879780606745e-09, 2.41048263988075107e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:609
        [1.26882357398550027e-09, 1.30631467101793852e-09, 7.99574240151201820e-10], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:610
        [-1.41169562567489137e-08, 1.27148955713198356e-06, -2.89386439707162157e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:611
        [-2.68794415198003733e-09, 8.73673404455654889e-10, 2.89557382238125882e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:612
        [-4.90264437380538709e-09, 1.89207244316591527e-09, 2.25393465003165261e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:613
        [-3.58274654665979853e-08, 2.91386646529383231e-07, -4.98477764412919022e-08], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:614
        [1.65722165851311942e-09, -1.11673743863338615e-09, -4.14131162695952071e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:615
        [-1.47751280626939874e-07, -2.41471865000848773e-07, -8.53552350049691100e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:616
        [-2.24352957583577790e-04, 1.60900273524284708e-03, -1.32260753549593617e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:617
        [2.05497643901431104e-09, 1.38702982710459111e-08, -3.09887516689033582e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:618
        [3.39770491949997755e-09, 9.41613393506957053e-09, -7.09844738544518350e-10], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:619
        [7.86209687630989862e-10, 1.93556837224662104e-10, -6.58630930350234678e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:620
        [-6.86841181152253455e-10, -5.57194149153339424e-09, 1.41214109156129197e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:621
        [2.59516074158083754e-07, 1.30703181255419770e-06, -4.02454784192984860e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:622
        [-5.79425202262839889e-10, 4.05071760856134944e-09, 3.02384985106929349e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:623
        [4.00677924866643664e-09, -2.25614611715219127e-09, 7.52819043214891792e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:624
        [2.34003759425061020e-09, 5.27462258592681366e-09, -2.05723854618256041e-10], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:625
        [2.29340174767722615e-07, 1.05507868574435809e-06, -4.45904844964539748e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:626
        [-3.91634245866523401e-09, 1.07849931763048801e-09, 1.85542686770290288e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:627
        [-6.62166513287765213e-09, 3.86355018811013196e-09, -1.87861701195224384e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:628
        [1.32112240848469842e-07, 4.39339645861430705e-08, -1.59384598983486336e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:629
        [2.02488462108796341e-09, -1.48427112267590644e-09, -4.32055485832805175e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:630
        [-4.27701540045566375e-07, -1.46229443391283215e-06, -2.38186369433401879e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:631
        [-9.86744509368740232e-04, 1.91104095070606826e-03, -8.17774843405986713e-04], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:632
        [2.06891823117949514e-10, -2.64060942556376688e-09, 1.86419366055012858e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:633
        [8.33785634979378187e-09, -1.00697171434571686e-08, -2.84106664583116952e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:634
        [5.07057938692323518e-09, -9.56246298811080919e-09, -6.33399999117045809e-11], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:635
        [-6.78808357162941078e-08, -2.21612941845184680e-07, 9.42031624998063144e-08], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:636
        [-3.04300065007145903e-09, 5.64120231083542478e-09, 1.65718606892628628e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:637
        [3.76240642807612602e-09, -4.58941407446844529e-09, 5.06162500801821125e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:638
        [7.25149885354159363e-07, -1.18149759075966698e-06, -6.82406347277120240e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:639
        [-4.84358128605144600e-09, 4.56893046833772853e-09, 2.67044331092591847e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:640
        [-2.54939737986958903e-07, -1.06106228658746360e-06, 5.04013386790069795e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:641
        [-2.17097468872509735e-03, 1.41624400187313607e-03, 8.11305605779899562e-04], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:642
        [2.24635331169675823e-10, -6.02144184513875302e-09, 4.15827878380570226e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:643
        [-4.55408258326350790e-09, 6.20319154376325343e-09, 2.08760821823750220e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:644
        [2.10871853867367065e-07, -4.29346688506603014e-07, 1.15683623843482186e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:645
        [1.00732072683129559e-09, 3.88267751283422058e-11, -6.73798626615873530e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:646
        [5.34506627847264326e-09, -8.01262819982717645e-08, 1.60888846226225901e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:647
        [5.83419066552946048e-04, -2.36474094848551555e-03, 8.79373865688287898e-04], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:648
        [-4.85158746510450101e-10, -6.78789624508624456e-09, 4.95385649168511577e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:649
        [3.47485142271342085e-07, 5.60944792101468470e-07, -4.35887910682497548e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:650
        [5.75824910919892421e-04, -2.18618554413632388e-03, 1.22736498224538170e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:651
        [-2.51838883195707221e-02, -8.23487774284355212e-02, 3.33658831723806573e-02], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:652
        [-8.70167529698484543e-09, -1.37080219501928280e-08, 1.80728228771354082e-08], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:653
        [-4.67111571644807100e-09, -2.72041008123058425e-09, 7.06648883852523113e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:654
        [7.26183221906172727e-10, -6.77816339167414128e-09, 4.52883232651690726e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:655
        [5.28852302228433047e-09, 6.47161005340457507e-09, -8.67298467766008940e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:656
        [-2.25465519365641853e-07, -6.46057585221293529e-07, 3.48151143400587948e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:657
        [-1.30051025504229756e-09, -3.25062288891730944e-09, 2.01775679498084060e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:658
        [-5.12724809831333062e-09, 9.33902577666956280e-10, -6.96327353416625883e-10], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:659
        [-3.10810940873373909e-09, -7.49756534634826721e-10, 6.87357185058523612e-10], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:660
        [-1.52109221995821997e-06, -4.22908767925417317e-07, 1.38629667568307413e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:661
        [1.42955317028459206e-09, -7.02968461219199980e-10, -3.81617160094549490e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:662
        [2.53707400921232562e-09, -1.60727622877665510e-09, -4.18765366827500429e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:663
        [-2.14750738948554787e-07, -6.40554276953864132e-07, 3.76128531993924486e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:664
        [3.83073214815787821e-09, 4.50296289838947317e-10, 2.29523194894554194e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:665
        [4.76340728555735282e-07, 6.83235613037347367e-07, -4.72205395646296822e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:666
        [-6.10651996176347607e-04, -1.06790499934057291e-03, 2.29083496655867842e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:667
        [3.95497823379997726e-09, 1.38236928154400474e-09, -6.26218820548585242e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:668
        [1.11904936705986557e-09, -1.37869946362223494e-08, -9.34049783699042457e-10], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:669
        [1.25499246411697740e-09, -2.73635453185150368e-09, -2.91506864740637139e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:670
        [-3.59882924006599270e-07, 1.32511373732895413e-06, -1.55110207063907657e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:671
        [1.07068498511608823e-09, 8.92087770321126072e-09, 2.62826524433101838e-10], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:672
        [-2.69316546841480431e-09, 9.61138280075601870e-10, 5.19946977139973399e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:673
        [-5.92563579700916554e-07, -1.05071339539294234e-06, 1.56249964602256375e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:674
        [1.32198180180509439e-09, 5.16087961255351502e-09, 8.46339526239248130e-10], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:675
        [2.07323220008381881e-06, 1.02309267446332522e-06, -2.07661522726165781e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:676
        [1.31402366846389393e-03, 3.78229792813366064e-04, 1.77496793932758741e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:677
        [8.59301428624004160e-10, -6.83071707530125138e-09, 3.36249680876754553e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:678
        [5.27310424491833629e-09, 2.09999085065692981e-08, -3.10459945807028959e-08], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:679
        [-8.88666080375855039e-08, 4.60897593930476024e-07, 7.41576575386676540e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:680
        [-4.85540663230921155e-10, -5.58243438975036810e-09, 7.40450811775872353e-10], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:681
        [4.03141117225058743e-07, 1.52035531639227450e-06, 9.06206514897367477e-08], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:682
        [5.61075629915620496e-04, -2.05847905628765053e-03, 1.12849817492909434e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:683
        [5.11216541321246609e-09, 7.26292920250060092e-09, -8.97145741030058730e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:684
        [-4.26211688914213127e-07, -7.03366608210270750e-07, 6.27995585866791828e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:685
        [1.15309052943982646e-03, 2.34474318844151959e-03, -4.91856748507475423e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:686
        [1.01104427799588961e-01, -4.22361682938472982e-02, -1.88750007538552200e-01], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:687
        [3.94738332298860684e-10, -7.81372397340440727e-10, 4.06815717224340290e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:688
        [-8.61483928638051566e-09, 5.37427180535843263e-09, 1.81738104426676372e-08], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:689
        [-8.48011268844706123e-10, -5.33803143354383280e-09, 2.99703953494934172e-10], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:690
        [3.89154099408092063e-07, -2.44166311268514957e-07, -8.03240371135063858e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:691
        [-1.20249536439409610e-08, 1.48908931921210019e-08, 1.88292573199966284e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:692
        [-1.16401289163015065e-08, 2.57866422936903206e-08, -5.27022399332555125e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:693
        [1.37065399911928676e-07, 2.16494406102361175e-08, -7.63924557662179482e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:694
        [-6.94754161319199870e-10, 6.65038621394664631e-09, -4.31779645371221932e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:695
        [4.72542155592614588e-07, -7.58546986886782931e-07, -2.35913417925837088e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:696
        [1.46133817312113241e-03, -3.25193103208258009e-04, -3.06625181254991741e-04], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:697
        [9.35794082672593210e-09, -7.92923574022275091e-09, -5.41426242728348939e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:698
        [-2.15279239157428748e-08, -4.16754339024882903e-09, -1.12896482995505920e-08], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:699
        [2.60645369870582400e-10, 1.44616071127263122e-06, -3.63334053799999057e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:700
        [9.17105741349288905e-09, -2.02295233654725681e-08, -1.20002956877085509e-08], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:701
        [-1.27759226226098477e-07, 1.28193771791124470e-06, -5.83097827522305323e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:702
        [2.26880791869919426e-03, -1.34042850080092401e-03, -7.65092051285704835e-04], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:703
        [7.03374036792325796e-09, -2.53508958270032281e-09, -7.66132998708535240e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:704
        [-9.71978722189015265e-07, -5.57836512454779054e-07, 1.96329328074063003e-06], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:705
        [-1.26115140811304343e-03, -4.81792074617704632e-04, -1.06803272537897391e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:706
        [1.19419564863885497e-01, 5.07766738901840875e-02, 4.87642090320925953e-02], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:707
        [1.14090414893297520e-09, 1.56073433760228752e-08, -1.78054684078429726e-08], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:708
        [3.03285130343056153e-09, -1.58615337531031741e-09, -4.94928394101368241e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:709
        [2.64483280249840080e-07, 2.97155396291660413e-07, -5.41608085095034164e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:710
        [2.68757552324139226e-09, -1.41400907649469332e-08, 2.93255796729452456e-08], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:711
        [-2.11094617584561828e-07, -6.56355695552793272e-07, 3.72180321686621518e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:712
        [-2.55073452371079590e-04, 1.57943859317488818e-03, -1.29154484940938240e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:713
        [1.40049266628139435e-09, 1.40747080656922208e-08, -2.58792021839981956e-09], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:714
        [-2.12330362681090179e-07, -1.30522733223815968e-06, 5.84417623253341567e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:715
        [-9.33144849909676392e-04, 1.90305575962152547e-03, -8.35564417983726418e-04], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:716
        [1.81624805201406961e-02, 6.84911174969819458e-02, -2.28291882522520390e-02], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:717
        [-8.25231299961259879e-09, -1.40227519596081152e-08, 1.78809529925716415e-08], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:718
        [1.90689491530449118e-07, 7.01057736002264065e-07, -4.26430629252294580e-07], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:719
        [-5.85146839837499930e-04, -1.07311215649546045e-03, 2.31986890222730339e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:720
        [-1.05962397073886522e-01, 5.51532131360410807e-02, 1.87542648909451215e-01], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:721
        [-1.37499370823599516e-03, -8.49619409242363438e-04, -1.18180356709159952e-03], // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:722
    ];

    #[rustfmt::skip]
    pub(super) static INTERCEPT: [f64; N_CHANNELS] = [
        -1.29208772400146188e+00, // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:726
        6.62251952866635918e+00, // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:727
        -1.35908984683965173e-01, // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:728
    ];
    // END AUTO-GENERATED COEFFICIENTS

    /// `filament_mixer::detail::compute_poly_features` (`FilamentMixerModel.hpp:732-748`): the
    /// 330-entry feature vector for one 7-input point, each entry the product of `x[j]` raised
    /// to `POWERS[i][j]` over every `j` whose power is non-zero.
    pub(super) fn compute_poly_features(x: &[f64; N_INPUTS]) -> [f64; N_FEATURES] {
        // `f64::default()` (`0.0`) rather than a literal: every entry is overwritten below before
        // being read, so this starting value has no vendored-source counterpart to cite (the
        // C++ `double out[330]` parameter is uninitialised stack memory, not zeroed) -- unlike
        // `val`/`p` a few lines down, which really do start from the oracle's own `1.0`.
        let mut out = [f64::default(); N_FEATURES];
        for i in 0..N_FEATURES {
            // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:734
            let mut val = 1.0f64;
            for j in 0..N_INPUTS {
                if POWERS[i][j] != 0 {
                    let base = x[j];
                    let exp = POWERS[i][j];
                    // Fast integer exponentiation (max exp = 4).
                    // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:740
                    let mut p = 1.0f64;
                    for _ in 0..exp {
                        p *= base;
                    }
                    val *= p;
                }
            }
            out[i] = val;
        }
        out
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        /// `POLY_DEGREE`/`N_FEATURES`/`N_INPUTS` and the two table shapes agree with the header
        /// comment's own claim (`FilamentMixerModel.hpp:53`, "Degree-4 polynomial, 330 features,
        /// 7 inputs") -- a plain shape check, not this unit's `// PROOF:` marker (see
        /// `filament_mixer_model_lerp_matches_the_oracle`, in the parent module, for that).
        #[test]
        fn filament_mixer_model_detail_table_shapes_match_the_header_comment() {
            assert_eq!(POLY_DEGREE, 4);
            assert_eq!(N_FEATURES, 330);
            assert_eq!(N_INPUTS, 7);
            assert_eq!(N_CHANNELS, 3);
            assert_eq!(POWERS.len(), N_FEATURES);
            assert_eq!(COEF.len(), N_FEATURES);
            assert_eq!(INTERCEPT.len(), N_CHANNELS);
            // Every POWERS row's max entry is <= POLY_DEGREE (FilamentMixerModel.hpp:60-389's
            // own invariant: a degree-4 polynomial's exponents never exceed 4).
            for row in POWERS.iter() {
                assert!(row.iter().all(|&p| (0..=4).contains(&p)));
            }
        }
    }
}

/// `filament_mixer::RGB` (`FilamentMixerModel.hpp:752-754`): the output type the convenience
/// `lerp` overload returns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// Mix two RGB colors using polynomial pigment mixing (`filament_mixer::lerp`,
/// `FilamentMixerModel.hpp:766-815`, both overloads folded into one -- see this module's doc
/// comment).
///
/// `t` is a mixing ratio: `0.0` is all of color 1, `1.0` is all of color 2 (`:763`).
pub fn lerp(r1: u8, g1: u8, b1: u8, r2: u8, g2: u8, b2: u8, t: f32) -> Rgb {
    // Clamp t -- FilamentMixerModel.hpp:770-778.
    // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:771
    if t <= 0.0f32 {
        return Rgb {
            r: r1,
            g: g1,
            b: b1,
        };
    }
    // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:775
    if t >= 1.0f32 {
        return Rgb {
            r: r2,
            g: g2,
            b: b2,
        };
    }

    // FilamentMixerModel.hpp:780-784: every input widened to f64 -- `t` (an f32 parameter) is
    // widened here only because the vendored source's own `static_cast<double>(t)` (:783) does;
    // see this module's doc comment.
    let x: [f64; detail::N_INPUTS] = [
        r1 as f64, g1 as f64, b1 as f64, r2 as f64, g2 as f64, b2 as f64, t as f64,
    ];

    let features = detail::compute_poly_features(&x);

    let mut channel = [0u8; detail::N_CHANNELS];
    for (c, out_channel) in channel.iter_mut().enumerate() {
        // FIDELITY(port): fixed summation order (by feature index, COEF's own row order,
        // FilamentMixerModel.hpp:792-794) -- floating-point addition is not associative, so this
        // order is load-bearing for exact equality against the oracle, which sums in exactly
        // this order too (docs/PORTING.md's "sum in a fixed order" rule).
        let mut sum = detail::INTERCEPT[c];
        for i in 0..detail::N_FEATURES {
            sum += features[i] * detail::COEF[i][c];
        }
        // Clamp to [0, 255] and truncate (FilamentMixerModel.hpp:795-798; matches numpy
        // astype(int) behavior, the vendored comment's own words).
        let mut val = sum as i32;
        if val < 0 {
            val = 0;
        }
        // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:798
        if val > 255 {
            // PORT SRC: src/libslic3r/FilamentMixerModel.hpp:798
            val = 255;
        }
        *out_channel = val as u8;
    }

    Rgb {
        r: channel[0],
        g: channel[1],
        b: channel[2],
    }
}

// **Proof path 1, reached via `slic3r-proof`'s re-export of `slic3r-ffi`'s per-unit proof
// bridges (`docs/PORTING.md`, "Decisions recorded on 2026-11-25", decision 1).** This task's
// shim is three new files under `crates/slic3r-ffi`
// (`cxx/proof_units/filament_mixer_model.{h,cpp}`, `src/proof_units/filament_mixer_model.rs`),
// not an append to `proof_shims.cpp`/`src/proof.rs` as an earlier draft of this task's text
// named -- those two files no longer exist on `main`; decision 5 states that
// `docs/PORTING.md`'s decisions govern over a task's file list, and the pull request body says
// so. `slic3r-filamentmixermodel`'s existing dev-dependency on `slic3r-proof`
// (`34-foundation-spike`'s crate skeleton) is the one dependency edge decision 1 requires, so no
// `Cargo.toml` changes.

#[cfg(test)]
mod tests {
    use super::*;

    /// A plain correctness check of the documented usage example (`FilamentMixerModel.hpp:11-13`)
    /// and of the two clamp branches (`:771-778`) at and past their exact boundary. Not a
    /// `// PROOF:` marker -- see `filament_mixer_model_lerp_matches_the_oracle` below for that.
    #[test]
    fn filament_mixer_model_lerp_matches_the_documented_example_and_clamps_t() {
        assert_eq!(
            lerp(0, 33, 133, 252, 211, 0, 0.5),
            Rgb {
                r: 47,
                g: 141,
                b: 56
            }
        );
        assert_eq!(
            lerp(10, 20, 30, 200, 210, 220, 0.0),
            Rgb {
                r: 10,
                g: 20,
                b: 30
            }
        );
        assert_eq!(
            lerp(10, 20, 30, 200, 210, 220, 1.0),
            Rgb {
                r: 200,
                g: 210,
                b: 220
            }
        );
        assert_eq!(
            lerp(10, 20, 30, 200, 210, 220, -1.0),
            Rgb {
                r: 10,
                g: 20,
                b: 30
            }
        );
        assert_eq!(
            lerp(10, 20, 30, 200, 210, 220, 2.0),
            Rgb {
                r: 200,
                g: 210,
                b: 220
            }
        );
    }

    /// `Slic3r::filament_mixer_lerp` (`FilamentMixer.cpp:58-64`) against this module's [`lerp`],
    /// over a spread of inputs including both clamp boundaries and the interior arithmetic path.
    /// Each side is driven independently -- the oracle through `slic3r-proof`'s re-export of
    /// `slic3r-ffi`'s per-unit proof bridge (`docs/PORTING.md`, "Decisions recorded on
    /// 2026-11-25", decision 1), the port through this module's own [`lerp`] -- and compared as
    /// `Vec<u32>` ([`u8`] has no `slic3r_proof::tiers::ProofValue` impl; widening to `u32` is a
    /// lossless, order-preserving wrapper, not a tolerance).
    // PROOF: 1
    #[test]
    fn filament_mixer_model_lerp_matches_the_oracle() {
        use slic3r_proof::proof_units::filament_mixer_model::path1_filament_mixer_lerp;
        use slic3r_proof::{compare_unit, Oracle};

        let cases: &[(u8, u8, u8, u8, u8, u8, f32)] = &[
            (0, 33, 133, 252, 211, 0, 0.5),
            (10, 20, 30, 200, 210, 220, 0.0),
            (10, 20, 30, 200, 210, 220, 1.0),
            (10, 20, 30, 200, 210, 220, -1.0),
            (10, 20, 30, 200, 210, 220, 2.0),
            (255, 0, 0, 0, 255, 0, 0.25),
            (0, 0, 0, 255, 255, 255, 0.75),
        ];
        for &(r1, g1, b1, r2, g2, b2, t) in cases {
            let oracle: Vec<u32> = path1_filament_mixer_lerp(r1, g1, b1, r2, g2, b2, t)
                .into_iter()
                .map(u32::from)
                .collect();
            let ported = lerp(r1, g1, b1, r2, g2, b2, t);
            let ported_vec: Vec<u32> = vec![
                u32::from(ported.r),
                u32::from(ported.g),
                u32::from(ported.b),
            ];
            compare_unit("FilamentMixerModel.hpp", Oracle::Built, ported_vec, oracle)
                .expect("filament_mixer_lerp: [r, g, b] against the oracle");
        }
    }
}

// PORT STATUS
// source: src/libslic3r/FilamentMixerModel.hpp
// confidence: high
// tier: exact
// oracle: built
