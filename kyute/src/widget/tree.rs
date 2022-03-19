//! Tree views.
use crate::{
    widget::{
        grid::GridTrackDefinition, prelude::*, Container, Grid, GridLength, GridSpan, Image, Text, WidgetWrapper,
    },
    SideOffsets, State,
};
use std::sync::Arc;

// implementation:
// - it's not directly a widget, but rather a function that fills a column of an existing grid

#[derive(Clone)]
pub struct TreeGrid {
    grid: Grid,
}

impl TreeGrid {
    pub fn new(columns: impl IntoIterator<Item = GridTrackDefinition>) -> TreeGrid {
        TreeGrid {
            grid: Grid::with_column_definitions(columns),
        }
    }

    /// Sets the root node.
    pub fn set_root(&mut self, tree_node: TreeNode) {
        let mut visit = vec![];
        visit.push((0, tree_node));

        let mut i = 0;
        while let Some((level, node)) = visit.pop() {
            if !node.children.is_empty() {
                let icon = Image::from_uri("data/icons/chevron.png");
                let mut grid = Grid::column(GridLength::Auto);
                grid.add_item(0, 0, icon);
                grid.add_item(0, 1, node.widget);
                self.grid.add_item(
                    i,
                    0,
                    Container::new(grid).content_padding(SideOffsets::new(0.0, 0.0, 0.0, level as f64 * 20.0)),
                );
            } else {
                self.grid.add_item(
                    i,
                    0,
                    Container::new(node.widget).content_padding(SideOffsets::new(
                        0.0,
                        0.0,
                        0.0,
                        20.0 + level as f64 * 20.0,
                    )),
                );
            }

            // add node in the grid
            for (columns, item) in node.row {
                self.grid.add_item(i, columns, item);
            }

            // add children to the visit stack
            visit.extend(node.children.into_iter().map(|n| (level + 1, n)).rev());
            i += 1;
        }
    }
}

impl WidgetWrapper for TreeGrid {
    type Inner = Grid;

    fn inner(&self) -> &Self::Inner {
        &self.grid
    }

    fn inner_mut(&mut self) -> &mut Self::Inner {
        &mut self.grid
    }
}

/// A node in a tree view widget.
#[derive(Clone)]
pub struct TreeNode<'a> {
    expanded: bool,
    widget: Arc<WidgetPod>,
    row: Vec<(GridSpan<'a>, Arc<WidgetPod>)>,
    children: Vec<TreeNode<'a>>,
    expanded_changed: Signal<bool>,
}

impl<'a> TreeNode<'a> {
    /// Returns a new tree node, initially collapsed.
    #[composable]
    pub fn new(contents: impl Widget + 'static) -> TreeNode<'a> {
        #[state]
        let mut expanded = false;
        Self::new_inner(expanded, contents).on_expanded_changed(|v| expanded = v)
    }

    #[composable]
    fn new_inner(expanded: bool, contents: impl Widget + 'static) -> TreeNode<'a> {
        TreeNode {
            row: vec![],
            children: vec![],
            expanded,
            expanded_changed: Signal::new(),
            widget: Arc::new(WidgetPod::new(contents)),
        }
    }

    pub fn on_expanded_changed(self, f: impl FnOnce(bool)) -> Self {
        self.expanded_changed.map(f);
        self
    }

    /// Whether the node is expanded, and the child nodes are visible.
    pub fn expanded(&self) -> bool {
        self.expanded
    }

    /// Adds a child node.
    pub fn add_child(&mut self, child: TreeNode<'a>) {
        self.children.push(child);
    }
}
