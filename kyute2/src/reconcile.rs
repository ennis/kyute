use crate::{elem_node::TransformNode, AnyWidget, ChangeFlags, Element, Environment, TreeCtx, Widget, WidgetId};
use std::any::Any;

/// Helper function to update a list of child elements (`TransformNode<Box<dyn Element>>`), from
/// a list of widgets, matching the elements by ID.
///
/// # Return value
/// `ChangeFlags::STRUCTURE` if any element was added or removed, plus any change flags signalled by
/// widgets.
///
/// # Details
/// This summarizes the "reconciliation" algorithm:
/// - start with a cursor position to 0
/// - for each widget in `widgets`, try to find an element in `elements` with matching type and ID.
///     - If found, the element will be "rotated in place" at the cursor position.
///     - Otherwise, a new element will be built from the widget (with `Widget::build`), and inserted at the cursor position.
///     - Increment the cursor position
/// - when there's no more widgets to insert or update, `elements[pos..]` contains all remaining elements that did not have
///   a matching widget. These are deleted.
///
pub fn reconcile_elements<W, E, GetWidget, GetElement, Update, Build>(
    cx: &mut TreeCtx,
    // Vec<T> + Fn(&T) -> &Widget,
    widgets: Vec<W>,
    elements: &mut Vec<E>,
    env: &Environment,
    get_widget: GetWidget,
    get_element: GetElement,
    build: Build,
    update: Update,
) -> ChangeFlags
where
    GetWidget: Fn(&W) -> &dyn AnyWidget,
    GetElement: Fn(&mut E) -> &mut dyn Element,
    Build: Fn(&mut TreeCtx, W, &Environment) -> E,
    Update: Fn(&mut TreeCtx, W, &mut E, &Environment) -> ChangeFlags,
{
    let mut pos = 0;
    let mut change_flags = ChangeFlags::empty();
    for widget in widgets {
        // find element matching ID and type
        let id = get_widget(&widget).id();
        let element_type_id = get_widget(&widget).element_type_id();
        let found = elements[pos..].iter_mut().position(|elem| {
            get_element(elem).id() == id && Any::type_id(get_element(elem).as_any_mut()) == element_type_id
        });
        if let Some(found) = found {
            // rotate element in place
            elements[pos..].rotate_left(found);
            // and update it
            change_flags |= update(cx, widget, &mut elements[pos], env);
            pos += 1;
        } else {
            // insert new item
            elements.insert(pos, build(cx, widget, env));

            if id != WidgetId::ANONYMOUS {
                cx.child_added(id);
            }
            change_flags |= ChangeFlags::STRUCTURE;
            pos += 1;
        }
    }

    if pos < elements.len() {
        // there are elements to be removed
        for elem in &mut elements[pos..] {
            let id = get_element(elem).id();
            if id != WidgetId::ANONYMOUS {
                cx.child_removed(id);
            }
        }
        elements.truncate(pos);
        change_flags |= ChangeFlags::STRUCTURE;
    }

    change_flags
}
