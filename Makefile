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
uname_m       =  $(shell uname -m)
cri_version   =  release-1.26

# Configuration Options
export GIT_PAGER = cat

#------------------------------------------------------------------------------#

# Convenience commands

.PHONY: all
all: install ## Build and install (debug) 🎉

.PHONY: build
build: install ## Build and install (debug) (+musl) 🎉

#------------------------------------------------------------------------------#

# Super commands

.PHONY: clean
clean: clean-certs clean-gens clean-crates ## Clean the repo

.PHONY: lint
lint: musl proto proto-lint libs-lint auraed-lint auraescript-lint aer-lint ## Run all lints

.PHONY: test
test: lint libs-test auraed-test auraescript-test aer-test ## Run lints and tests (does not include ignored tests)

.PHONY: test-all
test-all: lint libs-test-all auraed-test-all auraescript-test-all aer-test-all ## Run lints and tests (includes ignored tests)

.PHONY: install
install: musl proto auraed-debug auraescript-debug aer-debug ## Build and install (debug) 🎉

.PHONY: release
release: lint test auraed-release auraescript-release aer-release ## Build and install (release) 🎉

.PHONY: docs
docs: proto docs-crates docs-stdlib docs-other docs-lint ## Assemble all the /docs for the website locally.

.PHONY: prcheck
prcheck: install lint test-all docs ## Meant to mimic the GHA checks (includes ignored tests)

#------------------------------------------------------------------------------#

# Setup Commands

.PHONY: pki
pki: certs ## Alias for certs
.PHONY: certs
certs: clean-certs ## Generate x509 mTLS certs in /pki directory
	mkdir -p pki
	./hack/certgen
	sudo -E mkdir -p /etc/aurae/pki
	sudo -E cp -v pki/* /etc/aurae/pki
	@echo "Install PKI Auth Material [/etc/aurae]"

.PHONY: config
config: ## Set up default config
	@mkdir -p $(HOME)/.aurae
	@cp -v auraescript/default.config.toml $(HOME)/.aurae/config
	@sed -i 's|~|$(HOME)|g' $(HOME)/.aurae/config
	@mkdir -p $(HOME)/.aurae/pki
	@cp -v pki/* $(HOME)/.aurae/pki

.PHONY: musl
musl: ## Add target for musl
	rustup target add $(uname_m)-unknown-linux-musl

#------------------------------------------------------------------------------#

# Clean Commands

.PHONY: clean-crates
clean-crates: ## Clean target directory
	@cargo clean

.PHONY: clean-certs
clean-certs: ## Clean the cert material
	@rm -rvf pki/*

.PHONY: clean-gen
clean-gens: ## Clean gen directories
	@rm -rf aurae-proto/src/gen/*
	@rm -rf auraescript/gen/*

#------------------------------------------------------------------------------#

# Protobuf Commands

.PHONY: proto
proto: proto-lint ## Lint and Generate code from protobuf schemas
	@buf --version >/dev/null 2>&1 || (echo "Warning: buf is not installed! Please install the 'buf' command line tool: https://docs.buf.build/installation"; exit 1)
	buf generate -v api

.PHONY: proto-lint
proto-lint: ## Lint protobuf schemas
	buf lint api

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
auraed: proto auraed-lint auraed-debug ## Lint and install auraed (for use during development)

.PHONY: auraed-lint
auraed-lint:
	@$(cargo) clippy --target $(uname_m)-unknown-linux-musl -p auraed --all-features -- -D clippy::all -D warnings

.PHONY: auraed-test
auraed-test:
	@$(cargo) test --target $(uname_m)-unknown-linux-musl -p auraed

.PHONY: auraed-test-all
auraed-test-all:
	@sudo -E $(cargo) test --target $(uname_m)-unknown-linux-musl -p auraed -- --include-ignored

.PHONY: auraed-debug
auraed-debug:
	@$(cargo) install --target $(uname_m)-unknown-linux-musl --path ./auraed --debug --force

.PHONY: auraed-release
auraed-release:
	@$(cargo) install --target $(uname_m)-unknown-linux-musl --path ./auraed --force

#------------------------------------------------------------------------------#

# AuraeScript Commands

.PHONY: auraescript
auraescript: proto auraescript-lint auraescript-debug ## Lint and install auraescript (for use during development)

.PHONY: auraescript-lint
auraescript-lint:
	@$(cargo) clippy -p auraescript --all-features -- -D clippy::all -D warnings

.PHONY: auraescript-test
auraescript-test:
	@$(cargo) test -p auraescript

.PHONY: auraescript-test-all
auraescript-test-all:
	@$(cargo) test -p auraescript -- --include-ignored

.PHONY: auraescript-debug
auraescript-debug:
	@$(cargo) install --path ./auraescript --debug --force

.PHONY: auraescript-release
auraescript-release:
	@$(cargo) install --path ./auraescript --force

#------------------------------------------------------------------------------#

# aer Commands

.PHONY: aer
aer: proto aer-lint aer-debug ## Lint and install aer (for use during development)

.PHONY: aer-lint
aer-lint:
	@$(cargo) clippy -p aer --all-features -- -D clippy::all -D warnings

.PHONY: aer-test
aer-test:
	@$(cargo) test -p aer

.PHONY: aer-test-all
aer-test-all:
	@$(cargo) test -p aer -- --include-ignored

.PHONY: aer-debug
aer-debug:
	@$(cargo) install --path ./aer --debug --force

.PHONY: aer-release
aer-release:
	@$(cargo) install --path ./aer --force

#------------------------------------------------------------------------------#

# Commands for other crates

.PHONY: libs-lint
libs-lint:
	@$(cargo) clippy --all-features --workspace --exclude auraed --exclude auraescript --exclude aer  -- -D clippy::all -D warnings

.PHONY: libs-test
libs-test:
	@$(cargo) test --workspace --exclude auraed --exclude auraescript --exclude aer

.PHONY: libs-test-all
libs-test-all:
	@$(cargo) test --workspace --exclude auraed --exclude auraescript --exclude aer -- --include-ignored

#------------------------------------------------------------------------------#

# Commands for testing

.PHONY: test-workflow
test-workflow: ## Tests a github actions workflow locally using `act`
	@act -W ./.github/workflows/$(file)

#------------------------------------------------------------------------------#

# Documentation Commands

.PHONY: docs-lint
docs-lint: # Check the docs for typos
	@vale --no-wrap --glob='!docs/stdlib/v0/*' ./docs

.PHONY: docs-stdlib
## Generate the docs for the stdlib from the .proto files
ifeq (, $(wildcard /usr/local/bin/protoc-gen-doc))
docs-stdlib:
	$(error "No /usr/local/bin/protoc-gen-doc, install from https://github.com/pseudomuto/protoc-gen-doc")
else
docs-stdlib:
	protoc --plugin=/usr/local/bin/protoc-gen-doc -I api/v0/discovery -I api/v0/observe -I api/v0/cells --doc_out=docs/stdlib/v0 --doc_opt=markdown,index.md:Ignore* api/v0/*/*.proto --experimental_allow_proto3_optional
endif

.PHONY: docs-crates
docs-crates: musl ## Build the crate (documentation)
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

.PHONY: stage-release-artifacts
stage-release-artifacts: install ## Stage release artifacts (for CI use)
	@mkdir -p /tmp/release
	@cp /usr/local/cargo/bin/auraed /tmp/release/auraed-$(tag)-$(uname_m)-unknown-linux-musl
	@cp /usr/local/cargo/bin/auraescript /tmp/release/auraescript-$(tag)-$(uname_m)-unknown-linux-gnu

.PHONY: upload-release-artifacts
upload-release-artifacts: install ## Upload release artifacts to github (for CI use)
	gh release upload $(tag) /tmp/release/auraed-$(tag)-$(uname_m)-unknown-linux-musl
	gh release upload $(tag) /tmp/release/auraescript-$(tag)-$(uname_m)-unknown-linux-gnu

#------------------------------------------------------------------------------#

# Other Commands

.PHONY: tlsinfo
tlsinfo: ## Show TLS Info for /var/run/aurae*
	./hack/server-tls-info

.PHONY: fmt
fmt: headers ## Format the entire code base(s)
	@./hack/code-format

.PHONY: headers
headers: headers-write ## Fix headers. Run this if you want to clobber things.

.PHONY: headers-check
headers-check: ## Only check for problematic files.
	./hack/headers-check

.PHONY: headers-write
headers-write: ## Fix any problematic files blindly.
	./hack/headers-write

.PHONY: start
start:
	sudo -E $(HOME)/.cargo/bin/auraed

.PHONY: help
help:  ## Show help messages for make targets
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(firstword $(MAKEFILE_LIST)) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[32m%-30s\033[0m %s\n", $$1, $$2}'

.PHONY: check-deps
check-deps: musl ## Check if there are any unused dependencies in Cargo.toml
#	cargo +nightly udeps --target $(uname_m)-unknown-linux-musl --package auraed
#	cargo +nightly udeps --package auraescript
#	cargo +nightly udeps --package aurae-client
#	cargo +nightly udeps --package aer
