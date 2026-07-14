import asyncio
import json
import logging
import os
import time
from dataclasses import asdict, dataclass

from telemetry.kafka_client import KafkaPublisher

logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(message)s")

@dataclass
class TelemetrySample:
    timestamp: float
    source: str
    payload: dict

async def collect_device_telemetry() -> list[TelemetrySample]:
    samples = []
    samples.append(TelemetrySample(timestamp=time.time(), source="imu", payload=sample_imu_metrics()))
    samples.append(TelemetrySample(timestamp=time.time(), source="proximity", payload=sample_proximity_metrics()))
    return samples

def sample_imu_metrics() -> dict:
    return {
        "accel_x_g": 0.00,
        "accel_y_g": 0.00,
        "accel_z_g": 1.00,
        "gyro_x_dps": 0.00,
        "gyro_y_dps": 0.00,
        "gyro_z_dps": 0.00,
    }

def sample_proximity_metrics() -> dict:
    return {
        "distance_mm": 420,
        "object_detected": False,
        "sensor_status": "ok",
    }

async def run_event_loop() -> None:
    bootstrap_servers = os.getenv("KAFKA_BOOTSTRAP_SERVERS", "localhost:9092")
    publisher = KafkaPublisher(bootstrap_servers=bootstrap_servers)
    await publisher.start()

    try:
        while True:
            entries = await collect_device_telemetry()
            for entry in entries:
                topic = "imu.telemetry" if entry.source == "imu" else "proximity.telemetry"
                await publisher.publish(topic, json.dumps(asdict(entry)))
            await asyncio.sleep(1.0)
    finally:
        await publisher.stop()

if __name__ == "__main__":
    asyncio.run(run_event_loop())
