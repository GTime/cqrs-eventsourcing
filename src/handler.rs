use async_trait::async_trait;

use crate::{Aggregate, DomainEvent, FormatedEvent};

pub type Handlers<A, E> = Vec<Box<dyn Handler<A, E> + Send>>;

#[async_trait]
pub trait Handler<A, E>
where
    A: Aggregate,
    E: DomainEvent<A>,
{
    async fn handle(&self, events: &Vec<FormatedEvent<A, E>>);
}
