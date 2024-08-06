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
use common::cells::CellServiceAllocateRequestBuilder;
use pretty_assertions::assert_eq;
use proto::cells::{
    Cell, CellGraphNode, CellServiceListRequest, CellServiceListResponse,
};
use test_helpers::*;

mod common;

#[test_helpers_macros::shared_runtime_test]
async fn cells_list_must_list_allocated_cells_recursively() {
    skip_if_not_root!("cells_list_must_list_allocated_cells_recursively");
    skip_if_seccomp!("cells_list_must_list_allocated_cells_recursively");

    let client = common::auraed_client().await;

    // Allocate a cell
    let cell1_name = retry!(
        client.allocate(CellServiceAllocateRequestBuilder::new().build()).await
    )
    .unwrap()
    .into_inner()
    .cell_name;

    // Allocate a nested cell
    let nested_cell_name = retry!(
        client
            .allocate(
                CellServiceAllocateRequestBuilder::new()
                    .parent_cell_name(cell1_name.clone())
                    .build(),
            )
            .await
    )
    .unwrap()
    .into_inner()
    .cell_name;

    // Allocate a double nested cell
    let double_nested_cell_name = retry!(
        client
            .allocate(
                CellServiceAllocateRequestBuilder::new()
                    .parent_cell_name(nested_cell_name.clone())
                    .build(),
            )
            .await
    )
    .unwrap()
    .into_inner()
    .cell_name;

    // Allocate another non-nested cell
    let cell2_name = retry!(
        client.allocate(CellServiceAllocateRequestBuilder::new().build()).await
    )
    .unwrap()
    .into_inner()
    .cell_name;

    // List all cells
    let list_response = retry!(client.list(CellServiceListRequest {}).await)
        .unwrap()
        .into_inner();

    // The expected response
    let mut expected = CellServiceListResponse {
        cells: vec![
            CellGraphNode {
                cell: Some(Cell {
                    name: cell2_name,
                    cpu: None,
                    cpuset: None,
                    memory: None,
                    isolate_process: false,
                    isolate_network: false,
                }),
                children: vec![],
            },
            CellGraphNode {
                cell: Some(Cell {
                    name: cell1_name,
                    cpu: None,
                    cpuset: None,
                    memory: None,
                    isolate_process: false,
                    isolate_network: false,
                }),
                children: vec![CellGraphNode {
                    cell: Some(Cell {
                        name: nested_cell_name,
                        cpu: None,
                        cpuset: None,
                        memory: None,
                        isolate_process: false,
                        isolate_network: false,
                    }),
                    children: vec![CellGraphNode {
                        cell: Some(Cell {
                            name: double_nested_cell_name,
                            cpu: None,
                            cpuset: None,
                            memory: None,
                            isolate_process: false,
                            isolate_network: false,
                        }),
                        children: vec![],
                    }],
                }],
            },
        ],
    };

    // Assert that the actual response matches the expected
    if !list_response.eq(&expected) {
        // HACK: since we only have 2 children, and they may come in any order, swap if needed.
        expected.cells.swap(0, 1);
        assert_eq!(list_response, expected);
    }
}