
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


# Cache cell

* Constant size - 24 bytes (enough for a f32x4 color value + TypeId)
* Store inline if size is small
* Otherwise, uses a box
* Cloneable


# Support for MacOS?

There are several configurations:
- Windows, Linux: skia with vulkan device (via graal, or something else)
- MacOS: skia with metal device

# Native compositor layers

On windows, they are backed by swapchains. But this seems inefficient since they will allocate 2~3 times the memory
(for each buffer in the swap chain) for something that is not supposed to change a lot.

-> do not use swap chains for static content, use them only for 3D/video overlays

1. Create Compositor
2. Create CompositionGraphicsDevice from ID2D1Device/ID3D11Device (`ICompositorInterop::CreateGraphicsDevice`)
3. Create CompositionGraphicsSurface (`CompositionGraphicsDevice::CreateDrawingSurface`)
    * I assume this calls the underlying ID2D1Device/ID3D11Device passed earlier
4. Cast CompositionGraphicsSurface to ICompositionSurface
5. Set as the surface of a CreateSurfaceBrush

The CompositionGraphicsSurface surface created by CompositionGraphicsDevice are not shareable with other APIs, so don't bother.


Ideally, would like to draw directly on IDCompositionSurface, but how?
* Not possible with DX12 devices (Compositor doesn't support DX12)
* Should be possible with D3D11, but Windows.UI.Composition / CompositionGraphicsDevice doesn't support D3D11?

=> Don't bother, it creates a swap chain under the hood (call BeginDraw multiple times and you see that it flips between two different resources with the DXGI_USAGE_BACK_BUFFER flag)

Conclusion:
* static elements (e.g. text): render and cache to texture
* dynamic elements (gauges, button hover, etc.): re-render with small damage region
* scrollable regions: composition layer
* video, 3D: composition layer
* static content with dynamic transform: composition layer



# Skia stuff

- create from native compositor surface
   - different code paths for macos and vulkan (linux/vulkan or win32/vulkan)
- compositor surface interface
  - does nothing by default, but there are specific interfaces for macOS or win32/vulkan
  - vulkan interface for compositor surface:
    - acquire_image, present_and_release_image()

|         | macOS           | Win32/Vulkan         |   |
|---------|-----------------|----------------------|---|
| Image   | CAMetalDrawable | graal::Image         |   |
| Surface | CAMetalLayer    | CompositionSwapChain |   |
|         |                 |                      |   |

```rust
pub trait VulkanCompositionSurface {
    fn acquire_image(&self) -> graal::ImageInfo;
    unsafe fn present_and_release_image(&self, image: graal::ImageInfo, dirty_rect: Rect);
}
```

Note:
Skia supports D3D12, so instead of trying to shoehorn vulkan, use the D3D12 backend of skia. 
graal/vulkan becomes optional on windows, no need for complicated interop.
3D can still use vulkan via raw composition layers

See also: [Possible Deprecation / Removal of D3D Backend](https://groups.google.com/g/skia-discuss/c/WY7yzRjGGFA)


# Data structure for the retained widget tree

"container-owns":
(+) straightforward regarding ownership
(-) event delivery is complicated:
    - need participation of widgets for event delivery
    - need to maintain a bloom filter to avoid unnecessary traversals

ID-tree:
(+) event delivery is simpler, can directly address any widget
(-) forced type erasure
(-) can't easily borrow mutably multiple widgets at the same time (e.g. a parent and one of its children): deal-breaker for calculations that tend to access both (e.g. layout)

Possible way forward, as suggested on xilem zulip: container-owns synchronized with a side tree containing the widget hierarchy 


# Issue: UI diff evaluation is in the same thread as the UI handler

In other words: UI blocked when the UI diff is being calculated.

Q: Is that an issue?
A: It's easy to accidentally perform a costly operation in the UI eval function. If UI eval is done in another thread (the "application thread") by default,
   it would not block the event handlers.

Q: what about layout? should it be done in another thread as well?
A: would need to duplicate the element tree

Advantages:
* Doesn't block the UI by default

Problems:
* Signals would be emitted from the UI thread and received in the application thread, requiring Arc<CacheVar>
* Can't access `Application::global()`
  * no compositor
  * no GPU backend
  * no drawing

## What does "blocking the UI" mean?

User clicks/drags something and doesn't see any feedback / cannot interact with anything else.
This means that a long computation is preventing input events from being processed.

In that sense, the evaluation of the UI diff *cannot* be expensive. Whether it's calculated in the same thread or another, 
it will look the same to the user (except if we do UI updates directly on the element tree, without re-evaluating the widget tree).

Conclusion: it makes no sense to move the UI diff evaluation outside the UI thread.

# Multiple windows

## Option A
UI closure per-window.
App object retains a list of open windows (Idle handles), holds the app state in a refcell.
When the app state changes (either compare with the prev state or increment rev index), signal all windows to redraw their UI.
Windows hold a shared ref to the app state, borrow_mut and re-run the UI closure with it.

## Option B
UI closure for the whole app.
App logic runs in a separate thread. Inside the UI closure, send diffs to the windows via channels.

## Option C
UI closure for the whole app.
App logic runs in the UI thread.
App logic run after each window event.
App logic sets diffs via `Rc<RefCell<>>` in WinHandler.

# List diffs 
List of insertion/removals/modifications. Each widget has an optional ID to identify it in the list. 
ID produced from location in the call trace.

Each element linked to a widget by its call ID. Element containers hold a `Vec<Box<dyn Element>>`, each elem node stores ID + inner element.
Specialized function that performs reconciliation of widgets onto a `Vec<Box<dyn Element>>`.
Elements know their ID, returned with `Element::id`.

List patches: sequence of tokens:
- Start: anchor at the start of the sequence
- Modify(T): modify current element
- Advance(N): skip N elements
- Find(ID): go to element with specified ID
- Remove: remove current element
- Skip: skip to end
- End: end sequence

Example: insert 5 elements at position 5
- Start
- Advance(5)
- Insert (x5)
- Skip
- End

Example: replace the whole list

# Compositor API

Annoying to do this every time:
```
let app = Application::global();
let mut compositor = app.compositor();
compositor.do_thing_with_layer_or_surface(layer_id.unwrap());
```

Alternatives:

A: Surfaces / layers are refcounted, non-thread-safe objects:
```
// no need to access the compositor
let layer = Layer::new()?;
surface_layer.do_stuff()?;
layer.add_child(other_layer)?;
surface_layer.acquire_drawing_surface()?;
surface_layer.release_drawing_surface(surf)?;
```

Internally, store `Rc<Compositor>` + layer ID.
`Compositor` is clonable, but not Sync. IDs can still be sent across threads.

# Async rendering & presentation

Input/main task: receives and propagates input events to the element tree, which in turn may request repaints
Render task: a loop, synced with presentation:
```rust
fn render_task() {
   loop {
        // sync with presentation
        wait_for_presentation();
        // receive last 
        let request = rx.recv();
   }
}
```


Events by time:

- Input event #1
  - Propagate to element tree
  - If the event resulted in dirty regions, immediately synchronize with presentation, and schedule idle task UI_UPDATE
- Input event #2
- ...
- Input event #n
When the input event queue is clear:
Idle task: UI_UPDATE
 - evaluate widget tree by calling the UI function
 - apply to element tree
 - if repaint needed: invalidate dirty region and schedule REPAINT

(may process additional input events here)

Idle task: SYNC_WITH_PRESENTATION
 - sync with presentation
 - schedule UI_UPDATE

(process additional input events...)

Idle task: UI_UPDATE
- evaluate widget tree by calling the UI function
- apply to element tree
- repaint the element tree if needed

Doesn't work with glazier: schedule_idle puts the work on the message queue immediately

**Fact**: wait_for_presentation cannot run in the same thread as the UI handler, because otherwise it would block unrelated windows.
-> it's becoming clear that rendering should be done in a separate thread

# New event routing

Goals: require minimum cooperation from the widget/element implementation

Locate widgets using "ID paths" (slices of Widget IDs).

Two things:
- `event()`: receive an event destined to this widget
- `route_event()`: propagate an event to a child widget, event not meant for us specifically

Example: propagating an event through a VBox:
- `VBox::route_event()` is called
- VBox calls `Event::next_target(&mut self) -> WidgetID` to get the widget ID that should receive the event
- if ID is the vbox:
  - `VBox::event()`
- otherwise lookup the ID in a map of some sort
  - if ID not found that's an error (inconsistent tree)
- call `child.route_event(event)`

Propagating an event through a ElementNode:
- transform pointer events
- child.route_event

Default implementation of route_event:
- if next_event() return None, event is for us
- otherwise: error, widget should have a route_event implementation

```rust 
fn route_event(&mut self, ctx: &mut RouteEventCtx, event: &Event) {
    if let Some(target) = ctx.next_target() {
        let Some(target) = self.child_by_id(target) else {
          warn!("inconsistent tree");
          return;
        };
        target.route_event(ctx, event);
    }
  
    ctx.default_route_event(self, event);
}
```

Rule: every container widget should have a route_event implementation.

## Pointer event propagation:
These events have no target, except when the mouse is captured by a widget.

Should hit-testing be done as part of the event propagation? or should there be a separate hit-testing tree?
-> not a separate tree, but a separate Element method to get the list of widgets under a position

`Element::hit_test(&self, ctx: &mut HitTestCtx, position: Point) -> bool`
Q: does the element know its geometry?
A: yes, although wrappers can defer to their content widget

Q: what about elements that share the same ID but have different constraints? (e.g. Frames)
A: hit-test propagated to inner element

```rust 
fn hit_test(&self, ctx: &mut HitTestCtx, position: Point) {
    self.bounds.contains(position)
}
```

Summary:
* hit-test returns one or more targets
* event is sent to those targets, and bubbles up
  * 



Should hit-test be manually recursive?

`event` called for events that target the widget itself.
`route_event` called for events that should be routed to children.

Problem: broadcast events

Q: which events are broadcast in old kyute?
A: Some pointer events (because hit-test is done at the same time as propagation), UpdateChildFilter, dump_tree

Propagating "events", or "requests" in a larger sense:
1. Use events
2. Use events, and convert them into method calls when arriving at target
3. Use methods, implementation responsible for propagating to children
4. Use a generic visitor mechanism

In flutter:
- Hit-test: implementors must propagate to children
- Painting: implementors must propagate to children
- Layout: implementors must propagate to children

## Layout caching
ElementNodes can cache their layouts, and store a dirty flag for relayouts.



# Layout v2
More incrementality.

Events affecting the layout of a widget:
- structure of children changed (ChangeFlags::STRUCTURE)
- size of children changed (ChangeFlags::SIZE)
- positioning (alignment) of children changed (ChangeFlags::POSITIONING)
- parent constraints changed 

These may affect 
- only the size but not the positioning of children (rare?) 
- only the positioning, but not the size of children
- only the size of this widget, but not it's positioning, or its children
- only the positioning, but not it's size, or its children

4 separate components of layout:
- self size
- self positioning
- child offsets
- child geometry

In order:
1. compute child constraints (CONSTRAINTS, SIZE_DIRTY) -> CHILD_CONSTRAINTS
2. layout_children (CHILD_CONSTRAINTS) -> CHILD_GEOMETRY
3. compute_geometry (CONSTRAINTS, SIZE_DIRTY, CHILD_GEOMETRY) -> GEOMETRY

compute_geometry may not depend on CHILD_GEOMETRY

DirtyFlags:
- CONSTRAINTS: parent constraints have changed
~~- CHILD_CONSTRAINTS: child constraints have changed~~
- CHILD_GEOMETRY: child geometry may have changed
- CHILD_POSITIONS: child positions may have changed
- GEOMETRY: geometry may have changed
- PAINT: visual may have changed

Dirty flags are updated on events & on constraint change

E.g. for `Frame`:

```rust 
fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &LayoutParams) {
    if self.layout.constraints != constraints {
      self.layout_flags |= LayoutFlags::CONSTRAINTS | LayoutFlags::CHILD_GEOMETRY | LayoutFlags::CHILD_POSITIONS;
    }
}

fn event(&mut self, ctx: &mut EventCtx, event: &Event) {
  // ... propagate event ...
  if ctx.change_flags.intersects(ChangeFlags::SIZE) {
    // size of child item has changed
    self.layout_flags |= LayoutFlags::CHILD_GEOMETRY | LayoutFlags::CHILD_POSITIONS;
  } 
  if ctx.change_flags.intersects(ChangeFlags::POSITIONING) {
    // only the positioning has changed, not its size given the same constraints
    self.layout_flags |= LayoutFlags::CHILD_POSITIONS;
  }
  // child geometry changes do not affect the geometry of this frame
  ctx.change_flags.remove(ChangeFlags::GEOMETRY);
  
}
```

## Dirty flags

Proposal: a method to propagate dirty flags upwards, automatically called as a result of `Widget::event` and `TreeCtx::update`.

```rust
impl TreeCtx {
  pub fn update(&mut self, element: &mut E, widget: W) where W: Widget<Element=E>, E: Element {
    let change_flags = widget.update(&mut element);
    element.propagate_flags(change_flags)
  }
}

impl EventCtx {
  pub fn event(&mut self, child: &mut E, event: &E) where E: Element {
    child.event(e);
    element.propagate_flags(change_flags);
  }
}
```


## Length resolution

Issue: lengths can be relative to the current font size or the parent element size. 
When updating the element tree, even if the relative length does not change, the layout might still change
-> resolve everything in layout() for now, pass parent font size in LayoutParams

## Hit-testing contract

Q: What should a widget do in `hit_test`?
Q1  Should it return one hit? 
Q2  Should it return multiple hits ordered by Z-index?
Q3  Is it responsible for calling hit_test on the hit child elements? 
Q4  Should we hit-test children that are out of parent bounds?
Q5  Should elements report a hit on transparent parts?

A: hit testing should return all intersected elements (if requested)
There is demand For hit-testing outside parent bounds, see https://github.com/flutter/flutter/issues/75747. 
DOM events: hit-testing outside parent bounds by default.
For transparent parts: depends on the widget.

How to implement hit-testing outside parent bounds?
1. a separate data structure holding visual nodes
2. ID buffer (need separate rendering step, meh)
3. elements compute the union of the bounds of all children

(3) seems the most promising. However, it's costly, so need caching.

Elements are responsible for their own hit-test, so they must remember their geometry.
That means that every element other than simple wrappers will have a `geometry` field.

FIXME: bounds & paint bounds shouldn't be in Geometry
Example: ElementNode, with a non-zero transform. What is the returned `bounding_rect`?
Currently, it's the bounding rect of the content, *without the transform*, so the bounding rect in the content local coordinates.
It should be bounds in the ElementNode local coordinate system.


## Caching layout results

Stuff to cache:
- layout parameters, to determine if they have changed
- geometry, to reuse if the widget has determined that it hasn't changed
- total bounds (self + descendants)

Idea: include descendant bounds in geometry.

## Idea: attached properties?
Same as WPF, QML, and flutter [ParentData](https://api.flutter.dev/flutter/rendering/ParentData-class.html). Used to store layout info for the parent into the child.

## Idea: `ElementNode` shouldn't be an `Element`.
The parent element should be responsible for applying transforms when propagating events, hit-testing, painting...


## why alignment & padding should be treated differently than other layout parameters?
Such as grid positions, or docking status, or explicit offsets?

TODO: is it possible to design an extensible mechanism for a child to specify layout properties for a parent?
I.e. decouple positioning info from actual geometry.

```
fn test() -> impl Widget {
    button()       // Button
    .align(...)    // ???<Button, Alignment>    
    .grid_column() // ???<???<Button, Alignment>, GridLayoutInfo>
    .grid_row()    // ???<???<Button, Alignment>, GridLayoutInfo>
}
```

Trait-based solution?
E.g. for grid containers: `fn add(impl (Widget + HasGridLayoutProperties))`.
Issue: implementing `GridLayoutProperties` for every widget. Need specialization?

Associated types?

Type erasure?
Return a `dyn Any`, and downcast.


## Layout modifiers

Independent of the container (creates a sub-element):
- padding
- fixed width/height
- alignment? could work, but what about relative positioning?
    - would be a separate widget

Dependent on the container:
- alignment (flex/grid/frame)
- grid position (grid)
- flex factor (flex)
- dock index (dock)

Mixed:
- left/top/right/bottom: padding + alignment

Issue: overhead of transforms
e.g. padding + alignment would create two TransformNodes
=> Just create a widget that does both at the same time (e.g. frame)


## Text
- Use swash.
- should be a global font database, initialized from system fonts.

## Lengths
Is it possible to resolve them early? Like during widget update?
Need to know three things: 
- parent font size: OK
- scale factor: could be OK
- container size: obviously not known until layout

Reasonably, for font sizes, we'd like em-sizes and dips/pixels

More generally, early value resolutions would be easier to handle. 
Ideally we would like to resolve before widgets are created, otherwise we need two versions of some data structures.
For example, we'd need two TextSpan types: one for the user with properties specified in `Length`s, the other for the 
element tree with values resolved to `f64` DIP sizes => that would be **super annoying** (citation needed: maybe it would be reasonable)


However, we lose the pretty syntax to specify the font size for a whole widget subtree:
```
widget.align(...).font_size(...)  // sets font size for Align<Widget<...>>
```

And instead we need to work with closures and a thread-local environment:

```rust
fn test() -> impl Widget {
  with_environment(theme::FONT_SIZE, 16.0, || {
    ...
  })
}
```

Alternatively, we may use macros:
```rust
fn test() -> impl Widget {
  environment! {
     theme::FONT_SIZE=16.0, disabled=self.disabled => Align::new(Widget::new(..))
  }
}
```

Or alter the current model even more, threading the context explicitly

```rust
#[composable]
fn my_widget(cx: &Context, state: &Stuff) -> impl Widget {
  // ...
}
```

Some widgets need a context, but not all.
E.g. `Button::new(label)` should be just that, and not `Button::new(cx, label)`. 
The tree is a "tree of closures" taking a context parameter.
The tree is then evaluated, passing a "Context" parameter. It's only at this stage that the signals, events and other retained state are accessible.

```rust
fn my_widget(cx: &Context, data: &Data) -> impl Widget {
  //...
}

fn framed<'a>(cx: &Context, data: &'a Data) -> impl Widget + 'a {
  // issue: borrowing of data
  let button = Frame::new(200, 200, |cx| my_widget(cx, data)).clickable(cx);
  // issue: mutating data
  if button.clicked() {
  }
  // alternate design:
  Frame::new(200, 200, |cx| my_widget(cx, data)).clickable(|cx,data| {
    // do something with data? but then I'd need a mutable borrow of data, and I can't do that since my_widget already borrows it
    // this means that Widgets should now have an additional "data" type parameter
    // and then this basically becomes xilem
  });
  
  // It will need to be written this way however, for list views with incremental updates (can't render incremental list views with a for loop)
}
```

Issue with incremental updates? Consider:

1. a list widget sees that one element has been added to the list, and generates an incremental update to the element tree
2. however, at the same time, a signal has been triggered for another element of the list (e.g. a button has been clicked inside a list entry)
3. how does the list widget know which widget to recompute?

=> the cache system expects widget-producing functions to be called everytime (they may be skipped if they are cached). But the incremental list widget
only calls the widget function for newly added/removed entries

Conclusion: the incremental list widget *needs* to call the child closure for every child
-> Not a big deal, since most children can be skipped, and the final diff on the element tree won't be large

Can we do without calling the child item closure?
The problem is that the child closure serves two purposes: creating/updating the item, 
and reacting to events. If a list item receives an event, then the item closure must be called,
and the item rebuilt.

```rust

fn list(child_item: impl FnMut(Item) -> Widget) {
  for (id,item) in items {
    // enter scope and 
    cx.scoped(id, |dirty| {
        if dirty || diff.contains(id) {
          // re-evaluate
          let widget = child_item(item);
          
          true
        } else {
          // skip subtree
          false
        }
    });
  }
}

```

```

// input parameters
// state parameters [1]
// reactive closure parameters [2]
// (one of [1] or [2] but not both, they define the "state" type of the widget)
// contents 

ItemView(data: &Item) [data: &mut Item] {   // params between square brackets become visible to all things in square brackets (the "reactive" part) 
  Text(data.title)
  TextEdit(data.title) [on_text_changed: |new_text| {
    data.title = new_text;
  }]
}

TextEdit(text: &str) {
  Text(text)
  InternalTextEdit(text) [on_text_changed: on_text_changed]
}

MainView(data: &AppData) [data: &mut AppData] {
  VStack {
    ItemView(data.first_item) [data.first_item]     // two-way binding
    ItemView(data.second_item) [data.second_item]
  }
}

```

## Incremental lists:

```
MainView(data: &AppData) {
  VStack {
    for item in data.items() {    // items() returns a special kind of iterator able to provide a diff
      ItemView(item) [item]       // FIXME: how do I pass a mut ref to an item here? I'd need another iterator
    }
  }
}
```

## Mutations?
Pass something in the square brackets, but it can't be the same data as the input parameters.
I.e. we can't refer to input parameter data in reactive parts => this is annoying, can't get an ID to the data at all.

Alternatively: don't pass a mut ref to the data, but instead pass a "mutation" object for the data model.
Alternatively: capture input parameter data by value?
-> possible, but extremely annoying if data is not Copy. 
Explanation: at the location where the reactive closure is defined, it can see and capture stuff from input parameters (`&Data`).
The reactive closure cannot borrow from the input data, since it would lock the data for modification, and it would be impossible
to pass a `&mut Data` to the reactive closure.
So, the challenge here is to capture everything by value. And if the stuff to capture in `Data` is not `Copy` then
it's very annoying: we need to `.clone()` the data *outside* the closure and capture the clone. 


Q: you get a reference to data to build the UI, but then how to modify that data at the same time? (the "reactive" part).

Other issues:
- receiving events when the view is skipped
- for memoization, previous state not available until update, need to defer view creation at update time, which would need a borrow

## In search of a good layout system

CSS grid, but with an editor.
Must be fast; avoid speculative layout passes.
Issue: auto-sizing columns: need the maximum size of the contents.

Avoid allocations

Can it be incremental?


## Pass scale factor & font size in environment?
No need to resolve lengths anymore.

Issue with scale factor: scale factor changes will need a (full) recomposition
Issue with font size: every container that has a custom font size will need to open an environment scope, can't "push" child items into the container

Alternative: remove em-sizes?
QML, WPF don't have them.

Decision => em and physical pixel sizes removed for now. 

## Expose a widget that renders a SKSL shader
Good to prototype stuff.
Allow passing uniforms to it. 

## Pain points
- `event` vs `route_event`
- widget tree tracking (`child_added`, `child_removed`) is error-prone, and completely non-functional right now
  - it's necessary to build the event propagation path

## Switch back to winit

## Nuke winit

## Nuke every crate related to windowing and vendor everything
It's the only way to be sure.
Somehow winit, raw_window_handle and others are getting worse every update. 


## Event propagation?
There's no bubbling right now, nor capture.
It's difficult to predict what propagation should look like, so do something familiar to users, like https://www.w3.org/TR/uievents/#event-flow.
We already can determine the propagation path through the widget tree, which gives us a list of widget IDs.
Compared to the DOM, we have the additional complication that IDs can refer to multiple widgets, with the following restrictions:
- two sibling widgets (sharing the same parent) cannot have the same ID (unless it's the ANONYMOUS id).
- only widgets that have a direct parent-child relation can have the same ID, and only if the child is unique.
  - i.e. a container widget cannot have the same ID as its parent.
-> in short, the only case where two widgets can share the same IDs is with a widget that wraps one unique child widget.

Implementing the capture phase:
During the capture phase, the event is wrapped in the "Event::Propagate" wrapper. This wrapper holds the propagation path. If the widget wishes to capture the event, it can look inside this event and determine whether to continue propagation or stop it.

Roughly, the event logic for a widget will be:

```
match event {
  // handle events for this widget
  ...
}

// propagate event if necessary
if let Some(event) = event.next() {
  let target = event.target();
  // determine which child is the target and send the event
  let child = ...;
  ctx.propagate_event(child, event);
}

```


## Environment values

How to make a value depend on some environment value? How to check if the dependency should be recomputed?


```
// with_state(cx, init, F) where F: for<'a> FnOnce(cx, &'a mut State) -> Widget + 'a    // returns a widget that borrows 'a

fn with_state(cx, init, f) {
    let mut state = ???;
    let widget = f(cx, &mut state);
    // widget borrows state
    widget.build(cx);   // build or update element tree and invoke callbacks
    // state not borrowed anymore, so it's OK
}

// Not possible, but as an alternative, the state can "travel" via a context

let (value, set_value) = cx.state(|| false);
// set_value is a copyable token that identifies this particular state


Stateful::new(cx, || WidgetState(false), |cx| {
  
  // issue: must have one type per state
  
  let mut state = WidgetState::get(cx);
  // do something with state
  
  let inner = Stateful::new(cx, || WidgetState(false), |cx| {
    let mut inner_state = WidgetState::get(cx);
    // issue: how do I get the outer state?
  });
  
  let button = Button::new(cx).on_click(|cx| {
    WidgetState::set(cs, new_state);
  });
  
  // update state
  WidgetState::set(cx, new_state);

});

// cx is a stack of states (&mut refs directly)
// find nearest state by type id, then set value 


// top-level function:
fn () -> impl Widget {
}

// Widget has build(cx) and update(cx)
// 
// build(cx) and update(cx) can push modifiable state on the context


fn app_ui(app_state: &AppState) -> impl Widget {

   Frame::new(50,50).clickable().on_click(|cx| {
       // access app_state: we can't borrow from the closure above, but we can 
       // get it (a reference to it) from cx
       let app_state = AppState::get_mut(cx);
       
   });
}

fn main_ui() -> impl Widget {
  Stateful::new(AppState::default, |cx| {
    // closure invoked during `Widget::build` and `Widget::update`
    let app_state = AppState::get(cx);  // returns ref to app_state; it borrows cx but that's OK since it's not used by app_ui
    app_ui(app_state)
  });
}

```

    fn stateful_test(app: &AppState) -> impl Widget + '_ {
      // app: 'a
      // flag: 'b (anonymous inside closure)
      // inner_state: implies 'a == 'b, unprovable
  
      //let mut what = false;
      //let f = move |state: &'b mut bool| test2(app, state);
  
      // issue: closure can either return a borrow of the state (anonymous lifetime), or something borrowed externally.
      // the resulting lifetime is anonymous, and cannot be used to prove that the closure is valid.
      // closure with bounds for<'a: 'b> 'b + FnOnce(&'a mut bool) -> impl Debug + 'a
  
      Stateful::<bool, _>::new(move |ctx, state: &mut bool| {
          // issue: inner_state conflates two lifetimes that are unrelated to each other:
          // the anonymous lifetime of "state" which can be anything, and the lifetime of the borrowed data ('a).
          // in short: the widget returned by the closure can't borrow from both the local state and the app.
          //
          // Solving this would be an absolute win.
          //
          // First, the lifetime of the state should be definite. I.e. the closure type should NOT be for<'a> FnOnce(&'a mut bool),
          // but rather FnOnce(&'b mut bool) where 'b for "some concrete lifetime".
          //
          // Next question: where does this 'b lifetime come from?
          // It should be the lifetime of the state, but it's not known here
          inner_state(app, state)
  
          // Alternatives?
          // 1. not returning a widget, but rather build the widget in the closure

          // NOTE: 
      })
    }

https://github.com/audulus/rui/issues/26 seems to tackle a related/similar problem?

## Investigate [rui](https://github.com/audulus/rui/)

Interesting stuff:
- https://github.com/audulus/rui/issues/26: seems to tackle the "closure that returns a value borrowing input" problem
- There's only one trait to implement ("View") instead of the Element/Widget split
  - There's no retained element tree, so that might explain that
- Not sure about memoization
  - According to the readme: "everything is re-rendered when state changes", so no memoization / fine-grained invalidation
- Basically, "immediate mode with better layout options", which is interesting
- Passes state down the tree with a "context", like we do. However, the context is accessed explicitly with "Bindings" that identify the state within the context, instead of accessing it by looking up a TypeID. This feels much more principled: steal this idea :)
  - Bindings are just `Copy`able IDs to avoid borrowing issue in callbacks (still need `move` though?) 
  - Q: can we track dependencies this way? 
    - Idea: inside `Widget::build` or `update`, the TreeCtx can keep track of all referenced state entries.



## Shapes
Idea: apply "ShapeOps" in sequence, each shape op has layout and paint methods.
They can modify the shape for the operator above it (e.g. borders will inflate the shape).
Example of ShapeOps:
- Fill
- Stroke (stroke inside)
- Border (offset + stroke)
- DropShadow
- InnerShadow
- Offset (offset along normals) =>
- Transform


Text::new().padding(4.0).background(
  // Shape is sized according to the size of the text, does not affect 
  // available space for the text
  Shape::new(RoundedRect)
      .drop_shadow()    // painted first
      .fill()           // then this
      .inner_stroke()   // then this
      .outer_stroke()   // then this
      // then the text is rendered
);

In general, the background shape has no influence on the resulting geometry of the widget.
The inner widget defines the geometry, and the shape draws itself within that geometry.
This is different from CSS where borders affect the layout of the element.
=> This means that when changing the border width, users should also change the padding of the widget inside
to account for larger borders.

Q: Is that a problem?
A: Not sure; it's nice that changing the rendered shape doesn't affect the geometry and doesn't require a relayout in the general case, so I'd tend to keep that.
A2: nvm, flutter has decorations with content padding, so I'd just copy that

Idea: move shapes to "Frame", add `.decoration` method.

## Layout puzzle: size to fit content, but obey minimum constraints

E.g. button has min constraint 80x30, max constraint propagated from above, can be unbounded.
Button should have minimum possible size, but if not tight around the text, the text should be centered.

**Problem**: alignment widget will expand to max possible size if constrained. This may not be what we want


## Outline views

A generalization of list views.

### Previous work

- [flutter_tree_view](https://github.com/baumths/flutter_tree_view)
- [SwiftUI OutlineGroup](https://developer.apple.com/documentation/swiftui/outlinegroup)
  - Identifiable data + closure to access children
  - How to do incremental updates?


### Sketch

```rust

use std::hash::Hash;

pub trait Identifiable {
  type Id: Copy + Hash;
  fn id(&self) -> Self::Id;
}

pub trait DiffableCollection: Clone {
  type Item;
  // Indexable, access elements by ID
  // Return an iterator over elements added & removed, compared to a previous instance 
    // Basically implies immutable collections
}



pub trait TreeDataSource {
  type Item: Identifiable;
  fn element(&self, id: Self::Item::Id) -> &Self::Item;
  fn children(&self, id: Self::Item::Id) -> impl Iterator<Item=&Self::Item>;
  fn revision(&self) -> Revision;
  fn changes(&self, since: Revision) -> impl Iterator<Item=Diff<Self::Item>>;
}

```

Complete IDs: IDs uniquely identify an element in the tree.
Partial IDs: IDs uniquely identify a child node within a parent.

Partial IDs imply that we need to identify nodes in the tree by an "ID path".

Q: Full IDs or ID paths?



```rust 

struct TreeNode {
  id: u64,
  children: Vec<TreeNode>
}

```

## Window event handling
It works like this:
- (Application) the event loop receives a window event
- (WindowHandler) it is passed to the window handler (in event() or paint())
- (WindowHandler) the window handler handles the event, and determines where the UI event should be sent
- (WindowHandler) the window handler sends the UI event to the appropriate widget

There's an issue with FocusGained/FocusLost events. Now that those states are app-wide, application code should build
and send those events. Otherwise, the window handler may end up needing to send a FocusLost event to *another* window,
which it is not supposed to do.

Modified window event handling:
- (Application) the event loop receives a window event
- (WindowHandler) it is passed to the window handler (in window_event() or paint())
- (WindowHandler) the window handler determines if it is interested in it, and if so, queues events to be propagated
   by adding them to a queue in AppState
- (Application) control returns to application
- (Application) app dequeues pushed events and sends them to the elements, via their corresponding WindowHandler
- (WindowHandler) event() is called, window handler propagates the event to the element

Other ideas:
- WindowHandlers have a "UiTreeHandle": handle to a UI tree, visible by the application
- when registering the window, also optionally register the UI tree
- application can then send events directly to the UI tree without coordinating with the WindowHandler

Alternative:
- Windows are just Elements
- they receive a special type of event (`WindowEvent`).


Tentative:
- the app maintains a list of weak ptrs to window handlers
- window events received by the app are sent to window handlers
- window handlers create events and send them to the application

Issue with the current situation: a lot of things are done by the windowhandler, which is common to all windows hosting a UI tree.
Split that into a reusable component, tentatively named `UiContentHost`.
Windows with UI content should use this type, and forward the window events to it.
=> Note sure that's useful, short term: it would be useful if we wanted to host UI in a non-winit window, but then we'd need to have
an API on UiContentHost to receive window events (in a windowing-crate-agnostic fashion)

=> ignore all this above


## ElementIdTree is supremely annoying
Need to maintain it, store it somewhere, next to the UI element tree. Would  
