use crate::model::{Data, Revision};

/// Creates a value from data (a lesser version of a lens).
pub trait Binding<S: Data> {
    type Output: Data;

    /// Returns Some() if the value has changed.
    /// Could return a Revision<Self::Output>, but the state may not even live in the binding (so no ref)
    fn compute(&mut self, data: &S) -> Self::Output;
    fn compute_if_changed(&mut self, data: &Revision<S>) -> Option<Self::Output>;
}

/// A binding with an underlying state that you can get a reference to. (materialized)
pub trait CachedBinding<S: Data>: Binding<S> {
    fn get(&self) -> &Self::Output;
    fn get_mut(&mut self) -> &mut Self::Output;

    /// Updates the cached value.
    fn update(&mut self, rev: &Revision<S>) -> Revision<Self::Output>;
}

/*pub struct Binding<S: Data, V: View<S>, F: Fn(&mut V, Revision<S>)>
{
    view: V,
    f: F,
    // why do we need a PhantomData here and not in `Map`? variance?
    _phantom: PhantomData<*const S>
}

impl<S, V, F> Binding<S, V, F>
where
    S: Data,
    V: View<S>,
    F: Fn(&mut V, Revision<S>),
{
    pub fn new(view: V, f: F) -> Binding<S, V, F> {
        Binding { view, f, _phantom: PhantomData }
    }
}

impl<S, V, F> View<S> for Binding<S, V, F>
where
    S: Data,
    V: View<S>,
    F: Fn(&mut V, Revision<S>),
{
    type Action = V::Action;

    fn update(&mut self, rev: Revision<S>) {
        self.view.update(rev.clone());
        (self.f)(&mut self.view, rev);
    }

    fn mount(&mut self, actx: ActionCtx<Self::Action>) {
        self.view.mount(actx)
    }

    fn widget_ptr(&self) -> Option<Ptr<QWidget>> {
        self.view.widget_ptr()
    }
}
*/
