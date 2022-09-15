use std::ptr::NonNull;

use wasmer::{Instance, Module, NativeFunc};

// use crate::{
//     env::Context,
//     managed::{imports::make_imports, value::Value},
// };

// TODO: remove unwraps
// pub fn call_module(m: Module, gas_limit: u64, params: Value) -> Option<Value> {
//     let mut env = Box::new(Context {
//         instance: None,
//         pusher: None,
//         gas_limit,
//     });
//     let store = m.store();

//     let imports = make_imports(&env, store);
//     let mut instance = Box::new(Instance::new(&m, &imports).unwrap());
//     env.instance = NonNull::new(instance.as_mut());
//     let pusher = instance.exports.get_native_function("push").unwrap();
//     env.pusher = NonNull::new({
//         let mut fun = Box::new(pusher);
//         fun.as_mut()
//     });
//     let main: NativeFunc<ExternRef, ExternRef> =
//         instance.exports.get_native_function("main").unwrap();

//     let called = main.call(ExternRef::new(params)).unwrap();
//     called.downcast::<Value>().cloned()
// }

#[cfg(test)]
mod test {
    use bumpalo::{boxed::Box, Bump};
    use wasmer::{imports, wat2wasm};
    use wasmer_middlewares::metering::set_remaining_points;

    use crate::compile_store;

    use super::*;
    #[test]
    fn testing() {
        let expected = 6;
        let module = wat2wasm(
            br#"
            (module
              (func $main (param $n i64) (result i64)
                local.get $n
              )
            (export "main" (func $main))
          )
        "#,
        )
        .unwrap();
        #[derive(PartialEq, Debug)]
        pub struct T {
            pub v: String,
            pub z: Option<Box<'static, T>>,
        }
        let ar: Bump = Bump::with_capacity(8000);
        ar.set_allocation_limit(Some(8000));
        let vv = Box::new_in(
            T {
                v: "String".to_string(),
                z: None,
            },
            &ar,
        );
        let to_pass = Box::into_raw(vv) as *mut usize;
        let store = compile_store::new_compile_store();
        let module = Module::new(&store, module).unwrap();
        let imports = imports! {};
        let result = Instance::new(&module, &imports).unwrap();
        set_remaining_points(&result, 1000);
        let main: NativeFunc<i64, i64> = result.exports.get_native_function("main").unwrap();
        let res = main.call(to_pass as i64).unwrap();
        assert_eq!(res, to_pass as i64)
    }
}
