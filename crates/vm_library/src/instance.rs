use std::{cell::RefCell, ptr::NonNull, rc::Rc};

use slotmap::{DefaultKey, HopSlotMap, KeyData};
use wasmer::{Instance, Module, NativeFunc};

use crate::{
    conversions,
    env::Context,
    errors::VMResult,
    managed::{imports::make_imports, value::Value},
};

// TODO: remove unwraps
pub fn call_module(
    arena: Rc<RefCell<HopSlotMap<DefaultKey, Value>>>,
    m: Module,
    gas_limit: u64,
    params: DefaultKey,
    initial_storage: DefaultKey,
) -> VMResult<DefaultKey> {
    let mut env = Box::new(Context {
        arena,
        instance: None,
        pusher: None,
        gas_limit,
    });
    let store = m.store();

    let imports = make_imports(&env, store);
    let mut instance = Box::new(Instance::new(&m, &imports).unwrap());
    env.instance = NonNull::new(instance.as_mut());
    let pusher = instance.exports.get_native_function("push").unwrap();
    env.pusher = NonNull::new({
        let mut fun = Box::new(pusher);
        fun.as_mut()
    });
    let main: NativeFunc<i64, i64> = instance.exports.get_native_function("main").unwrap();
    let arg = env.bump(Value::Pair {
        fst: params,
        snd: initial_storage,
    });
    let key = conversions::to_i64(arg)?;
    let called = main.call(key).unwrap();
    Ok(DefaultKey::from(KeyData::from_ffi(called as u64)))
}

// #[cfg(test)]
// mod test {
//     use wasmer::{imports, wat2wasm};
//     use wasmer_middlewares::metering::set_remaining_points;

//     use crate::compile_store;

//     use super::*;
//     #[test]
//     fn testing() {
//         let expected = 6;
//         let module = wat2wasm(
//             br#"
//             (module
//               (func $main (param $n i64) (result i64)
//                 local.get $n
//               )
//             (export "main" (func $main))
//           )
//         "#,
//         )
//         .unwrap();
//         #[derive(PartialEq, Debug)]
//         pub struct T {
//             pub v: String,
//             pub z: Option<Box<'static, T>>,
//         }
//         let ar: Bump = Bump::with_capacity(8000);
//         ar.set_allocation_limit(Some(8000));
//         let vv = Box::new_in(
//             T {
//                 v: "String".to_string(),
//                 z: None,
//             },
//             &ar,
//         );
//         let to_pass = Box::into_raw(vv) as *mut usize;
//         let store = compile_store::new_compile_store();
//         let module = Module::new(&store, module).unwrap();
//         let imports = imports! {};
//         let result = Instance::new(&module, &imports).unwrap();
//         set_remaining_points(&result, 1000);
//         let main: NativeFunc<i64, i64> = result.exports.get_native_function("main").unwrap();
//         let res = main.call(to_pass as i64).unwrap();
//         assert_eq!(res, to_pass as i64)
//     }
// }
