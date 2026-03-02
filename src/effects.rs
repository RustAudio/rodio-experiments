pub mod amplify;

macro_rules! pure_effect {
    ($name:ident,
     struct {
        $field:ident: $field_ty:ty,
    },
    next($self:ident) {
        $body:expr
    },
    // mm stands for mutable method
    $($mm_vis:vis fn $mm_name:ident(&mut $mm_self:ident $(, $mm_arg:ident: $mm_arg_ty:ty)*) $(-> $mm_ret:ty)? {
        $mm_body:expr
    },)*
    // $($m_vis:vis fn $m_name:ident(&$m_self:ident $(, $m_arg:ident: $m_arg_ty:ty)*) $(-> $m_ret:ty)? {
    //     $m_body:expr
    // },)*
    ) => {
        pub mod fixed_source {
            pub struct $name<S> {
                pub(crate) inner: S,
                pub(crate) $field: $field_ty,
            }

            crate::fixed_source::add_inner_methods!{$name}
            crate::fixed_source::impl_wrapper!{$name}

        }

        impl<S: crate::FixedSource> fixed_source::$name<S> {
            $($mm_vis fn $mm_name(&mut $mm_self $(, $mm_arg: $mm_arg_ty)*) $(-> $mm_ret)? {
                $mm_body
            })*
            // $m_vis fn $m_name(&mut $m_self $(, $m_arg: $m_arg_ty)*) $(-> $m_ret)? {
            //     $m_body
            // }
        }

        impl<S: crate::FixedSource> Iterator for fixed_source::$name<S> {
            type Item = crate::Sample;

            fn next(&mut $self) -> Option<Self::Item> {
                $body
            }
        }

        pub mod const_source {
            pub struct $name<const SR: u32, const CH: u16, S> {
                pub(crate) inner: S,
                pub(crate) $field: $field_ty,
            }

            crate::const_source::add_inner_methods!{$name}
            crate::const_source::impl_wrapper!{$name}
        }

        impl<const SR: u32, const CH: u16, S: crate::ConstSource<SR, CH>> const_source::$name<SR, CH, S> {
            $($mm_vis fn $mm_name(&mut $mm_self $(, $mm_arg: $mm_arg_ty)*) $(-> $mm_ret)? {
                $mm_body
            })*
            // $m_vis fn $m_name(&mut $m_self $(, $m_arg: $m_arg_ty)*) $(-> $m_ret)? {
            //     $m_body
            // }
        }

        impl<const SR: u32, const CH: u16, S: crate::ConstSource<SR, CH>> Iterator for const_source::$name<SR, CH, S> {
            type Item = crate::Sample;

            fn next(&mut $self) -> Option<Self::Item> {
                $body
            }
        }
    };
}

pure_effect! {Hello, struct {
    factor: f32,
}, next(self) {
    self.inner.next().map(|value| value * self.factor)
},
    pub fn test(&mut self) -> &str {
        dbg!("")
    },
    // pub fn test2(&self) -> &str {
    //     dbg!("")
    // }
}

pub(crate) use pure_effect;
