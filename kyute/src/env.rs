use crate::{cache, Color, Data, Length, SideOffsets};
use std::{
    any::Any,
    collections::HashMap,
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
    sync::Arc,
};

/// A type that identifies a named value in an [`Environment`], of a particular type `T`.
#[derive(Debug, Eq, PartialEq)]
pub struct EnvKey<T> {
    key: &'static str,
    _type: PhantomData<T>,
}

impl<T> EnvKey<T> {
    pub fn name(&self) -> &'static str {
        self.key
    }
}

impl<T> Clone for EnvKey<T> {
    fn clone(&self) -> Self {
        EnvKey {
            key: self.key,
            _type: PhantomData,
        }
    }
}

impl<T> Copy for EnvKey<T> {}

impl<T> EnvKey<T> {
    pub const fn new(key: &'static str) -> EnvKey<T> {
        EnvKey {
            key,
            _type: PhantomData,
        }
    }
}

impl<T: EnvValue> EnvKey<T> {
    /// Returns the value of the environment variable in the current env.
    pub fn get(self, env: &Environment) -> Option<T> {
        env.get(self)
    }
}

/// Trait implemented by values that can be stored in an environment.
pub trait EnvValue: Sized + Any + Clone {
    fn as_any(&self) -> &dyn Any;
}

macro_rules! impl_env_value {
    ($t:ty) => {
        impl $crate::EnvValue for $t {
            fn as_any(&self) -> &dyn ::std::any::Any {
                self
            }
        }
    };
}

impl_env_value!(bool);
impl_env_value!(f64);
impl_env_value!(Color);
impl_env_value!(String);
impl_env_value!(SideOffsets);
impl_env_value!(Length);

impl<T: Any> EnvValue for Arc<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone, Debug)]
pub struct Environment(Arc<EnvImpl>);

impl Data for Environment {
    fn same(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Hash for Environment {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // reference semantics
        (&*self.0 as *const EnvImpl).hash(state);
    }
}

#[derive(Clone)]
struct EnvImpl {
    parent: Option<Arc<EnvImpl>>,
    values: HashMap<&'static str, Arc<dyn Any>>,
}

impl fmt::Debug for EnvImpl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO better debug impl
        f.debug_struct("EnvImpl").finish_non_exhaustive()
    }
}

impl EnvImpl {
    fn get<T>(&self, key: &str) -> Option<T>
    where
        T: EnvValue,
    {
        self.values
            .get(key)
            .map(|v| {
                v.downcast_ref::<T>()
                    .expect("unexpected type of environment value")
                    .clone()
            })
            .or_else(|| self.parent.as_ref().and_then(|parent| parent.get(key)))
    }
}

impl Environment {
    /// Creates a new, empty environment.
    pub fn new() -> Environment {
        Environment(Arc::new(EnvImpl {
            parent: None,
            values: HashMap::new(),
        }))
    }

    fn set_inner<T>(&mut self, key: &'static str, value: T)
    where
        T: EnvValue,
    {
        // checks that the type is correct
        self.0.get::<T>(key);

        match Arc::get_mut(&mut self.0) {
            Some(env) => {
                env.values.insert(key, Arc::new(value));
            }
            None => {
                let mut child_env = EnvImpl {
                    parent: Some(self.0.clone()),
                    values: HashMap::new(),
                };
                child_env.values.insert(key, Arc::new(value));
                self.0 = Arc::new(child_env);
            }
        }
    }

    /// Creates a new environment that adds or overrides a given key.
    pub fn add<T>(mut self, key: EnvKey<T>, value: T) -> Environment
    where
        T: EnvValue,
    {
        self.set_inner(key.key, value);
        self
    }

    /// Adds or overrides a given key in the given environment.
    pub fn set<T>(&mut self, key: EnvKey<T>, value: T)
    where
        T: EnvValue,
    {
        self.set_inner(key.key, value);
    }

    /// Returns the value corresponding to the key.
    pub fn get<T>(&self, key: EnvKey<T>) -> Option<T>
    where
        T: EnvValue,
    {
        self.0.get(key.key)
    }

    pub fn merged(&self, mut with: Environment) -> Environment {
        let inner = Arc::make_mut(&mut with.0);
        if let Some(parent) = inner.parent.take() {
            let tmp = self.merged(Environment(parent));
            inner.parent = Some(tmp.0);
        } else {
            inner.parent = Some(self.0.clone())
        }
        with
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

//--------------------------------------------------------------------------------------------------

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

    pub fn resolve_here(&self) -> Option<T> {
        let env = cache::environment();
        self.resolve(&env)
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
