# Contributing to DenseZK

Thank you for your interest in contributing to DenseZK!

## Getting Started

### Prerequisites

- **Rust**: 1.70+ (for prover/verifier code)
- **Node.js**: 18+ (for server components)
- **React Native**: 0.74+ (for mobile app)
- **Docker**: For running services

### Development Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/your-repo/denseZK.git
   cd denseZK
   ```

2. Install Rust dependencies:
   ```bash
   cargo build
   ```

3. Try running tests:
   ```bash
   cargo test
   ```

## Project Structure

```
denseZK/
├── src/                 # Core prover/verifier (Rust)
├── densezk_mobile_app/   # React Native mobile app
├── densezk_test_app/    # Test utilities
├── docs/                # API and architecture docs
└── knowledge/          # Research and design docs
```

## Good First Issues

Check the [issues](./issues/) directory for beginner-friendly tasks:
- Parallel proof generation
- Wireless two-device demo
- Benchmark harness
- Poseidon API refactor
- Rel1CS documentation

## Coding Standards

### Rust

- Run `cargo fmt` before committing
- Run `cargo clippy` and address warnings
- Add tests for new functionality

### React Native

- Follow existing component patterns
- Use TypeScript
- Test on iOS and Android

### General

- Add comments for non-obvious code
- Update documentation for API changes
- Keep PRs focused and small

## Submitting Changes

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/your-feature`
3. Make your changes
4. Run tests and formatting
5. Push and open a pull request

## Testing

Run the test suite:
```bash
cargo test
```

Run mobile app tests:
```bash
cd densezk_mobile_app && npm test
```

## Getting Help

- Open an issue for bugs or questions
- Check the docs in `/docs`
