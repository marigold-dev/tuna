use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ticket {
    pub owner: String,
    pub issuer: String,
    pub data: String,
    pub amount: usize,
}
