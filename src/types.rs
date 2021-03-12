use std::collections::HashMap;

use crate::{Error, FormatedEvent};

pub type MetaData = HashMap<String, String>;
pub type FormatedEvents<A, E> = Vec<FormatedEvent<A, E>>;
pub type FormatedResult<A, E> = Result<FormatedEvents<A, E>, Error>;
