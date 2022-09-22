use serde::{Deserialize, Serialize};

use crate::{managed::value::Value, ticket_table::Ticket};
#[derive(Debug, Deserialize, Serialize)]
pub struct InvokeManaged {
    pub mod_: Vec<u8>,
    pub arg: Value,
    pub initial_storage: Value,
    pub constants: Vec<(i32, Value)>,
    pub tickets: Vec<Ticket>,
    pub source: String,
    pub sender: String,
    pub self_addr: String,
    pub gas_limit: usize,
}

#[derive(Debug, Deserialize, Serialize)]
enum Incoming {
    OriginateManaged { payload: String },
    InvokeManaged { payload: Box<InvokeManaged> },
}
