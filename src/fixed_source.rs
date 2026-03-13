use core::fmt;
use std::time::Duration;

use crate::ChannelCount;
use crate::FixedSource;
use crate::Float;
use crate::Sample;
use crate::SampleRate;
use crate::effects::blt::BltFormula;
use crate::effects::fixed_source::ChannelVolume;
use crate::effects::fixed_source::Distortion;
use crate::effects::fixed_source::Dither;
use crate::effects::fixed_source::FadeIn;
use crate::effects::fixed_source::FadeOut;
use crate::effects::fixed_source::LinearGainRamp;
use crate::effects::fixed_source::TrackPosition;

use crate::ConstSource;
use crate::effects::amplify::Factor;
use crate::effects::automatic_gain_control::AutomaticGainControlSettings;
use crate::effects::dither::Algorithm as DitherAlgorithm;
use crate::effects::fixed_source::BltFilter;
use crate::effects::fixed_source::Limit;
use crate::effects::fixed_source::{
    Amplify, AutomaticGainControl, InspectFrame, Pausable, PeriodicAccess, Stoppable, TakeDuration,
    TakeSamples, WithData,
};
use crate::effects::limiter::LimitSettings;
use buffer::SamplesBuffer;
use chain::SourceChain;

pub mod buffer;
pub mod chain;
pub mod conversions;
/// can add stuff on the fly after creation
pub mod queue;
/// can not be changed after creation
pub mod queued;
/// can not be changed after creation
pub mod mixed;
/// can add stuff on the fly after creation
pub mod mixer;

pub mod macros;
pub(crate) use macros::{add_inner_methods, impl_wrapper, tuple_impl};

use conversions::channel_count::ChannelConverter;
use conversions::sample_rate::Resampler;

pub trait FixedSourceExt: FixedSource {
    #[doc = include_str!("effects/take_duration.md")]
    fn take_duration(self, duration: Duration) -> TakeDuration<Self>
    where
        Self: Sized,
    {
        TakeDuration::new(self, duration)
    }

    #[doc = include_str!("effects/take_samples.md")]
    fn take_samples(self, samples: usize) -> TakeSamples<Self>
    where
        Self: Sized,
    {
        TakeSamples::new(self, samples)
    }

    #[doc = include_str!("effects/periodic_access.md")]
    fn periodic_access(self, call_every: Duration, arg: fn(&mut Self)) -> PeriodicAccess<Self>
    where
        Self: Sized,
    {
        PeriodicAccess::new(self, call_every, arg)
    }

    #[doc = include_str!("effects/with_data.md")]
    fn with_data<D>(self, data: D) -> WithData<Self, D>
    where
        Self: Sized,
    {
        WithData::new(self, data)
    }

    fn with_sample_rate(self, sample_rate: SampleRate) -> Resampler<Self>
    where
        Self: Sized,
    {
        Resampler::new(self, sample_rate)
    }

    fn with_channel_count(self, channel_count: ChannelCount) -> ChannelConverter<Self>
    where
        Self: Sized,
    {
        ChannelConverter::new(self, channel_count)
    }

    /// Tries to convert from a fixed source to a const one assuming
    /// the parameters already match. If they do not this returns an error.
    ///
    /// If the parameters do not match you can resample using: ``
    fn try_into_const_source<const SR: u32, const CH: u16>(
        self,
    ) -> Result<IntoConstSource<SR, CH, Self>, ParameterMismatch<SR, CH>>
    where
        Self: Sized,
    {
        if self.channels().get() != CH || self.sample_rate().get() != SR {
            Err(ParameterMismatch {
                sample_rate: self.sample_rate(),
                channel_count: self.channels(),
            })
        } else {
            Ok(IntoConstSource(self))
        }
    }

    #[doc = include_str!("effects/pausable.md")]
    fn pausable(self, paused: bool) -> Pausable<Self>
    where
        Self: Sized,
    {
        Pausable::new(self, paused)
    }

    #[doc = include_str!("effects/amplify.md")]
    fn amplify(self, factor: Factor) -> Amplify<Self>
    where
        Self: Sized,
    {
        Amplify::new(self, factor)
    }

    #[doc = include_str!("effects/stoppable.md")]
    fn stoppable(self) -> Stoppable<Self>
    where
        Self: Sized,
    {
        Stoppable::new(self)
    }

    #[doc = include_str!("effects/inspect_frame.md")]
    fn inspect_frame<F: FnMut(Vec<Sample>) -> Vec<Sample>>(self, f: F) -> InspectFrame<Self, F>
    where
        Self: Sized,
    {
        InspectFrame::new(self, f)
    }

    #[doc = include_str!("effects/automatic_gain_control.md")]
    fn automatic_gain_control(
        self,
        settings: AutomaticGainControlSettings,
    ) -> AutomaticGainControl<Self>
    where
        Self: Sized,
    {
        AutomaticGainControl::new(self, settings)
    }

    #[doc = include_str!("effects/collect_into_buffer.md")]
    fn collect_into_buffer(self) -> SamplesBuffer
    where
        Self: Sized,
    {
        SamplesBuffer::new(
            self.channels(),
            self.sample_rate(),
            self.collect::<Vec<_>>(),
        )
    }

    #[doc = include_str!("effects/limit.md")]
    fn limit(self, settings: LimitSettings) -> Limit<Self>
    where
        Self: Sized,
    {
        Limit::new(self, settings)
    }

    #[doc = include_str!("effects/low_pass.md")]
    fn low_pass(self, freq: u32) -> BltFilter<Self>
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
    fn high_pass(self, freq: u32) -> BltFilter<Self>
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
    fn low_pass_with_bandwidth(self, freq: u32, bandwidth: Float) -> BltFilter<Self>
    where
        Self: Sized,
    {
        BltFilter::new(self, BltFormula::LowPass { freq, bandwidth })
    }

    #[doc = include_str!("effects/high_pass_with_bandwidth.md")]
    fn high_pass_with_bandwidth(self, freq: u32, bandwidth: Float) -> BltFilter<Self>
    where
        Self: Sized,
    {
        BltFilter::new(self, BltFormula::HighPass { freq, bandwidth })
    }

    #[doc = include_str!("effects/fade_in.md")]
    fn fade_in(self, duration: Duration) -> FadeIn<Self>
    where
        Self: Sized,
    {
        FadeIn::new(self, duration)
    }

    #[doc = include_str!("effects/fade_out.md")]
    fn fade_out(self, duration: Duration) -> FadeOut<Self>
    where
        Self: Sized,
    {
        FadeOut::new(self, duration)
    }

    #[doc = include_str!("effects/linear_gain_ramp.md")]
    fn linear_gain_ramp(
        self,
        duration: Duration,
        start_value: Float,
        end_value: Float,
        clamp_end: bool,
    ) -> LinearGainRamp<Self>
    where
        Self: Sized,
    {
        LinearGainRamp::new(self, duration, start_value, end_value, clamp_end)
    }

    #[doc = include_str!("effects/distortion.md")]
    fn distortion(self, gain: Float, threshold: Float) -> Distortion<Self>
    where
        Self: Sized,
    {
        Distortion::new(self, gain, threshold)
    }

    #[doc = include_str!("effects/channel_volume.md")]
    fn channel_volume(self, channel_volumes: Vec<Float>) -> ChannelVolume<Self>
    where
        Self: Sized,
    {
        ChannelVolume::new(self, channel_volumes)
    }

    #[doc = include_str!("effects/dither.md")]
    fn dither(self, target_bits: crate::BitDepth, algorithm: DitherAlgorithm) -> Dither<Self>
    where
        Self: Sized,
    {
        Dither::new(self, target_bits, algorithm)
    }

    #[doc = include_str!("effects/position.md")]
    fn track_position(self) -> TrackPosition<Self>
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
    /// # use rodio_experiments::nz;
    /// # use rodio_experiments::fixed_source::FixedSourceExt;
    /// # use rodio_experiments::fixed_source::buffer::SamplesBuffer;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let preamble = SamplesBuffer::new(nz!(1), nz!(1), [1.0, 1.0]);
    /// let signal = SamplesBuffer::new(nz!(1), nz!(1), [2.0, 2.0]);

    /// let mixed = preamble.try_chain_source(signal)?;
    /// assert_eq!(mixed.collect::<Vec<_>>(), vec![1.0,1.0,2.0,2.0]);
    /// # Ok(())
    /// # }
    /// ```
    fn try_chain_source<S: FixedSource>(
        self,
        next: S,
    ) -> Result<SourceChain<Self, S>, chain::ParamsMismatch>
    where
        Self: Sized,
    {
        SourceChain::new(self, next)
    }
}

impl<S: FixedSource> FixedSourceExt for S {}

pub struct IntoConstSource<const SR: u32, const CH: u16, S: FixedSource>(S);

impl<const SR: u32, const CH: u16, S: FixedSource> ConstSource<SR, CH>
    for IntoConstSource<SR, CH, S>
{
    fn total_duration(&self) -> Option<Duration> {
        self.0.total_duration()
    }
}

impl<const SR: u32, const CH: u16, S: FixedSource> Iterator for IntoConstSource<SR, CH, S> {
    type Item = Sample;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

#[derive(Debug)]
pub struct ParameterMismatch<const SR: u32, const CH: u16> {
    sample_rate: SampleRate,
    channel_count: ChannelCount,
}

impl<const SR: u32, const CH: u16> std::error::Error for ParameterMismatch<SR, CH> {}

impl<const SR: u32, const CH: u16> std::fmt::Display for ParameterMismatch<SR, CH> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.sample_rate.get() == SR && self.channel_count.get() == CH {
            unreachable!("ParameterMismatch error can only occur when params mismatch");
        } else if self.sample_rate.get() == SR && self.channel_count.get() != CH {
            f.write_fmt(format_args!("Fixed source's channel count: {}, does not match target const source's channel count: {}", self.channel_count.get(), CH))
        } else if self.sample_rate.get() != SR && self.channel_count.get() != CH {
            f.write_fmt(format_args!("Fixed source's sample rate and channel count ({}, {}) do not match target const source's sample rate and channel count ({} {})", self.sample_rate.get(), self.channel_count.get(), SR, CH))
        } else {
            f.write_fmt(format_args!("Fixed source's sample rate : {}, does not match target const source's sample rate: {}", self.sample_rate.get(), SR))
        }
    }
}

pub enum MaybeConvert<S: FixedSource> {
    OnlyResample(Resampler<S>),
    OnlyRechannel(ChannelConverter<S>),
    // more efficient when adding channels
    ResampleThenRechannel(ChannelConverter<Resampler<S>>),
    // more efficient when removing channels
    RechannelThenResamples(Resampler<ChannelConverter<S>>),
    Unchanged(S),
}

impl<S: FixedSource> fmt::Debug for MaybeConvert<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OnlyResample(_) => f.debug_tuple("OnlyResample").finish_non_exhaustive(),
            Self::OnlyRechannel(_) => f.debug_tuple("OnlyRechannel").finish_non_exhaustive(),
            Self::ResampleThenRechannel(_) => f
                .debug_tuple("ResampleThenRechannel")
                .finish_non_exhaustive(),
            Self::RechannelThenResamples(_) => f
                .debug_tuple("RechannelThenResamples")
                .finish_non_exhaustive(),
            Self::Unchanged(_) => f.debug_tuple("Unchanged").finish_non_exhaustive(),
        }
    }
}

impl<S: FixedSource> FixedSource for MaybeConvert<S> {
    fn channels(&self) -> ChannelCount {
        match self {
            MaybeConvert::OnlyResample(s) => s.channels(),
            MaybeConvert::OnlyRechannel(s) => s.channels(),
            MaybeConvert::ResampleThenRechannel(s) => s.channels(),
            MaybeConvert::RechannelThenResamples(s) => s.channels(),
            MaybeConvert::Unchanged(s) => s.channels(),
        }
    }

    fn sample_rate(&self) -> SampleRate {
        match self {
            MaybeConvert::OnlyResample(s) => s.sample_rate(),
            MaybeConvert::OnlyRechannel(s) => s.sample_rate(),
            MaybeConvert::ResampleThenRechannel(s) => s.sample_rate(),
            MaybeConvert::RechannelThenResamples(s) => s.sample_rate(),
            MaybeConvert::Unchanged(s) => s.sample_rate(),
        }
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        match self {
            MaybeConvert::OnlyResample(s) => s.total_duration(),
            MaybeConvert::OnlyRechannel(s) => s.total_duration(),
            MaybeConvert::ResampleThenRechannel(s) => s.total_duration(),
            MaybeConvert::RechannelThenResamples(s) => s.total_duration(),
            MaybeConvert::Unchanged(s) => s.total_duration(),
        }
    }
}

impl<S: FixedSource> Iterator for MaybeConvert<S> {
    type Item = crate::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            MaybeConvert::OnlyResample(s) => s.next(),
            MaybeConvert::OnlyRechannel(s) => s.next(),
            MaybeConvert::ResampleThenRechannel(s) => s.next(),
            MaybeConvert::RechannelThenResamples(s) => s.next(),
            MaybeConvert::Unchanged(s) => s.next(),
        }
    }
}

pub(crate) fn convert_if_needed<S: FixedSource>(
    s: S,
    sample_rate: SampleRate,
    channels: ChannelCount,
) -> MaybeConvert<S> {
    use crate::fixed_source::FixedSourceExt;
    use MaybeConvert as M;
    match (s.sample_rate() == sample_rate, s.channels() == channels) {
        (true, true) => M::Unchanged(s),
        (true, false) => M::OnlyRechannel(s.with_channel_count(channels)),
        (false, true) => M::OnlyResample(s.with_sample_rate(sample_rate)),
        (false, false) if s.channels() > channels => {
            M::ResampleThenRechannel(s.with_sample_rate(sample_rate).with_channel_count(channels))
        }
        (false, false) => {
            M::RechannelThenResamples(s.with_channel_count(channels).with_sample_rate(sample_rate))
        }
    }
}
