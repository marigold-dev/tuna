use serde::{Deserialize, Serialize};

use crate::{managed::value::Value, ticket::Ticket};

#[derive(Deserialize, Serialize)]
pub struct OutgoingManaged {
    pub new_storage: Value,
    pub operations: Vec<Value>,
    pub contract_tickets: Vec<Ticket>,
    pub remaining_gas: usize,
}

enum Outgoing {
    OutgoingManaged { payload: Box<OutgoingManaged> },
    CheckContract { payload: String },
}
