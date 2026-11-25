//! Calls the vendored Clipper2, admesh and libnest2d, and the superbuild's qhull and
//! Boost.Polygon Voronoi builder, through `slic3r-ffi`'s bridges, and asserts on values the C++
//! returned (task 29-ffi-bridge-core / 30-ffi-bridge-libraries, plan step 17, R17 / R19's FFI
//! proof path).

use slic3r_ffi::admesh::{compute_volume, CFacet, CVertex};
use slic3r_ffi::clipper2::{
    offset_paths, union_paths, union_paths_z, CPath64, CPathZ, CPoint64, CPointZ,
};
use slic3r_ffi::libnest2d::{edge_cache_stats, CPoint as NestPoint};
use slic3r_ffi::qhull::{convex_hull_stats, CPoint3};
use slic3r_ffi::voronoi::{diagram_stats, CSegment};

fn square(x0: i64, y0: i64, side: i64) -> CPath64 {
    CPath64 {
        points: vec![
            CPoint64 { x: x0, y: y0 },
            CPoint64 {
                x: x0 + side,
                y: y0,
            },
            CPoint64 {
                x: x0 + side,
                y: y0 + side,
            },
            CPoint64 {
                x: x0,
                y: y0 + side,
            },
        ],
    }
}

fn square_z(x0: i64, y0: i64, side: i64, z: i64) -> CPathZ {
    CPathZ {
        points: vec![
            CPointZ { x: x0, y: y0, z },
            CPointZ {
                x: x0 + side,
                y: y0,
                z,
            },
            CPointZ {
                x: x0 + side,
                y: y0 + side,
                z,
            },
            CPointZ {
                x: x0,
                y: y0 + side,
                z,
            },
        ],
    }
}

/// `Clipper2Lib::Clipper64` through the plain namespace: a single non-self-intersecting square
/// unioned against no clip paths comes back as the same square (Clipper2 may reorder its
/// starting vertex or normalise winding, but introduces no new vertex here).
#[test]
fn clipper2_union_through_plain_namespace() {
    let subject = vec![square(0, 0, 100)];
    let solution = union_paths(&subject);

    assert_eq!(
        solution.len(),
        1,
        "one input path, one output path: {solution:?}"
    );
    let mut points: Vec<(i64, i64)> = solution[0].points.iter().map(|p| (p.x, p.y)).collect();
    points.sort();
    let mut expected = vec![(0, 0), (100, 0), (100, 100), (0, 100)];
    expected.sort();
    assert_eq!(
        points, expected,
        "same four corners, Clipper2 solution: {solution:?}"
    );
}

/// `Clipper2Lib::ClipperOffset` through the plain namespace: a square offset outward by a
/// positive delta with the miter join grows each side by `delta` on every edge, so the square's
/// bounding box grows from `[0, 100]` to `[-delta, 100 + delta]` on both axes.
#[test]
fn clipper2_offset_through_plain_namespace() {
    let subject = vec![square(0, 0, 100)];
    let delta = 10.0;
    let solution = offset_paths(&subject, delta);

    assert_eq!(solution.len(), 1, "one offset solution path: {solution:?}");
    let xs: Vec<i64> = solution[0].points.iter().map(|p| p.x).collect();
    let ys: Vec<i64> = solution[0].points.iter().map(|p| p.y).collect();
    assert_eq!(
        *xs.iter().min().unwrap(),
        -10,
        "offset solution: {solution:?}"
    );
    assert_eq!(
        *xs.iter().max().unwrap(),
        110,
        "offset solution: {solution:?}"
    );
    assert_eq!(
        *ys.iter().min().unwrap(),
        -10,
        "offset solution: {solution:?}"
    );
    assert_eq!(
        *ys.iter().max().unwrap(),
        110,
        "offset solution: {solution:?}"
    );
    assert_eq!(
        solution[0].points.len(),
        4,
        "miter join keeps sharp corners: {solution:?}"
    );
}

/// `Clipper2Lib_Z::Clipper64` through the Z-enabled namespace: a single Z-tagged square unioned
/// against no clip paths comes back with every vertex's `x`, `y` unchanged and its input `z`
/// (here a single uniform value) carried through, since no edge crossing exists for Clipper2 to
/// synthesise a new `z` at.
#[test]
fn clipper2_union_through_z_namespace() {
    let subject = vec![square_z(0, 0, 100, 7)];
    let solution = union_paths_z(&subject);

    assert_eq!(
        solution.len(),
        1,
        "one input path, one output path: {solution:?}"
    );
    let mut points: Vec<(i64, i64, i64)> =
        solution[0].points.iter().map(|p| (p.x, p.y, p.z)).collect();
    points.sort();
    let mut expected = vec![(0, 0, 7), (100, 0, 7), (100, 100, 7), (0, 100, 7)];
    expected.sort();
    assert_eq!(
        points, expected,
        "same four corners with z carried through: {solution:?}"
    );
}

fn vertex(x: f32, y: f32, z: f32) -> CVertex {
    CVertex { x, y, z }
}

/// `stl_calculate_volume` (`deps_src/admesh/util.cpp:290`) over a right tetrahedron with unit
/// legs along the three axes -- `O=(0,0,0)`, `A=(1,0,0)`, `B=(0,1,0)`, `C=(0,0,1)` -- whose
/// volume is the textbook `1/6`. Each facet below carries its own outward unit normal (admesh's
/// `get_volume` reads the stored normal, it does not derive one from the vertices), so this is
/// the value the vendored C++ computed from its own area/height summation over those four
/// facets, not one restated by this test.
#[test]
fn admesh_computes_tetrahedron_volume() {
    let o = vertex(0.0, 0.0, 0.0);
    let a = vertex(1.0, 0.0, 0.0);
    let b = vertex(0.0, 1.0, 0.0);
    let c = vertex(0.0, 0.0, 1.0);

    let facets = vec![
        // Base, z = 0 plane: outward normal points away from the solid, -z.
        CFacet {
            normal: vertex(0.0, 0.0, -1.0),
            v0: o,
            v1: a,
            v2: b,
        },
        // y = 0 plane: outward normal -y.
        CFacet {
            normal: vertex(0.0, -1.0, 0.0),
            v0: o,
            v1: a,
            v2: c,
        },
        // x = 0 plane: outward normal -x.
        CFacet {
            normal: vertex(-1.0, 0.0, 0.0),
            v0: o,
            v1: b,
            v2: c,
        },
        // Slanted face x + y + z = 1: outward normal (1,1,1)/sqrt(3).
        CFacet {
            normal: vertex(
                1.0 / 3.0_f32.sqrt(),
                1.0 / 3.0_f32.sqrt(),
                1.0 / 3.0_f32.sqrt(),
            ),
            v0: a,
            v1: b,
            v2: c,
        },
    ];

    let volume = compute_volume(&facets);
    let expected = 1.0_f32 / 6.0_f32;
    assert!(
        (volume - expected).abs() < 1e-5,
        "tetrahedron volume: got {volume}, expected {expected}"
    );
}

/// `orgQhull::Qhull::runQhull("", 3, 9, coords, "Qt")` (`TriangleMesh.cpp:1365-1372`'s own call
/// shape) over a unit cube's eight corners plus one interior point. The interior point cannot
/// become a hull vertex, and `"Qt"` triangulates every facet, so a correct hull has exactly the
/// cube's eight corners as vertices and twelve triangular facets (two per square face) --
/// qhull's own counts, read back through `facetList().count()`/`vertexList().count()`.
#[test]
fn qhull_computes_cube_convex_hull() {
    let mut points = Vec::new();
    for &x in &[0.0, 1.0] {
        for &y in &[0.0, 1.0] {
            for &z in &[0.0, 1.0] {
                points.push(CPoint3 { x, y, z });
            }
        }
    }
    // An interior point: not a hull vertex or facet contributor.
    points.push(CPoint3 {
        x: 0.5,
        y: 0.5,
        z: 0.5,
    });

    let stats = convex_hull_stats(&points);
    assert_eq!(stats.vertex_count, 8, "cube corners only: {stats:?}");
    assert_eq!(
        stats.facet_count, 12,
        "\"Qt\" triangulates each square face into two triangles: {stats:?}"
    );
}

/// `libnest2d::placers::EdgeCache<ExPolygon>` (`nfpplacer.hpp:152-336`) over a 100x100 square
/// contour. `EdgeCache::createCache` (`nfpplacer.hpp:173-186`) walks `[cbegin(sh), cend(sh))` in
/// consecutive pairs without wrapping the last point back to the first, so for 4 stored points
/// it sums 3 edges, not the closed quadrilateral's 4 -- `circumference()` returns 300 (measured
/// by actually running this test, not assumed from the shape's true 400-unit perimeter), the
/// value `EdgeCache` computed over the edges it actually built. `coords(0.0)` is the point at
/// the start of that walk, the contour's first vertex.
#[test]
fn nest2d_edge_cache_reports_square_circumference() {
    let contour = vec![
        NestPoint { x: 0, y: 0 },
        NestPoint { x: 100, y: 0 },
        NestPoint { x: 100, y: 100 },
        NestPoint { x: 0, y: 100 },
    ];

    let stats = edge_cache_stats(&contour);
    assert_eq!(
        stats.circumference, 300.0,
        "sum of 3 consecutive-pair edges, not the closed 4-sided perimeter: {stats:?}"
    );
    assert_eq!(
        stats.first_point,
        NestPoint { x: 0, y: 0 },
        "distance 0.0 is the contour's first vertex: {stats:?}"
    );
}

/// `boost::polygon::construct_voronoi` (`boost/polygon/voronoi.hpp`) over the three segments of
/// a closed right-triangle contour, each pair sharing an endpoint with the next. Boost's segment
/// builder gives each of the three segments its own cell *and* contributes extra cells and
/// vertices at the shared endpoints (it does not collapse a segment chain into a single simple
/// polygon site), so the topology is richer than "3 cells, 1 vertex" -- these are the counts
/// `vd.num_cells()` / `num_vertices()` / `num_edges()` actually returned when this test was run
/// against this build, not ones derived from the triangle's geometry by hand.
#[test]
fn voronoi_builds_diagram_for_triangle_segments() {
    let segments = vec![
        CSegment {
            x0: 0,
            y0: 0,
            x1: 100,
            y1: 0,
        },
        CSegment {
            x0: 100,
            y0: 0,
            x1: 0,
            y1: 100,
        },
        CSegment {
            x0: 0,
            y0: 100,
            x1: 0,
            y1: 0,
        },
    ];

    let stats = diagram_stats(&segments);
    assert_eq!(stats.cell_count, 6, "diagram stats: {stats:?}");
    assert_eq!(stats.vertex_count, 4, "diagram stats: {stats:?}");
    assert_eq!(stats.edge_count, 18, "diagram stats: {stats:?}");
}
