use super::IntoMixed;
use crate::ConstSource;

macro_rules! tuple_impl {
    ($list:ident; $($generics:ident),+; $($count:tt),*; $last:tt) => {
        #[derive(Clone, Debug)]
        pub struct $list<const SR: u32, const CH: u16, $($generics),+> {
            sources: ($($generics),+),
        }

        impl<const SR: u32, const CH: u16, $($generics: ConstSource<SR, CH>),+> Iterator for $list<SR, CH, $($generics),+> {
            type Item = crate::Sample;

            fn next(&mut self) -> Option<Self::Item> {
                let mut samples = 0u8;
                let mut sum = None;
                $(
                    let sample = self.sources.$count.next().map(|s| s as f64);
                    samples += sample.is_some() as u8;
                    sum = match [sum, sample] {
                        [Some(sum), None] => Some(sum),
                        [Some(sum), Some(sample)] => Some(sum + sample),
                        [None, Some(sample)] => Some(sample),
                        [None, None] => None,
                    };
                )*
                let sample = self.sources.$last.next().map(|s| s as f64);
                samples += sample.is_some() as u8;
                sum = match [sum, sample] {
                    [Some(sum), None] => Some(sum),
                    [Some(sum), Some(sample)] => Some(sum + sample),
                    [None, Some(sample)] => Some(sample),
                    [None, None] => None,
                };
                sum.map(|sum| sum / samples as f64).map(|sum| sum as f32)
            } // next
        } // impl iter

        impl<const SR: u32, const CH: u16, $($generics: ConstSource<SR, CH>),+> ConstSource<SR, CH> for $list<SR, CH, $($generics),+> {
            fn total_duration(&self) -> Option<std::time::Duration> {
                let mut max = None;
                $(
                    let dur= self.sources.$count.total_duration();
                    max = match [max, dur] {
                        [Some(max), None] => Some(max),
                        [Some(max), Some(dur)] => Some(max.max(dur)),
                        [None, Some(dur)] => Some(dur),
                        [None, None] => None,
                    };
                )*
                let dur= self.sources.$last.total_duration();
                max = match [max, dur] {
                    [Some(max), None] => Some(max),
                    [Some(max), Some(dur)] => Some(max.max(dur)),
                    [None, Some(dur)] => Some(dur),
                    [None, None] => None,
                };
                max
            }
        } // impl FixedSource

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

tuple_impl! {Mixed2Tuple; S1,S2; 0;1}
tuple_impl! {Mixed3Tuple; S1,S2,S3; 0,1;2}
tuple_impl! {Mixed4Tuple; S1,S2,S3,S4; 0,1,2;3}
tuple_impl! {Mixed5Tuple; S1,S2,S3,S4,S5; 0,1,2,3;4}
tuple_impl! {Mixed6Tuple; S1,S2,S3,S4,S5,S6; 0,1,2,3,4;5}
tuple_impl! {Mixed7Tuple; S1,S2,S3,S4,S5,S6,S7; 0,1,2,3,4,5;6}
tuple_impl! {Mixed8Tuple; S1,S2,S3,S4,S5,S6,S7,S8; 0,1,2,3,4,5,6;7}
tuple_impl! {Mixed9Tuple; S1,S2,S3,S4,S5,S6,S7,S8,S9; 0,1,2,3,4,5,6,7;8}
tuple_impl! {Mixed10Tuple; S1,S2,S3,S4,S5,S6,S7,S8,S9,S10; 0,1,2,3,4,5,6,7,8;9}
tuple_impl! {Mixed11Tuple; S1,S2,S3,S4,S5,S6,S7,S8,S9,S10,S11; 0,1,2,3,4,5,6,7,8,9;10}
tuple_impl! {Mixed12Tuple; S1,S2,S3,S4,S5,S6,S7,S8,S9,S10,S11,S12; 0,1,2,3,4,5,6,7,8,9,10;11}

#[cfg(test)]
pub(crate) mod tests {
    use std::time::Duration;
    use crate::const_source::buffer::SamplesBuffer;
    use super::*;

    #[test]
    fn mixed_is_mean_and_longest() {
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
