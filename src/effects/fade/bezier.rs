use std::time::Duration;

use super::{AmplifyFactor, IntoAmplifyFactor};
use crate::{ChannelCount, SampleRate};

struct Point {
    x: f32,
    y: f32,
}

struct CubicBezier {
    p0: Point,
    p1: Point,
    p2: Point,
    p3: Point,
}

impl CubicBezier {
    fn new(controls: BezierControls, gain: std::ops::RangeInclusive<f32>) -> Self {
        Self {
            p0: Point {
                x: 0.,
                y: *gain.start(),
            },
            p1: controls.control1,
            p2: controls.control2,
            p3: Point {
                x: 1.,
                y: *gain.end(),
            },
        }
    }

    // https://stackoverflow.com/questions/66774792/given-a-cubic-bezier-curve-with-fixed-endpoints-how-can-i-find-the-x-position-o
    fn t(&self, x: f32) -> f32 {
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

    fn y(&self, t: f32) -> f32 {
        let Self { p0, p1, p2, p3 } = self;

        p0.y * (1.0 - t).powi(3)
            + 3.0 * p1.y * (1.0 - t).powi(2) * t
            + 3.0 * p2.y * (1.0 - t) * t.powi(2)
            + p3.y * t.powi(3)
    }
}

struct EnvelopeBuilder {
    curve: CubicBezier,
    duration: Duration,
    start_after: Duration,
}

struct BezierControls {
    control1: Point,
    control2: Point,
}

const ATTACK: BezierControls = BezierControls {
    control1: Point { x: 0.0, y: 0.77 },
    control2: Point { x: 0.32, y: 1.0 },
};

// nice editor for cubic bezier curves
// https://www.desmos.com/calculator/iogphoixw4
const DECAY: BezierControls = BezierControls {
    control1: Point { x: 0.0, y: 0.0 },
    control2: Point { x: 1.0, y: 0.0 },
};

const RELEASE: BezierControls = BezierControls {
    control1: Point { x: 0.0, y: 1.0 },
    control2: Point { x: 1.0, y: 0.0 },
};

trait IntoCurve {
    fn into_curve(self) -> CubicBezier;
}

impl IntoCurve for (BezierControls, std::ops::RangeInclusive<f32>) {
    fn into_curve(self) -> CubicBezier {
        CubicBezier {
            p0: Point {
                x: 0.0,
                y: *self.1.start(),
            },
            p1: self.0.control1,
            p2: self.0.control2,
            p3: Point {
                x: 1.0,
                y: *self.1.end(),
            },
        }
    }
}

impl EnvelopeBuilder {
    fn new() -> Self {
        const LINEAR_DOWN: BezierControls = BezierControls {
            control1: Point { x: 0.0, y: 1.0 },
            control2: Point { x: 1.0, y: 0.0 },
        };

        Self {
            curve: CubicBezier::new(LINEAR_DOWN, 1.0..=0.0),
            duration: Duration::from_secs(1),
            start_after: Duration::ZERO,
        }
    }

    fn with_curve(mut self, curve: impl IntoCurve) -> Self {
        self.curve = curve.into_curve();
        self
    }

    fn takes(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    fn start_after(mut self, duration: Duration) -> Self {
        self.start_after = duration;
        self
    }
}

struct Envelope {
    curve: CubicBezier,
    x: f32,
    skip: usize,
    step: usize,
    steps: usize,
}

impl Iterator for Envelope {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if !(self.skip..self.skip + self.steps).contains(&self.step) {
            return None;
        }

        let x = self.step as f32 / self.steps as f32;
        self.steps += 1;
        let t = self.curve.t(x);
        Some(self.curve.y(t))
    }
}

impl IntoAmplifyFactor for EnvelopeBuilder {
    type Factor = Envelope;

    fn into_factor(self, channel_count: ChannelCount, sample_rate: SampleRate) -> Self::Factor {
        let skip =
            channel_count.get() as f32 * sample_rate.get() as f32 * self.start_after.as_secs_f32();
        let steps =
            channel_count.get() as f32 * sample_rate.get() as f32 * self.duration.as_secs_f32();

        Self::Factor {
            curve: self.curve,
            x: 0.0,
            skip: skip as usize,
            step: 0,
            steps: steps as usize,
        }
    }
}

impl AmplifyFactor for Envelope {
    fn linear_gain(&mut self) -> f32 {
        self.next().unwrap_or(1.0)
    }
    fn seek(&mut self, _: usize) {
        todo!()
    }
}
