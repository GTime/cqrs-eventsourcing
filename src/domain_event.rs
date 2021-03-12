use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

use crate::Aggregate;

pub trait DomainEvent<A: Aggregate>:
    Serialize + DeserializeOwned + Clone + PartialEq + Debug + Sync + Send
{
    fn apply(self, aggregate: &mut A);
    fn name() -> &'static str;
}
