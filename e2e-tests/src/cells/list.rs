#[cfg(test)]
mod test {
    use client::Client;
    use proto::cells::{Cell, CellGraphNode, CellServiceListResponse};

    use crate::common::{
        helpers::{allocate_cell, list_cells},
        request_builders::CellServiceAllocateRequestBuilder,
    };

    #[tokio::test]
    #[ignore = "this test requires a fresh Aurae daemon which right now we cannot guarantee"]
    async fn must_list_allocated_cells_recursively() {
        let client = Client::default().await;
        let client = client.expect("failed to initialize aurae-client");

        // Allocate a cell
        let cell1_name = allocate_cell(
            &client,
            CellServiceAllocateRequestBuilder::new().build(),
        )
        .await;

        // Allocate a nested cell
        let nested_cell_name = allocate_cell(
            &client,
            CellServiceAllocateRequestBuilder::new()
                .parent_cell_name(cell1_name.clone())
                .build(),
        )
        .await;

        // Allocate another non-nested cell
        let cell2_name = allocate_cell(
            &client,
            CellServiceAllocateRequestBuilder::new().build(),
        )
        .await;

        // List all cells
        let list_response = list_cells(&client).await;

        // The expected response
        let expected = CellServiceListResponse {
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
                        children: vec![],
                    }],
                },
            ],
        };

        // Assert that the actual response matches the expected
        assert_eq!(
            list_response, expected,
            "got {:#?}\nexpected {:#?}",
            list_response, expected
        );
    }
}
