FROM rust:1 as builder
WORKDIR /usr/src/pegasus

ARG COMPONENT
ARG profile=release

RUN rustup default nightly

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --bin ${COMPONENT} --profile ${profile}

FROM debian:bookworm-slim as runner
WORKDIR /app

ARG COMPONENT

RUN apt update
RUN apt install -y openssl libssl-dev ca-certificates
RUN rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/pegasus/target/release/${COMPONENT} /app/entry

COPY docker/rust-basic/config.yaml* /app/

#RUN if [ "${COMPONENT}" = "network-functions-handler" ]; then sudo setcap cap_net_raw=eip /app/entry; fi # Required for the fast ping command

USER root
RUN chmod +x /app/entry

ENTRYPOINT ["/app/entry"]
