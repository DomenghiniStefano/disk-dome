//! Rectangles: integer pixels for the output, `f64` while the layout divides space.

/// A rectangle of whole pixels, half-open: it covers `x0..x1` × `y0..y1`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
}

impl Rect {
    pub const fn new(x0: i32, y0: i32, x1: i32, y1: i32) -> Self {
        Self { x0, y0, x1, y1 }
    }

    pub const fn width(&self) -> i32 {
        self.x1 - self.x0
    }

    pub const fn height(&self) -> i32 {
        self.y1 - self.y0
    }

    /// Pixels covered (0 for an empty or inverted rectangle).
    pub fn area(&self) -> u64 {
        if self.is_empty() {
            0
        } else {
            self.width() as u64 * self.height() as u64
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.x1 <= self.x0 || self.y1 <= self.y0
    }

    /// Whether the two rectangles share at least one pixel.
    pub const fn intersects(&self, other: &Rect) -> bool {
        self.x0 < other.x1 && other.x0 < self.x1 && self.y0 < other.y1 && other.y0 < self.y1
    }

    /// Whether `other` lies entirely inside `self`.
    pub const fn contains(&self, other: &Rect) -> bool {
        other.x0 >= self.x0 && other.y0 >= self.y0 && other.x1 <= self.x1 && other.y1 <= self.y1
    }
}

/// A rectangle in floating-point pixels, used while the layout divides space.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub(crate) struct FRect {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

impl FRect {
    pub fn width(&self) -> f64 {
        self.x1 - self.x0
    }

    pub fn height(&self) -> f64 {
        self.y1 - self.y0
    }

    /// Rounds every edge to the nearest pixel. Two rectangles sharing an edge round it the same
    /// way, so snapped rectangles neither overlap nor leave holes.
    pub fn snap(&self) -> Rect {
        Rect::new(
            self.x0.round() as i32,
            self.y0.round() as i32,
            self.x1.round() as i32,
            self.y1.round() as i32,
        )
    }

    pub fn from_rect(r: Rect) -> Self {
        Self {
            x0: f64::from(r.x0),
            y0: f64::from(r.y0),
            x1: f64::from(r.x1),
            y1: f64::from(r.y1),
        }
    }
}
