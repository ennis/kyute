use crate::env::Key;
use crate::layout::SideOffsets;
/// Environment keys that control the visual aspect (theme) of common widgets.
use kyute_shell::drawing::Color;

impl_keys!(
/// Preferred slider height.
SliderHeight : f64 [10.0];
/// Default font size.
FontSize: f64 [14.0];
/// Default font family
#[cfg(windows)]
FontName: &'a str ["Segoe UI"];
/// Minimum button width
MinButtonWidth : f64 [10.0];
/// Minimum button height
MinButtonHeight : f64 [10.0];
/// Button background color.
ButtonBackgroundColor : Color [Color::new(0.0, 0.0, 0.0, 1.0)];
/// Button border color.
ButtonBorderColor : Color [Color::new(0.0, 0.0, 0.0, 1.0)];
/// Label padding.
ButtonLabelPadding : SideOffsets [SideOffsets::default()];

FlexSpacing: f64 [2.0];

/// Label padding.
SliderPadding : SideOffsets [SideOffsets::default()];
/// Label padding.
SliderKnobWidth : f64 [5.0];
/// Label padding.
SliderKnobHeight : f64 [5.0];

///
TextEditFontSize: f64 [14.0];
TextEditFontName: &'a str ["Consolas"];
TextEditPadding: SideOffsets [SideOffsets::new_all_same(2.0)];
TextEditCaretColor: Color [Color::new(0.0,0.0,0.0,1.0)];

TextColor : Color [Color::new(0.0,0.0,0.0,1.0)];
SelectedTextBackgroundColor : Color [Color::new(0.0,0.0,0.0,1.0)];
SelectedTextColor : Color [Color::new(1.0,1.0,1.0,1.0)];
);
