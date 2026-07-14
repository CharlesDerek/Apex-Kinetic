use log::info;

pub const PIN_VOLTAGE: &str = "A3";
pub const ADC_TO_VOLTS: f32 = 0.0375;
pub const COMPENSATION_FACTOR: f32 = 1.08;

#[derive(Clone, Debug, Default)]
pub struct VoltageDriver;

impl VoltageDriver {
    pub fn new() -> Self {
        info!("Initializing voltage sense input on pin {}", PIN_VOLTAGE);
        Self
    }

    pub fn volts_from_adc(adc_value: u16) -> f32 {
        adc_value as f32 * ADC_TO_VOLTS * COMPENSATION_FACTOR
    }
}
