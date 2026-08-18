macro_rules! dto {
    ($name:ident { $($field:ident : $ty:ty),* $(,)? }) => {
        #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
        pub struct $name {
            $(pub $field: $ty),*
        }

        impl crate::ipc::shape::Ts for $name {
            fn ts() -> String {
                stringify!($name).to_owned()
            }
        }

        impl $name {
            pub fn shape() -> crate::ipc::shape::Shape {
                crate::ipc::shape::Shape {
                    name: stringify!($name).to_owned(),
                    fields: vec![$(crate::ipc::shape::Field {
                        name: stringify!($field).to_owned(),
                        ty: <$ty as crate::ipc::shape::Ts>::ts(),
                    }),*],
                }
            }
        }
    };
}

pub(crate) use dto;
