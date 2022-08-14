# Pointer hover & hot widgets

## Terminology
- hot: a widget is hot if the pointer is hovering over it and there's no other widget on top of it
- hover set: for a window, the set of widgets that the pointer is currently hovering over
- pointer input event: all pointer events except `PointerOver/Out/Enter/Exit`
- pointer derived event: `PointerOver/Out/Enter/Exit` events

## Tracking

Hot, focus, and hovered widgets need to be tracked so that widgets can receive correct pointer-derived events.
This state is stored in `WindowState::focus_state` and `WindowState::hovered`.
Only named widgets (those with an ID) are tracked, since it's impossible to send a targeting event to an anonymous widget.

## The `hit_test_pass` flag
Widgets that perform hit-test can choose to ignore events that fail the hit-test. In this case, they should set the 
`EventCtx::hit_test_pass` flag to false to signal hit-test failure to the caller. 
The caller relies on that flag to determine whether the widget should be added or removed to the hover set, so it's crucial
that this flag is set properly.

By default, the flag is true, so widgets not performing any hit-test for pointer input events will be added to the hover set.
Currently, only `WidgetPod` does hit-testing because it's the only widget that can apply a transform to its child.
In the future, other widgets that apply a transform to their children will need to hit-test them and properly set the flag as well.

## Updating the hover set

See `core::do_event`. After propagating a pointer input event to a widget, if the widget has an ID:
- if the hit test passed (`target_ctx.hit_test_pass == true`), then add the widget to the hover set (update `window_state.hovered`)
  - additionally, if no widget has claimed the "hot" status for the event being propagated (`target_ctx.hot.is_none()`), then claim it, and update the hot widget (`window_state.focus_state.hot`)
- if the hit test failed, then remove the widget from the hover set

### Alternate design
Maybe propagate the hot & hovered set upwards, like FocusChange?

## Focus changes
They are propagated upwards during event propagation, in `EventCtx::focus_change`.

