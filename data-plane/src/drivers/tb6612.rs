use log::info;

#[derive(Clone)]
pub struct Tb6612Controller;

impl Tb6612Controller {
    pub fn new() -> Self {
        info!("Initializing TB6612 dual H-bridge controller");
        Tb6612Controller
    }

    pub fn set_speed(&self, left_speed: i8, right_speed: i8) {
        info!("Setting TB6612 left_speed={} right_speed={}", left_speed, right_speed);
        // In a production node, this would update PWM registers and motor direction pins.
    }
}
