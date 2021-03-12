use async_trait::async_trait;

use crate::{Aggregate, AggregateContext, DomainEvent, Error, FormatedResult, MetaData};

#[async_trait]
pub trait Store<A, E>: Clone + Sync + Send
where
    A: Aggregate,
    E: DomainEvent<A>,
{
    /// Rebuilding the aggregate
    async fn assemble_aggregate(&self, id: Option<String>) -> Result<AggregateContext<A>, Error>;

    /// Append formated events to store
    async fn append(
        &self,
        events: Vec<E>,
        context: AggregateContext<A>,
        meta: MetaData,
    ) -> FormatedResult<A, E>;

    /// Retrive Events for command store
    async fn retrieve(&self, aggregate_id: &str) -> FormatedResult<A, E>;

    /// Retrive Events for query
    async fn retrieve_for_query(&self, aggregate_id: Option<&str>) -> FormatedResult<A, E>;
}
