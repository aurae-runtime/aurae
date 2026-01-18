/* -------------------------------------------------------------------------- *\
 *                |   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ |              *
 *                |  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ |              *
 *                |  ███████║██║   ██║██████╔╝███████║█████╗   |              *
 *                |  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   |              *
 *                |  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ |              *
 *                |  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ |              *
 *                +--------------------------------------------+              *
 *                                                                            *
 *                         Distributed Systems Runtime                        *
 * -------------------------------------------------------------------------- *
 * Copyright 2022 - 2024, the aurae contributors                              *
 * SPDX-License-Identifier: Apache-2.0                                        *
\* -------------------------------------------------------------------------- */

macros::subcommand!(
    "../api/v0/vms/vms.proto",
    vms,
    VmService,
    Allocate {
        machine_id[required = true],
        machine_mem_size_mb[long, alias = "mem-size-mb", default_value = "512"],
        machine_vcpu_count[long, alias = "vcpu-count", default_value = "1"],
        machine_kernel_img_path[required = true, long, alias = "kernel-img-path"],
        machine_kernel_args[long, alias = "kernel-args", default_value = ""],
        machine_root_drive_image_path[required = true, long, alias = "root-drive-path"],
        machine_root_drive_read_only[long, alias = "root-drive-readonly", default_value = "false"],
        machine_drive_mounts_image_path[long, alias = "drive-mount-path", default_value = ""],
        machine_drive_mounts_vm_path[long, alias = "drive-mount-vm-path", default_value = ""],
        machine_drive_mounts_fs_type[long, alias = "drive-mount-fs-type", default_value = ""],
        machine_drive_mounts_read_only[long, alias = "drive-mount-readonly", default_value = "false"],
        machine_auraed_address[long, alias = "auraed-address", default_value = ""],
    },
    Free {
        vm_id[required = true],
    },
    Start {
        vm_id[required = true],
    },
    Stop {
        vm_id[required = true],
    },
);
