//! Proof shim for the `Utils.hpp`/`utils.cpp` unit (task 57-foundation-utils, R19). Shaped like
//! proof path 1 (calls an externally-linked symbol directly, no `#include` of a vendored
//! `.cpp`), not proof path 3 as the manifest row's `proof path` cell names -- see
//! `cxx/proof_units/utils.cpp`'s header comment for the measurement that rules path 3 out, and
//! this task's pull request body for the same note.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::utils")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/utils.h");

        /// `Slic3r::get_utf8_sequence_length(const char*, size_t)` (`utils.cpp:1390-1452`):
        /// bytes in the UTF-8 sequence starting at `seq[0]`. `seq` must be non-empty.
        fn path1_get_utf8_sequence_length(seq: Vec<u8>) -> u64;

        /// `Slic3r::short_time(const std::string&)` (`Utils.hpp:578-620`). Must not be called
        /// with a `time` the vendored body leaves `seconds < 0` for: that branch returns an
        /// unwritten `char buffer[64]`.
        fn path1_short_time(time: String) -> String;
        /// `Slic3r::get_time_dhms(float)` (`Utils.hpp:624-646`).
        fn path1_get_time_dhms(time_in_secs: f32) -> String;
        /// `Slic3r::get_bbl_time_dhms(float)` (`Utils.hpp:648-668`).
        fn path1_get_bbl_time_dhms(time_in_secs: f32) -> String;
        /// `Slic3r::get_timezone_utc_hm(long)` (`Utils.hpp:670-686`).
        fn path1_get_timezone_utc_hm(second: i64) -> String;
        /// `Slic3r::get_time_dhm(float)` (`Utils.hpp:688-707`).
        fn path1_get_time_dhm(time_in_secs: f32) -> String;
        /// `Slic3r::get_time_hms(float)` (`Utils.hpp:709-720`).
        fn path1_get_time_hms(time_in_secs: f32) -> String;
        /// `Slic3r::get_bbl_monitor_time_dhm(float)` (`Utils.hpp:722-742`).
        fn path1_get_bbl_monitor_time_dhm(time_in_secs: f32) -> String;
        /// `Slic3r::get_bbl_remain_time_dhms(float)` (`Utils.hpp:759-779`).
        fn path1_get_bbl_remain_time_dhms(time_in_secs: f32) -> String;

        /// `Slic3r::is_path_within_root(const std::string&, const path&)` (`utils.cpp:1292`).
        fn path1_is_path_within_root(rel_path: String, root: String) -> bool;
        /// `Slic3r::is_symlink_target_within_root(...)` (`utils.cpp:1322`).
        fn path1_is_symlink_target_within_root(
            link_rel_path: String,
            target: String,
            root: String,
        ) -> bool;
        /// `Slic3r::is_absolute_path_within_root(const path&, const path&)` (`utils.cpp:1331`).
        fn path1_is_absolute_path_within_root(path: String, root: String) -> bool;

        /// `Slic3r::is_running_in_flatpak()` (`utils.cpp:403`).
        fn path1_is_running_in_flatpak() -> bool;
        /// `Slic3r::makedir(const std::string)` (`utils.cpp:1961`).
        fn path1_makedir(path: String) -> bool;
        /// `Slic3r::rename_file(const std::string&, const std::string&)` (`utils.cpp:818`),
        /// as the returned `std::error_code`'s `value()` (0 on success).
        fn path1_rename_file(from: String, to: String) -> i32;
        /// `Slic3r::write_file_atomically(const std::string&, const std::string&, bool)`
        /// (`Utils.hpp:271`, forwarding to `utils.cpp:869`), as the `std::error_code`'s
        /// `value()`.
        fn path1_write_file_atomically(path: String, content: String, binary: bool) -> i32;
        /// `Slic3r::check_copy(const std::string&, const std::string&)` (`utils.cpp:1220`), as
        /// the `CopyFileResult` enumerator's integer value.
        fn path1_check_copy(origin: String, copy: String) -> i32;
        /// `Slic3r::copy_file_inner(...)` (`utils.cpp:1107`), as the `CopyFileResult`'s value.
        fn path1_copy_file_inner(from: String, to: String) -> i32;
        /// `Slic3r::copy_file(..., const bool with_check)` (`utils.cpp:1144`), as the
        /// `CopyFileResult`'s value.
        fn path1_copy_file(from: String, to: String, with_check: bool) -> i32;
        /// `Slic3r::copy_framework(const std::string&, const std::string&)` (`utils.cpp:1192`).
        fn path1_copy_framework(from: String, to: String) -> bool;
        /// `Slic3r::copy_directory_recursively(...)` (`utils.cpp:1997`) with an empty `filter`;
        /// `false` where the vendored body throws.
        fn path1_copy_directory_recursively(
            source: String,
            target: String,
            merge_mode: bool,
        ) -> bool;
        /// `Slic3r::save_string_file(const path&, const std::string&)` (`utils.cpp:2229`);
        /// `false` where its stream raises.
        fn path1_save_string_file(p: String, str: Vec<u8>) -> bool;
        /// `Slic3r::load_string_file(const path&, std::string&)` (`utils.cpp:2237`); `false`
        /// where its stream raises, in which case `out` is left as it was.
        fn path1_load_string_file(p: String, out: &mut Vec<u8>) -> bool;
    }
}

pub use ffi::{
    path1_check_copy, path1_copy_directory_recursively, path1_copy_file, path1_copy_file_inner,
    path1_copy_framework, path1_get_bbl_monitor_time_dhm, path1_get_bbl_remain_time_dhms,
    path1_get_bbl_time_dhms, path1_get_time_dhm, path1_get_time_dhms, path1_get_time_hms,
    path1_get_timezone_utc_hm, path1_get_utf8_sequence_length, path1_is_absolute_path_within_root,
    path1_is_path_within_root, path1_is_running_in_flatpak, path1_is_symlink_target_within_root,
    path1_load_string_file, path1_makedir, path1_rename_file, path1_save_string_file,
    path1_short_time, path1_write_file_atomically,
};
