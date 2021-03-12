use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use uuid::Uuid;

pub trait Aggregate: Debug + Default + Serialize + DeserializeOwned + Sync + Send {
    fn aggregate_type() -> &'static str;
}

pub struct AggregateContext<A: Aggregate> {
    pub id: String,
    pub version: usize,
    pub aggregate: A,
}

impl<A: Aggregate> AggregateContext<A> {
    pub fn aggregate(&self) -> &A {
        &self.aggregate
    }

    pub fn set_id(&mut self, id: Option<String>) {
        self.id = match id {
            Some(x) => x,
            None => Uuid::new_v4().to_string(),
        };
    }
}

impl<A: Aggregate> Default for AggregateContext<A> {
    fn default() -> AggregateContext<A> {
        AggregateContext {
            id: String::default(),
            version: 0_usize,
            aggregate: A::default(),
        }
    }
}
