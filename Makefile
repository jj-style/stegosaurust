.PHONY: help
help:
	@echo "make <arg>"
	@echo "Arguments:"
	@echo "  all"
	@echo "  build"
	@echo "  test"
	@echo "  clean"
	@echo "  lint"
	@echo "  fmt"

.PHONY: all
all: build

.PHONY: build
build:
	@cargo build

.PHONY: test
test:
	@cargo test

.PHONY: clean
clean:
	@cargo clean

.PHONY: lint
lint:
	@cargo clippy -- -D warnings

.PHONY: fmt
fmt:
	@cargo fmt