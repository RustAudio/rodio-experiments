use std::num::NonZeroU16;
use std::num::NonZeroU32;
use std::time::Duration;

use crate::ChannelCount;
use crate::FixedSource;
use crate::Float;
use crate::Sample;
use crate::SampleRate;
use crate::Source as DynamicSource; // will be renamed to this upstream
use crate::effects::const_source::FadeIn;
use crate::effects::const_source::FadeOut;
use crate::effects::const_source::LinearGainRamp;

// pub mod adapter; replaced with into_fixed_source and into_const_source
pub mod buffer;
pub mod conversions;
pub mod list;
pub mod mixer;
pub mod queue;

use crate::const_source::buffer::SamplesBuffer;
use crate::const_source::conversions::channelcount::ChannelConvertor;
use crate::effects::amplify::Factor;
use crate::effects::automatic_gain_control::AutomaticGainControlSettings;
use crate::effects::const_source::Limit;
use crate::effects::blt::BltFormula;
use crate::effects::const_source::BltFilter;
use crate::effects::const_source::{
    Amplify, AutomaticGainControl, InspectFrame, Pausable, PeriodicAccess, Stoppable, TakeDuration,
    TakeSamples, WithData,
};
use crate::effects::limiter::LimitSettings;

pub trait ConstSource<const SR: u32, const CH: u16>: Iterator<Item = Sample> {
    fn sample_rate(&self) -> SampleRate {
        const { NonZeroU32::new(SR).expect("SampleRate must be > 0") }
    }
    fn channels(&self) -> ChannelCount {
        const { NonZeroU16::new(CH).expect("Channel count must be > 0") }
    }

    /// This value is free to change at any time
    fn total_duration(&self) -> Option<Duration>;

    fn into_dynamic_source(self) -> ConstSourceAdapter<SR, CH, Self>
    where
        Self: Sized,
    {
        ConstSourceAdapter { inner: self }
    }
    fn into_fixed_source(self) -> ConstSourceAdapter<SR, CH, Self>
    where
        Self: Sized,
    {
        ConstSourceAdapter { inner: self }
    }

    fn with_channel_count<const CH_OUT: u16>(self) -> ChannelConvertor<SR, CH, CH_OUT, Self>
    where
        Self: Sized,
    {
        ChannelConvertor::new(self)
    }

    fn take_duration(self, duration: Duration) -> TakeDuration<SR, CH, Self>
    where
        Self: Sized,
    {
        TakeDuration::new(self, duration)
    }

    fn take_samples(self, samples: usize) -> TakeSamples<SR, CH, Self>
    where
        Self: Sized,
    {
        TakeSamples::new(self, samples)
    }

    fn periodic_access(
        self,
        call_every: Duration,
        arg: fn(&mut Self),
    ) -> PeriodicAccess<SR, CH, Self>
    where
        Self: Sized,
    {
        PeriodicAccess::new(self, call_every, arg)
    }

    fn with_data<D>(self, data: D) -> WithData<SR, CH, Self, D>
    where
        Self: Sized,
    {
        WithData::new(self, data)
    }

    fn amplify(self, factor: Factor) -> Amplify<SR, CH, Self>
    where
        Self: Sized,
    {
        Amplify::new(self, factor)
    }

    fn stoppable(self) -> Stoppable<SR, CH, Self>
    where
        Self: Sized,
    {
        Stoppable::new(self)
    }

    fn pausable(self, paused: bool) -> Pausable<SR, CH, Self>
    where
        Self: Sized,
    {
        Pausable::new(self, paused)
    }

    fn inspect_frame<F: FnMut(Vec<Sample>) -> Vec<Sample>>(
        self,
        f: F,
    ) -> InspectFrame<SR, CH, Self, F>
    where
        Self: Sized,
    {
        InspectFrame::new(self, f)
    }

    fn collect_into_buffer(self) -> SamplesBuffer<SR, CH>
    where
        Self: Sized,
    {
        SamplesBuffer::new(self.collect::<Vec<_>>())
    }

    fn automatic_gain_control(
        self,
        settings: AutomaticGainControlSettings,
    ) -> AutomaticGainControl<SR, CH, Self>
    where
        Self: Sized,
    {
        AutomaticGainControl::new(self, settings)
    }

    fn limit(self, settings: LimitSettings) -> Limit<SR, CH, Self>
    where
        Self: Sized,
    {
        Limit::new(self, settings)
    }

    fn low_pass(self, freq: u32) -> BltFilter<SR, CH, Self>
    where
        Self: Sized,
    {
        BltFilter::new(self, BltFormula::LowPass { freq, q: 0.5 })
    }

    fn high_pass(self, freq: u32) -> BltFilter<SR, CH, Self>
    where
        Self: Sized,
    {
        BltFilter::new(self, BltFormula::HighPass { freq, q: 0.5 })
    }

    fn low_pass_with_q(self, freq: u32, q: Float) -> BltFilter<SR, CH, Self>
    where
        Self: Sized,
    {
        BltFilter::new(self, BltFormula::LowPass { freq, q })
    }

    fn high_pass_with_q(self, freq: u32, q: Float) -> BltFilter<SR, CH, Self>
    where
        Self: Sized,
    {
        BltFilter::new(self, BltFormula::HighPass { freq, q })
    }
    /// Fades in the sound.
    fn fade_in(self, duration: Duration) -> FadeIn<SR, CH, Self>
    where
        Self: Sized,
    {
        FadeIn::new(self, duration)
    }

    /// Fades out the sound.
    fn fade_out(self, duration: Duration) -> FadeOut<SR, CH, Self>
    where
        Self: Sized,
    {
        FadeOut::new(self, duration)
    }
    /// Applies a linear gain ramp to the sound.
    ///
    /// If `clamp_end` is `true`, all samples subsequent to the end of the ramp
    /// will be scaled by the `end_value`. If `clamp_end` is `false`, all
    /// subsequent samples will not have any scaling applied.
    fn linear_gain_ramp(
        self,
        duration: Duration,
        start_value: Float,
        end_value: Float,
        clamp_end: bool,
    ) -> LinearGainRamp<SR, CH, Self>
    where
        Self: Sized,
    {
        LinearGainRamp::new(self, duration, start_value, end_value, clamp_end)
    }
}

// we still need this. More fancy const generics will save us at some point :)
pub struct ConstSourceAdapter<const SR: u32, const CH: u16, S>
where
    S: ConstSource<SR, CH>,
{
    inner: S,
}

impl<const SR: u32, const CH: u16, S> Iterator for ConstSourceAdapter<SR, CH, S>
where
    S: ConstSource<SR, CH>,
{
    type Item = Sample;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<const SR: u32, const CH: u16, S> FixedSource for ConstSourceAdapter<SR, CH, S>
where
    S: ConstSource<SR, CH>,
{
    fn channels(&self) -> ChannelCount {
        const { NonZeroU16::new(CH).expect("channel count must be larger then zero") }
    }

    fn sample_rate(&self) -> SampleRate {
        const { NonZeroU32::new(SR).expect("sample rate must be larger then zero") }
    }

    fn total_duration(&self) -> Option<Duration> {
        self.inner.total_duration()
    }
}

impl<const SR: u32, const CH: u16, S> DynamicSource for ConstSourceAdapter<SR, CH, S>
where
    S: ConstSource<SR, CH>,
{
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> ChannelCount {
        const { NonZeroU16::new(CH).expect("channel count must be larger then zero") }
    }

    fn sample_rate(&self) -> SampleRate {
        const { NonZeroU32::new(SR).expect("sample rate must be larger then zero") }
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        self.inner.total_duration()
    }
}

impl<const SR: u32, const CH: u16> ConstSource<SR, CH> for Box<dyn ConstSource<SR, CH>> {
    fn total_duration(&self) -> Option<Duration> {
        self.as_ref().total_duration()
    }
}

pub trait CollectConstSource<const SR: u32, const CH: u16, const N: usize, S>
where
    S: ConstSource<SR, CH>,
{
    fn collect_mixed(self) -> mixer::UniformArrayMixer<SR, CH, N, S>;
    fn collect_list(self) -> list::UniformArrayList<SR, CH, N, S>;
}

impl<const SR: u32, const CH: u16, const N: usize, S> CollectConstSource<SR, CH, N, S> for [S; N]
where
    S: ConstSource<SR, CH>,
{
    fn collect_mixed(self) -> mixer::UniformArrayMixer<SR, CH, N, S> {
        mixer::UniformArrayMixer { sources: self }
    }
    fn collect_list(self) -> list::UniformArrayList<SR, CH, N, S> {
        list::UniformArrayList {
            sources: self,
            current: 0,
        }
    }
}

macro_rules! add_inner_methods {
    ($name:ident$(<$t:ident$(:$bound:path)?>)?) => {
        impl<const SR: u32, const CH: u16, S: crate::ConstSource<SR, CH>$(,$t$(:$bound)?)?> $name<SR, CH, S$(,$t)?> {
            pub fn inner(&self) -> &S {
                &self.inner
            }
            pub fn inner_mut(&mut self) -> &mut S {
                &mut self.inner
            }
            pub fn into_inner(self) -> S {
                self.inner
            }
        }
    };
}

pub(crate) use add_inner_methods;

macro_rules! impl_wrapper {
    ($name:ident$(<$t:ident$(:$bound:path)?>)?) => {
        impl<const SR: u32, const CH: u16, S: crate::ConstSource<SR, CH>$(,$t$(:$bound)?)?> crate::ConstSource<SR, CH>
            for $name<SR, CH, S$(,$t)?>
        {
            fn total_duration(&self) -> Option<std::time::Duration> {
                self.inner.total_duration()
            }
        }
    };
}
pub(crate) use impl_wrapper;
