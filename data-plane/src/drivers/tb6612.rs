use log::info;

use super::drv8835::{MotorDirection, MotorOutput};

pub const PIN_PWM_A: u8 = 5;
pub const PIN_PWM_B: u8 = 6;
pub const PIN_AIN_1: u8 = 7;
pub const PIN_BIN_1: u8 = 8;
pub const PIN_STBY: u8 = 3;
pub const MAX_SPEED: u8 = 255;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Tb6612DriveCommand {
    pub right: MotorOutput,
    pub left: MotorOutput,
    pub standby_high: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Tb6612Controller;

impl Tb6612Controller {
    pub fn new() -> Self {
        info!("Initializing TB6612 dual H-bridge controller");
        Self
    }

    pub fn set_speed(&self, left_speed: i8, right_speed: i8) -> Tb6612DriveCommand {
        info!(
            "Setting TB6612 left_speed={} right_speed={}",
            left_speed, right_speed
        );

        Self::drive(
            signed_direction(right_speed),
            signed_speed_to_pwm(right_speed),
            signed_direction(left_speed),
            signed_speed_to_pwm(left_speed),
            left_speed != 0 || right_speed != 0,
        )
    }

    pub fn drive(
        right_direction: MotorDirection,
        right_speed: u8,
        left_direction: MotorDirection,
        left_speed: u8,
        enabled: bool,
    ) -> Tb6612DriveCommand {
        if !enabled {
            return Tb6612DriveCommand {
                right: stopped_output(),
                left: stopped_output(),
                standby_high: false,
            };
        }

        let right = motor_output(right_direction, right_speed);
        let left = motor_output(left_direction, left_speed);
        let moving = right.pwm > 0 || left.pwm > 0;

        Tb6612DriveCommand {
            right,
            left,
            standby_high: moving,
        }
    }
}

fn signed_direction(speed: i8) -> MotorDirection {
    if speed > 0 {
        MotorDirection::Forward
    } else if speed < 0 {
        MotorDirection::Backward
    } else {
        MotorDirection::Coast
    }
}

fn signed_speed_to_pwm(speed: i8) -> u8 {
    speed.unsigned_abs().min(MAX_SPEED)
}

fn motor_output(direction: MotorDirection, speed: u8) -> MotorOutput {
    match direction {
        MotorDirection::Forward => MotorOutput {
            direction_pin_high: true,
            pwm: speed.min(MAX_SPEED),
        },
        MotorDirection::Backward => MotorOutput {
            direction_pin_high: false,
            pwm: speed.min(MAX_SPEED),
        },
        MotorDirection::Coast => stopped_output(),
    }
}

fn stopped_output() -> MotorOutput {
    MotorOutput {
        direction_pin_high: false,
        pwm: 0,
    }
}
