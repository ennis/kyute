
## Visual layer

For fast animations that do not require a full recomp on each frame, introduce a visual layer.
Possibly using the underlying composition framework (Core Animation, Windows.UI.Composition).

### Basic ideas
~~- the `layout` method now returns `Layer` elements (which can contain sublayers). Remove the `paint` method.~~
-> the `layout` methods still return Measurements, but also *animate* the widget's layer.
-> add a `layer` method to Widget that returns the animation layer of the widget. For wrappers, defer to the inner widget.
~~-> actually, do we still need a layer object? Just use methods on LayoutCtx to animate the "current" layer~~

- `Layer` elements have properties that can be animated somehow.
    - Common properties: transform (position, rotation, scale), opacity

### Questions
How to expose the layer hierarchy to the application? Is it immutable? Do we rebuild it from scratch everytime, with some kind of caching?
We need some caching because there are retained objects behind the tree (composition objects provided by the OS, shouldn't rebuild them from scratch all the time).

Promising approach: leverage the positional cache, and stash layers in it.
e.g. `Container::new` would retrieve (get-or-create) a layer with `cache::state(|| Layer::new())`.

Layers would have interior mutability: i.e. can call `set_width`, `set_height` on them, and they would still be considered to be the same object.


Adding sublayers during layout: `layer.add_child(...)`.
Problem with that: we also have to remove sublayers of child widgets that have been deleted.
It's easy to retain references to layers of widgets that have been deleted.
=> It's a feature: we may want to animate added/moved/removed children.

Properties can be changed from another thread: this could be useful for animations.
However, calling `set_<property>`
doesn't change the value of the property immediately
(that would involve locking a mutex and traversing the tree to mark nodes dirty).
Instead, it posts the new value of the property to the event loop, which will then perform the layer tree update
before paint, where it has exclusive access to it.


### Layer animations
- Go straight to the compositor API in most cases
- Otherwise (when no system compositor is available):
    - add the animation object to some global compositor queue
    - wake the event loop (if from another thread)
    - after layer eval, delete layers that are not reachable from the root
        - problem: we lose state by doing that
            - layer deletion should be tied to the deletion of the layer ref in the positional cache


### Lifecycle of Layers

`Layer::new(delegate: LayerDelegate)`: the delegate is in charge of drawing the contents of the layer

```rust
fn layout(&self) {

    // perform child layout, collect layers
    let child_layers = ...;

    // position child layers
    for l in child_layers.iter_mut() {
        // modifies the transform of the layer
        // copy-on-write?
        l.set_transform(...);
    }


}
```

### Animatable properties

#### Starting animations

### Case study: viewport

```rust
impl Widget for Viewport {
    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) {

        let mut child_constraints = constraints;
        if !self.constrain_width {
            child_constraints.min.width = 0.0;
            child_constraints.max.width = f64::INFINITY;
        }
        if !self.constrain_height {
            child_constraints.min.height = 0.0;
            child_constraints.max.height = f64::INFINITY;
        }

        // if transform changes, then layout() is called


        // layout contents
        // -> calls canvas.layout()
        // -> calls set_size on the visual, which does nothing since it hasn't changed
        // -> removes all visuals, adds canvas content
        //      this triggers a repaint (maybe)
        //      if we have a painted element that hasn't changed (e.g. a circle)
        //      1. add the circle layer to the canvas, circle widget updates the content of its visual
        //      2. canvas needs to be repainted
        self.contents.layout(ctx, child_constraints, env);

        let contents_visual = self.contents.visual();
        // create a surface for the contents
        contents_visual.make_surface_backed();
        self.visual.add(contents_visual);


        // unconstrained
        self.contents.set_transform(self.transform);

        // always take the maximum available space
        let width = constraints.finite_max_width().unwrap_or(0.0);
        let height = constraints.finite_max_height().unwrap_or(0.0);
        Measurements::from(Size::new(width, height))
    }
}
```



### Avoiding a shared interior-mutable tree
Alternatives:
- store layers in a slotmap, `Layer` is just `(Arc<Compositor>, LayerIndex)`, layers are internally refcounted
- pass around `&mut CompositionTree`, mutate it with IDs
    - problem: layers must be removed manually
        - should be removed when all layer references have dropped
            - layers have shared ownership
- Garbage-collect orphaned layers during update
    - easy to stash a layer ID and forget to

Problem: if the layer tree is immutable, then we must rebuild it on every event (an event that starts an animation
would have to be followed by a layout).
-> hence, must be an "imperative, mutable" kind of API

### Examples of GUI frameworks with compositing layers
- JavaFX? Not exposed through the API, not sure if it uses a compositor
- Flutter? RenderObjects, owned by widgets (via "elements"), dropped on unmount

### Issue: duplicated widget bounds
- Need to set the widget position in WidgetPod::offset AND in the widget's visual layer
- The visual layer should contain the truth (offset & bounds)
- But what about animations?
    - It's possible to animate the position of a layer; when a layer is animating, what bounds do we use for hit-testing?
    - alternatively: what value do we read back for the position when it's animating?
        - the *current* position? no way to get that when an animation is in progress (DirectComposition doesn't provide a way to read back values)
        - the *target* position?


## Layers during painting


```rust

struct RenderLayer {
    id: WidgetId,
    layer: Layer,
    dirty: Cell<bool>,
    contents: impl Widget
}

impl Widget for RenderLayer {
    fn paint(&self, ctx: &mut PaintCtx) {
        if self.dirty.get() {
            // must redraw
            // begin a new layer, using the specified layer ID
            // may reuse old comp layer with the same ID 
            ctx.layer(&self.layer, ...);
        } else {
            // reuse 
            ctx.add_layer(&self.layer);
        }
    }
}

// Issue:
// Parent
// - Child A (direct draw)
// - Child B (Layered)
// - Child C (direct draw)
// For correct rendering, the layered child must be "pasted" (and thus child A and C must be rendered as well)
//      OR A and C must be layered as well
//
// paint(parent)
//  -> child A: direct paint on parent layer
//  -> child B: start layer, push it on parent
//  -> child C: start another layer (implicit layer), push it on parent
// 
// Problem: animating a layer
//  e.g. animating B: 
//  -> will repaint A, but it's not needed
//  
// Animating a layer == setting offset on the RenderLayer
//  -> and then, call `request_compositing()`: to tell that a layer property has changed


// when a RenderLayer receives a RenderLayerRequest, it builds a PaintCtx on its layer, and repaints it.

impl<Content: Widget> Widget for Container<Content> {
    fn paint(&self, ctx: &mut PaintCtx) {

        // paint here


        ctx.layer(&self.layer_id, |ctx| {
            // paint child (if necessary)
            // how do we know if it's necessary?
            // 1. a child might have requested a repaint (ctx.request_repaint()), which sets a bit in the closest parent comp layer
            // how do we reach this place?
            // 1. we know the ID of the widget with the dirty layer, so send it here
        })
    }
}

```


## Formalizing relayout

Relayout is the process of recalculating the size of widgets under new constraints, and placing child widgets.
It may happen because:
- a widget explicitly requested a relayout during event handling (by calling `ctx.request_relayout`)
    - in which case, the `layout` method *will* be called at some point in the future on the widget that requested the update
- an external factor influencing the layout has changed: this includes the _box constraints_ and the _scale factor_.
    - Typically, this relayout is triggered by the parent window when it is resized.

By default, the only retained state modified by the layout process is the offset of child widgets, which is typically managed by the `WidgetPod` wrapper.
However, it's important to cache the calculation of subtrees if they are known to never change. This is also done in
`WidgetPod`: if the box constraints & scale factor haven't changed, then it returns the previously computed measurements, otherwise
it calls `layout` on the child. In the event that a child widget called `request_layout` during propagation, `WidgetPod` invalidates
its cached measurements, so `layout` will always be called.

Currently, a layout is always followed by a repaint: this is because `LayerWidget`
(which manages the composition layers on which the widgets are drawn, and which are in charge of repainting),
schedules a repaint if it's cached layout is invalidated.

Relayout is closely related to repaint: usually, calling `layout` on a widget is usually followed by a repaint.

## Next up: 3D layers

For 3D content, create a layered WidgetPod.
Add a new function to Widget, called `layer_paint`, which gets a native composition surface as input. Default impl
creates the corresponding skia surface and calls paint. 3D views override this and can present stuff as they like,
with whatever API.

## TODO
- Embed images in crate
- Pull-down buttons (https://developer.apple.com/design/human-interface-guidelines/macos/buttons/pull-down-buttons/)
- Checkboxes
- Radio buttons
- Boxes (https://developer.apple.com/design/human-interface-guidelines/macos/windows-and-views/boxes/)
- Tab views
- Disclosure triangles (a.k.a. "Titled pane"?)
- Popups/popovers
- Integrated toolbars
- Colorize images
- Fix LinearGradient build code

## Rethink grids
They are very flexible, but the API is not very ergonomic.
A big issue is the lack of immediate feedback. To solve this, create an interactive grid designer. Or at least, some kind of live reload.


## Different backgrounds
- Window default
    - Supposed to put form controls on it
    - Boxes background is just an overlay
- Toolbar
- Sidebar
- Content background
    - for tables, edit boxes, etc.
    - also: alternative content background

## The necessity for an interface designer
The edit/compile/check cycle is long and tedious: adjusting the size of an element takes >30sec.
It needs to be faster if we want the UI creation process to be pleasant.

There are several solutions to that:
- reduce compile times: not really possible
- hot-reload rust code: same, not really possible
- separate structure from styling and hot-reload styling information separately (a.k.a. the CSS way)
    - has a non-negligible impact on the API
- use an interface designer

Unfortunately, creating a visual interface designer from scratch is a huge project. However, we could start with
a small hot-reloadable DSL to quickly prototype interfaces.

### Another possibility: ad-hoc variables

For instance:
```rust
#[composable]
pub fn new() -> Toolbar {
    let mut grid = Grid::new();
    grid.push_row_definition(
        GridTrackDefinition::new(
            tweak("Toolbar icon row height", GridLength::Fixed(45.dip()))
        )
    );
    grid.set_row_gap(tweak("Toolbar icon-text gap", 5.dip()));
    grid.push_row_definition(GridTrackDefinition::new(tweak("Toolbar text row height", GridLength::Fixed(20.dip()))));
    grid.set_column_gap(tweak(10.dip()));
    grid.set_column_template(GridLength::Fixed(80.dip()));
    let inner = Container::new(grid)
        .background(Paint::from(
            LinearGradient::new()
                .angle(90.degrees())
                .stop(Color::from_hex("#D7D5D7"), 0.0)
                .stop(Color::from_hex("#F6F5F6"), 1.0),
        ))
        .content_padding(10.dip(), 10.dip(), 10.dip(), 10.dip())
        .centered();
    Toolbar { inner }
}
```

## Grid ad-hoc syntax
- rows/columns
- track names
- template
- gap size
- units
- area

### Option A
```
// anonymous tracks
"R(g=5px):200,200,1*,auto;C:[40px]"
// named tracks
"C(g=5):name(min=200,max=300)/type=200dip/value=1fr;R:[auto]"
// Area (row 3 col 3)
"3/3"
// rows 3-6 all cols
"3-6/*" 
```

### Option B: CSS grid
```
// named track lines
"[name] 200 [type] 200 [value] 1fr / [header] 6em {4em} [rows-end] / 5px 5px"
 
// anonymous track lines
"200 200 1fr / {4em} / 5dip"


// rows 3..end, cols 3..end
"3.. / 3.. "
// entire grid
"../.."
// row 3, col 3
"3/3"
// past-the-end row, name column
rows-end / name

```

## Paint & border syntax
CSS-like:

```
fill = "linear-gradient(...)"
fill = "url(...)"
fill = "#124522";

border = "1ppx outside" 
```

BoxStyle::parse:

```
sfdsd {
  background: linear-gradient($grey-800, 
  border: 
}
```

## Removing EnvRef
I don't like it. It forces us to defer resolving things like styles to layout.

The main use case for dynamic environment values are things like disabled widget trees.
=> replacement: widget state flag

Also: changing the font of a subtree.
Alternative? style inheritance

# Core data framework
- Undo/redo
- Fast collection diffs
- Persistence abstracted away

## TODO
- `#[composable(tweak_literals)]`
- more robust tweak macro (span fixup)

## General CSS support?
Style => a container for style properties. Like environments, can inherit from a parent style.
Style value resolution: cached?
Fast lookup of properties.

```rust

struct StyleImpl {
    // hashmap of properties (imbl::HashMap)
}

pub struct Style(imbl::HashMap<Property, PropertyValue>);

// style cascade done in layout


```

Issue with alignment:
- CSS alignment on an element specifies the alignment of the element _in its parent_.
- the alignment property on our containers specifies the alignment of _the contents inside the container_

In CSS, positioning properties are specified on the positioned element.

Due to our layout algorithm, we can't really do the same thing as CSS: we would need to propagate the alignment upwards during layout.
It is possible, though:
- replace `Measurements` with a proper `Layout` struct, containing:
    - the size
    - clip bounds
    - alignment within the parent container (grid area, container)
        - problem: all elements (text, etc) would need to carry an alignment property
        - more generally, it could return positioning information instead
            - e.g. relative(top,left)

```rust
struct Layout {
    size: Size,
    // positioning properties
    // if none of those are specified, positioned by the parent element
    left: Option<Length>,
    right: Option<Length>,
    top: Option<Length>,
    bottom: Option<Length>,
    // alignment properties

    // might as well return the computed style of the element...

    // alignment 
    //align: 
}
```

## Formalized containers
You have a widget, which may or may not draw something, and may or may not fill its provided space.
Use a container to force a specific size, align it within the provided space

Current problem: some methods (e.g. "align" or "padding") have different implementations:
- one as an extension trait on widgets
- the other as a method on containers
  They can have subtly different behaviors.
  Instead: proper widget modifiers
  -> WidgetAndModifiers<W, (Modifiers...)>
  -> derefs to W
  -> can look for a particular type in modifiers
  trait ModifiedWidget
  type Modifiers
  fn modifier(&self) -> Option<T>
  Accessing the modifiers
  -> containers now take impl ModifiedWidget
  -> problem: some modifiers generate wrapper widgets, others don't
  -> just require the widget to support the modifier?
  -> no, too much work on behalf of the widget implementor
  -> current widget impls shouldn't change too much

list of modifiers:
* .grid_row_span
* .grid_row
* .grid_column
* .grid_column_span
* .grid_area
* .clickable
* .style
* .background
* .font_size
* .text_color
* .border
* .border_radius
* .z_index
* .min_width
* .min_height
* .max_width
* .max_height
* .overlay

e.g.

    Rectangle::new()
        .grid_column_span(2)    // GridPositioning<>
        .min_width(100)     // Constrained<>
        .max_width(200)     // Constrained<>
        .background("linear-gradient(...)") // Style<>
        .align(...)         // Align<>
        .border(...)        // Is it affected by min-width and max-width?
        .font_size(10.dip)  // Sets the font size for all child elements
        .clickable()
    
    -> Clickable<Border<Align<Style<GridPositionModifier<Rectangle>>>>>
    -> Widget::layout_properties() -> returns Layout with grid position (and alignment?)

## Ambiguities?

    .padding(4).align(right).border(...)

VS

    .align(right).border(...).padding(4)

In a 500x500 fixed size box.

`.align` doesn't do anything on the widget until it's inserted in a container.

Other example:
    
    .max_width(50%).border().padding(40px)    // a box with a border around it, sized to 50% of the available space after padding
    .padding(40px).border().max_width(50%)    // the element, padded 50px, with a border around it, the whole box sized to 50% of the available space

    .max_width(50%).align(bottom-right).border().padding(40px)  //  
    .max_width(50%).border().align(bottom-right).padding(40px)  // same result (alignment "passes through" borders to the nearest enclosing container)

    .max_width(50%).top(5).border().padding(40px)  //  
    .max_width(50%).border().top(5).padding(40px)  // same result (anchoring passes through to the enclosing container)

    .width(3em).font_size(10)  //  width = 3em of 10dip
    .font_size(10).width(3em)  //  width = 3em of the parent font size

    .padding(50%).grid_column(1)  // padding = 50% of the size of grid column 1
            // problem: padding is evaluated during layout_params(), which doesn't know the size of column yet
            // layout_params() can be called again with different constraints, though
            // 

Commutativity (same result if the modifiers are switched):
.border <> .align
.{min/max}_width <> .padding .align



How does alignment work?
E.g. in the previous example, is the border drawn around the whole available space in which the rectangle is placed (because of align)
or only around the rectangle?
- Arguably, the least surprising behavior would be around the rectangle (align comes after).

Alignment mechanism:
- match position on unit rectangle


## Backgrounds, shapes, borders, etc.

What has been decided so far:
- in order to "style" an element, apply a modifier `Background` on it, which will draw stuff behind the element
- provide a `Rectangle` (possibly rounded) shape widget to be used for simple backgrounds.
- there are also `StyledBoxes`, which draw box decorations around a content element, but also handle the layout of the content within
  - `StyledBox` should stay

There's some duplication:
- borders are added by `StyledBox`, `Border` widgets, and `Rectangle` widgets.
  - there's duplicated code in all of those related to the computation of final border radii.
- we could remove borders from `Rectangle`, but we'd still need to keep the radii of the rectangle, which should be in sync
  with the radii of the border around it:

```
    widget.background(
        Rectangle::new()
            .radius(4.px()) // this length ...
            .paint(...)
            .border(4.px()))  // ... and this length must match!
```

Proposition:
- don't add borders to the rectangle shape widget, but make it so that border widgets push a clip mask
  - this way, to round a rectangle, simply add a rounded border to it

Alternative:
- keep border in rectangle, make it a "stroke style" 
  - problem: the stroke size wouldn't be taken into account  

Underlying question: do we emphasize the _shape_ (A) (rectangle, paths, etc.) or do we emphasize the _content_ (B) (text)
A: widgets are visual primitives, like rectangles, rounded rectangles, paths, text elements, etc. They are composed via overlays.
```
  Rectangle::new().fill(...).radius(4.px()).overlay(Text::new("hello"))
```
B: widgets are either content containers (text) or decorations around content. 
```
  Container::new(Null)
      .fill("rgb(255 255 255 / 30)")
      .border("4px solid blue")       // order-dependent: putting the fill after will fill the whole rectangle
```

Prefer B, that's what we started with, and what compose is doing.
What about drop shadows?

```
  Container::new(Null)
      .fill("rgb(255 255 255 / 30)")
      .border("4px solid blue")       // draw border and clip  
      .shadow("10px 5px 5px black")   // ??? for now, specify shape explicitly
```

# PointerOver / PointerOut events
Those got lost along the way. 

Proposed implementation: in "focus_state", keep a "hot" widget ID. Whenever _a widget successfully passes the hit-test_,
update the hot ID to this widget ID (this includes setting the hot ID to None if the widget has no ID).
The window that emitted the event then compares the previous and the new hot widget IDs. If they are different, a PointerOut
event is sent to the old widget ID (if not None), and a PointerOver event is sent to the new ID (if not None).

Problem: what does "successfully passing the hit-test" means? 
Hit-testing is only done in `WidgetPod` => PointerOver/PointerOut events will be received by all with the same ID.
It's confusing: if we have WidgetPod -> Padding(40px) -> CustomWidget, the custom will receive PointerOver events when the
cursor enters the **padding area**, and not the actual widget.
=> actually no, since within a frame, the inner widget is wrapped in a WidgetPod (that's the only mechanism for transforming child widgets)
=> Add a WidgetPod in "frame" widgets (that's already done)

```
└Window(9700B9170AE22AE2)  `title: "Counter demo"`
  └WidgetPod(1C6032094245E487)  `native layer 109x67 px`
    └Overlay(1C6032094245E487)
      └Grid(1C6032094245E487)  `0 by 0 grid`
        ├WidgetPod(FC0D06BD03C8099C)
        │ └Clickable(FC0D06BD03C8099C)
        │   └StyledBox
        │     └WidgetPod
        │       └Label
        │         └Text  `plain text: "-"`
        ├WidgetPod(4FD90939EAE70F73)
        │ └Clickable(4FD90939EAE70F73)
        │   └StyledBox
        │     └WidgetPod
        │       └Label
        │         └Text  `plain text: "+"`
        └WidgetPod
          └Text  `plain text: "Counter value: 1"`
```

# Tab navigation

Declare some widgets as tab-focusable. For the tab order, use the "logical sequence" => grid insertion order. 

A widget is tab-focusable if it accepts SetFocus events

On tab:
- send keyboard event to target
- target calls `ctx.move_focus()`
- event `Event::MoveFocus` is sent to the focused target
- bubbles down to the target
  - target doesn't handle it, bubbles up
  - eventually, bubbles up to the parent container
    - parent container sets the focus on the prev/next element (dispatch Event::SetFocus(direction) on children)
    - if no prev/next element: MoveFocus bubbles up to parent container

Alternative:
- event return values:
  - handled
  - focus move

- widget calls `ctx.move_focus`
  - route_event sees this result, marks parent widget 

Right now event return values are "stateful" => stashed in context.


Alternative:
- instead of juggling events, build the focus chain on recomp 
  - InternalEvent::BuildFocusChain { focus_chain: &mut FocusChain }
  - which widget adds to the focus chain?
    - clickables
    - editors
    - all widgets with an ID?

How does the widget adds itself to the focus chain?
- handles an event?
- overrides 

- problem: full tree traversal on each recomp
  - caching?
=> just do that, it's simple, easy to implement, flexible
  - makes recomp (potentially) costly
  - however, the event propagation code is already complicated enough as is, not much room for more


# Event propagation results
We want the widget that propagate the event to be able to intercept the result of event delivery:
- if the event was handled by a descendant widget
- whether a focus change was requested
- dirty regions / repaint requests
- relayout requests
- widgets that passed the hit-test

In `Widget::event`, EventCtx receives the return value, in a way.
Q: Not sure why it's preferred over actually returning a `EventResult` object?
A: Because with a return value, container/layout widgets need to merge the result manually; with a &mut-parameter, it's implicit, no additional code needed.

=> EventCtx collects the event result, route_event merges it with the parent context.
=> problem: if we add a vec to EventResult, lots of allocations for vectors that hold successful hit-tests

# Debugging event propagation
It can be difficult to understand how events propagate => debug visualization

# Should `Window` really be a widget?
Because of that, we're forced to have a dummy root widget and a bunch of `expect`s in EventCtx to account for this **unique** dummy root.
Alternatives:
- instead of a single root widget, store a list of root windows.

# Hit-testing on a separate tree?
Or rather: hit-testing in a separate tree traversal?
This might be necessary: consider the case of drag-and-drop.
The user clicks on the source widget, drags it towards the target.
Since the source widget captures pointer events, the target receives nothing, and can't react when the object is dragged into it.
Proposition:
```
InternalEvent::HitTest {
  hovered: &mut HashSet<WidgetId>,
  hot: Option<WidgetId>
}
```
This event is solely handled by WidgetPods. Before sending a pointer event, send a hit-test request and send the event to the hot widget.
(only if necessary: if the pointer position did not change, don't update)

# Definitive behavior for pointer events
- PointerMove:
  - deliver to root
  - WidgetPods do hit-test and stop propagation if outside the bounds

# Accessing inner widgets
FIXME: it can be difficult to access the inner widget when it is buried under several modifiers
It's a common pattern: provide a widget with the base functionality, without the style,
then provide a styled widget that wraps the base with style modifiers.
The styled widget needs to forward methods to the base, and this can be difficult (i.e. lots of `.inner()`)
It also makes it difficult to change the style by adding/removing modifiers because then you
have to also modify all the method wrappers (add/remove .inner() as needed).


Alternative proposal: modifiers implement Deref<Target=Widget>, inner widget is a TAIT like `impl Widget + Deref<Target=BaseWidget>`
Problem: this only works for one level of deref

Proposal: `Modified` trait, like Iterator:
```rust
pub trait Modified {
    type Inner = 
}
```

# When to access the environment?
Example: light mode / dark mode switch.

During composition, or during layout? Right now we have both, and it's confusing.

- During composition
  - Env override scope is tied to function calls: must wrap composable call in a lambda

- During layout
  - Env override scope is tied to the widget tree itself: preferred


# Styles with pseudo-class dependencies

Who is in charge of tracking the widget state? (focus, hover, active, disabled) 
How is it propagated to child widgets?
Should we avoid recomposition?

## Disabled

This code should work:
```
let widget = MyWidget::new().style("[if disabled] opacity: 50%;").disabled(true)
```

## Focus
Focus is tracked by the framework, but only widgets that call `ctx.request_focus` can be focused.

=> currently, FocusGained, FocusLost is propagated to child widgets, so widgets with the same ID also receive it
    but the styledbox doesn't have the same ID... it has the ID of its contents

## Active
It is the responsibility of the widget to set the active state.
No event is propagated when a parent widget turns active, so that's not an option. 

## Hover
Tracked by the framework, but not directly exposed. Widgets that have hover behavior should respond to PointerEnter/Exit/Over/Out events.
Should StyledBox have _hover behavior_ by default? Yes.

=> can be handled locally in StyledBox by handling the pointer events.


# Invalidating layout during pointer handling

That's an issue, because we can send multiple Pointer events (PointerEnter/PointerOver) without a relayout between.
E.g. send a PointerOver to a widget that invalidates its layout, and just after a pointer over, but the layout is now invalid.
=> solution: don't remove the old layout after invalidation

# Sidebars

MacOS-like:
- sections
- hierarchy

# Form layouts

Example user code:


```
let mut form = Form::new();

// push(name, widget)
form.push("Diffuse color", ColorPicker::new(color).on_color_changed(|c| *color = c)); 

// alternative

FormBuilder::new()
  .checkbox("Keep position when parenting", &mut value)
  .rgb_numeric_input("Translate", &mut translation)
  .rgb_numeric_input("Rotate", &mut rotation);
  
// with extension traits on FormBuilder


// Alternative
form.push(Labeled::new("Stuff", Checkbox::new(...)))

```

- collapsible sections
- automatically generate text
- labeled widgets?
  - `Checkbox::new(label: &str) -> Labeled<Checkbox>`
  - `Checkbox::unlabeled() -> Checkbox`


Q: is the label tied to the widget? or specified separately?

=> Collect use cases:
- label: static element
- label: dropdown
- label: collection of radio choices (multiple rows)
- label: text input
- label: checkbox (usually rendered as `[Checkbox] Label`, so the opposite of other inputs)

=> Use the same mechanism in other places? like toolbars?

Accessibility?

Main issue: specific layout behavior for some widgets.
E.g. checkboxes with the label on the other side.

## Option A: FormEntry trait
A trait implemented by things (widgets, etc.) that represent an entry in a form.
Through implementations of this trait, form entry widgets can insert themselves into a form, in the way best suited to
the widget type.

Pros:
- different layout behaviors for some widgets (e.g. checkboxes)

Cons: 
- must be implemented for *all* widgets (that is, until specialization lands)

### Suboption A.1: LabeledContent
More general than FormEntry, LabeledContent represents some content associated with a text label.
It has no inherent layout (it's not a widget), but is used by several widgets (forms, toolbars)
as their element type. => See SwiftUI LabeledContent


### Suboption A.2: widgets with built-in labels, and LabeledContent for the rest
There's a FormRow trait, blanked-implemented for all LabeledContent. Some widgets
directly implement FormEntry, like "toggles" (Checkbox+Label)

Basically two kinds of input widgets:
- "naked" widgets for which you need to provide a label, via `.labeled`
- labeled widgets, which implement LabeledContent


## Option B: extension traits on FormBuilder
All widgets that 

# Formatted text extension trait
So that users can do `text.font_style()`, with `text: impl Into<Arc<str>>`


# BUG: invalidating cached stuff during speculative layouts

The situation: 

Grid launches a speculative layout on an element to compute max track sizes. This invokes WidgetPod::layout, which in turn invokes StyledBox::layout. 
In addition to computing the layout, StyledBox::layout also computes and caches the CSS styles of the box. Currently, it *always*
invalidates (deletes) any previously computed styles (i.e. no caching).
However, since we're in a speculative layout, `LayoutCache::update` doesn't store the result.

Now, the grid launches the final layout. This invokes WidgetPod::layout, **but** WidgetPod has a valid cached layout, so it doesn't invoke StyledBox::layout.
Then, painting occurs, but StyledBox doesn't have the computed styles => crash.

There's a slightly misleading promise here: that a call to `paint` is always preceded by a call to `layout`. This is true, but
a **speculative** call to `layout` may happen between those. 
=> Conclusion: Widgets shouldn't invalidate cached results during speculative calls

The rules here are getting very confusing, and not even enforced by the compiler.
Ideally, there would be a way to pass data from `layout` to `paint` in a type-safe way.

Idea: `layout` returns a paint closure.
Problem: no control over how children are drawn.
Solution: child paint closures are moved into the closure.

Q: What about caching?
A: 

Q: overhead? this allocates yet another tree


# Drawing stuff

Like, e.g. the check box mark.
1. use a custom font
2. load & draw a PNG image
3. load & draw a SVG image
4. hardcode in rust

SVG spec too big. Alternatives:
* IconVG
* SVG native
* Haiku Vector Icon Format
* TinyVG
* Android vector drawables

Possible path forward: SVG native importer to `VectorImage` type (styles & paths).
roxmltree for the base SVG.

But how to generative SVG native?
-> svgomg
Just parse a SVG subset (minisvg) without css and stuff

In code: a fun and compact way of drawing dynamic icons, gauges, progress bars, etc.

-> minimal parametric vector drawing language that can reference variables from the environment

* rect
* path
* arc
* transform
* replicate
* randomize

# TODO: a simple layout to place two elements relative to each other, simpler than grid

e.g. 

```
// place label to the right of the content
label.to_right(content, VerticalAlignment)
// place label to the left of the content
label.to_left(content, VerticalAlignment)

right
left
above
below
over
under

// VStack
item1.above(item2).above(item3).above(item3)

// If feeling adventurous, implement an operator


```

Q: how to interpret vertical alignment with .above and .below modifiers,
   and horizontal alignment with .right and .left?
A: it is ignored
A': it is overwritten by the layout. However, instead of being interpreted as a position relative to edges of a containing box, 
    it's interpreted as a position relative to a line separating the A & B (horizontal for .above/.below, vertical for .right/.left).
E.g. with .above/.below: HorizontalAlignment::Relative(0.0) aligns the top edge of A to the separating line.
In a way it's similar to positioning within a containing box, except that the containing box is now a degenerate horizontal or vertical line (and doesn't contain the widgets at all). 


# Dynamic vector drawables

```rust

// access variables in env, but no conditionals
// variants (filled, not filled, etc)

// A vector drawing, with configurable variants.
// Variants are like "features" that can be enabled or not.
//
// Examples of features:
// - dark mode
// - 
// 
// Inside, drawing is represented as a series of operations, predicated on enabled variants
// Additionally, there are variables (floats & colors) that can be overriden.


const GAUGE: VectorDrawable = VectorDrawable {
    variants: &[
        Variant { n: "dark" },
        Variant { n: "light" },
    ],
    scalars: &[
      "gauge-value"
    ],
    colors: &[
        "gauge-color"
    ],
    paints: &[
        Paint::Color(Color::Ref(0))
    ],
    shapes: &[
        // paths go here
        Shape::Arc { .. },
        Shape::Path { .. },
    ],
    ops: &[
        Op::Fill { v: Some(VARIANT_DARK), s: 0, p: 1 }, 
        Op::Fill { v: None, s: 0, p: 0 }
    ]
};

```


# Writing modes, block flow directions, grids, etc.
Out of scope for UIs?



# Ideas/requirements for a data model

Requirements:
- no serialization code by hand except for tricky cases
- serialize to whatever
- ordered collections, works well with UI
- undo/redo
- objects cheap to copy 

Design:
- difficult to access objects directly; instead, functions (in the GUI) receive a `ModelObject<T>`. Which is like a smart pointer around an object of the data model.
- underlying structure is abstracted
- ModelObjects are value types: they can be cloned, and compared
- However, ModelObjects represent not a free standing value, but a value in a document.



# Next steps:
* Fix premult alpha on composited surfaces
* rework Layer API
  * remove `animation` module in kyute-shell (we won't be using that for animation)
  * layers (and their swap chains) will be owned by specific widgets
  * widgets paint to their swap chains when they want (usually during paint, but maybe as a result of a timer event)
    * widgets signal a native layer update by setting a flag in the EventCtx or PaintCtx
  * widgets register their native drawing layers during 
  * layers are registered to the parent window during `paint` (`paint_ctx.register_layer(transform, layer)`)