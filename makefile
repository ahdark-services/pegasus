GO=go

GOFLAGS=-ldflags="-s -w"
ifeq ($(shell uname -s),Darwin)
	GOFLAGS+=-buildmode=pie
endif

COMPONENTS=$(shell find components -type d -depth 1 -exec basename {} \;)

build-%:
	$(GO) build $(GOFLAGS) -o bin/$* components/$*/cmd/main.go

build:
	@for component in $(COMPONENTS); do \
		$(MAKE) build-$$component; \
	done

run-%:
	$(GO) run components/$*/cmd/main.go

test:
	$(GO) test -v ./...

deps:
	$(GO) mod download

work-init:
	$(GO) work init
	$(GO) work use . components/*

.PHONY: build-% build run-% test deps work-init
