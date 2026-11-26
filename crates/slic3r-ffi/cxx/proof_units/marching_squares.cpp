// Proof shim for the `MarchingSquares.hpp` unit (R19, proof path 1, task
// `60-foundation-trees-grids`), compiled onto `crates/slic3r-ffi`'s shared `cxx_build::Build` --
// the one sanctioned route a ported unit reaches the oracle by (`docs/PORTING.md`,
// "Decisions recorded on 2026-11-25", decision 1).
//
// Registers a minimal `marchsq::_RasterTraits<ShimRaster>` specialisation (the same extension
// point the Rust port's own `RasterTraits` trait is modelled on -- see
// `marching_squares.rs`'s module doc) over a flat `rust::Vec<double>` passed across the bridge,
// and calls the real vendored `marchsq::execute` free function.
#include "marching_squares.h"

#include "MarchingSquares.hpp"

#include <vector>

namespace slic3r_ffi {
namespace proof_units {
namespace marching_squares {

namespace {

struct ShimRaster {
    using ValueType = double;
    const std::vector<double> &data;
    size_t rows;
    size_t cols;
};

} // namespace

} // namespace marching_squares
} // namespace proof_units
} // namespace slic3r_ffi

// `_RasterTraits` is specialised at namespace scope (`marchsq`), exactly as
// `MarchingSquares.hpp:132`'s own comment ("Specialize this struct to register a raster type")
// asks a caller to do.
namespace marchsq {

template<>
struct _RasterTraits<slic3r_ffi::proof_units::marching_squares::ShimRaster> {
    using ValueType = double;
    static ValueType get(const slic3r_ffi::proof_units::marching_squares::ShimRaster &raster,
                          size_t row, size_t col) {
        return raster.data[row * raster.cols + col];
    }
    static size_t rows(const slic3r_ffi::proof_units::marching_squares::ShimRaster &raster) {
        return raster.rows;
    }
    static size_t cols(const slic3r_ffi::proof_units::marching_squares::ShimRaster &raster) {
        return raster.cols;
    }
};

} // namespace marchsq

namespace slic3r_ffi {
namespace proof_units {
namespace marching_squares {

rust::Vec<int64_t> path1_execute(const rust::Vec<double> &raster, size_t rows, size_t cols,
                                  double isoval, int64_t window_r, int64_t window_c) {
    std::vector<double> data(raster.begin(), raster.end());
    ShimRaster shim{data, rows, cols};
    marchsq::Coord windowsize(window_r, window_c);
    std::vector<marchsq::Ring> rings = marchsq::execute(shim, isoval, windowsize);

    rust::Vec<int64_t> out;
    for (const marchsq::Ring &ring : rings) {
        out.push_back(static_cast<int64_t>(ring.size()));
        for (const marchsq::Coord &c : ring) {
            out.push_back(c.r);
            out.push_back(c.c);
        }
    }
    return out;
}

} // namespace marching_squares
} // namespace proof_units
} // namespace slic3r_ffi
