#[derive(Debug, Deserialize, Serialize)]
pub struct Incoming<'a> {
    #[serde(borrow)]
    pub mod_: Cow<'a, str>,
    pub arg: Value,
    pub initial_storage: Value,
    pub tickets: Vec<Ticket>,
    #[serde(borrow)]
    pub source: Cow<'a, str>,
    #[serde(borrow)]
    pub sender: Cow<'a, str>,
    #[serde(borrow)]
    pub self_addr: Cow<'a, str>,
    pub gas_limit: usize,
}

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::{managed::value::Value, ticket::Ticket};
