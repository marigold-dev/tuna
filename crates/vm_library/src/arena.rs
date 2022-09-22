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
    map.insert("none".to_owned(), Value::Option(None));
    map.insert("unit".to_owned(), Value::Unit);

    map.insert("nil".to_owned(), Value::List(vector![]));
    map.insert("source".to_owned(), Value::String(source));
    map.insert("sender".to_owned(), Value::String(sender));
    map.insert("self".to_owned(), Value::String(self_));
    map.insert("empty_set".to_owned(), Value::Set(ordset![]));
    map.insert("empty_map".to_owned(), Value::Map(ordmap! {}));
    map.insert("zero".to_owned(), Value::Int(0.into()));
}
pub fn push_constants(vec: Vec<(i32, Value)>) {
    let map = unsafe { &mut CONSTANTS };
    map.clear();
    vec.into_iter().for_each(|(k, v)| {
        map.insert(k as usize, v);
    })
}
