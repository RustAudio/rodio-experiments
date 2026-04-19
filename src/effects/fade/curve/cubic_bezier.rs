use crate::effects::fade::curve::CurveControls;

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
    pub(crate) const fn new(controls: CurveControls, gain: std::ops::RangeInclusive<f32>) -> Self {
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

        fn pick_probable_root<const N: usize>(options: [f32; N]) -> f32 {
            options
                .into_iter().find(|pos| (-0.00001..=1.00001).contains(pos))
                .expect("no sensible root")
        }

        match find_roots_cubic::<f32>(a3, a2, a1, a0) {
            Roots::No(_) => panic!("Curve should be in range 0-1"),
            Roots::One([t]) => t,
            Roots::Two(options) => pick_probable_root(options),
            Roots::Three(options) => pick_probable_root(options),
            Roots::Four(options) => pick_probable_root(options),
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

#[cfg(test)]
mod tests {
    use super::*;

    const LINE_DOWN: CubicBezier = CubicBezier::new(
        CurveControls {
            x1: 0.0,
            y1: 1.0,
            x2: 1.0,
            y2: 0.0,
        },
        1.0..=0.0,
    );

    const EASE_IN_OUT_QUINT: CubicBezier = CubicBezier::new(
        CurveControls {
            x1: 0.83,
            y1: 0.0,
            x2: 0.17,
            y2: 1.0,
        },
        0.0..=1.0,
    );

    #[test]
    fn line() {
        assert_eq!(LINE_DOWN.pre_y(), 1.0);
        assert_eq!(LINE_DOWN.post_y(), 0.0);

        for x in (0..10).into_iter().map(|i| i as f32 / 10.0) {
            let t = LINE_DOWN.t(x);
            let y = LINE_DOWN.y(t);
            let expected_y = 1.0 - x;
            assert!((y - expected_y).abs() < 0.00001, "t: {t}");
        }
    }

    #[test]
    fn ease_in_out_is_always_ascending() {
        // always ascending
        let prev_y = 0.0;
        for x in (1..10).into_iter().map(|i| i as f32 / 10.0) {
            let t = EASE_IN_OUT_QUINT.t(x);
            let y = EASE_IN_OUT_QUINT.y(t);
            assert!(y > prev_y, "t: {t}");
        }
    }

    #[test]
    fn manually_checked_coords() {
        const KNOWN_CURVE: CubicBezier = CubicBezier::new(
            CurveControls {
                x1: 0.25,
                y1: 0.8,
                x2: 0.75,
                y2: 0.8,
            },
            0.0..=1.0,
        );

        let correct = [
            (0.000, 0.0000, 0.0000),
            (0.020, 0.0153, 0.0470),
            (0.030, 0.0232, 0.0699),
            (0.050, 0.0393, 0.1141),
            (0.120, 0.0999, 0.2552),
            (0.390, 0.3769, 0.6303),
            (0.430, 0.4214, 0.6677),
            (0.440, 0.4326, 0.6765),
            (0.770, 0.7939, 0.8816),
            (0.810, 0.8339, 0.9008),
            (0.830, 0.8533, 0.9104),
            (0.940, 0.9524, 0.9659),
            (0.990, 0.9924, 0.9941),
            (1.000, 1.0000, 1.0000),
        ];

        for (t, x, y) in correct {
            let t_calculated = KNOWN_CURVE.t(x);
            let y_calculated = KNOWN_CURVE.y(t_calculated);
            let msg = format!(
                "
    x: {x}
    t: {t}, t_calculated: {t_calculated}, 
    y: {y}, y_calculated: {y_calculated}"
            );
            assert!((t_calculated - t).abs() < 0.001, "t mismatch: {}", msg);
            assert!((y_calculated - y).abs() < 0.001, "y mismatch: {}", msg);
        }
    }
}
