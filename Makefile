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

all: compile

executable   ?=  aurae

compile: ## Compile for the local architecture ⚙
	@cargo build

install: ## Install the program to /bin 🎉
	@echo "Installing..."
	@cargo install --path .

#test: clean compile install ## 🤓 Run go tests
#	@echo "Testing..."
#	go test -v ./...

clean: ## Clean your artifacts 🧼
	@echo "Cleaning..."
	@cargo clean
	@rm -rvf target/*
	@rm -rvf $(executable)

#.PHONY: release
#release: ## Make the binaries for headers-check GitHub release 📦
#	mkdir -p release
#	GOOS="linux" GOARCH="amd64" go build -ldflags "-X 'github.com/$(org)/$(target).Version=$(version)'" -o release/$(target)-linux-amd64 bin/*.go
#	GOOS="linux" GOARCH="arm" go build -ldflags "-X 'github.com/$(org)/$(target).Version=$(version)'" -o release/$(target)-linux-arm bin/*.go
#	GOOS="linux" GOARCH="arm64" go build -ldflags "-X 'github.com/$(org)/$(target).Version=$(version)'" -o release/$(target)-linux-arm64 bin/*.go
#	GOOS="linux" GOARCH="386" go build -ldflags "-X 'github.com/$(org)/$(target).Version=$(version)'" -o release/$(target)-linux-386 bin/*.go
#	GOOS="darwin" GOARCH="amd64" go build -ldflags "-X 'github.com/$(org)/$(target).Version=$(version)'" -o release/$(target)-darwin-amd64 bin/*.go

.PHONY: help
help:  ## 🤔 Show help messages for make targets
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "[32m%-30s[0m %s", $$1, $$2}'
