use std::time::Duration;

use itertools::Itertools;

use super::super::MaybeConvert;
use super::super::convert_if_needed;
use super::ParamsMismatch;
use crate::FixedSource;
use crate::common::check_params_for_list;
use crate::common::queued_next_body;
use crate::{ChannelCount, SampleRate};

#[derive(Clone, Debug)]
pub struct QueuedVec<S> {
    sources: Vec<S>,
    current: usize,
}

impl<S: FixedSource> Iterator for QueuedVec<S> {
    type Item = crate::Sample;
    fn next(&mut self) -> Option<Self::Item> {
        queued_next_body! {self}
    }
}

impl<S: FixedSource> FixedSource for QueuedVec<S> {
    fn channels(&self) -> rodio::ChannelCount {
        self.sources[0].channels()
    }
    fn sample_rate(&self) -> rodio::SampleRate {
        self.sources[0].sample_rate()
    }
    fn total_duration(&self) -> Option<std::time::Duration> {
        self.sources
            .iter()
            .map(FixedSource::total_duration)
            .fold_options(Duration::ZERO, |sum, s| sum + s)
    }
}

impl<S: FixedSource> super::IntoQueued for Vec<S> {
    type TryQueuedSource = QueuedVec<S>;
    type IntoQueuedSource = QueuedVec<MaybeConvert<S>>;

    fn try_into_queued(self) -> Result<Self::TryQueuedSource, ParamsMismatch> {
        check_params_for_list! {self}
        Ok(Self::TryQueuedSource {
            sources: self,
            current: 0,
        })
    }
    fn into_queued_converted(
        self,
        sample_rate: SampleRate,
        channels: ChannelCount,
    ) -> Self::IntoQueuedSource {
        Self::IntoQueuedSource {
            sources: self
                .into_iter()
                .map(|s| convert_if_needed(s, sample_rate, channels))
                .collect(),
            current: 0,
        }
    }
}
