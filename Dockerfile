# syntax=docker/dockerfile:experimental
# !!! This won't build with docker-compose due to BUILDKIT !!!
# You can build it using (from the root of the repo):
#     docker build -f client/Dockerfile -t reinfer-dev/client .
ARG REINFER_BASE_IMAGE_TAG=latest
FROM eu.gcr.io/reinfer-gcr/builder:${REINFER_BASE_IMAGE_TAG} as builder

COPY ci/rshelp /src/ci/rshelp
COPY client /src/client

# Note that `PROJECT_PATH` needs to be updated here, but also in the volume on the first RUN line,
# env/arg expansion in --mount commands is not supported by docker yet: https://github.com/moby/buildkit/issues/815
ENV PROJECT_PATH="client"
WORKDIR "/src/${PROJECT_PATH}"

ARG SCCACHE_REDIS
ENV PKG_CONFIG_ALLOW_CROSS=1
RUN --mount=type=cache,id=cargo-git,target=/root/.cargo/git,sharing=locked                 \
    --mount=type=cache,id=cargo-registry,target=/root/.cargo/registry,sharing=locked       \
    --mount=type=cache,id=client-target,target=/src/client/target                          \
    /src/ci/rshelp/rs-build                                                   && \
    cargo build --locked --release --target x86_64-unknown-linux-musl         && \
    # So this is kinda ugly, but we want to keep the volumes around for          \
    # `cargo test`, so we need to put them somewhere else then link back to      \
    # them with the volumes unmounted.                                           \
    mkdir -p /build/${PROJECT_PATH}                                           && \
    cp -ar /src/${PROJECT_PATH}/target /build/${PROJECT_PATH}/target          && \
    cp -ar /root/.cargo/git /build/cargo-git                                  && \
    cp -ar /root/.cargo/registry /build/cargo-registry

RUN rmdir /src/${PROJECT_PATH}/target /root/.cargo/git /root/.cargo/registry  && \
    ln -s /build/${PROJECT_PATH}/target /src/${PROJECT_PATH}/                 && \
    ln -s /build/cargo-git /root/.cargo/git                                   && \
    ln -s /build/cargo-registry /root/.cargo/registry

ARG REINFER_BASE_IMAGE_TAG=latest
FROM eu.gcr.io/reinfer-gcr/rs:${REINFER_BASE_IMAGE_TAG}
COPY --from=builder /build/client/target/x86_64-unknown-linux-musl/release/re /usr/bin/re
ENTRYPOINT [ "/usr/bin/tini", "--", "/usr/bin/re" ]
