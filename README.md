# Panda API 2.0

Panda API 2.0 is being rebuilt as an **API Contract Workbench**: one Git-native
contract for documentation, mock responses, request execution, automated tests,
schema validation, industry-format exchange, and coding-agent access.

## Current status

The repository is at **M0 — Architecture Foundation** (`2.0.0-alpha.1`). The new
Rust workspace and its boundaries build and test, but the compiler, server, mock
runtime, test engine, web workbench, OpenAPI adapters, and MCP server are not
implemented yet.

Read the [Panda API 2.0 documentation](docs/README.md) before relying on a planned
capability.

## Development baseline

- Rust `1.98.0`, Edition 2024
- Cargo workspace with committed `Cargo.lock`
- Linux, macOS, and Windows CI
- No production third-party dependencies in M0

```shell
cargo build --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p panda-cli -- --version
```

The `panda` binary only exposes `--version` in M0. Other commands fail explicitly
until their owning milestone is implemented.

## Panda API 0.7

Panda API 0.7.4 remains available as historical and migration evidence:

- [0.7 documentation](docs/legacy/0.7/README.md)
- [frozen implementation](compatibility/panda-0.7/README.md)
- [legacy fixtures](fixtures/panda-0.7/README.md)
- existing Git tag `v0.74`

The archived implementation has known build and security problems and is not part
of the Panda API 2.0 workspace or CI.

## License

[MIT](LICENSE)
