use log::info;

pub const I2C_ADDRESS: u8 = 0x68;
pub const DEVICE_ID_RETRIES: u8 = 10;
pub const CALIBRATION_SAMPLES: usize = 100;
pub const GYRO_Z_SENSITIVITY: f32 = 131.0;
pub const GYRO_DEADBAND_DEGREES: f32 = 0.05;
pub const TIME_COMPENSATION: f32 = 4.0;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RawMotion {
    pub accel_x: i16,
    pub accel_y: i16,
    pub accel_z: i16,
    pub gyro_x: i16,
    pub gyro_y: i16,
    pub gyro_z: i16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ImuMetrics {
    pub raw: RawMotion,
    pub yaw_degrees: f32,
    pub gyro_z_offset: i32,
}

#[derive(Clone, Debug)]
pub struct Mpu6050Sensor {
    last_time_ms: Option<u64>,
    yaw_degrees: f32,
    gyro_z_offset: i32,
}

impl Mpu6050Sensor {
    pub fn new() -> Self {
        info!(
            "Configuring MPU6050 IMU sensor at I2C address 0x{:02x}",
            I2C_ADDRESS
        );
        Self {
            last_time_ms: None,
            yaw_degrees: 0.0,
            gyro_z_offset: 0,
        }
    }

    pub fn calibrate_gyro_z<I>(&mut self, samples: I)
    where
        I: IntoIterator<Item = i16>,
    {
        let mut count = 0_i32;
        let mut total = 0_i32;

        for sample in samples.into_iter().take(CALIBRATION_SAMPLES) {
            count += 1;
            total += sample as i32;
        }

        if count > 0 {
            self.gyro_z_offset = total / count;
        }
    }

    pub fn update_yaw(&mut self, gyro_z: i16, now_ms: u64) -> f32 {
        let compensated_now_ms = (now_ms as f32 * TIME_COMPENSATION) as u64;
        let last_time_ms = self.last_time_ms.replace(compensated_now_ms);
        let Some(last_time_ms) = last_time_ms else {
            return self.yaw_degrees;
        };

        let dt = compensated_now_ms.saturating_sub(last_time_ms) as f32 / 1000.0;
        let mut delta_degrees =
            -((gyro_z as i32 - self.gyro_z_offset) as f32) / GYRO_Z_SENSITIVITY * dt;

        if delta_degrees.abs() < GYRO_DEADBAND_DEGREES {
            delta_degrees = 0.0;
        }

        self.yaw_degrees += delta_degrees;
        self.yaw_degrees
    }

    pub fn metrics_from_raw(&mut self, raw: RawMotion, now_ms: u64) -> ImuMetrics {
        let yaw_degrees = self.update_yaw(raw.gyro_z, now_ms);
        ImuMetrics {
            raw,
            yaw_degrees,
            gyro_z_offset: self.gyro_z_offset,
        }
    }

    pub fn read_imu_metrics(&self) -> String {
        info!("Reading cached MPU6050 IMU metrics");
        format!(
            "addr=0x{:02x}, yaw_degrees={:.2}, gyro_z_offset={}",
            I2C_ADDRESS, self.yaw_degrees, self.gyro_z_offset
        )
    }
}

impl Default for Mpu6050Sensor {
    fn default() -> Self {
        Self::new()
    }
}
