pub trait Ts {
    fn ts() -> String;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shape {
    pub name: String,
    pub fields: Vec<Field>,
}

macro_rules! scalar {
    ($ty:ty => $ts:literal) => {
        impl Ts for $ty {
            fn ts() -> String {
                $ts.to_owned()
            }
        }
    };
}

scalar!(String => "string");
scalar!(bool => "boolean");
scalar!(u32 => "number");
scalar!(i32 => "number");
scalar!(u64 => "number");
scalar!(f64 => "number");

impl<T: Ts> Ts for Vec<T> {
    fn ts() -> String {
        format!("{}[]", T::ts())
    }
}

impl<T: Ts> Ts for Option<T> {
    fn ts() -> String {
        format!("{} | null", T::ts())
    }
}
