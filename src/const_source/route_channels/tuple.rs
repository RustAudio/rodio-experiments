use crate::ConstSource;
use crate::common::for_in_tuple;

pub(crate) mod helpful_compile_time_error;

macro_rules! tuple_impl {
    ($trait:ident; $struct:ident; $($channels:ident),+; $($generics:ident),+; $($count:tt),*) => {
        pub trait $trait<const SR: u32, $(const $channels: u16),+> {
            type CombinerSource<const CH: u16>: ConstSource<SR, CH>;
            fn combine_channels<const CH: u16>(self) -> Self::CombinerSource<CH>;
        }

        #[derive(Clone, Debug)]
        pub struct $struct<const SR: u32, const CH: u16,
            $(const $channels: u16,)+
            $($generics),+>
        {
            sources: ($($generics),+),
            current: u16,
        }

        impl<const SR: u32, const CH: u16,
            $(const $channels: u16,)+
            $($generics: ConstSource<SR, $channels>),+>
        Iterator for $struct<SR, CH, $($channels,)+ $($generics),+> {
            type Item = crate::Sample;

            fn next(&mut self) -> Option<Self::Item> {
                #![expect(unused,
                    reason = "channel += item.channels().get not used in the last iteration"
                )]

                let mut channel = 0;
                for_in_tuple! { $($count),*;
                    for mut item in self.sources; {
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
            } // fn next
        } // impl Iter

        impl<const SR: u32, const CH: u16,
            $(const $channels: u16,)+
            $($generics: ConstSource<SR, $channels>),+>
        ConstSource<SR, CH> for $struct<SR, CH, $($channels,)+ $($generics),+> {

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
            } // fn total_duration
        } // impl ConstSource

        impl<const SR: u32,
            $(const $channels: u16,)+
            $($generics: ConstSource<SR, $channels>),+>
        $trait<SR, $($channels),+> for ($($generics),+) {
            type CombinerSource<const CH: u16> = $struct<SR, CH,
                $($channels,)+ $($generics),+
            >;

            #[track_caller]
            fn combine_channels<const CH: u16>(self) -> Self::CombinerSource<CH> {
                helpful_compile_time_error::assert_channel_counts!(CH, $($channels),+);
                $struct {
                    sources: self,
                    current: 0,
                }
            }
        } // impl IntoList
    }; // transcriber
}

tuple_impl! {CH2T; CombinedChannels2Tuple; CH1,CH2; S1,S2; 0,1}
tuple_impl! {CH3T; CombinedChannels3Tuple; CH1,CH2,CH3; S1,S2,S3; 0,1,2}
tuple_impl! {CH4T; CombinedChannels4Tuple; CH1,CH2,CH3,CH4; S1,S2,S3,S4; 0,1,2,3}
tuple_impl! {CH5T; CombinedChannels5Tuple; CH1,CH2,CH3,CH4,CH5; S1,S2,S3,S4,S5; 0,1,2,3,4}
tuple_impl! {CH6T; CombinedChannels6Tuple; CH1,CH2,CH3,CH4,CH5,CH6;
S1,S2,S3,S4,S5,S6; 0,1,2,3,4,5}
tuple_impl! {CH7T; CombinedChannels7Tuple; CH1,CH2,CH3,CH4,CH5,CH6,CH7;
S1,S2,S3,S4,S5,S6,S7; 0,1,2,3,4,5,6}
tuple_impl! {CH8T; CombinedChannels8Tuple; CH1,CH2,CH3,CH4,CH5,CH6,CH7,CH8;
S1,S2,S3,S4,S5,S6,S7,S8; 0,1,2,3,4,5,6,7}
tuple_impl! {CH9T; CombinedChannels9Tuple; CH1,CH2,CH3,CH4,CH5,CH6,CH7,CH8,CH9;
S1,S2,S3,S4,S5,S6,S7,S8,S9; 0,1,2,3,4,5,6,7,8}
tuple_impl! {CH10T; CombinedChannels10Tuple;
CH1,CH2,CH3,CH4,CH5,CH6,CH7,CH8,CH9,CH10;
S1,S2,S3,S4,S5,S6,S7,S8,S9,S10; 0,1,2,3,4,5,6,7,8,9}
tuple_impl! {CH11T; CombinedChannels11Tuple;
CH1,CH2,CH3,CH4,CH5,CH6,CH7,CH8,CH9,CH10,CH11;
S1,S2,S3,S4,S5,S6,S7,S8,S9,S10,S11; 0,1,2,3,4,5,6,7,8,9,10}
tuple_impl! {CH12T; CombinedChannels12Tuple;
CH1,CH2,CH3,CH4,CH5,CH6,CH7,CH8,CH9,CH10,CH11,CH12;
S1,S2,S3,S4,S5,S6,S7,S8,S9,S10,S11,S12; 0,1,2,3,4,5,6,7,8,9,10,11}

pub mod prefix {
    pub use super::{CH2T, CH3T, CH4T, CH5T, CH6T, CH7T, CH8T, CH9T, CH10T, CH11T, CH12T};
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

    #[test]
    fn five_channel_audio_samples_in_correct_order() {
        let s1 = SamplesBuffer::<44100, 2>::new(vec![1.0, 2.0, 1.0, 2.0]);
        let s2 = SamplesBuffer::<44100, 1>::new(vec![3.0; 2]);
        let s3 = SamplesBuffer::<44100, 2>::new(vec![4.0, 5.0, 4.0, 5.0]);

        let combined = (s1, s2, s3).combine_channels::<5>();
        assert_eq!(
            combined.collect_vec(),
            vec![1.0, 2.0, 3.0, 4.0, 5.0, 1.0, 2.0, 3.0, 4.0, 5.0]
        )
    }

    #[test]
    fn crazy() {
        let s1 = SamplesBuffer::<44100, 2>::new(vec![1.0, 2.0]);
        let s2 = SamplesBuffer::<44100, 2>::new(vec![1.0, 2.0]);
        let s3 = SamplesBuffer::<44100, 2>::new(vec![1.0, 2.0]);
        let s4 = SamplesBuffer::<44100, 2>::new(vec![1.0, 2.0]);
        let s5 = SamplesBuffer::<44100, 2>::new(vec![1.0, 2.0]);
        let s6 = SamplesBuffer::<44100, 2>::new(vec![1.0, 2.0]);
        let s7 = SamplesBuffer::<44100, 2>::new(vec![1.0, 2.0]);
        let s8 = SamplesBuffer::<44100, 2>::new(vec![1.0, 2.0]);
        let s9 = SamplesBuffer::<44100, 2>::new(vec![1.0, 2.0]);
        let s10 = SamplesBuffer::<44100, 2>::new(vec![1.0, 2.0]);
        let s11 = SamplesBuffer::<44100, 2>::new(vec![1.0, 2.0]);
        let s12 = SamplesBuffer::<44100, 2>::new(vec![1.0, 2.0]);

        let combined = (s1, s2, s3, s4, s5, s6, s7, s8, s9, s10, s11, s12).combine_channels::<24>();

        assert_eq!(combined.channels().get(), 24);
        assert_eq!(
            combined.collect_vec(),
            [1f32, 2.0].into_iter().cycle().take(24).collect_vec()
        )
    }
}
