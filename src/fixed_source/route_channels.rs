use std::num::NonZeroU16;

use itertools::Itertools;

use crate::ChannelCount;
use crate::FixedSource;
use crate::SampleRate;
use crate::fixed_source::list_of_sources::ListOfSources;

use std::time::Duration;

#[derive(Debug)]
pub struct CombinedChannels<T> {
    inner: T,
    channels: ChannelCount,
    current: u16,
}

impl<T: ListOfSources> FixedSource for CombinedChannels<T> {
    fn channels(&self) -> ChannelCount {
        self.channels
    }

    fn sample_rate(&self) -> SampleRate {
        self.inner.sample_rate(0)
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        (0..self.inner.len())
            .into_iter()
            .map(|idx| self.inner.total_duration(idx))
            .fold_options(Duration::ZERO, |sum, dur| sum + dur)
    }
}

impl<T: ListOfSources> Iterator for CombinedChannels<T> {
    type Item = crate::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        let channels = self.channels().get();
        let mut channel = 0;
        for i in 0..self.inner.len() {
            if (channel..(channel + self.inner.channels(i).get())).contains(&self.current) {
                self.current += 1;
                self.current %= channels;
                return self.inner.next(i);
            } else {
                channel += self.inner.channels(i).get()
            }
        }
        None
    }
}

pub trait CombineChannels {
    fn try_combine_channels(self) -> Result<CombinedChannels<Self>, CombineChannelsError>
    where
        Self: ListOfSources + Sized;
}

impl<T> CombineChannels for T {
    fn try_combine_channels(self) -> Result<CombinedChannels<Self>, CombineChannelsError>
    where
        Self: ListOfSources + Sized,
    {
        let sources: &T = &self;
        let channels = (0..sources.len())
            .map(|i| sources.channels(i).get() as u32)
            .sum::<u32>();
        let channels: u16 = channels
            .try_into()
            .map_err(|_| CombineChannelsError::TooManyChannels(channels))?;
        let channels = NonZeroU16::new(channels).expect("Sum of NonZero items can not be zero");

        let mut rates = (0..sources.len()).map(|i| sources.sample_rate(i));
        let first = rates.next().ok_or(CombineChannelsError::Empty)?;

        if let Some((pos, sample_rate_right)) = rates.find_position(|sr| *sr != first) {
            return Err(CombineChannelsError::SampleRateMismatch {
                index_of_first_mismatch: pos,
                sample_rate_left: first,
                sample_rate_right,
            });
        }

        Ok(CombinedChannels {
            channels,
            inner: self,
            current: 0,
        })
    }
}

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

#[cfg(test)]
pub(crate) mod tests {
    use itertools::Itertools;

    use crate::fixed_source::buffer::SamplesBuffer;
    use crate::nz;

    use super::*;

    #[test]
    fn combined2() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(1), vec![1.0, 2.0, 3.0]);
        let s2 = SamplesBuffer::new(nz!(1), nz!(1), vec![4.0, 5.0, 6.0]);

        assert_eq!(
            vec![1., 4., 2., 5., 3., 6.],
            (s1, s2).try_combine_channels().unwrap().collect::<Vec<_>>()
        );
    }

    #[test]
    fn combined3() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(1), vec![1.0]);
        let s2 = SamplesBuffer::new(nz!(2), nz!(1), vec![2.0, 3.0]);
        let s3 = SamplesBuffer::new(nz!(1), nz!(1), vec![4.0]);

        assert_eq!(
            vec![1., 2., 3., 4.],
            (s1, s2, s3)
                .try_combine_channels()
                .unwrap()
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn combined3_mismatch() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(1), vec![1.0]);
        let s2 = SamplesBuffer::new(nz!(2), nz!(2), vec![2.0]);
        let s3 = SamplesBuffer::new(nz!(1), nz!(1), vec![3.0]);

        assert!((s1, s2, s3).try_combine_channels().is_err());
    }

    #[test]
    fn five_channel_audio_samples_in_correct_order() {
        let s1 = SamplesBuffer::new(nz!(2), nz!(44100), vec![1.0, 2.0, 1.0, 2.0]);
        let s2 = SamplesBuffer::new(nz!(1), nz!(44100), vec![3.0; 2]);
        let s3 = SamplesBuffer::new(nz!(2), nz!(44100), vec![4.0, 5.0, 4.0, 5.0]);

        let combined = (s1, s2, s3).try_combine_channels().unwrap();
        assert_eq!(combined.channels(), nz!(5));
        assert_eq!(
            combined.collect_vec(),
            vec![1.0, 2.0, 3.0, 4.0, 5.0, 1.0, 2.0, 3.0, 4.0, 5.0]
        )
    }

    #[test]
    fn combine_array() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(44100), vec![1.0, 3.0]);
        let s2 = SamplesBuffer::new(nz!(1), nz!(44100), vec![2.0, 4.0, 5.0, 6.0]);

        assert_eq!(
            vec![1.0, 2.0, 3.0, 4.0],
            [s1, s2].try_combine_channels().unwrap().collect::<Vec<_>>()
        );
    }

    #[test]
    fn refuse_mismatch() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(48000), vec![1.0, 3.0]);
        let s2 = SamplesBuffer::new(nz!(1), nz!(44100), vec![2.0, 4.0, 5.0, 6.0]);

        assert!([s1, s2].try_combine_channels().is_err());
    }
}
