use serde::{Deserialize, Serialize};

use crate::ticket::Ticket;

#[derive(Deserialize, Serialize)]
pub struct Incoming {
    pub new_storage: String,
    pub operations: String,
    pub contract_tickets: Vec<Ticket>,
    pub remaining_gas: usize,
}
