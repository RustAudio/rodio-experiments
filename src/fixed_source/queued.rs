use std::fmt::Display;

use crate::FixedSource;
use crate::{ChannelCount, SampleRate};

mod array;
mod tuple;
mod vec;

pub trait IntoQueued {
    type TryQueuedSource: FixedSource;
    type IntoQueuedSource: FixedSource;

    fn try_into_queued(self) -> Result<Self::TryQueuedSource, ParamsMismatch>;
    fn into_queued_converted(
        self,
        sample_rate: SampleRate,
        channels: ChannelCount,
    ) -> Self::IntoQueuedSource;
}

#[derive(Debug, Clone, Copy, thiserror::Error, PartialEq, Eq)]
pub struct ParamsMismatch {
    pub(crate) index_of_first_mismatch: usize,
    pub(crate) sample_rate_left: SampleRate,
    pub(crate) channel_count_left: ChannelCount,
    pub(crate) sample_rate_right: SampleRate,
    pub(crate) channel_count_right: ChannelCount,
}

impl Display for ParamsMismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ParamsMismatch {
            index_of_first_mismatch,
            sample_rate_left,
            channel_count_left,
            sample_rate_right,
            channel_count_right,
        } = self;
        f.write_fmt(format_args!("Parameters mismatch, the left {} sources in the list have sample rate: {sample_rate_left} and channel count: {channel_count_left} the next source has sample rate: {sample_rate_right} and {channel_count_right} which are not the same", index_of_first_mismatch-1))
    }
}
