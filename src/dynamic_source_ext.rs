use std::time::Duration;

use rodio::{ChannelCount, FixedSource, Sample, SampleRate};

use crate::Source as DynamicSource;
use crate::conversions::channelcount::VariableInputChannelConvertor;
use crate::conversions::resampler::variable_input::VariableInputResampler;
use crate::effects::amplify::Factor;
use crate::effects::dynamic_source::{Pausable, PeriodicAccess, Stoppable, WithData, Amplify};

/// Just here for the experimental phase, since we cant add anything
/// to Source/DynamicSource during it.
pub trait ExtendDynamicSource {
    fn into_fixed_source(
        self,
        sample_rate: SampleRate,
        channel_count: ChannelCount,
    ) -> IntoFixedSource<Self>
    where
        Self: DynamicSource + Sized;

    fn amplify(self, factor: Factor) -> Amplify<Self>
    where
        Self: DynamicSource + Sized,
    {
        Amplify::new(self, factor)
    }

    fn stoppable(self) -> Stoppable<Self>
    where
        Self: DynamicSource + Sized,
    {
        Stoppable::new(self)
    }

    fn pausable(self, paused: bool) -> Pausable<Self>
    where
        Self: DynamicSource + Sized,
    {
        Pausable::new(self, paused)
    }

    fn periodic_access(self, call_every: Duration, arg: fn(&mut Self)) -> PeriodicAccess<Self>
    where
        Self: DynamicSource + Sized,
    {
        PeriodicAccess::new(self, call_every, arg)
    }

    fn with_data<D>(self, data: D) -> WithData<Self, D>
    where
        Self: DynamicSource + Sized,
    {
        WithData::new(self, data)
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

impl<S: DynamicSource> ExtendDynamicSource for S {
    fn into_fixed_source(
        self,
        sample_rate: SampleRate,
        channel_count: ChannelCount,
    ) -> IntoFixedSource<Self> {
        let source = VariableInputChannelConvertor::new(self, channel_count);
        let source = VariableInputResampler::new(source, sample_rate);
        IntoFixedSource(source)
    }
}
