output "namespace" {
  description = "Namespace containing Apex Kinetic workloads."
  value       = kubernetes_namespace_v1.apex.metadata[0].name
}

output "workload_images" {
  description = "Images planned for each Kubernetes workload."
  value = {
    for name, pod in kubernetes_pod_v1.workload : name => pod.spec[0].container[0].image
  }
}

output "workload_security_contexts" {
  description = "Security contexts planned for each Kubernetes workload."
  value = {
    for name, pod in kubernetes_pod_v1.workload : name => {
      run_as_non_root            = pod.spec[0].container[0].security_context[0].run_as_non_root
      allow_privilege_escalation = pod.spec[0].container[0].security_context[0].allow_privilege_escalation
    }
  }
}
