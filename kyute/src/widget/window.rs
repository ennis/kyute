
struct Window<W> {
    inner: W,
}

impl<A,W: Widget<A>> Widget<A> for Window<W>
{
    fn layout(self, ctx: &mut LayoutCtx<A>, cursor: &mut Cursor, _constraints: &BoxConstraints)
    {
        // should box constraints not be ignored?
        let win_ctx = ctx.win_ctx;

        let window_node = cursor.open(None, || WindowVisual::new(win_ctx, WindowBuilder::new()));
        // the node needs to be registered in the event loop

    }
}
