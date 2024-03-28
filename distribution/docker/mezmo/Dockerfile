# use a source that has multi-arch support (amd64+arm64)
FROM rust:1-slim-bullseye as release-builder

# deps for compiler
RUN DEPS='gcc g++ make cmake xz-utils locales python libexpat1-dev gettext libz-dev libssl-dev autoconf pkg-config bzip2 protobuf-compiler libsasl2-dev git libclang-dev' \
  && apt-get update \
  && apt-get install -y --no-install-recommends $DEPS \
  && echo "en_US.UTF-8 UTF-8" >> /etc/locale.gen \
  && locale-gen en_US.UTF-8 \
  && /usr/sbin/update-locale LANG=en_US.UTF-8 \
  && apt-get remove --purge autoconf libz-dev libexpat1-dev -y \
  && apt-get autoremove -y \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/* /usr/share/doc


WORKDIR /build
COPY . .

# Output timberio/vector-dev distro (currently Ubuntu)
RUN cat /etc/os-release

RUN cargo build --release

RUN ls -lh /build/target/release

# Smoke test on relese-builder
RUN ["/build/target/release/vector", "--version"]

# use internal base image
FROM us.gcr.io/logdna-k8s/release:bullseye as release

# install remaining dependencies
RUN apt-get update && apt-get install -y --no-install-recommends systemd gdb \
  && apt-get autoremove -y \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/* /usr/share/doc

COPY --from=release-builder /build/target/release/vector /usr/local/bin/vector
RUN mkdir -p /var/lib/vector

RUN ["gdb", "--version"]
RUN ["vector", "--version"]

ARG BUILD_DATE
ARG VCS_REF
ARG BUILD_VERSION

LABEL org.opencontainers.image.title="answerbook/vector"
LABEL org.opencontainers.image.description="Mezmo vector fork"
LABEL org.opencontainers.image.url="https://mezmo.com"
LABEL org.opencontainers.image.source="https://github.com/answerbook/vector"
LABEL org.opencontainers.image.authors="Mezmo, Inc. <help@mezmo.com>"
LABEL org.opencontainers.image.vendor="Mezmo, Inc."
LABEL org.opencontainers.image.created="${BUILD_DATE}"
LABEL org.opencontainers.image.version="${BUILD_VERSION}"
LABEL org.opencontainers.image.revision="${VCS_REF}"

ENTRYPOINT ["vector"]
