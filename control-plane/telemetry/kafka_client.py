import asyncio
import logging
from aiokafka import AIOKafkaProducer

logger = logging.getLogger(__name__)

class KafkaPublisher:
    def __init__(self, bootstrap_servers: str):
        self.bootstrap_servers = bootstrap_servers
        self.producer = AIOKafkaProducer(bootstrap_servers=self.bootstrap_servers)

    async def start(self) -> None:
        logger.info("Starting Kafka producer for %s", self.bootstrap_servers)
        await self.producer.start()

    async def stop(self) -> None:
        logger.info("Stopping Kafka producer")
        await self.producer.stop()

    async def publish(self, topic: str, payload: str) -> None:
        logger.debug("Publishing event to %s", topic)
        await self.producer.send_and_wait(topic, payload.encode("utf-8"))
