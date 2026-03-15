use std::time::Duration;

use crate::FixedSource;
use crate::{ChannelCount, SampleRate};

use super::super::MaybeConvert;
use super::super::convert_if_needed;
use super::{IntoQueued, ParamsMismatch};
use crate::common::for_in_tuple;

macro_rules! tuple_impl {
    ($list:ident; $($generics:ident),+; $($count:tt),*) => {
        #[derive(Clone, Debug)]
        pub struct $list<$($generics),+> {
            sources: ($($generics),+),
            current: u8,
        }

        impl<$($generics: FixedSource),+> Iterator for $list<$($generics),+> {
            type Item = crate::Sample;

            fn next(&mut self) -> Option<Self::Item> {
                #![expect(unused, reason = "index += 1 is not used in the last iteration")]

                let mut index = 0;
                for_in_tuple! { $($count),+;
                    for mut source in self.sources; {
                        if index >= self.current {
                            if let Some(sample) = source.next() {
                                self.current = index;
                                return Some(sample)
                            }
                        }
                        index += 1;
                    }
                }
                None
            } // next
        } // impl iter

        impl<$($generics: FixedSource),+> FixedSource for $list<$($generics),+> {
            fn channels(&self) -> rodio::ChannelCount {
                self.sources.0.channels()
            }

            fn sample_rate(&self) -> rodio::SampleRate {
                self.sources.0.sample_rate()
            }

            fn total_duration(&self) -> Option<std::time::Duration> {
                let mut total_duration = Some(Duration::ZERO);
                for_in_tuple! { $($count),+;
                    for source in self.sources; {
                        total_duration = total_duration
                            .and_then(|d| source.total_duration()
                                .map(|sd| d + sd))
                    }
                }
                total_duration
            }
        } // impl FixedSource

        impl<$($generics: FixedSource),+> IntoQueued for ($($generics),+) {
            type TryQueuedSource = $list<$($generics),+>;
            type IntoQueuedSource = $list<$(MaybeConvert<$generics>),+>;

            fn try_into_list(self) -> Result<Self::TryQueuedSource, ParamsMismatch> {
                #![expect(unused, reason = "index += 1 is not used in the last iteration")]

                let left = (self.0.sample_rate(), self.0.channels());
                let mut index = 0;

                for_in_tuple! { $($count),+;
                    for source in self; {
                        let right = (source.sample_rate(), source.channels());
                        if left != right {
                            return Err(ParamsMismatch {
                                index_of_first_mismatch: index as usize,
                                sample_rate_left: left.0,
                                channel_count_left: left.1,
                                sample_rate_right: right.0,
                                channel_count_right: right.1,
                            });
                        }
                        index += 1;
                    }
                }

                Ok($list {
                    sources: self,
                    current: 0,
                })
            }

            fn into_list_converted(
                self,
                sample_rate: SampleRate,
                channels: ChannelCount,
            ) -> Self::IntoQueuedSource{

                let sources = (
                $(
                    convert_if_needed(self.$count, sample_rate, channels),
                )+
                );

                $list {
                    sources,
                    current: 0,
                }
            }
        } // impl IntoList
    }; // transcriber
}

tuple_impl! {Queued2Tuple; S1,S2; 0,1}
tuple_impl! {Queued3Tuple; S1,S2,S3; 0,1,2}
tuple_impl! {Queued4Tuple; S1,S2,S3,S4; 0,1,2,3}
tuple_impl! {Queued5Tuple; S1,S2,S3,S4,S5; 0,1,2,3,4}
tuple_impl! {Queued6Tuple; S1,S2,S3,S4,S5,S6; 0,1,2,3,4,5}
tuple_impl! {Queued7Tuple; S1,S2,S3,S4,S5,S6,S7; 0,1,2,3,4,5,6}
tuple_impl! {Queued8Tuple; S1,S2,S3,S4,S5,S6,S7,S8; 0,1,2,3,4,5,6,7}
tuple_impl! {Queued9Tuple; S1,S2,S3,S4,S5,S6,S7,S8,S9; 0,1,2,3,4,5,6,7,8}
tuple_impl! {Queued10Tuple; S1,S2,S3,S4,S5,S6,S7,S8,S9,S10; 0,1,2,3,4,5,6,7,8,9}
tuple_impl! {Queued11Tuple; S1,S2,S3,S4,S5,S6,S7,S8,S9,S10,S11; 0,1,2,3,4,5,6,7,8,9,10}
tuple_impl! {Queued12Tuple; S1,S2,S3,S4,S5,S6,S7,S8,S9,S10,S11,S12; 0,1,2,3,4,5,6,7,8,9,10,11}

#[cfg(test)]
pub(crate) mod tests {
    use crate::fixed_source::buffer::SamplesBuffer;
    use crate::nz;

    use super::*;

    #[test]
    fn list2() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(1), vec![1.0, 2.0, 3.0]);
        let s2 = SamplesBuffer::new(nz!(1), nz!(1), vec![4.0, 5.0, 6.0]);

        assert_eq!(
            vec![1., 2., 3., 4., 5., 6.],
            (s1, s2).try_into_list().unwrap().collect::<Vec<_>>()
        );
    }

    #[test]
    fn list3() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(1), vec![1.0]);
        let s2 = SamplesBuffer::new(nz!(1), nz!(1), vec![2.0]);
        let s3 = SamplesBuffer::new(nz!(1), nz!(1), vec![3.0]);

        assert_eq!(
            vec![1., 2., 3.],
            (s1, s2, s3).try_into_list().unwrap().collect::<Vec<_>>()
        );
    }

    #[test]
    fn list3_mismatch() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(1), vec![1.0]);
        let s2 = SamplesBuffer::new(nz!(2), nz!(2), vec![2.0]);
        let s3 = SamplesBuffer::new(nz!(1), nz!(1), vec![3.0]);

        assert_eq!(
            ParamsMismatch {
                index_of_first_mismatch: 1,
                sample_rate_left: nz!(1),
                channel_count_left: nz!(1),
                sample_rate_right: nz!(2),
                channel_count_right: nz!(2),
            },
            (s1, s2, s3).try_into_list().unwrap_err()
        );
    }

    #[test]
    fn into_list_converted_all_rechanneled() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(1), vec![1.0; 1]);
        let s2 = SamplesBuffer::new(nz!(3), nz!(1), vec![2.0; 3]);
        let s3 = SamplesBuffer::new(nz!(2), nz!(1), vec![3.0; 2]);

        assert_eq!(
            vec![1.0, 2.0, 3.0],
            (s1, s2, s3)
                .into_list_converted(nz!(1), nz!(1))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn into_list_converted_all_rechanneled_duration() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(1), vec![1.0; 1]); // 1s
        let s2 = SamplesBuffer::new(nz!(3), nz!(1), vec![2.0; 3]); // 1s
        let s3 = SamplesBuffer::new(nz!(2), nz!(1), vec![3.0; 4]); // 2s

        assert_eq!(
            Some(Duration::from_secs(1 + 1 + 2)),
            (s1, s2, s3)
                .into_list_converted(nz!(1), nz!(1))
                .total_duration()
        );
    }
}
