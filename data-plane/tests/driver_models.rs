use apex_kinetic_data_plane::drivers::{
    drv8835::{Drv8835Controller, MotorDirection, MotorOutput},
    ir_receiver::{decode_nec_value, IrCommand},
    key::KeyDriver,
    mpu6050::{Mpu6050Sensor, RawMotion},
    servo::{ServoAxis, ServoDriver},
    six_axis_arm::{ArmAxis, SixAxisArmDriver, ARM_CONTROL_TOPIC, ARM_STATUS_TOPIC},
    speaker::{AudioRoute, SpeakerConfig, SpeakerDriver, AUDIO_CONTROL_TOPIC, AUDIO_STATUS_TOPIC},
    tb6612::Tb6612Controller,
    tft_display::{
        DisplayMode, TftDisplayConfig, TftDisplayDriver, DISPLAY_CONTROL_TOPIC,
        DISPLAY_STATUS_TOPIC,
    },
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

#[test]
fn speaker_driver_clamps_volume_and_tracks_talkback_route() {
    let mut speaker = SpeakerDriver::new(SpeakerConfig::new(48_000, 2, 120));

    assert_eq!(AUDIO_CONTROL_TOPIC, "audio.control");
    assert_eq!(AUDIO_STATUS_TOPIC, "audio.status");
    assert_eq!(speaker.state().config.volume_percent, 100);
    assert_eq!(speaker.state().config.channels, 2);

    let routed = speaker.route_audio(AudioRoute::TwoWayCall);
    assert_eq!(routed.route, AudioRoute::TwoWayCall);

    let muted = speaker.set_muted(true);
    assert!(muted.muted);

    let quiet = speaker.set_volume(15);
    assert_eq!(quiet.config.volume_percent, 15);
}

#[test]
fn six_axis_arm_clamps_joint_targets_and_speed() {
    let arm = SixAxisArmDriver::default();

    assert_eq!(ARM_CONTROL_TOPIC, "arm.control");
    assert_eq!(ARM_STATUS_TOPIC, "arm.status");

    let shoulder = arm.command(ArmAxis::Shoulder, 180, 150);
    assert_eq!(shoulder.target_degrees, 120);
    assert_eq!(shoulder.speed_percent, 100);

    let gripper = arm.command(ArmAxis::Gripper, -45, 25);
    assert_eq!(gripper.target_degrees, 0);
    assert_eq!(gripper.speed_percent, 25);

    let neutral = arm.neutral_pose(10);
    assert_eq!(neutral.joints.len(), 6);
    assert!(neutral.joints.iter().all(|joint| joint.speed_percent == 10));
}

#[test]
fn tft_display_tracks_video_call_mode_and_backlight_limits() {
    let mut display = TftDisplayDriver::new(TftDisplayConfig::new(0, 240, 0));

    assert_eq!(DISPLAY_CONTROL_TOPIC, "display.control");
    assert_eq!(DISPLAY_STATUS_TOPIC, "display.status");
    assert_eq!(display.state().config.width_px, 1);
    assert_eq!(display.state().config.refresh_hz, 1);

    let call = display.set_mode(DisplayMode::RemoteVideoCall);
    assert_eq!(call.mode, DisplayMode::RemoteVideoCall);

    let bright = display.set_backlight(200);
    assert_eq!(bright.backlight_percent, 100);
}
