pub mod amplify;
pub mod automatic_gain_control;
pub mod blt;
mod channel_volume;
mod distortion;
pub mod dither;
mod fades;
mod fade; // new replacement
mod inspect;
pub mod limiter;
mod pausable;
mod periodic_access;
mod position;
mod stoppable;
mod take_duration;
mod take_samples;
mod with_data;

// we can only get the structure: effects::effect::source_type::Struct with a macro
// so we re-export the structs here to get the nicer structure:
// effects::source_type::Struct;
pub mod fixed_source {
    pub use super::amplify::fixed_source::Amplify;
    pub use super::automatic_gain_control::fixed_source::AutomaticGainControl;
    pub use super::blt::fixed_source::BltFilter;
    pub use super::channel_volume::fixed_source::ChannelVolume;
    pub use super::distortion::fixed_source::Distortion;
    pub use super::dither::fixed_source::Dither;
    pub use super::fades::fade_in::fixed_source::FadeIn;
    pub use super::fades::fade_out::fixed_source::FadeOut;
    pub use super::fades::fade_out_after::fixed_source::FadeOutAfter;
    pub use super::fades::linear_ramp::fixed_source::LinearGainRamp;
    pub use super::inspect::fixed_source::InspectFrame;
    pub use super::limiter::fixed_source::Limit;
    pub use super::pausable::fixed_source::Pausable;
    pub use super::periodic_access::fixed_source::PeriodicAccess;
    pub use super::position::fixed_source::TrackPosition;
    pub use super::stoppable::fixed_source::Stoppable;
    pub use super::take_duration::fixed_source::TakeDuration;
    pub use super::take_samples::fixed_source::TakeSamples;
    pub use super::with_data::fixed_source::WithData;
}
pub mod const_source {
    pub use super::amplify::const_source::Amplify;
    pub use super::automatic_gain_control::const_source::AutomaticGainControl;
    pub use super::blt::const_source::BltFilter;
    pub use super::channel_volume::const_source::ChannelVolume;
    pub use super::distortion::const_source::Distortion;
    pub use super::dither::const_source::Dither;
    pub use super::fades::fade_in::const_source::FadeIn;
    pub use super::fades::fade_out::const_source::FadeOut;
    pub use super::fades::fade_out_after::const_source::FadeOutAfter;
    pub use super::fades::linear_ramp::const_source::LinearGainRamp;
    pub use super::inspect::const_source::InspectFrame;
    pub use super::limiter::const_source::Limit;
    pub use super::pausable::const_source::Pausable;
    pub use super::periodic_access::const_source::PeriodicAccess;
    pub use super::position::const_source::TrackPosition;
    pub use super::stoppable::const_source::Stoppable;
    pub use super::take_duration::const_source::TakeDuration;
    pub use super::take_samples::const_source::TakeSamples;
    pub use super::with_data::const_source::WithData;
}
pub mod dynamic_source {
    pub use super::distortion::dynamic_source::Distortion;
    pub use super::pausable::dynamic_source::Pausable;
    pub use super::periodic_access::dynamic_source::PeriodicAccess;
    pub use super::stoppable::dynamic_source::Stoppable;
    pub use super::with_data::dynamic_source::WithData;
}

macro_rules! pure_effect {
    (
    supports_dynamic_source
    struct $name:ident$(<$t:ident$(:$bound:path)?>)? {
        $($field:ident: $field_ty:ty,)*
    }
    // like `struct` above the `fn`, `&mut` and `-> Option<Sample>` are just there
    // to make the macro input seem regular rust code
    fn next(&mut $self:ident) -> Option<Sample> $body:block
    fn new$(<$new_generic:ident : $new_bound:path>)?($($factory_args:tt)*) -> $factory_name:ident<Self> $factory_body:block
    // mm stands for mutable method
    $($(#[$m_meta:meta])* $m_vis:vis fn $m_name:ident($($args:tt)*) $(-> $m_ret:ty)? $m_body:block)*
    ) => {
        pub(crate) mod dynamic_source {
            #[allow(unused)]
            use super::*;
            #[derive(Clone)]
            pub struct $name<S: crate::DynamicSource$(,$t$(:$bound)?)?> {
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

            fn size_hint(&self) -> (usize, Option<usize>) {
                self.inner.size_hint()
            }
        }

        impl<S: crate::Source$(,$t$(:$bound)?)?> ExactSizeIterator for dynamic_source::$name<S$(,$t)?> where S: ExactSizeIterator {}

        crate::effects::inner!{
            struct $name$(<$t$(:$bound)?>)? {
                $($field: $field_ty,)*
            }
            fn next(&mut $self) -> Option<Sample> $body
            fn new($($factory_args)*) -> $factory_name<Self> $factory_body
            $($(#[$m_meta])* $m_vis fn $m_name($($args)*) $(-> $m_ret)? $m_body)*
        }
    };

    (
    struct $name:ident$(<$t:ident$(:$bound:path)?>)? {
        $($field:ident: $field_ty:ty,)*
    }
    // like `struct` above the `fn`, `&mut` and `-> Option<Sample>` are just there
    // to make the macro input seem regular rust code
    fn next(&mut $self:ident) -> Option<Sample> $body:block
    fn new$(<$new_generic:ident : $new_bound:path>)?($($factory_args:tt)*)
    -> $factory_name:ident<Self> $factory_body:block
    // mm stands for mutable method
    $($(#[$m_meta:meta])* $m_vis:vis fn $m_name:ident($($args:tt)*) $(-> $m_ret:ty)? $m_body:block)*
    ) => {
        crate::effects::inner!{
            struct $name$(<$t$(:$bound)?>)? {
                $($field: $field_ty,)*
            }
            fn next(&mut $self) -> Option<Sample> $body
            fn new($($factory_args)*) -> $factory_name<Self> $factory_body
            $($(#[$m_meta])* $m_vis fn $m_name($($args)*) $(-> $m_ret)? $m_body)*
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
    fn new$(<$new_generic:ident: $new_bound:path>)?($($factory_args:tt)*) -> $factory_name:ident<Self> $factory_body:block
    // mm stands for mutable method
    $($(#[$m_meta:meta])* $m_vis:vis fn $m_name:ident($($args:tt)*) $(-> $m_ret:ty)? $m_body:block)*
    ) =>  {
        pub(crate) mod fixed_source {
            #[allow(unused)]
            use super::*;

            #[derive(Clone)]
            pub struct $name<S: crate::FixedSource$(,$t$(:$bound)?)?> {
                pub(crate) inner: S,
                $(pub(crate) $field: $field_ty),*
            }

            crate::fixed_source::add_inner_methods!{$name$(<$t$(:$bound)?>)?}
            crate::fixed_source::impl_wrapper!{$name$(<$t$(:$bound)?>)?}
        }

        impl<S: crate::FixedSource $(,$t$(:$bound)?)?> fixed_source::$name<S$(,$t)?> {
            #[must_use]
            pub(crate) fn new$(<$new_generic: $new_bound>)?($($factory_args)*)
                -> fixed_source::$name<S$(,$t)?> {
                $factory_body
            }
            $($(#[$m_meta])* $m_vis fn $m_name($($args)*) $(-> $m_ret)? $m_body)*
        }

        impl<S: crate::FixedSource$(,$t$(:$bound)?)?>
            ExactSizeIterator for fixed_source::$name<S$(,$t)?>
                where S: ExactSizeIterator {}

        impl<S: crate::FixedSource$(,$t$(:$bound)?)?>
            Iterator for fixed_source::$name<S$(,$t)?> {

            type Item = crate::Sample;

            fn next(&mut $self) -> Option<Self::Item> {
                $body
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                self.inner.size_hint()
            }
        }

        pub(crate) mod const_source {
            #[allow(unused)]
            use super::*;

            #[derive(Clone)]
            pub struct $name<const SR: u32, const CH: u16, S: crate::ConstSource<SR, CH>
                $(,$t$(:$bound)?)?> {
                pub(crate) inner: S,
                $(pub(crate) $field: $field_ty),*
            }

            crate::const_source::add_inner_methods!{$name$(<$t$(:$bound)?>)?}
            crate::const_source::impl_wrapper!{$name$(<$t$(:$bound)?>)?}
        }

        impl<const SR: u32, const CH: u16, S: crate::ConstSource<SR, CH>$(,$t$(:$bound)?)?>
            const_source::$name<SR, CH, S$(,$t)?> {

            #[must_use]
            pub(crate) fn new$(<$new_generic: $new_bound>)?($($factory_args)*)
                -> const_source::$name<SR, CH, S$(,$t)?> {
                $factory_body
            }
            $($(#[$m_meta])* $m_vis fn $m_name($($args)*) $(-> $m_ret)? $m_body)*
        }


        impl<const SR: u32, const CH: u16, S: crate::ConstSource<SR, CH>$(,$t$(:$bound)?)?>
            ExactSizeIterator for const_source::$name<SR, CH, S$(,$t)?>
                where S: ExactSizeIterator {}

        impl<const SR: u32, const CH: u16, S: crate::ConstSource<SR, CH>$(,$t$(:$bound)?)?>
            Iterator for const_source::$name<SR, CH, S$(,$t)?> {
            type Item = crate::Sample;

            fn next(&mut $self) -> Option<Self::Item> {
                $body
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                self.inner.size_hint()
            }
        }
    }
}

pub(crate) use inner;
pub(crate) use pure_effect;
