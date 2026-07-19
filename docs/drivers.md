# Data-Plane Hardware Abstraction Layer (HAL)

The data-plane driver layer decouples legacy bare-metal peripheral primitives into isolated, type-safe Rust modules under `data-plane/src/drivers`.

The current implementation enforces a **target-agnostic architecture**. It models explicit pin configurations, mathematical conversion formulas, deterministic state transitions, and downstream command outputs without anchoring to a single, concrete hardware-specific crate (e.g., target-specific GPIO, PWM, ADC, or I2C runtimes). This decoupled design ensures the core business and safety logic remains completely unit-testable in simulated or virtual environments while remaining pluggable for any downstream physical target board.

## Driver Engineering Inventory

| Module | Legacy Architecture Origin | Apex Framework Responsibility |
| --- | --- | --- |
| `drv8835` | Monolithic Motor Control | Encapsulates PWM register limits, directional truth tables, and independent channel handling parameters. |
| `tb6612` | H-Bridge Logic Gate | Implements dual-motor pulse-width modulation boundaries and synchronous standby state-machine tracking. |
| `mpu6050` | Inertial Measurement Unit | Houses fixed I2C addressing matrices, gyro-calibration baseline bounds, deadband filtration thresholds, and numerical yaw integration helpers. |
| `rgb_led` | Single-Node Indicator | Maintains isolated brightness vectors and discrete state representations for single-line digital pins. |
| `key` | Hardware Interrupt Vector | Provides debounced edge-trigger state registers, implementing a finite-state machine to track continuous sequential state steps (0–4). |
| `line_tracking` | Phototransistor Array | Models parallel analog signal input registers correlated to specific underlying tracking matrices. |
| `voltage` | Telemetry Input Stage | Preserves analog-to-digital conversion models alongside non-linear compensation formulas for voltage rail tracking. |
| `ultrasonic` | Transceiver Ingress Loop | Decouples raw signal timing arrays into real-time distance matrices, enforcing runtime-safety data limits. |
| `servo` | Actuation Interface | Constrains dual-axis angular coordinates within explicit physical boundary limits and settles delay windows. |
| `six_axis_arm` | Manipulator Expansion Interface | Constrains six-axis joint commands, per-axis angle limits, neutral poses, and command speed bounds. |
| `speaker` | Two-Way Audio Interface | Models volume, mute, sample-rate, channel count, and route selection for speaker and talkback sessions. |
| `tft_display` | Local Video Display Interface | Models TFT resolution, refresh rate, backlight, and mode changes for previews and two-way video calls. |
| `ir_receiver` | NEC Demodulator Loop | Re-maps chronological pulse sequences into a structured, typed command lookup vector. |
| `vision-node::rtsp_control` | RTSP Session Control Plane | Defines scheduled camera stream control messages for future Kafka-backed RTSP lifecycle automation. |

## Refactoring and Migration Directives

* **Clean-Room Standardization:** Removed all undocumented localized nomenclature and legacy inline notes, replacing them with explicit system documentation and standardized English comments.
* **Primitive Abstraction:** Bare-metal macro utilities (`digitalWrite`, `analogWrite`, `analogRead`, `pulseIn`) and tightly coupled hardware library dependencies are refactored entirely into decoupled, typed input readings and output command matrices.
* **Target-Agnostic Compilation:** Modules compile as isolated data structures within the standalone Rust data-plane toolchain, ready to accept board-specific system implementations without modifying underlying logical validation routines.
* **Module Ingress:** The `drivers/mod.rs` registry manages public domain module exports while selectively managing dead-code compilation profiles during ongoing architectural integration phases.

## Verification & Continuous Integration

Execute local unit verification tests directly within the data-plane crate to validate formula accuracy and state transformations:

```sh
cd data-plane
cargo test
```
