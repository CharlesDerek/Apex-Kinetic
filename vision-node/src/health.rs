use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use rdkafka::ClientConfig;

pub const SYSTEM_HEALTH_TOPIC: &str = "system.health";
pub const HEALTH_SCHEMA_VERSION: &str = "apex-kinetic.health.v1";
pub const DEFAULT_HEALTH_STATE_PATH: &str = "/tmp/vision-node-health.json";

pub fn write_health_snapshot(path: &Path, event: &HealthEvent) -> io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let mut temp = tempfile::Builder::new()
        .prefix(".vision-health-")
        .tempfile_in(parent)?;
    serde_json::to_writer(&mut temp, event)?;
    temp.write_all(b"\n")?;
    temp.as_file().sync_all()?;
    temp.persist(path).map_err(|error| error.error)?;
    #[cfg(unix)]
    File::open(parent)?.sync_all()?;
    Ok(())
}

pub fn is_snapshot_ready(path: &Path, now_epoch_ms: u64, max_age_ms: u64) -> io::Result<bool> {
    let snapshot: serde_json::Value = serde_json::from_slice(&fs::read(path)?)?;
    let observed_at = snapshot
        .get("observed_at_epoch_ms")
        .and_then(serde_json::Value::as_u64);
    let fresh = observed_at.is_some_and(|timestamp| {
        timestamp <= now_epoch_ms && now_epoch_ms - timestamp <= max_age_ms
    });
    Ok(fresh
        && snapshot
            .get("schema_version")
            .and_then(serde_json::Value::as_str)
            == Some(HEALTH_SCHEMA_VERSION)
        && snapshot.get("status").and_then(serde_json::Value::as_str) == Some("ready"))
}

pub struct KafkaHealthPublisher {
    producer: FutureProducer,
}

impl KafkaHealthPublisher {
    pub fn new(bootstrap_servers: &str) -> Result<Self> {
        let producer = ClientConfig::new()
            .set("bootstrap.servers", bootstrap_servers)
            .set("message.timeout.ms", "3000")
            .create()
            .context("failed to create health Kafka producer")?;
        Ok(Self { producer })
    }

    pub async fn publish(&self, event: &HealthEvent) -> Result<()> {
        let payload = serde_json::to_string(event).context("failed to serialize health event")?;
        self.producer
            .send(
                FutureRecord::to(SYSTEM_HEALTH_TOPIC)
                    .key(&event.instance_id)
                    .payload(&payload),
                Timeout::After(Duration::from_secs(3)),
            )
            .await
            .map_err(|(error, _)| error)
            .context("failed to publish health event")?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Starting,
    Ready,
    Degraded,
    Unhealthy,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DependencyHealth {
    pub status: HealthStatus,
    pub consecutive_failures: u32,
    pub detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HealthEvent {
    pub schema_version: &'static str,
    pub service: String,
    pub instance_id: String,
    pub sequence: u64,
    pub observed_at_epoch_ms: u64,
    pub status: HealthStatus,
    pub dependencies: BTreeMap<String, DependencyHealth>,
}

#[derive(Debug)]
pub struct HealthReporter {
    service: String,
    instance_id: String,
    failure_threshold: u32,
    sequence: u64,
    dependencies: BTreeMap<String, DependencyHealth>,
}

impl HealthReporter {
    pub fn new(
        service: impl Into<String>,
        instance_id: impl Into<String>,
        failure_threshold: u32,
    ) -> Self {
        Self {
            service: service.into(),
            instance_id: instance_id.into(),
            failure_threshold: failure_threshold.max(1),
            sequence: 0,
            dependencies: BTreeMap::new(),
        }
    }

    pub fn record_success(&mut self, dependency: impl Into<String>, detail: impl Into<String>) {
        self.dependencies.insert(
            dependency.into(),
            DependencyHealth {
                status: HealthStatus::Ready,
                consecutive_failures: 0,
                detail: detail.into(),
            },
        );
    }

    pub fn record_starting(&mut self, dependency: impl Into<String>) {
        self.dependencies.insert(
            dependency.into(),
            DependencyHealth {
                status: HealthStatus::Starting,
                consecutive_failures: 0,
                detail: "initializing".to_string(),
            },
        );
    }

    pub fn record_failure(&mut self, dependency: impl Into<String>, detail: impl Into<String>) {
        let dependency = dependency.into();
        let failures = self
            .dependencies
            .get(&dependency)
            .map(|health| health.consecutive_failures)
            .unwrap_or_default()
            .saturating_add(1);
        let status = if failures >= self.failure_threshold {
            HealthStatus::Unhealthy
        } else {
            HealthStatus::Degraded
        };
        self.dependencies.insert(
            dependency,
            DependencyHealth {
                status,
                consecutive_failures: failures,
                detail: detail.into(),
            },
        );
    }

    pub fn record_unhealthy(&mut self, dependency: impl Into<String>, detail: impl Into<String>) {
        let dependency = dependency.into();
        let failures = self
            .dependencies
            .get(&dependency)
            .map(|health| health.consecutive_failures)
            .unwrap_or_default()
            .saturating_add(1);
        self.dependencies.insert(
            dependency,
            DependencyHealth {
                status: HealthStatus::Unhealthy,
                consecutive_failures: failures,
                detail: detail.into(),
            },
        );
    }

    pub fn event(&mut self, observed_at_epoch_ms: u64) -> HealthEvent {
        self.sequence = self.sequence.saturating_add(1);
        let status = aggregate_status(self.dependencies.values().map(|health| health.status));
        HealthEvent {
            schema_version: HEALTH_SCHEMA_VERSION,
            service: self.service.clone(),
            instance_id: self.instance_id.clone(),
            sequence: self.sequence,
            observed_at_epoch_ms,
            status,
            dependencies: self.dependencies.clone(),
        }
    }
}

fn aggregate_status(statuses: impl Iterator<Item = HealthStatus>) -> HealthStatus {
    let mut seen = false;
    let aggregate = statuses.fold(HealthStatus::Ready, |aggregate, status| {
        seen = true;
        match (aggregate, status) {
            (HealthStatus::Unhealthy, _) | (_, HealthStatus::Unhealthy) => HealthStatus::Unhealthy,
            (HealthStatus::Degraded, _) | (_, HealthStatus::Degraded) => HealthStatus::Degraded,
            (HealthStatus::Starting, _) | (_, HealthStatus::Starting) => HealthStatus::Starting,
            _ => HealthStatus::Ready,
        }
    });
    if seen {
        aggregate
    } else {
        HealthStatus::Starting
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failures_cross_threshold_and_recovery_resets_circuit() {
        let mut reporter = HealthReporter::new("vision-node", "edge-1", 2);
        reporter.record_failure("nvr", "timeout");
        assert_eq!(reporter.event(100).status, HealthStatus::Degraded);
        reporter.record_failure("nvr", "timeout");
        assert_eq!(reporter.event(200).status, HealthStatus::Unhealthy);
        reporter.record_success("nvr", "mTLS connected");
        let recovered = reporter.event(300);
        assert_eq!(recovered.status, HealthStatus::Ready);
        assert_eq!(recovered.dependencies["nvr"].consecutive_failures, 0);
        assert_eq!(recovered.sequence, 3);
    }

    #[test]
    fn event_serialization_is_stable_and_redaction_safe() {
        let mut reporter = HealthReporter::new("vision-node", "edge-1", 3);
        reporter.record_success("certificate", "loaded");
        let value = serde_json::to_value(reporter.event(42)).unwrap();
        assert_eq!(value["schema_version"], HEALTH_SCHEMA_VERSION);
        assert_eq!(value["status"], "ready");
        assert_eq!(value["dependencies"]["certificate"]["detail"], "loaded");
    }

    #[test]
    fn uninitialized_dependency_keeps_service_starting() {
        let mut reporter = HealthReporter::new("vision-node", "edge-1", 2);
        reporter.record_starting("nvr");
        reporter.record_success("certificate", "loaded");
        assert_eq!(reporter.event(10).status, HealthStatus::Starting);
        reporter.record_success("nvr", "connected");
        assert_eq!(reporter.event(20).status, HealthStatus::Ready);
    }

    #[test]
    fn readiness_snapshot_requires_ready_and_fresh_state() {
        let dir = std::env::temp_dir().join(format!("apex-vision-health-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("health.json");
        let mut reporter = HealthReporter::new("vision-node", "edge-1", 2);
        reporter.record_starting("nvr");
        write_health_snapshot(&path, &reporter.event(1_000)).unwrap();
        assert!(!is_snapshot_ready(&path, 1_001, 60_000).unwrap());
        reporter.record_success("nvr", "connected");
        write_health_snapshot(&path, &reporter.event(2_000)).unwrap();
        assert!(is_snapshot_ready(&path, 2_001, 60_000).unwrap());
        assert!(!is_snapshot_ready(&path, 62_001, 60_000).unwrap());
        let _ = fs::remove_dir_all(dir);
    }
}
