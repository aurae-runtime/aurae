#!/bin/bash

# -------------------------------------------------------------------------- #
#             Apache 2.0 License Copyright © 2022 The Aurae Authors          #
#                                                                            #
#                +--------------------------------------------+              #
#                |   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ |              #
#                |  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ |               #
#                |  ███████║██║   ██║██████╔╝███████║█████╗   |              #
#                |  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   |              #
#                |  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ |              #
#                |  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ |              #
#                +--------------------------------------------+              #
#                                                                            #
#                         Distributed Systems Runtime                        #
#                                                                            #
# -------------------------------------------------------------------------- #
#                                                                            #
#   Licensed under the Apache License, Version 2.0 (the "License");          #
#   you may not use this file except in compliance with the License.          #
#   You may obtain a copy of the License at                                  #
#                                                                            #
#       http://www.apache.org/licenses/LICENSE-2.0                           #
#                                                                            #
#   Unless required by applicable law or agreed to in writing, software      #
#   distributed under the License is distributed on an "AS IS" BASIS,        #
#   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. #
#   See the License for the specific language governing permissions and      #
#   limitations under the License.                                           #
#                                                                            #
# -------------------------------------------------------------------------- #

export KERNEL_VERSION=5.15.68
arch=$(arch)
case $arch in
x86)
    CONFIG_ARCH=x86
    ;;
x86_64)
    CONFIG_ARCH=x86
    ;;
aarch64)
    CONFIG_ARCH=arm
    ;;
arm64)
    CONFIG_ARCH=arm
    ;;
esac

export KERNEL_CONFIG="aurae-linux-${CONFIG_ARCH}-${KERNEL_VERSION}.config"
