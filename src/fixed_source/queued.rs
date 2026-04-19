use std::fmt::Display;
use std::time::Duration;

use itertools::Itertools;

use crate::FixedSource;
use crate::fixed_source::list_of_sources::{ConvertibleListOfSources, ListOfSources};
use crate::{ChannelCount, SampleRate};

#[derive(Debug)]
pub struct Queued<T> {
    inner: T,
    current: u8,
}

impl<T: ListOfSources> FixedSource for Queued<T> {
    fn channels(&self) -> ChannelCount {
        self.inner.channels(0)
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

impl<T: ListOfSources> Iterator for Queued<T> {
    type Item = crate::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(sample) = self.inner.next(self.current as usize) {
            return Some(sample);
        }

        loop {
            self.current += 1;
            if self.current as usize >= self.inner.len() {
                return None;
            }

            if let Some(sample) = self.inner.next(self.current as usize) {
                return Some(sample);
            }
        }
    }
}

pub trait IntoQueued {
    fn try_into_queued(self) -> Result<Queued<Self>, ParamsMismatch>
    where
        Self: ListOfSources + Sized;
    fn into_queued_converted(
        self,
        sample_rate: SampleRate,
        channels: ChannelCount,
    ) -> Queued<Self::Converted>
    where
        Self: ListOfSources + ConvertibleListOfSources + Sized;
}

impl<T> IntoQueued for T {
    fn try_into_queued(self) -> Result<Queued<Self>, ParamsMismatch>
    where
        Self: ListOfSources + Sized,
    {
        let left = (self.sample_rate(0), self.channels(0));

        for i in 0..self.len() {
            let right = (self.sample_rate(i), self.channels(i));
            if left != right {
                return Err(ParamsMismatch {
                    index_of_first_mismatch: i,
                    sample_rate_left: left.0,
                    channel_count_left: left.1,
                    sample_rate_right: right.0,
                    channel_count_right: right.1,
                });
            }
        }

        Ok(Queued {
            inner: self,
            current: 0,
        })
    }

    fn into_queued_converted(
        self,
        sample_rate: SampleRate,
        channels: ChannelCount,
    ) -> Queued<<Self as ConvertibleListOfSources>::Converted>
    where
        Self: ListOfSources + ConvertibleListOfSources + Sized,
    {
        Queued {
            inner: self.converted(sample_rate, channels),
            current: 0,
        }
    }
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

#[cfg(test)]
pub(crate) mod tests {
    use std::time::Duration;

    use crate::FixedSource;
    use crate::fixed_source::buffer::SamplesBuffer;
    use crate::fixed_source::queued::{IntoQueued, ParamsMismatch};
    use crate::nz;

    #[test]
    fn queued_vec() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(1), vec![1.0, 2.0, 3.0]);
        let s2 = SamplesBuffer::new(nz!(1), nz!(1), vec![4.0, 5.0, 6.0]);

        assert_eq!(
            vec![1., 2., 3., 4., 5., 6.],
            vec![s1, s2].try_into_queued().unwrap().collect::<Vec<_>>()
        );
    }

    #[test]
    fn queued2() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(1), vec![1.0, 2.0, 3.0]);
        let s2 = SamplesBuffer::new(nz!(1), nz!(1), vec![4.0, 5.0, 6.0]);

        assert_eq!(
            vec![1., 2., 3., 4., 5., 6.],
            (s1, s2).try_into_queued().unwrap().collect::<Vec<_>>()
        );
    }

    #[test]
    fn queued3() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(1), vec![1.0]);
        let s2 = SamplesBuffer::new(nz!(1), nz!(1), vec![2.0]);
        let s3 = SamplesBuffer::new(nz!(1), nz!(1), vec![3.0]);

        assert_eq!(
            vec![1., 2., 3.],
            (s1, s2, s3).try_into_queued().unwrap().collect::<Vec<_>>()
        );
    }

    #[test]
    fn queued3_mismatch() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(1), vec![1.0]);
        let s2 = SamplesBuffer::new(nz!(2), nz!(2), vec![2.0]);
        let s3 = SamplesBuffer::new(nz!(1), nz!(1), vec![3.0]);

        assert_eq!(
            ParamsMismatch {
                index_of_first_mismatch: 1,
                sample_rate_left: nz!(1),
                channel_count_left: nz!(1),
                sample_rate_right: nz!(2),
                channel_count_right: nz!(2),
            },
            (s1, s2, s3).try_into_queued().unwrap_err()
        );
    }

    #[test]
    fn into_queued_converted_all_rechanneled() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(1), vec![1.0; 1]);
        let s2 = SamplesBuffer::new(nz!(3), nz!(1), vec![2.0; 3]);
        let s3 = SamplesBuffer::new(nz!(2), nz!(1), vec![3.0; 2]);

        assert_eq!(
            vec![1.0, 2.0, 3.0],
            (s1, s2, s3)
                .into_queued_converted(nz!(1), nz!(1))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn into_queued_converted_all_rechanneled_duration() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(1), vec![1.0; 1]); // 1s
        let s2 = SamplesBuffer::new(nz!(3), nz!(1), vec![2.0; 3]); // 1s
        let s3 = SamplesBuffer::new(nz!(2), nz!(1), vec![3.0; 4]); // 2s

        assert_eq!(
            Some(Duration::from_secs(1 + 1 + 2)),
            (s1, s2, s3)
                .into_queued_converted(nz!(1), nz!(1))
                .total_duration()
        );
    }
}
