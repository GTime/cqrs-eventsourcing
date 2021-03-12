use async_trait::async_trait;

use crate::{Aggregate, AggregateContext, DomainEvent, Error, Store};

/// Command handler
#[async_trait]
pub trait Command<A, E>: Clone + Sync + Send
where
    A: Aggregate,
    E: DomainEvent<A>,
{
    fn id(&self) -> Option<String>;

    async fn handle(self, aggregate_context: &AggregateContext<A>) -> Result<Vec<E>, Error>;

    async fn before<S: Store<A, E>>(command: Self, _store: &S) -> Result<Self, Error>
    where
        Self: Sized,
    {
        Ok(command)
    }
}
