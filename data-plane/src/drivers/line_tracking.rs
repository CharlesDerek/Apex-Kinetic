use log::info;

pub const PIN_LEFT: &str = "A2";
pub const PIN_MIDDLE: &str = "A1";
pub const PIN_RIGHT: &str = "A0";

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LineTrackingReading {
    pub left: u16,
    pub middle: u16,
    pub right: u16,
}

#[derive(Clone, Debug, Default)]
pub struct Itr20001Driver;

impl Itr20001Driver {
    pub fn new() -> Self {
        info!("Initializing ITR20001 line tracking sensors on analog pins A2, A1, A0");
        Self
    }

    pub fn reading_from_adc(left: u16, middle: u16, right: u16) -> LineTrackingReading {
        LineTrackingReading {
            left,
            middle,
            right,
        }
    }
}
