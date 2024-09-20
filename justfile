set ignore-comments := true
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

default:
    @just --list

# Run all checks and build all packages with debug profile
[group('combined')]
check: check-fmt check-locked clippy test build-all

# Check that all files are formatted correctly
[group('lint')]
check-fmt:
    # Set unstable config here instead of .rustfmt.toml so they are usable on stable rustfmt:
    # https://github.com/rust-lang/rustfmt/issues/3387#issuecomment-1867606088
    cargo fmt --all --check -- --config unstable_features=true \
        --config group_imports=StdExternalCrate --config imports_granularity=Crate
    just --fmt --check --unstable

# Reformat all files
[group('edit')]
fmt:
    # Set unstable config here instead of .rustfmt.toml so they are usable on stable rustfmt:
    # https://github.com/rust-lang/rustfmt/issues/3387#issuecomment-1867606088
    cargo fmt --all -- --config unstable_features=true \
        --config group_imports=StdExternalCrate --config imports_granularity=Crate
    just --fmt --unstable

# Check if Cargo.lock is up-to-date
[group('lint')]
check-locked:
    cargo verify-project --locked

# Lint all packages
[group('lint')]
clippy:
    cargo clippy --workspace --all-targets --all-features -- --deny warnings

# Test all packages
[group('test')]
test:
    cargo test --workspace

# Build all packages with debug profile
[group('build')]
build-all:
    cargo build --workspace

# Build default packages with debug profile
[group('build')]
build:
    cargo build

# Build default packages with release profile
[group('build')]
build-release:
    cargo build --release

# Build documentation
[group('build')]
doc:
    cargo doc
