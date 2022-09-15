use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use im_rc::{OrdMap, OrdSet, Vector};
use serde::{
    de::{DeserializeSeed, Visitor},
    ser::SerializeTuple,
    Serialize,
};
use slotmap::{DefaultKey, HopSlotMap};
type Arena<A> = HopSlotMap<DefaultKey, A>;
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone)]
pub enum Union {
    Left(DefaultKey),
    Right(DefaultKey),
}
unsafe impl Sync for Union {}
unsafe impl Send for Union {}
struct UnionSerializer<'a> {
    arena: Rc<RefCell<Arena<Value>>>,
    value: &'a Union,
}
impl<'a> Serialize for UnionSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use Union::*;

        match self.value {
            Right(x) => {
                let mut seq = serializer.serialize_tuple(2)?;

                seq.serialize_element("Right")?;
                let value1 = self
                    .arena
                    .as_ref()
                    .borrow_mut()
                    .remove(*x)
                    .map_or_else(|| Err(serde::ser::Error::custom(&"error serializing")), Ok)?;
                seq.serialize_element(&ValueSerializer {
                    arena: Rc::clone(&self.arena),
                    value: value1,
                })?;
                seq.end()
            }
            Left(x) => {
                let mut seq = serializer.serialize_tuple(2)?;
                seq.serialize_element("Left")?;
                let value1 = self
                    .arena
                    .as_ref()
                    .borrow_mut()
                    .remove(*x)
                    .map_or_else(|| Err(serde::ser::Error::custom(&"error serializing")), Ok)?;
                seq.serialize_element(&ValueSerializer {
                    arena: Rc::clone(&self.arena),
                    value: value1,
                })?;
                seq.end()
            }
        }
    }
}
#[derive(Clone)]
struct UnionVisitor {
    arena: Rc<RefCell<Arena<Value>>>,
}
impl<'de> Visitor<'de> for UnionVisitor {
    type Value = Union;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Expected a Union")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let arena = Rc::clone(&self.arena);
        let structure = WithValueDeser { arena };
        seq.next_element::<&str>()?.map_or_else(
            || {
                Err(serde::de::Error::invalid_type(
                    serde::de::Unexpected::Str("unexpected sequence"),
                    &"value",
                ))
            },
            |x| match x {
                "Left" => {
                    let elem = seq.next_element_seed::<WithValueDeser>(structure)?;
                    elem.map_or_else(
                        || {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str("unexpected sequence"),
                                &"value",
                            ))
                        },
                        |x| {
                            let inserted = self.arena.as_ref().borrow_mut().insert(x);
                            Ok(Union::Left(inserted))
                        },
                    )
                }
                "Right" => {
                    let elem = seq.next_element_seed::<WithValueDeser>(structure)?;
                    elem.map_or_else(
                        || {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str("unexpected sequence"),
                                &"value",
                            ))
                        },
                        |x| {
                            let inserted = self.arena.as_ref().borrow_mut().insert(x);
                            Ok(Union::Right(inserted))
                        },
                    )
                }
                _ => Err(serde::de::Error::invalid_type(
                    serde::de::Unexpected::Str("unexpected sequence"),
                    &self,
                )),
            },
        )
    }
}
struct WithUnionDeser {
    arena: Rc<RefCell<Arena<Value>>>,
}
impl<'de> DeserializeSeed<'de> for WithUnionDeser {
    type Value = Union;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_tuple(2, UnionVisitor { arena: self.arena })
    }
}
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone)]
pub enum Value {
    Bytes(Vec<u8>),
    String(String),
    Int(rug::Integer),
    Union(Union),
    Pair { fst: DefaultKey, snd: DefaultKey },
    Bool(bool),
    Map(OrdMap<DefaultKey, DefaultKey>),
    Set(OrdSet<DefaultKey>),
    List(Vector<DefaultKey>),
    Unit,
    Option(Option<DefaultKey>),
}

unsafe impl Sync for Value {}
unsafe impl Send for Value {}

#[repr(transparent)]
#[derive(Clone)]
struct ValueVisitor {
    arena: Rc<RefCell<Arena<Value>>>,
}
impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;
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
                "Unit" => Ok(Value::Unit),
                "Bool" => {
                    let elem = seq.next_element::<bool>()?;
                    elem.map_or_else(
                        || {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str("unexpected sequence"),
                                &"value",
                            ))
                        },
                        |x| Ok(Value::Bool(x)),
                    )
                }
                "Int" => {
                    let elem = seq.next_element::<&str>()?;
                    let elem: Option<rug::Integer> = elem
                        .map(|elem| rug::Integer::from_str_radix(elem, 10))
                        .transpose()
                        .map_or_else(|_| None, |x| x);
                    elem.map_or_else(
                        || {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str("unexpected sequence"),
                                &self,
                            ))
                        },
                        |x| Ok(Value::Int(x)),
                    )
                }
                "Bytes" => {
                    let elem = seq.next_element::<&str>()?;
                    let elem = elem.map(|elem| elem.as_bytes().to_vec());
                    elem.map_or_else(
                        || {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str("unexpected sequence"),
                                &self,
                            ))
                        },
                        |x| Ok(Value::Bytes(x)),
                    )
                }
                "String" => {
                    let elem = seq.next_element::<&str>()?;
                    elem.map_or_else(
                        || {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str("unexpected sequence"),
                                &self,
                            ))
                        },
                        |x| Ok(Value::String(x.to_string())),
                    )
                }
                "Union" => {
                    let union_deser = WithUnionDeser { arena: self.arena };
                    let elem = seq.next_element_seed::<WithUnionDeser>(union_deser)?;
                    elem.map_or_else(
                        || {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str("unexpected sequence"),
                                &"test",
                            ))
                        },
                        |x| Ok(Value::Union(x)),
                    )
                }
                "List" => {
                    let structure = WithValueDeserSeq {
                        visitor: ValueVisitorSeq { arena: self.arena },
                        _p: PhantomData,
                    };
                    let elem = seq.next_element_seed(structure)?;
                    elem.map_or_else(
                        || {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str("unexpected structure in list"),
                                &"Value enum",
                            ))
                        },
                        |x| Ok(Value::List(Vector::from(x))),
                    )
                }
                "Map" => {
                    let elem = seq.next_element_seed(WithValueDeserSeq {
                        visitor: ValueVisitorTup {
                            arena: Rc::clone(&self.arena),
                        },
                        _p: PhantomData,
                    })?;
                    elem.map_or_else(
                        || {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str("unexpected structure in list"),
                                &"Value enum",
                            ))
                        },
                        |elem| Ok(Value::Map(OrdMap::from(elem))),
                    )
                }
                "Set" => {
                    let structure = WithValueDeserSeq {
                        visitor: ValueVisitorSeq { arena: self.arena },
                        _p: PhantomData,
                    };
                    let elem = seq.next_element_seed(structure)?;
                    elem.map_or_else(
                        || {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str("unexpected structure in list"),
                                &"Value enum",
                            ))
                        },
                        |x| Ok(Value::Set(OrdSet::from(x))),
                    )
                }
                "Pair" => {
                    let elem1 = seq.next_element::<&str>()?;
                    let elem1 = elem1.map_or_else(
                        || {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str("unexpected sequence"),
                                &self,
                            ))
                        },
                        |x| {
                            let structure = WithValueDeser {
                                arena: Rc::clone(&self.arena),
                            };
                            let mut deser = serde_json::Deserializer::from_str(x);
                            let elem = structure.deserialize(&mut deser);
                            let elem = elem
                                .map_err(|_| {
                                    serde::de::Error::invalid_type(
                                        serde::de::Unexpected::Str("unexpected sequence"),
                                        &"value",
                                    )
                                })
                                .map(|x| self.arena.as_ref().borrow_mut().insert(x))?;
                            Ok(elem)
                        },
                    )?;
                    let elem2 = seq.next_element::<&str>()?;
                    let elem2 = elem2.map_or_else(
                        || {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str("unexpected sequence"),
                                &"value",
                            ))
                        },
                        |x| {
                            let structure = WithValueDeser {
                                arena: Rc::clone(&self.arena),
                            };
                            let mut deser = serde_json::Deserializer::from_str(x);
                            let elem = structure.deserialize(&mut deser);
                            let elem = elem
                                .map_err(|_| {
                                    serde::de::Error::invalid_type(
                                        serde::de::Unexpected::Str("unexpected sequence"),
                                        &"value",
                                    )
                                })
                                .map(|x| self.arena.as_ref().borrow_mut().insert(x))?;
                            Ok(elem)
                        },
                    )?;

                    Ok(Value::Pair {
                        fst: elem1,
                        snd: elem2,
                    })
                }

                "Option" => {
                    let elem = seq.next_element::<&str>()?;
                    elem.map_or_else(
                        || {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str("unexpected sequence"),
                                &"value",
                            ))
                        },
                        |x| {
                            let structure = WithValueDeser {
                                arena: Rc::clone(&self.arena),
                            };
                            let mut deser = serde_json::Deserializer::from_str(x);
                            let elem = structure.deserialize(&mut deser);
                            let elem = elem
                                .map_err(|_| {
                                    serde::de::Error::invalid_type(
                                        serde::de::Unexpected::Str("unexpected sequence"),
                                        &"value",
                                    )
                                })
                                .map(|x| self.arena.as_ref().borrow_mut().insert(x))?;
                            Ok(Value::Option(Some(elem)))
                        },
                    )
                }
                _ => Err(serde::de::Error::invalid_type(
                    serde::de::Unexpected::Str("unexpected sequence"),
                    &"value",
                )),
            },
        )
    }
}
#[derive(Clone)]
struct WithValueDeser {
    arena: Rc<RefCell<Arena<Value>>>,
}
impl<'de> DeserializeSeed<'de> for WithValueDeser {
    type Value = Value;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_tuple(2, ValueVisitor { arena: self.arena })
    }
}

#[derive(Clone)]
struct ValueVisitorSeq {
    arena: Rc<RefCell<Arena<Value>>>,
}
impl<'de> Visitor<'de> for ValueVisitorSeq {
    type Value = Vec<DefaultKey>;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Expected a valid Value")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut val = Vec::with_capacity(30);
        loop {
            let next = seq.next_element_seed(WithValueDeser {
                arena: Rc::clone(&self.arena),
            });
            if let Ok(x) = next {
                if x.is_none() {
                    break;
                }
                x.map_or_else(
                    || {
                        Err(serde::de::Error::invalid_type(
                            serde::de::Unexpected::Str("unexpected sequence"),
                            &"value",
                        ))
                    },
                    |x| {
                        let key = self.arena.as_ref().borrow_mut().insert(x);
                        val.push(key);
                        Ok(())
                    },
                )?;
            } else {
                return Err(serde::de::Error::invalid_type(
                    serde::de::Unexpected::Str("unexpected sequence"),
                    &"value",
                ));
            }
        }
        Ok(val)
    }
}
#[derive(Clone)]
struct WithValueDeserSeq<'a, T: Visitor<'a> + 'a> {
    visitor: T,
    _p: PhantomData<&'a T>,
}
impl<'de, T: Visitor<'de>> DeserializeSeed<'de> for WithValueDeserSeq<'de, T> {
    type Value = T::Value;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(self.visitor)
    }
}
#[derive(Clone)]
struct WithValueDeserTup {
    arena: Rc<RefCell<Arena<Value>>>,
}
impl<'de> DeserializeSeed<'de> for WithValueDeserTup {
    type Value = Vec<(DefaultKey, DefaultKey)>;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ValueVisitorTup { arena: self.arena })
    }
}
#[derive(Clone)]
struct ValueVisitorTup {
    arena: Rc<RefCell<Arena<Value>>>,
}
impl<'de> Visitor<'de> for ValueVisitorTup {
    type Value = Vec<(DefaultKey, DefaultKey)>;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Expected a valid Value")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut val = Vec::with_capacity(1);
        loop {
            let next = seq.next_element_seed(WithValueDeserSeq {
                visitor: ValueVisitorSeq {
                    arena: Rc::clone(&self.arena),
                },
                _p: PhantomData,
            });
            if let Ok(x) = next {
                if x.is_none() {
                    break;
                }
                x.map_or_else(
                    || {
                        Err(serde::de::Error::invalid_type(
                            serde::de::Unexpected::Str("unexpected sequence"),
                            &"value",
                        ))
                    },
                    |x| {
                        val.push((x[0], x[1]));
                        Ok(())
                    },
                )?;
            } else {
                return Err(serde::de::Error::invalid_type(
                    serde::de::Unexpected::Str("unexpected sequence"),
                    &"value",
                ));
            }
        }

        Ok(val)
    }
}
struct ValueSerializer {
    arena: Rc<RefCell<Arena<Value>>>,
    value: Value,
}
impl Serialize for ValueSerializer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use Value::*;
        match &self.value {
            Int(x) => {
                let mut seq = serializer.serialize_tuple(2)?;
                seq.serialize_element("Int")?;
                seq.serialize_element(&x.to_string_radix(10))?;
                seq.end()
            }
            Bool(x) => {
                let mut seq = serializer.serialize_tuple(2)?;
                seq.serialize_element("Bool")?;
                seq.serialize_element(&x)?;
                seq.end()
            }
            Bytes(b) => {
                let mut seq = serializer.serialize_tuple(2)?;
                seq.serialize_element("Bytes")?;
                seq.serialize_element(
                    &std::string::String::from_utf8(b.to_vec())
                        .map_err(serde::ser::Error::custom)?,
                )?;
                seq.end()
            }
            String(b) => {
                let mut seq = serializer.serialize_tuple(2)?;
                seq.serialize_element("String")?;
                seq.serialize_element(&b)?;
                seq.end()
            }
            Union(union) => {
                let mut seq = serializer.serialize_tuple(2)?;
                seq.serialize_element("Union")?;
                let serializer = UnionSerializer {
                    arena: Rc::clone(&self.arena),
                    value: union,
                };
                seq.serialize_element(&serializer)?;
                seq.end()
            }
            Map(map) => {
                let mut seq = serializer.serialize_tuple(2)?;
                seq.serialize_element("Map")?;
                let serialized = map
                    .iter()
                    .map(|(x, y)| {
                        let value1 = self.arena.as_ref().borrow_mut().remove(*x)?;
                        let value2 = self.arena.as_ref().borrow_mut().remove(*y)?;
                        let tup = (
                            ValueSerializer {
                                arena: Rc::clone(&self.arena),
                                value: value1,
                            },
                            ValueSerializer {
                                arena: Rc::clone(&self.arena),
                                value: value2,
                            },
                        );
                        Some(tup)
                    })
                    .collect::<std::option::Option<Vec<(ValueSerializer, ValueSerializer)>>>();
                let serialized = serialized
                    .map_or_else(|| Err(serde::ser::Error::custom(&"error serializing")), Ok)?;
                seq.serialize_element(&serialized)?;
                seq.end()
            }
            Set(set) => {
                let mut seq = serializer.serialize_tuple(2)?;
                seq.serialize_element("Set")?;

                let serialized: std::option::Option<Vec<ValueSerializer>> = set
                    .iter()
                    .map(|x| {
                        let value = self.arena.as_ref().borrow_mut().remove(*x)?;
                        let val = ValueSerializer {
                            arena: Rc::clone(&self.arena),
                            value,
                        };
                        Some(val)
                    })
                    .collect();
                let serialized = serialized
                    .map_or_else(|| Err(serde::ser::Error::custom(&"error serializing")), Ok)?;
                seq.serialize_element(&serialized)?;
                seq.end()
            }
            Unit => {
                let mut seq = serializer.serialize_tuple(1)?;
                seq.serialize_element("Unit")?;
                seq.end()
            }
            Option(opt) => {
                let mut seq = serializer.serialize_tuple(2)?;
                seq.serialize_element("Option")?;
                match opt.map(|x| {
                    self.arena
                        .as_ref()
                        .borrow_mut()
                        .remove(x)
                        .map(|x| ValueSerializer {
                            arena: Rc::clone(&self.arena),
                            value: x,
                        })
                }) {
                    Some(x) => seq.serialize_element(&x)?,
                    None => seq.serialize_element(&None::<&ValueSerializer>)?,
                };
                seq.end()
            }
            Pair { fst, snd } => {
                let mut seq = serializer.serialize_tuple(3)?;
                seq.serialize_element("Pair")?;
                let value1 = self
                    .arena
                    .as_ref()
                    .borrow_mut()
                    .remove(*fst)
                    .map_or_else(|| Err(serde::ser::Error::custom(&"error serializing")), Ok)?;
                seq.serialize_element(&ValueSerializer {
                    arena: Rc::clone(&self.arena),
                    value: value1,
                })?;
                let value1 = self
                    .arena
                    .as_ref()
                    .borrow_mut()
                    .remove(*snd)
                    .map_or_else(|| Err(serde::ser::Error::custom(&"error serializing")), Ok)?;
                seq.serialize_element(&ValueSerializer {
                    arena: Rc::clone(&self.arena),
                    value: value1,
                })?;
                seq.end()
            }
            List(lst) => {
                let mut seq = serializer.serialize_tuple(3)?;
                seq.serialize_element("List")?;

                let serialized = lst
                    .iter()
                    .map(|x| {
                        let value = self.arena.as_ref().borrow_mut().remove(*x)?;
                        Some(ValueSerializer {
                            arena: Rc::clone(&self.arena),
                            value,
                        })
                    })
                    .collect::<std::option::Option<Vec<ValueSerializer>>>();
                let serialized = serialized
                    .map_or_else(|| Err(serde::ser::Error::custom(&"error serializing")), Ok)?;
                seq.serialize_element(&serialized)?;
                seq.end()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use im_rc::ordmap;

    use super::*;
    #[test]
    fn serialization_deserialization_yields_same_structures() {
        let mut ar = HopSlotMap::new();
        let refrer = ar.insert(Value::Int(1.into()));

        let refre2 = ar.insert(Value::Int(1.into()));
        let refre3 = ar.insert(Value::Int(1.into()));
        let refre4 = ar.insert(Value::Int(1.into()));
        let expected = Value::Union(Union::Left(
            ar.insert(Value::Map(ordmap! {refrer => refre3, refre2 => refre4})),
        ));
        let arena = Rc::new(RefCell::new(ar));

        let ser = &serde_json::to_string(&ValueSerializer {
            arena: Rc::clone(&arena),
            value: expected.clone(),
        })
        .unwrap();
        let arena = Rc::new(RefCell::new(HopSlotMap::new()));
        let structure = WithValueDeser {
            arena: Rc::clone(&arena),
        };
        let mut x = serde_json::Deserializer::from_str(ser);
        let res = structure.deserialize(&mut x).unwrap();
        // same keys
        assert_eq!(res, expected);
        let ser2 = &serde_json::to_string(&ValueSerializer {
            arena: Rc::clone(&arena),
            value: res,
        })
        .unwrap();
        // same serialized
        assert_eq!(ser, ser2);
    }
}
