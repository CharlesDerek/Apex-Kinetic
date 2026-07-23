#!/usr/bin/env bash
set -euo pipefail

ACTION="${1:-up}"
CLUSTER_NAME="${KIND_CLUSTER_NAME:-apex-kinetic}"
NAMESPACE="${APEX_NAMESPACE:-apex-kinetic}"
IMAGE_TAG="${IMAGE_TAG:-local}"
KUBE_CONTEXT="kind-${CLUSTER_NAME}"
REDPANDA_IMAGE="${REDPANDA_IMAGE:-docker.redpanda.com/redpandadata/redpanda:v24.3.1}"
TOFU_IMAGE="${TOFU_IMAGE:-ghcr.io/opentofu/opentofu:1.10.7}"

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TOFU_DIR="${ROOT_DIR}/infra/opentofu/app-k8s"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1" >&2
    exit 1
  fi
}

run_tofu() {
  if command -v tofu >/dev/null 2>&1; then
    (cd "${TOFU_DIR}" && tofu "$@")
    return
  fi

  docker run --rm --network host \
    -v "${ROOT_DIR}:/workspace" \
    -v "${HOME}/.kube:/root/.kube:ro" \
    -w /workspace/infra/opentofu/app-k8s \
    "${TOFU_IMAGE}" "$@"
}

build_and_load_images() {
  for service in control-plane data-plane vision-node; do
    docker build -t "apex-kinetic/${service}:${IMAGE_TAG}" "${ROOT_DIR}/${service}"
    kind load docker-image "apex-kinetic/${service}:${IMAGE_TAG}" --name "${CLUSTER_NAME}"
  done
}

ensure_cluster() {
  if kind get clusters | grep -Fxq "${CLUSTER_NAME}"; then
    return
  fi

  kind create cluster --name "${CLUSTER_NAME}"
}

ensure_local_broker() {
  kubectl create namespace "${NAMESPACE}" --dry-run=client -o yaml | kubectl apply -f -

  if ! kubectl -n "${NAMESPACE}" get deployment kafka >/dev/null 2>&1; then
    kubectl -n "${NAMESPACE}" create deployment kafka --image="${REDPANDA_IMAGE}" -- \
      redpanda start \
      --overprovisioned \
      --smp 1 \
      --memory 256M \
      --reserve-memory 0M \
      --node-id 0 \
      --check=false \
      --kafka-addr PLAINTEXT://0.0.0.0:9092 \
      --advertise-kafka-addr PLAINTEXT://kafka:9092
  fi

  kubectl -n "${NAMESPACE}" expose deployment kafka --port 9092 --target-port 9092 --dry-run=client -o yaml | kubectl apply -f -
  kubectl -n "${NAMESPACE}" rollout status deployment/kafka --timeout=180s
}

apply_apex_workloads() {
  run_tofu init -backend=false
  run_tofu apply -auto-approve \
    -var "namespace=${NAMESPACE}" \
    -var "image_tag=${IMAGE_TAG}" \
    -var "kubernetes_config_path=${HOME}/.kube/config" \
    -var "kubernetes_config_context=${KUBE_CONTEXT}"
}

status() {
  kubectl config use-context "${KUBE_CONTEXT}" >/dev/null
  kubectl -n "${NAMESPACE}" get pods
}

up() {
  require_command docker
  require_command kind
  require_command kubectl

  ensure_cluster
  kubectl config use-context "${KUBE_CONTEXT}" >/dev/null
  build_and_load_images
  ensure_local_broker
  apply_apex_workloads
  status
}

down() {
  require_command kind
  kind delete cluster --name "${CLUSTER_NAME}"
}

case "${ACTION}" in
  up)
    up
    ;;
  down)
    down
    ;;
  status)
    require_command kubectl
    status
    ;;
  *)
    echo "Usage: $0 [up|down|status]" >&2
    exit 1
    ;;
esac
