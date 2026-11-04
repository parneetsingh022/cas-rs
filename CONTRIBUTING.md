# Contributing to hashb

Thank you for your interest in contributing to **hashb**! Whether you are reporting a bug, proposing a new feature, or submitting a pull request, your help is appreciated.

## Development Setup

To build and test the project locally, you will need the standard Rust toolchain and the `pre-commit` framework.

1. **Install Rust:** Ensure you have `rustup` and the latest stable Rust compiler installed.
2. **Install Pre-commit:** We use `pre-commit` to enforce formatting and linting.
   ```bash
   # Install the framework (macOS / Linux)
   brew install pre-commit
   # OR via pip
   pip install pre-commit
   ```
3. **Install Git Hooks:** Run the following commands in the root of the repository to configure the local hooks.
   ```bash
   pre-commit install
   pre-commit install -t pre-push
   ```

## Local Checks and Testing

Our pre-commit configuration ensures code quality before it hits the repository:
* **Pre-commit:** Automatically runs `cargo fmt`, `cargo clippy`, and `cargo check` across the workspace when you commit.
* **Pre-push:** Automatically runs `cargo test --workspace` before pushing to the remote repository.

You can manually trigger these checks at any time by running:
```bash
pre-commit run --all-files
```

## Pull Request Guidelines

1. **Branching:** Create a feature branch from `main` (e.g., `feat/add-new-command` or `fix/issue-22`).
2. **Commit Messages:** Write clear, concise commit messages outlining what changed and why.
3. **Testing:** Ensure any new features or bug fixes include accompanying tests. All tests must pass before a PR can be merged.
4. **Code Review:** Once submitted, a maintainer will review your code. Be prepared to address feedback or iterate on your design.

## AI Usage Policy

We welcome the responsible use of AI coding assistants, but we strictly prohibit autonomous agents and AI-generated communication (such as PR descriptions or review comments).

Before contributing, please read our full [AI Usage Policy](AI_POLICY.md) to understand our expectations regarding human accountability, code quality, and communication.
