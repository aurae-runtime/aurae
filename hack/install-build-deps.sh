#!/usr/bin/env bash
# ---------------------------------------------------------------------------- #
#                |   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ |                #
#                |  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ |                #
#                |  ███████║██║   ██║██████╔╝███████║█████╗   |                #
#                |  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   |                #
#                |  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ |                #
#                |  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ |                #
#                +--------------------------------------------+                #
#                                                                              #
#                         Distributed Systems Runtime                          #
# ---------------------------------------------------------------------------- #
# Copyright 2022 - 2024, the aurae contributors                                #
# SPDX-License-Identifier: Apache-2.0                                          #
# ---------------------------------------------------------------------------- #

# Do not add GNU libraries here! Do not add GNU libraries here!
#
# Please (seriously please) be careful about adding commands here.
# This is our core way of validating that our binary is "healthy"
# If we need to install anything with the word "lib" in it to get
# the build to pass, we likely should be having other discussions
# instead of adding commands here.
#
# Do not add GNU libraries here! Do not add GNU libraries here!
#
# For example, we should NOT be adding libraries such as "libseccomp"
# or "libdbus".
#
# If in doubt, please ask in Discord in the build channel.
#
# Do not at GNU libraries here! Do not add GNU libraries here!
sudo apt-get update &&
	sudo apt-get install -y musl-tools

# install buf 1.60.0
BUF_VERSION="1.60.0"
INSTALL_BUF=false
if ! hash buf 2>/dev/null; then
	INSTALL_BUF=true
else
	CURRENT_BUF_VERSION=$(buf --version | tr -d ' \t\n')
	if [ "$CURRENT_BUF_VERSION" != "$BUF_VERSION" ]; then
		INSTALL_BUF=true
	fi
fi

if [ "$INSTALL_BUF" = true ]; then
	BUILD_PREFIX=$(mktemp -d)
	PREFIX="/usr/local"
	URL_BASE=https://github.com/bufbuild/buf/releases/download
	DOWNLOAD_SLUG="v${BUF_VERSION}/buf-$(uname -s)-$(uname -m).tar.gz"
	pushd "$BUILD_PREFIX" &&
		curl -sSL "${URL_BASE}/${DOWNLOAD_SLUG}" |
		sudo tar -xvzf - -C "${PREFIX}" --strip-components 1 &&
		popd &&
		sudo rm -rf "$BUILD_PREFIX"
fi

if [ "$INSTALL_BUF" = true ]; then
	hash -r buf 2>/dev/null || true
fi

# install protobuf deps
if ! hash protoc-gen-doc; then
	BUILD_PREFIX=$(mktemp -d)
	URL_BASE=https://github.com/pseudomuto/protoc-gen-doc/releases/download
	DOWNLOAD_SLUG=v1.5.1/protoc-gen-doc_1.5.1_linux_amd64.tar.gz
	pushd "$BUILD_PREFIX" &&
		wget "${URL_BASE}/${DOWNLOAD_SLUG}" &&
		tar -xzf protoc-gen-doc_1.5.1_linux_amd64.tar.gz &&
		chmod +x protoc-gen-doc &&
		sudo mv protoc-gen-doc /usr/local/bin/protoc-gen-doc &&
		sudo apt-get update &&
		sudo apt-get install -y protobuf-compiler &&
		popd &&
		sudo rm -rf "$BUILD_PREFIX"
fi
