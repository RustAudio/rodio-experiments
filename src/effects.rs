pub mod amplify;
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
    struct $name:ident$(<$t:ident>)? {
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
            $($m_vis fn $m_name($($args)*) $(-> $m_ret)? $m_body)*
        }

        impl<S: crate::Source$(,$t)?> Iterator for dynamic_source::$name<S$(,$t)?> {
            type Item = crate::Sample;

            fn next(&mut $self) -> Option<Self::Item> {
                $body
            }
        }

        crate::effects::inner!{
            struct $name$(<$t>)? {
                $($field: $field_ty,)*
            }
            fn next(&mut $self) -> Option<Sample> $body
            fn new($($factory_args)*) -> $factory_name<Self> $factory_body
            $($(#[$m_meta])? $m_vis fn $m_name($($args)*) $(-> $m_ret)? $m_body)*
        }
    };

    (
    struct $name:ident$(<$t:ident>)? {
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
            struct $name$(<$t>)? {
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
    struct $name:ident$(<$t:ident>)? {
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
            $($m_vis fn $m_name($($args)*) $(-> $m_ret)? $m_body)*
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
            $($m_vis fn $m_name($($args)*) $(-> $m_ret)? $m_body)*
        }

        impl<const SR: u32, const CH: u16, S: crate::ConstSource<SR, CH>$(,$t)?> Iterator for const_source::$name<SR, CH, S$(,$t)?> {
            type Item = crate::Sample;

            fn next(&mut $self) -> Option<Self::Item> {
                $body
            }
        }
    }
}

pub(crate) use inner;
pub(crate) use pure_effect;
