use std::time::Duration;

use crate::ConstSource;
use crate::Float;
use crate::Sample;
use crate::SampleRate;

use crate::math::TAU;

/// Generate a sine wave with an instantaneous frequency that changes/sweeps
/// linearly over time. At the end of the chirp, once the `end_frequency` is
/// reached, the source is exhausted.
#[derive(Clone, Debug)]
pub struct Chirp<const SR: u32, const CH: u16> {
    start_frequency: Float,
    end_frequency: Float,
    total_samples: u64,
    elapsed_samples: u64,
}

impl<const SR: u32, const CH: u16> Chirp<SR, CH> {
    pub fn new(
        sample_rate: SampleRate,
        start_frequency: Float,
        end_frequency: Float,
        duration: Duration,
    ) -> Self {
        Self {
            start_frequency,
            end_frequency,
            total_samples: (duration.as_secs_f64() * sample_rate.get() as f64) as u64,
            elapsed_samples: 0,
        }
    }
}

impl<const SR: u32, const CH: u16> Iterator for Chirp<SR, CH> {
    type Item = Sample;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.elapsed_samples;
        if i >= self.total_samples {
            return None; // Exhausted
        }

        let ratio = (i as f64 / self.total_samples as f64) as Float;
        let freq = self.start_frequency * (1.0 - ratio) + self.end_frequency * ratio;
        let t = (i as f64 / SR as f64) as Float * TAU * freq;

        self.elapsed_samples += 1;
        Some(t.sin())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.total_samples - self.elapsed_samples;
        (remaining as usize, Some(remaining as usize))
    }
}

impl<const SR: u32, const CH: u16> ConstSource<SR, CH> for Chirp<SR, CH> {
    fn total_duration(&self) -> Option<std::time::Duration> {
        let secs = self.total_samples as f64 / SR as f64;
        Some(Duration::from_secs_f64(secs))
    }
}
