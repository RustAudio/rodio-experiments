use std::ops::Add;
use std::time::Duration;

use super::IntoQueued;
use crate::common::queued_next_body;
use crate::ConstSource;

#[derive(Clone, Debug)]
pub struct QueuedVec<const SR: u32, const CH: u16, S> {
    sources: Vec<S>,
    current: usize,
}

impl<const SR: u32, const CH: u16, S: ConstSource<SR, CH>> Iterator for QueuedVec<SR, CH, S> {
    type Item = crate::Sample;
    fn next(&mut self) -> Option<Self::Item> {
        queued_next_body! {self}
    }
}

impl<const SR: u32, const CH: u16, S: ConstSource<SR, CH>> ConstSource<SR, CH>
    for QueuedVec<SR, CH, S>
{
    fn total_duration(&self) -> Option<std::time::Duration> {
        self.sources
            .iter()
            .filter_map(ConstSource::total_duration)
            .reduce(Duration::add)
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
