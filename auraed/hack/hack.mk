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

SHELL := /bin/bash

empty:

build-container:
	cd hack/build-container && ./mk-build-container
	mkdir -p target
	touch hack/build-container

container-release:
	docker run -it --rm \
	-v "$(shell pwd):/aurae" \
	aurae-test-kernel bash -c "cd /aurae/auraescript && make release && cd /aurae && make pki && cd /aurae/auraed && make release"

kernel:
	mkdir -p target/rootfs/boot
	docker run -it --rm -u $${UID} -v "$(shell pwd):/aurae" aurae-kernel-builder bash -c "cd hack/kernel && ./mk-kernel"

menuconfig:
	docker run -it --rm -u $${UID} -v "$(shell pwd):/aurae" aurae-kernel-builder bash -c "cd hack/kernel && ./mk-menuconfig"

virsh-init:
	./hack/libvirt/init.sh

virsh-start: virsh-init
	virsh --connect qemu:///system create target/libvirt.xml

virsh-stop:
	virsh --connect qemu:///system destroy aurae

virsh-console:
	virsh --connect qemu:///system console aurae

virsh-shutdown:
	virsh --connect qemu:///system shutdown aurae --mode acpi

network:
	sudo brctl addbr vm-br0
	sudo ip link set up dev vm-br0
	sudo ip addr add fe80::1/64 dev vm-br0
	sudo ip addr add 169.254.42.1/24 dev vm-br0