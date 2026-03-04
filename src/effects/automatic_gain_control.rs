//
//      Automatic Gain Control (AGC) Algorithm
//      Designed by @UnknownSuperficialNight
//
//   Features:
//   • Adaptive peak detection
//   • RMS-based level estimation
//   • Asymmetric attack/release
//   • RMS-based general adjustments with peak limiting
//
//   Optimized for smooth and responsive gain control
//
//   Crafted with love. Enjoy! :)
//

use crate::effects::pure_effect;
// use super::SeekError;
use crate::math::duration_to_coefficient;
use crate::{Float, Sample};
use std::time::Duration;

use crate::{SampleRate};

/// Ensures `RMS_WINDOW_SIZE` is a power of two
const fn power_of_two(n: usize) -> usize {
    assert!(
        n.is_power_of_two(),
        "RMS_WINDOW_SIZE must be a power of two"
    );
    n
}

/// Size of the circular buffer used for RMS calculation.
/// A larger size provides more stable RMS values but increases latency.
const RMS_WINDOW_SIZE: usize = power_of_two(8192);

/// Settings for the Automatic Gain Control (AGC).
///
/// This struct contains parameters that define how the AGC will function,
/// allowing users to customize its behaviour.
#[derive(Debug, Clone)]
pub struct AutomaticGainControlSettings {
    /// The desired output level that the AGC tries to maintain.
    /// A value of 1.0 means no change to the original level.
    pub target_level: Float,
    /// Time constant for gain increases (how quickly the AGC responds to level increases).
    /// Longer durations result in slower, more gradual gain increases.
    pub attack_time: Duration,
    /// Time constant for gain decreases (how quickly the AGC responds to level decreases).
    /// Shorter durations allow for faster response to sudden loud signals.
    pub release_time: Duration,
    /// Maximum allowable gain multiplication to prevent excessive amplification.
    /// This acts as a safety limit to avoid distortion from over-amplification.
    pub absolute_max_gain: Float,
}

impl Default for AutomaticGainControlSettings {
    fn default() -> Self {
        AutomaticGainControlSettings {
            target_level: 1.0,                    // Default to original level
            attack_time: Duration::from_secs(4),  // Recommended attack time
            release_time: Duration::from_secs(0), // Recommended release time
            absolute_max_gain: 7.0,               // Recommended max gain
        }
    }
}

/// A circular buffer for efficient RMS calculation over a sliding window.
///
/// This structure allows for constant-time updates and mean calculations,
/// which is crucial for real-time audio processing.
#[derive(Clone, Debug)]
struct CircularBuffer {
    buffer: Box<[Float; RMS_WINDOW_SIZE]>,
    sum: Float,
    index: usize,
}

impl CircularBuffer {
    /// Creates a new `CircularBuffer` with a fixed size determined at compile time.
    #[inline]
    fn new() -> Self {
        CircularBuffer {
            buffer: Box::new([0.0; RMS_WINDOW_SIZE]),
            sum: 0.0,
            index: 0,
        }
    }

    /// Pushes a new value into the buffer and returns the old value.
    ///
    /// This method maintains a running sum for efficient mean calculation.
    #[inline]
    fn push(&mut self, value: Float) -> Float {
        let old_value = self.buffer[self.index];
        // Update the sum by first subtracting the old value and then adding the new value; this is more accurate.
        self.sum = self.sum - old_value + value;
        self.buffer[self.index] = value;
        // Use bitwise AND for efficient index wrapping since RMS_WINDOW_SIZE is a power of two.
        self.index = (self.index + 1) & (RMS_WINDOW_SIZE - 1);
        old_value
    }

    /// Calculates the mean of all values in the buffer.
    ///
    /// This operation is `O(1)` due to the maintained running sum.
    #[inline]
    fn mean(&self) -> Float {
        self.sum / RMS_WINDOW_SIZE as Float
    }
}

// Note, because this depends on the sample rate to be constant this 
// is not dynamic safe.
pure_effect! {
    struct AutomaticGainControl {
        agc: Processor,
        is_enabled: bool,
    }

    fn next(&mut self) -> Option<Sample> {
        self.inner.next().map(|sample| {
            if self.is_enabled {
                self.agc.process_sample(sample)
            } else {
                sample
            }
        })
    }

    fn new(source: S, settings: AutomaticGainControlSettings) -> Amplify<Self> {
        Self {
            agc: Processor::from(settings, source.sample_rate()),
            inner: source,
            is_enabled: true,
        }
    }

    #[inline]
    pub fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    #[inline]
    pub fn set_enabled(&mut self, enabled: bool) {
        self.is_enabled = enabled;
    }

    #[inline]
    pub fn set_floor(&mut self, floor: Option<Float>) {
        self.agc.floor = floor.unwrap_or(0.0);
    }
}

pub(crate) struct Processor {
    target_level: Float,
    floor: Float,
    absolute_max_gain: Float,
    current_gain: Float,
    attack_coeff: Float,
    release_coeff: Float,
    peak_level: Float,
    rms_window: CircularBuffer,
}

impl Processor {
    fn from(settings: AutomaticGainControlSettings, sample_rate: SampleRate) -> Self {
        let AutomaticGainControlSettings {
            target_level,
            attack_time,
            release_time,
            absolute_max_gain,
        } = settings;

        let attack_coeff = duration_to_coefficient(attack_time, sample_rate);
        let release_coeff = duration_to_coefficient(release_time, sample_rate);

        Self {
            target_level,
            floor: 0.0,
            absolute_max_gain,
            current_gain: 1.0,
            attack_coeff,
            release_coeff,
            peak_level: 0.0,
            rms_window: CircularBuffer::new(),
        }
    }
}

impl Processor {
    #[inline]
    fn target_level(&self) -> Float {
        self.target_level
    }

    #[inline]
    fn absolute_max_gain(&self) -> Float {
        self.absolute_max_gain
    }

    #[inline]
    fn attack_coeff(&self) -> Float {
        self.attack_coeff
    }

    #[inline]
    fn release_coeff(&self) -> Float {
        self.release_coeff
    }

    /// Updates the peak level using instant attack and slow release behaviour
    ///
    /// This method uses instant response (0.0 coefficient) when the signal is increasing
    /// and the release coefficient when the signal is decreasing, providing
    /// appropriate tracking behaviour for peak detection.
    #[inline]
    fn update_peak_level(&mut self, sample_value: Sample, release_coeff: Float) {
        let coeff = if sample_value > self.peak_level {
            // Fast attack for rising peaks
            0.0
        } else {
            // Slow release for falling peaks
            release_coeff
        };

        self.peak_level = self.peak_level * coeff + sample_value * (1.0 - coeff);
    }

    /// Updates the RMS (Root Mean Square) level using a circular buffer approach.
    /// This method calculates a moving average of the squared input samples,
    /// providing a measure of the signal's average power over time.
    #[inline]
    fn update_rms(&mut self, sample_value: Sample) -> Float {
        let squared_sample = sample_value * sample_value;
        self.rms_window.push(squared_sample);
        self.rms_window.mean().sqrt()
    }

    /// Calculate gain adjustments based on peak levels
    /// This method determines the appropriate gain level to apply to the audio
    /// signal, considering the peak level.
    /// The peak level helps prevent sudden spikes in the output signal.
    #[inline]
    fn calculate_peak_gain(&self, target_level: Float, absolute_max_gain: Float) -> Float {
        if self.peak_level > 0.0 {
            (target_level / self.peak_level).min(absolute_max_gain)
        } else {
            absolute_max_gain
        }
    }

    #[inline]
    fn process_sample(&mut self, sample: crate::Sample) -> crate::Sample {
        // Cache atomic loads at the start - avoids repeated atomic operations
        let target_level = self.target_level();
        let absolute_max_gain = self.absolute_max_gain();
        let attack_coeff = self.attack_coeff();
        let release_coeff = self.release_coeff();

        // Convert the sample to its absolute float value for level calculations
        let sample_value = sample.abs();

        // Dynamically adjust peak level using cached release coefficient
        self.update_peak_level(sample_value, release_coeff);

        // Calculate the current RMS (Root Mean Square) level using a sliding window approach
        let rms = self.update_rms(sample_value);

        // Compute the gain adjustment required to reach the target level based on RMS
        let rms_gain = if rms > 0.0 {
            target_level / rms
        } else {
            absolute_max_gain // Default to max gain if RMS is zero
        };

        // Calculate the peak limiting gain
        let peak_gain = self.calculate_peak_gain(target_level, absolute_max_gain);

        // Use RMS for general adjustments, but limit by peak gain to prevent
        // clipping and apply a minimum floor value
        let desired_gain = rms_gain.min(peak_gain).max(self.floor);

        // Adaptive attack/release speed for AGC (Automatic Gain Control)
        //
        // This mechanism implements an asymmetric approach to gain adjustment:
        // 1. **Slow increase**: Prevents abrupt amplification of noise during
        //    quiet periods.
        // 2. **Fast decrease**: Rapidly attenuates sudden loud signals to avoid
        //    distortion.
        //
        // The asymmetry is crucial because: - Gradual gain increases sound more
        // natural and less noticeable to listeners. - Quick gain reductions are
        // necessary to prevent clipping and maintain audio quality.
        //
        // This approach addresses several challenges associated with high
        // attack times:
        // 1. **Slow response**: With a high attack time, the AGC responds very
        //    slowly to changes in input level. This means it takes longer for
        //    the gain to adjust to new signal levels.
        // 2. **Initial gain calculation**: When the audio starts or after a
        //    period of silence, the initial gain calculation might result in a
        //    very high gain value, especially if the input signal starts
        //    quietly.
        // 3. **Overshooting**: As the gain slowly increases (due to the high
        //    attack time), it might overshoot the desired level, causing the
        //    signal to become too loud.
        // 4. **Overcorrection**: The AGC then tries to correct this by reducing
        //    the gain, but due to the slow response, it might reduce the gain
        //    too much, causing the sound to drop to near-zero levels.
        // 5. **Slow recovery**: Again, due to the high attack time, it takes a
        //    while for the gain to increase back to the appropriate level.
        //
        // By using a faster release time for decreasing gain, we can mitigate
        // these issues and provide more responsive control over sudden level
        // increases while maintaining smooth gain increases.
        let attack_speed = if desired_gain > self.current_gain {
            attack_coeff
        } else {
            release_coeff
        };

        // Gradually adjust the current gain towards the desired gain for smooth transitions
        self.current_gain = self.current_gain * attack_speed + desired_gain * (1.0 - attack_speed);

        // Ensure the calculated gain stays within the defined operational range
        self.current_gain = self.current_gain.clamp(0.1, absolute_max_gain);

        // Output current gain value for developers to fine tune their inputs to
        // automatic_gain_control
        #[cfg(feature = "tracing")]
        tracing::debug!("AGC gain: {}", self.current_gain,);

        // Apply the computed gain to the input sample and return the result
        sample * self.current_gain
    }
}

