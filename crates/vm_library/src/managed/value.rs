use im_rc::{OrdMap, OrdSet, Vector};
use serde::{de::Visitor, ser::SerializeTuple, Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum Union<A> {
    Left(Box<A>),
    Right(Box<A>),
}
unsafe impl<A: Sync> Sync for Union<A> {}
unsafe impl<A: Send> Send for Union<A> {}

impl<A: Serialize> Serialize for Union<A> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use Union::*;
        match self {
            Right(x) => {
                let mut seq = serializer.serialize_tuple(2)?;
                seq.serialize_element("Right")?;
                seq.serialize_element(x)?;
                seq.end()
            }
            Left(x) => {
                let mut seq = serializer.serialize_tuple(2)?;
                seq.serialize_element("Left")?;
                seq.serialize_element(x)?;
                seq.end()
            }
        }
    }
}
struct UnionVisitor;
impl<'de> Visitor<'de> for UnionVisitor {
    type Value = Union<Value>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Expected a Union")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        seq.next_element::<&str>()?.map_or_else(
            || {
                Err(serde::de::Error::invalid_type(
                    serde::de::Unexpected::Str("unexpected sequence"),
                    &self,
                ))
            },
            |x| match x {
                "Left" => {
                    let elem = seq.next_element::<Value>()?;
                    elem.map_or_else(
                        || {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str("unexpected sequence"),
                                &self,
                            ))
                        },
                        |x| Ok(Union::Left(Box::new(x))),
                    )
                }
                "Right" => {
                    let elem = seq.next_element::<Value>()?;
                    elem.map_or_else(
                        || {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str("unexpected sequence"),
                                &self,
                            ))
                        },
                        |x| Ok(Union::Right(Box::new(x))),
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

impl<'de> Deserialize<'de> for Union<Value> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_tuple(2, UnionVisitor)
    }
}
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum Value {
    Bytes(Vec<u8>),
    String(String),
    Int(rug::Integer),
    Union(Union<Self>),
    Pair(Box<(Self, Self)>),
    Bool(bool),
    Map(OrdMap<Self, Self>),
    Set(OrdSet<Self>),
    List(Vector<Self>),
    Unit,
    Option(Box<Option<Self>>),
}
unsafe impl Sync for Value {}
unsafe impl Send for Value {}
struct ValueVisitor;
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
                    &self,
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
                                &self,
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
                    let elem = seq.next_element::<Union<Value>>()?;
                    elem.map_or_else(
                        || {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str("unexpected sequence"),
                                &self,
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
                                serde::de::Unexpected::Str("unexpected sequence"),
                                &self,
                            ))
                        },
                        |x| Ok(Value::List(x)),
                    )
                }
                "Map" => {
                    let elem = seq.next_element::<Vec<(Value, Value)>>()?;
                    let elem = elem.map(OrdMap::from);
                    elem.map_or_else(
                        || {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str("unexpected sequence"),
                                &self,
                            ))
                        },
                        |x| Ok(Value::Map(x)),
                    )
                }
                "Set" => {
                    let elem = seq.next_element::<Vec<Value>>()?;
                    let elem = elem.map(OrdSet::from);
                    elem.map_or_else(
                        || {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str("unexpected sequence"),
                                &self,
                            ))
                        },
                        |x| Ok(Value::Set(x)),
                    )
                }
                "Pair" => {
                    let elem1 = seq.next_element::<Value>()?;
                    let elem1 = elem1.map_or_else(
                        || {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str("unexpected sequence"),
                                &self,
                            ))
                        },
                        Ok,
                    )?;
                    let elem2 = seq.next_element::<Value>()?;
                    let elem2 = elem2.map_or_else(
                        || {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str("unexpected sequence"),
                                &self,
                            ))
                        },
                        Ok,
                    )?;
                    Ok(Value::Pair(Box::new((elem1, elem2))))
                }

                "Option" => {
                    let elem = seq.next_element::<Option<Value>>()?;
                    elem.map_or_else(
                        || {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str("unexpected sequence"),
                                &self,
                            ))
                        },
                        |x| Ok(Value::Option(Box::new(x))),
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
impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
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
                seq.serialize_element(x)?;
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
                seq.serialize_element(b)?;
                seq.end()
            }
            Union(union) => {
                let mut seq = serializer.serialize_tuple(2)?;
                seq.serialize_element("Union")?;
                seq.serialize_element(union)?;
                seq.end()
            }
            Map(map) => {
                let mut seq = serializer.serialize_tuple(2)?;
                seq.serialize_element("Map")?;
                let serialized: Vec<(&Value, &Value)> = map.iter().collect();
                seq.serialize_element(&serialized)?;
                seq.end()
            }
            Set(x) => {
                let mut seq = serializer.serialize_tuple(2)?;
                seq.serialize_element("Set")?;
                let serialized: Vec<&Value> = x.iter().collect();
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
                seq.serialize_element(&opt)?;
                seq.end()
            }
            Pair(pair) => {
                let mut seq = serializer.serialize_tuple(3)?;
                seq.serialize_element("Pair")?;
                seq.serialize_element(&pair.0)?;
                seq.serialize_element(&pair.1)?;
                seq.end()
            }
            List(lst) => {
                let mut seq = serializer.serialize_tuple(3)?;
                seq.serialize_element("List")?;
                seq.serialize_element(lst)?;
                seq.end()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use im_rc::vector;

    use super::*;
    #[test]
    fn list() {
        let x = Value::Union(Union::Left(Box::new(Value::List(vector![Value::Unit]))));
        let ser = &serde_json::to_string(&x).unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(ser).unwrap(),
            Value::Union(Union::Left(Box::new(Value::List(vector![Value::Unit]))))
        )
    }
}
