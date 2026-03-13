use std::time::Duration;

use itertools::Itertools;

use super::ParamsMismatch;
use crate::FixedSource;
use crate::fixed_source::list::tuple::{MaybeConvert, convert_if_needed};
use crate::{ChannelCount, SampleRate};

#[derive(Clone, Debug)]
pub struct VecList<S> {
    sources: Vec<S>,
    current: usize,
}
impl<S: FixedSource> Iterator for VecList<S> {
    type Item = crate::Sample;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let sample = self.sources[self.current as usize].next();
            if sample.is_some() {
                return sample;
            } else {
                self.current += 1;
                if self.current >= self.sources.len() {
                    return None;
                }
                continue;
            }
        }
    }
}

impl<S: FixedSource> FixedSource for VecList<S> {
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

impl<S: FixedSource> super::IntoList for Vec<S> {
    type TryListSource = VecList<S>;
    type IntoListSource = VecList<MaybeConvert<S>>;

    fn try_into_list(self) -> Result<Self::TryListSource, ParamsMismatch> {
        let mut list = self.iter().map(|s| (s.sample_rate(), s.channels()));
        let Some(first) = list.next() else {
            return Ok(VecList {
                sources: self,
                current: 0,
            });
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
        Ok(Self::TryListSource {
            sources: self,
            current: 0,
        })
    }
    fn into_list_converted(
        self,
        sample_rate: SampleRate,
        channels: ChannelCount,
    ) -> Self::IntoListSource {
        Self::IntoListSource {
            sources: self
                .into_iter()
                .map(|s| convert_if_needed(s, sample_rate, channels))
                .collect(),
            current: 0,
        }
    }
}
