// adapted from rodio. Copyright rodio contributors.
// Tests and docs removed for brevity (will be re-added once contributing to upstream)

use std::time::Duration;

use crate::nz;

use crate::SampleRate;

use crate::FixedSource;

use super::{Function, GeneratorFunction};
use super::{sawtooth_signal, sine_signal, square_signal, triangle_signal};

/// An infinite source that produces one of a selection of test waveforms.
#[derive(Clone, Debug)]
pub struct SignalGenerator {
    function: GeneratorFunction,
    sample_rate: SampleRate,
    phase_step: f32,
    phase: f32,
}

impl SignalGenerator {
    pub const fn new(frequency: f32, f: Function, sample_rate: SampleRate) -> Self {
        let function: GeneratorFunction = match f {
            Function::Sine => sine_signal,
            Function::Triangle => triangle_signal,
            Function::Square => square_signal,
            Function::Sawtooth => sawtooth_signal,
        };

        Self::with_function(frequency, function, sample_rate)
    }

    pub const fn with_function(
        frequency: f32,
        generator_function: GeneratorFunction,
        sample_rate: SampleRate,
    ) -> Self {
        assert!(frequency > 0.0, "frequency must be greater than zero");
        let period = sample_rate.get() as f32 / frequency;
        let phase_step = 1.0f32 / period;

        SignalGenerator {
            function: generator_function,
            sample_rate,
            phase_step,
            phase: 0.0f32,
        }
    }
}

impl Iterator for SignalGenerator {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<f32> {
        let f = self.function;
        let val = Some(f(self.phase));
        self.phase = (self.phase + self.phase_step).rem_euclid(1.0f32);
        val
    }
}

impl FixedSource for SignalGenerator {
    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        None
    }

    fn channels(&self) -> rodio::ChannelCount {
        nz!(1)
    }

    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    // TODO support try_seek (lets take classic impl will fix it up later)
}

macro_rules! signal_new_type {
    ($name:ident, $function:expr) => {
        #[derive(Clone, Debug)]
        pub struct $name {
            inner: SignalGenerator,
        }

        impl $name {
            /// The frequency of the sine.
            #[inline]
            pub fn new(freq: f32, sample_rate: SampleRate) -> Self {
                Self {
                    inner: SignalGenerator::new(freq, $function, sample_rate),
                }
            }
        }

        impl Iterator for $name {
            type Item = f32;

            #[inline]
            fn next(&mut self) -> Option<f32> {
                self.inner.next()
            }
        }

        impl FixedSource for $name {
            #[inline]
            fn total_duration(&self) -> Option<Duration> {
                self.inner.total_duration()
            }

            #[inline]
            fn channels(&self) -> rodio::ChannelCount {
                self.inner.channels()
            }

            #[inline]
            fn sample_rate(&self) -> SampleRate {
                self.inner.sample_rate
            }
        }
    };
}

signal_new_type!(SineWave, Function::Sine);
signal_new_type!(SawtoothWave, Function::Sawtooth);
signal_new_type!(SquareWave, Function::Square);
signal_new_type!(TriangleWave, Function::Triangle);
