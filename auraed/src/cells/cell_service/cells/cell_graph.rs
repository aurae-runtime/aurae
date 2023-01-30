use super::{CellName, CellSpec};

#[derive(Clone, Debug)]
pub struct GraphNode {
    pub cell_info: Option<(CellName, CellSpec)>,
    pub children: Vec<GraphNode>,
}

impl GraphNode {
    pub fn with_cell_info(
        self,
        cell_name: CellName,
        cell_spec: CellSpec,
    ) -> GraphNode {
        GraphNode {
            cell_info: Some((cell_name, cell_spec)),
            children: self.children,
        }
    }

    pub fn with_children(self, children: Vec<GraphNode>) -> GraphNode {
        GraphNode { cell_info: self.cell_info, children }
    }
}
