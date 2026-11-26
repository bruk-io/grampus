#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart at <crate-name>/src/proof_units/utils.rs.h on the include path it
// sets up, and `cxx/proof_units/utils.cpp` is the one translation unit that defines the
// function below.
#include "slic3r-ffi/src/proof_units/utils.rs.h"

#include <cstdint>

namespace slic3r_ffi {
namespace proof_units {
namespace utils {

std::uint64_t path1_get_utf8_sequence_length(rust::Vec<std::uint8_t> seq);

// The `Utils.hpp` time formatters. They are `inline` in the vendored header, so these wrappers
// compile the pinned header's own bodies into this translation unit -- the same route
// `cxx/proof_units/object_id.cpp`'s header-inline shims take, with no `#include` of any `.cpp`.
rust::String path1_short_time(rust::String time);
rust::String path1_get_time_dhms(float time_in_secs);
rust::String path1_get_bbl_time_dhms(float time_in_secs);
rust::String path1_get_timezone_utc_hm(std::int64_t second);
rust::String path1_get_time_dhm(float time_in_secs);
rust::String path1_get_time_hms(float time_in_secs);
rust::String path1_get_bbl_monitor_time_dhm(float time_in_secs);
rust::String path1_get_bbl_remain_time_dhms(float time_in_secs);

// The three canonicalization-based path-root checks (`utils.cpp:1292`, `:1322`, `:1331`), all
// externally linked in `utils.cpp.o`.
bool path1_is_path_within_root(rust::String rel_path, rust::String root);
bool path1_is_symlink_target_within_root(rust::String link_rel_path, rust::String target, rust::String root);
bool path1_is_absolute_path_within_root(rust::String path, rust::String root);

// The file-mutation family. Each returns the vendored result as a plain integer or bool; the
// `std::error_code` returners give `value()` (0 on success), and the `CopyFileResult` returners
// give the enumerator's own integer value. The two that throw on failure in the original
// (`save_string_file`/`load_string_file`, which set `failbit|badbit` on their stream) report it
// as `false`/an empty optional instead, since a C++ exception may not cross the `cxx` bridge.
// `is_running_in_flatpak` (`utils.cpp:403`), externally linked and without arguments.
bool path1_is_running_in_flatpak();

bool path1_makedir(rust::String path);
std::int32_t path1_rename_file(rust::String from, rust::String to);
std::int32_t path1_write_file_atomically(rust::String path, rust::String content, bool binary);
std::int32_t path1_check_copy(rust::String origin, rust::String copy);
std::int32_t path1_copy_file_inner(rust::String from, rust::String to);
std::int32_t path1_copy_file(rust::String from, rust::String to, bool with_check);
bool path1_copy_framework(rust::String from, rust::String to);
bool path1_copy_directory_recursively(rust::String source, rust::String target, bool merge_mode);
bool path1_save_string_file(rust::String p, rust::Vec<std::uint8_t> str);
bool path1_load_string_file(rust::String p, rust::Vec<std::uint8_t> &out);

} // namespace utils
} // namespace proof_units
} // namespace slic3r_ffi
