use crate::{
    errors::{vm::VmError, VMResult},
    managed::value::Value,
};
use slotmap::{DefaultKey, HopSlotMap, Key};
use std::{cell::RefCell, ptr::NonNull, rc::Rc};
use wasmer::{Instance, WasmerEnv};
use wasmer_middlewares::metering::{get_remaining_points, set_remaining_points, MeteringPoints};

#[derive(WasmerEnv, Clone)]
pub struct Context {
    pub instance: Option<NonNull<Instance>>,
    pub pusher: Option<NonNull<wasmer::NativeFunc<i64, ()>>>,
    pub gas_limit: u64,
    pub arena: Rc<RefCell<HopSlotMap<DefaultKey, Value>>>,
}
unsafe impl Send for Context {}

unsafe impl Sync for Context {}
impl Context {
    pub fn with_instance<C, R>(&self, callback: C) -> VMResult<R>
    where
        C: FnOnce(&Instance) -> VMResult<R>,
    {
        match self.instance {
            Some(instance_ptr) => {
                let instance_ref = unsafe { instance_ptr.as_ref() };
                callback(instance_ref)
            }
            None => Err(VmError::InstantiationErr(
                "instance missing, lifecycle error".to_string(),
            )),
        }
    }
    pub fn get_gas_left(&self) -> u64 {
        self.with_instance(|instance| {
            Ok(match get_remaining_points(instance) {
                MeteringPoints::Remaining(count) => count,
                MeteringPoints::Exhausted => 0,
            })
        })
        .expect("impossible")
    }

    fn set_gas_left(&self, new_value: u64) {
        self.with_instance(|instance| {
            set_remaining_points(instance, new_value);
            Ok(())
        })
        .expect("impossible")
    }
    pub fn update_gas(&self, cost: u64) -> VMResult<()> {
        let gas_left = self.get_gas_left();
        if cost > gas_left {
            Err(VmError::OutOfGas)
        } else {
            let new_limit = gas_left.saturating_sub(cost);
            self.set_gas_left(new_limit);
            Ok(())
        }
    }
    pub fn push_value(&self, value: i64) -> VMResult<()> {
        match self.pusher {
            Some(instance_ptr) => {
                let func = unsafe { instance_ptr.as_ref() };
                func.call(value)
                    .map_err(|x| VmError::RuntimeErr(x.to_string()))
            }
            None => Err(VmError::InstantiationErr(
                "instance missing, lifecycle error".to_string(),
            )),
        }
    }
    pub fn bump(&self, value: Value) -> u64 {
        self.arena
            .as_ref()
            .borrow_mut()
            .insert(value)
            .data()
            .as_ffi()
    }
}
