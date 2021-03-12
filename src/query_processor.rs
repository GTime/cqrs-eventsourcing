use std::marker::PhantomData;

use crate::{Aggregate, DomainEvent, Error, Query, Store};

pub struct QueryProcessor<A, E, Q>
where
    A: Aggregate,
    E: DomainEvent<A>,
    Q: Query<A, E>,
{
    _a: PhantomData<A>,
    _e: PhantomData<E>,
    _q: PhantomData<Q>,
}

impl<A, E, Q> QueryProcessor<A, E, Q>
where
    A: Aggregate,
    E: DomainEvent<A>,
    Q: Query<A, E>,
{
    pub async fn process<S: Store<A, E>>(
        store: &S,
        aggregate_id: Option<&str>,
    ) -> Result<Q, Error> {
        let mut query = Q::default();
        let events = store.retrieve_for_query(aggregate_id).await?;

        for event in events {
            query.populate(&event);
        }

        Ok(query)
    }
}
