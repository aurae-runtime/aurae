# ---------------------------------------------------------------------------- #
#        Apache 2.0 License Copyright © 2022-2023 The Aurae Authors            #
#                                                                              #
#                +--------------------------------------------+                #
#                |   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ |                #
#                |  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ |                #
#                |  ███████║██║   ██║██████╔╝███████║█████╗   |                #
#                |  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   |                #
#                |  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ |                #
#                |  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ |                #
#                +--------------------------------------------+                #
#                                                                              #
#                         Distributed Systems Runtime                          #
#                                                                              #
# ---------------------------------------------------------------------------- #
#                                                                              #
#   Licensed under the Apache License, Version 2.0 (the "License");            #
#   you may not use this file except in compliance with the License.           #
#   You may obtain a copy of the License at                                    #
#                                                                              #
#       http://www.apache.org/licenses/LICENSE-2.0                             #
#                                                                              #
#   Unless required by applicable law or agreed to in writing, software        #
#   distributed under the License is distributed on an "AS IS" BASIS,          #
#   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.   #
#   See the License for the specific language governing permissions and        #
#   limitations under the License.                                             #
#                                                                              #
# ---------------------------------------------------------------------------- #

# Variables and Settings
branch       ?=  main
message      ?=  Default commit message. Aurae Runtime environment.
cargo         =  cargo
oci           =  docker
ociopts       =  DOCKER_BUILDKIT=1
uid           =  $(shell id -u)
uname_m       =  $(shell uname -m)
cri_version   =  release-1.26

# Configuration Options
export GIT_PAGER = cat

#------------------------------------------------------------------------------#

# Aliases

# Keep all as the first command to have it be the default as per convention
.PHONY: all
all: install ## alias for install

#------------------------------------------------------------------------------#

# Notes:
# - make will not run the same step multiple times
# - The ideal order for cargo to reuse artifacts is build -> lint -> test
# - Different cargo target variants (nightly is a variant) do not produce compatible artifacts
# - Cargo's `install` artifacts are not usable for build, lint, or test (and vice versa)

# Super commands

.PHONY: clean
clean: clean-certs clean-gens clean-crates ## Clean the repo

.PHONY: lint
lint: musl libs-lint auraed-lint auraescript-lint aer-lint ## Run all lints

.PHONY: test
test: build lint libs-test auraed-test auraescript-test aer-test ## Builds, lints, and tests (does not include ignored tests)

.PHONY: test-all
test-all: build lint libs-test-all auraed-test-all auraescript-test-all aer-test-all ## Run lints and tests (includes ignored tests)

.PHONY: build
build: musl auraed-build auraescript-build aer-build lint ## Build and lint

.PHONY: install
install: musl lint test auraed-debug auraescript-debug aer-debug  ## Lint, test, and install (debug) 🎉

.PHONY: docs
docs: docs-crates docs-stdlib docs-other ## Assemble all the /docs for the website locally.

.PHONY: prcheck
prcheck: build lint test-all docs docs-lint ## Meant to mimic the GHA checks (includes ignored tests)

#------------------------------------------------------------------------------#

# Setup Commands

.PHONY: pki
pki: certs ## Alias for certs
.PHONY: certs
certs: clean-certs ## Generate x509 mTLS certs in /pki directory
	mkdir -p pki
	./hack/certgen
ifeq ($(uid), 0)
	mkdir -p /etc/aurae/pki
	cp -v pki/* /etc/aurae/pki
else
	sudo -E mkdir -p /etc/aurae/pki
	sudo -E cp -v pki/* /etc/aurae/pki
endif
	@echo "Install PKI Auth Material [/etc/aurae]"

.PHONY: config
config: ## Set up default config
	mkdir -p $(HOME)/.aurae
	cp -v auraescript/default.config.toml $(HOME)/.aurae/config
	sed -i 's|~|$(HOME)|g' $(HOME)/.aurae/config
	mkdir -p $(HOME)/.aurae/pki
	cp -v pki/* $(HOME)/.aurae/pki

.PHONY: musl
musl: ## Add target for musl
	rustup target add $(uname_m)-unknown-linux-musl

#------------------------------------------------------------------------------#

# Clean Commands

.PHONY: clean-crates
clean-crates: ## Clean target directory
	cargo clean

.PHONY: clean-certs
clean-certs: ## Clean the cert material
	rm -rvf pki/*

.PHONY: clean-gen
clean-gens: ## Clean gen directories
	rm -rf aurae-proto/src/gen/*
	rm -rf auraescript/gen/*

#------------------------------------------------------------------------------#

# Protobuf Commands

GEN_TS_PATTERN = auraescript/gen/v0/%.ts
GEN_RS_PATTERN = aurae-proto/src/gen/aurae.%.v0.rs
GEN_SERDE_RS_PATTERN = aurae-proto/src/gen/aurae.%.v0.serde.rs
GEN_TONIC_RS_PATTERN = aurae-proto/src/gen/aurae.%.v0.tonic.rs

PROTOS = $(wildcard api/v0/*/*.proto)
PROTO_DIRS = $(filter-out api/v0/README.md, $(wildcard api/v0/*))

GEN_RS = $(patsubst api/v0/%,$(GEN_RS_PATTERN),$(PROTO_DIRS))
GEN_RS += $(patsubst api/v0/%,$(GEN_SERDE_RS_PATTERN),$(PROTO_DIRS))
GEN_RS += $(patsubst api/v0/%,$(GEN_TONIC_RS_PATTERN),$(PROTO_DIRS))

GEN_TS = $(patsubst api/v0/%.proto,$(GEN_TS_PATTERN),$(PROTOS))

$(GEN_TS_PATTERN) $(GEN_RS_PATTERN) $(GEN_SERDE_RS_PATTERN) $(GEN_TONIC_RS_PATTERN):
	buf lint api
	buf generate -v api

.PHONY: proto-vendor
proto-vendor: proto-vendor-cri proto-vendor-grpc-health ## Copy the upstream protobuf interfaces

.PHONY: proto-vendor-cri
proto-vendor-cri: ## Copy the CRI interface from upstream
	curl https://raw.githubusercontent.com/kubernetes/cri-api/$(cri_version)/pkg/apis/runtime/v1/api.proto -o api/cri/v1/$(cri_version).proto

.PHONY: proto-vendor-grpc-health
proto-vendor-grpc-health: ## Copy the gRPC Health interface from upstream
	curl https://raw.githubusercontent.com/grpc/grpc/master/src/proto/grpc/health/v1/health.proto -o api/grpc/health/v1/health.proto

#------------------------------------------------------------------------------#

# Auraed Commands

.PHONY: auraed
auraed: musl $(GEN_RS) $(GEN_TS) auraed-lint auraed-debug ## Lint and install auraed (for use during development)

.PHONY: auraed-lint
auraed-lint: musl $(GEN_RS) $(GEN_TS)
	$(cargo) clippy --target $(uname_m)-unknown-linux-musl -p auraed --all-features -- -D clippy::all -D warnings

.PHONY: auraed-test
auraed-test: musl $(GEN_RS) $(GEN_TS)
	$(cargo) test --target $(uname_m)-unknown-linux-musl -p auraed

.PHONY: auraed-test-all
auraed-test-all: musl $(GEN_RS) $(GEN_TS)
	sudo -E $(cargo) test --target $(uname_m)-unknown-linux-musl -p auraed -- --include-ignored

.PHONY: auraed-test-watch
auraed-test-watch: musl $(GEN_RS) $(GEN_TS) # Use cargo-watch to continuously run a test (e.g. make auraed-test-watch name=path::to::test)
	sudo -E $(cargo) watch -- $(cargo) test --target $(uname_m)-unknown-linux-musl -p auraed $(name) -- --include-ignored

.PHONY: auraed-build
auraed-build: musl $(GEN_RS) $(GEN_TS)
	$(cargo) build --target $(uname_m)-unknown-linux-musl -p auraed

.PHONY: auraed-build-release
auraed-build-release: musl $(GEN_RS) $(GEN_TS)
	$(cargo) build --target $(uname_m)-unknown-linux-musl -p auraed --release

.PHONY: auraed-debug
auraed-debug: musl $(GEN_RS) $(GEN_TS) auraed-lint
	$(cargo) install --target $(uname_m)-unknown-linux-musl --path ./auraed --debug --force

.PHONY: auraed-release
auraed-release: musl $(GEN_RS) $(GEN_TS) auraed-lint auraed-test ## Lint, test, and install auraed
	$(cargo) install --target $(uname_m)-unknown-linux-musl --path ./auraed --force

.PHONY: start
auraed-start: ## Starts the installed auraed executable
	sudo -E $(HOME)/.cargo/bin/auraed

#------------------------------------------------------------------------------#

# AuraeScript Commands

.PHONY: auraescript
auraescript: $(GEN_TS) $(GEN_RS) auraescript-lint auraescript-debug ## Lint and install auraescript (for use during development)

.PHONY: auraescript-lint
auraescript-lint: $(GEN_TS) $(GEN_RS)
	$(cargo) clippy -p auraescript --all-features -- -D clippy::all -D warnings

.PHONY: auraescript-test
auraescript-test: $(GEN_TS) $(GEN_RS)
	$(cargo) test -p auraescript

.PHONY: auraescript-test-all
auraescript-test-all: $(GEN_TS) $(GEN_RS)
	$(cargo) test -p auraescript -- --include-ignored

.PHONY: auraescript-build
auraescript-build: musl $(GEN_TS) $(GEN_RS)
	$(cargo) build -p auraescript

.PHONY: auraescript-build-release
auraescript-build-release: musl $(GEN_RS) $(GEN_TS)
	$(cargo) build -p auraescript --release

.PHONY: auraescript-debug
auraescript-debug: $(GEN_RS) $(GEN_TS) auraescript-lint
	$(cargo) install --path ./auraescript --debug --force

.PHONY: auraescript-release
auraescript-release: $(GEN_RS) $(GEN_TS) auraescript-lint auraescript-test ## Lint, test, and install auraescript
	$(cargo) install --path ./auraescript --force

#------------------------------------------------------------------------------#

# aer Commands

.PHONY: aer
aer: $(GEN_RS) $(GEN_TS) aer-lint aer-debug ## Lint and install aer (for use during development)

.PHONY: aer-lint
aer-lint: $(GEN_RS) $(GEN_TS)
	$(cargo) clippy -p aer --all-features -- -D clippy::all -D warnings

.PHONY: aer-test
aer-test: $(GEN_RS) $(GEN_TS)
	$(cargo) test -p aer

.PHONY: aer-test-all
aer-test-all: $(GEN_RS) $(GEN_TS)
	$(cargo) test -p aer -- --include-ignored

.PHONY: aer-build
aer-build: musl $(GEN_RS) $(GEN_TS)
	$(cargo) build -p aer

.PHONY: aer-build-release
aer-build-release: musl $(GEN_RS) $(GEN_TS)
	$(cargo) build -p aer --release

.PHONY: aer-debug
aer-debug: $(GEN_RS) $(GEN_TS) aer-lint
	$(cargo) install --path ./aer --debug --force

.PHONY: aer-release
aer-release: $(GEN_RS) $(GEN_TS) aer-lint aer-test ## Lint, test, and install aer
	$(cargo) install --path ./aer --force

#------------------------------------------------------------------------------#

# Commands for other crates

.PHONY: libs-lint
libs-lint: $(GEN_RS) $(GEN_TS)
	$(cargo) clippy --all-features --workspace --exclude auraed --exclude auraescript --exclude aer  -- -D clippy::all -D warnings

.PHONY: libs-test
libs-test: $(GEN_RS) $(GEN_TS)
	$(cargo) test --workspace --exclude auraed --exclude auraescript --exclude aer

.PHONY: libs-test-all
libs-test-all: $(GEN_RS) $(GEN_TS)
	$(cargo) test --workspace --exclude auraed --exclude auraescript --exclude aer -- --include-ignored

#------------------------------------------------------------------------------#

# Documentation Commands

.PHONY: docs-lint
docs-lint: # Check the docs for typos
	vale --no-wrap --glob='!docs/stdlib/v0/*' ./docs

.PHONY: docs-stdlib
## Generate the docs for the stdlib from the .proto files
ifeq (, $(wildcard /usr/local/bin/protoc-gen-doc))
docs-stdlib:
	$(error "No /usr/local/bin/protoc-gen-doc, install from https://github.com/pseudomuto/protoc-gen-doc")
else
docs-stdlib: $(GEN_TS) $(GEN_RS)
	protoc --plugin=/usr/local/bin/protoc-gen-doc -I api/v0/discovery -I api/v0/observe -I api/v0/cells --doc_out=docs/stdlib/v0 --doc_opt=markdown,index.md:Ignore* api/v0/*/*.proto --experimental_allow_proto3_optional
endif


.PHONY: docs-crates
docs-crates: musl $(GEN_TS) $(GEN_RS) ## Build the crate (documentation)
	$(cargo) doc --target $(uname_m)-unknown-linux-musl --no-deps --package auraed
	$(cargo) doc --no-deps --package auraescript
	$(cargo) doc --no-deps --package aurae-client
	$(cargo) doc --no-deps --package aer
	cp -rv target/doc/* docs/crate

.PHONY: docs-other
docs-other:
	cp -rv README.md docs/index.md # Special copy for the main README
	cp -rv api/README.md docs/stdlib/index.md # Special copy for the main README

.PHONY: docs-serve
docs-serve: docs ## Run the aurae.io static website locally
	sudo -E ./hack/serve.sh

#------------------------------------------------------------------------------#

# Container Commands

.PHONY: oci-image-build
oci-image-build: ## Build the aurae/auraed OCI images
	$(ociopts) $(oci) build -t $(tag) -f $(ocifile) $(flags) .

.PHONY: oci-run
oci-run: ## Run the aurae/auraed OCI images
	$(ociopts) $(oci) run -v $(shell pwd):/app $(flags) $(tag) $(command)

.PHONY: oci-make
oci-make: ## Run the makefile inside the aurae/auraed OCI images
	$(ociopts) $(oci) run -v $(shell pwd):/app --rm -it $(tag) $(command)

.PHONY: oci-push
oci-push: ## Push to a user repository
	$(ociopts) $(oci) push $(tag)

.PHONY: oci-image-build-raw
oci-image-build-raw: ## Plain Jane oci build
	$(oci) build -t $(tag) -f $(ocifile) $(flags) .

.PHONY: container
container: ## Build the container defined in hack/container
	./hack/container

.PHONY: spawn
spawn: ## Spawn the current auraed binary and start it in a container
	./hack/spawn

.PHONY: busybox
busybox: ## Creat a "busybox" OCI bundle in target
	./hack/oci-busybox

.PHONY: alpine
alpine: ## Creat an "alpine" OCI bundle in target
	./hack/oci-alpine

#------------------------------------------------------------------------------#

# CI Commands

.PHONY: ci-release
ci-release: test auraed-build-release auraescript-build-release aer-build-release # Preps release artifacts (for CI use)

.PHONY: ci-stage-release-artifacts
ci-stage-release-artifacts: ci-release ## Preps and stages release artifacts (for CI use)
	mkdir -p /tmp/release
	cp target/$(uname_m)-unknown-linux-musl/release/auraed /tmp/release/auraed-$(tag)-$(uname_m)-unknown-linux-musl
	cp target/release/auraescript /tmp/release/auraescript-$(tag)-$(uname_m)-unknown-linux-gnu

.PHONY: ci-upload-release-artifacts
ci-upload-release-artifacts: ci-release ci-stage-release-artifacts ## Preps, stages, and uploads release artifacts to github (for CI use)
	gh release upload $(tag) /tmp/release/auraed-$(tag)-$(uname_m)-unknown-linux-musl
	gh release upload $(tag) /tmp/release/auraescript-$(tag)-$(uname_m)-unknown-linux-gnu

.PHONY: ci-local
ci-local: ## Tests a github action's workflow locally using `act` (e.g., `make ci-local file=001-tester-ubuntu-make-test.yml`)
	act -W ./.github/workflows/$(file)

#------------------------------------------------------------------------------#

# Other Commands

.PHONY: tlsinfo
tlsinfo: ## Show TLS Info for /var/run/aurae*
	./hack/server-tls-info

.PHONY: fmt
fmt: headers ## Format the entire code base(s)
	./hack/code-format

.PHONY: headers
headers: headers-write ## Fix headers. Run this if you want to clobber things.

.PHONY: headers-check
headers-check: ## Only check for problematic files.
	./hack/headers-check

.PHONY: headers-write
headers-write: ## Fix any problematic files blindly.
	./hack/headers-write

.PHONY: help
help:  ## Show help messages for make targets
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(firstword $(MAKEFILE_LIST)) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[32m%-30s\033[0m %s\n", $$1, $$2}'

.PHONY: check-deps
check-deps: musl ## Check if there are any unused dependencies in Cargo.toml
#	cargo +nightly udeps --target $(uname_m)-unknown-linux-musl --package auraed
#	cargo +nightly udeps --package auraescript
#	cargo +nightly udeps --package aurae-client
#	cargo +nightly udeps --package aer
