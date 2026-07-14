use log::info;

pub const PIN_RGB_LED: u8 = 4;
pub const NUM_LEDS: usize = 1;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RgbColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Clone, Debug)]
pub struct RgbLedDriver {
    brightness: u8,
    leds: [RgbColor; NUM_LEDS],
}

impl RgbLedDriver {
    pub fn new(brightness: u8) -> Self {
        info!(
            "Initializing RGB LED on pin {} with brightness {}",
            PIN_RGB_LED, brightness
        );
        Self {
            brightness,
            leds: [RgbColor::default(); NUM_LEDS],
        }
    }

    pub fn brightness(&self) -> u8 {
        self.brightness
    }

    pub fn set_color(&mut self, led_index: usize, color: RgbColor) {
        if led_index == NUM_LEDS {
            self.leds.fill(color);
        } else if let Some(led) = self.leds.get_mut(led_index) {
            *led = color;
        }
    }

    pub fn traversal(&mut self, traversal_count: usize, color: RgbColor) {
        let count = traversal_count.min(NUM_LEDS);
        for led in self.leds.iter_mut().take(count) {
            *led = color;
        }
    }

    pub fn leds(&self) -> &[RgbColor; NUM_LEDS] {
        &self.leds
    }
}
