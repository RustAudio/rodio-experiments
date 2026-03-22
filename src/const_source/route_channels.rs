use std::u16;

use crate::ConstSource;
use crate::common::for_in_tuple;

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
    current: u16,
}

impl<const SR: u32, const CH1: u16, const CH2: u16, const CH_OUT: u16, S1, S2>
    ConstSource<SR, CH_OUT> for TupleChannelCombiner<SR, CH_OUT, CH1, CH2, S1, S2>
where
    S1: ConstSource<SR, CH1>,
    S2: ConstSource<SR, CH2>,
{
    fn total_duration(&self) -> Option<std::time::Duration> {
        let mut max = None;
        for_in_tuple! { 0,1;
            for item in self.inner; {
                let dur= item.total_duration();
                max = match [max, dur] {
                    [Some(max), None] => Some(max),
                    [Some(max), Some(dur)] => Some(max.max(dur)),
                    [None, Some(dur)] => Some(dur),
                    [None, None] => None,
                };
            }
        }
        max
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
        #![expect(
            unused,
            reason = "channel += item.channels().get is not used in the last iteration"
        )]

        let mut channel = 0;
        for_in_tuple! { 0, 1;
            for mut item in self.inner; {
                if (channel..(channel+item.channels().get())).contains(&self.current) {
                    self.current += 1;
                    self.current %= CH;
                    return item.next();
                } else {
                    channel += item.channels().get()
                }
            }
        }

        None
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
        TupleChannelCombiner {
            inner: self,
            current: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::const_source::buffer::SamplesBuffer;

    use super::*;

    #[test]
    fn channel_count_adds() {
        let s1 = SamplesBuffer::<44100, 1>::new(vec![]);
        let s2 = SamplesBuffer::<44100, 1>::new(vec![]);

        let combined = (s1, s2).combine_channels::<2>();
        assert_eq!(combined.channels().get(), 2);
    }

    #[test]
    fn stereo_samples_alternate() {
        let s1 = SamplesBuffer::<44100, 1>::new(vec![1.0; 2]);
        let s2 = SamplesBuffer::<44100, 1>::new(vec![2.0; 2]);

        let combined = (s1, s2).combine_channels::<2>();
        assert_eq!(combined.collect_vec(), vec![1.0, 2.0, 1.0, 2.0])
    }

    // #[test]
    // fn five_channel_audio_samples_alternate() {
    //     let s1 = SamplesBuffer::<44100, 2>::new(vec![1.0, 2.0, 1.0, 2.0]);
    //     let s2 = SamplesBuffer::<44100, 1>::new(vec![3.0; 2]);
    //     let s3 = SamplesBuffer::<44100, 2>::new(vec![4.0, 5.0, 4.0, 5.0]);
    //
    //     let combined = (s1, s2, s3).combine_channels::<2>();
    //     assert_eq!(combined.collect_vec(), vec![1.0, 2.0, 1.0, 2.0])
    // }
}
