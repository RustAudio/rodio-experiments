use super::IntoMixed;
use crate::ConstSource;
use crate::common::mixed_next_body;

#[derive(Clone, Debug)]
pub struct MixedVec<const SR: u32, const CH: u16, S> {
    sources: Vec<S>,
}

impl<const SR: u32, const CH: u16, S: ConstSource<SR, CH>> Iterator for MixedVec<SR, CH, S> {
    type Item = crate::Sample;
    fn next(&mut self) -> Option<Self::Item> {
        mixed_next_body! {self}
    }
}

impl<const SR: u32, const CH: u16, S: ConstSource<SR, CH>> ConstSource<SR, CH>
    for MixedVec<SR, CH, S>
{
    fn total_duration(&self) -> Option<std::time::Duration> {
        self.sources
            .iter()
            .filter_map(ConstSource::total_duration)
            .reduce(Ord::max)
    }
}

impl<const SR: u32, const CH: u16, S: ConstSource<SR, CH>> IntoMixed<SR, CH> for Vec<S> {
    type MixedSource = MixedVec<SR, CH, S>;

    fn into_mixed(self) -> Self::MixedSource {
        Self::MixedSource { sources: self }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::IntoMixed;
    use crate::const_source::buffer::SamplesBuffer;

    #[test]
    fn mixed() {
        let s1 = SamplesBuffer::<44100, 1>::new(vec![1.0, 2.0, 3.0]);
        let s2 = SamplesBuffer::<44100, 1>::new(vec![4.0, 5.0]);

        assert_eq!(
            vec![2.5, 3.5, 3.0],
            vec![s1, s2].into_mixed().collect::<Vec<_>>()
        );
    }
}
