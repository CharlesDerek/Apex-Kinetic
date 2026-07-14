use log::info;

#[derive(Clone)]
pub struct Drv8835Controller {
    channel: String,
}

impl Drv8835Controller {
    pub fn new(channel: &str) -> Self {
        info!("Initializing DRV8835 motor controller on channel {}", channel);
        Drv8835Controller {
            channel: channel.to_string(),
        }
    }

    pub fn set_speed(&self, speed: i8) {
        info!("Setting DRV8835 motor {} speed to {}", self.channel, speed);
        // Hardware-specific PWM register writes would be implemented here.
    }

    pub fn stop(&self) {
        info!("Stopping DRV8835 motor {}", self.channel);
        self.set_speed(0);
    }
}
