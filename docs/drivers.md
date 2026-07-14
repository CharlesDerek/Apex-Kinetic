# Data-Plane Drivers

The data-plane driver layer remakes the legacy robot hardware primitives as Rust modules under `data-plane/src/drivers`.

The current implementation intentionally models pin mappings, conversion formulas, state transitions, and command outputs without binding to a concrete GPIO, PWM, ADC, I2C, servo, RGB LED, or infrared hardware crate. This keeps the translated logic testable and leaves the hardware access layer replaceable for the target board.

## Driver Inventory

| Module | Legacy component | Apex responsibility |
| --- | --- | --- |
| `drv8835` | DRV8835 motor driver | Captures PWM pins, direction pins, speed limits, and right/left motor direction behavior. |
| `tb6612` | TB6612 dual H-bridge | Captures PWM pins, direction pins, standby pin behavior, and dual-motor drive commands. |
| `mpu6050` | MPU6050 IMU helper | Captures I2C address, retry count, gyro calibration sample count, Z-axis deadband, timing compensation, and yaw integration. |
| `rgb_led` | Onboard RGB LED | Tracks brightness and RGB state for the single LED on pin 4. |
| `key` | Mode/input key | Tracks debounced falling-edge input and cycles the key value from 0 through 4. |
| `line_tracking` | ITR20001 tracking sensors | Models left, middle, and right analog readings using the A2, A1, and A0 pin mapping. |
| `voltage` | Battery voltage input | Preserves the ADC-to-voltage scale and compensation factor from the old code. |
| `ultrasonic` | Ultrasonic distance sensor | Models trigger/echo pins and converts echo pulse time to centimeters with the legacy reporting cap. |
| `servo` | Two-axis servo assembly | Models Z/Y servo pins, pulse range, angle limits, and settle timing. |
| `ir_receiver` | NEC infrared receiver | Preserves the remote control decode table as typed commands. |

## Migration Notes

- All Chinese comments from the legacy source were removed or replaced with English descriptions.
- Hardware-specific calls such as `digitalWrite`, `analogWrite`, `analogRead`, `pulseIn`, and library-specific Arduino objects are represented as typed command or reading structures.
- The modules compile as part of the Rust data plane today and can be connected to a board-specific HAL later.
- `drivers/mod.rs` exports all driver modules and suppresses dead-code warnings while hardware integration is still incomplete.

## Verification

Run the data-plane checks from the Apex repository:

```sh
cd data-plane
cargo test
```
