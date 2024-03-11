FROM rust:1 as builder
WORKDIR /usr/src/pegasus

ARG COMPONENT

RUN rustup default nightly

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release --bin ${COMPONENT}

FROM debian:bookworm-slim as runner
WORKDIR /app

ARG COMPONENT

RUN apt update && \
    apt install -y openssl libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/pegasus/target/release/${COMPONENT} /app/entry

COPY docker/rust-basic/config.yaml* /app/

RUN chown -R 1000:1000 /app
RUN chmod -R 755 /app

USER 1000

ENTRYPOINT ["/app/entry"]
