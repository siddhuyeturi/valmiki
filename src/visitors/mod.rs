mod middleware;

pub use middleware::{VisitorMiddleware, VisitorMiddlewareService};

use std::rc::Rc;

use rand::{Rng, distr::Alphanumeric};

#[derive(Debug, Clone)]
pub struct VisitorId(Rc<str>);

impl VisitorId {
    fn generate(len: usize) -> Self {
        Self(
            rand::rng()
                .sample_iter(Alphanumeric)
                .take(len)
                .map(char::from)
                .collect::<String>()
                .into(),
        )
    }
}

impl AsRef<str> for VisitorId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<T> From<T> for VisitorId
where
    T: Into<Rc<str>>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}
