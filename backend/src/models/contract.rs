use crate::impl_enum_conversions;

#[derive(Clone)]
pub enum Status {
    Active,
    Inactive,
}

#[derive(Clone)]
pub enum Type {
    Addendum,
    New,
    Renew,
}

#[derive(Clone)]
pub enum Service {
    Electricity,
    Water,
    Cleaning,
    Printers,
    Comunications,
}

#[derive(Clone)]
pub enum Location {
    VianaDoCastelo,
    Braga,
    Porto,
    VilaReal,
}

impl_enum_conversions!(Status, Active => 0, Inactive => 1);
impl_enum_conversions!(Type, Addendum => 0, New => 1, Renew => 2);
impl_enum_conversions!(Service, Electricity => 0, Water => 1, Cleaning => 2, Printers => 3, Comunications => 4);
impl_enum_conversions!(Location, VianaDoCastelo => 0, Braga => 1, Porto => 2, VilaReal => 3);
