use std::time::Duration;

use rodio::ChannelCount;
use rodio::FixedSource;
use rodio::Sample;
use rodio::SampleRate;

use crate::ConstSource;
use crate::conversions::channelcount::fixed_input::ChannelConverter;
use crate::conversions::resampler::fixed_input::Resampler;
use crate::effects::amplify::Factor;
use crate::effects::amplify::fixed_source::Amplify;
use crate::effects::inspect::fixed_source::InspectFrame;
use crate::effects::pausable::fixed_source::Pausable;
use crate::effects::periodic_access::fixed_source::PeriodicAccess;
use crate::effects::stoppable::fixed_source::Stoppable;
use crate::effects::take_duration::fixed_source::TakeDuration;
use crate::effects::take_samples::fixed_source::TakeSamples;
use crate::effects::with_data::fixed_source::WithData;
use crate::fixed_source::buffer::SamplesBuffer;

pub mod buffer;
pub mod queue;

pub trait FixedSourceExt: FixedSource {
    fn take_duration(self, duration: Duration) -> TakeDuration<Self>
    where
        Self: Sized,
    {
        TakeDuration::new(self, duration)
    }

    fn take_samples(self, samples: usize) -> TakeSamples<Self>
    where
        Self: Sized,
    {
        TakeSamples::new(self, samples)
    }

    fn periodic_access(self, call_every: Duration, arg: fn(&mut Self)) -> PeriodicAccess<Self>
    where
        Self: Sized,
    {
        PeriodicAccess::new(self, call_every, arg)
    }

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

    fn pausable(self, paused: bool) -> Pausable<Self>
    where
        Self: Sized,
    {
        Pausable::new(self, paused)
    }

    fn amplify(self, factor: Factor) -> Amplify<Self>
    where
        Self: Sized,
    {
        Amplify::new(self, factor)
    }

    fn stoppable(self) -> Stoppable<Self>
    where
        Self: Sized,
    {
        Stoppable::new(self)
    }

    fn inspect_frame<F: FnMut(Vec<Sample>) -> Vec<Sample>>(self, f: F) -> InspectFrame<Self, F>
    where
        Self: Sized,
    {
        InspectFrame::new(self, f)
    }

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

macro_rules! add_inner_methods {
    ($name:ident$(<$t:ident$(:$bound:path)?>)?) => {
        impl<S: crate::FixedSource $(,$t$(:$bound)?)?> $name<S $(,$t)?> {
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

macro_rules! impl_wrapper {
    ($name:ident$(<$t:ident$(:$bound:path)?>)?) => {
        impl<S: crate::FixedSource $(,$t$(:$bound)?)?> crate::FixedSource for $name<S $(,$t)?> {
            fn channels(&self) -> rodio::ChannelCount {
                self.inner.channels()
            }

            fn sample_rate(&self) -> rodio::SampleRate {
                self.inner.sample_rate()
            }

            fn total_duration(&self) -> Option<std::time::Duration> {
                self.inner.total_duration()
            }
        }
    };
}

pub(crate) use add_inner_methods;
pub(crate) use impl_wrapper;
