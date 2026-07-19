run "workloads_plan_with_expected_images_and_security_context" {
  command = plan

  variables {
    namespace = "apex-kinetic-ci"
    image_tag = "ci"
  }

  assert {
    condition     = output.namespace == "apex-kinetic-ci"
    error_message = "The module should honor the supplied namespace."
  }

  assert {
    condition = output.workload_images == {
      control-plane = "apex-kinetic/control-plane:ci"
      data-plane    = "apex-kinetic/data-plane:ci"
      vision-node   = "apex-kinetic/vision-node:ci"
    }
    error_message = "All workloads should use the configured image tag."
  }

  assert {
    condition = alltrue([
      for context in values(output.workload_security_contexts) :
      context.run_as_non_root && !context.allow_privilege_escalation
    ])
    error_message = "All pods must run as non-root without privilege escalation."
  }
}

run "namespace_validation_rejects_invalid_dns_label" {
  command = plan

  variables {
    namespace = "Invalid_Namespace"
  }

  expect_failures = [
    var.namespace,
  ]
}
