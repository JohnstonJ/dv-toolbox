# The build target is for continuous integration and for local testing.

# Use scm tag.  Regular buildpack-deps, let alone rust image, contains lots of stuff we don't need.
ARG DEBIAN_VERSION=bookworm
ARG LLVM_VERSION=19
FROM buildpack-deps:${DEBIAN_VERSION}-scm AS build

ARG DEBIAN_VERSION
ARG LLVM_VERSION
ARG MOLD_VERSION=2.34.0

ARG TARGETARCH

# Install all the Debian packages we need
RUN apt-get update \
    && apt-get -y upgrade \
    # Add LLVM apt repository
    && wget --quiet -O - https://apt.llvm.org/llvm-snapshot.gpg.key > /etc/apt/keyrings/llvm.asc \
    && echo "deb [signed-by=/etc/apt/keyrings/llvm.asc] http://apt.llvm.org/${DEBIAN_VERSION}/" \
        "llvm-toolchain-${DEBIAN_VERSION}-${LLVM_VERSION} main" \
        > /etc/apt/sources.list.d/llvm.list \
    && apt-get update \
    # Install packages
    && apt-get install -y --no-install-recommends \
        # Install general build tools
        jq \
        make \
        unzip \
        zip \
        # nasm is required by FFmpeg
        nasm \
        # Install LLVM:
        #
        # Compiler and linker: always necessary
        clang-${LLVM_VERSION} \
        lld-${LLVM_VERSION} \
        # Includes LLVM equivalent of binutils
        llvm-${LLVM_VERSION} \
        # libclang is needed by Rust bindgen crate
        libclang1-${LLVM_VERSION} \
    && rm -rf /var/lib/apt/lists/*

# Put LLVM in the PATH so we can use "clang" instead of "clang-19" for example.  Basically this
# makes it the default.  This is rather hacky vs a proper way of using update-alternatives, but it's
# also a one-liner that doesn't require calling update-alternatives for dozens of tools.
ENV PATH=/usr/lib/llvm-19/bin:$PATH

# Install a newer release of mold linker, which is much newer than what Debian provides.
# We'll only use it for 1st-party debug binaries for now.
RUN \
    # Set filename for architecture
    case "${TARGETARCH}" in \
        amd64) MOLD_FILENAME=mold-${MOLD_VERSION}-x86_64-linux ;; \
        *) echo "Unknown architecture." ; exit 1 ;; \
    esac ; \
    wget https://github.com/rui314/mold/releases/download/v${MOLD_VERSION}/${MOLD_FILENAME}.tar.gz \
    && tar -xvzf $MOLD_FILENAME.tar.gz -C /usr/local --strip-components 1 \
    && rm $MOLD_FILENAME.tar.gz

# Install Rust
ENV PATH=/root/.cargo/bin:$PATH
RUN \
    case "${TARGETARCH}" in \
        amd64) HOST_ARCH=x86_64-unknown-linux-gnu ;; \
        *) echo "Unknown architecture." ; exit 1 ;; \
    esac ; \
    # Download the latest rust and rustup
    wget https://static.rust-lang.org/rustup/dist/$HOST_ARCH/rustup-init \
    && chmod +x rustup-init \
    && ./rustup-init -y --no-modify-path \
    && rm rustup-init \
    # Used by cargo-llvm-cov
    && rustup component add llvm-tools-preview

# Install cargo-binstall: install prebuilt Rust binaries in a similar style to "cargo install"
# From https://github.com/cargo-bins/cargo-binstall?tab=readme-ov-file#linux-and-macos
RUN wget https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh \
    && chmod +x install-from-binstall-release.sh \
    && ./install-from-binstall-release.sh \
    && rm ./install-from-binstall-release.sh

# Install other binary tools:
# cargo-run-bin: install other cargo tools to project's bin directory
# just: a command runner
RUN cargo binstall -y \
        cargo-run-bin \
        just

# Install vcpkg and make it globally available
ENV VCPKG_ROOT=/root/.vcpkg \
    PATH=/root/.vcpkg:$PATH
RUN git clone https://github.com/microsoft/vcpkg.git ~/.vcpkg && \
    cd ~/.vcpkg && \
    ./bootstrap-vcpkg.sh && \
    vcpkg integrate install

# The dev target creates a development environment with more tools

FROM build AS dev

ARG LLVM_VERSION

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        # General tools
        bash-completion \
        vim \
        # LLVM debugger and language server
        clangd-${LLVM_VERSION} \
        lldb-${LLVM_VERSION} \
        # Web browser, in case you want to use it to view documentation or other such things
        firefox-esr \
    && rm -rf /var/lib/apt/lists/*

RUN echo >> /etc/bash.bashrc \
    # Enable bash completions in non-login shells
    && echo ". /etc/profile.d/bash_completion.sh" >> /etc/bash.bashrc \
    # Install various completions of interest
    && mkdir -p ~/.local/share/bash-completion/completions \
    && just --completions bash > ~/.local/share/bash-completion/completions/just \
    && rustup completions bash cargo >> ~/.local/share/bash-completion/completions/cargo \
    && rustup completions bash rustup >> ~/.local/share/bash-completion/completions/rustup
