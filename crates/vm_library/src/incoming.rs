pub struct Incoming {
    pub mod_: String,
    pub arg: DefaultKey,
    pub initial_storage: DefaultKey,
    pub tickets: Vec<Ticket>,
    pub source: String,
    pub sender: String,
    pub self_addr: String,
    pub gas_limit: usize,
}

use std::{cell::RefCell, fmt, rc::Rc};

use serde::{
    de::{self, DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor},
    Deserialize,
};
use slotmap::{DefaultKey, HopSlotMap};

use crate::{
    managed::value::{Value, WithValueDeser},
    ticket::Ticket,
};
struct VV {
    arena: Rc<RefCell<HopSlotMap<DefaultKey, Value>>>,
}
impl<'de> DeserializeSeed<'de> for VV {
    type Value = Incoming;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            MOD_,
            Arg,
            InitialStorage,
            Tickets,
            Source,
            Sender,
            SelfAddr,
            GasLimit,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`secs` or `nanos`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "mod_" => Ok(Field::MOD_),
                            "arg" => Ok(Field::Arg),
                            "initial_storage" => Ok(Field::InitialStorage),
                            "tickets" => Ok(Field::Tickets),

                            "source" => Ok(Field::Source),
                            "sender" => Ok(Field::Sender),
                            "self_address" => Ok(Field::SelfAddr),
                            "gas_limit" => Ok(Field::GasLimit),

                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct TV {
            arena: Rc<RefCell<HopSlotMap<DefaultKey, Value>>>,
        }

        impl<'de> Visitor<'de> for TV {
            type Value = Incoming;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Duration")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mod_ = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let arg = seq
                    .next_element_seed(WithValueDeser {
                        arena: Rc::clone(&self.arena),
                    })?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let arg = self.arena.as_ref().borrow_mut().insert(arg);
                let initial_storage = seq
                    .next_element_seed(WithValueDeser {
                        arena: Rc::clone(&self.arena),
                    })?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let initial_storage = self.arena.as_ref().borrow_mut().insert(initial_storage);
                let tickets = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let source = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let sender = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let self_addr = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let gas_limit = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                Ok(Incoming {
                    mod_,
                    arg,
                    initial_storage,
                    tickets,
                    sender,
                    source,
                    self_addr,
                    gas_limit,
                })
            }

            fn visit_map<V>(self, mut map: V) -> Result<Incoming, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut mod_ = None;
                let mut arg = None;
                let mut initial_storage = None;
                let mut tickets = None;
                let mut sender = None;
                let mut source = None;
                let mut self_addr = None;
                let mut gas_limit = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::MOD_ => {
                            if mod_.is_some() {
                                return Err(de::Error::duplicate_field("secs"));
                            }
                            mod_ = Some(map.next_value()?);
                        }
                        Field::Arg => {
                            if arg.is_some() {
                                return Err(de::Error::duplicate_field("nanos"));
                            }
                            arg = Some(map.next_value_seed(WithValueDeser {
                                arena: Rc::clone(&self.arena),
                            })?);
                        }
                        Field::InitialStorage => {
                            if initial_storage.is_some() {
                                return Err(de::Error::duplicate_field("nanos"));
                            }
                            initial_storage = Some(map.next_value_seed(WithValueDeser {
                                arena: Rc::clone(&self.arena),
                            })?);
                        }
                        Field::Tickets => {
                            if tickets.is_some() {
                                return Err(de::Error::duplicate_field("secs"));
                            }
                            tickets = Some(map.next_value()?);
                        }
                        Field::Source => {
                            if source.is_some() {
                                return Err(de::Error::duplicate_field("secs"));
                            }
                            source = Some(map.next_value()?);
                        }
                        Field::Sender => {
                            if sender.is_some() {
                                return Err(de::Error::duplicate_field("secs"));
                            }
                            sender = Some(map.next_value()?);
                        }
                        Field::SelfAddr => {
                            if self_addr.is_some() {
                                return Err(de::Error::duplicate_field("secs"));
                            }
                            self_addr = Some(map.next_value()?);
                        }

                        Field::GasLimit => {
                            if gas_limit.is_some() {
                                return Err(de::Error::duplicate_field("secs"));
                            }
                            gas_limit = Some(map.next_value()?);
                        }
                    }
                }
                let mod_ = mod_.ok_or_else(|| de::Error::missing_field("secs"))?;
                let arg = arg.ok_or_else(|| de::Error::missing_field("nanos"))?;
                let sender = sender.ok_or_else(|| de::Error::missing_field("secs"))?;
                let self_addr = self_addr.ok_or_else(|| de::Error::missing_field("nanos"))?;
                let source = source.ok_or_else(|| de::Error::missing_field("secs"))?;
                let tickets = tickets.ok_or_else(|| de::Error::missing_field("nanos"))?;
                let initial_storage =
                    initial_storage.ok_or_else(|| de::Error::missing_field("nanos"))?;
                let mut arena = self.arena.as_ref().borrow_mut();
                let initial_storage = arena.insert(initial_storage);
                let arg = arena.insert(arg);
                let gas_limit = gas_limit.ok_or_else(|| de::Error::missing_field("nanos"))?;

                Ok(Incoming {
                    mod_,
                    arg,
                    initial_storage,
                    self_addr,
                    sender,
                    source,
                    tickets,
                    gas_limit,
                })
            }
        }

        const FIELDS: &[&str] = &[
            "mod_",
            "arg",
            "initial_storage",
            "tickets",
            "source",
            "sender",
            "self_address",
        ];
        deserializer.deserialize_struct("T", FIELDS, TV { arena: self.arena })
    }
}
