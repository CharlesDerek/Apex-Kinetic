mod drivers;
mod sensors;

use drivers::drv8835::Drv8835Controller;
use drivers::mpu6050::Mpu6050Sensor;
use drivers::tb6612::Tb6612Controller;
use sensors::proximity::ProximitySensor;
use std::{sync::{Arc, Mutex}, thread, time::Duration};

#[derive(Debug, Default)]
struct HardwareState {
    imu_status: String,
    proximity_status: String,
    left_motor_speed: i8,
    right_motor_speed: i8,
}

fn main() {
    env_logger::init();
    let state = Arc::new(Mutex::new(HardwareState::default()));

    let imu = Mpu6050Sensor::new();
    let proximity = ProximitySensor::new();
    let left_motor = Drv8835Controller::new("left");
    let right_motor = Drv8835Controller::new("right");
    let track_motor = Tb6612Controller::new();

    let left_motor_thread = left_motor.clone();
    let right_motor_thread = right_motor.clone();
    let track_motor_thread = track_motor.clone();
    let health_state = state.clone();
    thread::spawn(move || {
        loop {
            let imu_reading = imu.read_imu_metrics();
            let mut state = health_state.lock().unwrap();
            state.imu_status = imu_reading;
            thread::sleep(Duration::from_millis(250));
        }
    });

    let proximity_state = state.clone();
    thread::spawn(move || {
        loop {
            let distance = proximity.poll_distance_mm();
            let mut state = proximity_state.lock().unwrap();
            state.proximity_status = format!("distance_mm={}", distance);
            if distance < 300 {
                left_motor_thread.stop();
                right_motor_thread.stop();
                track_motor_thread.set_speed(0, 0);
                let mut state = proximity_state.lock().unwrap();
                state.left_motor_speed = 0;
                state.right_motor_speed = 0;
            }
            thread::sleep(Duration::from_millis(200));
        }
    });

    loop {
        left_motor.set_speed(80);
        right_motor.set_speed(80);
        track_motor.set_speed(64, 64);

        {
            let mut state = state.lock().unwrap();
            state.left_motor_speed = 80;
            state.right_motor_speed = 80;
            log::info!("Hardware state: {:?}", *state);
        }

        thread::sleep(Duration::from_secs(1));
    }
}
