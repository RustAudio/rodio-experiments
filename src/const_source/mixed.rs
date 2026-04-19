use std::time::Duration;

use itertools::Itertools;

use super::list_of_sources::ListOfSources;
use crate::ConstSource;

pub struct Mixed<const SR: u32, const CH: u16, T> {
    inner: T,
}

impl<const SR: u32, const CH: u16, T: ListOfSources<SR, CH>> ConstSource<SR, CH>
    for Mixed<SR, CH, T>
{
    fn total_duration(&self) -> Option<std::time::Duration> {
        (0..self.inner.len())
            .into_iter()
            .map(|idx| self.inner.total_duration(idx))
            .fold_options(Duration::ZERO, |max, dur| max.max(dur))
    }
}

impl<const SR: u32, const CH: u16, T: ListOfSources<SR, CH>> Iterator for Mixed<SR, CH, T> {
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

pub trait IntoMixed<const SR: u32, const CH: u16> {
    fn into_mixed(self) -> Mixed<SR, CH, Self>
    where
        Self: ListOfSources<SR, CH> + Sized;
}

impl<const SR: u32, const CH: u16, T> IntoMixed<SR, CH> for T {
    fn into_mixed(self) -> Mixed<SR, CH, Self>
    where
        Self: ListOfSources<SR, CH> + Sized,
    {
        Mixed { inner: self }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::fixed_source::buffer::SamplesBuffer;
    use crate::fixed_source::mixed::IntoMixed;
    use crate::nz;

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
