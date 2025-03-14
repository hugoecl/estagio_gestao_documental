use crate::impl_enum_conversions;

pub enum Type {
    Addendum,
    Worker,
}

impl_enum_conversions!(Type, Addendum => 0, Worker => 1);
