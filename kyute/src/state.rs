/*
#[derive(Clone)]
pub struct State<T> {
    key: cache::Key<T>,
}

impl<T: Clone + 'static> State<T> {
    #[composable]
    pub fn new(init: impl FnOnce() -> T) -> State<T> {
        let key = cache::state(init);
        State { key }
    }

    pub fn get(&self) -> T {
        self.key.get()
    }

    pub fn update(&self, value: Option<T>) {
        if let Some(value) = value {
            self.key.set(value)
        }
    }

    pub fn set(&self, value: T) {
        self.key.set(value)
    }
}
*/
