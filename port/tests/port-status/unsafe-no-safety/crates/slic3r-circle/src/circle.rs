unsafe fn drop_raw(ptr: *mut f64) {
    drop(Box::from_raw(ptr));
}

// PORT STATUS
// source: src/libslic3r/Geometry/Circle.cpp
// confidence: high
// tier: exact
// oracle: built
