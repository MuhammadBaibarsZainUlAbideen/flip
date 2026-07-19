# Contributing to flip

Thanks for your interest in contributing! Here's how to get started.

## Development Setup

```bash
git clone https://github.com/flip-cli/flip.git
cd flip
cargo build
cargo test
```

## Project Structure

```
crates/
  flip-ir/          # Document IR (Intermediate Representation)
  flip-parse/       # Input parsers (20+ formats -> IR)
  flip-render/      # Output renderers (IR -> 20+ formats)
  flip-cli/         # CLI interface
```

## Adding a New Format

1. Add parser in `crates/flip-parse/src/`
2. Add renderer in `crates/flip-render/src/`
3. Register in `crates/flip-parse/src/lib.rs` and `crates/flip-render/src/lib.rs`
4. Add format variants in `crates/flip-ir/src/`
5. Add tests

## Code Style

- Run `cargo fmt --all` before committing
- Run `cargo clippy --all` and fix warnings
- Add tests for new functionality

## Pull Requests

1. Fork the repo
2. Create a feature branch (`git checkout -b my-feature`)
3. Make your changes
4. Run `cargo test` and `cargo clippy`
5. Push and open a PR

## Issues

Found a bug? Open an issue with:
- Input format and file (or a minimal example)
- Expected behavior
- Actual behavior
- `flip --version` output
