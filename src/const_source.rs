use std::num::NonZeroU16;
use std::num::NonZeroU32;
use std::time::Duration;

use crate::effects::amplify::Factor;
use crate::ChannelCount;
use crate::FixedSource;
use crate::Float;
use crate::Sample;
use crate::SampleRate;
use crate::Source as DynamicSource; // will be renamed to this upstream
use crate::const_source::chain::SourceChain;
use crate::effects::const_source::ChannelVolume;
use crate::effects::const_source::Distortion;
use crate::effects::const_source::Dither;
use crate::effects::const_source::FadeIn;
use crate::effects::const_source::FadeOut;
use crate::effects::const_source::FadeOutAfter;
use crate::effects::const_source::LinearGainRamp;
use crate::effects::const_source::TrackPosition;

// pub mod adapter; replaced with into_fixed_source and into_const_source
pub mod buffer;
mod chain;
pub mod conversions;
/// Can not be changed after creation
pub mod mixed;
/// Can be added to after creation
pub mod mixer;
/// Can be added to after creation
pub mod queue;
/// Can not be changed after creation
pub mod queued;
pub mod route_channels;

mod macros;
pub(crate) use macros::{add_inner_methods, impl_wrapper};

use crate::const_source::buffer::SamplesBuffer;
use crate::const_source::conversions::channelcount::ChannelConvertor;
use crate::effects::automatic_gain_control::AutomaticGainControlSettings;
use crate::effects::blt::BltFormula;
use crate::effects::const_source::BltFilter;
use crate::effects::const_source::Limit;
use crate::effects::const_source::{
    Amplify, AutomaticGainControl, InspectFrame, Pausable, PeriodicAccess, Stoppable, TakeDuration,
    TakeSamples, WithData,
};
use crate::effects::dither::Algorithm as DitherAlgorithm;
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

    #[doc = include_str!("effects/take_duration.md")]
    fn take_duration(self, duration: Duration) -> TakeDuration<SR, CH, Self>
    where
        Self: Sized,
    {
        TakeDuration::new(self, duration)
    }

    #[doc = include_str!("effects/take_samples.md")]
    fn take_samples(self, samples: usize) -> TakeSamples<SR, CH, Self>
    where
        Self: Sized,
    {
        TakeSamples::new(self, samples)
    }

    #[doc = include_str!("effects/periodic_access.md")]
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

    #[doc = include_str!("effects/with_data.md")]
    fn with_data<D>(self, data: D) -> WithData<SR, CH, Self, D>
    where
        Self: Sized,
    {
        WithData::new(self, data)
    }

    #[doc = include_str!("effects/amplify.md")]
    fn amplify(self, factor: Factor) -> Amplify<SR, CH, Self>
    where
        Self: Sized,
    {
        Amplify::new(self, factor)
    }

    #[doc = include_str!("effects/stoppable.md")]
    fn stoppable(self) -> Stoppable<SR, CH, Self>
    where
        Self: Sized,
    {
        Stoppable::new(self)
    }

    #[doc = include_str!("effects/pausable.md")]
    fn pausable(self, paused: bool) -> Pausable<SR, CH, Self>
    where
        Self: Sized,
    {
        Pausable::new(self, paused)
    }

    #[doc = include_str!("effects/inspect_frame.md")]
    fn inspect_frame<F: FnMut(Vec<Sample>) -> Vec<Sample>>(
        self,
        f: F,
    ) -> InspectFrame<SR, CH, Self, F>
    where
        Self: Sized,
    {
        InspectFrame::new(self, f)
    }

    #[doc = include_str!("effects/collect_into_buffer.md")]
    fn collect_into_buffer(self) -> SamplesBuffer<SR, CH>
    where
        Self: Sized,
    {
        SamplesBuffer::new(self.collect::<Vec<_>>())
    }

    #[doc = include_str!("effects/automatic_gain_control.md")]
    fn automatic_gain_control(
        self,
        settings: AutomaticGainControlSettings,
    ) -> AutomaticGainControl<SR, CH, Self>
    where
        Self: Sized,
    {
        AutomaticGainControl::new(self, settings)
    }

    #[doc = include_str!("effects/limit.md")]
    fn limit(self, settings: LimitSettings) -> Limit<SR, CH, Self>
    where
        Self: Sized,
    {
        Limit::new(self, settings)
    }

    #[doc = include_str!("effects/low_pass.md")]
    fn low_pass(self, freq: u32) -> BltFilter<SR, CH, Self>
    where
        Self: Sized,
    {
        BltFilter::new(
            self,
            BltFormula::LowPass {
                freq,
                bandwidth: 0.5,
            },
        )
    }

    #[doc = include_str!("effects/high_pass.md")]
    fn high_pass(self, freq: u32) -> BltFilter<SR, CH, Self>
    where
        Self: Sized,
    {
        BltFilter::new(
            self,
            BltFormula::HighPass {
                freq,
                bandwidth: 0.5,
            },
        )
    }

    #[doc = include_str!("effects/low_pass_with_bandwidth.md")]
    fn low_pass_with_q(self, freq: u32, bandwidth: Float) -> BltFilter<SR, CH, Self>
    where
        Self: Sized,
    {
        BltFilter::new(self, BltFormula::LowPass { freq, bandwidth })
    }

    #[doc = include_str!("effects/high_pass_with_bandwidth.md")]
    fn high_pass_with_q(self, freq: u32, bandwidth: Float) -> BltFilter<SR, CH, Self>
    where
        Self: Sized,
    {
        BltFilter::new(self, BltFormula::HighPass { freq, bandwidth })
    }
    #[doc = include_str!("effects/fade_in.md")]
    fn fade_in(self, duration: Duration) -> FadeIn<SR, CH, Self>
    where
        Self: Sized,
    {
        FadeIn::new(self, duration)
    }

    #[doc = include_str!("effects/fade_out.md")]
    fn fade_out(self, duration: Duration) -> FadeOut<SR, CH, Self>
    where
        Self: Sized,
    {
        FadeOut::new(self, duration)
    }

    #[doc = include_str!("effects/fade_out_after.md")]
    fn fade_out_after(
        self,
        start_after: Duration,
        fade_duration: Duration,
    ) -> FadeOutAfter<SR, CH, Self>
    where
        Self: Sized,
    {
        FadeOutAfter::new(self, start_after, fade_duration)
    }

    #[doc = include_str!("effects/linear_gain_ramp.md")]
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

    #[doc = include_str!("effects/distortion.md")]
    fn distortion(self, gain: Float, threshold: Float) -> Distortion<SR, CH, Self>
    where
        Self: Sized,
    {
        Distortion::new(self, gain, threshold)
    }

    #[doc = include_str!("effects/channel_volume.md")]
    fn channel_volume(self, channel_volumes: Vec<Float>) -> ChannelVolume<SR, CH, Self>
    where
        Self: Sized,
    {
        ChannelVolume::new(self, channel_volumes)
    }

    #[doc = include_str!("effects/dither.md")]
    fn dither(
        self,
        target_bits: crate::BitDepth,
        algorithm: DitherAlgorithm,
    ) -> Dither<SR, CH, Self>
    where
        Self: Sized,
    {
        Dither::new(self, target_bits, algorithm)
    }

    #[doc = include_str!("effects/position.md")]
    fn track_position(self) -> TrackPosition<SR, CH, Self>
    where
        Self: Sized,
    {
        TrackPosition::new(self)
    }

    /// Add another source to play directly after this one.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use rodio_experiments::const_source::ConstSource;
    /// # use rodio_experiments::const_source::buffer::SamplesBuffer;
    /// let preamble = SamplesBuffer::<44100, 1>::new([1.0, 1.0]);
    /// let signal = SamplesBuffer::<44100, 1>::new([2.0, 2.0]);

    /// let mixed = preamble.chain_source(signal);
    /// assert_eq!(mixed.collect::<Vec<_>>(), vec![1.0,1.0,2.0,2.0])
    /// ```
    fn chain_source<S>(self, next: S) -> SourceChain<SR, CH, Self, S>
    where
        Self: Sized,
        S: ConstSource<SR, CH>,
    {
        SourceChain::new(self, next)
    }
}

// we still need this. More fancy const generics will save us at some point :)
#[derive(Clone)]
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
