use crate::{ChannelCount, SampleRate};

use std::ops::RangeInclusive;
use std::time::Duration;

use super::{Envelope, IntoEnvelope, Scale};

#[derive(Clone)]
pub struct LinearEnvelope {
    skip: usize,
    step: usize,
    steps: usize,
    gain: RangeInclusive<f32>,
    scale: Scale,
}

impl LinearEnvelope {
    pub fn builder() -> LinearEnvelopeBuilder {
        LinearEnvelopeBuilder::default()
    }
}

pub struct LinearEnvelopeBuilder {
    gain: std::ops::RangeInclusive<f32>,
    takes: Duration,
    delay: Duration,
    scale: Scale,
}

impl Default for LinearEnvelopeBuilder {
    fn default() -> Self {
        Self {
            gain: 0.0..=1.0,
            takes: Duration::from_secs(1),
            delay: Duration::ZERO,
            scale: Scale::default(),
        }
    }
}

impl LinearEnvelopeBuilder {
    pub fn with_gain(mut self, gain: std::ops::RangeInclusive<f32>) -> Self {
        self.gain = gain;
        self
    }

    pub fn with_scale(mut self, scale: Scale) -> Self {
        self.scale = scale;
        self
    }

    pub fn takes(mut self, duration: Duration) -> Self {
        self.takes = duration;
        self
    }

    pub fn start_after(mut self, duration: Duration) -> Self {
        self.delay = duration;
        self
    }
}

impl IntoEnvelope for LinearEnvelopeBuilder {
    type Envelope = LinearEnvelope;
    fn into_envelope(self, channel_count: ChannelCount, sample_rate: SampleRate) -> Self::Envelope {
        let skip = channel_count.get() as f32 * sample_rate.get() as f32 * self.delay.as_secs_f32();
        let steps =
            channel_count.get() as f32 * sample_rate.get() as f32 * self.takes.as_secs_f32();

        Self::Envelope {
            skip: skip as usize,
            step: 0,
            steps: steps as usize,
            gain: self.gain,
            scale: self.scale,
        }
    }
}

impl Envelope for LinearEnvelope {
    fn linear_gain(&mut self) -> f32 {
        let gain = if self.step < self.skip {
            *self.gain.start()
        } else if self.step > self.skip + self.steps {
            *self.gain.end()
        } else {
            let diff = self.gain.end() - self.gain.start();
            let progress = (self.step - self.skip) as f32 / self.steps as f32;
            diff * progress
        };

        self.scale.gain_to_linear(gain)
    }
    fn seek(&mut self, _position: usize) {
        todo!()
    }
}
