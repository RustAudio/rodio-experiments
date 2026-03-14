use std::time::Duration;

use itertools::Itertools;

use super::super::MaybeConvert;
use super::super::convert_if_needed;
use super::ParamsMismatch;
use crate::FixedSource;
use crate::common::check_params_for_list;
use crate::common::queued_next_body;
use crate::fixed_source::queued::IntoQueued;
use crate::{ChannelCount, SampleRate};

#[derive(Clone, Debug)]
pub struct QueuedArray<const N: usize, S> {
    sources: [S; N],
    current: usize,
}

impl<const N: usize, S: FixedSource> Iterator for QueuedArray<N, S> {
    type Item = crate::Sample;
    fn next(&mut self) -> Option<Self::Item> {
        queued_next_body! {self}
    }
}

impl<const N: usize, S: FixedSource> FixedSource for QueuedArray<N, S> {
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

impl<const N: usize, S: FixedSource> IntoQueued for [S; N] {
    type TryQueuedSource = QueuedArray<N, S>;
    type IntoQueuedSource = QueuedArray<N, MaybeConvert<S>>;

    fn try_into_list(self) -> Result<Self::TryQueuedSource, ParamsMismatch> {
        check_params_for_list! {self}
        Ok(Self::TryQueuedSource {
            sources: self,
            current: 0,
        })
    }
    fn into_list_converted(
        self,
        sample_rate: SampleRate,
        channels: ChannelCount,
    ) -> Self::IntoQueuedSource {
        Self::IntoQueuedSource {
            sources: self.map(|s| convert_if_needed(s, sample_rate, channels)),
            current: 0,
        }
    }
}
