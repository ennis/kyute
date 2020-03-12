
pub struct PaintCtx<'a> {
    d2d_factory: &'a direct2d::factory::Factory1,
    dwrite_factory: &'a directwrite::Factory,
    d2d_target: &'a direct2d::render_target::RenderTarget,
}

impl<'a> PaintCtx<'a> {
    pub fn new(
        d2d_factory: &'a direct2d::factory::Factory1,
        dwrite_factory: &'a directwrite::Factory,
        d2d_target: &'a direct2d::render_target::RenderTarget) -> PaintCtx<'a>
    {
        PaintCtx {
            d2d_factory,
            dwrite_factory,
            d2d_target
        }
    }

    pub fn direct2d(&self) -> &'a direct2d::factory::Factory1 {
        self.d2d_factory
    }

    pub fn directwrite(&self) -> &'a directwrite::Factory {
        self.dwrite_factory
    }

    pub fn direct2d_target(&self) -> &'a direct2d::render_target::RenderTarget {
        self.d2d_target
    }
}

