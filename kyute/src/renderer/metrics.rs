use crate::renderer::{ButtonMetrics, WidgetMetrics};

pub(super) const WIDGET_METRICS: WidgetMetrics = WidgetMetrics {
    button_metrics: ButtonMetrics {
        min_width: 10.0,
        min_height: 10.0,
        label_padding: 4.0,
    },
};
