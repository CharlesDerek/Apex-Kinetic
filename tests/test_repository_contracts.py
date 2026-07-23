from pathlib import Path
import unittest

import yaml

ROOT = Path(__file__).resolve().parents[1]


def load_yaml(path: str) -> dict:
    with (ROOT / path).open(encoding="utf-8") as handle:
        return yaml.safe_load(handle)


def read_text(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


class RepositoryContractsTest(unittest.TestCase):
    def test_opentofu_is_the_single_kubernetes_deployment_source(self) -> None:
        static_kubernetes_manifests = list((ROOT / "config").glob("k8s/*.yaml"))

        self.assertEqual(static_kubernetes_manifests, [])

        module = read_text("infra/opentofu/app-k8s/main.tf")
        self.assertIn('resource "kubernetes_deployment_v1" "workload"', module)
        self.assertIn('resource "kubernetes_daemon_set_v1" "workload"', module)
        self.assertIn('resource "kubernetes_network_policy_v1" "zero_trust"', module)
        self.assertIn("readiness_probe", module)
        self.assertIn("liveness_probe", module)
        self.assertIn("resources", module)
        self.assertIn("run_as_non_root", module)
        self.assertIn("allow_privilege_escalation = false", module)

    def test_opentofu_network_policy_keeps_default_deny_with_named_exceptions(self) -> None:
        module = read_text("infra/opentofu/app-k8s/main.tf")

        self.assertIn("policy_types = [\"Ingress\", \"Egress\"]", module)
        self.assertIn('role = "kafka"', module)
        self.assertIn('role = "vision-node"', module)
        self.assertIn('port     = "9092"', module)
        self.assertIn('port     = "554"', module)
        self.assertNotIn("cidr", module)


    def test_ci_workflow_runs_rust_python_kubernetes_and_opentofu_checks(self) -> None:
        workflow = load_yaml(".github/workflows/ci.yml")
        jobs = workflow["jobs"]

        self.assertIn("rust", jobs)
        self.assertIn("repository-contracts", jobs)
        self.assertIn("container-images", jobs)
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

        image_commands = "\n".join(
            step.get("run", "") for step in jobs["container-images"]["steps"] if "run" in step
        )
        self.assertIn("docker build", image_commands)
        self.assertEqual(
            set(jobs["container-images"]["strategy"]["matrix"]["service"]),
            {"control-plane", "data-plane", "vision-node"},
        )

    def test_all_workloads_have_container_build_definitions(self) -> None:
        module = read_text("infra/opentofu/app-k8s/main.tf")

        for service in ("control-plane", "data-plane", "vision-node"):
            dockerfile = ROOT / service / "Dockerfile"

            self.assertTrue(dockerfile.exists(), f"{service} is missing a Dockerfile")
            self.assertIn(f"apex-kinetic/{service}:${{var.image_tag}}", module)


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
                "arm.control",
                "arm.status",
                "display.control",
                "display.status",
            },
            topic_names,
        )
        self.assertTrue(all(topic["partitions"] >= 1 for topic in topics))
        self.assertTrue(all(topic["replication_factor"] >= 1 for topic in topics))

    def test_future_peripheral_config_matches_control_topics(self) -> None:
        config = load_yaml("config/hardware/future-peripherals.yaml")["peripherals"]

        self.assertEqual(config["six_axis_arm"]["command_topic"], "arm.control")
        self.assertEqual(config["six_axis_arm"]["status_topic"], "arm.status")
        self.assertEqual(len(config["six_axis_arm"]["axes"]), 6)
        self.assertFalse(config["six_axis_arm"]["enabled"])

        display = config["tft_display"]
        self.assertEqual(display["command_topic"], "display.control")
        self.assertEqual(display["status_topic"], "display.status")
        self.assertEqual((display["width_px"], display["height_px"]), (480, 320))
        self.assertIn("remote_video_call", display["supported_modes"])
        self.assertFalse(display["enabled"])
