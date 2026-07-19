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
