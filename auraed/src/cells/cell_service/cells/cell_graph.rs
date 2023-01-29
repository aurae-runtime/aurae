use super::CellInfo;

#[derive(Clone)]
pub struct GraphNode {
    pub cell_info: Option<CellInfo>,
    pub children: Vec<GraphNode>,
}

impl GraphNode {
    pub fn with_cell_info(self, cell_info: &CellInfo) -> GraphNode {
        GraphNode {
            cell_info: Some(cell_info.clone()),
            children: self.children,
        }
    }

    pub fn with_children(self, children: Vec<GraphNode>) -> GraphNode {
        GraphNode { cell_info: self.cell_info.as_ref().cloned(), children }
    }
}
