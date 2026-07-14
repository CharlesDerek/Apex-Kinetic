# Project Apex-Kinetic

Project Apex-Kinetic is a hardened edge node framework that decouples legacy robotic controller primitives into a memory-safe, zero-trust architecture.

This repository contains the initial clean-room implementation for:

- `data-plane/`: Rust-based low-level runtime and hardware abstraction layer (HAL)
- `vision-node/`: Rust-based secure RTSP/mTLS ingress worker for edge video capture
- `control-plane/`: Python asyncio telemetry orchestration and Kafka ingestion
- `config/`: Kubernetes manifest and Kafka topic definitions for zero-trust deployment

This baseline strips hobbyist nomenclature and legacy dependencies, replacing them with explicit production-grade systems terminology and architecture.

## Data-Plane Driver Components

The data plane now includes Rust driver models translated from the legacy robot source with English-only names and comments:

- `drv8835`: DRV8835 motor controller pin map and direction/PWM command model
- `tb6612`: TB6612 dual H-bridge controller with standby handling
- `mpu6050`: MPU6050 IMU calibration constants and yaw integration helper
- `rgb_led`: single-pixel RGB LED state model
- `key`: debounced key input state model
- `line_tracking`: ITR20001 left, middle, and right line tracking readings
- `voltage`: battery voltage conversion and compensation formula
- `ultrasonic`: trigger/echo distance conversion for the ultrasonic sensor
- `servo`: two-axis servo command model with angle limits
- `ir_receiver`: NEC infrared command decode table

See `docs/drivers.md` for the driver inventory and migration notes.
