use async_trait::async_trait;
use cqrs_eventsourcing::{
    Aggregate, AggregateContext, Command, DomainEvent, Error, FileEventStore, FormatedEvent,
    GivenThen, Query, QueryProcessor, Store, CQRS,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod mock;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Dispatch {
    id: String,
    client: String,
    dispatcher: String,
    accepted_at: Option<String>,
}

impl Default for Dispatch {
    fn default() -> Dispatch {
        Dispatch {
            id: Default::default(),
            client: Default::default(),
            dispatcher: Default::default(),
            accepted_at: Default::default(),
        }
    }
}

impl Aggregate for Dispatch {
    fn aggregate_type() -> &'static str {
        "dispatch"
    }
}

// EVENTS

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
enum DispatchEvent {
    Requested(Requested),
    Accepted(Accepted),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct Requested {
    id: String,
    client: String,
    dispatcher: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct Accepted {
    dispatcher: String,
    accepted_at: String,
}

impl DomainEvent<Dispatch> for DispatchEvent {
    fn apply(self, dispatch: &mut Dispatch) {
        match self {
            DispatchEvent::Requested(e) => e.apply(dispatch),
            DispatchEvent::Accepted(e) => e.apply(dispatch),
        }
    }
    fn name() -> &'static str {
        "Requested"
    }
}

impl DomainEvent<Dispatch> for Requested {
    fn apply(self, dispatch: &mut Dispatch) {
        dispatch.id = self.id;
        dispatch.dispatcher = self.dispatcher;
        dispatch.client = self.client;
    }

    fn name() -> &'static str {
        "Requested"
    }
}

impl DomainEvent<Dispatch> for Accepted {
    fn apply(self, dispatch: &mut Dispatch) {
        dispatch.accepted_at = Some(self.accepted_at)
    }
    fn name() -> &'static str {
        "Accepted"
    }
}

// QUERY
type DispatchQuery = QueryProcessor<Dispatch, DispatchEvent, DispatchQueryData>;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct DispatchQueryData {
    dispatches: Vec<Dispatch>,
}

impl Default for DispatchQueryData {
    fn default() -> DispatchQueryData {
        DispatchQueryData {
            dispatches: Vec::default(),
        }
    }
}

impl DispatchQueryData {
    fn is_assigned_to(self, id: &str) -> bool {
        self.dispatches.iter().any(|i| i.dispatcher == id)
    }
}

impl Query<Dispatch, DispatchEvent> for DispatchQueryData {
    fn populate(&mut self, event: &FormatedEvent<Dispatch, DispatchEvent>) {
        match &event.payload {
            DispatchEvent::Requested(e) => self.dispatches.push(Dispatch {
                id: e.id.clone(),
                client: e.client.clone(),
                dispatcher: e.dispatcher.clone(),
                accepted_at: None,
            }),
            DispatchEvent::Accepted(e) => {
                if let Some(pos) = &self
                    .dispatches
                    .iter()
                    .position(|i| i.id == event.aggregate_id)
                {
                    let accepted_at = Some(e.accepted_at.clone());
                    self.dispatches[pos.clone()].accepted_at = accepted_at;
                }
            }
        }
    }
}

// COMMAND

#[derive(Clone)]
struct Request {
    client: String,
    dispatcher: String,
}

#[async_trait]
impl Command<Dispatch, DispatchEvent> for Request {
    fn id(&self) -> Option<String> {
        None
    }

    async fn handle(
        self,
        _context: &AggregateContext<Dispatch>,
    ) -> Result<Vec<DispatchEvent>, Error> {
        let events = vec![DispatchEvent::Requested(Requested {
            id: mock::DISPATCHID.to_string(),
            client: self.client,
            dispatcher: self.dispatcher,
        })];

        Ok(events)
    }

    async fn before<S: Store<Dispatch, DispatchEvent>>(
        command: Request,
        _store: &S,
    ) -> Result<Request, Error> {
        Ok(Request { ..command })
    }
}

#[derive(Clone)]
struct Accept {
    id: String,
    dispatcher: String,
    _query: Option<DispatchQueryData>,
}

#[async_trait]
impl Command<Dispatch, DispatchEvent> for Accept {
    fn id(&self) -> Option<String> {
        None
    }

    async fn handle(
        self,
        _context: &AggregateContext<Dispatch>,
    ) -> Result<Vec<DispatchEvent>, Error> {
        // Check for valid dispatch assignment
        if let Some(query) = self._query {
            if !query.is_assigned_to(&self.dispatcher) {
                return Err(Error::new(
                    "You were not requested for this dispatch",
                    Some("USERINPUT"),
                    None,
                ));
            }
        }

        let payload = vec![DispatchEvent::Accepted(Accepted {
            dispatcher: self.dispatcher,
            accepted_at: mock::FIXEDDATE.to_string(),
        })];

        Ok(payload)
    }

    async fn before<S: Store<Dispatch, DispatchEvent>>(
        command: Accept,
        store: &S,
    ) -> Result<Accept, Error> {
        // Update _query
        if let Ok(q) = DispatchQuery::process(store, None).await {
            return Ok(Accept {
                _query: Some(q),
                ..command
            });
        }

        Ok(command)
    }
}

// Handler

#[cfg(test)]
mod given_then_dispatch_test {
    use super::*;

    #[tokio::test]
    async fn test_request() -> Result<(), Error> {
        let command = Request {
            client: mock::CLIENT.to_string(),
            dispatcher: mock::DISPATCHER.to_string(),
        };

        let expected = vec![DispatchEvent::Requested(Requested {
            id: mock::DISPATCHID.to_string(),
            client: mock::CLIENT.to_string(),
            dispatcher: mock::DISPATCHER.to_string(),
        })];

        GivenThen::new().when(command).then(expected).run().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_accept() -> Result<(), Error> {
        let given = vec![DispatchEvent::Requested(Requested {
            id: mock::DISPATCHID.to_string(),
            client: mock::CLIENT.to_string(),
            dispatcher: mock::DISPATCHER.to_string(),
        })];

        let command = Accept {
            id: mock::DISPATCHID.to_string(),
            // dispatcher: mock::DISPATCHER.to_string(),
            dispatcher: "ba2a54a4-367d-450c-8ef3-9b677d41ff1c".to_string(),
            _query: None,
        };

        let expected = vec![DispatchEvent::Accepted(Accepted {
            dispatcher: mock::DISPATCHER.to_string(),
            accepted_at: mock::FIXEDDATE.to_string(),
        })];

        let expected_error = Error::new(
            "You were not requested for this dispatch",
            Some("USERINPUT"),
            None,
        );

        GivenThen::new()
            .given(given)
            .when(command)
            // .then(expected)
            .then_error(expected_error)
            .run()
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod with_store_dispatch_test {
    use super::*;

    type Store = FileEventStore<Dispatch, DispatchEvent>;

    async fn setup_cqrs() -> Result<CQRS<Dispatch, DispatchEvent, Store>, Error> {
        let store = Store::new(mock::FILESTORE);
        Ok(CQRS::new(store, vec![]))
    }

    #[tokio::test]
    async fn test_request() -> Result<(), Error> {
        let mut cqrs = setup_cqrs().await?;
        let command = Request {
            client: mock::CLIENT.to_string(),
            dispatcher: mock::DISPATCHER.to_string(),
        };
        let result = cqrs.execute(command, HashMap::new()).await?;

        assert_eq!(result, ());

        Ok(())
    }

    #[tokio::test]
    async fn test_accept() -> Result<(), Error> {
        let mut cqrs = setup_cqrs().await?;
        let command = Accept {
            id: mock::DISPATCHID.to_string(),
            dispatcher: mock::DISPATCHER.to_string(),
            _query: None,
        };

        let result = cqrs.execute(command, HashMap::new()).await?;

        assert_eq!(result, ());

        Ok(())
    }
}
