use std::marker::PhantomData;

use crate::{Aggregate, Command, DomainEvent, Error, Handlers, MetaData, Store};

// #[derive()]
pub struct CQRS<A, E, ES>
where
    A: Aggregate,
    E: DomainEvent<A>,
    ES: Store<A, E>,
{
    handlers: Handlers<A, E>,
    store: ES,
    _a: PhantomData<A>,
    _e: PhantomData<E>,
}

impl<A, E, ES> CQRS<A, E, ES>
where
    A: Aggregate,
    E: DomainEvent<A>,
    ES: Store<A, E>,
{
    pub fn new(store: ES, handlers: Handlers<A, E>) -> CQRS<A, E, ES> {
        Self {
            store,
            handlers,
            _a: PhantomData,
            _e: PhantomData,
        }
    }

    pub async fn execute<C: Command<A, E>>(
        &mut self,
        command: C,
        meta: MetaData,
    ) -> Result<(), Error> {
        // Call command's before
        let cmd = C::before(command, &self.store).await?;

        // Assemble Aggragate
        let id = &cmd.id();
        let aggregate_context = self.store.assemble_aggregate(id.clone()).await?;

        // Handle Command
        let generated_events = cmd.handle(&aggregate_context).await?;

        // Store New Events
        let commited_events = &self
            .store
            .append(generated_events, aggregate_context, meta)
            .await?;

        // Run Handlers
        for handler in &self.handlers {
            handler.handle(commited_events).await;
        }

        Ok(())
    }
}
