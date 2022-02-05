use crate::{data::Data, style::Length, Color, SideOffsets};

use std::{
    any::Any,
    collections::HashMap,
    hash::{Hash, Hasher},
    marker::PhantomData,
    sync::Arc,
};
//use crate::style::StyleSet;

/// A type that identifies a named value in an [`Environment`], of a particular type `T`.
///
/// FIXME: this trait and the helper macro `impl_key` is only there to allow default values for
/// types that cannot be created in const contexts. If we decide to remove compile-time default
/// values for environment keys, then it might be cleaner to revert to representing keys with
/// const `Key` values instead of `impl Key` types.
#[derive(Debug, Eq, PartialEq)]
pub struct EnvKey<T> {
    key: &'static str,
    _type: PhantomData<T>,
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

/// Trait implemented by values that can be stored in an environment.
pub trait EnvValue: Sized + Any + Clone {
    fn as_any(&self) -> &dyn Any;
}

macro_rules! impl_env_value {
    ($t:ty) => {
        impl EnvValue for $t {
            fn as_any(&self) -> &dyn Any {
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

#[derive(Clone)]
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

impl EnvImpl {
    fn get<T>(&self, key: EnvKey<T>) -> Option<T>
    where
        T: EnvValue,
    {
        self.values
            .get(key.key)
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

    /// Creates a new environment that adds or overrides a given key.
    pub fn add<T>(mut self, key: EnvKey<T>, value: T) -> Environment
    where
        T: EnvValue,
    {
        match Arc::get_mut(&mut self.0) {
            Some(env) => {
                env.values.insert(key.key, Arc::new(value));
                self
            }
            None => {
                let mut child_env = EnvImpl {
                    parent: Some(self.0.clone()),
                    values: HashMap::new(),
                };
                child_env.values.insert(key.key, Arc::new(value));
                Environment(Arc::new(child_env))
            }
        }
    }

    /// Returns the value corresponding to the key.
    pub fn get<T>(&self, key: EnvKey<T>) -> Option<T>
    where
        T: EnvValue,
    {
        self.0.get(key)
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
