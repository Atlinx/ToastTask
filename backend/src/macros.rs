#[macro_export]
macro_rules! name_of {
    ($name:ident in $ty:ty) => {{
        #[allow(dead_code)]
        fn dummy(v: $ty) {
            let _ = &v.$name;
        }
        stringify!($name)
    }};

    ($name:ident) => {{
        let _ = &$name;
        stringify!($name)
    }};
}

pub use name_of;
