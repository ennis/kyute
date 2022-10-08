//! Tree views.
use crate::{
    cache,
    drawing::Paint,
    style,
    style::Style,
    theme,
    widget::{
        align::VerticalAlignment,
        grid,
        grid::{GridLayoutExt, GridTemplate, TrackBreadth, TrackSize},
        prelude::*,
        Clickable, DebugFlags, DragController, Grid, Image, Null, Placeholder, Scaling,
    },
    Data, Length, State, UnitExt,
};
use kyute_common::imbl;
use kyute_shell::winit;
use std::{cmp, convert::TryFrom, hash::Hash, sync::Arc};

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

/// Data model trait for the contents of a table column.
pub trait ColumnModel<Row> {
    fn cell(&self, row: &Row) -> Arc<WidgetPod>;
}

pub trait Identifiable {
    /// The row identity type
    type Id: Clone + Eq + PartialEq + Ord + PartialOrd + Hash;

    fn id(&self) -> Self::Id;
}

impl<'a, T: Identifiable> Identifiable for &'a T {
    type Id = T::Id;

    fn id(&self) -> Self::Id {
        (*self).id()
    }
}

pub trait Collection<Item: Identifiable> {
    /// The number of items in the collection.
    fn len(&self) -> usize;

    /// Fetches a row at the specified index.
    fn row(&self, index: usize) -> Item;

    /// Returns the number of child items.
    fn child_count(&self, parent: &Item) -> usize;

    /// Fetches the nth child of a row.
    fn child(&self, parent: &Item, index: usize) -> Item;
}

/*/// A row of a TableView.
///
/// See `TableViewParams::rows`
#[derive(Clone)]
pub struct Row<Id> {
    /// Uniquely identifies this row among others in the same table.
    id: Id,
    /// The widgets to put in the other columns.
    cells: Vec<(WidgetId, Arc<WidgetPod>)>,
    /// Whether the children of this row are expanded, if there is any.
    expanded: bool,
    /// Whether to show the chevron.
    show_chevron: bool,
    /// Child rows
    children: Vec<Row<Id>>,
    expanded_changed: Signal<bool>,
}

impl<Id> Row<Id> {
    /// Creates a new row with the specified ID.
    #[composable]
    pub fn new(id: Id) -> Row<Id> {
        #[state]
        let mut expanded = false;
        Self::new_inner(id, expanded, true).on_expanded_changed(|v| expanded = v)
    }

    /// Creates a new, initially expanded, row with the specified ID.
    #[composable]
    pub fn new_expanded(id: Id) -> Row<Id> {
        Self::new_inner(id, true, false)
    }

    #[composable]
    fn new_inner(id: Id, expanded: bool, show_chevron: bool) -> Row<Id> {
        Row {
            id,
            cells: vec![],
            expanded,
            show_chevron,
            children: vec![],
            expanded_changed: Signal::new(),
        }
    }

    /// Forces this row to display expanded or collapsed.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Invokes the provided closure when the row is expanded or collapsed.
    pub fn on_expanded_changed(self, f: impl FnOnce(bool)) -> Self {
        self.expanded_changed.map(f);
        self
    }

    /// Whether the node is expanded, and the child nodes are visible.
    pub fn is_expanded(&self) -> bool {
        self.expanded
    }

    /// Sets the widget of the cell corresponding to the given column.
    /// TODO it might be more practical to identify the column by ID
    pub fn cell(mut self, column: &Column, widget: impl Widget + 'static) -> Self {
        self.cells.push((column.inner.widget_id().unwrap(), widget.arc_pod()));
        self
    }

    /// Sets the widget of the cell corresponding to the given column.
    pub fn push_cell(&mut self, column: &Column, widget: impl Widget + 'static) {
        self.cells
            .push((column.inner.widget_id().unwrap(), Arc::new(WidgetPod::new(widget))));
    }

    /// Adds a child row node.
    pub fn add_row(&mut self, child: Row<Id>) {
        self.children.push(child);
    }
}*/

/// A column with a clickable header.
pub struct Column<'a, Row> {
    /// The contents of the column header, made clickable. Usually a text element.
    inner: Clickable<Arc<WidgetPod>>,
    delegate: &'a dyn Fn(&Row) -> Arc<WidgetPod>,
    /// Requested size of the column.
    size: TrackSize,
    /// Current size of the column. `None` if the column is not resizable.
    current_size: Option<State<f64>>,
    /// Whether this is the outline column.
    outline: bool,
    moved: Signal<()>,
    clicked: Signal<()>,
}

impl<'a, Row> Column<'a, Row> {
    /// Creates a new column with a fixed size.
    #[composable]
    pub fn new(header: impl Widget + 'static, delegate: &'a dyn Fn(&Row) -> Arc<WidgetPod>) -> Column<'a, Row> {
        let inner: Clickable<Arc<WidgetPod>> = Clickable::new(header.arc_pod());
        let moved = Signal::new();
        let clicked = Signal::new();
        Column {
            inner,
            size: TrackSize::new(TrackBreadth::Auto),
            current_size: None,
            outline: false,
            moved,
            delegate,
            clicked,
        }
    }

    /// Marks this column as the outline column, which displays the row hierarchy indentation.
    pub fn outline(mut self) -> Self {
        self.outline = true;
        self
    }

    /// Sets the width of the column.
    pub fn width(mut self, width: TrackSize) -> Self {
        self.size = width;
        self
    }

    /// Makes this column resizable.
    ///
    /// # Arguments
    /// * initial_size the initial size of the column
    #[composable]
    pub fn resizable(mut self, initial_size: f64) -> Self {
        let size = cache::state(|| initial_size);
        self.current_size = Some(size);
        self
    }

    /// Invokes the provided closure when the column has been moved.
    pub fn on_move(self, f: impl FnOnce()) -> Self {
        if self.moved.signalled() {
            f()
        }
        self
    }

    /// Invokes the provided closure when the column header has been clicked.
    pub fn on_click_header(self, f: impl FnOnce()) -> Self {
        if self.clicked.signalled() {
            f()
        }
        self
    }

    fn is_resizable(&self) -> bool {
        self.current_size.is_some()
    }

    /*#[composable]
    pub fn add(mut self, widget: impl Widget + 'static) -> Self {
        self.widgets.push(Arc::new(WidgetPod::new(widget)));
        self
    }*/
}

/// Style of a table view.
pub struct TableViewStyle {
    /// Background style.
    pub background: style::Image,

    /// Alternate background style, used every other row.
    pub alternate_background: style::Image,

    /// Width of the row separators.
    pub row_separator_width: Length,

    /// Width of the column separators.
    pub column_separator_width: Length,

    /// Style of the row separators.
    pub row_separator_background: style::Image,

    /// Style of the column separators.
    pub column_separator_background: style::Image,

    /// Style of selected items.
    pub selected_style: Style,

    /// Expanded indicator image URI.
    /// TODO make this a VectorIcon
    pub expanded_row_marker_uri: String,

    /// Expanded indicator image URI.
    /// TODO make this a VectorIcon
    pub collapsed_row_marker_uri: String,

    /// Row indentation.
    pub indentation: Length,
}

impl Default for TableViewStyle {
    fn default() -> Self {
        TableViewStyle {
            background: theme::CONTENT_BACKGROUND_COLOR.into(),
            alternate_background: theme::ALTERNATE_CONTENT_BACKGROUND_COLOR.into(),
            row_separator_width: 1.px(),
            column_separator_width: 1.px(),
            row_separator_background: theme::TEXT_COLOR.into(),
            column_separator_background: theme::TEXT_COLOR.into(),
            selected_style: Default::default(),
            expanded_row_marker_uri: "data/icons/chevron.png".to_string(),
            collapsed_row_marker_uri: "data/icons/chevron-collapsed.png".to_string(),
            indentation: 16.dip(),
        }
    }
}

/// Builder helper for a TableView widget.
pub struct TableViewParams<'a, Item>
where
    Item: Identifiable,
{
    /// Reference to the current table selection.
    ///
    /// If None, selection is disabled.
    pub selection: Option<&'a mut TableSelection<Item::Id>>,

    /// Column headers.
    pub columns: Vec<Column<'a, Item>>,

    pub show_expand_buttons: bool,

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

    /// Table style.
    pub style: TableViewStyle,
}

impl<'a, Item: Identifiable> Default for TableViewParams<'a, Item> {
    fn default() -> Self {
        TableViewParams {
            selection: None,
            columns: vec![],
            show_expand_buttons: true,
            resizeable_columns: false,
            reorderable_rows: false,
            reorderable_columns: false,
            style: TableViewStyle::default(),
        }
    }
}

impl<'a, Item: Identifiable> TableViewParams<'a, Item> {
    /// Adds a table column.
    pub fn column(mut self, column: Column<'a, Item>) -> Self {
        self.columns.push(column);
        self
    }

    /// Sets whether to display the row expand buttons.
    pub fn show_expand_buttons(mut self, show: bool) -> Self {
        self.show_expand_buttons = show;
        self
    }
}

// Tables: collection models VS procedural?
//
// Collection models:
// - fetch items & cells on-demand
// - callbacks
//
// Procedural:
// - user creates all rows by hand, places all widgets manually

pub struct TableView {
    grid: Grid,
}

impl TableView {
    /// Creates a new tree grid.
    #[composable]
    pub fn new<Item: Identifiable>(mut params: TableViewParams<Item>, collection: impl Collection<Item>) -> TableView {
        // create the main grid

        // TODO identify columns by name
        let mut grid_template = GridTemplate::new();
        for p in params.columns.iter() {
            if let Some(current_size) = p.current_size.as_ref() {
                grid_template
                    .columns
                    .sizes
                    .push(TrackSize::new(current_size.get().dip()));
            } else {
                grid_template.columns.sizes.push(p.size);
            }
        }

        let mut grid = Grid::new(Arc::new(grid_template));

        //let num_columns = grid.column_count();

        // row counter
        let mut row_index = 0;

        grid.set_row_background(params.style.background);
        grid.set_alternate_row_background(params.style.alternate_background);
        grid.set_row_gap_background(params.style.row_separator_background);
        grid.set_column_gap_background(params.style.column_separator_background);
        grid.set_row_gap(params.style.row_separator_width);
        grid.set_column_gap(params.style.column_separator_width);

        // insert rows
        {
            let icon_size = params.style.indentation;
            let chevron_expanded = Image::from_uri(&params.style.expanded_row_marker_uri, Scaling::Contain)
                .frame(icon_size, icon_size)
                .arc_pod();
            let chevron_collapsed = Image::from_uri(&params.style.collapsed_row_marker_uri, Scaling::Contain)
                .frame(icon_size, icon_size)
                .arc_pod();

            // fill the visit stack with the initial rows
            //let mut visit: Vec<_> = params.rows.into_iter().map(|row| (0usize, row)).rev().collect();

            let mut visit = Vec::new();
            for row_index in (0..collection.len()).rev() {
                visit.push((0usize, collection.row(row_index)));
            }

            // depth-order traversal of the row hierarchy
            while let Some((indent_level, row)) = visit.pop() {
                let id = row.id();

                cache::enter(&id);
                // row selection highlight
                if let Some(selection) = params.selection.as_mut() {
                    if selection.contains(&id) {
                        // draw a filled rect with the selection style that spans the whole row

                        // .box_style(params.selected_style.clone())
                        grid.insert(Null.fill().grid_area((row_index, ..)));
                    }
                    // also add a clickable rect, and clicking it adds the row to the selection
                    /*grid.insert(
                        Null.clickable()
                            .on_click(|| selection.flip(row.id.clone()))
                            .fill()
                            .grid_area((i, ..)),
                    );*/
                }
                cache::exit();

                // state that remembers whether the row is expanded or not
                let expanded_state = cache::state(|| false);
                let expanded = expanded_state.get();

                // add row cells to the grid
                for (column_index, column) in params.columns.iter().enumerate() {
                    cache::enter((id.clone(), column_index));
                    let cell_widget = (column.delegate)(&row);
                    let child_count = collection.child_count(&row);

                    if column.outline {
                        // it's an outline column, apply indent level
                        if params.show_expand_buttons && child_count != 0 {
                            // showing the expand buttons & the column has children

                            // FIXME: cache::scoped ugliness
                            let expand_button = Clickable::new(if expanded {
                                chevron_expanded.clone()
                            } else {
                                chevron_collapsed.clone()
                            })
                            .on_click(|| {
                                expanded_state.set(!expanded);
                            });

                            grid.insert(
                                expand_button
                                    .left_of(cell_widget.clone(), Alignment::CENTER)
                                    .padding_left((indent_level as f64) * params.style.indentation)
                                    .grid_area((row_index, column_index)),
                            );
                        } else {
                            // no expand button
                            // first padding is the space for the chevron, second is the indent
                            grid.insert(
                                cell_widget
                                    .clone()
                                    .padding_left(icon_size)
                                    .padding_left((indent_level as f64) * params.style.indentation)
                                    .grid_area((row_index, column_index)),
                            );
                        }
                    } else {
                        grid.insert(cell_widget.clone().grid_area((row_index, column_index)));
                    }
                    cache::exit();
                }

                // visit child rows if the row is expanded
                if expanded || !params.show_expand_buttons {
                    let child_count = collection.child_count(&row);
                    for child_index in (0..child_count).rev() {
                        visit.push((indent_level + 1, collection.child(&row, child_index)));
                    }
                }
                row_index += 1;
            }
        }

        //------------------------------------------
        // column resizing
        for i_col_split in 1..params.columns.len() {
            let (left_cols, right_cols) = params.columns.split_at_mut(i_col_split);
            let left_column = left_cols.last_mut().unwrap();
            let right_column = right_cols.first_mut().unwrap();

            // insert an invisible resize handle between two resizable columns
            if left_column.is_resizable() && right_column.is_resizable() {
                let left_column_size = left_column.current_size.as_ref().unwrap().get();
                let right_column_size = right_column.current_size.as_ref().unwrap().get();

                let resize_handle = DragController::new(
                    (left_column_size, right_column_size),
                    Placeholder
                        .frame(4.dip(), 50.dip())
                        .left(-2.dip())
                        .cursor_icon(winit::window::CursorIcon::ColResize)
                        .debug(DebugFlags::DUMP_GEOMETRY | DebugFlags::DUMP_CONSTRAINTS),
                )
                .on_delta(|(left, right), offset| {
                    trace!("column resize drag offset={offset:?}");
                    left_column.current_size.as_mut().unwrap().set(left + offset.x);
                    right_column
                        .current_size
                        .as_mut()
                        .unwrap()
                        .set((right - offset.x).max(0.0));
                })
                .debug_name("table resize handle");

                grid.place(
                    (0..row_index, i_col_split), // span all rows, but only the current column
                    99,                          // place it over all other grid items
                    resize_handle.arc_pod(),
                );
            }
        }

        TableView { grid }
    }
}

impl Widget for TableView {
    fn widget_id(&self) -> Option<WidgetId> {
        self.grid.widget_id()
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutParams, env: &Environment) -> Geometry {
        self.grid.layout(ctx, constraints, env)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.grid.route_event(ctx, event, env);
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        self.grid.paint(ctx)
    }
}
