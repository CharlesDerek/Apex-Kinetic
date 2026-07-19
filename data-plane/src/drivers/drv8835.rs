use log::info;

pub const PIN_PWM_A: u8 = 5;
pub const PIN_PWM_B: u8 = 6;
pub const PIN_BIN_1: u8 = 7;
pub const PIN_AIN_1: u8 = 8;
pub const MAX_SPEED: u8 = 255;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MotorDirection {
    Forward,
    Backward,
    Coast,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MotorOutput {
    pub direction_pin_high: bool,
    pub pwm: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Drv8835DriveCommand {
    pub right: MotorOutput,
    pub left: MotorOutput,
    pub enabled: bool,
}

#[derive(Clone, Debug)]
pub struct Drv8835Controller {
    channel: String,
}

impl Drv8835Controller {
    pub fn new(channel: &str) -> Self {
        info!(
            "Initializing DRV8835 motor controller on channel {}",
            channel
        );
        Self {
            channel: channel.to_string(),
        }
    }

    pub fn set_speed(&self, speed: i8) -> Drv8835DriveCommand {
        let direction = if speed > 0 {
            MotorDirection::Forward
        } else if speed < 0 {
            MotorDirection::Backward
        } else {
            MotorDirection::Coast
        };
        let pwm = signed_speed_to_pwm(speed);
        let output = self.output_for_channel(direction, pwm);

        info!(
            "Setting DRV8835 motor {} speed to {} with pwm={}",
            self.channel, speed, output.pwm
        );

        if self.channel.eq_ignore_ascii_case("right") {
            Drv8835DriveCommand {
                right: output,
                left: motor_b_output(MotorDirection::Coast, 0),
                enabled: output.pwm > 0,
            }
        } else {
            Drv8835DriveCommand {
                right: motor_a_output(MotorDirection::Coast, 0),
                left: output,
                enabled: output.pwm > 0,
            }
        }
    }

    pub fn drive(
        right_direction: MotorDirection,
        right_speed: u8,
        left_direction: MotorDirection,
        left_speed: u8,
        enabled: bool,
    ) -> Drv8835DriveCommand {
        if !enabled {
            return Drv8835DriveCommand {
                right: motor_a_output(MotorDirection::Coast, 0),
                left: motor_b_output(MotorDirection::Coast, 0),
                enabled: false,
            };
        }

        Drv8835DriveCommand {
            right: motor_a_output(right_direction, right_speed),
            left: motor_b_output(left_direction, left_speed),
            enabled: true,
        }
    }

    pub fn stop(&self) -> Drv8835DriveCommand {
        info!("Stopping DRV8835 motor {}", self.channel);
        self.set_speed(0)
    }

    fn output_for_channel(&self, direction: MotorDirection, pwm: u8) -> MotorOutput {
        if self.channel.eq_ignore_ascii_case("right") {
            motor_a_output(direction, pwm)
        } else {
            motor_b_output(direction, pwm)
        }
    }
}

fn signed_speed_to_pwm(speed: i8) -> u8 {
    speed.unsigned_abs()
}

fn motor_a_output(direction: MotorDirection, speed: u8) -> MotorOutput {
    match direction {
        MotorDirection::Forward => MotorOutput {
            direction_pin_high: false,
            pwm: speed,
        },
        MotorDirection::Backward => MotorOutput {
            direction_pin_high: true,
            pwm: speed,
        },
        MotorDirection::Coast => MotorOutput {
            direction_pin_high: false,
            pwm: 0,
        },
    }
}

fn motor_b_output(direction: MotorDirection, speed: u8) -> MotorOutput {
    match direction {
        MotorDirection::Forward => MotorOutput {
            direction_pin_high: true,
            pwm: speed,
        },
        MotorDirection::Backward => MotorOutput {
            direction_pin_high: false,
            pwm: speed,
        },
        MotorDirection::Coast => MotorOutput {
            direction_pin_high: false,
            pwm: 0,
        },
    }
}
