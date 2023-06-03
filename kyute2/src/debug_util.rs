//! Debugging utilities

#[derive(Clone, Debug)]
pub struct DebugWidgetTreeNode {
    pub name: String,
    pub debug_node: DebugNode,
    pub id: Option<WidgetId>,
    pub cached_layout: Option<Geometry>,
    pub transform: Option<Transform>,
    pub children: Vec<DebugWidgetTreeNode>,
}

impl DebugWidgetTreeNode {
    /// Try to extract the base widget type name (e.g. `Container` in `kyute::widgets::Container<...>`).
    pub fn base_type_name(&self) -> &str {
        let first_angle_bracket = self.name.find('<');
        let last_double_colon = if let Some(p) = first_angle_bracket {
            self.name[0..p].rfind("::").map(|p| p + 2)
        } else {
            self.name.rfind("::").map(|p| p + 2)
        };
        &self.name[last_double_colon.unwrap_or(0)..first_angle_bracket.unwrap_or(self.name.len())]
    }
}

pub(crate) fn get_debug_widget_tree<W: Widget>(w: &W) -> DebugWidgetTreeNode {
    let mut nodes = Vec::new();
    send_utility_event(
        w,
        &mut Event::Internal(InternalEvent::DumpTree { nodes: &mut nodes }),
        &Environment::default(),
    );
    assert_eq!(nodes.len(), 1);
    nodes.into_iter().next().unwrap()
}

pub(crate) fn dump_widget_tree_rec(node: &DebugWidgetTreeNode, indent: usize, lines: &mut Vec<usize>, is_last: bool) {
    let mut pad = vec![' '; indent];
    for &p in lines.iter() {
        pad[p] = '│';
    }

    let mut msg: String = pad.into_iter().collect();
    msg += &format!("{}{}", if is_last { "└" } else { "├" }, node.base_type_name());
    if let Some(id) = node.id {
        msg += &format!("({:?})", id);
    }
    if let Some(ref content) = node.debug_node.content {
        msg += "  `";
        msg += content;
        msg += "`";
    }
    println!("{}", msg);

    if !is_last {
        lines.push(indent);
    }

    for (i, n) in node.children.iter().enumerate() {
        if i == node.children.len() - 1 {
            dump_widget_tree_rec(n, indent + 2, lines, true);
        } else {
            dump_widget_tree_rec(n, indent + 2, lines, false);
        }
    }

    if !is_last {
        lines.pop();
    }
}

pub(crate) fn dump_widget_tree<W: Widget>(w: &W) {
    let node = get_debug_widget_tree(w);
    dump_widget_tree_rec(&node, 0, &mut Vec::new(), true);
}
