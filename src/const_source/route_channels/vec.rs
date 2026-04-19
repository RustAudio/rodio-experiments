use crate::ConstSource;
use crate::common::channel_combined_next_body;
use crate::const_source::route_channels::CombineChannels;

#[derive(Clone, Debug)]
pub struct CombinedChannels<const SR: u32, const CH_IN: u16, const CH_OUT: u16, S> {
    sources: Vec<S>,
    current: u16,
}

impl<const SR: u32, const CH_IN: u16, const CH_OUT: u16, S: ConstSource<SR, CH_IN>> Iterator
    for CombinedChannels<SR, CH_IN, CH_OUT, S>
{
    type Item = crate::Sample;
    fn next(&mut self) -> Option<Self::Item> {
        channel_combined_next_body! {self}
    }
}

impl<const SR: u32, const CH_IN: u16, const CH_OUT: u16, S: ConstSource<SR, CH_IN>>
    ConstSource<SR, CH_OUT> for CombinedChannels<SR, CH_IN, CH_OUT, S>
{
    fn total_duration(&self) -> Option<std::time::Duration> {
        self.sources
            .iter()
            .filter_map(ConstSource::total_duration)
            .reduce(Ord::min)
    }
}

#[derive(thiserror::Error, Debug, Clone)]
#[error(
    "Wrong output channel count ({ch_out}. It should be the \
    sum of the inputs ({len} * {ch_in})"
)]
pub struct ChannelCountMismatch {
    ch_out: u16,
    ch_in: u16,
    len: usize,
}

impl<const SR: u32, const CH_IN: u16, S: ConstSource<SR, CH_IN>> CombineChannels<SR, CH_IN>
    for Vec<S>
{
    type CombinerSource<const CH: u16> = CombinedChannels<SR, CH_IN, CH, S>;
    type Result<T> = Result<T, ChannelCountMismatch>;

    fn combine_channels<const CH: u16>(self) -> Self::Result<Self::CombinerSource<CH>> {
        if CH as usize != CH_IN as usize * self.len() {
            Err(ChannelCountMismatch {
                ch_out: CH,
                ch_in: CH_IN,
                len: self.len(),
            })
        } else {
            Ok(Self::CombinerSource {
                sources: self,
                current: 0,
            })
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
            vec![s1, s2]
                .combine_channels::<2>()
                .unwrap()
                .collect::<Vec<_>>()
        );
    }
}
