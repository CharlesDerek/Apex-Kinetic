mod drivers;
mod sensors;

use apex_kinetic_data_plane::safety::SafetyController;
use drivers::drv8835::Drv8835Controller;
use drivers::mpu6050::Mpu6050Sensor;
use drivers::tb6612::Tb6612Controller;
use sensors::proximity::ProximitySensor;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

#[derive(Debug, Default)]
struct HardwareState {
    imu_status: String,
    proximity_status: String,
    left_motor_speed: i8,
    right_motor_speed: i8,
    proximity_mm: Option<u32>,
}

fn main() {
    env_logger::init();
    let state = Arc::new(Mutex::new(HardwareState::default()));

    let imu = Mpu6050Sensor::new();
    let proximity = ProximitySensor::new();
    let left_motor = Drv8835Controller::new("left");
    let right_motor = Drv8835Controller::new("right");
    let track_motor = Tb6612Controller::new();

    let controller = SafetyController::new(500, 11_000, 300);
    let started = std::time::Instant::now();
    let health_state = state.clone();
    thread::spawn(move || loop {
        let imu_reading = imu.read_imu_metrics();
        health_state.lock().unwrap().imu_status = imu_reading;
        thread::sleep(Duration::from_millis(250));
    });

    let proximity_state = state.clone();
    thread::spawn(move || loop {
        let distance = proximity.poll_distance_mm();
        let mut state = proximity_state.lock().unwrap();
        state.proximity_status = format!("distance_mm={}", distance);
        state.proximity_mm = Some(distance);
        drop(state);
        thread::sleep(Duration::from_millis(200));
    });

    loop {
        let distance = state.lock().unwrap().proximity_mm;
        // No controller or voltage adapter is attached in this modeled runtime.
        // The authority gate therefore emits stop commands only.
        let decision =
            controller.decide(started.elapsed().as_millis() as u64, false, None, distance);
        left_motor.set_speed(decision.left);
        right_motor.set_speed(decision.right);
        track_motor.set_speed(decision.left, decision.right);
        {
            let mut state = state.lock().unwrap();
            state.left_motor_speed = decision.left;
            state.right_motor_speed = decision.right;
            log::info!("Hardware state: {:?}", *state);
        }

        thread::sleep(Duration::from_secs(1));
    }
}
