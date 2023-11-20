# TODO

- [x] Switch back to winit. It seems that Glazier (and Xilem) isn't receiving a lot of attention right now. Also, it has a complicated design with AppHandler/WinHandlers that necessitated a bunch of Rc/RefCell wrappers on the app state. I ended up with a RefCell BorrowMut error when trying to use an AppHandle within a `run_on_main` callback. I don't have time for puzzles. 
  - note that winit comes with its own share of pain: 0.29 broke many keyboard-related types (sometimes for no good reason), and crates like egui-winit haven't updated. So in the long term, replace winit by something else, probably a simpler windowing crate without support for android, web, wayland.
  - raw_window_handle 0.6 has also changed APIs (gratuitously, handles were made !Copy) 
   
- [ ] Add cursor position to winit events (see https://github.com/rust-windowing/winit/pull/1289/files). There's no reason not to have them in winit since the OS provides them already, and it saves us the hassle of tracking it separately in our application.
- [ ] Re-introduce "modifiers" in winit KeyEvents (see https://github.com/rust-windowing/winit/issues/1824). The reasoning for the removal is weak, especially since winit tracks modifiers internally.
- [x] Find a way to make widget tree construction more reliable (i.e. ensure that the user doesn't forget to call child_added and child_removed in Widget::build and Widget::update)
  - `build_with_id`, `update_with_id`
- [ ] Don't use floating-point `Size` where it doesn't make sense: for example, to specify a window size or the size of a composition layer. Replace with (u32,u32) or equivalent.
- [x] Add a debug window rendered with egui. 
- [x] Debug window: show element tree
- [x] Debug window: visualize event propagation
  - elements affected by the event are painted in red
- [ ] Debug window: visualize ChangeFlags
- [x] Debug window: record events on timeline
- [ ] Debug window: show overflowing geometry 
- [x] Debug: collect event propagation information
- [x] Debug utility: allocate everything in the recording buffer
- [x] Debug window: highlight if selected or hovering
- [x] TransformNode: remove debug stuff. It's handled by `LayoutCtx` now for all widgets.
- [ ] TransformNode -> TransformBox
- [ ] Add a "OffsetBox" which is like TransformBox but only with a 2D offset
- [x] Fix WGPU/DX12 crash in debug mode
  - Used vulkan instead
- [ ] Grid: natural width/height
- [ ] Grid: styling (row/column separators & alternating row backgrounds)
- [ ] Debug: make the debug window work with multiple windows
- [ ] Focus & pointer capture state should be application states.
- [ ] Element tree should be application state.
- [ ] Store owning window IDs in element tree.


## Bugs
- [ ] Debug: highlight on hover is broken
- [ ] Child window flashes to transparent when opened
- [ ] Close window broken
 
# Widgets

| Feature                     | Difficulty | Details                                                                               |
|-----------------------------|------------|---------------------------------------------------------------------------------------|
| ~~Grid layout~~             | ★★ DONE    |                                                                                       |
| Relative layouts            | ★          | A modifier to position a widget above/below another widget, or to the left/right.     |
| Simple spinner (numeric)    | ★          |                                                                                       |
| Numeric Drags/Sliders       | ★          |                                                                                       |
| Disabled widgets            | ★          |                                                                                       |
| Separator                   | ★          |                                                                                       |
| Form layout                 | ★          |                                                                                       |
| Group box                   | ★          | aka "Box"                                                                             |
| Disclosure groups           | ★          | aka "Collapsible Group"                                                               |
| Checkbox                    | ★          |                                                                                       |
| Radio button                | ★          |                                                                                       |
| Password field              | ★          |                                                                                       |
| Custom painter widget       | ★          |                                                                                       |
| Toolbars                    | ★          |                                                                                       |
| Progress bar                | ★          |                                                                                       |
| Segmented controls          | ★          | https://developer.apple.com/design/human-interface-guidelines/segmented-controls      |
| Cursor modifier             | ★          | Changes the cursor that is displayed when hovering over the widget                    |
| Rich text macro             | ★★         |                                                                                       |
| Icon font support           | ★★         |                                                                                       |
| Single-line text input      | ★★         |                                                                                       |
| Filtered text input         | ★★         |                                                                                       |
| Split views                 | ★★         |                                                                                       |
| Image widget                | ★★         |                                                                                       |
| Combo box / pull down       | ★★         |                                                                                       |
| Tab navigation              | ★★         |                                                                                       |
| Tab view                    | ★★         |                                                                                       |
| File picker / std dialogs   | ★★         |                                                                                       |
| Testing infrastructure      | ★★         | Take screenshot of UI and compare with reference                                      |
| Path controls               | ★★         | https://developer.apple.com/design/human-interface-guidelines/path-controls           |
| Advanced numeric input      | ★★         | Text-input, drag, mouse wheel, multi-dimensional.                                     |
| Context menus               | ★★?        |                                                                                       |
| Main menu                   | ★★?        |                                                                                       |
| Drag-drop                   | ★★?        |                                                                                       |
| Tooltips                    | ★★?        |                                                                                       |
| Token fields                | ★★         | https://developer.apple.com/design/human-interface-guidelines/token-fields            |
| ~~Child/popup windows~~     | ★★★ DONE   |                                                                                       |
| Scroll views                | ★★★        |                                                                                       |
| Advanced tables             | ★★★        | Tables + reorderable rows, sorting, selection (possibly mergeable with outline views) |
| Color picker                | ★★★        |                                                                                       |
| Modal dialogs               | ★★★        |                                                                                       |
| Generic outline views       | ★★★        | Basically tree views, take inspiration from NSOutlineView                             |
| Timeline widget             | ★★★        |                                                                                       |
| Drop down with autocomplete | ★★★        |                                                                                       |
| Property grid               | ★★★        |                                                                                       |
| Rive integration            | ★★★★       | https://rive.app/                                                                     |
| Multi-line text input       | ★★★?       |                                                                                       |
| Curve editor                | ★★★★       | Animation curves                                                                      |
| Gradient editor             | ★★★★       | Color gradients                                                                       |
| Docking system              | ★★★★       |                                                                                       |
| Code editor                 | ★★★★       | Text editor with syntax highlighting                                                  |
| Charts                      | ★★★★       |                                                                                       |
| Web views                   | ★★★★       |                                                                                       |


## Child windows

Q: Do we want parent windows to be able to "intercept" the `WindowEvent`s destined to their child windows?
A: Not sure yet, no use case in mind

Design: child windows are "anchored" in the parent window UI tree:

    ChildWindow::new(..., is_opened, button("open the window"))

There's no "imperative" API to open/close/move child windows. 
Do it all declaratively? Yes, if something more complicated is needed, just reimplement Widget with a UiHostWindowHandler

## Popup windows

On windows at least, popup windows don't gain focus. They receive mouse events, but not keyboard events.
So any keyboard event that should be sent to a control in a popup should transit through the PopupWindowElement 
which will propagate it to the UI tree of the popup window (stored in the UiHostWindowHandler).

From the POV of the user, popup windows should be exactly like other UI elements, except that they can overlap 
other elements and "go outside" the bounds of the host window.

Q: what about focus state? 
The ID of the keyboard-focused element and the ID of the pointer capturing element are stored in the UiHostWindowHandler.
With popup windows, this assumption breaks down because there are two UiHostWindowHandler, but there should be only
one set of focused elements.
In other words, the parent & the child window should share the same state regarding focused elements.
Alternatively, this means that focus state (pointer capture, etc.) is not a window state, but rather an app-wide
state.
A: It's completely necessary to keep the focus state **per window**: we need to remember what text box/button/etc. had
the focus when switch back-and-forth between windows. However, there can be a concept of "focused widget" global to 
the application, which is defined as the focused widget within the currently active window.

Q: but then what about popups? Popup windows don't activate (for technical reasons: the popup parent shouldn't appear inactive)
but we still want to have the focus on elements inside the popup.
A: Popup window handlers should use the focus state of the parent window.




Solution:
Move focus state to the application.

~~Issue: ElementId is not enough to store the focused element, because ElementIds are relative to a parent window.
More precisely: ElementIds are unique across the whole application (by construction), but the ElementId alone doesn't
say to which window the element belongs (and is stored in).~~
~~Solutions:~~
~~1. An app-wide map ElementId -> WindowId to recover the window from the element ID~~
~~2. Store windowId alongside ElementId -> this doubles the size of the ElementId~~


Q: should pointer events targeting a widget in a popup "bubble up" to (and also be capturable by) the anchor widget 
in the parent window? This is tricky because window pointer events are received on the popup window,
and it's the handler for that window that propagates it to its UI subtree. The handler doesn't see the other windows.
If popup events bubble up, then the popup handler should communicate with the handler of the parent window to ask it to 
deliver the event.
A: Let's say that events targeting a child window don't bubble up to the parent window 
(bubbling stops at the UI root for the child window). This would tend to complicate things anyway. 




## Disabled widgets

The "disabled" state is stored in an ambient variable of type `WidgetState`.


## Relative layout

Start with `.above`, `.below` modifiers, such that `widget.above(other)` positions `widget` above `other`.
Then add `.left_of`, `right_of`, such that `widget.left_of(other)` positions `widget` to the left of `other`.
Take care to correctly propagate the baselines.

## Custom painter widget

Users shouldn't have to convert to skia types manually, so propose a thin wrapper over "Canvas" that does that.
It should be stateless.

## Single-line text editor

- Copy/paste
- Keyboard navigation

