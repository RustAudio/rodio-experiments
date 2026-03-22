use super::IntoMixed;
use crate::ConstSource;
use crate::common::for_in_tuple;

macro_rules! tuple_impl {
    ($list:ident; $($generics:ident),+; $($count:tt),*) => {
        #[derive(Clone, Debug)]
        pub struct $list<const SR: u32, const CH: u16, $($generics),+> {
            sources: ($($generics),+),
        }

        impl<const SR: u32, const CH: u16, $($generics: ConstSource<SR, CH>),+> Iterator for $list<SR, CH, $($generics),+> {
            type Item = crate::Sample;

            fn next(&mut self) -> Option<Self::Item> {
                let mut summed = 0u8;
                let mut sum = None;
                for_in_tuple! { $($count),*;
                    for mut item in self.sources; {
                        let sample = item.next().map(|s| s as f64);
                        summed += sample.is_some() as u8;
                        sum = match [sum, sample] {
                            [Some(sum), None] => Some(sum),
                            [Some(sum), Some(sample)] => Some(sum + sample),
                            [None, Some(sample)] => Some(sample),
                            [None, None] => None,
                        };
                    } // for every item body
                }

                sum.map(|sum| sum / summed as f64).map(|sum| sum as f32)
            } // next
        } // impl iter

        impl<const SR: u32, const CH: u16, $($generics: ConstSource<SR, CH>),+> ConstSource<SR, CH> for $list<SR, CH, $($generics),+> {
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
        } // impl ConstSource

        impl<const SR: u32, const CH: u16, $($generics: ConstSource<SR, CH>),+> IntoMixed<SR, CH> for ($($generics),+) {
            type MixedSource = $list<SR, CH, $($generics),+>;

            fn into_mixed(self) -> Self::MixedSource {
                Self::MixedSource {
                    sources: self,
                }
            }
        } // impl IntoList
    }; // transcriber
}

tuple_impl! {Mixed2Tuple; S1,S2; 0,1}
tuple_impl! {Mixed3Tuple; S1,S2,S3; 0,1,2}
tuple_impl! {Mixed4Tuple; S1,S2,S3,S4; 0,1,2,3}
tuple_impl! {Mixed5Tuple; S1,S2,S3,S4,S5; 0,1,2,3,4}
tuple_impl! {Mixed6Tuple; S1,S2,S3,S4,S5,S6; 0,1,2,3,4,5}
tuple_impl! {Mixed7Tuple; S1,S2,S3,S4,S5,S6,S7; 0,1,2,3,4,5,6}
tuple_impl! {Mixed8Tuple; S1,S2,S3,S4,S5,S6,S7,S8; 0,1,2,3,4,5,6,7}
tuple_impl! {Mixed9Tuple; S1,S2,S3,S4,S5,S6,S7,S8,S9; 0,1,2,3,4,5,6,7,8}
tuple_impl! {Mixed10Tuple; S1,S2,S3,S4,S5,S6,S7,S8,S9,S10; 0,1,2,3,4,5,6,7,8,9}
tuple_impl! {Mixed11Tuple; S1,S2,S3,S4,S5,S6,S7,S8,S9,S10,S11; 0,1,2,3,4,5,6,7,8,9,10}
tuple_impl! {Mixed12Tuple; S1,S2,S3,S4,S5,S6,S7,S8,S9,S10,S11,S12; 0,1,2,3,4,5,6,7,8,9,10,11}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::const_source::buffer::SamplesBuffer;
    use std::time::Duration;

    #[test]
    fn mixed_is_mean_of_input_and_does_not_end_early() {
        let s1 = SamplesBuffer::<44100, 1>::new(vec![1.0, 2.0, 3.0]);
        let s2 = SamplesBuffer::<44100, 1>::new(vec![4.0, 5.0]);

        assert_eq!(
            vec![2.5, 3.5, 3.0],
            (s1, s2).into_mixed().collect::<Vec<_>>()
        );
    }

    #[test]
    fn duration() {
        let s1 = SamplesBuffer::<1, 1>::new(vec![1.0, 2.0, 3.0]);
        let s2 = SamplesBuffer::<1, 1>::new(vec![4.0, 5.0]);

        assert_eq!(
            Some(Duration::from_secs(3)),
            (s1, s2).into_mixed().total_duration()
        );
    }
}
