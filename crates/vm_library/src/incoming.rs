#[derive(Debug, Deserialize, Serialize)]
pub struct Incoming {
    pub mod_: String,
    pub arg: Value,
    pub initial_storage: Value,
    pub tickets: Vec<Ticket>,
    pub source: String,
    pub sender: String,
    pub self_addr: String,
    pub gas_limit: usize,
}

use serde::{Deserialize, Serialize};

use crate::{managed::value::Value, ticket::Ticket};
