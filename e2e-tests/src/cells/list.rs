#[cfg(test)]
mod test {
    use crate::common::request_builders::CellServiceAllocateRequestBuilder;
    use client::{cells::cell_service::CellServiceClient, Client};
    use pretty_assertions::assert_eq;
    use proto::cells::{
        Cell, CellGraphNode, CellServiceListRequest, CellServiceListResponse,
    };

    #[tokio::test]
    #[ignore = "this test requires a fresh Aurae daemon which right now we cannot guarantee"]
    async fn must_list_allocated_cells_recursively() {
        let client = Client::default().await;
        let client = client.expect("failed to initialize aurae-client");

        // Allocate a cell
        let cell1_name = client
            .allocate(CellServiceAllocateRequestBuilder::new().build())
            .await
            .unwrap()
            .into_inner()
            .cell_name;

        // Allocate a nested cell
        let nested_cell_name = client
            .allocate(
                CellServiceAllocateRequestBuilder::new()
                    .parent_cell_name(cell1_name.clone())
                    .build(),
            )
            .await
            .unwrap()
            .into_inner()
            .cell_name;

        // Allocate another non-nested cell
        let cell2_name = client
            .allocate(CellServiceAllocateRequestBuilder::new().build())
            .await
            .unwrap()
            .into_inner()
            .cell_name;

        // List all cells
        let list_response =
            client.list(CellServiceListRequest {}).await.unwrap().into_inner();

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
        assert_eq!(list_response, expected);
    }
}
