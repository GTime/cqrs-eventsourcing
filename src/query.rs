use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

use crate::{Aggregate, DomainEvent, FormatedEvent};

pub trait Query<A, E>: Debug + Default + Serialize + DeserializeOwned
where
    A: Aggregate,
    E: DomainEvent<A>,
{
    fn populate(&mut self, event: &FormatedEvent<A, E>);
}
