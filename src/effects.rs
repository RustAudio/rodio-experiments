pub mod amplify;
pub mod automatic_gain_control;
pub mod inspect;
pub mod pausable;
pub mod periodic_access;
pub mod stoppable;
pub mod take_duration;
pub mod take_samples;
pub mod with_data;

/// Note: methods taking &mut self must have mut ref as a prefix, they must be
/// specified before methods taking &self
macro_rules! pure_effect {
    (
    supports_dynamic_source
    struct $name:ident$(<$t:ident$(:$bound:path)?>)? {
        $($field:ident: $field_ty:ty,)*
    }
    // like `struct` above the `fn`, `&mut` and `-> Option<Sample>` are just there
    // to make the macro input seem regular rust code
    fn next(&mut $self:ident) -> Option<Sample> $body:block
    fn new($($factory_args:tt)*) -> $factory_name:ident<Self> $factory_body:block
    // mm stands for mutable method
    $($(#[$m_meta:meta])? $m_vis:vis fn $m_name:ident($($args:tt)*) $(-> $m_ret:ty)? $m_body:block)*
    ) => {
        pub mod dynamic_source {
            #[allow(unused)]
            use super::*;
            pub struct $name<S$(,$t$(:$bound)?)?> {
                pub(crate) inner: S,
                $(pub(crate) $field: $field_ty),*
            }

            crate::dynamic_source::add_inner_methods!{$name$(<$t$(:$bound)?>)?}
            crate::dynamic_source::impl_wrapper!{$name$(<$t$(:$bound)?>)?}
        }

        impl<S: crate::Source$(,$t$(:$bound)?)?> dynamic_source::$name<S$(,$t)?> {
            #[must_use]
            pub(crate) fn new($($factory_args)*) -> dynamic_source::$name<S$(,$t)?> {
                $factory_body
            }
            $($m_vis fn $m_name($($args)*) $(-> $m_ret)? $m_body)*
        }

        impl<S: crate::Source$(,$t$(:$bound)?)?> Iterator for dynamic_source::$name<S$(,$t)?> {
            type Item = crate::Sample;

            fn next(&mut $self) -> Option<Self::Item> {
                $body
            }
        }


        impl<S: crate::Source$(,$t$(:$bound)?)?> ExactSizeIterator for dynamic_source::$name<S$(,$t)?> where S: ExactSizeIterator {}

        crate::effects::inner!{
            struct $name$(<$t$(:$bound)?>)? {
                $($field: $field_ty,)*
            }
            fn next(&mut $self) -> Option<Sample> $body
            fn new($($factory_args)*) -> $factory_name<Self> $factory_body
            $($(#[$m_meta])? $m_vis fn $m_name($($args)*) $(-> $m_ret)? $m_body)*
        }
    };

    (
    struct $name:ident$(<$t:ident$(:$bound:path)?>)? {
        $($field:ident: $field_ty:ty,)*
    }
    // like `struct` above the `fn`, `&mut` and `-> Option<Sample>` are just there
    // to make the macro input seem regular rust code
    fn next(&mut $self:ident) -> Option<Sample> $body:block
    fn new($($factory_args:tt)*) -> $factory_name:ident<Self> $factory_body:block
    // mm stands for mutable method
    $($(#[$m_meta:meta])? $m_vis:vis fn $m_name:ident($($args:tt)*) $(-> $m_ret:ty)? $m_body:block)*
    ) => {
        crate::effects::inner!{
            struct $name$(<$t$(:$bound)?>)? {
                $($field: $field_ty,)*
            }
            fn next(&mut $self) -> Option<Sample> $body
            fn new($($factory_args)*) -> $factory_name<Self> $factory_body
            $($(#[$m_meta])? $m_vis fn $m_name($($args)*) $(-> $m_ret)? $m_body)*
        }
    }
}

macro_rules! inner {
(
    struct $name:ident$(<$t:ident$(:$bound:path)?>)? {
        $($field:ident: $field_ty:ty,)*
    }
    // like `struct` above the `fn`, `&mut` and `-> Option<Sample>` are just there
    // to make the macro input seem regular rust code
    fn next(&mut $self:ident) -> Option<Sample> $body:block
    fn new($($factory_args:tt)*) -> $factory_name:ident<Self> $factory_body:block
    // mm stands for mutable method
    $($(#[$m_meta:meta])? $m_vis:vis fn $m_name:ident($($args:tt)*) $(-> $m_ret:ty)? $m_body:block)*
    ) =>  {
        pub mod fixed_source {
            #[allow(unused)]
            use super::*;

            pub struct $name<S$(,$t$(:$bound)?)?> {
                pub(crate) inner: S,
                $(pub(crate) $field: $field_ty),*
            }

            crate::fixed_source::add_inner_methods!{$name$(<$t$(:$bound)?>)?}
            crate::fixed_source::impl_wrapper!{$name$(<$t$(:$bound)?>)?}
        }

        impl<S: crate::FixedSource$(,$t$(:$bound)?)?> fixed_source::$name<S$(,$t)?> {
            #[must_use]
            pub(crate) fn new($($factory_args)*) -> fixed_source::$name<S$(,$t)?> {
                $factory_body
            }
            $($m_vis fn $m_name($($args)*) $(-> $m_ret)? $m_body)*
        }

        impl<S: crate::FixedSource$(,$t$(:$bound)?)?> ExactSizeIterator for fixed_source::$name<S$(,$t)?> where S: ExactSizeIterator {}

        impl<S: crate::FixedSource$(,$t$(:$bound)?)?> Iterator for fixed_source::$name<S$(,$t)?> {
            type Item = crate::Sample;

            fn next(&mut $self) -> Option<Self::Item> {
                $body
            }
        }

        pub mod const_source {
            #[allow(unused)]
            use super::*;

            pub struct $name<const SR: u32, const CH: u16, S$(,$t$(:$bound)?)?> {
                pub(crate) inner: S,
                $(pub(crate) $field: $field_ty),*
            }

            crate::const_source::add_inner_methods!{$name$(<$t$(:$bound)?>)?}
            crate::const_source::impl_wrapper!{$name$(<$t$(:$bound)?>)?}
        }

        impl<const SR: u32, const CH: u16, S: crate::ConstSource<SR, CH>$(,$t$(:$bound)?)?> const_source::$name<SR, CH, S$(,$t)?> {
            #[must_use]
            pub(crate) fn new($($factory_args)*) -> const_source::$name<SR, CH, S$(,$t)?> {
                $factory_body
            }
            $($m_vis fn $m_name($($args)*) $(-> $m_ret)? $m_body)*
        }


        impl<const SR: u32, const CH: u16, S: crate::ConstSource<SR, CH>$(,$t$(:$bound)?)?>  ExactSizeIterator for const_source::$name<SR, CH, S$(,$t)?> where S: ExactSizeIterator {}

        impl<const SR: u32, const CH: u16, S: crate::ConstSource<SR, CH>$(,$t$(:$bound)?)?> Iterator for const_source::$name<SR, CH, S$(,$t)?> {
            type Item = crate::Sample;

            fn next(&mut $self) -> Option<Self::Item> {
                $body
            }
        }
    }
}

pub(crate) use inner;
pub(crate) use pure_effect;
