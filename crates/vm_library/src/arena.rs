use slotmap::{DefaultKey, HopSlotMap};

use crate::managed::value::Value;
use once_cell::unsync::Lazy;

pub static mut ARENA: Lazy<HopSlotMap<DefaultKey, Value>> =
    Lazy::new(|| HopSlotMap::with_capacity(4000));
