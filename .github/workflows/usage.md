# GitHub Actions Workflows

This repository includes two GitHub Actions workflows:

## 🔄 CI Workflow (`ci.yml`)

Runs on every push and pull request to `main`/`master` branches:

- **Code formatting check**: Ensures code follows Rust formatting standards
- **Clippy linting**: Catches common mistakes and suggests improvements
- **Tests**: Runs the test suite (if any tests exist)
- **Build verification**: Ensures the code compiles successfully

## 🚀 Release Workflow (`release.yml`)

Triggered when you create a git tag starting with `v` (e.g., `v1.0.0`):

### Linux Builds
Automatically builds binaries for:
- **Linux x86_64** (most common desktop/server)
- **Linux ARM64** (ARM servers/devices like Raspberry Pi)

### Creating a Release

1. **Tag your release:**
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

2. **GitHub Actions will automatically:**
   - Build binaries for all platforms
   - Create a GitHub release
   - Upload all binaries as downloadable assets
   - Generate release notes

### Downloaded Files
Users can download Linux binaries:
- `niri-compact-linux-x86_64` - Linux 64-bit (Intel/AMD)
- `niri-compact-linux-aarch64` - Linux ARM64 (ARM processors)

## 📦 Artifacts

Even on non-release builds (like PRs), the build artifacts are temporarily saved and can be downloaded from the Actions page for testing.

## 🛠️ Local Development

To run the same checks locally:
```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --all -- --check

# Lint code (requires clippy)
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test

# Build release
cargo build --release
```