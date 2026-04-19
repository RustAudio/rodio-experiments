use std::time::Duration;

use itertools::Itertools;

use crate::ConstSource;
use crate::const_source::list_of_sources::ListOfSources;

#[derive(Debug)]
pub struct Queued<const SR: u32, const CH: u16, T> {
    inner: T,
    current: u8,
}

impl<const SR: u32, const CH: u16, T: ListOfSources<SR, CH>> ConstSource<SR, CH>
    for Queued<SR, CH, T>
{
    fn total_duration(&self) -> Option<std::time::Duration> {
        (0..self.inner.len())
            .map(|idx| self.inner.total_duration(idx))
            .fold_options(Duration::ZERO, |sum, dur| sum + dur)
    }
}

impl<const SR: u32, const CH: u16, T: ListOfSources<SR, CH>> Iterator for Queued<SR, CH, T> {
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

pub trait IntoQueued<const SR: u32, const CH: u16> {
    fn into_queued(self) -> Queued<SR, CH, Self>
    where
        Self: ListOfSources<SR, CH> + Sized;
}

impl<const SR: u32, const CH: u16, T> IntoQueued<SR, CH> for T {
    fn into_queued(self) -> Queued<SR, CH, Self>
    where
        Self: ListOfSources<SR, CH> + Sized,
    {
        Queued {
            inner: self,
            current: 0,
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::const_source::buffer::SamplesBuffer;

    use super::*;

    #[test]
    fn queue2() {
        let s1 = SamplesBuffer::<44100, 1>::new(vec![1.0, 2.0, 3.0]);
        let s2 = SamplesBuffer::<44100, 1>::new(vec![4.0, 5.0, 6.0]);

        assert_eq!(
            vec![1., 2., 3., 4., 5., 6.],
            (s1, s2).into_queued().collect::<Vec<_>>()
        );
    }

    #[test]
    fn duration3() {
        let s1 = SamplesBuffer::<1, 1>::new(vec![1.0; 3]);
        let s2 = SamplesBuffer::<1, 1>::new(vec![1.0; 1]);
        let s3 = SamplesBuffer::<1, 1>::new(vec![1.0; 2]);

        let mut queued = (s1, s2, s3).into_queued();

        assert_eq!(Some(Duration::from_secs(6)), queued.total_duration());

        let _ = queued.by_ref().take(4).count();
        assert_eq!(Some(Duration::from_secs(6)), queued.total_duration());
    }
}
