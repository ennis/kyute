use std::{
    any::{Any, TypeId},
    fmt,
    rc::Rc,
};

/// Trait implemented by values that can be stored in an environment.
pub trait EnvValue: Any {
    fn into_storage(self) -> Rc<dyn Any>;
    fn from_storage(storage: Rc<dyn Any>) -> Self;
}

#[derive(Clone)]
pub struct Environment {
    map: im::HashMap<TypeId, Rc<dyn Any>>,
}

impl fmt::Debug for Environment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO better debug impl
        f.debug_struct("Environment").finish_non_exhaustive()
    }
}

impl Default for Environment {
    fn default() -> Self {
        Environment::new()
    }
}

impl Environment {
    /// Creates a new, empty environment.
    pub fn new() -> Environment {
        Environment {
            map: Default::default(),
        }
    }

    /// Creates a new environment that adds or overrides a given key.
    #[must_use]
    pub fn add<T: EnvValue>(mut self, value: T) -> Environment {
        self.set(value);
        self
    }

    /// Adds or overrides a given key in the given environment.
    pub fn set<T: EnvValue>(&mut self, value: T) {
        self.map.insert(value.type_id(), value.into_storage());
    }

    /// Returns the value corresponding to the key.
    pub fn get<T: EnvValue>(&self) -> Option<T> {
        self.map.get(&TypeId::of::<T>()).map(|v| T::from_storage(v.clone()))
    }

    #[must_use]
    pub fn union(self, other: Environment) -> Environment {
        Environment {
            map: self.map.union(other.map),
        }
    }
}

/*
////////////////////////////////////////////////////////////////////////////////////////////////////
// EnvRef
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Either a value or a reference to a value in an environment.
#[derive(Clone, serde::Deserialize)]
#[serde(untagged)]
pub enum EnvRef<T> {
    /// Inline value.
    Inline(T),
    /// Fetch the value from the environment.
    #[serde(skip)]
    Env(EnvKey<T>),
    /// Evaluate the function with the environment.
    #[serde(skip)]
    Fn(fn(&Environment) -> T),
    /// Evaluates a closure within the environment.
    #[serde(skip)]
    Closure(Arc<dyn Fn(&Environment) -> Option<T>>),
}

// manual impl to avoid "implementation of `Debug` is not general enough" error
impl<T: fmt::Debug> fmt::Debug for EnvRef<T> {
    fn fmt<'a>(&'a self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EnvRef::Inline(val) => f.debug_tuple("Inline").field(val).finish(),
            EnvRef::Env(key) => f.debug_tuple("Env").field(key).finish(),
            EnvRef::Fn(ptr) => f.debug_tuple("Fn").field(ptr as &fn(&'a Environment) -> T).finish(),
            EnvRef::Closure(_ptr) => f.debug_struct("Closure").finish_non_exhaustive(),
        }
    }
}

impl<T: EnvValue> EnvRef<T> {
    pub fn resolve(&self, env: &Environment) -> Option<T> {
        match self {
            EnvRef::Inline(v) => Some(v.clone()),
            EnvRef::Env(k) => k.get(env),
            EnvRef::Fn(f) => Some(f(env)),
            EnvRef::Closure(f) => f(env),
        }
    }

    pub fn map<U>(self, f: impl Fn(T) -> U + 'static) -> EnvRef<U> {
        match self {
            EnvRef::Inline(v) => EnvRef::Inline(f(v)),
            _ => EnvRef::Closure(Arc::new(move |env| self.resolve(env).map(&f))),
        }
    }
}

impl<T: EnvValue + Default> EnvRef<T> {
    pub fn resolve_or_default(&self, env: &Environment) -> T {
        self.resolve(env).unwrap_or_default()
    }
}

impl<T> From<T> for EnvRef<T> {
    fn from(v: T) -> Self {
        EnvRef::Inline(v)
    }
}

impl<T> From<EnvKey<T>> for EnvRef<T> {
    fn from(k: EnvKey<T>) -> Self {
        EnvRef::Env(k)
    }
}

impl<T> From<fn(&Environment) -> T> for EnvRef<T> {
    fn from(f: fn(&Environment) -> T) -> Self {
        EnvRef::Fn(f)
    }
}

impl<T> Default for EnvRef<T>
where
    T: Default,
{
    fn default() -> Self {
        EnvRef::Inline(T::default())
    }
}
*/
