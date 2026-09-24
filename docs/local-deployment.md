# Local Deployment

This workflow creates a local `kind` cluster, builds the three Apex Kinetic images, loads them into the cluster, starts a Kafka-compatible Redpanda broker, and applies `infra/opentofu/app-k8s`.

## Prerequisites

- Docker
- `kind`
- `kubectl`
- OpenTofu as `tofu`, or Docker for the fallback OpenTofu container runner

## Start

```bash
make local-up
```

The default cluster is `apex-kinetic`, the default namespace is `apex-kinetic`, and the default image tag is `local`.

Override defaults with environment variables:

```bash
KIND_CLUSTER_NAME=apex-dev APEX_NAMESPACE=apex-dev IMAGE_TAG=dev make local-up
```

## Inspect

```bash
make local-status
kubectl -n apex-kinetic get pods
kubectl -n apex-kinetic logs deploy/apex-kinetic-control-plane
kubectl -n apex-kinetic logs deploy/apex-kinetic-vision-node
```

The vision-node readiness probe now checks its own recent mTLS health snapshot.
The sample deployment does not provision camera or NVR certificates; supply
trusted client, key, and CA material to the container before expecting the
vision deployment to become ready. The current worker emits synthetic payloads
and does not yet validate camera capture.

## Stop

```bash
make local-down
```

## Deployment Source

The Apex Kinetic Kubernetes resources are applied through OpenTofu only. The local script creates the supporting `kind` cluster and local broker dependency, then runs the same OpenTofu module used by CI.
