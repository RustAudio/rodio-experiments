use crate::ChannelCount;
use crate::SampleRate;
use crate::effects::pure_effect;
use crate::math::normalized_to_linear;

mod curve;
mod linear;
pub use curve::{CurveControls, CurveEnvelope};
pub use linear::LinearEnvelope;

pub trait IntoEnvelope {
    type Envelope: Envelope;
    fn into_envelope(self, channel_count: ChannelCount, sample_rate: SampleRate) -> Self::Envelope;
}

// TODO what is a fadeout but a changing amplify?
pub trait Envelope {
    fn linear_gain(&mut self) -> f32;
    fn seek(&mut self, position: usize);
}

#[derive(Debug, Default, Clone, Copy)]
pub enum Scale {
    #[default]
    Linear,
    Normalized,
}
impl Scale {
    fn gain_to_linear(&self, gain: f32) -> f32 {
        match self {
            Scale::Linear => gain,
            Scale::Normalized => normalized_to_linear(gain),
        }
    }
}

pure_effect! {
    struct Fade<F: Envelope> {
        factor: F,
    }

    fn next(&mut self) -> Option<Sample> {
        self.inner.next().map(|value| value * self.factor.linear_gain())
    }

    fn new<A: IntoEnvelope<Envelope = F>>(source: S, factor: A) -> Amplify<Self> {
        Self {
            factor: factor.into_envelope(source.channels(), source.sample_rate()),
            inner: source,
        }
    }
}
