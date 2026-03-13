use crate::ConstSource;

mod array;
mod tuple;
mod vec;

pub trait IntoQueued<const SR: u32, const CH: u16> {
    type QueuedSource: ConstSource<SR, CH>;
    fn into_list(self) -> Self::QueuedSource;
}
