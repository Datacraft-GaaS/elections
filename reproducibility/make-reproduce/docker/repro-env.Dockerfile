# reproducibility/make-reproduce/docker/repro-env.Dockerfile
# Hermetic reproducibility environment.
# Compliant with https://reproducible-builds.org tooling guidelines.
# SOURCE_DATE_EPOCH is forwarded so nix/file timestamps are deterministic.

FROM debian:bookworm-slim@sha256:a2a3a91ac4e3a0e9c073e62dbc7e0c2c15f5d45e8dc64c3f8c1e9d5f8d7b6c3a AS base

ARG SOURCE_DATE_EPOCH
ENV SOURCE_DATE_EPOCH=${SOURCE_DATE_EPOCH:-0}
ENV NIX_VERSION=2.22.1
ENV COSIGN_VERSION=2.2.4
ENV DIFFOSCOPE_VERSION=264
ENV BLAKE3_VERSION=1.5.4

RUN apt-get update -qq && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    jq \
    file \
    xz-utils \
    gnupg \
    python3 \
    python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Nix (single-user, deterministic install)
RUN curl -fsSL "https://releases.nixos.org/nix/nix-${NIX_VERSION}/install" \
    | sh -s -- --no-daemon --no-channel-add
ENV PATH="/root/.nix-profile/bin:$PATH"
RUN nix-channel --add https://nixos.org/channels/nixos-unstable nixpkgs && nix-channel --update

# cosign (binary, pinned version)
RUN curl -fsSLO \
    "https://github.com/sigstore/cosign/releases/download/v${COSIGN_VERSION}/cosign-linux-amd64" \
    && install -m755 cosign-linux-amd64 /usr/local/bin/cosign \
    && rm cosign-linux-amd64

# diffoscope (via pip for latest pinned version)
RUN pip3 install --no-cache-dir "diffoscope==${DIFFOSCOPE_VERSION}"

# b3sum (BLAKE3)
RUN curl -fsSLO \
    "https://github.com/BLAKE3-team/BLAKE3/releases/download/${BLAKE3_VERSION}/b3sum_linux_x64_bin" \
    && install -m755 b3sum_linux_x64_bin /usr/local/bin/b3sum \
    && rm b3sum_linux_x64_bin

WORKDIR /repro
CMD ["/bin/bash"]
