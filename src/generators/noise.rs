//! Noise generators for audio synthesis and testing.
//!
//! ## Available Noise Types
//!
//! | **Noise Type** | **Best For** | **Sound Character** | **Technical Notes* |
//! |----------------|--------------|---------------------|--------------------|
//! | **White noise** | Testing equipment linearly, masking sounds | Harsh, static-like, evenly bright | RPDF (uniform), equal power all frequencies |
//! | **Gaussian white** | Scientific modeling, natural processes | Similar to white but more natural | GPDF (bell curve), better statistical properties |
//! | **Triangular white** | High-quality audio dithering | Similar to white noise | TPDF, eliminates quantization correlation |
//! | **Pink noise** | Speaker testing, calibration, background sounds | Warm, pleasant, like rainfall | 1/f spectrum, matches human hearing |
//! | **Blue noise** | High-passed dithering, reducing low-frequency artifacts | Bright but smoother than white | High-pass filtered white, less harsh |
//! | **Violet noise** | Testing high frequencies, harsh effects | Very bright, sharp, can be piercing | Heavy high-frequency emphasis |
//! | **Brownian noise** | Scientific modeling of Brownian motion | Very deep, muffled, lacks highs | True stochastic process, Gaussian increments |
//! | **Red noise** | Practical deep/muffled effects, distant rumbles | Very deep, muffled, lacks highs | 1/f² spectrum, uniform input |
//! | **Velvet noise** | Artificial reverb, room simulation | Sparse random impulses | Computationally efficient, decorrelated |
//!
//! Note: Previously the noise generators accepted an existing rng as argument.
//! A short survey showed this was not getting used a lot. Therefore we removed
//! it to lower code complexity. If you have a good use case for this please
//! open an issue and we can re-introduce it.
//!
//! ## Basic Usage
//!
//! ```rust
//! use std::num::NonZero;
//! use rodio::source::noise::{WhiteUniform, Pink, WhiteTriangular, Blue, Red};
//! use rodio::SampleRate;
//!
//! let sample_rate = NonZero::new(44100).unwrap();
//!
//! // For testing equipment linearly
//! let white = WhiteUniform::new(sample_rate);
//! // For pleasant background sound
//! let pink = Pink::new(sample_rate);
//! // For TPDF dithering
//! let triangular = WhiteTriangular::new(sample_rate);
//! // For high-passed dithering applications
//! let blue = Blue::new(sample_rate);
//! // For practical deep/muffled effects
//! let red = Red::new(sample_rate);
//! ```

use std::num::NonZero;

use rand::RngExt;
use rand::distr::Uniform;
use rand::rngs::SmallRng;
use rand_distr::{Normal, Triangular};

mod tests;

use crate::Float;
use crate::SampleRate;
use crate::math::PI;

export_noise! {WhiteUniform, WhiteTriangular, WhiteGaussian, Velvet, Pink, Blue, Violet, Brownian, Red}

impl_default! {WhiteUniform, WhiteTriangular, WhiteGaussian, Velvet, Pink, Blue, Violet, Brownian, Red}


impl_noise! {
    /// Generates an infinite stream of uniformly distributed white noise
    /// samples in [-1.0, 1.0]. White noise generator - sounds like radio static
    /// (RPDF).
    ///
    /// Generates uniformly distributed random samples with equal power at all
    /// frequencies. This is the most basic noise type and serves as a building
    /// block for other noise generators. Uses RPDF (Rectangular Probability
    /// Density Function) - uniform distribution.
    ///
    /// **When to use:** Audio equipment testing, sound masking, or as a base
    /// for other effects.
    ///
    /// **Sound:** Harsh, bright, evenly distributed across all frequencies.
    pub struct WhiteUniform {
        rng: SmallRng,
    }

    fn next(&mut self) -> Option<Sample> {
        // Uniform range should be created compile time
        Some(self.rng.sample(Uniform::new_inclusive(-1.0, 1.0).expect("params are correct")))
    }
    fn new<SR>(sample_rate) -> Self {
        let rng = rand::make_rng::<SmallRng>()
    }
}

impl_noise! {
    /// Triangular white noise generator - ideal for TPDF dithering.
    ///
    /// Generates triangular-distributed white noise by summing two uniform
    /// random samples. This creates TPDF (Triangular Probability Density
    /// Function) which is superior to RPDF for audio dithering because it
    /// completely eliminates correlation between the original signal and
    /// quantization error.
    ///
    /// **When to use:** High-quality audio dithering when reducing bit depth.
    ///
    /// **Sound:** Similar to white noise but with better statistical
    /// properties.
    ///
    /// **Distribution**: TPDF - triangular distribution from sum of two uniform
    /// samples.
    pub struct WhiteTriangular {
        rng: SmallRng,
    }

    fn next(&mut self) -> Option<Sample> {
        Some(self.rng.sample(Triangular::new(-1.0, 1.0, 0.0).expect("params are correct")))
    }

    fn new<SR>(sample_rate) -> Self {
        let rng = rand::make_rng::<SmallRng>()
    }
}

impl_noise! {
    /// Gaussian white noise generator - statistically perfect white noise
    /// (GPDF). Also known as normal noise or bell curve noise.
    ///
    /// Like regular white noise but with normal distribution (bell curve)
    /// instead of uniform. More closely mimics analog circuits and natural
    /// processes, which typically follow bell curves. Uses GPDF (Gaussian
    /// Probability Density Function) - 99.7% of samples within [-1.0, 1.0].
    ///
    /// **When to use:** Modeling analog circuits, natural random processes, or
    /// when you need more realistic noise that mimics how natural systems
    /// behave (most follow bell curves).
    ///
    /// **Sound character**: Very similar to regular white noise, but with more
    /// analog-like character.
    ///
    /// **vs White Noise:** Gaussian mimics natural/analog systems better,
    /// uniform white is faster and simpler.
    ///
    /// **Clipping Warning:** Can exceed [-1.0, 1.0] bounds. Consider
    /// attenuation or limiting if clipping is critical.
    pub struct WhiteGaussian {
        rng: SmallRng,
    }

    fn next(&mut self) -> Option<Sample> {
        Some(self.rng.sample(Normal::new(0.0, 0.6).expect("params are correct")))
    }

    fn new<SR>(sample_rate) -> Self {
        let rng = rand::make_rng::<SmallRng>()
    }
}

const PINK_NOISE_GENERATORS: usize = 16;
impl_noise! {
    /// Pink noise generator - sounds much more natural than white noise.
    ///
    /// Pink noise emphasizes lower frequencies, making it sound warmer and more
    /// pleasant than harsh white noise. Often described as sounding like gentle
    /// rainfall or wind. Uses the industry-standard Voss-McCartney algorithm
    /// with 16 generators.
    ///
    /// **When to use:** Audio testing (matches human hearing better), pleasant
    /// background sounds, speaker testing, or any time you want "natural"
    /// sounding noise.
    ///
    /// **Sound:** Warmer, more pleasant than white noise - like distant
    /// rainfall.
    ///
    /// **vs White Noise:** Pink sounds much more natural and less harsh to
    /// human ears.
    ///
    /// Technical: 1/f frequency spectrum (power decreases 3dB per octave).
    /// Works correctly at all sample rates from 8kHz to 192kHz+.
    pub struct Pink {
        white_noise: super::WhiteUniform::fixed_source::WhiteUniform,
        values: [crate::Sample; PINK_NOISE_GENERATORS],
        counters: [u32; PINK_NOISE_GENERATORS],
        max_counts: [u32; PINK_NOISE_GENERATORS],
    }

    fn next(&mut self) -> Option<Sample> {
        let mut sum = 0.0;

        // Update each generator when its counter reaches the update interval
        for i in 0..PINK_NOISE_GENERATORS {
            if self.counters[i] >= self.max_counts[i] {
                // Time to update this generator with a new white noise sample
                self.values[i] = self.white_noise.next().expect("noise never returns none");
                self.counters[i] = 0;
            }
            sum += self.values[i];
            self.counters[i] += 1;
        }

        // Normalize by number of generators to keep output in reasonable range
        Some(sum / PINK_NOISE_GENERATORS as crate::Sample)
    }

    fn new<SR>(sample_rate) -> Self {
        use super::WhiteUniform::fixed_source::WhiteUniform;
        let mut max_counts = [1u32; PINK_NOISE_GENERATORS];
        // Each generator updates at half the rate of the previous one: 1, 2, 4, 8, 16, ...
        for i in 1..PINK_NOISE_GENERATORS {
            max_counts[i] = max_counts[i - 1] * 2;
        }
        let white_noise = WhiteUniform::new(sample_rate);
        let values = [0.0; PINK_NOISE_GENERATORS];
        let counters = [0; PINK_NOISE_GENERATORS];
    }
}

impl_noise! {
    /// Velvet noise generator - creates sparse random impulses, not continuous
    /// noise. Also known as sparse noise or decorrelated noise.
    ///
    /// Unlike other noise types, velvet noise produces random impulses separated by
    /// periods of silence. Divides time into regular intervals and places one
    /// impulse randomly within each interval.
    ///
    /// **When to use:** Building reverb effects, room simulation, decorrelating
    /// audio channels.
    ///
    /// **Sound:** Random impulses with silence between - smoother than continuous
    /// noise.
    ///
    /// **Default:** 2000 impulses per second.
    ///
    /// **Efficiency:** Very computationally efficient - mostly outputs zeros, only
    /// occasional computation.
    pub struct Velvet {
        rng: SmallRng,
        grid_size: usize,
        grid_pos: usize,
        impulse_pos: usize,
    }

    fn next(&mut self) -> Option<Sample> {
        let output = if self.grid_pos == self.impulse_pos {
            // Generate impulse with random polarity
            if self.rng.random::<bool>() {
                1.0
            } else {
                -1.0
            }
        } else {
            0.0
        };

        self.grid_pos = self.grid_pos.wrapping_add(1);

        // Start new grid cell when we reach the end
        if self.grid_pos >= self.grid_size {
            self.grid_pos = 0;
            self.impulse_pos = if self.grid_size > 0 {
                self.rng.random_range(0..self.grid_size)
            } else {
                0
            };
        }

        Some(output)
    }

    fn new<SR>(sample_rate) -> Self {
        const VELVET_DEFAULT_DENSITY: std::num::NonZero<usize> = crate::nz!(2000);

        let density = VELVET_DEFAULT_DENSITY;
        let mut rng = rand::make_rng::<SmallRng>();
        let grid_pos = 0;
        let grid_size = (sample_rate.get() as f32 / density.get() as f32).ceil() as usize;
        let impulse_pos = if grid_size > 0 {
            rng.random_range(0..grid_size)
        } else {
            0
        };
    }
}

// Velvet noise has two extra factories that can not be easily added
// to the macro. So we add them manually here.
impl<const SR: u32> const_source::Velvet<SR> {
    /// Create a new velvet noise generator with custom density (impulses per second) and RNG.
    ///
    /// **Density guidelines:**
    /// - 500-1000 Hz: Sparse, distant reverb effects
    /// - 1000-2000 Hz: Balanced reverb simulation (default: 2000 Hz)
    /// - 2000-4000 Hz: Dense, close reverb effects
    /// - >4000 Hz: Very dense, approaching continuous noise
    pub fn new_with_density(density: NonZero<usize>) -> Self {
        let mut rng = rand::make_rng::<SmallRng>();
        let grid_size = (SR as f32 / density.get() as f32).ceil() as usize;
        let impulse_pos = if grid_size > 0 {
            rng.random_range(0..grid_size)
        } else {
            0
        };
        Self {
            rng,
            grid_size,
            grid_pos: 0,
            impulse_pos,
        }
    }
}

impl fixed_source::Velvet {
    /// Create a new velvet noise generator with custom density (impulses per second) and RNG.
    ///
    /// **Density guidelines:**
    /// - 500-1000 Hz: Sparse, distant reverb effects
    /// - 1000-2000 Hz: Balanced reverb simulation (default: 2000 Hz)
    /// - 2000-4000 Hz: Dense, close reverb effects
    /// - >4000 Hz: Very dense, approaching continuous noise
    pub fn new_with_density(sample_rate: SampleRate, density: NonZero<usize>) -> Self {
        let mut rng = rand::make_rng::<SmallRng>();
        let grid_size = (sample_rate.get() as f32 / density.get() as f32).ceil() as usize;
        let impulse_pos = if grid_size > 0 {
            rng.random_range(0..grid_size)
        } else {
            0
        };
        Self {
            rng,
            grid_size,
            grid_pos: 0,
            impulse_pos,
            sample_rate,
        }
    }
}

impl_noise! {
    /// Blue noise generator - sounds brighter than white noise but smoother. Also
    /// known as azure noise.
    ///
    /// Blue noise emphasizes higher frequencies while distributing energy more
    /// evenly than white noise. It's "brighter" sounding but less harsh and
    /// fatiguing. Generated by differentiating pink noise.
    ///
    /// **When to use:** High-passed audio dithering (preferred over violet),
    /// digital signal processing, or when you want bright sound without the
    /// harshness of white noise.
    ///
    /// **Sound:** Brighter than white noise but smoother and less fatiguing.
    ///
    /// **vs White Noise:** Blue has better frequency distribution and less
    /// clustering.
    ///
    /// **vs Violet Noise:** Blue is better for dithering - violet pushes too much
    /// energy to very high frequencies.
    ///
    /// **Clipping Warning:** Can exceed [-1.0, 1.0] bounds due to differentiation.
    /// Consider attenuation or limiting if clipping is critical.
    ///
    /// Technical: f frequency spectrum (power increases 3dB per octave).
    pub struct Blue {
        white_noise: super::WhiteUniform::fixed_source::WhiteUniform,
        prev_white: crate::Sample,
    }

    fn next(&mut self) -> Option<Sample> {
        let white = self.white_noise.next().expect("noise is never None");
        let blue = white - self.prev_white;
        self.prev_white = white;
        Some(blue)
    }

    fn new<SR>(sample_rate) -> Self {
        use super::WhiteUniform::fixed_source::WhiteUniform;
        let white_noise = WhiteUniform::new(sample_rate);
        let prev_white = 0.0;
    }
}

impl_noise! {
    /// Violet noise generator - very bright and sharp sounding. Also known as
    /// purple noise.
    ///
    /// Violet noise (also called purple noise) heavily emphasizes high frequencies,
    /// creating a very bright, sharp, sometimes harsh sound. It's the opposite of
    /// brownian noise in terms of frequency emphasis.
    ///
    /// **When to use:** Testing high-frequency equipment response, creating
    /// bright/sharp sound effects, or when you need to emphasize treble
    /// frequencies.
    ///
    /// **Sound:** Very bright, sharp, can be harsh - use sparingly in audio
    /// applications.
    ///
    /// **vs Blue Noise:** Violet is much brighter and more aggressive than blue
    /// noise.
    ///
    /// **Not ideal for dithering:** Too much energy at very high frequencies can
    /// cause aliasing.
    ///
    /// **Clipping Warning:** Can exceed [-1.0, 1.0] bounds due to differentiation.
    /// Consider attenuation or limiting if clipping is critical.
    ///
    /// Technical: f² frequency spectrum (power increases 6dB per octave). Generated
    /// by differentiating uniform random samples.
    pub struct Violet {
        blue_noise: super::Blue::fixed_source::Blue,
        prev: crate::Sample,
    }

    fn next(&mut self) -> Option<Sample> {
        let blue = self.blue_noise.next().expect("noise is never None");
        let violet = blue - self.prev;
        self.prev = blue;
        Some(violet)
    }

    fn new<SR>(sample_rate) -> Self {
        use super::Blue::fixed_source::Blue;
        let blue_noise = Blue::new(sample_rate);
        let prev = 0.0;
    }
}

impl_noise! {
    /// Common leaky integration algorithm used by other noises
    pub(crate) struct IntegratedNoise<W: Iterator<Item = crate::Sample>> {
        white_noise: W,
        accumulator: crate::Sample,
        leak_factor: crate::Float,
        scale: crate::Float,
    }

    fn next(&mut self) -> Option<Sample> {
        let white = self.white_noise.next()?;
        self.accumulator = self.accumulator * self.leak_factor + white;
        Some(self.accumulator * self.scale)
    }

    #[allow(unused)]
    fn new<SR>(sample_rate white_noise: W, white_noise_stddev: Float) -> Self {
        let center_freq_hz = 5.0;
        let leak_factor = 1.0 - ((2.0 * PI * center_freq_hz) / sample_rate.get() as Float);
        let variance =
            (white_noise_stddev * white_noise_stddev) / (1.0 - leak_factor * leak_factor);
        let scale = 1.0 / variance.sqrt();
        let accumulator = 0.0;
    }
}

const UNIFORM_VARIANCE: crate::Sample = 1.0 / 3.0;
impl_noise! {
    /// Brownian noise generator - true stochastic Brownian motion with Gaussian
    /// increments.
    ///
    /// Brownian noise is the mathematically precise implementation of Brownian
    /// motion using Gaussian white noise increments. This creates the
    /// theoretically correct stochastic process with proper statistical
    /// properties. Generated by integrating Gaussian white noise with a 5Hz
    /// center frequency leak factor to prevent DC buildup.
    ///
    /// **When to use:** Scientific modeling, research applications, or when
    /// mathematical precision of Brownian motion is required.
    ///
    /// **Sound:** Very muffled, deep, lacks high frequencies - sounds
    /// "distant".
    ///
    /// **vs Red Noise:** Brownian noise is a specific stochastic process with
    /// Gaussian properties. For general 1/f² spectrum without Gaussian
    /// requirements, use Red noise instead.
    ///
    /// **Technical:** Uses Gaussian white noise as mathematically required for
    /// true Brownian motion.
    ///
    /// **Clipping Warning:** Can exceed [-1.0, 1.0] bounds due to integration.
    /// Consider attenuation or limiting if clipping is critical.
    pub struct Brownian {
        inner: super::IntegratedNoise::fixed_source::IntegratedNoise<
            super::WhiteGaussian::fixed_source::WhiteGaussian>,
    }

    fn next(&mut self) -> Option<Sample> {
        self.inner.next()
    }

    fn new<SR>(sample_rate) -> Self {
        use super::WhiteGaussian::fixed_source::WhiteGaussian;
        use super::IntegratedNoise::fixed_source::IntegratedNoise;

        let white_noise = WhiteGaussian::new(sample_rate);
        let std_dev = UNIFORM_VARIANCE.sqrt();
        let inner = IntegratedNoise::new(sample_rate, white_noise, std_dev)
    }
}

impl_noise! {
    /// Red noise generator - practical 1/f² spectrum with bounded output.
    ///
    /// Red noise provides the same 1/f² power spectral density as Brownian noise
    /// but uses uniform white noise input for better practical behavior. This
    /// avoids the clipping issues of Gaussian input while maintaining the
    /// characteristic deep, muffled sound with heavy low-frequency emphasis.
    ///
    /// **When to use:** Audio applications where you want Brownian-like sound
    /// characteristics but need predictable bounded output, background rumbles, or
    /// muffled distant effects.
    ///
    /// **Sound:** Very muffled, deep, lacks high frequencies - sounds "distant",
    /// similar to Brownian.
    ///
    /// **vs Brownian Noise:** Red noise uses uniform input (less clipping) while
    /// Brownian noise uses Gaussian input (more clipping). Both have 1/f² spectrum
    /// and can exceed bounds.
    ///
    /// **Technical:** Uses uniform white noise input with variance-adjusted scaling
    /// for proper normalization.
    ///
    /// **Clipping Warning:** Can exceed [-1.0, 1.0] bounds due to integration,
    /// though less frequently than Brownian noise due to uniform input. Consider
    /// attenuation or limiting if clipping is critical.
    pub struct Red {
        inner: super::IntegratedNoise::fixed_source::IntegratedNoise<
            super::WhiteUniform::fixed_source::WhiteUniform>,
    }

    fn next(&mut self) -> Option<Sample> {
        self.inner.next()
    }

    fn new<SR>(sample_rate) -> Self {
        use super::WhiteUniform::fixed_source::WhiteUniform;
        use super::IntegratedNoise::fixed_source::IntegratedNoise;

        let white_noise = WhiteUniform::new(sample_rate);
        let std_dev = Normal::new(0.0, 0.6).expect("Correct params").std_dev();
        let inner = IntegratedNoise::new(sample_rate, white_noise, std_dev)
    }
}


macro_rules! impl_noise {
    (
    $(#[$doc:meta])+
    $vis:vis struct $name:ident$(<$t:ident: $bound:path>)? {
        $(= integrated: IntegratedNoise<$inner_noise:ident>,)?
        $($field:ident: $field_ty:ty,)*
    }
    // like `struct` above the `fn`, `&mut` and `-> Option<Sample>` are just there
    // to make the macro input seem regular rust code
    fn next(&mut $self:ident) -> Option<Sample> $body:block
    $(#[$new_attr:meta])?
    fn new<$SR:ident>($sample_rate:ident $($factory_args:tt)*)
    -> Self {
        $($factory_body:stmt)*
    }
    ) =>  {
    #[allow(non_snake_case)]
    pub(crate) mod $name {
        use super::*;
        pub(crate) mod fixed_source {
            #[allow(unused)]
            use super::*;

            #[derive(Clone, Debug, PartialEq)]
            $vis struct $name<$($t:$bound)?> {
                pub(crate) sample_rate: crate::SampleRate,
                $(pub(crate) $field: $field_ty),*
            }
        }

        impl<$($t:$bound)?> crate::FixedSource for fixed_source::$name$(<$t>)? {
            fn channels(&self) -> crate::ChannelCount {
                crate::nz!(1)
            }

            fn sample_rate(&self) -> crate::SampleRate {
                self.sample_rate
            }

            fn total_duration(&self) -> Option<std::time::Duration> {
                None
            }
        }

        impl<$($t:$bound)?> fixed_source::$name$(<$t>)? {
            #[must_use]
            // statements are transcribed with a semicolon but captured with one
            // too. We could leave them out in the input but that looks off so allow
            // unnecessary trailing semicolons.
            #[allow(redundant_semicolons)]
            $(#[$new_attr])?
            $vis fn new($sample_rate: crate::SampleRate, $($factory_args)*) -> Self {
                $($factory_body)*

                Self {
                    $sample_rate,
                    $($field),*
                }
            }
        }

        impl<$($t:$bound)?> Iterator for fixed_source::$name$(<$t>)? {
            type Item = crate::Sample;

            fn next(&mut $self) -> Option<Self::Item> {
                $body
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                (usize::MAX, None)
            }
        }

        pub(crate) mod const_source {
            #[allow(unused)]
            use super::*;

            #[derive(Clone, Debug, PartialEq)]
            pub struct $name<const SR: u32, $($t:$bound)?> {
                $(pub(crate) $field: $field_ty),*
            }
        }

        impl<const SR: u32, $($t:$bound)?> crate::ConstSource<SR, 1> for const_source::$name<SR, $($t)?> {
            fn total_duration(&self) -> Option<std::time::Duration> {
                None
            }
        }

        impl<const SR: u32, $($t:$bound)?> const_source::$name<SR, $($t)?> {
            #[must_use]
            // statements are transcribed with a semicolon but captured with one
            // too. We could leave them out in the input but that looks off so allow
            // unnecessary trailing semicolons.
            #[allow(redundant_semicolons)]
            $(#[$new_attr])?
            $vis fn new($($factory_args)*) -> Self {
                const { assert!(SR > 0) };
                #[allow(unused)]
                let $sample_rate = core::num::NonZero::<u32>::new(SR).expect("assert above");
                $($factory_body)*

                Self {
                    $($field),*
                }
            }
        }

        impl<const SR: u32, $($t:$bound)?> Iterator for const_source::$name<SR, $($t)?> {
            type Item = crate::Sample;

            fn next(&mut $self) -> Option<Self::Item> {
                $body
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                (usize::MAX, None)
            }
        }
    }
} // end transcriber
} // end macro

pub(crate) use impl_noise;


macro_rules! export_noise {
    ($($name:ident),*) => {
        pub mod fixed_source {
            $(pub use super::$name::fixed_source::$name;)*
        }

        pub mod const_source {
            $(pub use super::$name::const_source::$name;)*
        }
    };
}
pub(crate) use export_noise;


macro_rules! impl_default {
    ($($name:ident),+) => {
        $(impl<const SR: u32> Default for const_source::$name<SR> {
            fn default() -> Self {
               Self::new()
            }
        })+
    } //end transcriber
}

pub(crate) use impl_default; 
