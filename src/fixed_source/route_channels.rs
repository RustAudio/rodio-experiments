use std::num::NonZeroU16;

use itertools::Itertools;

use crate::ChannelCount;
use crate::FixedSource;
use crate::SampleRate;

mod array;
mod tuple;
mod vec;

#[derive(thiserror::Error, Debug, Clone)]
pub enum CombineChannelsError {
    #[error("Parameters mismatch, the left {} sources in the list have sample rate: {sample_rate_left} the next source has sample rate: {sample_rate_right} which are not the same", index_of_first_mismatch-1)]
    SampleRateMismatch {
        index_of_first_mismatch: usize,
        sample_rate_left: SampleRate,
        sample_rate_right: SampleRate,
    },
    /// Would love to know what you are trying to do if you run into this :)
    #[error("Trying to combine {0} channels which is more then the maximum of u16::MAX")]
    TooManyChannels(u32),
    #[error("Can not combine channels of zero sources")]
    Empty,
}

pub trait CombineChannels: Sized {
    type TryCombinerSource: FixedSource;

    fn try_combine_channels(self) -> Result<Self::TryCombinerSource, CombineChannelsError>;
}

pub fn verify_params_and_determine_channel_count<S: FixedSource>(
    sources: &[S],
) -> Result<ChannelCount, CombineChannelsError> {
    let channels = sources
        .iter()
        .map(FixedSource::channels)
        .map(|c| c.get() as u32)
        .sum::<u32>();
    let channels: u16 = channels
        .try_into()
        .map_err(|_| CombineChannelsError::TooManyChannels(channels))?;
    let channels = NonZeroU16::new(channels).expect("Sum of NonZero items can not be zero");

    let mut list = sources.iter().map(FixedSource::sample_rate);
    let Some(first) = list.next() else {
        return Err(CombineChannelsError::Empty);
    };

    if let Some((pos, sample_rate_right)) = list.find_position(|sr| *sr != first) {
        Err(CombineChannelsError::SampleRateMismatch {
            index_of_first_mismatch: pos,
            sample_rate_left: first,
            sample_rate_right,
        })
    } else {
        Ok(channels)
    }
}

// pub fn verify_params_and_determine_channel_count_for_tuple<S: FixedSource>(
