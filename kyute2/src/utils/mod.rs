#[derive(Copy, Clone, Debug)]
pub struct Cached<K, V> {
    key: K,
    value: V,
}

impl<K: Clone + PartialEq, V> Cached<K, V> {
    pub fn new(initial_key: K, initial_value: V) -> Cached<K, V> {
        Cached {
            key: initial_key,
            value: initial_value,
        }
    }

    pub fn get(&mut self, key: &K, calculate: impl FnOnce(&K) -> V) -> &V {
        if self.key != *key {
            self.value = calculate(key);
            self.key = key.clone();
        }
        &self.value
    }

    pub fn get_cached(&self) -> &V {
        &self.value
    }
}
