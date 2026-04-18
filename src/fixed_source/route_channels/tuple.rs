use itertools::Itertools;

use crate::ChannelCount;
use crate::FixedSource;
use crate::common::SampleRate;
use crate::fixed_source::tuple_access::TupleSourceAccess;

use super::{CombineChannels, CombineChannelsError};
use std::num::NonZeroU16;
use std::time::Duration;

#[derive(Debug)]
pub struct ChannelCombiningTuple<T> {
    inner: T,
    channels: ChannelCount,
    current: u16,
}

impl<T: TupleSourceAccess> FixedSource for ChannelCombiningTuple<T> {
    fn channels(&self) -> ChannelCount {
        self.channels
    }

    fn sample_rate(&self) -> SampleRate {
        self.inner.sample_rate(0)
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        (0..T::LEN)
            .into_iter()
            .map(|idx| self.inner.total_duration(idx))
            .fold_options(Duration::ZERO, |sum, dur| sum + dur)
    }
}

impl<T: TupleSourceAccess> Iterator for ChannelCombiningTuple<T> {
    type Item = crate::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        let channels = self.channels().get();
        let mut channel = 0;
        for i in 0..T::LEN {
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

impl<T: TupleSourceAccess> CombineChannels for T {
    type TryCombinerSource = ChannelCombiningTuple<T>;

    fn try_combine_channels(self) -> Result<Self::TryCombinerSource, CombineChannelsError> {
        // let channels = super::verify_params_and_determine_channel_count(self.as_slice())?;
        let channels = todo!();

        Ok(Self::TryCombinerSource {
            channels,
            inner: self,
            current: 0,
        })
    }
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
}
