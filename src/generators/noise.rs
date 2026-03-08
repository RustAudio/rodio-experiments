// FIXME(yara) these need to be documented (copied over) and comments need to be
// supported in the macro and copied over

use rand::RngExt;
use rand::distr::Uniform;
use rand::rngs::SmallRng;
use rand_distr::{Normal, Triangular};

mod tests;

use crate::Float;
use crate::math::PI;

export_noise! {WhiteUniform, WhiteTriangular, WhiteGaussian, Velvet, Pink, Blue, Violet, Brownian, Red}

impl_noise! {
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

impl_noise! {
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

macro_rules! impl_noise {
    (
    $vis:vis struct $name:ident$(<$t:ident: $bound:path>)? {
        $(= integrated: IntegratedNoise<$inner_noise:ident>,)?
        $($field:ident: $field_ty:ty,)*
    }
    // like `struct` above the `fn`, `&mut` and `-> Option<Sample>` are just there
    // to make the macro input seem regular rust code
    fn next(&mut $self:ident) -> Option<Sample> $body:block
    $([use $needed:ident]),*
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
                $(use super::$needed::fixed_source::$needed;)*

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
        }

        pub(crate) mod const_source {
            #[allow(unused)]
            use super::*;

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
                $(use super::$needed::const_source::$needed;)*

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
        }
    }
} // end transcriber
} // end macro

pub(crate) use impl_noise;
