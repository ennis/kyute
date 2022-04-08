//! Tree views.
use crate::{
    cache,
    style::{BoxStyle, Paint},
    theme,
    widget::{
        grid::GridTrackDefinition, prelude::*, Clickable, Container, DragController, Grid, GridLength, GridSpan, Image,
        Null, Scaling, WidgetWrapper,
    },
    Data, Length, UnitExt, ValueRef, WidgetExt,
};
use kyute_common::imbl;
use std::{collections::HashSet, hash::Hash, sync::Arc};

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

    /// Table columns.
    pub columns: Vec<GridTrackDefinition>,

    /// Column headers.
    ///
    /// If empty, the column header row is not shown.
    pub column_headers: Option<ColumnHeaders>,

    /// Index of the main column.
    pub main_column: usize,

    /// Table row height.
    pub row_height: GridLength,

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
    pub selected_style: BoxStyle,
}

impl<'a, Id> Default for TableViewParams<'a, Id> {
    fn default() -> Self {
        TableViewParams {
            selection: None,
            columns: vec![GridTrackDefinition::new(GridLength::Flex(1.0))],
            column_headers: None,
            main_column: 0,
            row_height: GridLength::Auto,
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
        }
    }
}

#[derive(Clone)]
pub struct TableView {
    grid: Grid,
}

impl TableView {
    /// Creates a new tree grid.
    #[composable]
    pub fn new<Id: Hash + Eq + Clone>(mut params: TableViewParams<Id>) -> TableView {
        // create the main grid
        let mut grid = Grid::new();
        let num_columns = params.columns.len();
        grid.append_column_definitions(params.columns);

        // row counter
        let mut i = 0;

        // insert row for column headers
        if let Some(headers) = params.column_headers {
            // header row doesn't follow the specified height of the other rows.
            grid.push_row_definition(GridTrackDefinition::new(GridLength::Auto));
            // insert headers
            for (i, header) in headers.widgets.into_iter().enumerate() {
                grid.add_item_pod(0, i, header);
            }

            if params.resizeable_columns {
                // insert transparent draggable items between each column (over the column separator).
                // they are drag handles for resizing the columns.
                for i in 1..num_columns {
                    let resize_handle = DragController::new(
                        Container::new(Null::new())
                            .background(theme::palette::RED_800)
                            .fixed_width(4.dip())
                            .fixed_height(100.percent()),
                    );
                    // positioning
                    //let resize_handle = LayoutWrapper::with_offset((-2.0, 0.0), resize_handle);
                    // FIXME arbitrary Z-order here should be documented
                    grid.add_item_with_line_alignment(.., i..i, 100, Alignment::CENTER, resize_handle);
                }
            }

            i += 1;
        }

        // set template for the content rows
        grid.set_row_template(params.row_height);

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
            let chevron_expanded =
                Image::from_uri("data/icons/chevron.png", Scaling::Contain).fix_size(icon_size, icon_size);
            let chevron_collapsed =
                Image::from_uri("data/icons/chevron-collapsed.png", Scaling::Contain).fix_size(icon_size, icon_size);

            // fill the visit stack with the initial rows
            let mut visit: Vec<_> = params.rows.into_iter().map(|row| (0usize, row)).rev().collect();

            // depth-order traversal of the row hierarchy
            while let Some((level, row)) = visit.pop() {
                // row selection highlight
                if let Some(selection) = params.selection.as_mut() {
                    if selection.contains(&row.id) {
                        // draw a filled rect with the selection style that spans the whole row
                        grid.add_item(
                            i,
                            ..,
                            -1,
                            Container::new(Null::new())
                                .fill()
                                .box_style(params.selected_style.clone()),
                        );
                    }
                    // also add a clickable rect, and clicking it adds the row to the selection
                    grid.add_item(
                        i,
                        ..,
                        -1,
                        Clickable::new(Null::new())
                            .on_click(|| selection.flip(row.id.clone()))
                            .fill(),
                    );
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

                    let mut widget_with_chevron = Grid::row(GridTrackDefinition::new(GridLength::Auto));
                    widget_with_chevron.add_item(0, 0, 0, icon);
                    widget_with_chevron.add_item(0, 1, 0, row.widget);
                    grid.add_item(
                        i,
                        params.main_column,
                        0,
                        widget_with_chevron.padding_left((level as f64) * params.row_indent),
                    );
                } else {
                    // no chevron
                    grid.add_item(
                        i,
                        params.main_column,
                        0,
                        // first padding is the space for the chevron, second is the indent
                        row.widget
                            .padding_left(icon_size)
                            .padding_left((level as f64) * params.row_indent),
                    );
                }

                // add row cells to the grid
                for (column, cell) in row.cells {
                    grid.add_item_pod(i, column, cell);
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

    fn layer(&self) -> &LayerHandle {
        self.grid.layer()
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        self.grid.layout(ctx, constraints, env)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.grid.route_event(ctx, event, env);
        // handle
    }
}
