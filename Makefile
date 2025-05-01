# Docker image name
IMAGE_NAME := rpc-gateway
DOCKER_REGISTRY := whatsgood
FULL_IMAGE_NAME := $(DOCKER_REGISTRY)/$(IMAGE_NAME)
DOCKER_IMAGE_VERSION := $(shell git rev-parse --short HEAD)

# Ensure shell commands exit on error
.SHELLFLAGS := -e

##@ build
.PHONY: build	
build: ## Build the Rust binary.
	@echo "Building Rust binary..."
	cargo build --release

.PHONY: docker-build
docker-build: ## Build the Docker image.
	@echo "Building Docker image..."
	docker build -t $(IMAGE_NAME) .

.PHONY: docker-clean
docker-clean: ## Clean up Docker resources.
	@echo "Cleaning up Docker resources..."
	docker rmi $(IMAGE_NAME) || true

.PHONY: docker-publish
docker-publish: ## Publish the Docker image to the Docker Hub repository.
	@echo "Setting up Docker Buildx for multi-platform builds..."
	@if ! docker buildx inspect multiplatform >/dev/null 2>&1; then \
		docker buildx create --name multiplatform --driver docker --use; \
	fi
	@echo "Building and pushing multi-platform images..."
	docker buildx build \
		--platform linux/amd64,linux/arm64 \
		--tag $(FULL_IMAGE_NAME):$(DOCKER_IMAGE_VERSION) \
		--tag $(FULL_IMAGE_NAME):latest \
		--push \
		.
	@echo "Successfully published $(FULL_IMAGE_NAME):$(DOCKER_IMAGE_VERSION) for multiple platforms"
	@echo "Successfully published $(FULL_IMAGE_NAME):latest for multiple platforms"


.PHONY: helm-build
helm-build: ## Build the Helm chart.
	@echo "Building Helm chart..."
	cd ./helm && \
		helm package rpc-gateway && \
		helm repo index . --url https://whats-good.github.io/rpc-gateway/helm

.PHONY: helm-publish
helm-publish: ## Publish the Helm chart to the GitHub Pages repository.
	@echo "Publishing Helm chart..."
	@if [ -n "$$(git status --porcelain)" ]; then \
		echo "Error: There are uncommitted changes. Please commit or stash them first."; \
		exit 1; \
	fi
	@CURRENT_BRANCH=$$(git rev-parse --abbrev-ref HEAD) && \
		if [ "$$CURRENT_BRANCH" != "main" ]; then \
			echo "Error: Not on main branch. Please checkout main branch first."; \
			exit 1; \
		fi
	@HELM_VERSION=$$(yq '.version' ./helm/rpc-gateway/Chart.yaml) && \
		echo "Creating git tag helm-v$$HELM_VERSION" && \
		git tag -a "helm-v$$HELM_VERSION" -m "Helm chart version $$HELM_VERSION" && \
		git push origin "helm-v$$HELM_VERSION"

##@ docker compose

.PHONY: docker-up
docker-up: ## Start all Docker services.
	@echo "Starting all Docker services..."
	docker-compose up -d --build

.PHONY: docker-up-no-gateway
docker-up-no-gateway: ## Start all Docker services except the gateway.
	@echo "Starting supporting services..."
	docker-compose up -d mainnet polygon arbitrum prometheus redis redis-commander 
	@echo "Waiting for services to be ready..."

.PHONY: docker-down
docker-down: ## Stop all Docker services.
	@echo "Stopping all Docker services..."
	docker-compose down

##@ minikube

.PHONY: minikube-deploy
minikube-deploy: ## Deploy the Helm chart to Minikube.
	@if ! minikube status > /dev/null 2>&1; then \
		echo "Error: Minikube is not running. Please start minikube first."; \
		exit 1; \
	fi
	@if [ "$$(kubectl config current-context)" != "minikube" ]; then \
		echo "Error: kubectl is not connected to minikube. Please run 'kubectl config use-context minikube'"; \
		exit 1; \
	fi
	@if [ -z "$$ALCHEMY_URL" ]; then \
		echo "Error: ALCHEMY_URL environment variable is not set"; \
		exit 1; \
	fi
	@echo "Remove existing dependencies..."
	rm -rf ./helm/minikube-example/charts || true
	rm ./helm/minikube-example/Chart.lock || true

	@echo "Building Helm dependencies..."
	cd ./helm/minikube-example && helm dependency build
	@echo "Deploying Helm chart to Minikube..."
	helm upgrade --install minikube-example ./helm/minikube-example \
		--namespace default \
		--create-namespace \
		--set image.pullPolicy=Always \
		--set secrets.ALCHEMY_URL="$$ALCHEMY_URL"
	@$(MAKE) minikube-port-forward

.PHONY: minikube-port-forward
minikube-port-forward: ## Start port-forwarding for the Minikube gateway.
	@echo "Starting port-forward..."
	@if [ -f .port-forward.pid ]; then \
		kill $$(cat .port-forward.pid) 2>/dev/null || true; \
		rm .port-forward.pid; \
	fi
	@kubectl port-forward svc/minikube-example-rpc-gateway 8080:8080 > /dev/null 2>&1 & \
		echo $$! > .port-forward.pid
	@echo "Waiting for pod to be ready..."
	@kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=rpc-gateway --timeout=60s

.PHONY: minikube-delete
minikube-delete: ## Delete the Helm chart from Minikube.
	@echo "Deleting Helm chart from Minikube..."
	@if [ -f .port-forward.pid ]; then \
		kill $$(cat .port-forward.pid) 2>/dev/null || true; \
		rm .port-forward.pid; \
	fi
	helm uninstall minikube-example --namespace default

.PHONY: minikube-test
minikube-test: ## Test the Minikube gateway by sending an eth_getBlock request.
	@echo "Testing Minikube gateway..."
	@curl -X POST http://localhost:8080/1 \
		-H "Content-Type: application/json" \
		-d '{"jsonrpc":"2.0","method":"eth_getBlockByNumber","params":["latest",false],"id":1}'

##@ development

.PHONY: dev
dev: ## Start development server with file watching. Usage: make dev CONFIG=path/to/config.yml
	@echo "Starting development server with file watching..."
	@echo "Press Ctrl+C to stop"
	@echo "----------------------------------------"
	@echo "Server will be available at: http://localhost:8080"
	@echo "----------------------------------------"
	@echo "Logs will be written to: logs/rpc-gateway.log"
	@echo "----------------------------------------"
	@echo "Starting server..."
	@mkdir -p logs
	watchexec -e rs -r cargo run --bin rpc-gateway -- -c $(if $(CONFIG),$(CONFIG),$(PWD)/example.config.yml)

.PHONY: udeps
udeps: ## Find unused dependencies.
	@echo "Running udeps..."
	cargo +nightly udeps --all-targets --all-features --workspace

loadtest:
	@echo "Running load test..."
	@mkdir -p loadtest-reports
	cargo run --bin loadtest -- --host http://localhost:8080 --report-file loadtest-reports/report.html --run-time 15s --hatch-rate 10 --users 20

.PHONY: test
test: ## Run all tests.
	@echo "Running tests..."
	cargo test --workspace

.PHONY: coverage
coverage: ## Generate test coverage report.
	@echo "Generating test coverage report..."
	cargo tarpaulin --workspace --out Html --output-dir ./target/coverage
	@echo "Coverage report generated at ./target/coverage/tarpaulin-report.html"

.PHONY: check
check: ## Run all checks.
	@echo "Running checks..."
	cargo check --workspace

.PHONY: lint
lint: ## Run all linting checks.
	@echo "Running linting checks..."
	cargo clippy --workspace

##@ help
# Show help
.PHONY: help
help: ## Display this help.
	@awk 'BEGIN {FS = ":.*##"; printf "Usage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_0-9-]+:.*?##/ { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)