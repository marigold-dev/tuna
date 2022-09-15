use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use mimalloc::MiMalloc;
use serde::{de::DeserializeSeed, ser::SerializeStruct, Deserialize, Serialize};
use slotmap::{DefaultKey, HopSlotMap};
use vm_library::{
    compile_store,
    managed::value::{Value, ValueSerializer, WithValueDeser},
};
use wasmer::{imports, wat2wasm, Instance, Module};
use wasmer_middlewares::metering::set_remaining_points;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn deser(arena: &Rc<RefCell<HopSlotMap<DefaultKey, Value>>>, json: &str) -> T {
    let top = Rc::clone(arena);
    let deser = VV { arena: top };
    let mut str = serde_json::Deserializer::from_str(json);
    let x = deser.deserialize(&mut str).unwrap();
    x
}
fn ser(res: T, arena: &Rc<RefCell<HopSlotMap<DefaultKey, Value>>>) -> String {
    let serializer = VVSer {
        arena: Rc::clone(arena),
        value: res,
    };

    serde_json::ser::to_string(&serializer).unwrap()
}
fn benchmark(
    num: i64,
    arena: &Rc<RefCell<HopSlotMap<DefaultKey, Value>>>,
    json: &str,
) -> (Module, String, i64) {
    let x = deser(arena, json);

    let module = Module::new(
        &compile_store::new_compile_store(),
        wat2wasm(x.mod_.as_bytes()).unwrap(),
    )
    .unwrap();
    let instance = Instance::new(&module, &imports! {}).unwrap();
    set_remaining_points(&instance, 1000);
    let caller = instance
        .exports
        .get_native_function::<i64, i64>("main")
        .unwrap();
    let result = caller.call(num).expect("error");
    let serialized = ser(x, arena);
    arena.as_ref().borrow_mut().clear();
    (module, serialized, result)
}
//RUSTFLAGS='-C target-cpu=native' cargo bench
struct T {
    mod_: String,
    arg: DefaultKey,
    initial_storage: DefaultKey,
}
use std::{cell::RefCell, fmt, rc::Rc};

use serde::de::{self, Deserializer, MapAccess, SeqAccess, Visitor};
struct VV {
    arena: Rc<RefCell<HopSlotMap<DefaultKey, Value>>>,
}
impl<'de> DeserializeSeed<'de> for VV {
    type Value = T;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            MOD_,
            ARG,
            INITIALSTORAGE,
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
                            "arg" => Ok(Field::ARG),
                            "initial_storage" => Ok(Field::INITIALSTORAGE),
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
            type Value = T;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Duration")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<T, V::Error>
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
                Ok(T {
                    mod_,
                    arg,
                    initial_storage,
                })
            }

            fn visit_map<V>(self, mut map: V) -> Result<T, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut mod_ = None;
                let mut arg = None;
                let mut initial_storage = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::MOD_ => {
                            if mod_.is_some() {
                                return Err(de::Error::duplicate_field("secs"));
                            }
                            mod_ = Some(map.next_value()?);
                        }
                        Field::ARG => {
                            if arg.is_some() {
                                return Err(de::Error::duplicate_field("nanos"));
                            }
                            arg = Some(map.next_value_seed(WithValueDeser {
                                arena: Rc::clone(&self.arena),
                            })?);
                        }
                        Field::INITIALSTORAGE => {
                            if initial_storage.is_some() {
                                return Err(de::Error::duplicate_field("nanos"));
                            }
                            initial_storage = Some(map.next_value_seed(WithValueDeser {
                                arena: Rc::clone(&self.arena),
                            })?);
                        }
                    }
                }
                let mod_ = mod_.ok_or_else(|| de::Error::missing_field("secs"))?;
                let arg = arg.ok_or_else(|| de::Error::missing_field("nanos"))?;
                let initial_storage =
                    initial_storage.ok_or_else(|| de::Error::missing_field("nanos"))?;
                let initial_storage = self.arena.as_ref().borrow_mut().insert(initial_storage);
                let arg = self.arena.as_ref().borrow_mut().insert(arg);

                Ok(T {
                    mod_,
                    arg,
                    initial_storage,
                })
            }
        }

        const FIELDS: &'static [&'static str] = &["mod_", "arg", "initial_storage"];
        deserializer.deserialize_struct("T", FIELDS, TV { arena: self.arena })
    }
}
struct VVSer {
    arena: Rc<RefCell<HopSlotMap<DefaultKey, Value>>>,
    value: T,
}

impl Serialize for VVSer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut structture = serializer.serialize_struct("T", 3)?;
        structture.serialize_field("mod_", &self.value.mod_)?;
        let (arg, init) = {
            let mut arena = self.arena.as_ref().borrow_mut();
            let arg = arena.remove(self.value.arg).unwrap();
            let initial_arg = arena.remove(self.value.initial_storage).unwrap();
            (arg, initial_arg)
        };
        structture.serialize_field(
            "arg",
            &ValueSerializer {
                arena: Rc::clone(&self.arena),
                value: arg,
            },
        )?;
        let init = ValueSerializer {
            arena: Rc::clone(&self.arena),
            value: init,
        };
        structture.serialize_field("initial_storage", &init)?;
        structture.end()
    }
}
fn criterion_benchmark(c: &mut Criterion) {
    let json = r#"{"mod_":" (module\n(func $main (param $n i64) (result i64)\n  local.get $n\n)\n(export \"main\" (func $main))\n)","arg":["Union",["Left",["List",[["Int","0"],["Int","1"],["Int","2"],["Int","3"],["Int","4"],["Int","5"],["Int","6"],["Int","7"],["Int","8"],["Int","9"],["Int","10"],["Int","11"],["Int","12"],["Int","13"],["Int","14"],["Int","15"],["Int","16"],["Int","17"],["Int","18"],["Int","19"],["Int","20"],["Int","21"],["Int","22"],["Int","23"],["Int","24"],["Int","25"],["Int","26"],["Int","27"],["Int","28"],["Int","29"],["Int","30"],["Int","31"],["Int","32"],["Int","33"],["Int","34"],["Int","35"],["Int","36"],["Int","37"],["Int","38"],["Int","39"],["Int","40"],["Int","41"],["Int","42"],["Int","43"],["Int","44"],["Int","45"],["Int","46"],["Int","47"],["Int","48"],["Int","49"],["Int","50"],["Int","51"],["Int","52"],["Int","53"],["Int","54"],["Int","55"],["Int","56"],["Int","57"],["Int","58"],["Int","59"],["Int","60"],["Int","61"],["Int","62"],["Int","63"],["Int","64"],["Int","65"],["Int","66"],["Int","67"],["Int","68"],["Int","69"],["Int","70"],["Int","71"],["Int","72"],["Int","73"],["Int","74"],["Int","75"],["Int","76"],["Int","77"],["Int","78"],["Int","79"],["Int","80"],["Int","81"],["Int","82"],["Int","83"],["Int","84"],["Int","85"],["Int","86"],["Int","87"],["Int","88"],["Int","89"],["Int","90"],["Int","91"],["Int","92"],["Int","93"],["Int","94"],["Int","95"],["Int","96"],["Int","97"],["Int","98"],["Int","99"]]]]],"initial_storage":["List",[["Int","0"],["Int","1"],["Int","2"],["Int","3"],["Int","4"],["Int","5"],["Int","6"],["Int","7"],["Int","8"],["Int","9"],["Int","10"],["Int","11"],["Int","12"],["Int","13"],["Int","14"],["Int","15"],["Int","16"],["Int","17"],["Int","18"],["Int","19"],["Int","20"],["Int","21"],["Int","22"],["Int","23"],["Int","24"],["Int","25"],["Int","26"],["Int","27"],["Int","28"],["Int","29"],["Int","30"],["Int","31"],["Int","32"],["Int","33"],["Int","34"],["Int","35"],["Int","36"],["Int","37"],["Int","38"],["Int","39"],["Int","40"],["Int","41"],["Int","42"],["Int","43"],["Int","44"],["Int","45"],["Int","46"],["Int","47"],["Int","48"],["Int","49"],["Int","50"],["Int","51"],["Int","52"],["Int","53"],["Int","54"],["Int","55"],["Int","56"],["Int","57"],["Int","58"],["Int","59"],["Int","60"],["Int","61"],["Int","62"],["Int","63"],["Int","64"],["Int","65"],["Int","66"],["Int","67"],["Int","68"],["Int","69"],["Int","70"],["Int","71"],["Int","72"],["Int","73"],["Int","74"],["Int","75"],["Int","76"],["Int","77"],["Int","78"],["Int","79"],["Int","80"],["Int","81"],["Int","82"],["Int","83"],["Int","84"],["Int","85"],["Int","86"],["Int","87"],["Int","88"],["Int","89"],["Int","90"],["Int","91"],["Int","92"],["Int","93"],["Int","94"],["Int","95"],["Int","96"],["Int","97"],["Int","98"],["Int","99"]]]}"#;
    let arena: HopSlotMap<DefaultKey, Value> = HopSlotMap::with_capacity(8000);
    let arena = Rc::new(RefCell::new(arena));

    let input = (&arena, json, 1000);

    c.bench_with_input(
        BenchmarkId::new("test", "simple module"),
        &input,
        |b, (arena, json, num)| b.iter(|| benchmark(*num, arena, json)),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
