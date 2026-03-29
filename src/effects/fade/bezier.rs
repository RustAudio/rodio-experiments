use std::time::Duration;

use super::{Envelope, IntoEnvelope};
use crate::effects::fade::Scale;
use crate::{ChannelCount, SampleRate};

mod cubic_bezier;
use cubic_bezier::CubicBezier;

pub struct CurveBuilder {
    curve: CurveControls,
    gain: std::ops::RangeInclusive<f32>,
    duration: Duration,
    start_after: Duration,
    scale: Scale,
}

impl Default for CurveBuilder {
    fn default() -> Self {
        const LINEAR_DOWN: CurveControls = CurveControls {
            x1: 0.0,
            y1: 1.0,
            x2: 1.0,
            y2: 0.0,
        };

        Self {
            curve: LINEAR_DOWN,
            gain: 1.0..=0.0,
            duration: Duration::from_secs(1),
            start_after: Duration::ZERO,
            scale: Scale::default(),
        }
    }
}

pub struct CurveControls {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl CurveBuilder {
    pub fn with_curve(mut self, curve: CurveControls) -> Self {
        self.curve = curve;
        self
    }

    pub fn with_scale(mut self, scale: Scale) -> Self {
        self.scale = scale;
        self
    }

    pub fn with_gain(mut self, gain: std::ops::RangeInclusive<f32>) -> Self {
        self.gain = gain;
        self
    }

    pub fn takes(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn start_after(mut self, duration: Duration) -> Self {
        self.start_after = duration;
        self
    }
}

pub struct CurveEnvelope {
    bezier: CubicBezier,
    skip: usize,
    step: usize,
    steps: usize,
    scale: Scale,
}

impl CurveEnvelope {
    pub fn builder() -> CurveBuilder {
        CurveBuilder::default()
    }
}

impl IntoEnvelope for CurveBuilder {
    type Envelope = CurveEnvelope;

    fn into_envelope(self, channel_count: ChannelCount, sample_rate: SampleRate) -> Self::Envelope {
        let skip =
            channel_count.get() as f32 * sample_rate.get() as f32 * self.start_after.as_secs_f32();
        let steps =
            channel_count.get() as f32 * sample_rate.get() as f32 * self.duration.as_secs_f32();

        let bezier = CubicBezier::new(self.curve, self.gain);
        Self::Envelope {
            bezier,
            skip: skip as usize,
            step: 0,
            steps: steps as usize,
            scale: self.scale,
        }
    }
}

impl Envelope for CurveEnvelope {
    fn linear_gain(&mut self) -> f32 {
        let gain = if self.step < self.skip {
            self.bezier.pre_y()
        } else if self.step > self.skip + self.steps {
            self.bezier.post_y()
        } else {
            let x = self.step as f32 / self.steps as f32;
            self.steps += 1;
            let t = self.bezier.t(x);
            self.bezier.y(t)
        };

        self.scale.gain_to_linear(gain)
    }

    fn seek(&mut self, _: usize) {
        todo!()
    }
}
