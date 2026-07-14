use log::info;

pub const PIN_SERVO_Z: u8 = 10;
pub const PIN_SERVO_Y: u8 = 11;
pub const MIN_PULSE_US: u16 = 500;
pub const MAX_PULSE_US: u16 = 2400;
pub const MIN_ANGLE_DEGREES: u16 = 0;
pub const MAX_ANGLE_DEGREES: u16 = 180;
pub const DEFAULT_SETTLE_MS: u64 = 500;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServoAxis {
    Z,
    Y,
    Both,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServoCommand {
    pub axis: ServoAxis,
    pub angle_degrees: u16,
    pub settle_ms: u64,
}

#[derive(Clone, Debug)]
pub struct ServoDriver {
    z_angle: u16,
    y_angle: u16,
}

impl ServoDriver {
    pub fn new(initial_angle: u16) -> Self {
        let angle = clamp_angle(initial_angle);
        info!(
            "Initializing servos on pins {} and {} at {} degrees",
            PIN_SERVO_Z, PIN_SERVO_Y, angle
        );
        Self {
            z_angle: angle,
            y_angle: angle,
        }
    }

    pub fn set_z(&mut self, angle_degrees: u16) -> ServoCommand {
        let angle_degrees = clamp_angle(angle_degrees);
        self.z_angle = angle_degrees;
        ServoCommand {
            axis: ServoAxis::Z,
            angle_degrees,
            settle_ms: 450,
        }
    }

    pub fn set_axis(&mut self, axis: ServoAxis, angle_degrees: u16) -> ServoCommand {
        let angle_degrees = clamp_angle(angle_degrees);
        match axis {
            ServoAxis::Z => self.z_angle = angle_degrees,
            ServoAxis::Y => self.y_angle = angle_degrees,
            ServoAxis::Both => {
                self.z_angle = angle_degrees;
                self.y_angle = angle_degrees;
            }
        }

        ServoCommand {
            axis,
            angle_degrees,
            settle_ms: DEFAULT_SETTLE_MS,
        }
    }

    pub fn angles(&self) -> (u16, u16) {
        (self.z_angle, self.y_angle)
    }
}

fn clamp_angle(angle_degrees: u16) -> u16 {
    angle_degrees.clamp(MIN_ANGLE_DEGREES, MAX_ANGLE_DEGREES)
}
