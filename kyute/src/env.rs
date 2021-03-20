use crate::{style::StyleCollection, BoxedWidget, SideOffsets};
use kyute_shell::drawing::Color;
use std::{
    any::{Any, TypeId},
    borrow::Borrow,
    collections::HashMap,
    fmt,
    marker::PhantomData,
    rc::Rc,
};

/// A type that identifies a named value in an [`Environment`], of a particular type `T`.
///
/// FIXME: this trait and the helper macro `impl_key` is only there to allow default values for
/// types that cannot be created in const contexts. If we decide to remove compile-time default
/// values for environment keys, then it might be cleaner to revert to representing keys with
/// const `Key` values instead of `impl Key` types.
pub trait Key<'a> {
    type Value: EnvValue<'a>;
    const NAME: &'static str;
    fn default() -> Self::Value;
}

#[macro_export]
macro_rules! impl_keys {
    ($($(#[$outer:meta])* $name:ident : $valty:ty [$default:expr];)*) => {
        $(
            $(#[$outer])*
            #[derive(Copy,Clone,Debug,Eq,PartialEq,Hash)]
            pub struct $name;

            $(#[$outer])*
            impl<'a> Key<'a> for $name {
                const NAME: &'static str = stringify!($name);
                type Value = $valty;
                fn default() -> $valty {
                    $default
                }
            }
        )*
    };
}

/// Trait implemented by values that can be stored in an environment.
pub trait EnvValue<'a>: Sized {
    fn into_storage(self) -> EnvValueStorage;
    fn try_from_storage(storage: &'a EnvValueStorage) -> Option<Self>;
}

macro_rules! impl_env_value_builtin {
    ($t:ty; $variant:ident) => {
        impl<'a> EnvValue<'a> for $t {
            fn into_storage(self) -> EnvValueStorage {
                EnvValueStorage::$variant(self)
            }
            fn try_from_storage(storage: &'a EnvValueStorage) -> Option<Self> {
                match storage {
                    EnvValueStorage::$variant(x) => Some(*x),
                    _ => None,
                }
            }
        }
    };
}

impl_env_value_builtin!(bool; Bool);
impl_env_value_builtin!(f64; F64);
impl_env_value_builtin!(Color; Color);
impl_env_value_builtin!(SideOffsets; SideOffsets);

/// String slices in environment.
impl<'a> EnvValue<'a> for &'a str {
    fn into_storage(self) -> EnvValueStorage {
        EnvValueStorage::String(self.to_string())
    }

    fn try_from_storage(storage: &'a EnvValueStorage) -> Option<Self> {
        match storage {
            EnvValueStorage::String(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct Environment(Rc<EnvImpl>);

pub enum EnvValueStorage {
    Bool(bool),
    F64(f64),
    Color(Color),
    SideOffsets(SideOffsets),
    String(String),
    Other(Box<dyn Any>),
}

struct EnvImpl {
    parent: Option<Rc<EnvImpl>>,
    values: HashMap<&'static str, EnvValueStorage>,
}

// <'a> Key<'a> => Value: &'a str, Value::Store = String

impl EnvImpl {
    fn get<'a, K: Key<'a>>(&'a self, key: K) -> K::Value {
        self.values
            .get(K::NAME)
            .map(|v| K::Value::try_from_storage(v).expect("unexpected type of environment value"))
            .or_else(|| self.parent.as_ref().map(|parent| parent.get(key)))
            .unwrap_or_else(|| K::default())
    }
}

impl Environment {
    /// Creates a new, empty environment.
    pub fn new() -> Environment {
        Environment(Rc::new(EnvImpl {
            parent: None,
            values: HashMap::new(),
        }))
    }

    /// Creates a new environment that adds or overrides a given key.
    pub fn add<'a, K: Key<'a>>(mut self, _key: K, value: K::Value) -> Environment {
        match Rc::get_mut(&mut self.0) {
            Some(env) => {
                env.values.insert(K::NAME, value.into_storage());
                self
            }
            None => {
                let mut child_env = EnvImpl {
                    // note: the compiler seems smart enough to understand that the borrow of `self.0` is not held here?
                    parent: Some(self.0.clone()),
                    values: HashMap::new(),
                };
                child_env.values.insert(K::NAME, value.into_storage());
                Environment(Rc::new(child_env))
            }
        }
    }

    /// Returns the value corresponding to the key.
    pub fn get<'a, K: Key<'a>>(&'a self, key: K) -> K::Value {
        self.0.get(key)
    }
}
