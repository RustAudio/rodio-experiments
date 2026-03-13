use std::time::Duration;

use itertools::Itertools;

use super::IntoQueued;
use crate::ConstSource;

#[derive(Clone, Debug)]
pub struct QueuedVec<const SR: u32, const CH: u16, S> {
    sources: Vec<S>,
    current: usize,
}

impl<const SR: u32, const CH: u16, S: ConstSource<SR, CH>> Iterator for QueuedVec<SR, CH, S> {
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

impl<const SR: u32, const CH: u16, S: ConstSource<SR, CH>> ConstSource<SR, CH>
    for QueuedVec<SR, CH, S>
{
    fn channels(&self) -> rodio::ChannelCount {
        self.sources[0].channels()
    }
    fn sample_rate(&self) -> rodio::SampleRate {
        self.sources[0].sample_rate()
    }
    fn total_duration(&self) -> Option<std::time::Duration> {
        self.sources
            .iter()
            .map(ConstSource::total_duration)
            .fold_options(Duration::ZERO, |sum, s| sum + s)
    }
}

impl<const SR: u32, const CH: u16, S: ConstSource<SR, CH>> IntoQueued<SR, CH> for Vec<S> {
    type QueuedSource = QueuedVec<SR, CH, S>;

    fn into_list(self) -> Self::QueuedSource {
        Self::QueuedSource {
            sources: self,
            current: 0,
        }
    }
}
