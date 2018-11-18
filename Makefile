#
# Run "make" to have a list of the main targets
#

# Rust compiler image - see https://github.com/clux/muslrust
muslrust-img = clux/muslrust:1.30.0-stable

# Rust-in-Docker invocation
# The 'muslrust-cache' volume avoids downloading the dependencies at each build.
# The 'risso-musl-target' volume caches the 'target' dir. We could also just mount '$PWD/target' but on
# MacOS this makes the build unbearably slow.
muslrust-builder := docker run --rm -it\
  -v $(PWD):/volume:cached\
  -v muslrust-cache:/root/.cargo\
  -v risso-musl-target:/volume/target\
  $(muslrust-img)

help:
	@echo "Main targets:"
	@echo "- check:        check code for errors (faster than 'build')"
	@echo "- lint:         run the clippy linter"
	@echo "- deps-tree:    print the dependency tree"
	@echo "- cloc:         count lines of code"
	@echo "- docker-image: build the 'risso-actix' Docker image"
	@echo "- test:         runs the tests"
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

test:
	cargo test

build:
	cargo build --package risso_actix --bin risso_actix

build-release:
	cargo build --package risso_actix --bin risso_actix --release

musl-test:
	$(muslrust-builder) cargo test --release

musl-build: musl-test
	$(muslrust-builder) sh -c "\
		cargo build --package risso_actix --release --bin risso_actix && \
		strip --only-keep-debug target/x86_64-unknown-linux-musl/release/risso_actix"
	# Create a dummy container for the purpose of copying the executable locally.
	docker container create --name musl-dummy -v risso-musl-target:/volume/target $(muslrust-img)
	docker cp musl-dummy:/volume/target/x86_64-unknown-linux-musl/release/risso_actix target/risso_actix-musl-release
	docker rm musl-dummy

musl-clean:
	docker volume rm -f muslrust-cache
	docker volume rm -f risso-musl-target

musl-shell:
	$(muslrust-builder) bash

docker-image: musl-build
	docker build -t risso-actix .

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
	@echo " Security audit"
	@echo "--------------------------------------------------------------------------------"
	@cargo audit
