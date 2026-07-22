output "namespace" {
  description = "Namespace containing Apex Kinetic workloads."
  value       = kubernetes_namespace_v1.apex.metadata[0].name
}

output "workload_images" {
  description = "Images planned for each Kubernetes workload."
  value = merge(
    {
      for name, deployment in kubernetes_deployment_v1.workload :
      name => deployment.spec[0].template[0].spec[0].container[0].image
    },
    {
      for name, daemon_set in kubernetes_daemon_set_v1.workload :
      name => daemon_set.spec[0].template[0].spec[0].container[0].image
    }
  )
}

output "workload_kinds" {
  description = "Kubernetes controller kind planned for each workload."
  value = merge(
    {
      for name in keys(kubernetes_deployment_v1.workload) :
      name => "Deployment"
    },
    {
      for name in keys(kubernetes_daemon_set_v1.workload) :
      name => "DaemonSet"
    }
  )
}

output "workload_security_contexts" {
  description = "Security contexts planned for each Kubernetes workload."
  value = merge(
    {
      for name, deployment in kubernetes_deployment_v1.workload : name => {
        run_as_non_root            = deployment.spec[0].template[0].spec[0].container[0].security_context[0].run_as_non_root
        allow_privilege_escalation = deployment.spec[0].template[0].spec[0].container[0].security_context[0].allow_privilege_escalation
      }
    },
    {
      for name, daemon_set in kubernetes_daemon_set_v1.workload : name => {
        run_as_non_root            = daemon_set.spec[0].template[0].spec[0].container[0].security_context[0].run_as_non_root
        allow_privilege_escalation = daemon_set.spec[0].template[0].spec[0].container[0].security_context[0].allow_privilege_escalation
      }
    }
  )
}

output "workload_resource_limits" {
  description = "CPU and memory limits planned for each workload."
  value = merge(
    {
      for name, deployment in kubernetes_deployment_v1.workload :
      name => deployment.spec[0].template[0].spec[0].container[0].resources[0].limits
    },
    {
      for name, daemon_set in kubernetes_daemon_set_v1.workload :
      name => daemon_set.spec[0].template[0].spec[0].container[0].resources[0].limits
    }
  )
}
