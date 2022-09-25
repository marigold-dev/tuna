use wasmer::Module;

use crate::{managed::value::Value, ticket_table::Ticket};
pub struct InvokeManaged {
    pub mod_: Module,
    pub arg: Value,
    pub initial_storage: Value,
    pub constants: Vec<(i32, Value)>,
    pub tickets: Vec<Ticket>,
    pub source: String,
    pub sender: String,
    pub self_addr: String,
    pub gas_limit: u64,
}
