#![allow(unused)]
//! Math utilities for audio processing.

use crate::common::SampleRate;
use std::time::Duration;

/// Nanoseconds per second, used for Duration calculations.
pub(crate) const NANOS_PER_SEC: u64 = 1_000_000_000;

// Re-export float constants with appropriate precision for the Float type.
// This centralizes all cfg gating for constants in one place.
#[cfg(not(feature = "64bit"))]
pub use std::f32::consts::{E, LN_2, LN_10, LOG2_10, LOG2_E, LOG10_2, LOG10_E, PI, TAU};
#[cfg(feature = "64bit")]
pub use std::f64::consts::{E, LN_2, LN_10, LOG2_10, LOG2_E, LOG10_2, LOG10_E, PI, TAU};

/// Linear interpolation between two samples.
///
/// The result should be equivalent to
/// `first * (1 - numerator / denominator) + second * numerator / denominator`.
///
/// To avoid numeric overflows pick smaller numerator.
// TODO (refactoring) Streamline this using coefficient instead of numerator and denominator.
#[inline]
pub(crate) fn lerp(first: Sample, second: Sample, numerator: u32, denominator: u32) -> Sample {
    first + (second - first) * numerator as Float / denominator as Float
}

/// Converts decibels to linear amplitude scale.
///
/// This function converts a decibel value to its corresponding linear amplitude value
/// using the formula: `linear = 10^(decibels/20)` for amplitude.
///
/// # Arguments
///
/// * `decibels` - The decibel value to convert. Common ranges:
///   - 0 dB = linear value of 1.0 (no change)
///   - Positive dB values represent amplification (> 1.0)
///   - Negative dB values represent attenuation (< 1.0)
///   - -60 dB ≈ 0.001 (barely audible)
///   - +20 dB = 10.0 (10x amplification)
///
/// # Returns
///
/// The linear amplitude value corresponding to the input decibels.
///
/// # Performance
///
/// This implementation is optimized for speed, being ~3-4% faster than the standard
/// `10f32.powf(decibels * 0.05)` approach, with a maximum error of only 2.48e-7
/// (representing about -132 dB precision).
#[inline]
pub fn db_to_linear(decibels: Float) -> Float {
    // ~3-4% faster than using `10f32.powf(decibels * 0.05)`,
    // with a maximum error of 2.48e-7 representing only about -132 dB.
    Float::powf(2.0, decibels * 0.05 * LOG2_10)
}

/// Converts linear amplitude scale to decibels.
///
/// This function converts a linear amplitude value to its corresponding decibel value
/// using the formula: `decibels = 20 * log10(linear)` for amplitude.
///
/// # Arguments
///
/// * `linear` - The linear amplitude value to convert. Must be positive for meaningful results:
///   - 1.0 = 0 dB (no change)
///   - Values > 1.0 represent amplification (positive dB)
///   - Values < 1.0 represent attenuation (negative dB)
///   - 0.0 results in negative infinity
///   - Negative values are not physically meaningful for amplitude
///
/// # Returns
///
/// The decibel value corresponding to the input linear amplitude.
///
/// # Performance
///
/// This implementation is optimized for speed, being faster than the standard
/// `20.0 * linear.log10()` approach while maintaining high precision.
///
/// # Special Cases
///
/// - `linear_to_db(0.0)` returns negative infinity
/// - Very small positive values approach negative infinity
/// - Negative values return NaN (not physically meaningful for amplitude)
#[inline]
pub fn linear_to_db(linear: Float) -> Float {
    // Same as `to_linear`: faster than using `20f32.log10() * linear`
    linear.log2() * LOG10_2 * 20.0
}

/// Converts a time duration to a smoothing coefficient for exponential filtering.
///
/// Used for both attack and release filtering in the limiter's envelope detector.
/// Creates a coefficient that determines how quickly the limiter responds to level changes:
/// * Longer times = higher coefficients (closer to 1.0) = slower, smoother response
/// * Shorter times = lower coefficients (closer to 0.0) = faster, more immediate response
///
/// The coefficient is calculated using the formula: `e^(-1 / (duration_seconds * sample_rate))`
/// which provides exponential smoothing behavior suitable for audio envelope detection.
///
/// # Arguments
///
/// * `duration` - Desired response time (attack or release duration)
/// * `sample_rate` - Audio sample rate in Hz
///
/// # Returns
///
/// Smoothing coefficient in the range [0.0, 1.0] for use in exponential filters
#[must_use]
pub(crate) fn duration_to_coefficient(duration: Duration, sample_rate: SampleRate) -> Float {
    Float::exp(-1.0 / (duration_to_float(duration) * sample_rate.get() as Float))
}

pub(crate) fn normalized_to_linear(normalized: f32) -> f32 {
    const NORMALIZATION_MIN: f32 = 0.0;
    const NORMALIZATION_MAX: f32 = 1.0;
    const LOG_VOLUME_GROWTH_RATE: f32 = 6.907_755_4;
    const LOG_VOLUME_SCALE_FACTOR: f32 = 1000.0;

    let normalized = normalized.clamp(NORMALIZATION_MIN, NORMALIZATION_MAX);

    let mut amplitude = f32::exp(LOG_VOLUME_GROWTH_RATE * normalized) / LOG_VOLUME_SCALE_FACTOR;
    if normalized < 0.1 {
        amplitude *= normalized * 10.0;
    }
    amplitude
}

/// Convert Duration to Float with appropriate precision for the Sample type.
#[inline]
#[must_use]
pub(crate) fn duration_to_float(duration: Duration) -> Float {
    #[cfg(not(feature = "64bit"))]
    {
        duration.as_secs_f32()
    }
    #[cfg(feature = "64bit")]
    {
        duration.as_secs_f64()
    }
}

#[must_use]
pub(crate) fn nearest_multiple_of_two(n: u32) -> u32 {
    if n <= 1 {
        return 1;
    }
    let next = n.next_power_of_two();
    let prev = next >> 1;
    if n - prev <= next - n { prev } else { next }
}

/// Utility macro for getting a `NonZero` from a literal. Especially
/// useful for passing in `ChannelCount` and `Samplerate`.
/// Equivalent to: `const { core::num::NonZero::new($n).unwrap() }`
///
/// # Example
/// ```
/// use rodio::nz;
/// use rodio::static_buffer::StaticSamplesBuffer;
/// let buffer = StaticSamplesBuffer::new(nz!(2), nz!(44_100), &[0.0, 0.5, 0.0, 0.5]);
/// ```
///
/// # Panics
/// If the literal passed in is zero this panicks.
#[macro_export]
macro_rules! nz {
    ($n:literal) => {
        const { core::num::NonZero::new($n).unwrap() }
    };
}

pub use nz;

use crate::{Sample, common::Float};
