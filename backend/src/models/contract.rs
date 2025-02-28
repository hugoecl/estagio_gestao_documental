use std::str::FromStr;

pub enum Status {
    Active,
    Inactive,
}

impl FromStr for Status {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "0" => Ok(Status::Active),
            "1" => Ok(Status::Inactive),
            _ => Err(()),
        }
    }
}

pub enum Types {
    Addendum,
    New,
    Renew,
}

impl FromStr for Types {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "0" => Ok(Types::Addendum),
            "1" => Ok(Types::New),
            "2" => Ok(Types::Renew),
            _ => Err(()),
        }
    }
}

pub enum Services {
    Electricity,
    Water,
    Cleaning,
    Printers,
    Comunications,
}

impl FromStr for Services {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "0" => Ok(Services::Electricity),
            "1" => Ok(Services::Water),
            "2" => Ok(Services::Cleaning),
            "3" => Ok(Services::Printers),
            "4" => Ok(Services::Comunications),
            _ => Err(()),
        }
    }
}

pub enum Locations {
    VianaDoCastelo,
    Braga,
    Porto,
    VilaReal,
}

impl FromStr for Locations {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "0" => Ok(Locations::VianaDoCastelo),
            "1" => Ok(Locations::Braga),
            "2" => Ok(Locations::Porto),
            "3" => Ok(Locations::VilaReal),
            _ => Err(()),
        }
    }
}
