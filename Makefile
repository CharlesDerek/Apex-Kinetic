.PHONY: local-up local-down local-status test-contracts test-opentofu

local-up:
	./scripts/local-kind-deploy.sh up

local-down:
	./scripts/local-kind-deploy.sh down

local-status:
	./scripts/local-kind-deploy.sh status

test-contracts:
	python -m unittest discover -s tests -p 'test_*.py'

test-opentofu:
	docker run --rm -v "$$PWD":/workspace -w /workspace/infra/opentofu/app-k8s ghcr.io/opentofu/opentofu:1.10.7 fmt -check -recursive
	docker run --rm -v "$$PWD":/workspace -w /workspace/infra/opentofu/app-k8s ghcr.io/opentofu/opentofu:1.10.7 init -backend=false
	docker run --rm -v "$$PWD":/workspace -w /workspace/infra/opentofu/app-k8s ghcr.io/opentofu/opentofu:1.10.7 validate
	docker run --rm -v "$$PWD":/workspace -w /workspace/infra/opentofu/app-k8s ghcr.io/opentofu/opentofu:1.10.7 test
