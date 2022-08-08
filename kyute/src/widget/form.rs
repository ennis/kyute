use crate::widget::{grid, prelude::*, Grid, ScrollArea, TableView};
use std::sync::Arc;

pub trait LabeledContent {
    type Label: Widget + 'static;
    type Content: Widget + 'static;

    fn into_label_content(self) -> (Self::Label, Self::Content);
}

/// A field in a form layout.
#[derive(Debug)]
pub enum Row {
    Section {
        title: Arc<WidgetPod>,
        rows: Vec<Row>,
    },
    Field {
        label: Arc<WidgetPod>,
        content: Arc<WidgetPod>,
        /// Put the content before the label.
        ///
        /// Used for checkboxes / radio groups, which usually appear before their labels.
        swap_content_and_label: bool,
    },
}

/// Form layout.
///
/// At least two columns: label | value.
/// Possibly additional columns that contain buttons, drop down menu, etc.
/// Possible to group rows into sections.
#[derive(Widget)]
pub struct Form {
    inner: ScrollArea,
}

fn place_rows_recursive(grid: &mut Grid, current_row: &mut usize, rows: impl IntoIterator<Item = Row>) {
    for row in rows.into_iter() {
        match row {
            Row::Field {
                label,
                content,
                swap_content_and_label,
            } => {
                if !swap_content_and_label {
                    grid.place((*current_row, 0), 0, label);
                    grid.place((*current_row, 1), 0, content);
                } else {
                    grid.place((*current_row, 0), 0, content);
                    grid.place((*current_row, 1), 0, label);
                }
            }
            Row::Section { title, rows } => {
                grid.place((*current_row, ..), 0, title);
                *current_row += 1;
                place_rows_recursive(grid, current_row, rows)
            }
        }
        *current_row += 1;
    }
}

impl Form {
    #[composable]
    pub fn new(rows: impl IntoIterator<Item = Row>) -> Form {
        let mut grid = Grid::with_template("/ 1fr 3fr");
        grid.set_row_gap(4.px());

        place_rows_recursive(&mut grid, &mut 0, rows);

        Form {
            inner: ScrollArea::new(grid),
        }
    }
}

pub struct Section<Title> {
    title: Title,
    rows: Vec<Row>,
}

impl<Title> Section<Title> {
    pub fn new(title: Title, rows: impl IntoIterator<Item = Row>) -> Section<Title> {
        Section {
            title,
            rows: rows.into_iter().collect(),
        }
    }
}

impl<Title> From<Section<Title>> for Row
where
    Title: Widget + 'static,
{
    fn from(section: Section<Title>) -> Self {
        Row::Section {
            title: section.title.font_size(0.8.em()).padding_top(5.px()).arc_pod(),
            rows: section.rows,
        }
    }
}
