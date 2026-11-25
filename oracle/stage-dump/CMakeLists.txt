# oracle/stage-dump/CMakeLists.txt
#
# Builds the per-stage dumper (R5, plan step 8) against the libslic3r task
# 13-build-libslic3r already built at vendor/OrcaSlicer/build/arm64, and the dependency
# superbuild task 02-deps-superbuild already installed at
# vendor/OrcaSlicer/deps/build/arm64/OrcaSlicer_dep/usr/local. This is a SEPARATE CMake
# project from vendor/OrcaSlicer's own (configured and built into the gitignored
# oracle/.cache/stage-dump/arm64, never into vendor/OrcaSlicer/): it links the two
# already-built static libraries, liblibslic3r.a and liblibslic3r_cgal.a, plus every
# library transitively needed to resolve them, by the exact link line
# fff_print_tests -- the one upstream target that already links libslic3r with the GUI
# off on this platform -- used in that same build tree (read from its own build.ninja,
# never re-derived from a find_package call this standalone project has no CMake cache
# to run): compiler DEFINES and FLAGS, and the include and link-library lists, copied
# from that target's own compile and link edges so this binary is ABI-compatible with
# the prebuilt .a files (same preprocessor macros deciding struct layout, same C++
# standard and target triple). The one normative addition is -ffp-contract=off (C2's
# decision), which is already first in the copied FLAGS because fff_print_tests carries
# it too.

cmake_minimum_required(VERSION 3.20)
project(oracle_stage_dump CXX)

if(NOT CMAKE_BUILD_TYPE)
  set(CMAKE_BUILD_TYPE Release)
endif()
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_OSX_ARCHITECTURES arm64)
set(CMAKE_OSX_DEPLOYMENT_TARGET 12.0)

# Paths into the vendored tree, relative to this file so the project resolves
# correctly from any worktree -- never hardcoded to one checkout's absolute path.
get_filename_component(ORCA_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../../vendor/OrcaSlicer" ABSOLUTE)
set(ORCA_SRC "${ORCA_ROOT}/src")
set(ORCA_DEPS_SRC "${ORCA_ROOT}/deps_src")
set(ORCA_BUILD "${ORCA_ROOT}/build/arm64")
set(ORCA_DEPS_PREFIX "${ORCA_ROOT}/deps/build/arm64/OrcaSlicer_dep/usr/local")

# Task 13-build-libslic3r is a precondition and not something to work around (C7's
# decision): fail loudly and early, naming the missing build, rather than producing a
# confusing linker error deep in the configure or build step.
if(NOT EXISTS "${ORCA_BUILD}/src/libslic3r/Release/liblibslic3r.a")
  message(FATAL_ERROR
    "vendor/OrcaSlicer/build/arm64 has no built liblibslic3r.a -- run 'just oracle-build' "
    "first (task 13-build-libslic3r is a precondition of this task, not something this "
    "CMakeLists.txt works around).")
endif()
if(NOT EXISTS "${ORCA_BUILD}/src/libslic3r/libslic3r_version.h")
  message(FATAL_ERROR
    "vendor/OrcaSlicer/build/arm64/src/libslic3r/libslic3r_version.h is missing -- "
    "run 'just oracle-build' first.")
endif()

add_executable(stage_dump dump_main.cpp dump_seams.cpp nanosvg_impl.cpp)

# DEFINES and FLAGS below are copied verbatim from the fff_print_tests target's own
# Print.cpp compile edge in vendor/OrcaSlicer/build/arm64's generated build.ninja, minus
# the -Werror machinery (this is new code, not yet proven against upstream's warning
# set) and the precompiled-header flags (this project has no PCH of its own). Every
# define here decides a struct layout or a template instantiation in a header this
# program includes, so dropping one silently would be an ABI mismatch against the
# prebuilt .a files, not a compile error.
target_compile_definitions(stage_dump PRIVATE
  BBL_RELEASE_TO_PUBLIC=1
  BOOST_ASIO_DISABLE_KQUEUE
  BOOST_ATOMIC_NO_LIB
  BOOST_ATOMIC_STATIC_LINK
  BOOST_CHRONO_NO_LIB
  BOOST_CHRONO_STATIC_LINK
  BOOST_CONTAINER_NO_LIB
  BOOST_CONTAINER_STATIC_LINK
  BOOST_DATE_TIME_NO_LIB
  BOOST_DATE_TIME_STATIC_LINK
  BOOST_FILESYSTEM_NO_LIB
  BOOST_FILESYSTEM_STATIC_LINK=1
  BOOST_IOSTREAMS_NO_LIB
  BOOST_IOSTREAMS_STATIC_LINK
  BOOST_LOCALE_NO_LIB
  BOOST_LOCALE_STATIC_LINK
  BOOST_LOG_NO_LIB
  BOOST_LOG_STATIC_LINK
  BOOST_NOWIDE_NO_LIB
  BOOST_NO_CXX98_FUNCTION_BASE
  BOOST_PROGRAM_OPTIONS_NO_LIB
  BOOST_PROGRAM_OPTIONS_STATIC_LINK
  BOOST_RANDOM_NO_LIB
  BOOST_RANDOM_STATIC_LINK
  BOOST_SERIALIZATION_NO_LIB
  BOOST_SERIALIZATION_STATIC_LINK
  BOOST_THREAD_NO_LIB
  BOOST_THREAD_STATIC_LINK
  BOOST_THREAD_USE_LIB
  CGAL_LINKED_WITH_TBB
  LIBNEST2D_GEOMETRIES_libslic3r
  LIBNEST2D_OPTIMIZER_nlopt
  LIBNEST2D_STATIC
  LIBNEST2D_THREADING_tbb
  NDEBUG
  OCC_CONVERT_SIGNALS
  OPENVDB_OPENEXR_STATICLIB
  OPENVDB_STATICLIB
  SLIC3R_CAD
  SLIC3R_DESKTOP_INTEGRATION
  TBB_USE_CAPTURED_EXCEPTION=0
  UNICODE
  USE_TBB
  WXINSPECTOR_DISABLE
  WXINTL_NO_GETTEXT_MACRO
  _HAS_AUTO_PTR_ETC=0
  _UNICODE
  wxNO_UNSAFE_WXSTRING_CONV
  wxUSE_UNICODE
)

target_compile_options(stage_dump PRIVATE
  -ffp-contract=off
  -Wall
  -O3
  -DNDEBUG
  -std=gnu++17
  -arch
  arm64
  -mmacosx-version-min=12.0
  -fPIC
  -fsigned-char
  -Wno-reorder
  -Wno-sign-compare
  -Wno-misleading-indentation
  -Wno-switch
  -Wno-unused-function
  -Wno-unused-variable
  -Wno-unused-but-set-variable
  -Wno-unused-label
  -Wno-unused-local-typedefs
  -Wno-deprecated-declarations
  -Wno-ignored-attributes
  -DOPENVDB_ABI_VERSION_NUMBER=8
)

target_include_directories(stage_dump PRIVATE
  ${ORCA_SRC}/libslic3r
  ${ORCA_SRC}/libslic3r/TextureToColor
  ${ORCA_BUILD}/src/libslic3r
  ${ORCA_DEPS_SRC}/clipper2/Clipper2Lib/include
  ${ORCA_DEPS_SRC}/nanosvg
)
target_include_directories(stage_dump SYSTEM PRIVATE
  ${ORCA_SRC}
  ${ORCA_BUILD}/src/dev-utils/platform
  ${ORCA_SRC}/libigl
  ${ORCA_DEPS_PREFIX}/include
  ${ORCA_DEPS_PREFIX}/include/occt
  ${ORCA_DEPS_PREFIX}/include/eigen3
  ${ORCA_DEPS_SRC}/admesh
  ${ORCA_DEPS_SRC}/admesh/..
  ${ORCA_DEPS_SRC}/libigl
  ${ORCA_DEPS_SRC}/libnest2d/include
  ${ORCA_DEPS_SRC}/miniz
  ${ORCA_DEPS_PREFIX}/include/opencv4
  ${ORCA_DEPS_SRC}/glu-libtess/include
  ${ORCA_DEPS_SRC}/semver
  ${ORCA_DEPS_PREFIX}/include/OpenEXR
)

# The link line: liblibslic3r.a and liblibslic3r_cgal.a (already built by
# 13-build-libslic3r) plus every library they transitively need, in the order
# fff_print_tests links them (static-archive order matters to the linker). Catch2 and
# Catch2Main are dropped -- this program is not a test binary and declares no
# dependency on either.
target_link_libraries(stage_dump PRIVATE
  ${ORCA_BUILD}/src/libslic3r/Release/liblibslic3r.a
  -liconv
  "-framework IOKit"
  "-framework CoreFoundation"
  -lc++
  ${ORCA_BUILD}/deps_src/libnest2d/Release/liblibnest2d.a
  ${ORCA_BUILD}/src/libslic3r/Release/liblibslic3r.a
  ${ORCA_BUILD}/deps_src/libnest2d/Release/liblibnest2d.a
  ${ORCA_BUILD}/deps_src/miniz/Release/libminiz_static.a
  ${ORCA_DEPS_PREFIX}/lib/libopencv_world.a
  ${ORCA_DEPS_PREFIX}/lib/opencv4/3rdparty/libtegra_hal.a
  ${ORCA_DEPS_PREFIX}/lib/libjpeg.a
  ${ORCA_DEPS_PREFIX}/lib/libpng.a
  "-framework AppKit"
  ${ORCA_DEPS_PREFIX}/lib/libassimp.a
  "/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk/usr/lib/libz.tbd"
  ${ORCA_DEPS_PREFIX}/lib/libexpat.a
  ${ORCA_DEPS_PREFIX}/lib/occt/libTKOffset.a
  ${ORCA_DEPS_PREFIX}/lib/occt/libTKFillet.a
  ${ORCA_DEPS_PREFIX}/lib/occt/libTKBool.a
  ${ORCA_DEPS_PREFIX}/lib/occt/libTKDESTEP.a
  ${ORCA_DEPS_PREFIX}/lib/occt/libTKDE.a
  ${ORCA_DEPS_PREFIX}/lib/occt/libTKXCAF.a
  ${ORCA_DEPS_PREFIX}/lib/occt/libTKXSBase.a
  ${ORCA_DEPS_PREFIX}/lib/occt/libTKVCAF.a
  ${ORCA_DEPS_PREFIX}/lib/occt/libTKCAF.a
  ${ORCA_DEPS_PREFIX}/lib/occt/libTKLCAF.a
  ${ORCA_DEPS_PREFIX}/lib/occt/libTKCDF.a
  ${ORCA_DEPS_PREFIX}/lib/occt/libTKV3d.a
  ${ORCA_DEPS_PREFIX}/lib/occt/libTKService.a
  -lobjc
  "-framework AppKit"
  "-framework IOKit"
  ${ORCA_DEPS_PREFIX}/lib/occt/libTKMesh.a
  ${ORCA_DEPS_PREFIX}/lib/occt/libTKBO.a
  ${ORCA_DEPS_PREFIX}/lib/occt/libTKPrim.a
  ${ORCA_DEPS_PREFIX}/lib/occt/libTKHLR.a
  ${ORCA_DEPS_PREFIX}/lib/occt/libTKShHealing.a
  ${ORCA_DEPS_PREFIX}/lib/occt/libTKTopAlgo.a
  ${ORCA_DEPS_PREFIX}/lib/occt/libTKGeomAlgo.a
  ${ORCA_DEPS_PREFIX}/lib/occt/libTKBRep.a
  ${ORCA_DEPS_PREFIX}/lib/occt/libTKGeomBase.a
  ${ORCA_DEPS_PREFIX}/lib/occt/libTKG3d.a
  ${ORCA_DEPS_PREFIX}/lib/occt/libTKG2d.a
  ${ORCA_DEPS_PREFIX}/lib/occt/libTKMath.a
  ${ORCA_DEPS_PREFIX}/lib/occt/libTKernel.a
  ${ORCA_BUILD}/deps_src/clipper2/Release/libClipper2.a
  -lm
  ${ORCA_DEPS_PREFIX}/lib/libdraco.a
  ${ORCA_BUILD}/deps_src/glu-libtess/Release/libglu-libtess.a
  ${ORCA_DEPS_PREFIX}/lib/libjpeg.a
  ${ORCA_BUILD}/src/libslic3r/Release/liblibslic3r_cgal.a
  ${ORCA_BUILD}/deps_src/admesh/Release/libadmesh.a
  ${ORCA_DEPS_PREFIX}/lib/libboost_log_setup.a
  ${ORCA_DEPS_PREFIX}/lib/libboost_log.a
  ${ORCA_DEPS_PREFIX}/lib/libboost_filesystem.a
  ${ORCA_DEPS_PREFIX}/lib/libboost_coroutine.a
  ${ORCA_DEPS_PREFIX}/lib/libboost_context.a
  ${ORCA_DEPS_PREFIX}/lib/libboost_serialization.a
  ${ORCA_DEPS_PREFIX}/lib/libboost_locale.a
  ${ORCA_DEPS_PREFIX}/lib/libboost_thread.a
  "/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk/usr/lib/libiconv.tbd"
  ${ORCA_DEPS_PREFIX}/lib/libboost_chrono.a
  ${ORCA_DEPS_PREFIX}/lib/libboost_atomic.a
  ${ORCA_DEPS_PREFIX}/lib/libboost_date_time.a
  ${ORCA_DEPS_PREFIX}/lib/libboost_exception.a
  ${ORCA_DEPS_PREFIX}/lib/libboost_program_options.a
  ${ORCA_DEPS_PREFIX}/lib/libboost_container.a
  ${ORCA_DEPS_PREFIX}/lib/libboost_nowide.a
  ${ORCA_DEPS_PREFIX}/lib/libgmpxx.a
  ${ORCA_DEPS_PREFIX}/lib/libmpfr.a
  ${ORCA_DEPS_PREFIX}/lib/libgmp.a
  ${ORCA_BUILD}/deps_src/mcut/Release/libmcut.a
  ${ORCA_DEPS_PREFIX}/lib/liblibnoise_static.a
  ${ORCA_DEPS_PREFIX}/lib/libpng.a
  ${ORCA_DEPS_PREFIX}/lib/libqhullcpp.a
  ${ORCA_DEPS_PREFIX}/lib/libqhullstatic_r.a
  ${ORCA_BUILD}/deps_src/qoi/Release/libqoi.a
  ${ORCA_BUILD}/lib/Release/libsemver.a
  ${ORCA_DEPS_PREFIX}/lib/libtbbmalloc.a
  ${ORCA_DEPS_PREFIX}/lib/libcrypto.a
  "-framework Foundation"
  "-framework ModelIO"
  ${ORCA_DEPS_PREFIX}/lib/libopenvdb.a
  ${ORCA_DEPS_PREFIX}/lib/libboost_iostreams.a
  ${ORCA_DEPS_PREFIX}/lib/libboost_random.a
  ${ORCA_DEPS_PREFIX}/lib/libHalf-2_5.a
  ${ORCA_DEPS_PREFIX}/lib/libblosc.a
  "/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk/usr/lib/libz.tbd"
  ${ORCA_DEPS_PREFIX}/lib/libslvs.a
  ${ORCA_DEPS_PREFIX}/lib/libtbb.a
  ${ORCA_DEPS_PREFIX}/lib/libnlopt.a
  -lm
)

set_target_properties(stage_dump PROPERTIES
  LINK_FLAGS "-Wl,-search_paths_first -Wl,-headerpad_max_install_names"
)

