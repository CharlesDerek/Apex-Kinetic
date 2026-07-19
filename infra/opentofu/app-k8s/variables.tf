variable "namespace" {
  type        = string
  description = "Kubernetes namespace for Apex Kinetic workloads."
  default     = "apex-kinetic"

  validation {
    condition     = can(regex("^[a-z0-9]([-a-z0-9]*[a-z0-9])?$", var.namespace))
    error_message = "Namespace must be a valid Kubernetes DNS label."
  }
}

variable "image_tag" {
  type        = string
  description = "Container image tag deployed for all Apex Kinetic workloads."
  default     = "latest"

  validation {
    condition     = length(trimspace(var.image_tag)) > 0
    error_message = "Image tag must not be empty."
  }
}

variable "kubernetes_host" {
  type        = string
  description = "Kubernetes API server URL."
  default     = "https://127.0.0.1:6443"
}

variable "kubernetes_token" {
  type        = string
  description = "Bearer token used by the Kubernetes provider."
  sensitive   = true
  default     = "ci-token"
}

variable "kubernetes_ca_certificate" {
  type        = string
  description = "PEM-encoded cluster CA certificate."
  sensitive   = true
  default     = null
}
