use std::{cell::RefCell, ptr::NonNull, rc::Rc};

use slotmap::{DefaultKey, Key, KeyData};
use wasmer::{Instance, Module};

use crate::{
    arena::{populate_predef, push_constants, ARENA},
    compile_store,
    env::{Context, Inner},
    errors::{vm::VmError, VMResult},
    incoming::InvokeManaged,
    managed::{imports, value::Value},
    outgoing::{Outgoing, OutgoingManaged},
    ticket_table::TicketTable,
};

pub fn invoke_managed(t: InvokeManaged) -> VMResult<Outgoing> {
    let arena = unsafe { &mut ARENA };

    let module: VMResult<Module> =
        unsafe { Module::deserialize(&compile_store::new_headless(), &t.mod_).map_err(Into::into) };
    let env = Context {
        inner: Rc::new(RefCell::new(Inner {
            instance: None,
            pusher: None,
            gas_limit: 10000,
            call_unit: None,
            call: None,
            ticket_table: TicketTable::default(),
        })),
    };
    populate_predef(t.sender, t.self_addr, t.source);
    push_constants(t.constants);
    let module = module?;
    let store = module.store();

    let instance = Box::from(
        Instance::new(&module, &imports::make_imports(&env, store))
            .map_err(|_| VmError::RuntimeErr("Failed to create instance".to_owned()))?,
    );

    {
        let new = NonNull::from(instance.as_ref());
        let pusher = Box::from(
            instance
                .exports
                .get_native_function::<i64, ()>("push")
                .map_err(|_| VmError::RuntimeErr("Miscompiled contract".to_owned()))?,
        );
        let call_unit = Box::from(
            instance
                .exports
                .get_native_function::<(i64, i32), ()>("call_callback_unit")
                .map_err(|_| VmError::RuntimeErr("Miscompiled contract".to_owned()))?,
        );
        let call = Box::from(
            instance
                .exports
                .get_native_function::<(i64, i32), i64>("call_callback")
                .map_err(|_| VmError::RuntimeErr("Miscompiled contract".to_owned()))?,
        );

        env.set_instance(Some(new));
        env.set_pusher(Some(NonNull::from(pusher.as_ref())));
        env.set_call_unit(Some(NonNull::from(call_unit.as_ref())));

        env.set_call(Some(NonNull::from(call.as_ref())));

        env.set_gas_left(t.gas_limit as u64);
    }
    let fst = arena.insert(t.arg);
    let snd = arena.insert(t.initial_storage);
    let arg = Value::Pair { fst, snd };
    let arg = arena.insert(arg).data().as_ffi();

    let caller = instance
        .exports
        .get_native_function::<i64, i64>("main")
        .map_err(|_| VmError::RuntimeErr("Miscompiled contract".to_owned()))?;

    let result: VMResult<i64> = caller.call(arg as i64).map_err(Into::into);
    let result = result?;
    let key = DefaultKey::from(KeyData::from_ffi(result as u64));
    let value = arena.get(key);

    value.map_or_else(
        || {
            Err(VmError::RuntimeErr(
                "Runtime Error, result not available".to_owned(),
            ))
        },
        |ok| match ok {
            Value::Pair { fst, snd } => {
                let value = env.get(*snd)?;
                let ops = env.get(*fst)?;
                Ok(Outgoing::OutgoingManaged {
                    payload: Box::from(OutgoingManaged {
                        new_storage: value,
                        operations: ops,
                        contract_tickets: vec![],
                        remaining_gas: env.get_gas_left() as usize,
                    }),
                })
            }
            _ => Err(VmError::RuntimeErr(
                "Type mismatch in final result, result not available".to_owned(),
            )),
        },
    )
}
