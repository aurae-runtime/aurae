use client::cells::cell_service::CellServiceClient;
use iter_tools::Itertools;
use proto::cells::{Cell, CellServiceAllocateRequest, CellServiceListRequest};
use test_helpers::*;

mod common;

#[test_helpers_macros::shared_runtime_test]
async fn list() {
    skip_if_not_root!("test_list");
    skip_if_seccomp!("test_list");

    let client = common::auraed_client().await;

    let parent_cell_name = format!("ae-test-{}", uuid::Uuid::new_v4());
    assert!(retry!(
        CellServiceClient::allocate(
            &client,
            allocate_request(&parent_cell_name)
        )
        .await
    )
    .is_ok());

    let nested_cell_name =
        format!("{}/ae-test-{}", &parent_cell_name, uuid::Uuid::new_v4());
    assert!(retry!(
        CellServiceClient::allocate(
            &client,
            allocate_request(&nested_cell_name)
        )
        .await
    )
    .is_ok());

    let cell_without_children_name =
        format!("ae-test-{}", uuid::Uuid::new_v4());
    assert!(retry!(
        CellServiceClient::allocate(
            &client,
            allocate_request(&cell_without_children_name)
        )
        .await
    )
    .is_ok());

    let result = retry!(
        CellServiceClient::list(&client, CellServiceListRequest {}).await
    );
    assert!(result.is_ok());

    let list = result.unwrap().into_inner();
    assert_eq!(list.cells.len(), 2);

    let mut expected_root_cell_names =
        vec![&parent_cell_name, &cell_without_children_name];
    expected_root_cell_names.sort();

    let mut actual_root_cell_names = list
        .cells
        .iter()
        .map(|c| c.cell.as_ref().unwrap().name.as_str())
        .collect_vec();
    actual_root_cell_names.sort();
    assert_eq!(actual_root_cell_names, expected_root_cell_names);

    let parent_cell = list
        .cells
        .iter()
        .find(|p| p.cell.as_ref().unwrap().name.eq(&parent_cell_name));
    assert!(parent_cell.is_some());

    let expected_nested_cell_names = vec![&nested_cell_name];
    let actual_nested_cell_names = parent_cell
        .unwrap()
        .children
        .iter()
        .map(|c| c.cell.as_ref().unwrap().name.as_str())
        .collect_vec();
    assert_eq!(actual_nested_cell_names, expected_nested_cell_names);
}

fn allocate_request(cell_name: &str) -> CellServiceAllocateRequest {
    CellServiceAllocateRequest {
        cell: Some(Cell {
            name: cell_name.to_string(),
            cpu: None,
            cpuset: None,
            memory: None,
            isolate_process: false,
            isolate_network: false,
        }),
    }
}
