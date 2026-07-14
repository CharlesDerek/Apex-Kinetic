# Project Apex-Kinetic

Project Apex-Kinetic is a hardened edge node framework that decouples legacy robotic controller primitives into a memory-safe, zero-trust architecture.

This repository contains the initial clean-room implementation for:

- `data-plane/`: Rust-based low-level runtime and hardware abstraction layer (HAL)
- `vision-node/`: Rust-based secure RTSP/mTLS ingress worker for edge video capture
- `control-plane/`: Python asyncio telemetry orchestration and Kafka ingestion
- `config/`: Kubernetes manifest and Kafka topic definitions for zero-trust deployment

This baseline strips hobbyist nomenclature and legacy dependencies, replacing them with explicit production-grade systems terminology and architecture.
