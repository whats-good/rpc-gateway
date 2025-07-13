# RPC Gateway

A high-performance, configurable RPC gateway for Ethereum networks. This service provides load balancing, caching, and health monitoring for RPC endpoints.

## Features

- **Load Balancing**: Distribute requests across multiple RPC endpoints
- **Caching**: Cache responses to reduce load on upstream providers
- **Health Monitoring**: Automatic health checks for upstream providers
- **Canned Responses**: Predefined responses for specific RPC methods
- **Configurable**: Flexible configuration through YAML
- **Docker Support**: Ready-to-use Docker images
- **Kubernetes Support**: Helm charts for easy deployment
- **Logging**: Configurable logging with multiple backends

## Quick Start

### Prerequisites

- Rust 1.85 or later
- Docker (optional)
- Kubernetes (optional)

### Installation

1. Clone the repository:

```bash
git clone https://github.com/whats-good/rpc-gateway.git
cd rpc-gateway
```

2. Build the project:

```bash
make build
```

### Configuration

Create a configuration file (e.g., `config.yml`) based on the example:

```yaml
server:
  host: "127.0.0.1"
  port: 8080

load_balancing:
  strategy: "primary_only"

upstream_health_checks:
  enabled: true
  interval: "5m"

cache:
  enabled: true

chains:
  1:
    upstreams:
      - url: "$ALCHEMY_URL"
        timeout: "10s"
        weight: 1
```

### Running the Service

1. Set your environment variables:

```bash
export ALCHEMY_URL="your-alchemy-url"
```

2. Start the service in development mode:

```bash
cargo run -- -c config.yml
```

- Or, if you want to use our pre-defined example config:

```bash
make dev
```

### Docker

Build and run using Docker:

```bash
docker compose up
```

Or pull the latest image from Docker Hub:

```bash
docker pull whats-good/rpc-gateway:latest
```

### Kubernetes

Deploy using Helm:

```bash
helm install rpc-gateway oci://ghcr.io/whats-good/rpc-gateway/helm
```

## API Usage

The service exposes a JSON-RPC endpoint that's compatible with Ethereum clients. The endpoint is `/1` for the mainnet, `/11155111` for sepolia, and so on. These chains must be configured in the `config.yml` file, under the `chains` section.

### Example CURL Request

```bash
curl -X POST http://localhost:8080/1 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "eth_blockNumber",
    "params": [],
    "id": 1
  }'
```

### Example Response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": "0x1234"
}
```

## Configuration Options

### Server Configuration

- `host`: Server host address
- `port`: Server port number

### Load Balancing

- `strategy`: Load balancing strategy ("primary_only", "round_robin", "weighted")

### Upstream Health Checks

- `enabled`: Enable/disable health checks
- `interval`: Health check interval

### Cache

- `enabled`: Enable/disable response caching

### Chains

Configure multiple chains with their respective upstream providers:

- `url`: RPC endpoint URL
- `timeout`: Request timeout
- `weight`: Load balancing weight

## Logging

Configure logging through the configuration file:

- Console logging
- File logging
- Log rotation
- Log levels and formats

## Development

### Building

```bash
make build
```

### Testing

```bash
make test
```

### Linting

```bash
make lint
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Honorable Mentions

This project wouldn't be possible without the amazing work of the following projects:

- [Alloy](https://github.com/alloy-rs/alloy)
- [Foundry](https://github.com/foundry-rs/foundry)

## License

This project is licensed under the MIT License - see the LICENSE file for details.
