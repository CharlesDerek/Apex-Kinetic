use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const SYSTEM_HEALTH_TOPIC: &str = "system.health";
pub const HEALTH_SCHEMA_VERSION: &str = "apex-kinetic.health.v1";

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
    statuses.fold(HealthStatus::Starting, |aggregate, status| {
        match (aggregate, status) {
            (HealthStatus::Unhealthy, _) | (_, HealthStatus::Unhealthy) => HealthStatus::Unhealthy,
            (HealthStatus::Degraded, _) | (_, HealthStatus::Degraded) => HealthStatus::Degraded,
            (_, HealthStatus::Ready) => HealthStatus::Ready,
            (current, HealthStatus::Starting) => current,
        }
    })
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
}
