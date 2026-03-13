use std::time::Duration;

use itertools::Itertools;

use crate::ConstSource;

mod array;
mod tuple;
mod vec;

pub trait IntoList<const SR: u32, const CH: u16> {
    type ListSource: ConstSource<SR, CH>;
    fn into_list(self) -> Self::ListSource;
}
