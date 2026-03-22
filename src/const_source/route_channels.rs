use std::u16;

use crate::ConstSource;

mod helpful_compile_time_error;

pub trait IntoCombinedChannels<const SR: u32, const CH1: u16, const CH2: u16> {
    type CombinerSource<const CH: u16>: ConstSource<SR, CH>;

    fn combine_channels<const CH: u16>(self) -> Self::CombinerSource<CH>;
}

pub struct TupleChannelCombiner<
    const SR: u32,
    const CH: u16,
    const CH1: u16,
    const CH2: u16,
    S1,
    S2,
> {
    inner: (S1, S2),
}

// SPAM a ton of these for ALL the numbers using macros❤️
impl<const SR: u32, const CH1: u16, const CH2: u16, const CH_OUT: u16, S1, S2>
    ConstSource<SR, CH_OUT> for TupleChannelCombiner<SR, CH_OUT, CH1, CH2, S1, S2>
where
    S1: ConstSource<SR, CH1>,
    S2: ConstSource<SR, CH2>,
{
    fn total_duration(&self) -> Option<std::time::Duration> {
        const { assert!(CH1 + CH2 == CH_OUT) }
        todo!()
    }
}

impl<const SR: u32, const CH: u16, const CH1: u16, const CH2: u16, S1, S2> Iterator
    for TupleChannelCombiner<SR, CH, CH1, CH2, S1, S2>
where
    S1: ConstSource<SR, CH1>,
    S2: ConstSource<SR, CH2>,
{
    type Item = rodio::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl<const SR: u32, const CH1: u16, const CH2: u16, S1, S2> IntoCombinedChannels<SR, CH1, CH2>
    for (S1, S2)
where
    S1: ConstSource<SR, CH1>,
    S2: ConstSource<SR, CH2>,
{
    type CombinerSource<const CH: u16> = TupleChannelCombiner<SR, CH, CH1, CH2, S1, S2>;

    fn combine_channels<const CH: u16>(self) -> Self::CombinerSource<CH> {
        helpful_compile_time_error::assert_channel_counts::<CH, CH1, CH2>();
        todo!()
    }
}


#[cfg(test)]
mod tests {
    use crate::const_source::buffer::SamplesBuffer;

    use super::*;

    #[test]
    fn hi() {
        // channel_count_mismatch(15);
    }

    #[test]
    fn compiles() {
        let s1 = SamplesBuffer::<44100, 1>::new(vec![]);
        let s2 = SamplesBuffer::<44100, 1>::new(vec![]);

        let combined = (s1, s2).combine_channels::<2>();
        assert_eq!(combined.channels().get(), 2);
    }
}
