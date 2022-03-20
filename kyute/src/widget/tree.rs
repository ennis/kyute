//! Tree views.
use crate::{
    widget::{
        grid::GridTrackDefinition, prelude::*, Clickable, Container, Grid, GridLength, GridSpan, Image, Scaling,
        WidgetWrapper,
    },
    UnitExt, WidgetExt,
};
use std::sync::Arc;

// implementation:
// - it's not directly a widget, but rather a function that fills a column of an existing grid

#[derive(Clone)]
pub struct TreeGrid {
    grid: Grid,
}

impl TreeGrid {
    /// Creates a new tree grid.
    pub fn new(row_height: impl Into<GridLength>, columns: impl IntoIterator<Item = GridTrackDefinition>) -> TreeGrid {
        TreeGrid {
            grid: Grid::with_column_definitions(columns).row_template(row_height.into()),
        }
    }

    /// Sets the root node.
    #[composable]
    pub fn set_root(&mut self, tree_node: TreeNode) {
        // visit stack
        let mut visit = vec![(0, tree_node)];

        let mut i = 0;
        while let Some((level, node)) = visit.pop() {
            if !node.children.is_empty() {
                let icon = Clickable::new(
                    Image::from_uri(
                        if node.expanded {
                            "data/icons/chevron.png"
                        } else {
                            "data/icons/chevron-collapsed.png"
                        },
                        Scaling::Contain,
                    )
                    .fix_size(20.dip(), 20.dip()),
                )
                .on_click(|| {
                    node.expanded_changed.signal(!node.expanded);
                });

                let mut grid = Grid::column(GridLength::Auto);
                grid.add_item(0, 0, icon);
                grid.add_item(0, 1, node.widget);
                self.grid.add_item(
                    i,
                    0,
                    Container::new(grid).content_padding(0.dip(), 0.dip(), 0.dip(), (level as f64 * 20.0).dip()),
                );
            } else {
                self.grid.add_item(
                    i,
                    0,
                    Container::new(node.widget).content_padding(
                        0.dip(),
                        0.dip(),
                        0.dip(),
                        (20.0 + level as f64 * 20.0).dip(),
                    ),
                );
            }

            // add node in the grid
            for (columns, item) in node.row {
                self.grid.add_item(i, columns, item);
            }

            // add children to the visit stack, if expanded
            if node.expanded {
                visit.extend(node.children.into_iter().map(|n| (level + 1, n)).rev());
            }
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

    /// Adds a column.
    pub fn add_column(&mut self, column_span: impl Into<GridSpan<'a>>, widget: impl Widget + 'static) {
        self.row.push((column_span.into(), Arc::new(WidgetPod::new(widget))));
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
