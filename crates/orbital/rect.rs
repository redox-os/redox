use std::cmp::{min, max};

#[derive(Copy, Clone, Debug, Default)]
pub struct Rect {
    x: i32,
    y: i32,
    w: i32,
    h: i32
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Rect {
        assert!(w >= 0);
        assert!(h >= 0);

        Rect{
            x: x,
            y: y,
            w: w,
            h: h
        }
    }

    pub fn area(&self) -> i32 {
        self.w * self.h
    }

    pub fn left(&self) -> i32 {
        self.x
    }

    pub fn right(&self) -> i32 {
        self.x + self.w
    }

    pub fn top(&self) -> i32 {
        self.y
    }

    pub fn bottom(&self) -> i32 {
        self.y + self.h
    }

    pub fn width(&self) -> i32 {
        self.w
    }

    pub fn height(&self) -> i32 {
        self.h
    }

    pub fn container(&self, other: &Rect) -> Rect {
        let left = min(self.left(), other.left());
        let right = max(self.right(), other.right());
        let top = min(self.top(), other.top());
        let bottom = max(self.bottom(), other.bottom());

        assert!(left <= right);
        assert!(top <= bottom);

        Rect::new(left, top, right - left, bottom - top)
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        self.left() <= x
        && self.right() >= x
        && self.top() <= y
        && self.bottom() >= y
    }

    pub fn contains_rect(&self, other: &Rect) -> bool {
        self.left() <= other.left()
        && self.right() >= other.right()
        && self.top() <= other.top()
        && self.bottom() >= other.bottom()
    }

    pub fn is_empty(&self) -> bool {
        self.w == 0 || self.h == 0
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        ! (
            self.left() > other.right() ||
            self.right() < other.left() ||
            self.top() > other.bottom() ||
            self.bottom() < other.top()
        )
    }

    pub fn intersection(&self, other: &Rect) -> Rect {
        let left = max(self.left(), other.left());
        let right = min(self.right(), other.right());
        let top = max(self.top(), other.top());
        let bottom = min(self.bottom(), other.bottom());

        Rect::new(left, top, max(0, right - left), max(0, bottom - top))
    }

    pub fn offset(&self, x: i32, y: i32) -> Rect {
        Rect::new(self.x + x, self.y + y, self.w, self.h)
    }
}
