ARG RUST_VERSION
FROM docker.io/rust:${RUST_VERSION}-slim-bookworm

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    cmake \
    curl \
    g++ \
    libclang1 \
    libsasl2-dev \
    libssl-dev \
    llvm \
    pkg-config \
    zlib1g-dev \
    unzip \
    git \
  && rm -rf /var/lib/apt/lists/*

ARG GITHUB_TOKEN
RUN git config --global url."https://${GITHUB_TOKEN}@github.com".insteadOf ssh://git@github.com

RUN rustup run "${RUST_VERSION}" cargo install cargo-nextest --version 0.9.25 --locked

COPY scripts/environment/install-protoc.sh /
COPY tests/data/ca/certs /certs
RUN bash /install-protoc.sh
