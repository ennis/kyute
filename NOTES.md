
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

