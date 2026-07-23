provider "kubernetes" {
  config_path    = var.kubernetes_config_path
  config_context = var.kubernetes_config_context

  host                   = var.kubernetes_config_path == null ? var.kubernetes_host : null
  token                  = var.kubernetes_config_path == null ? var.kubernetes_token : null
  cluster_ca_certificate = var.kubernetes_config_path == null ? var.kubernetes_ca_certificate : null
}
