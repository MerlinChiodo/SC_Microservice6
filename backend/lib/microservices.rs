use std::fmt::{Display, Formatter, write};

pub enum Microservices {
    Buergerbuero = 1,
    Stadtbus = 2,
    Kita = 3,
    Forum = 4,
    Tierheim = 5,
    SmartAuth = 6,
    Fitnessstudio = 7,
    Finanzamt = 8,
    Integration = 9,
    LandingPage = 10,
}

impl Display for Microservices {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Microservices::Buergerbuero => write!(f, "Bürgerbüro"),
            Microservices::Stadtbus => write!(f, "Stadtbus"),
            Microservices::Kita => write!(f, "Kita"),
            Microservices::Forum => write!(f, "Forum"),
            Microservices::Tierheim => write!(f, "Tierheim"),
            Microservices::SmartAuth => write!(f, "SmartAuth"),
            Microservices::Fitnessstudio => write!(f, "Fitnessstudio"),
            Microservices::Finanzamt => write!(f, "Finanzamt"),
            Microservices::Integration => write!(f, "Integration"),
            Microservices::LandingPage => write!(f, "LandingPage")
        }
    }
}
