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
	@$(cargo) install --path ./auraescript --debug

.PHONY: auraed
auraed: ## Initialize and compile auraed
	@if [ ! -d auraed ]; then printf "\n\nError:\nun 'make submodule' to download auraed source before compiling.\n\n"; exit 1; fi
	@$(cargo) clippy -p auraed
	@$(cargo) install --path ./auraed --debug

.PHONY: website
website: ## Assemble all the /docs for the website locally.
	@if [ ! -d auraed ]; then printf "\n\nError:\nun 'make submodule' to download auraed source before compiling.\n\n"; exit 1; fi
	cp -rv README.md website/docs/index.md # Special copy for the main README
	cp -rv docs/* website/docs

	# Auraed
	cp -rv auraed/docs/* website/docs/auraed
	cp -rv auraed/README.md website/docs/auraed/index.md # Special copy for the main README

	# AuraeScript
	cp -rv auraescript/docs/* website/docs/auraescript
	cp -rv auraescript/README.md website/docs/auraescript/index.md # Special copy for the main README

	# Standard Library
	cp -rv auraed/stdlib/v0/docs/* website/docs/stdlib/v0
	cp -rv auraed/stdlib/v0/README.md website/docs/stdlib/v0/index.md
	cp -rv auraed/stdlib/README.md website/docs/stdlib/index.md # Special copy for the main README


.PHONY: docs
docs: ## Run the documentation site locally on port 8000.
	cd website && make serve

test: ## Run the tests
	@$(cargo) test

push: ## (git) Push branch="NAME"
	cd auraescript && git push origin $(branch)
	cd auraed && git push origin $(branch)
	cd website && git push origin $(branch)
	cd community && git push origin $(branch)
	git push origin $(branch)

add: ## (git) Add . (dangerous)
	cd auraescript && git add .
	cd auraed && git add .
	cd website && git add .
	cd community && git add .
	git add .

commit: ## (git) Commit message="MESSAGE"
	cd auraescript && git commit -s -m "$(message)" || true
	cd auraed && git commit -s -m "$(message)" || true
	cd website && git commit -s -m "$(message)" || true
	cd community && git commit -s -m "$(message)" || true
	git add .
	git commit -s -m "$(message)" || true

checkout: ## (git) Checkout branch="NAME"
	git checkout $(branch) || git checkout -b $(branch)
	cd auraescript && git checkout $(branch) || git checkout -b $(branch)
	cd auraed && git checkout $(branch) || git checkout -b $(branch)
	cd website && git checkout $(branch) || git checkout -b $(branch)
	cd website && git checkout $(branch) || git checkout -b $(branch)

status: ## (git) Status
	git status
	cd auraescript && git status
	cd auraed && git status
	cd website && git status
	cd community && git status

pull: ## (git) Pull branch="NAME"
	git pull origin $(branch)
	cd auraescript && git pull origin $(branch)
	cd auraed && git pull origin $(branch)
	cd website && git pull origin $(branch)
	cd community && git pull origin $(branch)

submodules: submodule ## Alias for submodule
submodule: ## Initialize all submodules
	@echo "Initializing submodules"
	@echo ""
	@read -p "Warning: This will destroy all work in subdirectories! Press any key to continue." FOO

	# AuraeScript
	@if [ -d /tmp/auraescript ]; then rm -rvf /tmp/auraescript; fi
	@if [ -d auraescript ]; then mv -v auraescript /tmp/auraescript; fi

	# Auraed
	@if [ -d /tmp/auraed ]; then rm -rvf /tmp/auraed; fi
	@if [ -d auraed ]; then mv -v auraed /tmp/auraed; fi

	# Website
	@if [ -d /tmp/website ]; then rm -rvf /tmp/website; fi
	@if [ -d website ]; then mv -v website /tmp/website; fi

	# Community
	@if [ -d /tmp/community ]; then rm -rvf /tmp/community; fi
	@if [ -d community ]; then mv -v community /tmp/community; fi

	# Init and update
	@git submodule update --init --recursive
	@git submodule update --remote --rebase

	# Attach to main
	cd auraescript && git checkout $(branch) && git branch && git pull origin $(branch)
	cd auraed && git checkout $(branch) && git branch && git pull origin $(branch)
	cd website && git checkout $(branch) && git branch && git pull origin $(branch)
	cd community && git checkout $(branch) && git branch && git pull origin $(branch)

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

.PHONY: clean
clean: clean-certs
	cd aurae && make clean
	#cd auraectl && make clean
	cd auraed && make clean
	@rm -rvf target/*

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

