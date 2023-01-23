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

default: all ## Build and install (debug) 🎉
all: install ## Build and install (debug) 🎉
install: build ## Build and install (debug) 🎉
build: musl auraed auraescript ## Build and install (debug) (+musl) 🎉
prcheck: build lint test-all ## Meant to mimic the GHA checks (includes ignored tests)

lint: ## Lint the code
	@$(cargo) clippy -p auraed --target $(uname_m)-unknown-linux-musl --all-features -- -D clippy::all -D warnings
	@$(cargo) clippy -p auraescript --all-features -- -D clippy::all -D warnings
	@$(cargo) clippy -p aer --all-features -- -D clippy::all -D warnings
	@$(cargo) clippy -p aurae-client --all-features -- -D clippy::all -D warnings

release: proto## Build (static+musl) and install (release) 🎉
	$(cargo) install --target $(uname_m)-unknown-linux-musl --path ./auraed
	$(cargo) install --path ./auraescript

.PHONY: auraescript
auraescript: proto ## Initialize and compile auraescript
	@$(cargo) clippy -p auraescript
	@$(cargo) install --path ./auraescript --debug --force

.PHONY: aer
aer: proto ## Initialize and compile aer
	@$(cargo) clippy -p aer
	@$(cargo) install --path ./aer --debug --force

musl: ## Add target for musl
	rustup target add $(uname_m)-unknown-linux-musl

.PHONY: auraed
auraed: proto ## Initialize and static-compile auraed with musl
	@$(cargo) clippy --target $(uname_m)-unknown-linux-musl -p auraed
	@$(cargo) install --target $(uname_m)-unknown-linux-musl --path ./auraed --debug --force

.PHONY: check-docs
check-docs: # spell checking
	@vale --no-wrap --glob='!docs/stdlib/v0/*' ./docs

.PHONY: docs
docs: proto crate stdlibdocs ## Assemble all the /docs for the website locally.
	cp -rv README.md docs/index.md # Special copy for the main README
	cp -rv api/README.md docs/stdlib/index.md # Special copy for the main README

## Generate the docs for the stdlib from the .proto files
ifeq (, $(wildcard /usr/local/bin/protoc-gen-doc))
stdlibdocs:
	$(error "No /usr/local/bin/protoc-gen-doc, install from https://github.com/pseudomuto/protoc-gen-doc")
else
stdlibdocs:
	protoc --plugin=/usr/local/bin/protoc-gen-doc -I api/v0/discovery -I api/v0/observe -I api/v0/runtime --doc_out=docs/stdlib/v0 --doc_opt=markdown,index.md:Ignore* api/v0/*/*.proto --experimental_allow_proto3_optional
endif

crate: ## Build the crate (documentation)
	$(cargo) doc --target $(uname_m)-unknown-linux-musl --no-deps --package auraed
	$(cargo) doc --no-deps --package auraescript
	$(cargo) doc --no-deps --package aurae-client
	$(cargo) doc --no-deps --package aer
	cp -rv target/doc/* docs/crate

serve: docs ## Run the aurae.io static website locally
	sudo -E ./hack/serve.sh

test: ## Run the tests
	@$(cargo) test --target $(uname_m)-unknown-linux-musl --workspace --exclude auraescript
	@$(cargo) test -p auraescript

test-all: ## Run the tests (including ignored)
	@sudo -E $(cargo) test --target $(uname_m)-unknown-linux-musl --workspace --exclude auraescript -- --include-ignored
	@$(cargo) test -p auraescript -- --include-ignored

.PHONY: config
config: ## Set up default config
	@mkdir -p $(HOME)/.aurae
	@cp -v auraescript/default.config.toml $(HOME)/.aurae/config
	@sed -i 's|~|$(HOME)|g' $(HOME)/.aurae/config
	@mkdir -p $(HOME)/.aurae/pki
	@cp -v pki/* $(HOME)/.aurae/pki

tlsinfo: ## Show TLS Info for /var/run/aurae*
	./hack/server-tls-info

.PHONY: pki
pki: certs ## Alias for certs
certs: clean-certs ## Generate x509 mTLS certs in /pki directory
	mkdir -p pki
	./hack/certgen
	sudo -E mkdir -p /etc/aurae/pki
	sudo -E cp -v pki/* /etc/aurae/pki
	@echo "Install PKI Auth Material [/etc/aurae]"

clean-certs: ## Clean the cert material
	@rm -rvf pki/*

fmt: headers ## Format the entire code base(s)
	@./hack/code-format

.PHONY: proto
proto: ## Generate code from protobuf schemas
	@buf --version >/dev/null 2>&1 || (echo "Warning: buf is not installed! Please install the 'buf' command line tool: https://docs.buf.build/installation"; exit 1)
	buf generate -v api

.PHONY: cri
cri: ## Copy the CRI interface from upstream
	curl https://raw.githubusercontent.com/kubernetes/cri-api/$(cri_version)/pkg/apis/runtime/v1/api.proto -o api/kubernetes/cri/v1/$(cri_version).proto

.PHONY: proto-lint
proto-lint: ## Lint protobuf schemas
	buf lint api

.PHONY: clean
clean: clean-certs
	@cargo clean

headers: headers-write ## Fix headers. Run this if you want to clobber things.

headers-check: ## Only check for problematic files.
	./hack/headers-check

headers-write: ## Fix any problematic files blindly.
	./hack/headers-write

.PHONY: spawn
spawn: ## Spawn the current auraed binary and start it in a container
	./hack/spawn

.PHONY: busybox
busybox: ## Creat a "busybox" OCI bundle in target
	./hack/oci-busybox

.PHONY: alpine
alpine: ## Creat an "alpine" OCI bundle in target
	./hack/oci-alpine

.PHONY: start
start:
	sudo -E $(HOME)/.cargo/bin/auraed

.PHONY: help
help:  ## Show help messages for make targets
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(firstword $(MAKEFILE_LIST)) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[32m%-30s\033[0m %s\n", $$1, $$2}'

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

.PHONY: check-deps
check-deps: ## Check if there are any unused dependencies in Cargo.toml
#	cargo +nightly udeps --target $(uname_m)-unknown-linux-musl --package auraed
#	cargo +nightly udeps --package auraescript
#	cargo +nightly udeps --package aurae-client
#	cargo +nightly udeps --package aer


.PHONY: test-workflow
test-workflow: ## Tests a github actions workflow locally using `act`
	@act -W ./.github/workflows/$(file)

.PHONY: stage-release-artifacts
stage-release-artifacts: ## Stage release artifacts
	@mkdir -p /tmp/release
	@cp /usr/local/cargo/bin/auraed /tmp/release/auraed-$(tag)-$(uname_m)-unknown-linux-musl
	@cp /usr/local/cargo/bin/auraescript /tmp/release/auraescript-$(tag)-$(uname_m)-unknown-linux-gnu

.PHONY: upload-release-artifacts
upload-release-artifacts: ## Upload release artifacts to github
	gh release upload $(tag) /tmp/release/auraed-$(tag)-$(uname_m)-unknown-linux-musl
	gh release upload $(tag) /tmp/release/auraescript-$(tag)-$(uname_m)-unknown-linux-gnu
