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

all: install

executable   ?=  aurae
cargo         =  cargo
apibranch     =  main

.PHONY: api
api: ## Download the api to the local directory [v1]
	@if [ ! -d api/.repo ]; then git clone https://github.com/aurae-runtime/api.git api/.repo; fi
	cd api/.repo && git checkout $(apibranch) && git pull origin $(apibranch)
	@cd api/.repo
	cp -rv api/.repo/v* api # Move all versions [v*] up

cleanapi: ## Download the api to the local directory [v1]
	@rm -rvf api/.repo
	@rm -rvf api/*

compile: ## Compile for the local architecture ⚙
	@$(cargo) build

install: ## Build and install (debug) 🎉
	@echo "Installing..."
	@$(cargo) install --debug --path .

release: ## Build and install (release) 🎉
	@echo "Installing..."
	@$(cargo) install --path .

clean: cleanapi ## Clean your artifacts 🧼
	@echo "Cleaning..."
	@cargo clean
	@rm -rvf target/*
	@rm -rvf $(executable)

.PHONY: help
help:  ## Show help messages for make targets
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(firstword $(MAKEFILE_LIST)) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[32m%-30s\033[0m %s\n", $$1, $$2}'
