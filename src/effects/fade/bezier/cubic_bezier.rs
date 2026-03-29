use crate::effects::fade::bezier::CurveControls;

pub struct Point {
    pub x: f32,
    pub y: f32,
}

pub(crate) struct CubicBezier {
    p0: Point,
    p1: Point,
    p2: Point,
    p3: Point,
}

impl CubicBezier {
    pub(crate) fn new(controls: CurveControls, gain: std::ops::RangeInclusive<f32>) -> Self {
        Self {
            p0: Point {
                x: 0.,
                y: *gain.start(),
            },
            p1: Point {
                x: controls.x1,
                y: controls.y1,
            },
            p2: Point {
                x: controls.x2,
                y: controls.y2,
            },
            p3: Point {
                x: 1.,
                y: *gain.end(),
            },
        }
    }

    // https://stackoverflow.com/questions/66774792/given-a-cubic-bezier-curve-with-fixed-endpoints-how-can-i-find-the-x-position-o
    pub(crate) fn t(&self, x: f32) -> f32 {
        use roots::{Roots, find_roots_cubic};
        let Self { p0, p1, p2, p3 } = self;

        let a3 = p3.x - 3.0 * p2.x + 3.0 * p1.x - p0.x;
        let a2 = 3.0 * p2.x - 6.0 * p1.x + 3.0 * p0.x;
        let a1 = 3.0 * p1.x - 3.0 * p0.x;
        let a0 = p0.x - x;

        match find_roots_cubic::<f32>(a3, a2, a1, a0) {
            Roots::No(_) => panic!("Curve should be in range 0-1"),
            Roots::One([t]) => t,
            Roots::Two([t1, _]) => t1,
            Roots::Three([t1, ..]) => t1,
            Roots::Four([t1, ..]) => t1,
        }
    }

    pub(crate) fn y(&self, t: f32) -> f32 {
        let Self { p0, p1, p2, p3 } = self;

        p0.y * (1.0 - t).powi(3)
            + 3.0 * p1.y * (1.0 - t).powi(2) * t
            + 3.0 * p2.y * (1.0 - t) * t.powi(2)
            + p3.y * t.powi(3)
    }

    pub(crate) fn pre_y(&self) -> f32 {
        self.p0.y
    }

    pub(crate) fn post_y(&self) -> f32 {
        self.p3.y
    }
}
