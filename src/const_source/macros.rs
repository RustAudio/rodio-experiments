
macro_rules! tuple_impl {
    ($list:ident; $($generics:ident),+; $($count:tt),*; $last:tt;
    fn next(&mut $self:ident) -> Option<Sample> $body:block) => {
        #[derive(Clone, Debug)]
        pub struct $list<const SR: u32, const CH: u16, $($generics),+> {
            sources: ($($generics),+),
            current: u8,
        }

        impl<const SR: u32, const CH: u16, $($generics: ConstSource<SR, CH>),+>
            Iterator for $list<SR, CH, $($generics),+> {

            type Item = crate::Sample;

            fn next(&mut $self) -> Option<Self::Item> {
                $body
            }
        } // impl iter

        impl<const SR: u32, const CH: u16, $($generics: ConstSource<SR, CH>),+>
            ConstSource<SR, CH> for $list<SR, CH, $($generics),+> {

            fn channels(&self) -> rodio::ChannelCount {
                self.sources.0.channels()
            }

            fn sample_rate(&self) -> rodio::SampleRate {
                self.sources.0.sample_rate()
            }

            fn total_duration(&self) -> Option<std::time::Duration> {
                self.sources
                    .0
                    .total_duration()
                    .and_then(|d0| self.sources.1.total_duration().map(|d1| d1 + d0))
            }
        } // impl FixedSource

        impl<const SR: u32, const CH: u16, $($generics: ConstSource<SR, CH>),+> IntoList<SR, CH> for ($($generics),+) {
            type ListSource = $list<SR, CH, $($generics),+>;

            fn into_list(self) -> Self::ListSource {
                $list {
                    sources: self,
                    current: 0,
                }
            }
        } // impl IntoList
    }; // transcriber
}

pub(crate) use tuple_impl;

macro_rules! add_inner_methods {
    ($name:ident$(<$t:ident$(:$bound:path)?>)?) => {
        impl<const SR: u32, const CH: u16, S: crate::ConstSource<SR, CH>$(,$t$(:$bound)?)?> $name<SR, CH, S$(,$t)?> {
            pub fn inner(&self) -> &S {
                &self.inner
            }
            pub fn inner_mut(&mut self) -> &mut S {
                &mut self.inner
            }
            pub fn into_inner(self) -> S {
                self.inner
            }
        }
    };
}

pub(crate) use add_inner_methods;

macro_rules! impl_wrapper {
    ($name:ident$(<$t:ident$(:$bound:path)?>)?) => {
        impl<const SR: u32, const CH: u16, S: crate::ConstSource<SR, CH>$(,$t$(:$bound)?)?> crate::ConstSource<SR, CH>
            for $name<SR, CH, S$(,$t)?>
        {
            fn total_duration(&self) -> Option<std::time::Duration> {
                self.inner.total_duration()
            }
        }
    };
}
pub(crate) use impl_wrapper;
