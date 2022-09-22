use serde::{Deserialize, Serialize};

use crate::{managed::value::Value, ticket_table::Ticket};

#[derive(Deserialize, Serialize)]
pub struct OutgoingManaged {
    pub new_storage: Value,
    pub operations: Value,
    pub contract_tickets: Vec<Ticket>,
    pub remaining_gas: usize,
}

#[derive(Deserialize, Serialize)]
pub enum Outgoing {
    OutgoingManaged { payload: Box<OutgoingManaged> },
    CheckContract { payload: String },
}
