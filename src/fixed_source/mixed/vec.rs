use std::time::Duration;

use itertools::Itertools;

use super::super::MaybeConvert;
use super::super::convert_if_needed;
use super::ParamsMismatch;
use crate::FixedSource;
use crate::fixed_source::mixed::IntoMixed;
use crate::{ChannelCount, SampleRate};

#[derive(Clone, Debug)]
pub struct MixedVec<S> {
    sources: Vec<S>,
}

impl<S: FixedSource> Iterator for MixedVec<S> {
    type Item = crate::Sample;
    fn next(&mut self) -> Option<Self::Item> {
        let (summed, sum) = self
            .sources
            .iter_mut()
            .fold((0, None), |(summed, sum), source| {
                let sample = source.next().map(|s| s as f64);
                let summed = summed + sample.is_some() as usize;
                let sum = match [sum, sample] {
                    [Some(sum), None] => Some(sum),
                    [Some(sum), Some(sample)] => Some(sum + sample),
                    [None, Some(sample)] => Some(sample),
                    [None, None] => None,
                };
                (summed, sum)
            });
        sum.map(|sum| sum / summed as f64).map(|sum| sum as f32)
    }
}

impl<S: FixedSource> FixedSource for MixedVec<S> {
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

impl<S: FixedSource> IntoMixed for Vec<S> {
    type TryMixedSource = MixedVec<S>;
    type IntoMixedSource = MixedVec<MaybeConvert<S>>;

    fn try_into_mixed(self) -> Result<Self::TryMixedSource, ParamsMismatch> {
        let mut list = self.iter().map(|s| (s.sample_rate(), s.channels()));
        let Some(first) = list.next() else {
            return Ok(MixedVec { sources: self });
        };
        if let Some((pos, (sample_rate_right, channel_count_right))) =
            list.find_position(|params| *params != first)
        {
            return Err(ParamsMismatch {
                index_of_first_mismatch: pos,
                sample_rate_left: first.0,
                channel_count_left: first.1,
                sample_rate_right,
                channel_count_right,
            });
        }
        Ok(Self::TryMixedSource { sources: self })
    }
    fn into_mixed_converted(
        self,
        sample_rate: SampleRate,
        channels: ChannelCount,
    ) -> Self::IntoMixedSource {
        Self::IntoMixedSource {
            sources: self
                .into_iter()
                .map(|s| convert_if_needed(s, sample_rate, channels))
                .collect(),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::fixed_source::buffer::SamplesBuffer;
    use crate::nz;

    use super::*;

    #[test]
    fn mixed() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(1), vec![1.0, 2.0, 3.0]);
        let s2 = SamplesBuffer::new(nz!(1), nz!(1), vec![4.0, 5.0, 6.0]);

        assert_eq!(
            vec![2.5, 3.5, 4.5],
            vec![s1, s2].try_into_mixed().unwrap().collect::<Vec<_>>()
        );
    }
}
