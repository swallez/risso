#
# Run "make" to have a list of the main targets
#

# Rust compiler image - see https://github.com/clux/muslrust
muslrust-img = clux/muslrust:1.30.0-stable

# Rust-in-Docker invocation
# The 'muslrust-cache' volume avoids downloading the dependencies at each build
muslrust-builder := docker run --rm -it\
  -v $(PWD):/volume\
  -v muslrust-cache:/root/.cargo\
  $(muslrust-img)

.PHONY: help check lint build release reports

help:
	@echo "Main targets:"
	@echo "- check:        check code for errors"
	@echo "- lint:         run the clippy linter"
	@echo "- deps-tree:    print the dependency tree"
	@echo "- cloc:         count lines of code"
	@echo "- docker-image: build the 'risso-actix' Docker image"
	@echo "- doc:          build the docs"
	@echo "- build:        build the risso_actix binary"
	@echo "- release:      build the optimized risso_actix binary"
	@echo "- reports:      reports on license, outdated crates and security warnings"

check:
	cargo check --all --all-targets

lint:
	cargo clippy

deps-tree:
	(cd risso_actix; cargo tree --no-dev-dependencies)

cloc:
	cloc --vcs=git

doc:
	cargo doc --no-deps

build:
	cargo build --package risso_actix --bin risso_actix

release:
	cargo build --package risso_actix --bin risso_actix --release

build-musl:
	$(muslrust-builder) cargo build --package risso_actix --release --bin risso_actix
	$(muslrust-builder) strip --only-keep-debug target/x86_64-unknown-linux-musl/release/risso_actix

docker-image: build-musl
	docker build -t risso-actix .

docker-shell:
	$(muslrust-builder) bash

reports:
	@echo "--------------------------------------------------------------------------------"
	@echo " License report"
	@echo "--------------------------------------------------------------------------------"
	@cargo license
	@echo
	@echo "--------------------------------------------------------------------------------"
	@echo " Outdated crates"
	@echo "--------------------------------------------------------------------------------"
	@cargo outdated
	@echo
	@echo "--------------------------------------------------------------------------------"
	@echo " Security audit
	@echo "--------------------------------------------------------------------------------"
	@cargo audit
