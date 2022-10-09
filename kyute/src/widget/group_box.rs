//! Group boxes
use crate::{
    text::FormattedText,
    widget::{prelude::*, StyledBox, Text, WidgetExt},
};

const GROUP_BOX_STYLE: &str = r#"
    border-radius: 4px;
    padding: 6px;
    [$dark-mode] background: #00000040;
    [!$dark-mode] background: #20202010;
"#;

type GroupBoxInner<T: Widget + 'static> = impl Widget;

#[composable]
fn group_box_inner<Content: Widget + 'static>(
    label: impl Into<FormattedText>,
    content: Content,
) -> GroupBoxInner<Content> {
    content
        .style(GROUP_BOX_STYLE)
        .below(Text::new(label), Alignment::START)
        .padding(4.dip())
}

/// A container with a fixed width and height, into which an unique widget is placed.
#[derive(Widget)]
pub struct GroupBox<Content: Widget + 'static> {
    inner: GroupBoxInner<Content>,
}

impl<Content: Widget + 'static> GroupBox<Content> {
    #[composable]
    pub fn new(label: impl Into<FormattedText>, content: Content) -> GroupBox<Content> {
        GroupBox {
            inner: group_box_inner(label, content),
        }
    }
}
