use kyute::{
    application, composable,
    shell::{application::Application, winit::window::WindowBuilder},
    style::BoxStyle,
    theme,
    widget::{
        grid::GridTrackDefinition, Button, Container, Flex, Grid, GridLength, Image, Label, Null, Popup, Text,
        TitledPane, TreeGrid, TreeNode,
    },
    Alignment, AssetId, BoxConstraints, Color, EnvKey, Environment, Orientation, Size, UnitExt, Widget, WidgetExt,
    WidgetPod, Window,
};
use kyute_common::SideOffsets;
use std::sync::Arc;
use tracing::trace;

#[composable]
fn tree_test() -> impl Widget + Clone {
    let mut tree = TreeGrid::new([GridTrackDefinition::new(GridLength::Fixed(300.0))]);

    let mut root = TreeNode::new(Text::new("root"));
    for i in 0..3 {
        let mut n1 = TreeNode::new(Text::new(format!("Node {}", i)));
        for j in 0..3 {
            let mut n2 = TreeNode::new(Text::new(format!("Node {}.{}", i, j)));
            for k in 0..2 {
                n2.add_child(TreeNode::new(
                    Container::new(Text::new(format!("Node {}.{}.{}", i, j, k)))
                        .content_padding(SideOffsets::new_all_same(3.0))
                        .box_style(theme::DROP_DOWN),
                ));
            }
            n1.add_child(n2);
        }
        root.add_child(n1);
    }

    tree.set_root(root);
    tree
}

#[composable]
fn ui_root() -> impl Widget {
    Window::new(WindowBuilder::new().with_title("Tree view"), tree_test(), None)
}

fn main() {
    tracing_subscriber::fmt()
        .compact()
        .with_target(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let _app = Application::new();
    application::run(ui_root);
    Application::shutdown();
}
