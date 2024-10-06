set ignore-comments := true
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# Configure vcpkg to build using appropriate triplets for the current platform

# Statically link as appropriate with the C/C++ runtime, which aligns with our preferences
# in .cargo/config.toml.
export VCPKG_DEFAULT_TRIPLET := if arch() + "-" + os() == "x86_64-windows" {
    "x64-windows-static"
} else if arch() + "-" + os() == "x86_64-linux" {
    "x64-linux-clang"
} else {
    "unsupported-triplet"
}

export VCPKG_DEFAULT_HOST_TRIPLET := if arch() + "-" + os() == "x86_64-windows" {
    "x64-windows-release"
} else if arch() + "-" + os() == "x86_64-linux" {
    "x64-linux-clang-release"
} else {
    "unsupported-triplet"
}

# Configure the Rust target to match the vcpkg triplet
# Note that CRT linkage is set in .cargo/config.toml
export CARGO_BUILD_TARGET := if arch() + "-" + os() == "x86_64-windows" {
    "x86_64-pc-windows-msvc"
} else if arch() + "-" + os() == "x86_64-linux" {
    "x86_64-unknown-linux-gnu"
} else {
    "unsupported-target"
}

EXE_EXTENSION := if os_family() == "windows" { ".exe" } else { "" }
PATH_SEP := if os_family() == "windows" { "\\" } else { "/" }

# We need to tell ffmpeg-next's build.rs how to find FFmpeg.  Finding using vcpkg-rs is broken for
# reasons outlined in .cargo/config.toml.  pkg-config is a good alternative, and vcpkg builds
# pkgconf for us, which is compatible.
#
# Unfortunately, cargo doesn't have a way of configuring per-profile/platform environment variables.
# So, we can't use cargo to set these.  We'll set them using just.
#
# And, unfortunately, just doesn't have a good way to export environment variables only for
# certain recipes.  So this is ultimately very hacky-looking due to all these limitations.

export PKG_CONFIG := (
    justfile_directory() + PATH_SEP + "vcpkg_installed" + PATH_SEP +
    VCPKG_DEFAULT_HOST_TRIPLET + PATH_SEP + "tools" + PATH_SEP + "pkgconf" + PATH_SEP +
    "pkgconf" + EXE_EXTENSION
)

FFMPEG_PKG_CONFIG_PATH_DEBUG := (
    justfile_directory() + PATH_SEP + "vcpkg_installed" + PATH_SEP +
    VCPKG_DEFAULT_TRIPLET + PATH_SEP + "debug" + PATH_SEP + "lib" + PATH_SEP + "pkgconfig"
)
FFMPEG_PKG_CONFIG_PATH_RELEASE := (
    justfile_directory() + PATH_SEP + "vcpkg_installed" + PATH_SEP +
    VCPKG_DEFAULT_TRIPLET + PATH_SEP + "lib" + PATH_SEP + "pkgconfig"
)

SETUP_DEBUG_ENV := if os_family() == "windows" {
    "$env:FFMPEG_PKG_CONFIG_PATH = \"" + FFMPEG_PKG_CONFIG_PATH_DEBUG + "\"\n"
} else {
    "export FFMPEG_PKG_CONFIG_PATH='" + FFMPEG_PKG_CONFIG_PATH_DEBUG + "'\n"
}

SETUP_RELEASE_ENV := if os_family() == "windows" {
    "$env:FFMPEG_PKG_CONFIG_PATH = \"" + FFMPEG_PKG_CONFIG_PATH_RELEASE + "\"\n"
} else {
    "export FFMPEG_PKG_CONFIG_PATH='" + FFMPEG_PKG_CONFIG_PATH_RELEASE + "'\n"
}

# Set up some other variables for Rust

export RUSTC_BACKTRACE := "1"

# Reduce the number of full rebuilds that happen when switching between builds/tests invoked from
# Visual Studio Code and commands invoked from the terminal via the justfile or other commands.
# See: https://github.com/rust-lang/rust-analyzer/issues/17149#issuecomment-2080396613
export RUSTC_BOOTSTRAP := "1"

default:
    @just --list

# Run all checks and build all packages with debug profile
[group('combined')]
verify: base-deps check-fmt check-locked clippy coverage doctest build-all

# Check that all files are formatted correctly
[group('lint')]
check-fmt:
    # Set unstable config here instead of .rustfmt.toml so they are usable on stable rustfmt:
    # https://github.com/rust-lang/rustfmt/issues/3387#issuecomment-1867606088
    cargo fmt --all --check -- --config unstable_features=true \
        --config group_imports=StdExternalCrate --config imports_granularity=Crate
    #just --fmt --check --unstable

# Reformat all files
[group('edit')]
fmt:
    # Set unstable config here instead of .rustfmt.toml so they are usable on stable rustfmt:
    # https://github.com/rust-lang/rustfmt/issues/3387#issuecomment-1867606088
    cargo fmt --all -- --config unstable_features=true \
        --config group_imports=StdExternalCrate --config imports_granularity=Crate
    # For now, this is abysmally bad and makes variable assignments too long:
    # https://github.com/casey/just/issues/2387
    #just --fmt --unstable
    vcpkg format-manifest --all

# Check if Cargo.lock is up-to-date
[group('lint')]
check-locked:
    cargo verify-project --locked

# Check all packages
[group('lint')]
check:
    {{ SETUP_DEBUG_ENV }} cargo check --workspace --all-targets --all-features

# Lint all packages
[group('lint')]
clippy:
    {{ SETUP_DEBUG_ENV }} cargo clippy --workspace --all-targets --all-features -- --deny warnings

# Test all packages
[group('test')]
test testname="":
    {{ SETUP_DEBUG_ENV }} cargo test --workspace {{ testname }}

# Test all packages with coverage; excludes doctests at this time due to stable Rust limitations
[group('test')]
coverage testname="":
    {{ SETUP_DEBUG_ENV }} cargo llvm-cov test --workspace {{ testname }}
    {{ SETUP_DEBUG_ENV }} cargo llvm-cov report --html
    {{ SETUP_DEBUG_ENV }} cargo llvm-cov report --lcov --output-path target/llvm-cov/lcov.info

# Run only doctests
[group('test')]
doctest testname="":
    {{ SETUP_DEBUG_ENV }} cargo test --workspace --doc {{ testname }}

# Review new snapshots taken by insta
[group('test')]
insta-review:
    {{ SETUP_DEBUG_ENV }} cargo insta review

# Build all packages with debug profile
[group('build')]
build-all:
    {{ SETUP_DEBUG_ENV }} cargo build --workspace

# Build default packages with debug profile
[group('build')]
build:
    {{ SETUP_DEBUG_ENV }} cargo build

# Build default packages with release profile
[group('build')]
build-release:
    {{ SETUP_RELEASE_ENV }} cargo build --release

# Build documentation
[group('build')]
doc:
    {{ SETUP_DEBUG_ENV }} cargo doc

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
vcpkg-install:
    vcpkg install

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
    {{ SETUP_DEBUG_ENV }} \
    $rust_env = @{ \
        "rust-analyzer.cargo.extraEnv" = @{ \
            "PKG_CONFIG" = $env:PKG_CONFIG ; \
            "FFMPEG_PKG_CONFIG_PATH" = $env:FFMPEG_PKG_CONFIG_PATH ; \
            "RUSTC_BOOTSTRAP" = $env:RUSTC_BOOTSTRAP \
        } ; \
        "rust-analyzer.runnables.extraEnv" = @{ \
            "PKG_CONFIG" = $env:PKG_CONFIG ; \
            "FFMPEG_PKG_CONFIG_PATH" = $env:FFMPEG_PKG_CONFIG_PATH ; \
            "RUSTC_BOOTSTRAP" = $env:RUSTC_BOOTSTRAP \
        } \
    } | ConvertTo-Json ; \
    New-Item -Force .vscode\rust-environment.json -Value $rust_env | Out-Null

# Set environment variables for use with Visual Studio Code (also requires tabaqa extension)
[group('ide')]
[unix]
vscode-setup:
    {{ SETUP_DEBUG_ENV }} \
    jq -n '{ \
        "rust-analyzer.cargo.extraEnv": { \
            PKG_CONFIG: env.PKG_CONFIG, \
            FFMPEG_PKG_CONFIG_PATH: env.FFMPEG_PKG_CONFIG_PATH, \
            RUSTC_BOOTSTRAP: env.RUSTC_BOOTSTRAP \
        }, \
        "rust-analyzer.runnables.extraEnv": { \
            PKG_CONFIG: env.PKG_CONFIG, \
            FFMPEG_PKG_CONFIG_PATH: env.FFMPEG_PKG_CONFIG_PATH, \
            RUSTC_BOOTSTRAP: env.RUSTC_BOOTSTRAP \
        }, \
    }' > .vscode/rust-environment.json

# Run PowerShell with environment variables set for using cargo.  Profile is "debug" or "release".
[group('tools')]
[windows]
@pwsh profile="debug":
    if ("{{ profile }}" -eq "debug" ) { \
        {{ SETUP_DEBUG_ENV }} \
    } elseif ("{{ profile }}" -eq "release" ) { \
        {{ SETUP_RELEASE_ENV }} \
    } else { \
        Write-Host "Unknown profile {{ profile }}" ; \
        exit 1 \
    } \
    Write-Host "Opening interactive shell for {{ profile }} profile..." ; \
    pwsh

# Run bash with environment variables set for using cargo.  Profile is "debug" or "release".
[group('tools')]
[unix]
@bash profile="debug":
    if [ "{{ profile }}" = "debug" ] ; then \
        {{ SETUP_DEBUG_ENV }} \
    elif [ "{{ profile }}" = "release" ] ; then \
        {{ SETUP_RELEASE_ENV }} \
    else \
        echo "Unknown profile {{ profile }}" ; exit 1 ; \
    fi ; \
    echo "Opening interactive shell for {{ profile }} profile..." ; \
    bash

# Run the "cargo expand" command to show the results of expanding macros in a library target.
[group('tools')]
expand-lib crate path="":
    {{ SETUP_DEBUG_ENV }} cargo expand --package {{ crate }} --lib --tests {{ path }}
