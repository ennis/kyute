# TODO

- [x] Switch back to winit. It seems that Glazier (and Xilem) isn't receiving a lot of attention right now. Also, it has a complicated design with AppHandler/WinHandlers that necessitated a bunch of Rc/RefCell wrappers on the app state. I ended up with a RefCell BorrowMut error when trying to use an AppHandle within a `run_on_main` callback. I don't have time for puzzles. 
  - note that winit comes with its own share of pain: 0.29 broke many keyboard-related types (sometimes for no good reason), and crates like egui-winit haven't updated. So in the long term, replace winit by something else, probably a simpler windowing crate without support for android, web, wayland.
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

# Widgets

| Feature                     | Difficulty | Details                                                                               |
|-----------------------------|------------|---------------------------------------------------------------------------------------|
| ~~Grid layout~~             | DONE       |                                                                                       |
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
| Child windows               | ★★         |                                                                                       |
| Icon font support           | ★★         |                                                                                       |
| Filtered text input         | ★★         |                                                                                       |
| Split views                 | ★★         |                                                                                       |
| Image widget                | ★★         |                                                                                       |
| Combo box / pull down       | ★★         |                                                                                       |
| Single-line text editor     | ★★         |                                                                                       |
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
| Scroll views                | ★★★        |                                                                                       |
| Advanced tables             | ★★★        | Tables + reorderable rows, sorting, selection (possibly mergeable with outline views) |
| Color picker                | ★★★        |                                                                                       |
| Modal dialogs               | ★★★        |                                                                                       |
| Generic outline views       | ★★★        | Basically tree views, take inspiration from NSOutlineView                             |
| Timeline widget             | ★★★        |                                                                                       |
| Drop down with autocomplete | ★★★        |                                                                                       |
| Property grid               | ★★★        |                                                                                       |
| Rive integration            | ★★★        | https://rive.app/                                                                     |
| Multi-line text editor      | ★★★?       |                                                                                       |
| Curve editor                | ★★★★       | Animation curves                                                                      |
| Gradient editor             | ★★★★       | Color gradients                                                                       |
| Docking system              | ★★★★       |                                                                                       |
| Code editor                 | ★★★★       | Text editor with syntax highlighting                                                  |
| Charts                      | ★★★★       |                                                                                       |
| Web views                   | ★★★★       |                                                                                       |


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