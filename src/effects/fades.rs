use std::time::Duration;

use crate::{ChannelCount, SampleRate};

use crate::Float;
use crate::Sample;
use crate::math::NANOS_PER_SEC;
use crate::math::duration_to_float;

use crate::effects::pure_effect;

pub(super) mod fade_out {
    use super::*;
    pure_effect! {
        struct FadeOut {
            ramp: Ramp,
        }

        fn next(&mut self) -> Option<Sample> {
            let sr = self.inner.sample_rate();
            let ch = self.inner.channels();
            self.inner.next().map(|s| self.ramp.apply(s, ch, sr))
        }

        fn new(source: S, duration: Duration) -> FadeOut<Self> {
            Self {
                inner: source,
                ramp: Ramp::new(duration, 1.0, 0.0, true),
            }
        }
    }
}

pub(super) mod fade_out_after {
    use super::*;
    pure_effect! {
        struct FadeOutAfter {
            ramp: Ramp,
            samples_until_ramp: usize,
        }

        fn next(&mut self) -> Option<Sample> {
            let sr = self.inner.sample_rate();
            let ch = self.inner.channels();
            let sample = self.inner.next()?;

            Some(if self.samples_until_ramp == 0 {
                self.ramp.apply(sample, ch, sr)
            } else {
                self.samples_until_ramp -= 1;
                sample
            })
        }

        fn new(source: S, start_after: Duration, fade_duration: Duration) -> FadeOut<Self> {
            let samples_until_ramp = source.sample_rate().get() as f64
                * source.channels().get() as f64
                * start_after.as_secs_f64();

            Self {
                inner: source,
                ramp: Ramp::new(fade_duration, 1.0, 0.0, true),
                samples_until_ramp: samples_until_ramp as usize,
            }
        }
    }
}

pub(super) mod fade_in {
    use super::*;
    pure_effect! {
        struct FadeIn {
            ramp: Ramp,
        }

        fn next(&mut self) -> Option<Sample> {
            let sr = self.inner.sample_rate();
            let ch = self.inner.channels();
            self.inner.next().map(|s| self.ramp.apply(s, ch, sr))
        }

        fn new(source: S, duration: Duration) -> FadeIn<Self> {
            Self {
                inner: source,
                ramp: Ramp::new(duration, 0.0, 1.0, false),
            }
        }
    }
}

pub(super) mod linear_ramp {
    use super::*;
    pure_effect! {
        struct LinearGainRamp {
            ramp: Ramp,
        }

        fn next(&mut self) -> Option<Sample> {
            let sr = self.inner.sample_rate();
            let ch = self.inner.channels();
            self.inner.next().map(|s| self.ramp.apply(s, ch, sr))
        }

        fn new(source: S, duration: Duration, start: Float, end: Float, clamp: bool) -> LinearGainRamp<Self> {
            Self {
                inner: source,
                ramp: Ramp::new(duration, start, end, clamp),
            }
        }
    }
}

#[derive(Clone)]
pub(crate) struct Ramp {
    elapsed: Duration,
    total: Duration,
    start_gain: Float,
    end_gain: Float,
    clamp_end: bool,
    sample_idx: u64,
}

impl Ramp {
    fn new(duration: Duration, start_gain: Float, end_gain: Float, clamp_end: bool) -> Self {
        assert!(!duration.is_zero(), "duration must be greater than zero");
        Self {
            elapsed: Duration::ZERO,
            total: duration,
            start_gain,
            end_gain,
            clamp_end,
            sample_idx: 0u64,
        }
    }

    fn apply(&mut self, sample: Sample, channels: ChannelCount, sample_rate: SampleRate) -> Sample {
        let factor: Float;

        if self.elapsed >= self.total {
            if self.clamp_end {
                factor = self.end_gain;
            } else {
                factor = 1.0;
            }
        } else {
            self.sample_idx += 1;

            // Calculate progress (0.0 to 1.0) using appropriate precision for Float type
            let p = duration_to_float(self.elapsed) / duration_to_float(self.total);

            factor = self.start_gain * (1.0 - p) + self.end_gain * p;
        }

        if self.sample_idx.is_multiple_of(channels.get() as u64) {
            let sample_duration = Duration::from_nanos(NANOS_PER_SEC / sample_rate.get() as u64);
            self.elapsed += sample_duration;
        }

        sample * factor
    }
}
