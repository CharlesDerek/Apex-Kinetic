from pathlib import Path
import unittest

import yaml

ROOT = Path(__file__).resolve().parents[1]


def load_yaml(path: str) -> dict:
    with (ROOT / path).open(encoding="utf-8") as handle:
        return yaml.safe_load(handle)


class RepositoryContractsTest(unittest.TestCase):
    def test_kubernetes_pods_run_as_non_root_without_privilege_escalation(self) -> None:
        for manifest_path in sorted((ROOT / "config/k8s").glob("*-pod.yaml")):
            manifest = load_yaml(str(manifest_path.relative_to(ROOT)))
            container = manifest["spec"]["containers"][0]

            self.assertEqual(manifest["kind"], "Pod")
            self.assertEqual(manifest["metadata"]["namespace"], "apex-kinetic")
            self.assertIs(container["securityContext"]["runAsNonRoot"], True)
            self.assertIs(container["securityContext"]["allowPrivilegeEscalation"], False)
            self.assertTrue(container["image"].startswith("apex-kinetic/"))


    def test_network_policy_keeps_default_deny_with_named_exceptions(self) -> None:
        manifest = load_yaml("config/k8s/network-policy.yaml")

        self.assertEqual(manifest["kind"], "NetworkPolicy")
        self.assertEqual(manifest["spec"]["podSelector"], {})
        self.assertEqual(set(manifest["spec"]["policyTypes"]), {"Ingress", "Egress"})
        self.assertTrue(
            any(
                port["port"] == 9092
                for rule in manifest["spec"]["egress"]
                for port in rule.get("ports", [])
            )
        )
        self.assertTrue(
            any(
                port["port"] == 554
                for rule in manifest["spec"]["egress"]
                for port in rule.get("ports", [])
            )
        )


    def test_ci_workflow_runs_rust_python_kubernetes_and_opentofu_checks(self) -> None:
        workflow = load_yaml(".github/workflows/ci.yml")
        jobs = workflow["jobs"]

        self.assertIn("rust", jobs)
        self.assertIn("repository-contracts", jobs)
        self.assertIn("opentofu", jobs)

        rust_commands = "\n".join(
            step.get("run", "") for step in jobs["rust"]["steps"] if "run" in step
        )
        self.assertIn("cargo fmt", rust_commands)
        self.assertIn("cargo clippy", rust_commands)
        self.assertIn("cargo test", rust_commands)

        tofu_commands = "\n".join(
            step.get("run", "") for step in jobs["opentofu"]["steps"] if "run" in step
        )
        self.assertIn("tofu fmt -check", tofu_commands)
        self.assertIn("tofu validate", tofu_commands)
        self.assertIn("tofu test", tofu_commands)


    def test_kafka_topic_contract_matches_control_plane_publishers(self) -> None:
        topics = load_yaml("config/kafka/topics.yaml")["topics"]
        topic_names = {topic["name"] for topic in topics}

        self.assertLessEqual(
            {
                "imu.telemetry",
                "proximity.telemetry",
                "system.health",
                "rtsp.control",
                "rtsp.schedule",
                "audio.control",
                "audio.status",
            },
            topic_names,
        )
        self.assertTrue(all(topic["partitions"] >= 1 for topic in topics))
        self.assertTrue(all(topic["replication_factor"] >= 1 for topic in topics))
