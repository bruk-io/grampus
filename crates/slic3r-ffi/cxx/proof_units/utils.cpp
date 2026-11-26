// Proof shim for the `Utils.hpp`/`utils.cpp` unit (task 57-foundation-utils, R19), compiled onto
// `crates/slic3r-ffi`'s shared `cxx_build::Build` by this crate's `build.rs` -- the one
// sanctioned route a ported unit reaches the oracle by (`docs/PORTING.md`, "Decisions recorded
// on 2026-11-25"). `crates/slic3r-utils`'s proof calls this through
// `slic3r_proof::proof_units::utils`.
//
// **Route note, recorded here and in this task's pull request body (`docs/WORKFLOW.md`,
// "Decisions that govern over a task's file list", corollary 1, plus a second divergence this
// comment also covers): the manifest row's `proof path` cell is `3`, but this shim is shaped
// like proof path 1, not proof path 3.** Path 3 means `#include`ing the vendored `.cpp` whole
// to reach an internal-linkage helper. `utils.cpp` was measured before this shim was written:
//
//   nm -m vendor/OrcaSlicer/build/arm64/src/libslic3r/CMakeFiles/libslic3r.dir/Release/utils.cpp.o \
//     | grep -E '^\S+ \(.*\) external ' | grep -v weak | c++filt | sort -u | wc -l
//   -> 99
//
// 99 strong, defined, externally-linked `Slic3r::` symbols -- not the 1-4 symbol range
// `cxx/proof_units/clipper_z_utils.cpp`'s own sweep found viable for the same `#include` trick
// (it measured 94 proof-path-3 `.cpp` files and the chosen one, at 2 symbols, was already tied
// for the practical minimum). `crates/slic3r-ffi/build.rs`'s own doc comment states that the
// derived link line reproduces the oracle's `libslic3r_tests`/`fff_print_tests` link, i.e. the
// whole `liblibslic3r.a` archive -- so every one of these 99 symbols (`set_logging_level`,
// `data_dir`, `is_plain_file`, `split_string`, `xml_escape`, ...) is already provided by that
// archive on every build of this crate. `#include`ing `utils.cpp` here a second time would
// define all 99 again in this translation unit, and `ld` would extract both archives' copies
// into the same link the moment anything needed even one of them -- a duplicate-symbol
// collision across effectively the whole file, not the few-symbol risk `ClipperZUtils.cpp`
// accepted. This is far past the "may not be `#include`d here this way" line that file's header
// comment states, so path 3 is not attempted for this unit.
//
// R19's stated fallback applies instead: "the helper is proven through its enclosing file's
// exported surface." `Slic3r::get_utf8_sequence_length(const char*, size_t)` (`utils.cpp:1390-
// 1452`) is one of those 99 externally-linked symbols, is pure (no global state, no I/O), and
// takes/returns only plain integer data -- exactly the shape decision 1 sanctions for a shim
// payload. This shim declares the vendored header and calls that symbol directly, the same
// shape `cxx/proof_units/object_id.cpp`'s path-1 shims use, with no `#include` of any `.cpp`.
//
// The marker on the unit's `// PROOF: 3` test still reads `3`, because `port/port-status`'s
// `proof_present` matches the manifest row's own cell verbatim (`docs/WORKFLOW.md`, "How a
// proof is recognised"); the route actually taken is path 1 in shape, recorded here and in the
// pull request body rather than silently reconciled with the cell.
#include "utils.h"

#include "Utils.hpp"

#include <boost/filesystem/path.hpp>
#include <functional>
#include <string>

namespace slic3r_ffi {
namespace proof_units {
namespace utils {

// `seq` must be non-empty: the vendored function itself reads `seq[0]` with no length check
// (`utils.cpp:1393`), so an empty `seq` would be undefined behaviour on the oracle side too.
// The Rust-side proof test never calls this with an empty vector.
std::uint64_t path1_get_utf8_sequence_length(rust::Vec<std::uint8_t> seq) {
    return static_cast<std::uint64_t>(
        Slic3r::get_utf8_sequence_length(reinterpret_cast<const char *>(seq.data()), seq.size()));
}

// --- The Utils.hpp time formatters -----------------------------------------------------------
//
// Each of these is `inline` in the pinned header, so the body this calls is the vendored body
// compiled into this translation unit from `Utils.hpp` itself -- nothing is retyped here.
//
// `short_time` has one input range in which the vendored body returns a `char buffer[64]` it
// never wrote (`Utils.hpp:620`, reached when `seconds < 0` and `days`/`hours`/`minutes` are all
// zero) -- undefined behaviour, reading uninitialised stack. The Rust-side proof never calls
// that range; see the ported module's doc comment for the disclosure.

rust::String path1_short_time(rust::String time) {
    return rust::String(Slic3r::short_time(std::string(time)));
}

rust::String path1_get_time_dhms(float time_in_secs) {
    return rust::String(Slic3r::get_time_dhms(time_in_secs));
}

rust::String path1_get_bbl_time_dhms(float time_in_secs) {
    return rust::String(Slic3r::get_bbl_time_dhms(time_in_secs));
}

rust::String path1_get_timezone_utc_hm(std::int64_t second) {
    return rust::String(Slic3r::get_timezone_utc_hm(static_cast<long>(second)));
}

rust::String path1_get_time_dhm(float time_in_secs) {
    return rust::String(Slic3r::get_time_dhm(time_in_secs));
}

rust::String path1_get_time_hms(float time_in_secs) {
    return rust::String(Slic3r::get_time_hms(time_in_secs));
}

rust::String path1_get_bbl_monitor_time_dhm(float time_in_secs) {
    return rust::String(Slic3r::get_bbl_monitor_time_dhm(time_in_secs));
}

rust::String path1_get_bbl_remain_time_dhms(float time_in_secs) {
    return rust::String(Slic3r::get_bbl_remain_time_dhms(time_in_secs));
}

// --- The three canonicalization-based path-root checks ---------------------------------------

bool path1_is_path_within_root(rust::String rel_path, rust::String root) {
    return Slic3r::is_path_within_root(std::string(rel_path),
                                       boost::filesystem::path(std::string(root)));
}

bool path1_is_symlink_target_within_root(rust::String link_rel_path, rust::String target,
                                         rust::String root) {
    return Slic3r::is_symlink_target_within_root(std::string(link_rel_path), std::string(target),
                                                 boost::filesystem::path(std::string(root)));
}

bool path1_is_absolute_path_within_root(rust::String path, rust::String root) {
    return Slic3r::is_absolute_path_within_root(boost::filesystem::path(std::string(path)),
                                                boost::filesystem::path(std::string(root)));
}

// --- The file-mutation family ----------------------------------------------------------------

bool path1_is_running_in_flatpak() { return Slic3r::is_running_in_flatpak(); }

bool path1_makedir(rust::String path) { return Slic3r::makedir(std::string(path)); }

std::int32_t path1_rename_file(rust::String from, rust::String to) {
    return static_cast<std::int32_t>(Slic3r::rename_file(std::string(from), std::string(to)).value());
}

std::int32_t path1_write_file_atomically(rust::String path, rust::String content, bool binary) {
    // The single-string overload (`Utils.hpp:271`), which forwards one `string_view` chunk to
    // the `initializer_list` overload (`utils.cpp:869`).
    const std::string content_str(content);
    return static_cast<std::int32_t>(
        Slic3r::write_file_atomically(std::string(path), content_str, binary).value());
}

std::int32_t path1_check_copy(rust::String origin, rust::String copy) {
    return static_cast<std::int32_t>(Slic3r::check_copy(std::string(origin), std::string(copy)));
}

std::int32_t path1_copy_file_inner(rust::String from, rust::String to) {
    std::string error_message;
    return static_cast<std::int32_t>(
        Slic3r::copy_file_inner(std::string(from), std::string(to), error_message));
}

std::int32_t path1_copy_file(rust::String from, rust::String to, bool with_check) {
    std::string error_message;
    return static_cast<std::int32_t>(
        Slic3r::copy_file(std::string(from), std::string(to), error_message, with_check));
}

bool path1_copy_framework(rust::String from, rust::String to) {
    return Slic3r::copy_framework(std::string(from), std::string(to));
}

bool path1_copy_directory_recursively(rust::String source, rust::String target, bool merge_mode) {
    // The vendored function throws `Slic3r::CriticalException` on a failed per-file copy
    // (`utils.cpp:2022`) and lets `boost::filesystem`'s own throwing calls out; a C++ exception
    // may not cross the bridge, so both are reported as `false`.
    try {
        Slic3r::copy_directory_recursively(boost::filesystem::path(std::string(source)),
                                           boost::filesystem::path(std::string(target)),
                                           std::function<bool(const std::string)>(), merge_mode);
        return true;
    } catch (...) {
        return false;
    }
}

bool path1_save_string_file(rust::String p, rust::Vec<std::uint8_t> str) {
    try {
        Slic3r::save_string_file(boost::filesystem::path(std::string(p)),
                                 std::string(reinterpret_cast<const char *>(str.data()),
                                             str.size()));
        return true;
    } catch (...) {
        return false;
    }
}

bool path1_load_string_file(rust::String p, rust::Vec<std::uint8_t> &out) {
    try {
        std::string str;
        Slic3r::load_string_file(boost::filesystem::path(std::string(p)), str);
        out.clear();
        out.reserve(str.size());
        for (const char byte : str)
            out.push_back(static_cast<std::uint8_t>(byte));
        return true;
    } catch (...) {
        return false;
    }
}

} // namespace utils
} // namespace proof_units
} // namespace slic3r_ffi
