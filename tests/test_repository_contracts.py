from pathlib import Path
import unittest

import yaml

ROOT = Path(__file__).resolve().parents[1]


def load_yaml(path: str) -> dict:
    with (ROOT / path).open(encoding="utf-8") as handle:
        return yaml.safe_load(handle)


def workload_container(manifest: dict) -> dict:
    return manifest["spec"]["template"]["spec"]["containers"][0]


class RepositoryContractsTest(unittest.TestCase):
    def test_kubernetes_workloads_use_controllers_with_probe_and_resource_contracts(self) -> None:
        workload_paths = sorted((ROOT / "config/k8s").glob("*-deployment.yaml"))
        workload_paths.extend(sorted((ROOT / "config/k8s").glob("*-daemonset.yaml")))

        self.assertEqual(len(workload_paths), 3)
        self.assertFalse(list((ROOT / "config/k8s").glob("*-pod.yaml")))

        workload_kinds = {}
        for manifest_path in workload_paths:
            manifest = load_yaml(str(manifest_path.relative_to(ROOT)))
            container = workload_container(manifest)
            role = manifest["metadata"]["labels"]["role"]
            workload_kinds[role] = manifest["kind"]

            self.assertIn(manifest["kind"], {"Deployment", "DaemonSet"})
            self.assertEqual(manifest["metadata"]["namespace"], "apex-kinetic")
            self.assertEqual(
                manifest["spec"]["selector"]["matchLabels"],
                manifest["spec"]["template"]["metadata"]["labels"],
            )
            self.assertIs(container["securityContext"]["runAsNonRoot"], True)
            self.assertIs(container["securityContext"]["allowPrivilegeEscalation"], False)
            self.assertTrue(container["image"].startswith("apex-kinetic/"))
            self.assertIn("readinessProbe", container)
            self.assertIn("livenessProbe", container)
            self.assertIn("requests", container["resources"])
            self.assertIn("limits", container["resources"])

        self.assertEqual(
            workload_kinds,
            {
                "control-plane": "Deployment",
                "data-plane": "DaemonSet",
                "vision-node": "Deployment",
            },
        )


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
        manifest_paths = {
            "control-plane": "config/k8s/control-plane-deployment.yaml",
            "data-plane": "config/k8s/data-plane-daemonset.yaml",
            "vision-node": "config/k8s/vision-node-deployment.yaml",
        }

        for service, manifest_path in manifest_paths.items():
            dockerfile = ROOT / service / "Dockerfile"

            self.assertTrue(dockerfile.exists(), f"{service} is missing a Dockerfile")
            self.assertIn(
                f"apex-kinetic/{service}:latest",
                workload_container(load_yaml(manifest_path))["image"],
            )


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
