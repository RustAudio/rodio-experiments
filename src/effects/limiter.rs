//! Audio peak limiting for dynamic range control.
//!
//! This module implements a feedforward limiter that prevents audio peaks from exceeding
//! a specified threshold while maintaining audio quality. The limiter is based on:
//! Giannoulis, D., Massberg, M., & Reiss, J.D. (2012). Digital Dynamic Range Compressor Design,
//! A Tutorial and Analysis. Journal of The Audio Engineering Society, 60, 399-408.
//!
//! # What is Limiting?
//!
//! A limiter reduces the amplitude of audio signals that exceed a threshold level.
//! For example, with a -6dB threshold, peaks above that level are reduced to stay near the
//! threshold, preventing clipping and maintaining consistent output levels.
//!
//! # Features
//!
//! * **Soft-knee limiting** - Gradual transition into limiting for natural sound
//! * **Per-channel detection** - Decoupled peak detection per channel
//! * **Coupled gain reduction** - Uniform gain reduction across channels preserves stereo imaging
//! * **Configurable timing** - Adjustable attack/release times for different use cases
//! * **Efficient processing** - Optimized implementations for mono, stereo, and multi-channel audio
//!
//! # Usage
//!
//! Use [`LimitSettings`] to configure the limiter, then apply it to any audio source:
//!
//! ```rust
//! use rodio::source::{SineWave, Source, LimitSettings};
//! use std::time::Duration;
//!
//! // Create a loud sine wave
//! let source = SineWave::new(440.0).amplify(2.0);
//!
//! // Apply limiting with -6dB threshold
//! let settings = LimitSettings::default().with_threshold(-6.0);
//! let limited = source.limit(settings);
//! ```
//!
//! # Presets
//!
//! [`LimitSettings`] provides optimized presets for common use cases:
//!
//! * [`LimitSettings::default()`] - General-purpose limiting (-1 dBFS, balanced)
//! * [`LimitSettings::dynamic_content()`] - Music and sound effects (-3 dBFS, transparent)
//! * [`LimitSettings::broadcast()`] - Streaming and voice chat (fast response, consistent)
//! * [`LimitSettings::mastering()`] - Final production stage (-0.5 dBFS, tight peak control)
//! * [`LimitSettings::gaming()`] - Interactive audio (-3 dBFS, responsive dynamics)
//! * [`LimitSettings::live_performance()`] - Real-time applications (ultra-fast protection)
//!
//! ```rust
//! use rodio::source::{SineWave, Source, LimitSettings};
//!
//! // Use preset optimized for music
//! let music = SineWave::new(440.0).amplify(1.5);
//! let limited_music = music.limit(LimitSettings::dynamic_content());
//!
//! // Use preset optimized for streaming
//! let stream = SineWave::new(440.0).amplify(2.0);
//! let limited_stream = stream.limit(LimitSettings::broadcast());
//! ```

use crate::effects::pure_effect;
use crate::{
    Float,
    common::Sample,
    math::{self, duration_to_coefficient},
};

mod settings;
pub use settings::LimitSettings;

type Db = Float;

pure_effect! {
    struct Limit {
        implementation: LimitImpl,
    }
    fn next(&mut self)->Option<Sample>{
        let sample = self.inner.next()?;
        let sample = self.implementation.process_next(sample);
        Some(sample)
    }
    fn new(input: S, settings: LimitSettings)-> Limit<Self> {
        let sample_rate = input.sample_rate();
        let attack = duration_to_coefficient(settings.attack, sample_rate);
        let release = duration_to_coefficient(settings.release, sample_rate);
        let channels = input.channels().get() as usize;

        let base = LimitBase::new(settings.threshold, settings.knee_width, attack, release);

        let implementation = match channels {
            1 => LimitImpl::Mono(LimitMono {
                base,
                limiter_integrator: 0.0,
                limiter_peak: 0.0,
            }),
            2 => LimitImpl::Stereo(LimitStereo {
                base,
                limiter_integrators: [0.0; 2],
                limiter_peaks: [0.0; 2],
                curr_channel: 0,
            }),
            n => LimitImpl::MultiChannel(LimitMulti {
                base,
                limiter_integrators: vec![0.0; n],
                limiter_peaks: vec![0.0; n],
                curr_channel: 0,
            }),
        };

        Self {
            inner: input,
            implementation,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum LimitImpl {
    Mono(LimitMono),
    Stereo(LimitStereo),
    MultiChannel(LimitMulti),
}

impl LimitImpl {
    fn process_next(&mut self, sample: crate::Sample) -> crate::Sample {
        match self {
            LimitImpl::Mono(mono) => mono.process_next(sample),
            LimitImpl::Stereo(stereo) => stereo.process_next(sample),
            LimitImpl::MultiChannel(multi) => multi.process_next(sample),
        }
    }
}

/// Common parameters and processing logic shared across all limiter variants.
///
/// Handles:
/// * Parameter storage (threshold, knee width, attack/release coefficients)
/// * Per-channel state updates for peak detection
/// * Gain computation through soft-knee limiting
#[derive(Clone, Debug)]
struct LimitBase {
    /// Level where limiting begins (dB)
    threshold: Float,
    /// Width of the soft-knee region (dB)
    knee_width: Float,
    /// Inverse of 8 times the knee width (precomputed for efficiency)
    inv_knee_8: Float,
    /// Attack time constant (ms)
    attack: Float,
    /// Release time constant (ms)
    release: Float,
}

/// Mono channel limiter optimized for single-channel processing.
#[derive(Clone, Debug)]
pub(crate) struct LimitMono {
    base: LimitBase,
    /// Peak detection integrator state
    limiter_integrator: Db,
    /// Peak detection state
    limiter_peak: Db,
}

/// Stereo channel limiter with optimized two-channel processing.
//
// # Performance
// The fixed arrays and channel position tracking provide optimal performance
// for interleaved stereo sample processing, avoiding the dynamic allocation
// overhead of the multi-channel variant.
#[derive(Clone, Debug)]
pub(crate) struct LimitStereo {
    base: LimitBase,
    /// Peak detection integrator states for left and right channels
    limiter_integrators: [Db; 2],
    /// Peak detection states for left and right channels
    limiter_peaks: [Db; 2],
    curr_channel: usize,
}

/// Generic multi-channel limiter for surround sound or other configurations.
///
/// # Performance
/// While this variant has slightly more overhead than the mono/stereo variants
/// due to vector allocation and dynamic indexing, it provides the flexibility
/// needed for complex audio setups while maintaining good performance.
#[derive(Clone, Debug)]
pub(crate) struct LimitMulti {
    base: LimitBase,
    /// Peak detector integrator states (one per channel)
    limiter_integrators: Vec<Db>,
    /// Peak detector states (one per channel)
    limiter_peaks: Vec<Db>,
    curr_channel: usize,
}

/// Computes the gain reduction amount in dB based on input level.
///
/// Implements soft-knee compression with three regions:
/// 1. Below threshold - knee_width: No compression (returns 0.0)
/// 2. Within knee region: Gradual compression with quadratic curve
/// 3. Above threshold + knee_width: Linear compression
///
/// Optimized for the most common case where samples are below threshold and no
/// limiting is needed
#[inline]
fn process_sample(
    sample: Sample,
    threshold: Db,     // dB
    knee_width: Db,    // dB
    inv_knee_8: Float, // precomputed value
) -> Sample {
    // Add slight DC offset. Some samples are silence, which is -inf dB and gets the limiter stuck.
    // Adding a small positive offset prevents this.
    let bias_db = math::linear_to_db(sample.abs() + Sample::MIN_POSITIVE) - threshold;
    let knee_boundary_db = bias_db * 2.0;
    if knee_boundary_db < -knee_width {
        0.0
    } else if knee_boundary_db.abs() <= knee_width {
        // Faster than powi(2)
        let x = knee_boundary_db + knee_width;
        x * x * inv_knee_8
    } else {
        bias_db
    }
}

impl LimitBase {
    fn new(threshold: Db, knee_width: Db, attack: Float, release: Float) -> Self {
        let inv_knee_8 = 1.0 / (8.0 * knee_width);
        Self {
            threshold,
            knee_width,
            inv_knee_8,
            attack,
            release,
        }
    }

    /// Updates the channel's envelope detection state.
    ///
    /// For each channel, processes:
    /// 1. Initial gain and dB conversion
    /// 2. Soft-knee limiting calculation
    /// 3. Envelope detection with attack/release filtering
    /// 4. Peak level tracking
    ///
    /// The envelope detection uses a dual-stage approach:
    /// - First stage: Max of current signal and smoothed release
    /// - Second stage: Attack smoothing of the peak detector output
    ///
    /// Note: Only updates state, gain application is handled by the variant implementations to
    /// allow for coupled gain reduction across channels.
    #[must_use]
    #[inline]
    fn process_channel(&self, sample: Sample, integrator: &mut Float, peak: &mut Float) -> Sample {
        // step 1-4: half-wave rectification and conversion into dB, and gain computer with soft
        // knee and subtractor
        let limiter_db = process_sample(sample, self.threshold, self.knee_width, self.inv_knee_8);

        // step 5: smooth, decoupled peak detector
        *integrator = Float::max(
            limiter_db,
            self.release * *integrator + (1.0 - self.release) * limiter_db,
        );
        *peak = self.attack * *peak + (1.0 - self.attack) * *integrator;

        sample
    }
}

impl LimitMono {
    #[inline]
    fn process_next(&mut self, sample: crate::Sample) -> crate::Sample {
        let processed =
            self.base
                .process_channel(sample, &mut self.limiter_integrator, &mut self.limiter_peak);

        // steps 6-8: conversion into level and multiplication into gain stage
        processed * math::db_to_linear(-self.limiter_peak)
    }
}

impl LimitStereo {
    #[inline]
    fn process_next(&mut self, sample: crate::Sample) -> crate::Sample {
        let processed = self.base.process_channel(
            sample,
            &mut self.limiter_integrators[self.curr_channel],
            &mut self.limiter_peaks[self.curr_channel],
        );
        self.curr_channel ^= 1;

        // steps 6-8: conversion into level and multiplication into gain stage. Find maximum peak
        // across both channels to couple the gain and maintain stereo imaging.
        let max_peak = Float::max(self.limiter_peaks[0], self.limiter_peaks[1]);
        processed * math::db_to_linear(-max_peak)
    }
}

impl LimitMulti {
    #[inline]
    fn process_next(&mut self, sample: crate::Sample) -> crate::Sample {
        let processed = self.base.process_channel(
            sample,
            &mut self.limiter_integrators[self.curr_channel],
            &mut self.limiter_peaks[self.curr_channel],
        );
        self.curr_channel = (self.curr_channel + 1) % self.limiter_integrators.len();

        // steps 6-8: conversion into level and multiplication into gain stage. Find maximum peak
        // across all channels to couple the gain and maintain multi-channel imaging.
        let max_peak = self
            .limiter_peaks
            .iter()
            .fold(0.0, |max, &peak| Float::max(max, peak));
        processed * math::db_to_linear(-max_peak)
    }
}
