//! Tree views.
use crate::{
    cache,
    drawing::Paint,
    style::Style,
    widget::{
        grid,
        grid::{GridLayoutExt, GridTemplate},
        prelude::*,
        Clickable, DragController, Grid, Image, Null, Scaling,
    },
    Data, Length, UnitExt,
};
use kyute_common::imbl;
use std::{hash::Hash, sync::Arc};

/// Represents a set of selected table rows.
#[derive(Default, Clone, Data)]
pub struct TableSelection<Id> {
    set: imbl::HashSet<Id>,
}

impl<Id: Clone + Hash + Eq> TableSelection<Id> {
    pub fn contains(&self, id: &Id) -> bool {
        self.set.contains(id)
    }

    pub fn insert(&mut self, id: Id) {
        self.set.insert(id);
    }

    pub fn flip(&mut self, id: Id) {
        if self.set.insert(id.clone()).is_some() {
            self.set.remove(&id);
        }
    }
}

#[derive(Clone)]
pub struct TableRow<Id> {
    /// Uniquely identifies this row among others in the same table.
    id: Id,
    /// The widget to put in the main column.
    widget: Arc<WidgetPod>,
    /// The widgets to put in the other columns.
    cells: Vec<(usize, Arc<WidgetPod>)>,
    /// Whether the children of this row are expanded, if there is any.
    expanded: bool,
    /// Child rows
    children: Vec<TableRow<Id>>,
    expanded_changed: Signal<bool>,
}

impl<Id> TableRow<Id> {
    #[composable]
    pub fn new(id: Id, widget: impl Widget + 'static) -> TableRow<Id> {
        #[state]
        let mut expanded = false;
        Self::new_inner(id, widget, expanded).on_expanded_changed(|v| expanded = v)
    }

    #[composable]
    fn new_inner(id: Id, widget: impl Widget + 'static, expanded: bool) -> TableRow<Id> {
        TableRow {
            id,
            widget: Arc::new(WidgetPod::new(widget)),
            cells: vec![],
            expanded,
            children: vec![],
            expanded_changed: Signal::new(),
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

    /// Sets a cell.
    #[composable]
    pub fn add_cell(&mut self, column: usize, widget: impl Widget + 'static) {
        self.cells.push((column, Arc::new(WidgetPod::new(widget))));
    }

    /// Adds a child row node.
    pub fn add_row(&mut self, child: TableRow<Id>) {
        self.children.push(child);
    }
}

/// Column headers.
pub struct ColumnHeaders {
    widgets: Vec<Arc<WidgetPod>>,
}

impl ColumnHeaders {
    pub fn new() -> ColumnHeaders {
        ColumnHeaders { widgets: vec![] }
    }

    #[composable]
    pub fn add(mut self, widget: impl Widget + 'static) -> Self {
        self.widgets.push(Arc::new(WidgetPod::new(widget)));
        self
    }
}

/// Builder helper for a TableView widget.
pub struct TableViewParams<'a, Id> {
    /// Reference to the current table selection.
    ///
    /// If None, selection is disabled.
    pub selection: Option<&'a mut TableSelection<Id>>,

    /// Table grid template.
    pub template: GridTemplate,

    /// Column headers.
    ///
    /// If empty, the column header row is not shown.
    pub column_headers: Option<ColumnHeaders>,

    /// Index of the main column.
    pub main_column: usize,

    /// Root-level rows.
    pub rows: Vec<TableRow<Id>>,

    /// Length of the hierachical ident.
    pub row_indent: Length,

    /// Whether the columns are resizable by the user.
    pub resizeable_columns: bool,

    /// Whether the rows are reorderable by the user.
    ///
    /// If true, the table will issue `on_reorder` events.
    pub reorderable_rows: bool,

    /// Whether the columns are reorderable by the user.
    ///
    /// If true, the table will issue `on_column_reorder` events.
    pub reorderable_columns: bool,

    /// Background style.
    pub background: Paint,

    /// Alternate background style, used every other row.
    pub alternate_background: Paint,

    /// Width of the row separators.
    pub row_separator_width: Length,

    /// Width of the column separators.
    pub column_separator_width: Length,

    /// Style of the row separators.
    pub row_separator_background: Paint,

    /// Style of the column separators.
    pub column_separator_background: Paint,

    /// Style of selected items.
    pub selected_style: Style,
}

impl<'a, Id> Default for TableViewParams<'a, Id> {
    fn default() -> Self {
        TableViewParams {
            selection: None,
            column_headers: None,
            main_column: 0,
            rows: vec![],
            row_indent: Default::default(),
            resizeable_columns: false,
            reorderable_rows: false,
            reorderable_columns: false,
            background: Default::default(),
            alternate_background: Default::default(),
            row_separator_width: Default::default(),
            column_separator_width: Default::default(),
            row_separator_background: Default::default(),
            column_separator_background: Default::default(),
            selected_style: Default::default(),
            template: Default::default(),
        }
    }
}

pub struct TableView {
    grid: Grid,
}

impl TableView {
    /// Creates a new tree grid.
    #[composable]
    pub fn new<Id: Hash + Eq + Clone>(mut params: TableViewParams<Id>) -> TableView {
        // create the main grid
        // TODO fix the Arc<GridTemplate> mess
        let mut grid = Grid::new(Arc::new(params.template));
        let num_columns = grid.column_count();

        // row counter
        let mut i = 0;

        // insert column headers
        if let Some(headers) = params.column_headers {
            // insert headers
            for header in headers.widgets.into_iter() {
                grid.insert(header);
            }

            if params.resizeable_columns {
                // insert transparent draggable items between each column (over the column separator).
                // they are drag handles for resizing the columns.
                for i in 1..num_columns {
                    let resize_handle = DragController::new(
                        Null.fill().fix_width(4.dip()),
                        //.background(Paint::from(theme::palette::RED_800))
                    );
                    // positioning
                    //let resize_handle = LayoutWrapper::with_offset((-2.0, 0.0), resize_handle);
                    // FIXME arbitrary Z-order here should be documented
                    //grid.insert(resize_handle.grid_area((.., i..i)));
                }
            }

            i += 1;
        }

        // set backgrounds
        grid.set_row_background(params.background);
        grid.set_alternate_row_background(params.alternate_background);
        grid.set_row_gap_background(params.row_separator_background);
        grid.set_column_gap_background(params.column_separator_background);
        grid.set_row_gap(params.row_separator_width);
        grid.set_column_gap(params.column_separator_width);

        // insert elements in the grid
        {
            // size of the chevron icon
            // FIXME should be less than the indent size?
            let icon_size = 20.dip();

            // chevron icons
            let chevron_expanded = Image::from_uri("data/icons/chevron.png", Scaling::Contain)
                .min_width(icon_size)
                .min_height(icon_size);
            let chevron_collapsed = Image::from_uri("data/icons/chevron-collapsed.png", Scaling::Contain)
                .min_width(icon_size)
                .min_height(icon_size);

            // fill the visit stack with the initial rows
            let mut visit: Vec<_> = params.rows.into_iter().map(|row| (0usize, row)).rev().collect();

            // depth-order traversal of the row hierarchy
            while let Some((level, row)) = visit.pop() {
                // row selection highlight
                if let Some(selection) = params.selection.as_mut() {
                    if selection.contains(&row.id) {
                        // draw a filled rect with the selection style that spans the whole row

                        // .box_style(params.selected_style.clone())
                        grid.insert(Null.fill().grid_area((i, ..)));
                    }
                    // also add a clickable rect, and clicking it adds the row to the selection
                    /*grid.insert(
                        Null.clickable()
                            .on_click(|| selection.flip(row.id.clone()))
                            .fill()
                            .grid_area((i, ..)),
                    );*/
                }

                // add the main widget (widget in the main column)
                if !row.children.is_empty() {
                    // there are children, so add the chevron
                    // FIXME: cache::scoped ugliness
                    let icon = cache::scoped(&row.id, || {
                        Clickable::new(if row.expanded {
                            chevron_expanded.clone()
                        } else {
                            chevron_collapsed.clone()
                        })
                        .on_click(|| {
                            row.expanded_changed.signal(!row.expanded);
                        })
                    });

                    let mut widget_with_chevron = Grid::row(grid::TrackBreadth::Auto);
                    widget_with_chevron.insert(icon);
                    widget_with_chevron.insert(row.widget);
                    grid.insert(
                        widget_with_chevron
                            .padding_left((level as f64) * params.row_indent)
                            .grid_area((i, params.main_column)),
                    );
                } else {
                    // no chevron
                    grid.insert(
                        // first padding is the space for the chevron, second is the indent
                        row.widget
                            .padding_left(icon_size)
                            .padding_left((level as f64) * params.row_indent)
                            .grid_area((i, params.main_column)),
                    );
                }

                // add row cells to the grid
                for (column, cell) in row.cells {
                    grid.insert(cell.grid_area((i, column)));
                }

                // visit child rows if the row is expanded
                if row.expanded {
                    visit.extend(row.children.into_iter().map(|n| (level + 1, n)).rev());
                }
                i += 1;
            }
        }

        TableView { grid }
    }
}

impl Widget for TableView {
    fn widget_id(&self) -> Option<WidgetId> {
        self.grid.widget_id()
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutConstraints, env: &Environment) -> Layout {
        self.grid.layout(ctx, constraints, env)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.grid.route_event(ctx, event, env);
        // handle
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        self.grid.paint(ctx)
    }
}
