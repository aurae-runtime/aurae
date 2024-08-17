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

use client::cells::cell_service::CellServiceClient;
use client::discovery::discovery_service::DiscoveryServiceClient;
use client::vms::vm_service::VmServiceClient;
use client::{Client, ClientError};
use common::cells::{
    CellServiceAllocateRequestBuilder, CellServiceStartRequestBuilder,
};
use common::remote_auraed_client;
use log::info;
use proto::cells::{CellServiceFreeRequest, CellServiceStopRequest};
use proto::discovery::DiscoverRequest;
use proto::vms::{
    RootDrive, VirtualMachine, VmServiceAllocateRequest, VmServiceListRequest,
    VmServiceStartRequest,
};

mod common;

#[test_helpers_macros::shared_runtime_test]
#[ignore]
async fn vms_with_auraed() {
    let vm_id = format!("ae-test-vm-{}", uuid::Uuid::new_v4());
    let client = common::auraed_client().await;
    let res = retry!(
        client
            .allocate(VmServiceAllocateRequest {
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
                        image_path: "/var/lib/aurae/vm/image/disk.raw".into(),
                        read_only: false,
                    }),
                    drive_mounts: vec![],
                    auraed_address: String::new()
                }),
            })
            .await
    );

    assert!(res.is_ok(), "{:?}", res);

    let vm = VmServiceClient::start(
        &client,
        VmServiceStartRequest { vm_id: vm_id.clone() },
    )
    .await
    .expect("failed to start vm")
    .into_inner();

    // Try for 5 seconds to get a client to the VM
    let mut remote_client: Result<Client, ClientError> =
        remote_auraed_client(vm.auraed_address).await;
    for _ in 0..5 {
        if remote_client.is_ok() {
            break;
        }
        let vm = VmServiceClient::list(&client, VmServiceListRequest {})
            .await
            .expect("failed to list vms")
            .into_inner()
            .machines
            .into_iter()
            .find(|m| m.id == vm_id)
            .expect("vm not found");
        if vm.auraed_address.is_empty() || vm.status != "Running" {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            continue;
        }
        remote_client = remote_auraed_client(vm.auraed_address).await;
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    // NOTE: for now this passes when cloud-hypervisor is running a VM with auraed
    // as PID 1 with a tuntap device at the provided scope_id
    let remote_client = remote_client.expect("could not build remote client");

    let res = remote_client.discover(DiscoverRequest {}).await;
    assert!(res.is_ok(), "{:?}", res);

    let res = res.expect("this shouldn't happen").into_inner();
    assert!(res.healthy);

    let res = remote_client
        .allocate(CellServiceAllocateRequestBuilder::new().build())
        .await;
    assert!(res.is_ok(), "{:?}", res);
    let cell_name = res.unwrap().into_inner().cell_name;
    info!("Allocated cell: {}", cell_name);

    let res = CellServiceClient::start(
        &remote_client,
        CellServiceStartRequestBuilder::new()
            .cell_name(cell_name.clone())
            .executable_name("sleeper".into())
            .build(),
    )
    .await;
    assert!(res.is_ok(), "{:?}", res);

    CellServiceClient::stop(
        &remote_client,
        CellServiceStopRequest {
            cell_name: Some(cell_name.clone()),
            executable_name: "sleeper".into(),
        },
    )
    .await
    .expect("failed to stop cell");

    CellServiceClient::free(
        &remote_client,
        CellServiceFreeRequest { cell_name },
    )
    .await
    .expect("failed to free cell");
}
