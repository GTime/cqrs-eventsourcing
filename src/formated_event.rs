use chrono::prelude::*;
use serde::Serialize;
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::{Aggregate, DomainEvent, FormatedEvents, MetaData};

#[derive(Debug, Serialize)]
pub struct FormatedEvent<A, E>
where
    A: Aggregate,
    E: DomainEvent<A>,
{
    pub aggregate_id: String,
    pub aggregate_type: String,
    pub version: usize,
    pub payload: E,
    pub meta: MetaData,
    pub created_at: String,
    pub(crate) _phantom: PhantomData<A>,
}

impl<A, E> Clone for FormatedEvent<A, E>
where
    A: Aggregate,
    E: DomainEvent<A>,
{
    fn clone(&self) -> FormatedEvent<A, E> {
        FormatedEvent {
            aggregate_id: self.aggregate_id.clone(),
            aggregate_type: self.aggregate_type.clone(),
            version: self.version,
            payload: self.payload.clone(),
            meta: self.meta.clone(),
            created_at: self.created_at.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<A, E> FormatedEvent<A, E>
where
    A: Aggregate,
    E: DomainEvent<A>,
{
    /// Create a new FormatedEvent
    pub fn new(
        aggregate_id: String,
        aggregate_type: String,
        version: usize,
        payload: E,
        meta: MetaData,
        created_at: Option<&str>,
    ) -> FormatedEvent<A, E> {
        FormatedEvent {
            aggregate_id,
            aggregate_type,
            version,
            payload,
            meta,
            created_at: match created_at {
                Some(x) => x.to_string(),
                None => Utc::now().to_rfc2822(),
            },
            _phantom: PhantomData,
        }
    }

    /// Create FormatedEvents from DomainEvents
    pub fn create_many(
        aggregate_id: &str,
        current_version: usize,
        events: Vec<E>,
        meta: MetaData,
    ) -> FormatedEvents<A, E> {
        let mut formated_events: FormatedEvents<A, E> = Vec::new();
        let mut version = current_version;

        for payload in events {
            let aggregate_type = A::aggregate_type().to_string();

            version += 1;

            formated_events.push(FormatedEvent::new(
                aggregate_id.to_string(),
                aggregate_type,
                version,
                payload,
                meta.clone(),
                None,
            ))
        }

        formated_events
    }
}
