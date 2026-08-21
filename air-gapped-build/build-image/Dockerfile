# EEL build-image, Docker variant.
# Identical to Containerfile; provided for environments where Podman is unavailable.
# Prefer Containerfile for air-gapped Podman builds.
# Syntax is compatible with BuildKit (docker buildx).
# syntax=docker/dockerfile:1.6

FROM nixos/nix:2.24.9 AS nix-base

RUN mkdir -p /etc/nix && echo 'experimental-features = nix-command flakes' >> /etc/nix/nix.conf

RUN nix profile install \
    nixpkgs#git \
    nixpkgs#jq \
    nixpkgs#yq-go \
    nixpkgs#curl \
    nixpkgs#openssl \
    nixpkgs#skopeo \
    nixpkgs#pre-commit \
    --no-write-lock-file

RUN nix profile install \
    nixpkgs#cosign \
    nixpkgs#syft \
    nixpkgs#grype \
    nixpkgs#trivy \
    nixpkgs#slsa-verifier \
    --no-write-lock-file

RUN nix profile install \
    nixpkgs#nodePackages.spectral \
    nixpkgs#buf \
    nixpkgs#nodePackages.ajv-cli \
    --no-write-lock-file

RUN nix profile install \
    nixpkgs#rustup \
    --no-write-lock-file && \
    rustup toolchain install stable nightly && \
    rustup component add rustfmt clippy --toolchain stable && \
    cargo install cargo-nextest cargo-fuzz cargo-mutants cargo-deny cargo-audit cargo-vet \
        --locked 2>/dev/null || true

RUN nix profile install \
    nixpkgs#uv \
    nixpkgs#python312 \
    --no-write-lock-file && \
    uv tool install ruff bandit pip-audit

RUN nix profile install \
    nixpkgs#nodejs_22 \
    nixpkgs#pnpm \
    --no-write-lock-file && \
    pnpm add -g vitest openapi-diff

RUN nix profile install \
    nixpkgs#elixir \
    --no-write-lock-file

RUN nix profile install \
    nixpkgs#flutter \
    --no-write-lock-file

RUN nix profile install \
    nixpkgs#spire \
    nixpkgs#argocd \
    --no-write-lock-file

RUN nix-collect-garbage --delete-old

WORKDIR /workspace

LABEL org.opencontainers.image.title="eel/build-image" \
      org.opencontainers.image.description="EEL CI build environment, Docker variant" \
      org.opencontainers.image.source="https://gitea.internal/eel/eel" \
      org.opencontainers.image.licenses="Apache-2.0"
