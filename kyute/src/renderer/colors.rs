use lazy_static::lazy_static;
use palette::Srgba;

pub struct Colors {
    pub text: [f32; 4],
    pub text_disabled: [f32; 4],
    pub window_bg: [f32; 4],
    pub child_bg: [f32; 4],
    pub popup_bg: [f32; 4],
    pub border: [f32; 4],
    pub border_shadow: [f32; 4],
    pub frame_bg: [f32; 4],
    pub frame_bg_hovered: [f32; 4],
    pub frame_bg_active: [f32; 4],
    pub title_bg: [f32; 4],
    pub title_bg_active: [f32; 4],
    pub title_bg_collapsed: [f32; 4],
    pub menu_bar_bg: [f32; 4],
    pub scrollbar_bg: [f32; 4],
    pub scrollbar_grab: [f32; 4],
    pub scrollbar_grab_hovered: [f32; 4],
    pub scrollbar_grab_active: [f32; 4],
    pub check_mark: [f32; 4],
    pub slider_grab: [f32; 4],
    pub slider_grab_active: [f32; 4],
    pub button: [f32; 4],
    pub button_hovered: [f32; 4],
    pub button_active: [f32; 4],
    pub header: [f32; 4],
    pub header_hovered: [f32; 4],
    pub header_active: [f32; 4],
    pub separator: [f32; 4],
    pub separator_hovered: [f32; 4],
    pub separator_active: [f32; 4],
    pub resize_grip: [f32; 4],
    pub resize_grip_hovered: [f32; 4],
    pub resize_grip_active: [f32; 4],
    pub tab: [f32; 4],
    pub tab_hovered: [f32; 4],
    pub tab_active: [f32; 4],
    pub tab_unfocused: [f32; 4],
    pub tab_unfocused_active: [f32; 4],
    pub docking_preview: [f32; 4],
    pub docking_empty_bg: [f32; 4],
    pub plot_lines: [f32; 4],
    pub plot_lines_hovered: [f32; 4],
    pub plot_histogram: [f32; 4],
    pub plot_histogram_hovered: [f32; 4],
    pub text_selected_bg: [f32; 4],
    pub drag_drop_target: [f32; 4],
    pub nav_highlight: [f32; 4],
    pub nav_windowing_highlight: [f32; 4],
    pub nav_windowing_dim_bg: [f32; 4],
    pub modal_window_dim_bg: [f32; 4],
}

lazy_static! {
    pub static ref DEFAULT_COLORS: Colors = Colors {
        text: Srgba::new(0.95f32, 0.96f32, 0.98f32, 1.00f32),
        text_disabled: Srgba::new(0.36f32, 0.42f32, 0.47f32, 1.00f32),
        window_bg: Srgba::new(0.11f32, 0.15f32, 0.17f32, 1.00f32),
        child_bg: Srgba::new(0.15f32, 0.18f32, 0.22f32, 1.00f32),
        popup_bg: Srgba::new(0.08f32, 0.08f32, 0.08f32, 0.94f32),
        border: Srgba::new(0.08f32, 0.10f32, 0.12f32, 1.00f32),
        border_shadow: Srgba::new(0.00f32, 0.00f32, 0.00f32, 0.00f32),
        frame_bg: Srgba::new(0.20f32, 0.25f32, 0.29f32, 1.00f32),
        frame_bg_hovered: Srgba::new(0.12f32, 0.20f32, 0.28f32, 1.00f32),
        frame_bg_active: Srgba::new(0.09f32, 0.12f32, 0.14f32, 1.00f32),
        title_bg: Srgba::new(0.09f32, 0.12f32, 0.14f32, 0.65f32),
        title_bg_active: Srgba::new(0.08f32, 0.10f32, 0.12f32, 1.00f32),
        title_bg_collapsed: Srgba::new(0.00f32, 0.00f32, 0.00f32, 0.51f32),
        menu_bar_bg: Srgba::new(0.15f32, 0.18f32, 0.22f32, 1.00f32),
        scrollbar_bg: Srgba::new(0.02f32, 0.02f32, 0.02f32, 0.39f32),
        scrollbar_grab: Srgba::new(0.20f32, 0.25f32, 0.29f32, 1.00f32),
        scrollbar_grab_hovered: Srgba::new(0.18f32, 0.22f32, 0.25f32, 1.00f32),
        scrollbar_grab_active: Srgba::new(0.09f32, 0.21f32, 0.31f32, 1.00f32),
        check_mark: Srgba::new(0.28f32, 0.56f32, 1.00f32, 1.00f32),
        slider_grab: Srgba::new(0.28f32, 0.56f32, 1.00f32, 1.00f32),
        slider_grab_active: Srgba::new(0.37f32, 0.61f32, 1.00f32, 1.00f32),
        button: Srgba::new(0.20f32, 0.25f32, 0.29f32, 1.00f32),
        button_hovered: Srgba::new(0.28f32, 0.56f32, 1.00f32, 1.00f32),
        button_active: Srgba::new(0.06f32, 0.53f32, 0.98f32, 1.00f32),
        header: Srgba::new(0.20f32, 0.25f32, 0.29f32, 0.55f32),
        header_hovered: Srgba::new(0.26f32, 0.59f32, 0.98f32, 0.80f32),
        header_active: Srgba::new(0.26f32, 0.59f32, 0.98f32, 1.00f32),
        separator: Srgba::new(0.20f32, 0.25f32, 0.29f32, 1.00f32),
        separator_hovered: Srgba::new(0.10f32, 0.40f32, 0.75f32, 0.78f32),
        separator_active: Srgba::new(0.10f32, 0.40f32, 0.75f32, 1.00f32),
        resize_grip: Srgba::new(0.26f32, 0.59f32, 0.98f32, 0.25f32),
        resize_grip_hovered: Srgba::new(0.26f32, 0.59f32, 0.98f32, 0.67f32),
        resize_grip_active: Srgba::new(0.26f32, 0.59f32, 0.98f32, 0.95f32),
        tab: Srgba::new(0.11f32, 0.15f32, 0.17f32, 1.00f32),
        tab_hovered: Srgba::new(0.26f32, 0.59f32, 0.98f32, 0.80f32),
        tab_active: Srgba::new(0.20f32, 0.25f32, 0.29f32, 1.00f32),
        tab_unfocused: Srgba::new(0.11f32, 0.15f32, 0.17f32, 1.00f32),
        tab_unfocused_active: Srgba::new(0.11f32, 0.15f32, 0.17f32, 1.00f32),
        plot_lines: Srgba::new(0.61f32, 0.61f32, 0.61f32, 1.00f32),
        plot_lines_hovered: Srgba::new(1.00f32, 0.43f32, 0.35f32, 1.00f32),
        plot_histogram: Srgba::new(0.90f32, 0.70f32, 0.00f32, 1.00f32),
        plot_histogram_hovered: Srgba::new(1.00f32, 0.60f32, 0.00f32, 1.00f32),
        text_selected_bg: Srgba::new(0.26f32, 0.59f32, 0.98f32, 0.35f32),
        drag_drop_target: Srgba::new(1.00f32, 1.00f32, 0.00f32, 0.90f32),
        nav_highlight: Srgba::new(0.26f32, 0.59f32, 0.98f32, 1.00f32),
        nav_windowing_highlight: Srgba::new(1.00f32, 1.00f32, 1.00f32, 0.70f32),
        nav_windowing_dim_bg: Srgba::new(0.80f32, 0.80f32, 0.80f32, 0.20f32),
        modal_window_dim_bg: Srgba::new(0.80f32, 0.80f32, 0.80f32, 0.35f32),
        docking_preview: Srgba::new(0.0, 0.0, 0.0, 1.0),
        docking_empty_bg: Srgba::new(0.0, 0.0, 0.0, 1.0),
    };
}
