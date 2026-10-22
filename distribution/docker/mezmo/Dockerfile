# use a source that has multi-arch support (amd64+arm64)
FROM rust:1-slim-trixie AS release-builder
ARG GITHUB_TOKEN
ARG BUILD_DATE
ARG VCS_REF
ARG BUILD_VERSION
ARG RELEASE_VERSION

# deps for compiler
RUN DEPS='gcc g++ make cmake xz-utils locales python3 libexpat1-dev gettext libz-dev libssl-dev autoconf pkg-config bzip2 protobuf-compiler libsasl2-dev git libclang-dev' \
  && apt-get update \
  && apt-get install -y --no-install-recommends $DEPS \
  && echo "en_US.UTF-8 UTF-8" >> /etc/locale.gen \
  && locale-gen en_US.UTF-8 \
  && /usr/sbin/update-locale LANG=en_US.UTF-8 \
  && apt-get remove --purge autoconf libz-dev libexpat1-dev -y \
  && apt-get autoremove -y \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/* /usr/share/doc

RUN git config --global url."https://${GITHUB_TOKEN}@github.com".insteadOf ssh://git@github.com

WORKDIR /build
COPY . .

# Output timberio/vector-dev distro (currently Ubuntu)
RUN cat /etc/os-release

ENV RELEASE_VERSION=${RELEASE_VERSION}
RUN cargo build --release

# The compile leaves the line-table debug info from `[profile.release] debug = 1`
# embedded, which inflates the binary to well over 1GB. Keep the full binary
# (`vector.full`) for the debug image, where gdb and the jeprof heap-profile
# tooling symbolize against it, and strip a copy for the ordinary runtime image
# so routine pulls/deploys move a fraction of the bytes.
RUN cp target/release/vector target/release/vector.full \
  && strip --strip-all target/release/vector

RUN ls -lh /build/target/release

# Smoke test on release-builder
RUN ["/build/target/release/vector", "--version"]

# Shared runtime base for both the release and debug images. The vector binary
# is added by each leaf stage because they ship different builds (stripped vs
# full), so it is intentionally not copied here.
FROM us.gcr.io/logdna-k8s/release:trixie AS runtime-base
ARG BUILD_DATE
ARG VCS_REF
ARG BUILD_VERSION

# install remaining dependencies
RUN apt-get update && apt-get install -y --no-install-recommends systemd \
  && apt-get autoremove -y \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/* /usr/share/doc

RUN mkdir -p /var/lib/vector
RUN mkdir -p /etc/protobuf_descs
COPY distribution/docker/mezmo/opentelemetry-proto.desc /etc/protobuf_descs/opentelemetry-proto.desc

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

# Debug image (built on demand via `--target release-debug`): ships the full,
# unstripped binary plus gdb so a debugger can attach and the jeprof tooling can
# symbolize heap dumps against /usr/local/bin/vector exactly as before.
FROM runtime-base AS release-debug
RUN apt-get update && apt-get install -y --no-install-recommends gdb \
  && apt-get autoremove -y \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/* /usr/share/doc

COPY --from=release-builder /build/target/release/vector.full /usr/local/bin/vector

RUN ["gdb", "--version"]
RUN ["vector", "--version"]

# Default runtime image: stripped binary only, no debugger. This is the last
# stage, so a build with no `--target` produces it (unchanged behavior for the
# existing Jenkins/buildx builds, now with a much smaller binary).
FROM runtime-base AS release
COPY --from=release-builder /build/target/release/vector /usr/local/bin/vector
RUN ["vector", "--version"]
