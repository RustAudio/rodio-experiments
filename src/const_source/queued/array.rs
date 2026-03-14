use std::ops::Add;
use std::time::Duration;

use super::IntoQueued;
use crate::ConstSource;
use crate::common::queued_next_body;

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
        queued_next_body! {self}
    }
}

impl<const N: usize, const SR: u32, const CH: u16, S: ConstSource<SR, CH>> ConstSource<SR, CH>
    for QueuedArray<N, SR, CH, S>
{
    fn total_duration(&self) -> Option<std::time::Duration> {
        self.sources
            .iter()
            .filter_map(ConstSource::total_duration)
            .reduce(Duration::add)
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
