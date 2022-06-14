use crate::{
    cache,
    style::{BorderStyle, BoxShadow, Paint},
    Color, Data, Length, SideOffsets,
};
use kyute_common::{imbl::Vector, Angle};
use std::{
    any::Any,
    collections::HashMap,
    fmt,
    fmt::Formatter,
    hash::{Hash, Hasher},
    marker::PhantomData,
    sync::Arc,
};

#[derive(Clone, Debug)]
pub enum PropertyValue {
    String(Arc<str>),
    Number(f64),
    Length(Length),
    Paint(Paint),
    Angle(Angle),
    BoxShadows(Vector<BoxShadow>),
    BorderStyle(BorderStyle),
}

/// Style property identifier.
#[derive(Debug, Eq, PartialEq)]
pub struct Property<T> {
    key: &'static str,
    inherited: bool,
    _type: PhantomData<T>,
}

impl<T> Property<T> {
    pub fn name(&self) -> &'static str {
        self.key
    }
}

impl<T> Clone for Property<T> {
    fn clone(&self) -> Self {
        Property {
            key: self.key,
            inherited: self.inherited,
            _type: PhantomData,
        }
    }
}

impl<T> Copy for Property<T> {}

impl<T> Property<T> {
    pub const fn new(key: &'static str, inherited: bool) -> Property<T> {
        Property {
            key,
            inherited,
            _type: PhantomData,
        }
    }
}

macro_rules! define_properties {
    (@inherited inherited) => { true };
    (@inherited) => { false };
    ($($(#[$inherited:ident])? $name:ident[ $key:literal ] : $t:ty ;)*) => {
        $(
            pub const $name: Property<$t> = Property::new($key, define_properties!(@inherited $($inherited)?));
        )*
    };
}

define_properties! {
    BORDER_BOTTOM_WIDTH["border-bottom-width"]: Length;
    BORDER_TOP_WIDTH["border-top-width"]: Length;
    BORDER_LEFT_WIDTH["border-left-width"]: Length;
    BORDER_RIGHT_WIDTH["border-right-width"]: Length;
    BORDER_TOP_LEFT_RADIUS["border-top-left-radius"]: Length;
    BORDER_TOP_RIGHT_RADIUS["border-top-right-radius"]: Length;
    BORDER_BOTTOM_RIGHT_RADIUS["border-bottom-right-radius"]: Length;
    BORDER_BOTTOM_LEFT_RADIUS["border-bottom-left-radius"]: Length;
    BORDER_BOTTOM_COLOR["border-bottom-color"]: Color;
    BORDER_TOP_COLOR["border-top-color"]: Color;
    BORDER_LEFT_COLOR["border-left-color"]: Color;
    BORDER_RIGHT_COLOR["border-right-color"]: Color;
    BORDER_STYLE["border-style"]: BorderStyle;
    BACKGROUND_IMAGE["background-image"]: Paint;
    BACKGROUND_COLOR["background-color"]: Color;
    //BOX_SHADOW["box-shadow"]: BoxShadows;
    MIN_WIDTH["min-width"]: Length;
    MIN_HEIGHT["min-height"]: Length;
    MAX_WIDTH["max-width"]: Length;
    MAX_HEIGHT["max-height"]: Length;
    WIDTH["width"]: Length;
    HEIGHT["height"]: Length;
    PADDING_LEFT["padding-left"]: Length;
    PADDING_RIGHT["padding-right"]: Length;
    PADDING_TOP["padding-top"]: Length;
    PADDING_BOTTOM["padding-bottom"]: Length;

    #[inherited] FONT_SIZE["FONT_SIZE"]: Length;
}
