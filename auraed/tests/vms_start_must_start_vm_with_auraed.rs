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

use std::thread;

use client::discovery::discovery_service::DiscoveryServiceClient;
use client::vms::vms_service::VmServiceClient;
use proto::discovery::DiscoverRequest;
use proto::vms::{
    RootDrive, VirtualMachine, VmServiceCreateRequest, VmServiceStartRequest,
};

mod common;

#[test_helpers_macros::shared_runtime_test]
async fn vms_must_start_vm_with_auraed() {
    let vm_id = format!("ae-test-vm-{}", uuid::Uuid::new_v4());
    let client = common::auraed_client().await;
    let res = retry!(
        client
            .create(VmServiceCreateRequest {
                machine: Some(VirtualMachine {
                    id: vm_id.clone(),
                    mem_size_mb: 1024,
                    vcpu_count: 2,
                    kernel_img_path: "/var/lib/aurae/vm/kernel/vmlinux.bin"
                        .to_string(),
                    kernel_args: vec![
                        "console=hvc0".to_string(),
                        "root=/dev/vda1".to_string(),
                        "rw".to_string(),
                    ],
                    root_drive: Some(RootDrive {
                        host_path: "/var/lib/aurae/vm/image/disk.raw".into(),
                        is_writeable: true,
                    }),
                    drive_mounts: vec![],
                }),
            })
            .await
    );

    assert!(res.is_ok(), "{:?}", res);

    let res = retry!(
        client.start(VmServiceStartRequest { vm_id: vm_id.clone() }).await
    );

    assert!(res.is_ok(), "{:?}", res);
    thread::sleep(std::time::Duration::from_secs(10));

    // NOTE: for now this passes when cloud-hypervisor is running a VM with auraed
    // as PID 1 with a tuntap device at the provided scope_id
    let scope_id = res.unwrap().into_inner().socket_scope_id;
    let remote_client =
        common::remote_auraed_client(format!("[fe80::2%{scope_id}]:8080"))
            .await;
    let res = remote_client.discover(DiscoverRequest {}).await;

    assert!(res.is_ok(), "{:?}", res);

    let res = res.expect("this shouldn't happen").into_inner();
    assert!(res.healthy);
}
