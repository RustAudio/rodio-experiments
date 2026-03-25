use crate::ChannelCount;
use crate::FixedSource;

use super::{CombineChannels, CombineChannelsError};
use crate::common::for_in_tuple;
use std::num::NonZeroU16;

macro_rules! tuple_impl {
    ($list:ident; $($generics:ident),+; $($count:tt),*) => {
        #[derive(Clone, Debug)]
        pub struct $list<$($generics),+> {
            sources: ($($generics),+),
            channels: ChannelCount,
            current: u16,
        }

        impl<$($generics: FixedSource),+> Iterator for $list<$($generics),+> {
            type Item = crate::Sample;

            fn next(&mut self) -> Option<Self::Item> {
                #![expect(unused, reason = "channel += item.channels().get() \
                    is not used in the last iteration")]

                let channels = self.channels().get();
                let mut channel = 0;
                for_in_tuple! { $($count),*;
                    for mut item in self.sources; {
                        if (channel..(channel + item.channels().get())).contains(&self.current) {
                            self.current += 1;
                            self.current %= channels;
                            return item.next();
                        } else {
                            channel += item.channels().get()
                        }
                    } // for every item body
                }

                None
            } // next
        } // impl iter

        impl<$($generics: FixedSource),+> FixedSource for $list<$($generics),+> {
            fn channels(&self) -> rodio::ChannelCount {
                self.channels
            }

            fn sample_rate(&self) -> rodio::SampleRate {
                self.sources.0.sample_rate()
            }

            fn total_duration(&self) -> Option<std::time::Duration> {
                let mut max = None;
                for_in_tuple! { $($count),*;
                    for item in self.sources; {
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
        } // impl FixedSource

        impl<$($generics: FixedSource),+> CombineChannels for ($($generics),+) {
            type TryCombinerSource = $list<$($generics),+>;

            fn try_combine_channels(self) -> Result<Self::TryCombinerSource, CombineChannelsError> {
                #![expect(unused, reason = "index += 1 is not used in the last iteration")]

                let left = self.0.sample_rate();
                let mut channels = 0u32;
                let mut index = 0;

                for_in_tuple! { $($count),*;
                    for source in self; {
                        channels += source.channels().get() as u32;
                        let right = source.sample_rate();
                        if left != right {
                            return Err(CombineChannelsError::SampleRateMismatch {
                                index_of_first_mismatch: index as usize,
                                sample_rate_left: left,
                                sample_rate_right: right,
                            });
                        }
                        index += 1;
                    }
                }

                let channels: u16 = channels
                    .try_into()
                    .map_err(|_| CombineChannelsError::TooManyChannels(channels))?;
                let channels = NonZeroU16::new(channels).expect("Sum of NonZero items can not be zero");

                Ok(Self::TryCombinerSource {
                    sources: self,
                    channels,
                    current: 0,
                })
            }
        } // impl CombineChannels
    }; // transcriber
}

tuple_impl! {ChannelCombining2Tuple; S1,S2; 0,1}
tuple_impl! {ChannelCombining3Tuple; S1,S2,S3; 0,1,2}
tuple_impl! {ChannelCombining4Tuple; S1,S2,S3,S4; 0,1,2,3}
tuple_impl! {ChannelCombining5Tuple; S1,S2,S3,S4,S5; 0,1,2,3,4}
tuple_impl! {ChannelCombining6Tuple; S1,S2,S3,S4,S5,S6; 0,1,2,3,4,5}
tuple_impl! {ChannelCombining7Tuple; S1,S2,S3,S4,S5,S6,S7; 0,1,2,3,4,5,6}
tuple_impl! {ChannelCombining8Tuple; S1,S2,S3,S4,S5,S6,S7,S8; 0,1,2,3,4,5,6,7}
tuple_impl! {ChannelCombining9Tuple; S1,S2,S3,S4,S5,S6,S7,S8,S9; 0,1,2,3,4,5,6,7,8}
tuple_impl! {ChannelCombining10Tuple; S1,S2,S3,S4,S5,S6,S7,S8,S9,S10; 0,1,2,3,4,5,6,7,8,9}
tuple_impl! {ChannelCombining11Tuple; S1,S2,S3,S4,S5,S6,S7,S8,S9,S10,S11;
0,1,2,3,4,5,6,7,8,9,10}
tuple_impl! {ChannelCombining12Tuple; S1,S2,S3,S4,S5,S6,S7,S8,S9,S10,S11,S12;
0,1,2,3,4,5,6,7,8,9,10,11}

#[cfg(test)]
pub(crate) mod tests {
    use itertools::Itertools;

    use crate::fixed_source::buffer::SamplesBuffer;
    use crate::nz;

    use super::*;

    #[test]
    fn combined2() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(1), vec![1.0, 2.0, 3.0]);
        let s2 = SamplesBuffer::new(nz!(1), nz!(1), vec![4.0, 5.0, 6.0]);

        assert_eq!(
            vec![1., 4., 2., 5., 3., 6.],
            (s1, s2).try_combine_channels().unwrap().collect::<Vec<_>>()
        );
    }

    #[test]
    fn combined3() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(1), vec![1.0]);
        let s2 = SamplesBuffer::new(nz!(2), nz!(1), vec![2.0, 3.0]);
        let s3 = SamplesBuffer::new(nz!(1), nz!(1), vec![4.0]);

        assert_eq!(
            vec![1., 2., 3., 4.],
            (s1, s2, s3)
                .try_combine_channels()
                .unwrap()
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn combined3_mismatch() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(1), vec![1.0]);
        let s2 = SamplesBuffer::new(nz!(2), nz!(2), vec![2.0]);
        let s3 = SamplesBuffer::new(nz!(1), nz!(1), vec![3.0]);

        assert!((s1, s2, s3).try_combine_channels().is_err());
    }

    #[test]
    fn five_channel_audio_samples_in_correct_order() {
        let s1 = SamplesBuffer::new(nz!(2), nz!(44100), vec![1.0, 2.0, 1.0, 2.0]);
        let s2 = SamplesBuffer::new(nz!(1), nz!(44100), vec![3.0; 2]);
        let s3 = SamplesBuffer::new(nz!(2), nz!(44100), vec![4.0, 5.0, 4.0, 5.0]);

        let combined = (s1, s2, s3).try_combine_channels().unwrap();
        assert_eq!(combined.channels(), nz!(5));
        assert_eq!(
            combined.collect_vec(),
            vec![1.0, 2.0, 3.0, 4.0, 5.0, 1.0, 2.0, 3.0, 4.0, 5.0]
        )
    }
}
