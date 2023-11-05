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
- [ ] Debug window: visualize event propagation
- [ ] Debug window: visualize ChangeFlags
- [ ] Debug window: record events on timeline
- [ ] Debug window: show overflowing geometry 
- [ ] Debug utility: allocate everything in the recording buffer