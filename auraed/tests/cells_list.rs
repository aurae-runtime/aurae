use client::cells::cell_service::CellServiceClient;
use common::cells::CellServiceAllocateRequestBuilder;
use iter_tools::Itertools;
use proto::cells::{CellGraphNode, CellServiceListRequest};
use test_helpers::*;

mod common;

#[test_helpers_macros::shared_runtime_test]
async fn cells_list_must_list_allocated_cells_recursively() {
    skip_if_not_root!("test_list");
    skip_if_seccomp!("test_list");

    let client = common::auraed_client().await;

    // allocate the grandparent cell
    let req = CellServiceAllocateRequestBuilder::new().build();
    let grandparent_cell_name = req.cell.as_ref().unwrap().name.clone();
    assert!(retry!(client.allocate(req.clone()).await).is_ok());

    // allocate the parent cell
    let req = CellServiceAllocateRequestBuilder::new()
        .parent_cell_name(grandparent_cell_name.clone())
        .build();
    let parent_cell_name = req.cell.as_ref().unwrap().name.clone();
    assert!(retry!(client.allocate(req.clone()).await).is_ok());

    // allocate the child cell
    let req = CellServiceAllocateRequestBuilder::new()
        .parent_cell_name(parent_cell_name.clone())
        .build();
    let child_cell_name = req.cell.as_ref().unwrap().name.clone();
    assert!(retry!(client.allocate(req.clone()).await).is_ok());

    // allocate a root cell that won't have children
    let req = CellServiceAllocateRequestBuilder::new().build();
    let childless_cell_name = req.cell.as_ref().unwrap().name.clone();
    assert!(retry!(client.allocate(req.clone()).await).is_ok());

    let result = retry!(client.list(CellServiceListRequest {}).await);
    assert!(result.is_ok());

    // we should have 2 root cells (grandparent and childless)
    let list = result.unwrap().into_inner();
    assert_eq!(list.cells.len(), 2);
    assert!(children_names_are_eq(
        &list.cells,
        &mut [&grandparent_cell_name, &childless_cell_name],
    ));

    // get the grandparent node to check its children
    let grandparent_cell_node = list
        .cells
        .iter()
        .find(|p| p.cell.as_ref().unwrap().name.eq(&grandparent_cell_name))
        .unwrap();

    assert!(children_names_are_eq(
        &grandparent_cell_node.children,
        &mut [&parent_cell_name],
    ));

    // get the parent node to check its children
    let parent_cell_node = grandparent_cell_node
        .children
        .iter()
        .find(|p| p.cell.as_ref().unwrap().name.eq(&parent_cell_name))
        .unwrap();

    assert!(children_names_are_eq(
        &parent_cell_node.children,
        &mut [&child_cell_name],
    ));

    // child shouldn't have children
    assert_eq!(
        parent_cell_node
            .children
            .iter()
            .find(|c| c.cell.as_ref().unwrap().name.eq(&child_cell_name))
            .unwrap()
            .children
            .len(),
        0
    );

    // childless shouldn't have children
    assert_eq!(
        list.cells
            .iter()
            .find(|c| c.cell.as_ref().unwrap().name.eq(&childless_cell_name))
            .unwrap()
            .children
            .len(),
        0
    );
}

#[must_use]
fn children_names_are_eq(
    list: &[CellGraphNode],
    expected_names: &mut [&str],
) -> bool {
    expected_names.sort();

    let mut actual_names = list
        .iter()
        .map(|c| c.cell.as_ref().unwrap().name.as_str())
        .collect_vec();
    actual_names.sort();

    actual_names.eq(expected_names)
}
