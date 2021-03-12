use async_trait::async_trait;
use std::marker::PhantomData;
use std::{
    fs::{File, OpenOptions},
    io::{self, prelude::*},
    path::Path,
};

use crate::{
    Aggregate, AggregateContext, DomainEvent, Error, FormatedEvent, FormatedResult, Handlers,
    MetaData, Store, CQRS,
};

/// FileEventStore
///
/// NOTE: Only use the for develpment and not for production
pub struct FileEventStore<A: Aggregate, E: DomainEvent<A>> {
    path: String,
    _a: PhantomData<A>,
    _e: PhantomData<E>,
}

impl<A: Aggregate, E: DomainEvent<A>> FileEventStore<A, E> {
    pub fn new(path: &str) -> FileEventStore<A, E> {
        FileEventStore {
            path: path.to_owned(),
            _a: PhantomData,
            _e: PhantomData,
        }
    }

    /// Creates CQRS with store
    pub fn create_cqrs(path: &str, handlers: Handlers<A, E>) -> CQRS<A, E, FileEventStore<A, E>> {
        CQRS::new(FileEventStore::new(path), handlers)
    }

    /// Get store file from device
    pub fn get_file(&self) -> Result<File, Error> {
        let path = Path::new(&self.path);

        return match OpenOptions::new().read(true).append(true).open(path) {
            Ok(file) => Ok(file),
            Err(e) => match e.kind() {
                io::ErrorKind::NotFound => match File::create(path) {
                    Ok(file) => Ok(file),
                    _ => {
                        panic!("Oops! Error create store")
                    }
                },
                _ => {
                    panic!("Oops! Error open store")
                }
            },
        };
    }

    /// Find Events from store
    async fn read_file(&self) -> FormatedResult<A, E> {
        let mut file = self.get_file()?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let mut events = Vec::new();
        if content.len() > 0 {
            events = content
                .lines()
                .map(|e| {
                    let data: FileData = serde_json::from_str(&e).unwrap();
                    let payload: E = serde_json::from_str(&data.payload).unwrap();

                    return FormatedEvent::new(
                        data.aggregate_id,
                        data.aggregate_type,
                        data.version,
                        payload,
                        data.meta,
                        Some(&data.created_at),
                    );
                })
                .collect();
        }

        Ok(events)
    }
}

impl<A: Aggregate, E: DomainEvent<A>> Clone for FileEventStore<A, E> {
    fn clone(&self) -> FileEventStore<A, E> {
        FileEventStore {
            path: self.path.clone(),
            _a: PhantomData,
            _e: PhantomData,
        }
    }
}

#[async_trait]
impl<A: Aggregate, E: DomainEvent<A>> Store<A, E> for FileEventStore<A, E> {
    /// Rebuilding the aggregate
    async fn assemble_aggregate(&self, id: Option<String>) -> Result<AggregateContext<A>, Error> {
        let mut context = AggregateContext::default();
        context.set_id(id.clone());

        // Populate aggregate if id is provided
        if let Some(x) = id {
            for fmt_event in self.retrieve(x.as_str()).await? {
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

        let data =
            formated_events
                .iter()
                .fold(String::default(), |acc, cur| {
                    match serde_json::to_string(&cur) {
                        Ok(e) => format!("{}\n", e),
                        _ => acc,
                    }
                });

        if data.len() == 0 {
            return Ok(Vec::default());
        }

        // Insert into store
        let mut file = self.get_file()?;
        file.write_all(data.as_bytes())?;

        println!("[FileEventStore: Events Appended]\n");
        Ok(formated_events)
    }

    /// Retrive Events for command store
    async fn retrieve(&self, aggregate_id: &str) -> FormatedResult<A, E> {
        let events = self.read_file().await?;
        let mut filtered_events = Vec::new();

        for e in events.iter() {
            if e.aggregate_id == aggregate_id && e.aggregate_type == A::aggregate_type() {
                filtered_events.push(e.clone());
            }
        }

        Ok(filtered_events)
    }

    /// Retrive Events for query
    async fn retrieve_for_query(&self, aggregate_id: Option<&str>) -> FormatedResult<A, E> {
        let events = self.read_file().await?;
        let mut filtered_events = Vec::new();

        let id = match aggregate_id {
            Some(i) => i,
            None => "",
        };

        for e in events.iter() {
            if e.aggregate_id == id || e.aggregate_type == A::aggregate_type() {
                filtered_events.push(e.clone());
            }
        }

        Ok(filtered_events)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct FileData {
    pub aggregate_id: String,
    pub aggregate_type: String,
    pub version: usize,
    pub payload: String,
    pub meta: MetaData,
    pub created_at: String,
}
