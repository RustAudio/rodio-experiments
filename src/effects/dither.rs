//! Dithering for audio quantization and requantization.
//!
//! Dithering is a technique in digital audio processing that eliminates quantization
//! artifacts during various stages of audio processing. This module provides tools for
//! adding appropriate dither noise to maintain audio quality during quantization
//! operations.
//!
//! ## Example
//!
//! ```rust
//! use rodio::source::{DitherAlgorithm, SineWave};
//! use rodio::{BitDepth, Source};
//!
//! let source = SineWave::new(440.0);
//! let dithered = source.dither(BitDepth::new(16).unwrap(), DitherAlgorithm::TPDF);
//! ```
//!
//! ## Guidelines
//!
//! - **Apply dithering before volume changes** for optimal results
//! - **Dither once** - Apply only at the final output stage to avoid noise accumulation
//! - **Choose TPDF** for most professional audio applications (it's the default)
//! - **Use target output bit depth** - Not the source bit depth!
//!
//! When you later change volume (e.g., with `Player::set_volume()`), both the signal
//! and dither noise scale together, maintaining proper dithering behavior.

use super::pure_effect;
use crate::{
    BitDepth, ChannelCount, Float, Sample,
    generators::noise::fixed_source::{Blue, WhiteGaussian, WhiteTriangular, WhiteUniform},
    nz,
};

pure_effect! {
    struct Dither {
        noise: NoiseGenerator,
        lsb_amplitude: Float,
        current_channel: usize,
    }

    fn next(&mut self) -> Option<Sample> {
        let input_sample = self.inner.next()?;

        let noise_sample = self
            .noise
            .next(self.current_channel)
            .expect("Noise generator should always produce samples");

        // Apply subtractive dithering at the target quantization level
        Some(input_sample - noise_sample * self.lsb_amplitude)
    }

    fn new(input: S, target_bits: BitDepth, algorithm: Algorithm) -> Dither<Self> {
        // LSB amplitude for signed audio: 1.0 / (2^(bits-1)) Using f64
        // intermediate prevents precision loss and u64 handles all bit depths
        // without overflow (64-bit being the theoretical maximum for audio
        // samples). Values stay well above f32 denormal threshold, avoiding
        // denormal arithmetic performance penalty.
        let lsb_amplitude = (1.0 / (1_u64 << (target_bits.get() - 1)) as f64) as Float;

        Self {
            noise: NoiseGenerator::new(algorithm, input.channels()),
            inner: input,
            current_channel: 0,
            lsb_amplitude,
        }
    }

    /// Change the dithering algorithm at runtime
    pub fn set_algorithm(&mut self, algorithm: Algorithm) {
        if self.noise.algorithm() != algorithm {
            self.noise =
                NoiseGenerator::new(algorithm, self.inner.channels());
        }
    }

    /// Get the current dithering algorithm
    pub fn algorithm(&self) -> Algorithm {
        self.noise.algorithm()
    }
}

/// Dither algorithm selection for runtime choice
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
    /// GPDF (Gaussian PDF) - normal/bell curve distribution.
    ///
    /// Uses Gaussian white noise which more closely mimics natural processes and
    /// analog circuits. Higher noise floor than TPDF.
    GPDF,

    /// High-pass dithering - reduces low-frequency artifacts.
    ///
    /// Uses blue noise (high-pass filtered white noise) to push dither energy
    /// toward higher frequencies. Particularly effective for reducing audible
    /// low-frequency modulation artifacts.
    HighPass,

    /// RPDF (Rectangular PDF) - uniform distribution.
    ///
    /// Uses uniform white noise for basic decorrelation. Simpler than TPDF but
    /// allows some correlation between signal and quantization error at low levels.
    /// Slightly lower noise floor than TPDF.
    RPDF,

    /// TPDF (Triangular PDF) - triangular distribution.
    ///
    /// The gold standard for audio dithering. Provides mathematically optimal
    /// decorrelation by completely eliminating correlation between the original
    /// signal and quantization error.
    #[default]
    TPDF,
}

#[derive(Clone, Debug)]
#[allow(clippy::upper_case_acronyms)]
pub(crate) enum NoiseGenerator {
    TPDF(WhiteTriangular),
    RPDF(WhiteUniform),
    GPDF(WhiteGaussian),
    HighPass(Vec<Blue>),
}

impl NoiseGenerator {
    fn new(algorithm: Algorithm, channels: ChannelCount) -> Self {
        match algorithm {
            Algorithm::TPDF => Self::TPDF(WhiteTriangular::new(nz!(1))),
            Algorithm::RPDF => Self::RPDF(WhiteUniform::new(nz!(1))),
            Algorithm::GPDF => Self::GPDF(WhiteGaussian::new(nz!(1))),
            Algorithm::HighPass => {
                // Create per-channel generators for HighPass to prevent
                // prev_white state from crossing channel boundaries in
                // interleaved audio. Each channel must have an independent RNG
                // to avoid correlation. Use this iterator instead of the `vec!`
                // macro to avoid cloning the RNG.
                Self::HighPass((0..channels.get()).map(|_| Blue::new(nz!(1))).collect())
            }
        }
    }

    #[inline]
    fn next(&mut self, channel: usize) -> Option<Sample> {
        match self {
            Self::TPDF(noise) => noise.next(),
            Self::RPDF(noise) => noise.next(),
            Self::GPDF(noise) => noise.next(),
            Self::HighPass(noise) => noise[channel].next(),
        }
    }

    #[inline]
    fn algorithm(&self) -> Algorithm {
        match self {
            Self::TPDF(_) => Algorithm::TPDF,
            Self::RPDF(_) => Algorithm::RPDF,
            Self::GPDF(_) => Algorithm::GPDF,
            Self::HighPass(_) => Algorithm::HighPass,
        }
    }
}
