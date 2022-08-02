use kyute::{
    text::{FontStyle, FontWeight, FormattedText, FormattedTextExt},
    widget::{prelude::*, ColumnHeaders, ScrollArea, TableRow, TableSelection, TableView, TableViewParams, Text},
};
use kyute_common::Color;
use std::sync::Arc;

// Some comments:
// - making the RowId by hand is just busywork
// - the API for formatted text is atrocious
// - creating rows by hand is annoying

// Overall:
// - is it possible to do things more declaratively?
// - turn a struct into a hierarchy of rows by implementing a trait
//      - root rows
//      - get child rows

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum Section {
    Artists,
    Copyrights,
    Characters,
    Tags,
    Meta,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum RowId {
    Section(Section),
    Artist(u32),
    Copyright(u32),
    Character(u32),
    Tag(u32),
    Meta(u32),
}

impl Default for RowId {
    fn default() -> Self {
        RowId::Meta(0)
    }
}

#[composable]
fn section_header(text: &str) -> impl Widget + 'static {
    Text::new(text.font_size(10.0).font_weight(FontWeight::BOLD)).padding_top(10.dip())
}

#[composable]
fn post_count(count: u32) -> impl Widget + 'static {
    Text::new(format!("{}", count).font_style(FontStyle::Italic)).color(Color::from_hex("#E8E8EA"))
}

#[composable]
fn artist_row(id: u32, artist: &str, count: u32) -> TableRow<RowId> {
    let mut row = TableRow::new(
        RowId::Artist(id),
        Text::new(artist.to_string()).color(Color::from_hex("#B92E12")),
    );
    row.add_cell(1, post_count(count));
    row
}

#[composable]
fn copyright_row(id: u32, copyright: &str, count: u32) -> TableRow<RowId> {
    let mut row = TableRow::new(
        RowId::Copyright(id),
        Text::new(copyright).color(Color::from_hex("#B21EC8")),
    );
    row.add_cell(1, post_count(count));
    row
}

#[composable]
fn character_row(id: u32, character: &str, count: u32) -> TableRow<RowId> {
    let mut row = TableRow::new(
        RowId::Character(id),
        Text::new(character).color(Color::from_hex("#00C82C")),
    );
    row.add_cell(1, post_count(count));
    row
}

#[composable]
fn tag_row(id: u32, tag: &str, count: u32) -> TableRow<RowId> {
    let mut row = TableRow::new(RowId::Tag(id), Text::new(tag).color(Color::from_hex("#2E92C8")));
    row.add_cell(1, post_count(count));
    row
}

#[composable]
fn meta_row(id: u32, meta: &str, count: u32) -> TableRow<RowId> {
    let mut row = TableRow::new(RowId::Meta(id), Text::new(meta).color(Color::from_hex("#E69514")));
    row.add_cell(1, post_count(count));
    row
}

#[composable]
pub fn showcase() -> Arc<WidgetPod> {
    let mut selection = TableSelection::default();

    let mut section_artists = TableRow::new_expanded(RowId::Section(Section::Artists), section_header("Artists"));
    let mut section_copyrights =
        TableRow::new_expanded(RowId::Section(Section::Copyrights), section_header("Copyrights"));
    let mut section_characters =
        TableRow::new_expanded(RowId::Section(Section::Characters), section_header("Characters"));
    let mut section_tags = TableRow::new_expanded(RowId::Section(Section::Tags), section_header("Tags"));
    let mut section_meta = TableRow::new_expanded(RowId::Section(Section::Meta), section_header("Meta"));

    let artists = [("k0nfette", 30)];
    let copyrights = [("touhou", 753047)];
    let characters = [("kicchou yachie", 1214), ("otter spirit (touhou)", 248)];
    let tags = [
        ("1girl", 3990445),
        ("antlers", 8800),
        ("blonde hair", 1001401),
        ("bobby socks", 6800),
        ("chinese clothes", 55000),
        ("dragon tail", 13000),
        ("high heels", 119000),
        ("horns", 251000),
        ("otter", 588),
        ("pointy ears", 214000),
        ("red eyes", 837000),
        ("sharp teeth", 37000),
        ("short hair", 1500000),
        ("skirt", 1000000),
        ("socks", 213000),
        ("solo", 3300000),
        ("square neckline", 227),
        ("tail", 440000),
        ("teeth", 226000),
    ];
    let meta = [("highres", 2800000)];

    for (i, (artist, count)) in artists.iter().enumerate() {
        section_artists.add_row(artist_row(i as u32, artist, *count));
    }
    for (i, (copyright, count)) in copyrights.iter().enumerate() {
        section_copyrights.add_row(copyright_row(i as u32, copyright, *count));
    }
    for (i, (character, count)) in characters.iter().enumerate() {
        section_characters.add_row(character_row(i as u32, character, *count));
    }
    for (i, (tag, count)) in tags.iter().enumerate() {
        section_tags.add_row(tag_row(i as u32, tag, *count));
    }
    for (i, (meta, count)) in meta.iter().enumerate() {
        section_meta.add_row(meta_row(i as u32, meta, *count));
    }

    let table_params = TableViewParams {
        selection: Some(&mut selection),
        template: Default::default(),
        column_headers: None,
        main_column: 0,
        rows: vec![
            section_artists,
            section_copyrights,
            section_characters,
            section_tags,
            section_meta,
        ],
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
    };

    ScrollArea::new(TableView::new(table_params)).arc_pod()
}
