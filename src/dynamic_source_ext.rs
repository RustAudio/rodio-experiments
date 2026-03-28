use std::time::Duration;

use rodio::{ChannelCount, FixedSource, Sample, SampleRate};

use crate::Float;
use crate::Source as DynamicSource;
use crate::dynamic_source::conversions::channel_count::VariableInputChannelConvertor;
use crate::dynamic_source::conversions::sample_rate::VariableInputResampler;
use crate::effects::dynamic_source::Distortion;
use crate::effects::dynamic_source::{Pausable, PeriodicAccess, Stoppable, WithData};

/// Just here for the experimental phase, since we cant add anything
/// to Source/DynamicSource during it.
pub trait ExtendDynamicSource {
    /// Only succeeds if the span length is `None` (infinite).
    fn try_as_fixed_source(self) -> Result<AsFixedSource<Self>, ParametersCanChange>
    where
        Self: DynamicSource + Sized;
    fn resample_into_fixed_source(
        self,
        sample_rate: SampleRate,
        channel_count: ChannelCount,
    ) -> IntoFixedSource<Self>
    where
        Self: DynamicSource + Sized;

    #[doc = include_str!("effects/stoppable.md")]
    fn stoppable(self) -> Stoppable<Self>
    where
        Self: DynamicSource + Sized,
    {
        Stoppable::new(self)
    }

    #[doc = include_str!("effects/pausable.md")]
    fn pausable(self, paused: bool) -> Pausable<Self>
    where
        Self: DynamicSource + Sized,
    {
        Pausable::new(self, paused)
    }

    #[doc = include_str!("effects/periodic_access.md")]
    fn periodic_access(self, call_every: Duration, arg: fn(&mut Self)) -> PeriodicAccess<Self>
    where
        Self: DynamicSource + Sized,
    {
        PeriodicAccess::new(self, call_every, arg)
    }

    #[doc = include_str!("effects/with_data.md")]
    fn with_data<D>(self, data: D) -> WithData<Self, D>
    where
        Self: DynamicSource + Sized,
    {
        WithData::new(self, data)
    }
    #[doc = include_str!("effects/distortion.md")]
    fn distortion(self, gain: Float, threshold: Float) -> Distortion<Self>
    where
        Self: DynamicSource + Sized,
    {
        Distortion::new(self, gain, threshold)
    }
}

pub struct IntoFixedSource<S: DynamicSource>(
    VariableInputResampler<VariableInputChannelConvertor<S>>,
);

impl<S: DynamicSource> FixedSource for IntoFixedSource<S> {
    fn channels(&self) -> ChannelCount {
        self.0.channels()
    }

    fn sample_rate(&self) -> SampleRate {
        self.0.sample_rate()
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        self.0.total_duration()
    }
}

impl<S: DynamicSource> Iterator for IntoFixedSource<S> {
    type Item = Sample;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub struct AsFixedSource<S: DynamicSource>(S);

impl<S: DynamicSource> FixedSource for AsFixedSource<S> {
    fn channels(&self) -> ChannelCount {
        self.0.channels()
    }

    fn sample_rate(&self) -> SampleRate {
        self.0.sample_rate()
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        self.0.total_duration()
    }
}

impl<S: DynamicSource> Iterator for AsFixedSource<S> {
    type Item = Sample;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

#[derive(Debug, thiserror::Error)]
#[error(
    "Source has a span which means its parameters may change. Try resampling it into a
        fixed source with `resample_into_fixed_source`"
)]
pub struct ParametersCanChange;

impl<S: DynamicSource> ExtendDynamicSource for S {
    fn resample_into_fixed_source(
        self,
        sample_rate: SampleRate,
        channel_count: ChannelCount,
    ) -> IntoFixedSource<Self> {
        let source = VariableInputChannelConvertor::new(self, channel_count);
        let source = VariableInputResampler::new(source, sample_rate);
        IntoFixedSource(source)
    }

    fn try_as_fixed_source(self) -> Result<AsFixedSource<Self>, ParametersCanChange>
    where
        Self: DynamicSource + Sized,
    {
        if self.current_span_len().is_none() {
            Ok(AsFixedSource(self))
        } else {
            Err(ParametersCanChange)
        }
    }
}
