use crate::ChannelCount;
use crate::SampleRate;
use crate::effects::pure_effect;

mod bezier;

pub trait IntoAmplifyFactor {
    type Factor: AmplifyFactor;
    fn into_factor(self, channel_count: ChannelCount, sample_rate: SampleRate) -> Self::Factor;
}

// TODO what is a fadeout but a changing amplify?
trait AmplifyFactor {
    fn linear_gain(&mut self) -> f32;
    fn seek(&mut self, position: usize);
}

pure_effect! {
    struct Amplify<F: AmplifyFactor> {
        factor: F,
    }

    fn next(&mut self) -> Option<Sample> {
        self.inner.next().map(|value| value * self.factor.linear_gain())
    }

    fn new<A: IntoAmplifyFactor>(source: S, factor: A) -> Amplify<Self> {
        Self {
            factor: factor.into_factor(source.channels(), source.sample_rate()),
            inner: source,
        }
    }

    pub fn set_factor(&mut self, factor: F) {
        self.factor = factor;
    }
}
