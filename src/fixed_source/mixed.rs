use std::time::Duration;

use itertools::Itertools;

use crate::FixedSource;
use crate::{ChannelCount, SampleRate};

use crate::fixed_source::list_of_sources::{ConvertibleListOfSources, ListOfSources};

pub struct Mixed<T> {
    inner: T,
}

impl<T: ListOfSources> FixedSource for Mixed<T> {
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
            .fold_options(Duration::ZERO, |max, dur| max.max(dur))
    }
}

impl<T: ListOfSources> Iterator for Mixed<T> {
    type Item = crate::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        let (sum, summed) = (0..self.inner.len())
            .into_iter()
            .filter_map(|idx| self.inner.next(idx))
            .map(|sample| sample as f64)
            .zip((1usize..).into_iter())
            .reduce(|(sum, _), (sample, summed)| (sum + sample, summed))?;
        Some((sum / summed as f64) as crate::Float)
    }
}

pub trait IntoMixed {
    fn try_into_mixed(self) -> Result<Mixed<Self>, ParamsMismatch>
    where
        Self: ListOfSources + Sized;
    fn into_mixed_converted(
        self,
        sample_rate: SampleRate,
        channels: ChannelCount,
    ) -> Mixed<Self::Converted>
    where
        Self: ListOfSources + ConvertibleListOfSources + Sized;
}

impl<T> IntoMixed for T {
    fn try_into_mixed(self) -> Result<Mixed<Self>, ParamsMismatch>
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

        Ok(Mixed { inner: self })
    }

    fn into_mixed_converted(
        self,
        sample_rate: SampleRate,
        channels: ChannelCount,
    ) -> Mixed<<Self as ConvertibleListOfSources>::Converted>
    where
        Self: ListOfSources + ConvertibleListOfSources + Sized,
    {
        Mixed {
            inner: self.converted(sample_rate, channels),
        }
    }
}

pub type ParamsMismatch = super::queued::ParamsMismatch;

#[cfg(test)]
pub(crate) mod tests {
    use crate::fixed_source::buffer::SamplesBuffer;
    use crate::nz;

    use super::*;

    #[test]
    fn mixed2() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(1), vec![1.0, 2.0, 3.0]);
        let s2 = SamplesBuffer::new(nz!(1), nz!(1), vec![4.0, 5.0, 6.0]);

        assert_eq!(
            vec![2.5, 3.5, 4.5],
            (s1, s2).try_into_mixed().unwrap().collect::<Vec<_>>()
        );
    }

    #[test]
    fn mixed3() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(1), vec![1.0]);
        let s2 = SamplesBuffer::new(nz!(1), nz!(1), vec![2.0]);
        let s3 = SamplesBuffer::new(nz!(1), nz!(1), vec![3.0]);

        assert_eq!(
            vec![2.],
            (s1, s2, s3).try_into_mixed().unwrap().collect::<Vec<_>>()
        );
    }

    #[test]
    fn mixed3_mismatch() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(1), vec![1.0]);
        let s2 = SamplesBuffer::new(nz!(2), nz!(2), vec![2.0]);
        let s3 = SamplesBuffer::new(nz!(1), nz!(1), vec![3.0]);

        assert!((s1, s2, s3).try_into_mixed().is_err());
    }

    #[test]
    fn into_mixed_converted_all_rechanneled() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(1), vec![1.0; 1]);
        let s2 = SamplesBuffer::new(nz!(3), nz!(1), vec![2.0; 3]);
        let s3 = SamplesBuffer::new(nz!(2), nz!(1), vec![3.0; 2]);

        assert_eq!(
            vec![2.0],
            (s1, s2, s3)
                .into_mixed_converted(nz!(1), nz!(1))
                .collect::<Vec<_>>()
        );
    }
}
