# Future Integration Plans

This document tracks forward-looking integration surfaces that are intentionally modeled before hardware-specific runtime bindings are selected. The goal is to keep message contracts, safety constraints, and local simulation tests stable while later board, camera, and display adapters are added behind those contracts.

## RTSP Communication and Programmatic Controls

The `vision-node` crate owns RTSP ingress behavior and now includes `rtsp_control` command models for future Kafka-backed control loops.

Planned runtime flow:

1. `control-plane` publishes scheduled stream intents to `rtsp.schedule`.
2. A scheduler worker resolves due entries and emits executable commands to `rtsp.control`.
3. `vision-node` consumes `rtsp.control` and maps typed actions to RTSP session lifecycle operations.
4. Correlation IDs are carried in Kafka message keys and log contexts so operational events can be traced across scheduling, dispatch, and stream handling.

Initial command types:

- `StartStream`
- `StopStream`
- `RestartStream`
- `EnableTalkback`
- `DisableTalkback`

The current implementation is intentionally transport-neutral. It models topic names, command keys, scheduled execution checks, and due-command filtering without requiring a live Kafka broker or camera in CI.

## Two-Way Speaker and Talkback Audio

The data-plane now includes a `speaker` driver model for future speaker, amplifier, and talkback routing integrations.

Planned runtime flow:

1. `control-plane` publishes speaker commands to `audio.control`.
2. `data-plane` consumes routed commands and applies target-specific I2S, DAC, amplifier, or GPIO operations behind the typed driver model.
3. `data-plane` reports local speaker state and talkback session health to `audio.status`.
4. `vision-node` coordinates talkback enablement with RTSP session state so camera video and remote audio permissions stay aligned.

Initial control surfaces:

- Volume clamping from `0..=100`
- Mute and unmute state transitions
- Local speaker, remote talkback, and two-way call routing
- Sample-rate and channel-count configuration

The initial model deliberately avoids naming a concrete audio chip. A physical adapter can later map `SpeakerState` into ALSA, I2S, USB audio, or board-specific amplifier calls.

## Six-Axis Arm Expansion

The data-plane now includes a `six_axis_arm` model that constrains future manipulator commands before any specific servo bus, CAN controller, or PWM expansion board is selected.

Planned runtime flow:

1. `control-plane` publishes joint commands or higher-level poses to `arm.control`.
2. `data-plane` clamps every target against configured per-axis limits before dispatching to hardware.
3. `data-plane` emits pose and limit state to `arm.status`.
4. Future safety logic can reject conflicting arm motion when proximity or platform-motion telemetry indicates an unsafe workspace.

The initial supported axes are base, shoulder, elbow, wrist pitch, wrist roll, and gripper. Default limits are documented in `config/hardware/future-peripherals.yaml`.

## TFT Screen and Two-Way Video Calls

The data-plane now includes a `tft_display` model for a future local screen used by camera previews and two-way calls.

Planned runtime flow:

1. `control-plane` publishes screen commands to `display.control`.
2. `vision-node` negotiates remote video-call media and emits display mode changes.
3. `data-plane` applies target-specific frame-buffer, SPI, DSI, or HDMI updates behind the typed TFT model.
4. `data-plane` publishes display health and current mode to `display.status`.

Initial screen modes:

- `Standby`
- `LocalPreview`
- `RemoteVideoCall`

The initial configuration assumes a 480x320 TFT at 30 Hz and keeps the feature disabled until a physical display adapter is selected.
