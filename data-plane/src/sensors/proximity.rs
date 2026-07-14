use log::info;

pub struct ProximitySensor;

impl ProximitySensor {
    pub fn new() -> Self {
        info!("Initializing proximity sensor interface");
        ProximitySensor
    }

    pub fn poll_distance_mm(&self) -> u32 {
        info!("Polling proximity sensor distance");
        // In a production system, this would read an ultrasonic or time-of-flight sensor.
        450
    }
}
