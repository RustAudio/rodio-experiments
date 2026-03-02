use rodio::Sample;

use crate::ConstSource;
use crate::const_source::{add_inner_methods, impl_wrapper};

pub type Factor = crate::effects::amplify::Factor;

pub struct Amplify<const SR: u32, const CH: u16, S: ConstSource<SR, CH>> {
    pub(crate) inner: S,
    pub(crate) factor: f32,
}

add_inner_methods! {Amplify}
impl_wrapper! {Amplify}

impl<const SR: u32, const CH: u16, S: ConstSource<SR, CH>> Amplify<SR, CH, S> {
    pub fn set_factor(&mut self, factor: Factor) {
        self.factor = factor.as_linear();
    }
}

impl<const SR: u32, const CH: u16, S: ConstSource<SR, CH>> Iterator for Amplify<SR, CH, S> {
    type Item = Sample;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|value| value * self.factor)
    }
}
