set ignore-comments := true
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# We need to tell ffmpeg-next's build.rs how to find FFmpeg.  Finding using vcpkg-rs is broken for
# reasons outlined in .cargo/config.toml.  pkg-config is a good alternative, and vcpkg builds
# pkgconf for us, which is compatible.
#
# Unfortunately, cargo doesn't have a way of configuring per-profile/platform environment variables.
# So, we can't use cargo to set these.  We'll set them using just.
#
# And, unfortunately, just doesn't have a good way to export environment variables only for
# certain recipes.  So this is ultimately very hacky-looking due to all these limitations.
#
# Non-Windows platforms are not yet supported, so we will fail the recipe in that case.

SETUP_DEBUG_ENV := if os_family() == "windows" { "$env:PKG_CONFIG = \"" + justfile_directory() + "\\vcpkg_installed\\x64-windows-release\\tools\\pkgconf\\pkgconf.exe\"\n" + "$env:PKG_CONFIG_PATH = \"" + justfile_directory() + "\\vcpkg_installed\\x64-windows-static\\debug\\lib\\pkgconfig\"\n" } else { "exit 1" }
SETUP_RELEASE_ENV := if os_family() == "windows" { "$env:PKG_CONFIG = \"" + justfile_directory() + "\\vcpkg_installed\\x64-windows-release\\tools\\pkgconf\\pkgconf.exe\"\n" + "$env:PKG_CONFIG_PATH = \"" + justfile_directory() + "\\vcpkg_installed\\x64-windows-static\\lib\\pkgconfig\"\n" } else { "exit 1" }

default:
    @just --list

# Run all checks and build all packages with debug profile
[group('combined')]
check: base-deps check-fmt check-locked clippy coverage doctest build-all

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
    vcpkg format-manifest --all

# Check if Cargo.lock is up-to-date
[group('lint')]
check-locked:
    cargo verify-project --locked

# Lint all packages
[group('lint')]
clippy:
    {{ SETUP_DEBUG_ENV }} ; cargo clippy --workspace --all-targets --all-features -- --deny warnings

# Test all packages
[group('test')]
test testname="":
    {{ SETUP_DEBUG_ENV }} ; cargo test --workspace {{ testname }}

# Test all packages with coverage; excludes doctests at this time due to stable Rust limitations
[group('test')]
coverage testname="":
    {{ SETUP_DEBUG_ENV }} ; cargo llvm-cov test --workspace {{ testname }}
    {{ SETUP_DEBUG_ENV }} ; cargo llvm-cov report --html
    {{ SETUP_DEBUG_ENV }} ; cargo llvm-cov report --lcov --output-path target/llvm-cov/lcov.info

# Run only doctests
[group('test')]
doctest testname="":
    {{ SETUP_DEBUG_ENV }} ; cargo test --workspace --doc {{ testname }}

# Review new snapshots taken by insta
[group('test')]
insta-review:
    {{ SETUP_DEBUG_ENV }} ; cargo insta review

# Build all packages with debug profile
[group('build')]
build-all:
    {{ SETUP_DEBUG_ENV }} ; cargo build --workspace

# Build default packages with debug profile
[group('build')]
build:
    {{ SETUP_DEBUG_ENV }} ; cargo build

# Build default packages with release profile
[group('build')]
build-release:
    {{ SETUP_RELEASE_ENV }} ; cargo build --release

# Build documentation
[group('build')]
doc:
    {{ SETUP_DEBUG_ENV }} ; cargo doc

# Install base tools and packages that still require extra commands to install.
[group('build')]
base-deps: bin-install vcpkg-install
    # Most recipes don't have a declared dependency on this recipe to save time, but they do
    # require them.

# Install tools using cargo-run-bin
[group('build')]
bin-install:
    cargo bin --install

# Update aliases in .cargo/config.toml so you can run e.g. "cargo llvm-cov".
[group('edit')]
bin-sync-aliases:
    cargo bin --sync-aliases

# Install C/C++ packages via vcpkg
[group('build')]
[windows]
vcpkg-install:
    # This builds both debug and release builds.  It is statically linked with the C/C++ runtime,
    # which aligns with our preference in .cargo/config.toml.
    vcpkg install --triplet=x64-windows-static --host-triplet=x64-windows-release

# Set environment variables for use with Visual Studio Code (also requires tabaqa extension)
[group('ide')]
[windows]
vscode-setup:
    # Using New-Item is required to write UTF-8 without a BOM on Windows PowerShell
    #
    # Setting RUSTC_BOOTSTRAP is necessary to work around poor performance switching to/from the
    # VS Code Test Explorer - see:
    # https://github.com/rust-lang/rust-analyzer/issues/17149#issuecomment-2080396613
    # You may also want to set this from the shell that you run "just" commands from.
    {{ SETUP_DEBUG_ENV }} ; \
    $rust_env = @{ \
        "rust-analyzer.cargo.extraEnv" = @{ \
            "PKG_CONFIG" = $env:PKG_CONFIG ; \
            "PKG_CONFIG_PATH" = $env:PKG_CONFIG_PATH ; \
            "RUSTC_BOOTSTRAP" = "1" \
        } ; \
        "rust-analyzer.runnables.extraEnv" = @{ \
            "PKG_CONFIG" = $env:PKG_CONFIG ; \
            "PKG_CONFIG_PATH" = $env:PKG_CONFIG_PATH ; \
            "RUSTC_BOOTSTRAP" = "1" \
        } \
    } | ConvertTo-Json ; \
    New-Item -Force .vscode\rust-environment.json -Value $rust_env | Out-Null

# Show environment variables that we set
[group('tools')]
[windows]
show-debug-env:
    {{ SETUP_DEBUG_ENV }} ; \
    Write-Host "PKG_CONFIG=$($env:PKG_CONFIG)" ; \
    Write-Host "PKG_CONFIG_PATH=$($env:PKG_CONFIG_PATH)"

# Run the "cargo expand" command to show the results of expanding macros in a library target.
[group('tools')]
expand-lib crate path="":
    {{ SETUP_DEBUG_ENV }} ; cargo expand --package {{ crate }} --lib {{ path }}
