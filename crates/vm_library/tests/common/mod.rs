use serde::{Deserialize, Serialize};
use vm_library::{compile_store, incoming::InvokeManaged, managed::value::Value};
use wasmer::{wat2wasm, Module};

pub fn create_incoming_managed(s: String, arg: Value, initial_storage: Value) -> InvokeManaged {
    let deser: Init = serde_json::from_str(&s).unwrap();
    let mod_ = Module::new(
        &compile_store::new_compile_store(),
        wat2wasm(deser.module_.as_bytes()).unwrap(),
    )
    .unwrap();
    InvokeManaged {
        mod_,
        arg,
        initial_storage,
        constants: deser.constants,
        tickets: vec![],
        source: "tz1gvF4cD2dDtqitL3ZTraggSR1Mju2BKFEM".to_string(),
        sender: "tz1gvF4cD2dDtqitL3ZTraggSR1Mju2BKFEM".to_string(),
        self_addr: "KT1WiBZHtvv3EczaN628DkNob4cayHzTEDNK".to_string(),
        gas_limit: u64::MAX,
    }
}

#[derive(Deserialize, Serialize)]
struct Init {
    module_: String,
    constants: Vec<(i32, Value)>,
}
