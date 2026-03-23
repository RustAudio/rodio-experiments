use crate::ConstSource;
use crate::common::channel_combined_next_body;
use crate::const_source::route_channels::CombineChannels;

mod helpful_compile_time_error;

#[derive(Clone, Debug)]
pub struct ChannelCombiningArray<
    const N: usize,
    const SR: u32,
    const CH_IN: u16,
    const CH_OUT: u16,
    S,
> {
    sources: [S; N],
    current: u16,
}

impl<const N: usize, const SR: u32, const CH_IN: u16, const CH_OUT: u16, S: ConstSource<SR, CH_IN>>
    Iterator for ChannelCombiningArray<N, SR, CH_IN, CH_OUT, S>
{
    type Item = crate::Sample;
    fn next(&mut self) -> Option<Self::Item> {
        channel_combined_next_body! {self}
    }
}

impl<const N: usize, const SR: u32, const CH_IN: u16, const CH_OUT: u16, S: ConstSource<SR, CH_IN>>
    ConstSource<SR, CH_OUT> for ChannelCombiningArray<N, SR, CH_IN, CH_OUT, S>
{
    fn total_duration(&self) -> Option<std::time::Duration> {
        self.sources
            .iter()
            .filter_map(ConstSource::total_duration)
            .reduce(Ord::max)
    }
}

impl<const N: usize, const SR: u32, const CH_IN: u16, S: ConstSource<SR, CH_IN>>
    CombineChannels<SR, CH_IN> for [S; N]
{
    type CombinerSource<const CH: u16> = ChannelCombiningArray<N, SR, CH_IN, CH, S>;
    type Result<T> = T;

    fn combine_channels<const CH: u16>(self) -> Self::Result<Self::CombinerSource<CH>> {
        const {
            assert!(
                CH == CH_IN * N as u16,
                "{}",
                helpful_compile_time_error::channel_count_mismatch::<CH, CH_IN, N>()
            )
        }
        Self::CombinerSource {
            sources: self,
            current: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::const_source::buffer::SamplesBuffer;

    use super::*;

    #[test]
    fn combine_vec() {
        let s1 = SamplesBuffer::<44100, 1>::new(vec![1.0, 3.0]);
        let s2 = SamplesBuffer::<44100, 1>::new(vec![2.0, 4.0, 5.0, 6.0]);

        assert_eq!(
            vec![1.0, 2.0, 3.0, 4.0],
            [s1, s2].combine_channels::<2>().collect::<Vec<_>>()
        );
    }
}
