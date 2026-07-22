locals {
  labels = {
    app = "apex-kinetic"
  }

  health_probe_command = ["sh", "-c", "test -r /proc/1/stat"]

  deployments = {
    control-plane = {
      image   = "apex-kinetic/control-plane:${var.image_tag}"
      command = ["python", "/app/main.py"]
      env = {
        KAFKA_BOOTSTRAP_SERVERS = "kafka:9092"
      }
      requests = {
        cpu    = "50m"
        memory = "64Mi"
      }
      limits = {
        cpu    = "250m"
        memory = "256Mi"
      }
    }
    vision-node = {
      image   = "apex-kinetic/vision-node:${var.image_tag}"
      command = ["/usr/local/bin/vision-node"]
      env = {
        RTSP_SOURCE_URL = "rtsp://edge-camera.local/stream"
        NVR_TARGET_HOST = "annke-nvr.local"
        NVR_TARGET_PORT = "554"
      }
      requests = {
        cpu    = "100m"
        memory = "128Mi"
      }
      limits = {
        cpu    = "500m"
        memory = "512Mi"
      }
    }
  }

  daemon_sets = {
    data-plane = {
      image   = "apex-kinetic/data-plane:${var.image_tag}"
      command = ["/usr/local/bin/data-plane"]
      env     = {}
      requests = {
        cpu    = "100m"
        memory = "64Mi"
      }
      limits = {
        cpu    = "500m"
        memory = "256Mi"
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

resource "kubernetes_deployment_v1" "workload" {
  for_each = local.deployments

  metadata {
    name      = "apex-kinetic-${each.key}"
    namespace = kubernetes_namespace_v1.apex.metadata[0].name
    labels = merge(local.labels, {
      role = each.key
    })
  }

  spec {
    replicas = 1

    selector {
      match_labels = merge(local.labels, {
        role = each.key
      })
    }

    template {
      metadata {
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

          resources {
            requests = each.value.requests
            limits   = each.value.limits
          }

          readiness_probe {
            exec {
              command = local.health_probe_command
            }

            initial_delay_seconds = 5
            period_seconds        = 10
          }

          liveness_probe {
            exec {
              command = local.health_probe_command
            }

            initial_delay_seconds = 15
            period_seconds        = 20
          }

          security_context {
            run_as_non_root            = true
            allow_privilege_escalation = false
          }
        }
      }
    }
  }
}

resource "kubernetes_daemon_set_v1" "workload" {
  for_each = local.daemon_sets

  metadata {
    name      = "apex-kinetic-${each.key}"
    namespace = kubernetes_namespace_v1.apex.metadata[0].name
    labels = merge(local.labels, {
      role = each.key
    })
  }

  spec {
    selector {
      match_labels = merge(local.labels, {
        role = each.key
      })
    }

    template {
      metadata {
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

          resources {
            requests = each.value.requests
            limits   = each.value.limits
          }

          readiness_probe {
            exec {
              command = local.health_probe_command
            }

            initial_delay_seconds = 5
            period_seconds        = 10
          }

          liveness_probe {
            exec {
              command = local.health_probe_command
            }

            initial_delay_seconds = 15
            period_seconds        = 20
          }

          security_context {
            run_as_non_root            = true
            allow_privilege_escalation = false
          }
        }
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

      ports {
        port     = "9092"
        protocol = "TCP"
      }

      ports {
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

      ports {
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

      ports {
        port     = "554"
        protocol = "TCP"
      }
    }
  }
}
