# Two stages, the house shape: a stable-Rust builder compiles the server, a
# bookworm-slim runtime receives one binary — no source, no cargo, no rustc.
# The server speaks LSP over stdio, so the image's contract is: run it, connect
# stdio, done. No ports are exposed on purpose.

FROM rust:slim-bookworm AS build
WORKDIR /src
# Dependencies first: this layer caches across source-only changes.
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates crates
RUN cargo build --release --locked

FROM debian:bookworm-slim
COPY --from=build /src/target/release/tessariql-lsp /usr/local/bin/tessariql-lsp
ENTRYPOINT ["tessariql-lsp"]
