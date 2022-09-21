use im_rc::{ordmap, ordset, vector};
use slotmap::{DefaultKey, HopSlotMap};

use crate::managed::value::Value;
use once_cell::unsync::Lazy;

pub static mut ARENA: Lazy<HopSlotMap<DefaultKey, Value>> =
    Lazy::new(|| HopSlotMap::with_capacity(4000));

use std::collections::BTreeMap;
pub static mut PREDEF: Lazy<BTreeMap<String, Value>> = Lazy::new(BTreeMap::new);
pub static mut CONSTANTS: Lazy<Vec<Value>> = Lazy::new(|| Vec::with_capacity(3000));

pub fn populate_predef(sender: String, self_: String, source: String) {
    let map = unsafe { &mut PREDEF };
    map.clear();
    map.insert("none".to_string(), Value::Option(None));
    map.insert("nil".to_string(), Value::List(vector![]));
    map.insert("source".to_string(), Value::String(source));
    map.insert("sender".to_string(), Value::String(sender));
    map.insert("self".to_string(), Value::String(self_));
    map.insert("empty_set".to_string(), Value::Set(ordset![]));
    map.insert("empty_map".to_string(), Value::Map(ordmap! {}));
    map.insert("zero".to_string(), Value::Int(0.into()));
}
pub fn push_constants(vec: Vec<(i32, Value)>) {
    let map = unsafe { &mut CONSTANTS };
    map.clear();
    vec.into_iter().for_each(|(k, v)| {
        map.insert(k as usize, v);
    })
}
