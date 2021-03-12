use async_trait::async_trait;
use std::collections::HashMap;
use std::marker::PhantomData;

use crate::{
    Aggregate, AggregateContext, Command, DomainEvent, Error, FormatedEvent, FormatedResult,
    MetaData, Store,
};

pub type GivenThen<A, E, C> = GivenThenTest<A, E, TestStore<A, E>, C>;

pub struct GivenThenTest<A, E, S, C>
where
    A: Aggregate,
    E: DomainEvent<A>,
    S: Store<A, E>,
    C: Command<A, E>,
{
    given: Vec<E>,
    when: Option<C>,
    then: Vec<E>,
    then_error: Option<Error>,
    _a: PhantomData<A>,
    _s: PhantomData<S>,
}

impl<A, E, S, C> GivenThenTest<A, E, S, C>
where
    A: Aggregate,
    E: DomainEvent<A>,
    S: Store<A, E>,
    C: Command<A, E>,
{
    pub fn new() -> GivenThenTest<A, E, S, C> {
        GivenThenTest {
            given: Vec::new(),
            when: None,
            then: Vec::new(),
            then_error: None,
            _a: PhantomData,
            _s: PhantomData,
        }
    }
    pub fn given(self, events: Vec<E>) -> GivenThenTest<A, E, S, C> {
        GivenThenTest {
            given: events,
            ..self
        }
    }

    pub fn when(self, command: C) -> GivenThenTest<A, E, S, C> {
        GivenThenTest {
            when: Some(command),
            ..self
        }
    }

    pub fn then(self, expected: Vec<E>) -> GivenThenTest<A, E, S, C> {
        GivenThenTest {
            then: expected,
            ..self
        }
    }

    pub fn then_error(self, expected_error: Error) -> GivenThenTest<A, E, S, C> {
        GivenThenTest {
            then_error: Some(expected_error),
            ..self
        }
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        match self.execute().await {
            Ok(events) => {
                assert_eq!(events, self.then); // Check for Then
            }
            Err(e) => {
                assert_eq!(self.then_error, Some(e)); // Check for Then Error
            }
        };

        Ok(())
    }

    async fn execute(&mut self) -> Result<Vec<E>, Error> {
        let command = match &self.when {
            Some(c) => c,
            None => {
                return Err(Error::new(
                    "GivenThenTest `when` was not provided",
                    Some("INTERNAL"),
                    None,
                ))
            }
        };

        let store = TestStore::new(self.given.clone());
        let cmd = C::before(command.clone(), &store).await?;
        let context = &store.assemble_aggregate(cmd.id()).await?;
        let generated = &cmd.handle(context).await?;

        Ok(generated.clone())
    }
}

/// TestStore
pub struct TestStore<A: Aggregate, E: DomainEvent<A>> {
    events: Vec<FormatedEvent<A, E>>,
    _a: PhantomData<A>,
    _e: PhantomData<E>,
}

impl<A: Aggregate, E: DomainEvent<A>> TestStore<A, E> {
    pub fn new(given: Vec<E>) -> TestStore<A, E> {
        let formated = FormatedEvent::create_many(
            "86d786e8-4e24-4abf-b2f3-ccd24e606335",
            0,
            given,
            HashMap::new(),
        );

        TestStore {
            events: formated,
            _a: PhantomData,
            _e: PhantomData,
        }
    }
}

impl<A: Aggregate, E: DomainEvent<A>> Clone for TestStore<A, E> {
    fn clone(&self) -> TestStore<A, E> {
        TestStore {
            events: self.events.clone(),
            _a: PhantomData,
            _e: PhantomData,
        }
    }
}

#[async_trait]
impl<A: Aggregate, E: DomainEvent<A>> Store<A, E> for TestStore<A, E> {
    /// Rebuilding the aggregate
    async fn assemble_aggregate(&self, id: Option<String>) -> Result<AggregateContext<A>, Error> {
        let mut context = AggregateContext::default();
        context.set_id(id.clone());

        // Populate aggregate if id is provided
        if let Some(_x) = id {
            for fmt_event in &self.events {
                fmt_event.payload.clone().apply(&mut context.aggregate);
                context.version = fmt_event.version;
            }
        }

        Ok(context)
    }

    ///  Append formated events to store
    async fn append(
        &self,
        events: Vec<E>,
        context: AggregateContext<A>,
        meta: MetaData,
    ) -> FormatedResult<A, E> {
        let formated_events =
            FormatedEvent::create_many(context.id.as_str(), context.version, events, meta);

        if formated_events.len() == 0 {
            return Ok(Vec::default());
        }

        Ok(formated_events)
    }

    /// Retrive Events for command store
    async fn retrieve(&self, aggregate_id: &str) -> FormatedResult<A, E> {
        let mut filtered_events = Vec::new();

        for e in self.events.iter() {
            if e.aggregate_id == aggregate_id && e.aggregate_type == A::aggregate_type() {
                filtered_events.push(e.clone());
            }
        }

        Ok(filtered_events)
    }

    /** Retrive Events for query */
    async fn retrieve_for_query(&self, aggregate_id: Option<&str>) -> FormatedResult<A, E> {
        let mut filtered_events = Vec::new();

        let id = match aggregate_id {
            Some(i) => i,
            None => "",
        };

        for e in self.events.iter() {
            if e.aggregate_id == id || e.aggregate_type == A::aggregate_type() {
                filtered_events.push(e.clone());
            }
        }

        Ok(filtered_events)
    }
}
