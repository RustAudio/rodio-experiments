use std::time::Duration;

use itertools::Itertools;

use super::IntoQueued;
use crate::ConstSource;

#[derive(Clone, Debug)]
pub struct QueuedArray<const N: usize, const SR: u32, const CH: u16, S> {
    sources: [S; N],
    current: usize,
}

impl<const N: usize, const SR: u32, const CH: u16, S: ConstSource<SR, CH>> Iterator
    for QueuedArray<N, SR, CH, S>
{
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

impl<const N: usize, const SR: u32, const CH: u16, S: ConstSource<SR, CH>> ConstSource<SR, CH>
    for QueuedArray<N, SR, CH, S>
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

impl<const N: usize, const SR: u32, const CH: u16, S: ConstSource<SR, CH>> IntoQueued<SR, CH>
    for [S; N]
{
    type QueuedSource = QueuedArray<N, SR, CH, S>;

    fn into_list(self) -> Self::QueuedSource {
        Self::QueuedSource {
            sources: self,
            current: 0,
        }
    }
}
