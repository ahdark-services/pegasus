FROM golang:1.22 as builder
WORKDIR /go/src/github.com/ahdark-services/pegasus

ARG COMPONENT

COPY . .

RUN make work-init

RUN --mount=type=cache,target=/go/pkg/mod make deps

RUN CGO_ENABLED=0 GOOS=linux make build-${COMPONENT}
FROM golang:1.22 as builder
WORKDIR /go/src/github.com/ahdark-services/pegasus

ARG COMPONENT

COPY . .

RUN make work-init

RUN --mount=type=cache,target=/go/pkg/mod make deps
RUN --mount=type=cache,target=/go/pkg/mod CGO_ENABLED=0 GOOS=linux make build-${COMPONENT}

FROM alpine:3.7 as runner
ARG COMPONENT
WORKDIR /app

COPY --from=builder /go/src/github.com/ahdark-services/pegasus/bin/${COMPONENT} /app/entry
COPY docker/basic/config.yaml /app/config.yaml

USER 1000
EXPOSE 9201

CMD ["/app/entry"]