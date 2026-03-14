use crate::ConstSource;

mod array;
mod tuple;
mod vec;

pub trait IntoMixed<const SR: u32, const CH: u16>  {
    type MixedSource: ConstSource<SR, CH>;

    fn into_mixed(self) -> Self::MixedSource;
}
