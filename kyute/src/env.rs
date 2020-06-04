use std::any::{TypeId, Any};
use kyute_shell::drawing::Color;
use crate::layout::SideOffsets;
use crate::BoxedWidget;
use std::rc::Rc;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::borrow::Borrow;
use std::fmt;

/// A type that identifies a named value in an [`Environment`], of a particular type `T`.
///
/// FIXME: this trait and the helper macro `impl_key` is only there to allow default values for
/// types that cannot be created in const contexts. If we decide to remove compile-time default
/// values for environment keys, then it might be cleaner to revert to representing keys with
/// const `Key` values instead of `impl Key` types.
pub trait Key<'a> {
    type Value: EnvValue<'a>;
    const NAME: &'static str;
    fn default() -> <Self::Value as EnvValue<'a>>::Borrowed;
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
                fn default() -> <Self::Value as $crate::env::EnvValue<'a>>::Borrowed {
                    $default
                }
            }
        )*
    };
}


/// Trait implemented by values that can be stored in an environment.
///
/// This trait is implemented by default for all `T: Any + Sized` with `Sized = T`,
/// but you might want to implement it also for some unsized types or trait object types
/// (`dyn Trait`), by specifying a wrapper type for storage (typically `Box<T>`).
pub trait EnvValue<'a>: Any {
    /// The actual, sized type of the value stored in Env.
    type Borrowed;
    fn to_borrowed(&'a self) -> Self::Borrowed;
}

// all copy types are copied, not borrowed
// FIXME this does not work:
// error[E0119]: conflicting implementations of trait `env::EnvValue<'_>` for type `std::string::String`
// because "upstream crates may add a new impl of trait `std::marker::Copy` for type `std::string::String` in future versions" (lol)
// -> So:
//      - we need to impl EnvValue manually for all types that we want to put in env
//      - in turn, users of the library won't be able to implement this trait for foreign types. PERFECT.
// TODO:
// - evaluate what kind of stuff we actually want to put in the environment
// - consider removing static key default values, replace by a function that does the initialization
// -
impl<'a, T: Copy> EnvValue<'a> for T {
    type Borrowed = T;
    fn to_borrowed(&'a self) -> T { *self }
}

// strings are stored as String, but borrowed
impl<'a> EnvValue<'a> for String {
    type Borrowed = &'a str;
    fn to_borrowed(&'a self) -> &'a str {
        &*self
    }
}

#[derive(Clone)]
pub struct Environment(Rc<EnvImpl>);

struct EnvImpl {
    parent: Option<Rc<EnvImpl>>,
    values: HashMap<&'static str, Box<dyn Any>>
}

// <'a> Key<'a> => Value: &'a str, Value::Store = String

impl EnvImpl {
    fn get<'a, K: Key<'a>>(&'a self, key: K) -> <K::Value as EnvValue<'a>>::Borrowed
    {
        self.values.get(K::NAME)
            .map(|v| v.downcast_ref::<K::Value>().expect("unexpected type of environment value").to_borrowed())
            .or_else(|| self.parent.and_then(|parent| parent.get(key)))
            .or_else(|| K::default())
    }
}

impl Environment {
    /// Creates a new, empty environment.
    pub fn new() -> Environment {
        Environment(Rc::new(EnvImpl {
            parent: None,
            values: HashMap::new()
        }))
    }

    /// Creates a new environment that adds or overrides a given key.
    pub fn add<'a, K: Key<'a>>(mut self, _key: K, value: K::Value) -> Environment
    {
        match Rc::get_mut(&mut self.0) {
            Some(env) => {
                env.values.insert(K::NAME, Box::new(value));
                self
            }
            None => {
                let mut child_env = EnvImpl {
                    // note: the compiler seems smart enough to understand that the borrow of `self.0` is not held here?
                    parent: Some(self.0.clone()),
                    values: HashMap::new()
                };
                child_env.values.insert(K::NAME, Box::new(value));
                Environment(Rc::new(child_env))
            }
        }
    }

    /// Returns the value corresponding to the key.
    pub fn get<'a, K: Key<'a>>(&'a self, key: K) -> <K::Value as EnvValue<'a>>::Borrowed {
        self.0.get(key)
    }
}
