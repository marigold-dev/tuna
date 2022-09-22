use serde::{de::Visitor, Deserialize};

use crate::{
    contract_address::ContractAddress,
    managed::value::Value,
    outgoing::{Init, SetOwned},
    ticket_table::TicketId,
};

#[derive(Deserialize, Debug)]
#[serde(tag = "type_", content = "content")]
pub enum Operation {
    Originate {
        module: String,
        constants: Vec<(u32, Value)>,
        initial_storage: Value,
    },
    Invoke {
        address: ContractAddress,
        argument: Value,
        #[serde(default = "def")]
        gas_limit: u64,
    },
}
fn def() -> u64 {
    u64::MAX
}
#[derive(Debug, Deserialize)]
pub struct Transaction {
    pub source: String,
    pub operation: String,
    pub operation_raw_hash: String,
    pub tickets: Vec<(TicketId, u32)>,
}
#[derive(Debug)]
pub enum ClientMessage {
    Transaction(Transaction),
    Set(SetOwned),
    GetInitialState,
    SetInitialState(Init),
    Get(ContractAddress),
    GiveTickets(Vec<(TicketId, u32)>),
}
struct ClientVisitor;
impl<'de> Visitor<'de> for ClientVisitor {
    type Value = ClientMessage;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Expected a valid Value")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        seq.next_element::<&str>()?.map_or_else(
            || {
                Err(serde::de::Error::invalid_type(
                    serde::de::Unexpected::Str("unexpected sequence"),
                    &"value",
                ))
            },
            |x| match x {
                "Set" => {
                    let elem: Option<SetOwned> = seq.next_element()?;
                    elem.map_or_else(
                        || {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str("unexpected sequence"),
                                &"value",
                            ))
                        },
                        |x| Ok(ClientMessage::Set(x)),
                    )
                }
                "Give_Tickets" => {
                    let elem: Option<Vec<(TicketId, u32)>> = seq.next_element()?;
                    elem.map_or_else(
                        || {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str("unexpected sequence"),
                                &"value",
                            ))
                        },
                        |x| Ok(ClientMessage::GiveTickets(x)),
                    )
                }
                "Get_Initial_State" => Ok(ClientMessage::GetInitialState),
                "Transaction" => {
                    let elem: Option<Transaction> = seq.next_element()?;
                    elem.map_or_else(
                        || {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str("unexpected sequence"),
                                &"value",
                            ))
                        },
                        |x| Ok(ClientMessage::Transaction(x)),
                    )
                }
                "Get" => {
                    let elem: Option<ContractAddress> = seq.next_element()?;
                    elem.map_or_else(
                        || {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str("unexpected sequence"),
                                &"value",
                            ))
                        },
                        |x| Ok(ClientMessage::Get(x)),
                    )
                }
                "Set_Initial_State" => {
                    let elem: Option<Init> = seq.next_element()?;
                    elem.map_or_else(
                        || {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str("unexpected sequence"),
                                &"value",
                            ))
                        },
                        |x| Ok(ClientMessage::SetInitialState(x)),
                    )
                }
                _ => Err(serde::de::Error::invalid_type(
                    serde::de::Unexpected::Str("unexpected sequence"),
                    &x,
                )),
            },
        )
    }
}
impl<'de> Deserialize<'de> for ClientMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_tuple(2, ClientVisitor)
    }
}
