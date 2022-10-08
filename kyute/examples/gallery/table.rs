use crate::main;
use kyute::{
    text::{FontStyle, FontWeight, FormattedText, FormattedTextExt},
    widget::{
        prelude::*,
        table,
        table::{Collection, Column, Identifiable},
        Null, ScrollArea, TableSelection, TableView, TableViewParams, Text,
    },
};
use kyute_common::Color;
use std::{str, sync::Arc};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum Section {
    Artists,
    Copyrights,
    Characters,
    Tags,
    Meta,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum RowId {
    Section(Section),
    Artist(u32),
    Copyright(u32),
    Character(u32),
    Tag(u32),
    Meta(u32),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum TagType {
    Artist,
    Copyright,
    Character,
    Tag,
    Meta,
}

enum Row<'a> {
    Section(Section),
    Tag {
        id: u32,
        ty: TagType,
        name: &'a str,
        post_count: u64,
    },
}

impl<'a> Identifiable for Row<'a> {
    type Id = RowId;

    fn id(&self) -> Self::Id {
        match *self {
            Row::Section(section) => RowId::Section(section),
            Row::Tag { ty, id, .. } => match ty {
                TagType::Artist => RowId::Artist(id),
                TagType::Copyright => RowId::Copyright(id),
                TagType::Character => RowId::Character(id),
                TagType::Tag => RowId::Tag(id),
                TagType::Meta => RowId::Meta(id),
            },
        }
    }
}

struct TagsModel<'a> {
    artists: &'a [(&'a str, u64)],
    copyrights: &'a [(&'a str, u64)],
    characters: &'a [(&'a str, u64)],
    tags: &'a [(&'a str, u64)],
    meta: &'a [(&'a str, u64)],
}

impl<'a> Collection<Row<'a>> for TagsModel<'a> {
    fn len(&self) -> usize {
        5
    }

    fn row(&self, index: usize) -> Row<'a> {
        match index {
            0 => Row::Section(Section::Artists),
            1 => Row::Section(Section::Copyrights),
            2 => Row::Section(Section::Characters),
            3 => Row::Section(Section::Tags),
            4 => Row::Section(Section::Meta),
            _ => panic!("out-of-bounds access"),
        }
    }

    fn child_count(&self, parent: &Row) -> usize {
        match parent {
            Row::Section(section) => match section {
                Section::Artists => self.artists.len(),
                Section::Copyrights => self.copyrights.len(),
                Section::Characters => self.characters.len(),
                Section::Tags => self.tags.len(),
                Section::Meta => self.meta.len(),
            },
            _ => 0,
        }
    }

    fn child(&self, parent: &Row, index: usize) -> Row<'a> {
        match parent {
            Row::Section(section) => match section {
                Section::Artists => Row::Tag {
                    ty: TagType::Artist,
                    post_count: self.artists[index].1,
                    name: self.artists[index].0,
                    id: index as u32,
                },
                Section::Copyrights => Row::Tag {
                    ty: TagType::Copyright,
                    post_count: self.copyrights[index].1,
                    name: self.copyrights[index].0,
                    id: index as u32,
                },
                Section::Characters => Row::Tag {
                    ty: TagType::Character,
                    post_count: self.characters[index].1,
                    name: self.characters[index].0,
                    id: index as u32,
                },
                Section::Tags => Row::Tag {
                    ty: TagType::Tag,
                    post_count: self.tags[index].1,
                    name: self.tags[index].0,
                    id: index as u32,
                },
                Section::Meta => Row::Tag {
                    ty: TagType::Meta,
                    post_count: self.meta[index].1,
                    name: self.meta[index].0,
                    id: index as u32,
                },
            },
            _ => panic!("out-of-bounds access"),
        }
    }
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
fn post_count(count: u64) -> impl Widget + 'static {
    Text::new(format!("{count}").font_style(FontStyle::Italic)).color(Color::from_hex("#E8E8EA"))
}

#[composable]
pub fn showcase() -> Arc<WidgetPod> {
    //let mut selection = TableSelection::default();

    let model = TagsModel {
        artists: &[("k0nfette", 30)],
        copyrights: &[("touhou", 753047)],
        characters: &[("kicchou yachie", 1214), ("otter spirit (touhou)", 248)],
        tags: &[
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
        ],
        meta: &[("highres", 2800000)],
    };

    let main_column_delegate = |row: &Row| match *row {
        Row::Section(Section::Artists) => section_header("Artists").arc_dyn_pod(),
        Row::Section(Section::Copyrights) => section_header("Copyrights").arc_dyn_pod(),
        Row::Section(Section::Characters) => section_header("Characters").arc_dyn_pod(),
        Row::Section(Section::Tags) => section_header("Tags").arc_dyn_pod(),
        Row::Section(Section::Meta) => section_header("Meta").arc_dyn_pod(),
        Row::Tag { ty, name, .. } => match ty {
            TagType::Artist => Text::new(name).color(Color::from_hex("#B92E12")).arc_dyn_pod(),
            TagType::Copyright => Text::new(name).color(Color::from_hex("#B21EC8")).arc_dyn_pod(),
            TagType::Character => Text::new(name).color(Color::from_hex("#00C82C")).arc_dyn_pod(),
            TagType::Tag => Text::new(name).color(Color::from_hex("#2E92C8")).arc_dyn_pod(),
            TagType::Meta => Text::new(name).color(Color::from_hex("#E69514")).arc_dyn_pod(),
        },
    };

    let post_count_column_delegate = |row: &Row| match *row {
        Row::Section(_) => Null.arc_dyn_pod(),
        Row::Tag { post_count: count, .. } => post_count(count).arc_dyn_pod(),
    };

    let main_column = Column::new(Text::new("Tag"), &main_column_delegate);
    let post_count_column = Column::new(Text::new("Post count"), &post_count_column_delegate);

    let mut table_params = TableViewParams::default();
    table_params.show_expand_buttons = false;
    table_params.columns.push(main_column);
    table_params.columns.push(post_count_column);

    ScrollArea::new(TableView::new(table_params, model)).arc_pod()
}
