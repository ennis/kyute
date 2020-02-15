use crate::model::{Data, Revision};

pub trait Property {
    type Value: Data;

    /// TODO return by reference?
    fn get(&self) -> Self::Value;

    /// TODO return another revision?
    fn update(&mut self, rev: Revision<Self::Value>);

    fn set(&mut self, value: Self::Value) {
        self.update((&value).into())
    }
}

pub struct SimpleProperty<'a, T, V: Data, Get: Fn(&T) -> V, Update: Fn(&mut T, Revision<V>)> {
    pub this: &'a mut T,
    pub get: Get,
    pub update: Update,
}

impl<'a, T, V: Data, Get: Fn(&T) -> V, Update: Fn(&mut T, Revision<V>)> Property
    for SimpleProperty<'a, T, V, Get, Update>
{
    type Value = V;

    fn get(&self) -> Self::Value {
        (self.get)(self.this)
    }

    fn update(&mut self, rev: Revision<V>) {
        (self.update)(self.this, rev)
    }
}

#[macro_export]
macro_rules! simple_property {
    (
        self: $this:expr,
        get: $get:expr,
        update: $update:expr
    ) => {
        $crate::view::property::SimpleProperty {
            this: $this,
            get: $get,
            update: $update,
        }
    };
}
