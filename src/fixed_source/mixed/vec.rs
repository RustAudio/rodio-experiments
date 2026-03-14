use itertools::Itertools;

use super::super::MaybeConvert;
use super::super::convert_if_needed;
use super::ParamsMismatch;
use crate::FixedSource;
use crate::common::check_params_for_list;
use crate::common::mixed_next_body;
use crate::fixed_source::mixed::IntoMixed;
use crate::{ChannelCount, SampleRate};

#[derive(Clone, Debug)]
pub struct MixedVec<S> {
    sources: Vec<S>,
}

impl<S: FixedSource> Iterator for MixedVec<S> {
    type Item = crate::Sample;
    fn next(&mut self) -> Option<Self::Item> {
        mixed_next_body! {self}
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
            .filter_map(FixedSource::total_duration)
            .reduce(|max, s| max.max(s))
    }
}

impl<S: FixedSource> IntoMixed for Vec<S> {
    type TryMixedSource = MixedVec<S>;
    type IntoMixedSource = MixedVec<MaybeConvert<S>>;

    fn try_into_mixed(self) -> Result<Self::TryMixedSource, ParamsMismatch> {
        check_params_for_list! {self}
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
