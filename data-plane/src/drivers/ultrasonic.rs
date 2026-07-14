use log::info;

pub const TRIG_PIN: u8 = 13;
pub const ECHO_PIN: u8 = 12;
pub const MAX_DISTANCE_CM: u16 = 200;
pub const REPORTED_DISTANCE_LIMIT_CM: u16 = 150;

#[derive(Clone, Debug, Default)]
pub struct UltrasonicDriver;

impl UltrasonicDriver {
    pub fn new() -> Self {
        info!(
            "Initializing ultrasonic sensor with trigger pin {} and echo pin {}",
            TRIG_PIN, ECHO_PIN
        );
        Self
    }

    pub fn centimeters_from_echo_us(echo_high_us: u32) -> u16 {
        let distance = (echo_high_us / 58) as u16;
        distance.min(REPORTED_DISTANCE_LIMIT_CM)
    }

    pub fn poll_distance_mm(&self) -> u32 {
        info!("Polling ultrasonic sensor distance");
        450
    }
}
