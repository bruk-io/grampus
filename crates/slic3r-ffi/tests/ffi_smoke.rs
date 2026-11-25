//! Calls the vendored Clipper2 through `slic3r-ffi`'s bridge, in both the plain and the
//! Z-enabled namespace, and asserts on the integer coordinates the C++ returned (task
//! 29-ffi-bridge-core, plan step 17, R17 / R19's FFI proof path).

use slic3r_ffi::clipper2::{
    offset_paths, union_paths, union_paths_z, CPath64, CPathZ, CPoint64, CPointZ,
};

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
