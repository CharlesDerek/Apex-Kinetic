locals {
  labels = {
    app = "apex-kinetic"
  }

  workloads = {
    control-plane = {
      image   = "apex-kinetic/control-plane:${var.image_tag}"
      command = ["python", "/app/main.py"]
      env = {
        KAFKA_BOOTSTRAP_SERVERS = "kafka:9092"
      }
    }
    data-plane = {
      image   = "apex-kinetic/data-plane:${var.image_tag}"
      command = ["/usr/local/bin/data-plane"]
      env     = {}
    }
    vision-node = {
      image   = "apex-kinetic/vision-node:${var.image_tag}"
      command = ["/usr/local/bin/vision-node"]
      env = {
        RTSP_SOURCE_URL = "rtsp://edge-camera.local/stream"
        NVR_TARGET_HOST = "annke-nvr.local"
        NVR_TARGET_PORT = "554"
      }
    }
  }
}

resource "kubernetes_namespace_v1" "apex" {
  metadata {
    name = var.namespace
    labels = {
      app = "apex-kinetic"
    }
  }
}

resource "kubernetes_pod_v1" "workload" {
  for_each = local.workloads

  metadata {
    name      = "apex-kinetic-${each.key}"
    namespace = kubernetes_namespace_v1.apex.metadata[0].name
    labels = merge(local.labels, {
      role = each.key
    })
  }

  spec {
    container {
      name    = each.key
      image   = each.value.image
      command = each.value.command

      dynamic "env" {
        for_each = each.value.env

        content {
          name  = env.key
          value = env.value
        }
      }

      security_context {
        run_as_non_root             = true
        allow_privilege_escalation = false
      }
    }
  }
}

resource "kubernetes_network_policy_v1" "zero_trust" {
  metadata {
    name      = "apex-kinetic-zero-trust"
    namespace = kubernetes_namespace_v1.apex.metadata[0].name
  }

  spec {
    pod_selector {}
    policy_types = ["Ingress", "Egress"]

    ingress {
      from {
        pod_selector {
          match_labels = {
            role = "control-plane"
          }
        }
      }

      port {
        port     = "9092"
        protocol = "TCP"
      }

      port {
        port     = "554"
        protocol = "TCP"
      }
    }

    egress {
      to {
        pod_selector {
          match_labels = {
            role = "kafka"
          }
        }
      }

      port {
        port     = "9092"
        protocol = "TCP"
      }
    }

    egress {
      to {
        pod_selector {
          match_labels = {
            role = "vision-node"
          }
        }
      }

      port {
        port     = "554"
        protocol = "TCP"
      }
    }
  }
}
