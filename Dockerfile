# syntax=docker/dockerfile:1

FROM rust:1.81.0-slim-bookworm AS base

FROM base AS build-linux
WORKDIR /src
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=bind,target=. \
    cargo build --release --target-dir /target

FROM base AS build-windows
RUN rustup target add x86_64-pc-windows-gnu
RUN --mount=type=cache,target=/var/lib/apt,sharing=locked \
    --mount=type=cache,target=/var/cache/apt,sharing=locked \
    apt update && apt install -y gcc-mingw-w64-x86-64-win32
WORKDIR /src
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=bind,target=. \
    cargo build --release --target-dir /target --target=x86_64-pc-windows-gnu

FROM scratch AS linux
COPY --from=build-linux /target/release/e2e /

FROM scratch AS windows
COPY --from=build-windows /target/x86_64-pc-windows-gnu/release/e2e.exe /

FROM base AS test
WORKDIR /src
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=bind,target=.,rw=true \
    cargo test
