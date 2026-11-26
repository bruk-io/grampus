//! Port of `src/libslic3r/MarchingSquares.hpp` (task `60-foundation-trees-grids`, plan step 20,
//! `foundation-09` row of `port/foundation.tsv`): marching squares over a 2D scalar raster,
//! extracting the contour rings of the cells above an isovalue threshold.
//! `template-heavy: yes` in `port/manifest.tsv`.
//!
//! **`_RasterTraits<T>` becomes a trait implemented directly on the raster type, not a
//! specialisation point.** The vendored header lets a caller "specialize this struct to register
//! a raster type" (`:132`) without modifying either the raster or this header; Rust's own
//! trait-impl-on-an-external-type *is* that same extension point (`docs/PORTING.md`'s idiom map:
//! "A method on a class -> ... a trait method where the crate map's module defines a trait for
//! it"), so [`RasterTraits`] below is implemented directly on a caller's own raster type rather
//! than through a second generic indirection this module would have to invent.
//!
//! **`m_rst` (`Grid::m_rst`, `:225`) and `CellIt::grid` (`:404`) are `&'a Rst`, taken verbatim
//! from `port/ownership.tsv`** (both rows are `class: borrowed`) -- the two raw-pointer fields
//! this task's own table names. [`Grid`] and [`CellIt`] below both carry the lifetime `'a` this
//! forces.
//!
//! **`_Loop<ExecutionPolicy>` (generic default plus one specialisation for `ExecutionTBB`,
//! `:147-164`) becomes a trait with a default method, not Rust template specialisation** (which
//! does not exist on stable Rust): [`GridLoop`] below gives every `ExecutionPolicy` the
//! sequential default (a plain indexed loop, the vendored generic `_Loop<EP,Enable>`'s own body),
//! and `55-foundation-libslic3r-execution`'s own `ExecutionTbb` overrides it to dispatch through
//! `slic3r_execution::execution::for_each` -- two concrete `impl` blocks, not a blanket-vs-
//! specific conflict, so stable trait dispatch resolves it exactly like the vendored template
//! specialisation does. `execute()` (no policy argument, `:645-648`) passes `nullptr` to
//! `execute_with_policy`, which the generic (non-TBB) `_Loop` default then runs against; this
//! port's own [`execute`] below runs that same sequential default inline, rather than inventing
//! a Rust "null policy" type solely to route through [`GridLoop`].
//!
//! **`Coord`'s `r`/`c` are `long` (`:96`), ported as `i64`.** `long` is not a `type map` row (that
//! table's `coord_t` is a different, `int64_t`-at-the-pin type `libslic3r.h` defines); on this
//! port's build platform (LP64 macOS/Linux arm64/x86_64, the only platform `just oracle-build`
//! targets) `long` is 64 bits, and `Coord` never leaves this header for a scaled/serialised
//! coordinate the type map governs, so `i64` is the simplest faithful width -- stated once here
//! rather than as a marker per occurrence, since no single line transcribes a numeric constant
//! for it.

/// `_RasterTraits<T>` (`MarchingSquares.hpp:132-144`), implemented directly on a caller's raster
/// type -- see the module doc.
pub trait RasterTraits {
    /// `using ValueType = typename T::ValueType;` (`:136`).
    type ValueType: PartialOrd + Copy;
    /// `static ValueType get(const T &raster, size_t row, size_t col);` (`:139`).
    fn get(&self, row: usize, col: usize) -> Self::ValueType;
    /// `static size_t rows(const T &raster);` (`:142`).
    fn rows(&self) -> usize;
    /// `static size_t cols(const T &raster);` (`:143`).
    fn cols(&self) -> usize;
}

/// `struct Coord { long r = 0, c = 0; ... };` (`:94-117`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Coord {
    pub r: i64,
    pub c: i64,
}

impl Coord {
    /// `Coord(long s) : r(s), c(s) {}` (`:99`).
    pub fn splat(s: i64) -> Self {
        Coord { r: s, c: s }
    }

    /// `Coord(long _r, long _c) : r(_r), c(_c) {}` (`:100`).
    pub fn new(r: i64, c: i64) -> Self {
        Coord { r, c }
    }

    /// `size_t seq(const Coord &res) const` (`:102`).
    pub fn seq(&self, res: &Coord) -> usize {
        (self.r * res.c + self.c) as usize
    }
}

impl std::ops::Add for Coord {
    type Output = Coord;
    /// `Coord operator+(const Coord &b) const` (`:109-114`).
    fn add(self, b: Coord) -> Coord {
        Coord::new(self.r + b.r, self.c + b.c)
    }
}

impl std::ops::AddAssign for Coord {
    /// `Coord &operator+=(const Coord &b)` (`:103-108`).
    fn add_assign(&mut self, b: Coord) {
        self.r += b.r;
        self.c += b.c;
    }
}

impl std::fmt::Display for Coord {
    /// `operator<<(std::ostream&, const Coord&)` (`:119`).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(r={},c={})", self.r, self.c)
    }
}

/// `using Ring = std::vector<Coord>;` (`:122`).
pub type Ring = Vec<Coord>;

/// `std::ostream &operator<<(std::ostream&, const Ring&)` (`:124-130`).
pub fn display_ring(ring: &Ring, f: &mut impl std::fmt::Write) -> std::fmt::Result {
    write!(f, "[{}]:", ring.len())?;
    for c in ring {
        write!(f, " {c}")?;
    }
    writeln!(f)
}

/// `enum class Dir : uint8_t { ... };` (`:186-195`). The composite variants (`leftright`,
/// `updown`, `all`) are real, separate discriminants in the vendored enum, not derived at use
/// sites, so they stay separate variants here too rather than becoming a bitflags type (contrast
/// `kd_tree_indirect.rs`'s `VisitorReturnMask`, which really is only ever used as an OR'd mask).
/// No variant carries an explicit Rust discriminant: `leftright`/`updown` are not spelled as
/// literals anywhere in the vendored enum either (`:191`, `:193` compute them as `left | right`/
/// `up | down`), so [`dir_bits`] below mirrors that -- every *other* variant's bit pattern is a
/// transcribed literal with its own `// PORT SRC:` marker, and the two composites are computed
/// from those same named constants, exactly as the vendored enum computes them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    None,
    Left,
    Down,
    Right,
    LeftRight,
    Up,
    UpDown,
    All,
}

/// The bit pattern each [`Dir`] variant carries, named so [`Dir::bits`]/[`Dir::from_bits`] can
/// match on them (a `const` is valid in match-pattern position, which is what lets
/// `LEFTRIGHT`/`UPDOWN` below be *computed* rather than literal and still usable in both
/// directions).
mod dir_bits {
    /// `none = 0b0000` (`MarchingSquares.hpp:187`).
    // PORT SRC: src/libslic3r/MarchingSquares.hpp:187
    pub const NONE: u8 = 0b0000;
    /// `left = 0b0001` (`:188`).
    // PORT SRC: src/libslic3r/MarchingSquares.hpp:188
    pub const LEFT: u8 = 0b0001;
    /// `down = 0b0010` (`:189`).
    // PORT SRC: src/libslic3r/MarchingSquares.hpp:189
    pub const DOWN: u8 = 0b0010;
    /// `right = 0b0100` (`:190`).
    // PORT SRC: src/libslic3r/MarchingSquares.hpp:190
    pub const RIGHT: u8 = 0b0100;
    /// `leftright = left | right` (`:191`): no literal on that line to transcribe.
    pub const LEFTRIGHT: u8 = LEFT | RIGHT;
    /// `up = 0b1000` (`:192`).
    // PORT SRC: src/libslic3r/MarchingSquares.hpp:192
    pub const UP: u8 = 0b1000;
    /// `updown = up | down` (`:193`): no literal on that line to transcribe.
    pub const UPDOWN: u8 = UP | DOWN;
    /// `all = 0b1111` (`:194`).
    // PORT SRC: src/libslic3r/MarchingSquares.hpp:194
    pub const ALL: u8 = 0b1111;
}

impl Dir {
    /// `constexpr std::underlying_type_t<E> _t(E e)` (`:182`), specialised to `Dir`.
    pub fn bits(self) -> u8 {
        use dir_bits::*;
        match self {
            Dir::None => NONE,
            Dir::Left => LEFT,
            Dir::Down => DOWN,
            Dir::Right => RIGHT,
            Dir::LeftRight => LEFTRIGHT,
            Dir::Up => UP,
            Dir::UpDown => UPDOWN,
            Dir::All => ALL,
        }
    }

    /// `Dir((m_dirs[i] >> (o * 4)) & 0b1111)` (`get_dirs`, `:323`): every 4-bit pattern is a
    /// valid `Dir` (the type's own range is exactly 0..=15), so this never fails.
    pub fn from_bits(bits: u8) -> Dir {
        use dir_bits::*;
        match bits & ALL {
            NONE => Dir::None,
            LEFT => Dir::Left,
            DOWN => Dir::Down,
            RIGHT => Dir::Right,
            LEFTRIGHT => Dir::LeftRight,
            UP => Dir::Up,
            UPDOWN => Dir::UpDown,
            ALL => Dir::All,
            // `(m_dirs[i] >> ...) & 0b1111` can only ever read back a value this module's own
            // code wrote, and every write (`clr_dirs`'s `&=`, `NEXT_CCW`'s own table) stays
            // within the seven named patterns above; the eighth 4-bit value with no vendored
            // name (`0b0011`/`0b0110`/.../`0b1101`, any pattern mixing two non-complementary
            // bits) is unreachable, exactly as the vendored `switch` statements' missing
            // `default` cases (`step`, `stepidx`) assume.
            other => unreachable!("Dir::from_bits: unreachable 4-bit pattern {other:#06b}"),
        }
    }
}

impl std::fmt::Display for Dir {
    /// `operator<<(std::ostream&, const Dir&)` (`:197`): `".<v#>x##^#X#####"[_t(d)]`, a
    /// lookup string indexed by the 4-bit discriminant. `&[u8]` (an unsized slice, no explicit
    /// length) rather than `&[u8; 16]`, so the lookup string's own length is never written a
    /// second time as a Rust array-size literal with no vendored line to cite.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const CHARS: &[u8] = b".<v#>x##^#X#####";
        write!(f, "{}", CHARS[self.bits() as usize] as char)
    }
}

/// `NEXT_CCW` (`:202-210`): the square-tag-to-next-direction lookup, computed the same way the
/// vendored immediately-invoked lambda computes it (a `const fn` is this port's direct
/// counterpart of a `constexpr` IIFE).
// PORT SRC: src/libslic3r/MarchingSquares.hpp:202
const fn compute_next_ccw() -> [u32; 16] {
    // `auto map = decltype(NEXT_CCW){};` (`:203`) -- a 16-element array of zero, the same size
    // as the lookup table being built.
    // PORT SRC: src/libslic3r/MarchingSquares.hpp:202
    let mut map = [0u32; 16];
    let mut cbda: u32 = 0;
    // `for (uint32_t cbda = 0; cbda < 16; cbda++)` (`:204`).
    // PORT SRC: src/libslic3r/MarchingSquares.hpp:204
    while cbda < 16 {
        // `const uint32_t dcba = (cbda & 0b0001) | ((cbda >> 1) & 0b0110) | ((cbda << 2) &
        // 0b1000);` (`:205`).
        // PORT SRC: src/libslic3r/MarchingSquares.hpp:205
        let dcba = (cbda & 0b0001) | ((cbda >> 1) & 0b0110) | ((cbda << 2) & 0b1000);
        // `const uint32_t adcb = (dcba >> 1) | ((dcba << 3) & 0b1000);` (`:206`).
        // PORT SRC: src/libslic3r/MarchingSquares.hpp:206
        let adcb = (dcba >> 1) | ((dcba << 3) & 0b1000);
        // `map[cbda] = ~dcba & adcb;` (`:207`) -- no `& 0b1111` mask in the original (`adcb` is
        // already within that range, so `~dcba & adcb` alone is the faithful transcription).
        map[cbda as usize] = !dcba & adcb;
        cbda += 1;
    }
    map
}

// PORT SRC: src/libslic3r/MarchingSquares.hpp:202
const NEXT_CCW: [u32; 16] = compute_next_ccw();

/// `void step(Coord &crd, const Dir d, const long n = 1)` (`:213-221`). The vendored `switch`
/// carries no `default`, silently leaving `crd` unchanged for any other `Dir` value -- `None`,
/// `LeftRight`, `UpDown` and `All` never reach `step` at the pin (only a single resolved
/// direction from `next_dir` ever does), so the wildcard arm below is unreachable in practice,
/// matching that silent fallthrough rather than panicking on it.
pub fn step(crd: &mut Coord, d: Dir, n: i64) {
    match d {
        Dir::Left => crd.c -= n,
        Dir::Down => crd.r += n,
        Dir::Right => crd.c += n,
        Dir::Up => crd.r -= n,
        _ => {}
    }
}

/// `template<class Rst> class Grid` (`MarchingSquares.hpp:223-618`).
pub struct Grid<'a, Rst: RasterTraits> {
    m_rst: &'a Rst,
    m_window: Coord,
    m_rastsize: Coord,
    m_gridsize: Coord,
    m_gridlen: usize,
    m_tags: Vec<u32>,
    m_dirs: Vec<u32>,
}

/// `struct CellIt { ... };` (`:394-468`): a bidirectional walk along one cell edge, stepping by
/// `dir` each increment -- the random-access iterator `interpolate_edge`'s binary search walks.
pub struct CellIt<'a, Rst: RasterTraits> {
    pub crd: Coord,
    pub dir: Dir,
    grid: &'a Rst,
}

// Written by hand rather than `#[derive(Clone, Copy)]`: the derive macro adds a `where Rst:
// Copy`/`Rst: Clone` bound on the generic raster parameter even though this struct only ever
// holds a `&'a Rst` (itself `Copy` regardless of `Rst`), which would force every caller's raster
// type to be `Copy` for no reason this struct's own fields need.
impl<'a, Rst: RasterTraits> Clone for CellIt<'a, Rst> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'a, Rst: RasterTraits> Copy for CellIt<'a, Rst> {}

impl<'a, Rst: RasterTraits> CellIt<'a, Rst> {
    /// `TRasterValue<Rst> operator*() const` (`:406`).
    pub fn value(&self) -> Rst::ValueType {
        self.grid.get(self.crd.r as usize, self.crd.c as usize)
    }

    /// `CellIt &operator+=(long n)` / `operator-=` (`:430-439`).
    pub fn advance(&mut self, n: i64) {
        step(&mut self.crd, self.dir, n);
    }

    /// `long operator-(const CellIt &o) const` (`:452-461`): `*this - o`, in steps along `dir`.
    pub fn distance_from(&self, o: &CellIt<'a, Rst>) -> i64 {
        match self.dir {
            Dir::Left => o.crd.c - self.crd.c,
            Dir::Down => self.crd.r - o.crd.r,
            Dir::Right => self.crd.c - o.crd.c,
            Dir::Up => o.crd.r - self.crd.r,
            _ => 0,
        }
    }
}

/// `struct Edge { CellIt from, to; };` (`:472-475`).
pub struct Edge<'a, Rst: RasterTraits> {
    pub from: CellIt<'a, Rst>,
    pub to: CellIt<'a, Rst>,
}

impl<'a, Rst: RasterTraits> Grid<'a, Rst> {
    /// `explicit Grid(const Rst &rst, const Coord &window)` (`:523-531`).
    pub fn new(rst: &'a Rst, window: Coord) -> Self {
        let rastsize = Coord::new(rst.rows() as i64, rst.cols() as i64);
        let gridsize = Coord::new(2 + rastsize.r / window.r, 2 + rastsize.c / window.c);
        let gridlen = (gridsize.r * gridsize.c) as usize;
        Grid {
            m_rst: rst,
            m_window: window,
            m_rastsize: rastsize,
            m_gridsize: gridsize,
            m_gridlen: gridlen,
            // `m_tags(m_gridlen / 32 + 1, 0)` (`:529`).
            // PORT SRC: src/libslic3r/MarchingSquares.hpp:529
            m_tags: vec![0u32; gridlen / 32 + 1],
            // `m_dirs(m_gridlen / 8 + 1, 0)` (`:530`).
            // PORT SRC: src/libslic3r/MarchingSquares.hpp:530
            m_dirs: vec![0u32; gridlen / 8 + 1],
        }
    }

    /// `Coord rastercoord(const Coord &crd) const` (`:232-236`).
    fn rastercoord(&self, crd: &Coord) -> Coord {
        Coord::new((crd.r - 1) * self.m_window.r, (crd.c - 1) * self.m_window.c)
    }

    /// `Coord tl(const Coord &crd) const` (`:242`).
    fn tl(&self, crd: &Coord) -> Coord {
        self.rastercoord(crd)
    }
    /// `Coord bl(const Coord &crd) const` (`:239`).
    fn bl(&self, crd: &Coord) -> Coord {
        self.tl(crd) + Coord::new(self.m_window.r, 0)
    }
    /// `Coord br(const Coord &crd) const` (`:240`).
    fn br(&self, crd: &Coord) -> Coord {
        self.tl(crd) + Coord::new(self.m_window.r, self.m_window.c)
    }
    /// `Coord tr(const Coord &crd) const` (`:241`).
    fn tr(&self, crd: &Coord) -> Coord {
        self.tl(crd) + Coord::new(0, self.m_window.c)
    }

    /// `bool is_within(const Coord &crd) const` (`:245`).
    fn is_within(&self, crd: &Coord) -> bool {
        crd.r >= 0 && crd.r < self.m_rastsize.r && crd.c >= 0 && crd.c < self.m_rastsize.c
    }

    /// `Coord coord(size_t i) const` (`:327`).
    fn coord(&self, i: usize) -> Coord {
        Coord::new(i as i64 / self.m_gridsize.c, i as i64 % self.m_gridsize.c)
    }

    /// `size_t seq(const Coord &crd) const` (`:330`).
    fn seq(&self, crd: &Coord) -> usize {
        crd.seq(&self.m_gridsize)
    }

    /// `uint32_t get_tags_block32(const size_t bidx, const TRasterValue<Rst> v) const`
    /// (`:248-266`).
    fn get_tags_block32(&self, bidx: usize, v: Rst::ValueType) -> u32 {
        // `Coord gcrd = coord(bidx * 32);` (`:250`).
        // PORT SRC: src/libslic3r/MarchingSquares.hpp:250
        let mut gcrd = self.coord(bidx * 32);
        let mut rcrd = self.rastercoord(&gcrd);
        let mut tags: u32 = 0;
        let mut b: u32 = 1;
        while b != 0 {
            if self.is_within(&rcrd) && self.m_rst.get(rcrd.r as usize, rcrd.c as usize) > v {
                tags |= b;
            }
            gcrd.c += 1;
            rcrd.c += self.m_window.c;
            if gcrd.c >= self.m_gridsize.c {
                gcrd = Coord::new(gcrd.r + 1, 0);
                rcrd = self.rastercoord(&gcrd);
            }
            b <<= 1;
        }
        tags
    }

    /// `uint32_t get_tags9(size_t gidx) const` (`:289-304`).
    fn get_tags9(&self, gidx: usize) -> u32 {
        let mut tags: u32 = 0;
        // `size_t i = gidx / 32;` (`:292`).
        // PORT SRC: src/libslic3r/MarchingSquares.hpp:292
        let i = gidx / 32;
        // `int o = gidx % 32;` (`:293`).
        // PORT SRC: src/libslic3r/MarchingSquares.hpp:293
        let o = gidx % 32;

        if gidx < self.m_gridlen {
            // `tags = (m_tags[i] >> o) & 0x1ff;` (`:297`).
            // PORT SRC: src/libslic3r/MarchingSquares.hpp:297
            tags = (self.m_tags[i] >> o) & 0x1ff;
            // `if ((o > (32 - 9)) && ((i + 1) < m_tags.size()))` (`:299`).
            // PORT SRC: src/libslic3r/MarchingSquares.hpp:299
            if o > (32 - 9) && (i + 1) < self.m_tags.len() {
                // `tags |= (m_tags[i + 1] << (32 - o)) & 0x1ff;` (`:300`).
                // PORT SRC: src/libslic3r/MarchingSquares.hpp:300
                tags |= (self.m_tags[i + 1] << (32 - o)) & 0x1ff;
            }
        }
        tags
    }

    /// `uint32_t get_dirs_block8(const size_t bidx) const` (`:269-286`).
    fn get_dirs_block8(&self, bidx: usize) -> u32 {
        // `size_t gidx = bidx * 8;` (`:271`).
        // PORT SRC: src/libslic3r/MarchingSquares.hpp:271
        let gidx = bidx * 8;
        let mut dirs: u32 = 0;

        // `uint32_t tags9 = get_tags9(gidx) | (get_tags9(gidx + m_gridsize.c) << 16);` (`:276`).
        let tags9 =
            self.get_tags9(gidx) | (self.get_tags9(gidx + self.m_gridsize.c as usize) << 16); // PORT SRC: src/libslic3r/MarchingSquares.hpp:276
                                                                                              // `if ((tags9 != 0) && (tags9 != 0x01ff01ff))` (`:278`).
                                                                                              // PORT SRC: src/libslic3r/MarchingSquares.hpp:278
        if tags9 != 0 && tags9 != 0x01ff01ff {
            let mut tags9 = tags9;
            let mut s = 0;
            // `for (auto s = 0; s < 32; s += 4)` (`:279`).
            // PORT SRC: src/libslic3r/MarchingSquares.hpp:279
            while s < 32 {
                // `uint8_t tags = (tags9 & 0b11) | ((tags9 >> 14) & 0b1100);` (`:280`).
                // PORT SRC: src/libslic3r/MarchingSquares.hpp:280
                let tags = (tags9 & 0b11) | ((tags9 >> 14) & 0b1100);
                dirs |= NEXT_CCW[tags as usize] << s;
                tags9 >>= 1;
                // `s += 4` (`:279`, the same `for` header as the loop condition above).
                // PORT SRC: src/libslic3r/MarchingSquares.hpp:279
                s += 4;
            }
        }
        dirs
    }

    /// `void clr_dirs(const size_t gidx, const Dir d)` (`:307-314`).
    fn clr_dirs(&mut self, gidx: usize, d: Dir) {
        debug_assert!(gidx < self.m_gridlen);
        // `size_t i = gidx / 8;` (`:310`).
        // PORT SRC: src/libslic3r/MarchingSquares.hpp:310
        let i = gidx / 8;
        // `int o = (gidx % 8);` (`:311`).
        // PORT SRC: src/libslic3r/MarchingSquares.hpp:311
        let o = gidx % 8;
        // `m_dirs[i] &= ~(static_cast<uint32_t>(d) << (o * 4));` (`:313`).
        // PORT SRC: src/libslic3r/MarchingSquares.hpp:313
        self.m_dirs[i] &= !((d.bits() as u32) << (o * 4));
    }

    /// `Dir get_dirs(const size_t gidx) const` (`:317-324`).
    fn get_dirs(&self, gidx: usize) -> Dir {
        debug_assert!(gidx < self.m_gridlen);
        // `size_t i = gidx / 8;` (`:320`).
        // PORT SRC: src/libslic3r/MarchingSquares.hpp:320
        let i = gidx / 8;
        // `int o = (gidx % 8);` (`:321`).
        // PORT SRC: src/libslic3r/MarchingSquares.hpp:321
        let o = gidx % 8;
        // `return Dir((m_dirs[i] >> (o * 4)) & 0b1111);` (`:323`).
        // PORT SRC: src/libslic3r/MarchingSquares.hpp:323
        Dir::from_bits(((self.m_dirs[i] >> (o * 4)) & 0b1111) as u8)
    }

    /// `size_t stepidx(const size_t idx, const Dir d) const` (`:333-343`). The vendored
    /// `default: return idx;` fallthrough is kept explicitly, for the same reason [`step`]'s is.
    fn stepidx(&self, idx: usize, d: Dir) -> usize {
        debug_assert!(idx < self.m_gridlen);
        match d {
            Dir::Left => idx - 1,
            Dir::Down => idx + self.m_gridsize.c as usize,
            Dir::Right => idx + 1,
            Dir::Up => idx - self.m_gridsize.c as usize,
            _ => idx,
        }
    }

    /// `size_t search_start_cell(size_t gidx = 0) const` (`:346-367`).
    fn search_start_cell(&self, gidx: usize) -> usize {
        // `size_t i = gidx / 8;` (`:348`).
        // PORT SRC: src/libslic3r/MarchingSquares.hpp:348
        let mut i = gidx / 8;
        // `int o = gidx % 8;` (`:349`).
        // PORT SRC: src/libslic3r/MarchingSquares.hpp:349
        let mut o = gidx % 8;

        while i < self.m_dirs.len() {
            if self.m_dirs[i] == 0 {
                i += 1;
                o = 0;
            } else {
                // `for (uint32_t m = 0b1111 << (o * 4); !(m_dirs[i] & m); m <<= 4)` (`:361`).
                // PORT SRC: src/libslic3r/MarchingSquares.hpp:361
                let mut m: u32 = 0b1111 << (o * 4);
                while self.m_dirs[i] & m == 0 {
                    o += 1;
                    // `m <<= 4` (`:361`, the same `for` header as the initialiser above).
                    // PORT SRC: src/libslic3r/MarchingSquares.hpp:361
                    m <<= 4;
                }
                break;
            }
        }
        // `return i * 8 + o;` (`:366`).
        // PORT SRC: src/libslic3r/MarchingSquares.hpp:366
        i * 8 + o
    }

    /// `Dir next_dir(size_t idx, Dir prev = Dir::all) const` (`:370-392`).
    fn next_dir(&self, idx: usize, prev: Dir) -> Dir {
        let next = self.get_dirs(idx);

        match next {
            Dir::LeftRight => {
                debug_assert!(prev == Dir::Up || prev == Dir::Down || prev == Dir::All);
                if prev == Dir::Up {
                    Dir::Left
                } else {
                    Dir::Right
                }
            }
            Dir::UpDown => {
                debug_assert!(prev == Dir::Left || prev == Dir::Right || prev == Dir::All);
                if prev == Dir::Right {
                    Dir::Up
                } else {
                    Dir::Down
                }
            }
            _ => {
                debug_assert!(matches!(
                    next,
                    Dir::None | Dir::Left | Dir::Down | Dir::Right | Dir::Up
                ));
                next
            }
        }
    }

    /// `Edge _edge(const Coord &ringvertex) const` (`:477-491`).
    fn edge_raw(&self, ringvertex: &Coord) -> Edge<'a, Rst> {
        let idx = ringvertex.r as usize;
        let cell = self.coord(idx);
        let d = Dir::from_bits(ringvertex.c as u8);

        let (from_crd, from_dir, to_crd) = match d {
            Dir::Left => (self.tl(&cell), Dir::Down, self.bl(&cell)),
            Dir::Down => (self.bl(&cell), Dir::Right, self.br(&cell)),
            Dir::Right => (self.br(&cell), Dir::Up, self.tr(&cell)),
            Dir::Up => (self.tr(&cell), Dir::Left, self.tl(&cell)),
            _ => {
                // PORT NOTE: MarchingSquares.hpp:488's `default: assert(false);` is compiled out
                // under the oracle's release build (NDEBUG), falling through to `return {};`
                // (`:490`) with a default, null-grid `Edge`. This port makes that unreachable
                // branch a live panic instead of silently returning a bogus `Edge`.
                unreachable!("edge_raw: ringvertex direction must be one of the four resolved ones")
            }
        };
        Edge {
            from: CellIt {
                crd: from_crd,
                dir: from_dir,
                grid: self.m_rst,
            },
            to: CellIt {
                crd: to_crd,
                dir: from_dir,
                grid: self.m_rst,
            },
        }
    }

    /// `Edge edge(const Coord &ringvertex) const` (`:493-504`).
    fn edge(&self, ringvertex: &Coord) -> Edge<'a, Rst> {
        let mut e = self.edge_raw(ringvertex);
        e.to.dir = e.from.dir;
        e.to.advance(1);

        e.from.crd.r = e.from.crd.r.clamp(0, self.m_rastsize.r - 1);
        e.from.crd.c = e.from.crd.c.clamp(0, self.m_rastsize.c - 1);
        e.to.crd.r = e.to.crd.r.clamp(0, self.m_rastsize.r);
        e.to.crd.c = e.to.crd.c.clamp(0, self.m_rastsize.c);
        e
    }

    /// `void interpolate_edge(Coord &ecrd, TRasterValue<Rst> isoval) const` (`:506-520`).
    /// `std::lower_bound(e.from, e.to, isoval)` is translated as an explicit binary search over
    /// the half-open `[from, to)` range along the edge's own direction, rather than a generic
    /// Rust iterator abstraction built solely to carry one call: the same technique
    /// `kd_tree_indirect.rs`'s `partition_input` uses for translating a `std::` algorithm call
    /// statement for statement.
    fn interpolate_edge(&self, ecrd: &mut Coord, isoval: Rst::ValueType) {
        debug_assert!(ecrd.r >= 0 && (ecrd.r as usize) < self.m_gridlen);
        let d = Dir::from_bits(ecrd.c as u8);
        debug_assert!(matches!(d, Dir::Left | Dir::Down | Dir::Right | Dir::Up));

        let e = self.edge(ecrd);
        let n = e.to.distance_from(&e.from);
        let mut lo: i64 = 0;
        let mut hi: i64 = n;
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            let mut candidate = e.from;
            candidate.advance(mid);
            if candidate.value() < isoval {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        let mut found = e.from;
        found.advance(lo);
        *ecrd = found.crd;

        // Shift bottom and right side points "out" by one to account for raster pixel width.
        if e.from.dir == Dir::Up {
            ecrd.r += 1;
        } else if e.from.dir == Dir::Left {
            ecrd.c += 1;
        }
    }

    /// `void tag_grid(ExecutionPolicy &&policy, TRasterValue<Rst> isoval)` (`:534-544`).
    ///
    /// Each pass takes its own vector (`m_tags`, then `m_dirs`) out of `self` with
    /// `std::mem::take` before looping over it, so the loop's own `&self.get_..._block..` calls
    /// -- which need the *whole* `&self` -- never alias the `&mut` loop target, which is now a
    /// local rather than a field of `self`; the vector is put back once the pass finishes. This
    /// is Rust's borrow checker forcing out explicitly what the vendored method leaves implicit
    /// (`this->m_tags[...]` written while `this->get_tags_block32` reads other fields of the
    /// same `this`), not a behaviour difference from the vendored two-pass body.
    pub fn tag_grid<P: GridLoop>(&mut self, policy: &P, isoval: Rst::ValueType) {
        let mut tags = std::mem::take(&mut self.m_tags);
        policy.for_each_idx(&mut tags, |tag_block, bidx| {
            *tag_block = self.get_tags_block32(bidx, isoval);
        });
        self.m_tags = tags;

        let mut dirs = std::mem::take(&mut self.m_dirs);
        policy.for_each_idx(&mut dirs, |dirs_block, bidx| {
            *dirs_block = self.get_dirs_block8(bidx);
        });
        self.m_dirs = dirs;
    }

    /// `std::vector<Ring> scan_rings()` (`:550-577`).
    pub fn scan_rings(&mut self) -> Vec<Ring> {
        let mut rings = Vec::new();
        let mut startidx = 0usize;
        loop {
            startidx = self.search_start_cell(startidx);
            if startidx >= self.m_gridlen {
                break;
            }
            let mut ring = Ring::new();

            let mut idx = startidx;
            let mut next = self.next_dir(idx, Dir::All);
            loop {
                debug_assert!(next != Dir::None);
                let ringvertex = Coord::new(idx as i64, next.bits() as i64);
                ring.push(ringvertex);
                self.clr_dirs(idx, next);
                idx = self.stepidx(idx, next);
                next = self.next_dir(idx, next);
                debug_assert!(idx < self.m_gridlen);
                if idx == startidx {
                    break;
                }
            }
            debug_assert!(matches!(next, Dir::None | Dir::Left | Dir::Up));
            if ring.len() > 1 {
                rings.push(ring);
            }
        }
        rings
    }

    /// `void interpolate_rings(ExecutionPolicy &&policy, std::vector<Ring> &rings,
    /// TRasterValue<Rst> isov)` (`:581-587`).
    pub fn interpolate_rings<P: GridLoop>(
        &self,
        policy: &P,
        rings: &mut Vec<Ring>,
        isov: Rst::ValueType,
    ) {
        policy.for_each_idx(rings, |ring, _| {
            for e in ring.iter_mut() {
                self.interpolate_edge(e, isov);
            }
        });
    }

    /// `std::ostream &streamtags(std::ostream &os)` (`:589-602`).
    pub fn streamtags(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(f, "   :")?;
        for c in 0..self.m_gridsize.c {
            // `os << (c % 10);` (`:593`).
            // PORT SRC: src/libslic3r/MarchingSquares.hpp:593
            write!(f, "{}", c % 10)?;
        }
        writeln!(f)?;
        for r in 0..self.m_gridsize.r {
            write!(f, "{r:>3}:")?;
            for c in 0..self.m_gridsize.c {
                let on = self.get_tags9(self.seq(&Coord::new(r, c))) & 1 != 0;
                write!(f, "{}", if on { "H" } else { "." })?;
            }
            writeln!(f)?;
        }
        Ok(())
    }

    /// `std::ostream &streamdirs(std::ostream &os)` (`:604-617`).
    pub fn streamdirs(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(f, "   :")?;
        for c in 0..self.m_gridsize.c {
            // `os << (c % 10);` (`:608`).
            // PORT SRC: src/libslic3r/MarchingSquares.hpp:608
            write!(f, "{}", c % 10)?;
        }
        writeln!(f)?;
        for r in 0..self.m_gridsize.r {
            write!(f, "{r:>3}:")?;
            for c in 0..self.m_gridsize.c {
                write!(f, "{}", self.get_dirs(self.seq(&Coord::new(r, c))))?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

/// `_Loop<ExecutionPolicy, Enable>` (`:147-164`) -- see the module doc.
pub trait GridLoop {
    /// `_Loop<ExecutionPolicy>::for_each_idx` generic default (`:149-153`): a plain sequential
    /// indexed walk.
    fn for_each_idx<T>(&self, items: &mut [T], mut f: impl FnMut(&mut T, usize)) {
        for (i, item) in items.iter_mut().enumerate() {
            f(item, i);
        }
    }
}

impl GridLoop for slic3r_execution::execution_seq::ExecutionSeq {}

impl GridLoop for slic3r_execution::execution_tbb::ExecutionTbb {
    /// `_Loop<Slic3r::ExecutionTBB>::for_each_idx` (`:157-164`): dispatches through
    /// `Slic3r::execution::for_each`, which `55-foundation-libslic3r-execution`'s own
    /// `ExecutionTbb::for_each` runs as a single serial loop (see that module's own doc) -- so
    /// this is the real vendored dispatch, not a second sequential loop invented here.
    fn for_each_idx<T>(&self, items: &mut [T], mut f: impl FnMut(&mut T, usize)) {
        let n = items.len();
        let max_concurrency = slic3r_execution::execution::max_concurrency(self);
        slic3r_execution::execution::for_each(self, 0..n, |i| f(&mut items[i], i), max_concurrency);
    }
}

/// `execute_with_policy(ExecutionPolicy&&, const Raster&, TRasterValue<Raster>, Coord)`
/// (`:620-643`).
pub fn execute_with_policy<Rst: RasterTraits, P: GridLoop>(
    policy: &P,
    raster: &Rst,
    isoval: Rst::ValueType,
    mut windowsize: Coord,
) -> Vec<Ring> {
    if raster.rows() == 0 || raster.cols() == 0 {
        return Vec::new();
    }

    let ratio = (raster.cols() / raster.rows()) as i64;

    if windowsize.r == 0 {
        windowsize.r = 2;
    }
    if windowsize.c == 0 {
        windowsize.c = std::cmp::max(2, windowsize.r * ratio);
    }

    let mut grid = Grid::new(raster, windowsize);

    grid.tag_grid(policy, isoval);
    let mut rings = grid.scan_rings();
    grid.interpolate_rings(policy, &mut rings, isoval);

    rings
}

/// `execute(const Raster &raster, TRasterValue<Raster> isoval, Coord windowsize = {})`
/// (`:645-648`): `execute_with_policy(nullptr, ...)`, which dispatches to the generic (non-TBB)
/// `_Loop` default -- see the module doc's "`nullptr` policy" note for why this port runs that
/// sequential default inline rather than through [`GridLoop`].
pub fn execute<Rst: RasterTraits>(
    raster: &Rst,
    isoval: Rst::ValueType,
    windowsize: Coord,
) -> Vec<Ring> {
    struct Sequential;
    impl GridLoop for Sequential {}
    execute_with_policy(&Sequential, raster, isoval, windowsize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use slic3r_proof::{compare_unit, proof_units, Oracle};

    /// A flat row-major raster, the simplest faithful `RasterTraits` implementor -- the same
    /// shape the proof shim's own `ShimRaster` registers on the C++ side.
    struct VecRaster {
        data: Vec<f64>,
        rows: usize,
        cols: usize,
    }

    impl RasterTraits for VecRaster {
        type ValueType = f64;
        fn get(&self, row: usize, col: usize) -> f64 {
            self.data[row * self.cols + col]
        }
        fn rows(&self) -> usize {
            self.rows
        }
        fn cols(&self) -> usize {
            self.cols
        }
    }

    /// An 8x8 raster with one filled blob above `isoval`, flattened row-major -- small enough to
    /// trace by hand, large enough to exercise the multi-block `m_tags`/`m_dirs` packing (32 tags
    /// and 8 dirs per `u32` word) across more than one word.
    fn blob_raster() -> VecRaster {
        #[rustfmt::skip]
        let data: Vec<f64> = vec![
            0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 1., 1., 1., 0., 0., 0.,
            0., 1., 1., 1., 1., 1., 0., 0.,
            0., 1., 1., 1., 1., 1., 1., 0.,
            0., 1., 1., 1., 1., 1., 0., 0.,
            0., 0., 1., 1., 1., 0., 0., 0.,
            0., 0., 0., 1., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0.,
        ];
        VecRaster {
            data,
            rows: 8,
            cols: 8,
        }
    }

    #[test]
    fn execute_finds_one_ring_around_the_blob() {
        let raster = blob_raster();
        let rings = execute(&raster, 0.5, Coord::new(2, 2));
        assert_eq!(rings.len(), 1, "one connected blob makes exactly one ring");
        assert!(
            rings[0].len() > 4,
            "a non-trivial blob's ring has more than 4 vertices"
        );
    }

    /// Proof path 1 (R19, `port/manifest.tsv`'s `MarchingSquares.hpp` row): `crates/slic3r-ffi/
    /// cxx/proof_units/marching_squares.cpp` registers a `marchsq::_RasterTraits` specialisation
    /// over the same flat raster and calls the real vendored `marchsq::execute` free function.
    /// Every ring and vertex this port finds, in the same order, must match the oracle exactly
    /// -- the flattened `[len, r0, c0, r1, c1, ...]*` encoding both sides return makes that one
    /// `Vec<i64>` equality check.
    // PROOF: 1
    #[test]
    fn marching_squares_execute_matches_the_oracle() {
        let raster = blob_raster();
        let window = Coord::new(2, 2);
        let isoval = 0.5;

        let rings = execute(&raster, isoval, window);
        let mut actual: Vec<i64> = Vec::new();
        for ring in &rings {
            actual.push(ring.len() as i64);
            for c in ring {
                actual.push(c.r);
                actual.push(c.c);
            }
        }

        let oracle = proof_units::marching_squares::path1_execute(
            &raster.data,
            raster.rows,
            raster.cols,
            isoval,
            window.r,
            window.c,
        );
        compare_unit("MarchingSquares.hpp", Oracle::Built, actual, oracle)
            .expect("marchsq::execute must match the vendored template exactly");
    }
}

// PORT STATUS
// source: src/libslic3r/MarchingSquares.hpp
// confidence: medium
// tier: exact
// oracle: built
