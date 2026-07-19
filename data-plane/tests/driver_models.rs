use apex_kinetic_data_plane::drivers::{
    drv8835::{Drv8835Controller, MotorDirection, MotorOutput},
    ir_receiver::{decode_nec_value, IrCommand},
    key::KeyDriver,
    mpu6050::{Mpu6050Sensor, RawMotion},
    servo::{ServoAxis, ServoDriver},
    tb6612::Tb6612Controller,
    ultrasonic::UltrasonicDriver,
    voltage::VoltageDriver,
};

#[test]
fn drv8835_maps_signed_speed_to_channel_outputs() {
    let right = Drv8835Controller::new("right").set_speed(80);

    assert!(right.enabled);
    assert_eq!(
        right.right,
        MotorOutput {
            direction_pin_high: false,
            pwm: 80
        }
    );
    assert_eq!(right.left.pwm, 0);

    let left_reverse = Drv8835Controller::new("left").set_speed(-30);
    assert_eq!(
        left_reverse.left,
        MotorOutput {
            direction_pin_high: false,
            pwm: 30
        }
    );
}

#[test]
fn tb6612_uses_standby_only_when_drive_outputs_move() {
    let controller = Tb6612Controller::new();

    let stopped = controller.set_speed(0, 0);
    assert!(!stopped.standby_high);
    assert_eq!(stopped.left.pwm, 0);
    assert_eq!(stopped.right.pwm, 0);

    let moving = Tb6612Controller::drive(
        MotorDirection::Forward,
        64,
        MotorDirection::Backward,
        32,
        true,
    );
    assert!(moving.standby_high);
    assert_eq!(moving.right.pwm, 64);
    assert_eq!(moving.left.pwm, 32);
}

#[test]
fn key_driver_debounces_and_wraps_key_values() {
    let mut driver = KeyDriver::new();

    assert_eq!(driver.register_falling_edge(1_000), Some(1));
    assert_eq!(driver.register_falling_edge(1_100), None);
    assert_eq!(driver.key_value(), 1);

    assert_eq!(driver.register_falling_edge(1_501), Some(2));
    assert_eq!(driver.register_falling_edge(2_002), Some(3));
    assert_eq!(driver.register_falling_edge(2_503), Some(4));
    assert_eq!(driver.register_falling_edge(3_004), Some(0));
}

#[test]
fn imu_calibration_and_yaw_integration_are_deterministic() {
    let mut sensor = Mpu6050Sensor::new();
    sensor.calibrate_gyro_z([131, 132, 130, 131]);

    let first = sensor.metrics_from_raw(
        RawMotion {
            gyro_z: 262,
            ..RawMotion::default()
        },
        1_000,
    );
    assert_eq!(first.gyro_z_offset, 131);
    assert_eq!(first.yaw_degrees, 0.0);

    let second = sensor.metrics_from_raw(
        RawMotion {
            gyro_z: 262,
            ..RawMotion::default()
        },
        1_250,
    );
    assert!((second.yaw_degrees + 1.0).abs() < f32::EPSILON);
}

#[test]
fn peripheral_helpers_clamp_and_decode_values() {
    assert_eq!(UltrasonicDriver::centimeters_from_echo_us(8_700), 150);
    assert!((VoltageDriver::volts_from_adc(100) - 4.05).abs() < f32::EPSILON);
    assert_eq!(decode_nec_value(16_736_925), Some(IrCommand::Up));
    assert_eq!(decode_nec_value(0), None);

    let mut servo = ServoDriver::new(500);
    assert_eq!(servo.angles(), (180, 180));

    let command = servo.set_axis(ServoAxis::Both, 90);
    assert_eq!(command.axis, ServoAxis::Both);
    assert_eq!(servo.angles(), (90, 90));
}
