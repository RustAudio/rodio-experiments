pub mod amplify;
pub mod periodic_access;
pub mod with_data;
pub mod stoppable;
pub mod pausable;
pub mod take_duration;
pub mod take_samples;

/// Note: methods taking &mut self must have mut ref as a prefix, they must be
/// specified before methods taking &self
macro_rules! pure_effect {
    (
    struct $name:ident$(<$t:ident>)? {
        $($field:ident: $field_ty:ty,)*
    }
    // like `struct` above the `fn`, `&mut` and `-> Option<Sample>` are just there
    // to make the macro input seem regular rust code
    fn next(&mut $self:ident) -> Option<Sample> $body:block
    fn new($($factory_args:tt)*) -> $factory_name:ident<Self> $factory_body:block
    // mm stands for mutable method
    $($(#[$mm_meta:meta])? $mm_vis:vis fn $mm_name:ident($($args:tt)*) $(-> $mm_ret:ty)? $mm_body:block)*
    ) => {
        pub mod dynamic_source {
            pub struct $name<S$(,$t)?> {
                pub(crate) inner: S,
                $(pub(crate) $field: $field_ty),*
            }

            crate::dynamic_source::add_inner_methods!{$name$(<$t>)?}
            crate::dynamic_source::impl_wrapper!{$name$(<$t>)?}
        }

        impl<S: crate::Source$(,$t)?> dynamic_source::$name<S$(,$t)?> {
            #[must_use]
            pub(crate) fn new($($factory_args)*) -> dynamic_source::$name<S$(,$t)?> {
                $factory_body
            }
            $($mm_vis fn $mm_name($($args)*) $(-> $mm_ret)? {
                $mm_body
            })*
        }

        impl<S: crate::Source$(,$t)?> Iterator for dynamic_source::$name<S$(,$t)?> {
            type Item = crate::Sample;

            fn next(&mut $self) -> Option<Self::Item> {
                $body
            }
        }

        pub mod fixed_source {
            pub struct $name<S$(,$t)?> {
                pub(crate) inner: S,
                $(pub(crate) $field: $field_ty),*
            }

            crate::fixed_source::add_inner_methods!{$name$(<$t>)?}
            crate::fixed_source::impl_wrapper!{$name$(<$t>)?}
        }

        impl<S: crate::FixedSource$(,$t)?> fixed_source::$name<S$(,$t)?> {
            #[must_use]
            pub(crate) fn new($($factory_args)*) -> fixed_source::$name<S$(,$t)?> {
                $factory_body
            }
            $($mm_vis fn $mm_name($($args)*) $(-> $mm_ret)? {
                $mm_body
            })*
        }

        impl<S: crate::FixedSource$(,$t)?> Iterator for fixed_source::$name<S$(,$t)?> {
            type Item = crate::Sample;

            fn next(&mut $self) -> Option<Self::Item> {
                $body
            }
        }

        pub mod const_source {
            pub struct $name<const SR: u32, const CH: u16, S$(,$t)?> {
                pub(crate) inner: S,
                $(pub(crate) $field: $field_ty),*
            }

            crate::const_source::add_inner_methods!{$name$(<$t>)?}
            crate::const_source::impl_wrapper!{$name$(<$t>)?}
        }

        impl<const SR: u32, const CH: u16, S: crate::ConstSource<SR, CH>$(,$t)?> const_source::$name<SR, CH, S$(,$t)?> {
            #[must_use]
            pub(crate) fn new($($factory_args)*) -> const_source::$name<SR, CH, S$(,$t)?> {
                $factory_body
            }
            $($mm_vis fn $mm_name($($args)*) $(-> $mm_ret)? {
                $mm_body
            })*
        }

        impl<const SR: u32, const CH: u16, S: crate::ConstSource<SR, CH>$(,$t)?> Iterator for const_source::$name<SR, CH, S$(,$t)?> {
            type Item = crate::Sample;

            fn next(&mut $self) -> Option<Self::Item> {
                $body
            }
        }
    };
}

pub(crate) use pure_effect;
