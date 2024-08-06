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
#!/bin/bash

# -------------------------------------------------------------------------- #
#         Apache 2.0 License Copyright © 2022-2023 The Aurae Authors         #
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

copy_lib() {
    cp -Lr $1 `echo $1 | awk '{print substr($1,2); }'`
}

install_libs() {
    libs=$(ldd $1 \
        | grep so \
        | sed -e '/^[^	]/ d' \
        | sed -e 's/	//' \
        | sed -e 's/.*=..//' \
        | sed -e 's/ (0.*)//' \
        | sort \
        | uniq -c \
        | sort -n \
        | awk '{$1=$1;print}' \
        | cut -d' ' -f2 \
        | grep "^/")
    
    for l in ${libs[@]}; do
        copy_lib $l
    done
}
