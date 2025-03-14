use crate::impl_enum_conversions;
#[derive(Clone)]
pub enum Location {
    VianaDoCastelo,
    Braga,
    Porto,
    VilaReal,
}

impl_enum_conversions!(Location, VianaDoCastelo => 0, Braga => 1, Porto => 2, VilaReal => 3);
