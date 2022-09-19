use crate::arena::ARENA;
use im_rc::{OrdMap, OrdSet, Vector};
use serde::{de::Visitor, ser::SerializeTuple, Deserialize, Serialize};
use slotmap::DefaultKey;
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone)]
pub enum Union {
    Left(DefaultKey),
    Right(DefaultKey),
}
unsafe impl Sync for Union {}
unsafe impl Send for Union {}
impl Serialize for Union {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use Union::*;
        let for_removal = unsafe { &ARENA };

        match self {
            Right(x) => {
                let mut seq = serializer.serialize_tuple(2)?;

                seq.serialize_element("Right")?;
                let value1 = for_removal
                    .get(*x)
                    .map_or_else(|| Err(serde::ser::Error::custom(&"error serializing")), Ok)?;
                seq.serialize_element(value1)?;
                seq.end()
            }
            Left(x) => {
                let mut seq = serializer.serialize_tuple(2)?;
                seq.serialize_element("Left")?;
                let value1 = for_removal
                    .get(*x)
                    .map_or_else(|| Err(serde::ser::Error::custom(&"error serializing")), Ok)?;

                seq.serialize_element(value1)?;
                seq.end()
            }
        }
    }
}
struct UnionVisitor;
impl<'de> Visitor<'de> for UnionVisitor {
    type Value = Union;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Expected a Union")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let arena = unsafe { &mut ARENA };

        seq.next_element::<&str>()?.map_or_else(
            || {
                Err(serde::de::Error::invalid_type(
                    serde::de::Unexpected::Str("unexpected sequence"),
                    &"value",
                ))
            },
            |x| match x {
                "Left" => {
                    let elem = seq.next_element::<Value>()?;
                    elem.map_or_else(
                        || {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str("unexpected sequence"),
                                &"value",
                            ))
                        },
                        |x| {
                            let inserted = arena.insert(x);
                            Ok(Union::Left(inserted))
                        },
                    )
                }
                "Right" => {
                    let elem = seq.next_element::<Value>()?;
                    elem.map_or_else(
                        || {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str("unexpected sequence"),
                                &"value",
                            ))
                        },
                        |x| {
                            let inserted = arena.insert(x);
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
impl<'de> Deserialize<'de> for Union {
    fn deserialize<D>(deserializer: D) -> Result<Union, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_tuple(2, UnionVisitor)
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
pub struct ValueVisitor;
impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Expected a valid Value")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let arena = unsafe { &mut ARENA };
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
                    let elem = seq.next_element::<Union>()?;
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
                    let elem = seq.next_element::<Vector<Value>>()?;
                    elem.map_or_else(
                        || {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str("unexpected structure in list"),
                                &"Value enum",
                            ))
                        },
                        |x| {
                            Ok(Value::List(
                                x.into_iter().map(|x| arena.insert(x)).collect(),
                            ))
                        },
                    )
                }
                "Map" => {
                    let elem = seq.next_element::<Vec<(Value, Value)>>()?;
                    elem.map_or_else(
                        || {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str("unexpected structure in list"),
                                &"Value enum",
                            ))
                        },
                        |elem| {
                            Ok(Value::Map(OrdMap::from(
                                elem.into_iter()
                                    .map(|(k, v)| (arena.insert(k), arena.insert(v)))
                                    .collect::<Vec<(DefaultKey, DefaultKey)>>(),
                            )))
                        },
                    )
                }
                "Set" => {
                    let elem = seq.next_element::<Vec<Value>>()?;
                    elem.map_or_else(
                        || {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str("unexpected structure in list"),
                                &"Value enum",
                            ))
                        },
                        |x| {
                            Ok(Value::Set(OrdSet::from(
                                x.into_iter()
                                    .map(|x| arena.insert(x))
                                    .collect::<Vec<DefaultKey>>(),
                            )))
                        },
                    )
                }
                "Pair" => {
                    let elem1 = seq.next_element::<[Value; 2]>()?;
                    elem1.map_or_else(
                        || {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str("unexpected sequence"),
                                &self,
                            ))
                        },
                        |[value1, value2]| {
                            let res = Value::Pair {
                                fst: arena.insert(value1),
                                snd: arena.insert(value2),
                            };
                            Ok(res)
                        },
                    )
                }

                "Option" => {
                    let elem = seq.next_element::<Option<Value>>()?;
                    elem.map_or_else(
                        || {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str("unexpected sequence"),
                                &"value",
                            ))
                        },
                        |x| {
                            let x = x.map(|x| arena.insert(x));
                            Ok(Value::Option(x))
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
impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_tuple(2, ValueVisitor)
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use Value::*;
        let arena = unsafe { &mut ARENA };
        match self {
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
                seq.serialize_element(&union)?;
                seq.end()
            }
            Map(map) => {
                let mut seq = serializer.serialize_tuple(2)?;
                seq.serialize_element("Map")?;
                let serialized = map
                    .into_iter()
                    .map(|(x, y)| {
                        let tup = (arena.get(*x)?, arena.get(*y)?);
                        Some(tup)
                    })
                    .collect::<std::option::Option<Vec<(&Value, &Value)>>>();
                let serialized = serialized
                    .map_or_else(|| Err(serde::ser::Error::custom(&"error serializing")), Ok)?;
                seq.serialize_element(&serialized)?;
                seq.end()
            }
            Set(set) => {
                let mut seq = serializer.serialize_tuple(2)?;
                seq.serialize_element("Set")?;

                let serialized: std::option::Option<Vec<&Value>> =
                    set.into_iter().map(|x| arena.get(*x)).collect();
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
                seq.serialize_element(&opt.map(|x| arena.get(x)))?;
                seq.end()
            }
            Pair { fst, snd } => {
                let mut seq = serializer.serialize_tuple(3)?;
                seq.serialize_element("Pair")?;
                let value1 = arena
                    .get(*fst)
                    .map_or_else(|| Err(serde::ser::Error::custom(&"error serializing")), Ok)?;
                let value2 = arena
                    .get(*snd)
                    .map_or_else(|| Err(serde::ser::Error::custom(&"error serializing")), Ok)?;
                seq.serialize_element(value1)?;

                seq.serialize_element(value2)?;
                seq.end()
            }
            List(lst) => {
                let mut seq = serializer.serialize_tuple(3)?;
                seq.serialize_element("List")?;
                let serialized = lst
                    .into_iter()
                    .map(|x| arena.get(*x))
                    .collect::<std::option::Option<Vec<&Value>>>();
                serialized.map_or_else(
                    || Err(serde::ser::Error::custom(&"error serializing")),
                    |x| seq.serialize_element(&x),
                )?;
                seq.end()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use im_rc::ordmap;
    use once_cell::unsync::Lazy;
    use slotmap::HopSlotMap;

    use super::*;
    #[test]
    fn serialization_deserialization_yields_same_structures() {
        let arena = unsafe { &mut ARENA };
        let refrer = arena.insert(Value::Int(1.into()));

        let refre2 = arena.insert(Value::Int(1.into()));
        let refre3 = arena.insert(Value::Int(1.into()));
        let refre4 = arena.insert(Value::Int(1.into()));
        let expected = Value::Union(Union::Left(arena.insert(Value::Pair {
            fst: refre2,
            snd: refre4,
        })));

        let ser = &serde_json::to_string(&expected).unwrap();
        unsafe { ARENA = Lazy::new(HopSlotMap::new) };
        let x: Value = serde_json::from_str(ser).unwrap();
        // same keys
        assert_eq!(x, expected);
        let ser2 = &serde_json::to_string(&x).unwrap();
        // same serialized
        assert_eq!(ser, ser2);
    }
}
