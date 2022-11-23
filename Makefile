# ---------------------------------------------------------------------------- #
#             Apache 2.0 License Copyright © 2022 The Aurae Authors            #
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

# Configuration Options
export GIT_PAGER = cat

default: compile install
all: compile install

compile: auraescript auraed ## Compile for the local architecture ⚙

prcheck: build lint test

build:
	@$(cargo) build

lint:
	@$(cargo) clippy --all-features --workspace -- -D clippy::all -D warnings

install: ## Build and install (debug) 🎉
	@$(cargo) install --path ./auraescript --debug
	@$(cargo) install --path ./auraed --debug

release: ## Build and install (release) 🎉
	@$(cargo) install --path ./auraescript
	@$(cargo) install --path ./auraed

.PHONY: auraescript
auraescript: ## Initialize and compile aurae
	@if [ ! -d auraescript ]; then printf "\n\nError: Missing submodules. Run 'make submodule' to download aurae source before compiling.\n\n"; exit 1; fi
	@$(cargo) clippy -p auraescript
	@$(cargo) install --path ./auraescript --debug --force

.PHONY: auraed
auraed: ## Initialize and compile auraed
	@if [ ! -d auraed ]; then printf "\n\nError:\nun 'make submodule' to download auraed source before compiling.\n\n"; exit 1; fi
	@$(cargo) clippy -p auraed
	@$(cargo) install --path ./auraed --debug --force

.PHONY: docs
docs: crate stdlibdocs ## Assemble all the /docs for the website locally.
	cp -rv README.md docs/index.md # Special copy for the main README
	cp -rv api/README.md docs/stdlib/index.md # Special copy for the main README

## Generate the docs for the stdlib from the .proto files
ifeq (, $(wildcard /usr/local/bin/protoc-gen-doc))
stdlibdocs:
	$(error "No /usr/local/bin/protoc-gen-doc, install from https://github.com/pseudomuto/protoc-gen-doc")
else
stdlibdocs:
	protoc --plugin=/usr/local/bin/protoc-gen-doc -I api/v0 --doc_out=docs/stdlib/v0 --doc_opt=markdown,index.md:Ignore* api/v0/*.proto
endif

crate: ## Build the crate (documentation)
	$(cargo) doc --no-deps
	cp -rv target/doc/* docs/crate

serve: docs ## Run the aurae.io static website locally
	sudo -E ./hack/serve.sh

test: ## Run the tests
	@$(cargo) test

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
	./hack/certgen
	sudo -E mkdir -p /etc/aurae/pki
	sudo -E cp -v pki/* /etc/aurae/pki
	@echo "Install PKI Auth Material [/etc/aurae]"

clean-certs: ## Clean the cert material
	@rm -rvf pki/*

fmt: headers ## Format the entire code base(s)
	@./hack/code-format

clean-auraescript:
	cd auraescript && make clean 

clean-auraed:
	cd auraed && make clean

.PHONY: proto
proto: ## Generate code from protobuf schemas
	buf generate -v api

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

.PHONY: start
start:
	sudo $(HOME)/.cargo/bin/auraed

.PHONY: help
help:  ## Show help messages for make targets
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(firstword $(MAKEFILE_LIST)) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[32m%-30s\033[0m %s\n", $$1, $$2}'

