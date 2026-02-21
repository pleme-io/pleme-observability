# pleme-observability

Observability library for Pleme platform - tracing, metrics, distributed tracing, and metric definition macros

## Installation

```toml
[dependencies]
pleme-observability = "0.1"
```

## Usage

```rust
use pleme_observability::{init_observability, define_metrics};

// Auto-detects OpenTelemetry vs basic tracing
init_observability("my-service").await?;

define_metrics! {
    REQUESTS_TOTAL: Counter = "http_requests_total";
    REQUEST_DURATION: Histogram = "http_request_duration_seconds";
}
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `tracing-basic` | Basic tracing subscriber setup |
| `distributed-tracing` | OpenTelemetry distributed tracing |
| `metrics` | Prometheus metric definitions via `define_metrics!` |
| `full` | All features enabled |

Enable features in your `Cargo.toml`:

```toml
pleme-observability = { version = "0.1", features = ["full"] }
```

## Development

This project uses [Nix](https://nixos.org/) for reproducible builds:

```bash
nix develop            # Dev shell with Rust toolchain
nix run .#check-all    # cargo fmt + clippy + test
nix run .#publish      # Publish to crates.io (--dry-run supported)
nix run .#regenerate   # Regenerate Cargo.nix
```

## License

MIT - see [LICENSE](LICENSE) for details.
