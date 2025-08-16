# Contributing to Quantum Config

Thank you for your interest in contributing! This document describes how to set up your environment, coding standards, testing and release processes.

## Table of Contents
- Getting Started
- Development Workflow
- Coding Standards
- Testing
- Commit Messages
- Pull Requests
- Documentation
- Release Process
- Issue Reporting

## Getting Started

Prerequisites:
- Rust toolchain (stable). Install via rustup: https://rustup.rs
- Git

Project layout:
- quantum_config/ — main library crate
- quantum_config_derive/ — procedural macro crate
- examples/ — usage examples

Bootstrap:
```bash
# clone
git clone https://github.com/Kirky-X/quantum_config.git
cd quantum_config

# run tests and lints
cargo test
cargo fmt -- --check
cargo clippy -- -D warnings
```

## Development Workflow

1. Create an issue (or comment on an existing one) to discuss your proposal.
2. Fork + create a feature branch: feature/short-topic
3. Implement changes with small, focused commits.
4. Keep the public API stable unless agreed. Add docs and tests for any API change.
5. Ensure CI passes locally: fmt, clippy, tests.
6. Open a Pull Request (PR) and link to the related issue.

## Coding Standards

- Formatting: `cargo fmt` (rustfmt).
- Lints: `cargo clippy` with `-D warnings` clean.
- Error handling:
  - Prefer `?` over `unwrap`/`expect` in library code.
  - Provide meaningful error context.
- Public APIs must have rustdoc comments with examples where appropriate.
- Follow SOLID principles and keep functions small and cohesive.
- Avoid breaking changes unless they are explicitly discussed.

## Testing

- Add unit tests for new functionality (happy path and edge cases).
- Add integration tests when behavior spans multiple components.
- Run all tests before submitting PR:
```bash
cargo test --all
```

## Commit Messages

Follow Conventional Commits where possible:
- feat: a new feature
- fix: a bug fix
- docs: documentation only changes
- refactor: neither fixes a bug nor adds a feature
- test: adding or correcting tests
- ci: CI/CD related changes
- chore: maintenance tasks (format, tidy, etc.)

Examples:
- feat(config): support async loading
- fix(template): correct default value rendering
- docs(readme): add quick start section

## Pull Requests

- Fill in the PR description: motivation, summary of changes, and any breaking changes.
- Include tests and documentation updates.
- Ensure CI is green: fmt, clippy, tests.
- Be responsive to review feedback; keep PRs focused and reasonably small.

## Documentation

- Update `README.md` and `README_EN.md` when user-facing behavior changes.
- Keep examples under `examples/` in sync with the latest APIs.
- Public items should include rustdoc comments and examples if practical.

## Release Process

Crates: `quantum_config_derive` (proc-macro) and `quantum_config` (main).

1. Verify workspace is clean; update versions if needed.
2. Update CHANGELOG.md and READMEs if applicable.
3. Dry run packaging for both crates:
```bash
# in quantum_config_derive/
cargo publish --dry-run -v

# in repo root (quantum_config)
cargo publish --dry-run -v
```
4. Publish order (important):
```bash
# 1) publish derive crate first
cd quantum_config_derive && cargo publish -v

# 2) then publish main crate
cd .. && cargo publish -v
```
5. Tag and release:
```bash
git tag vX.Y.Z
git push origin vX.Y.Z
```

## Issue Reporting

When filing an issue, please include:
- Your Rust version (`rustc -V`), OS, and crate versions
- Steps to reproduce (minimal example preferred)
- Expected vs actual behavior
- Relevant logs or error messages

Thanks again for contributing to Quantum Config!