GO=go
CARGO=cargo

GOFLAGS=-ldflags="-s -w"
ifeq ($(shell uname -s),Darwin)
	GOFLAGS+=-buildmode=pie
endif

GO_COMPONENTS=$(shell find components -type d -depth 1 -exec basename {} \;)
RUST_COMPONENTS=$(shell find rust-components -type d -depth 1 -exec basename {} \;)

build-go-%:
	$(GO) build $(GOFLAGS) -o bin/$* components/$*/cmd/main.go

build-rust-%:
	$(CARGO) build --release --manifest-path=rust-components/$*/Cargo.toml

build:
	@for component in $(GO_COMPONENTS); do \
		$(MAKE) build-go-$$component; \
	done

	@for component in $(RUST_COMPONENTS); do \
		$(MAKE) build-rust-$$component; \
	done

test:
	$(GO) test -v ./...
	$(CARGO) test

work-init:
	$(GO) work init
	$(GO) work use . components/*

.PHONY: build-go-% build-rust-% build test deps work-init
