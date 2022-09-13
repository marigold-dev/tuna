use crate::{
    errors::{vm::VmError, VMResult},
    managed::value::Value,
};
use std::ptr::NonNull;
use wasmer::{ExternRef, Instance, WasmerEnv};
use wasmer_middlewares::metering::{get_remaining_points, set_remaining_points, MeteringPoints};

#[derive(WasmerEnv, Clone)]
pub struct Context {
    pub instance: Option<NonNull<Instance>>,
    pub pusher: Option<NonNull<wasmer::NativeFunc<ExternRef, ()>>>,
    pub gas_limit: u64,
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
    pub fn push_value(&self, value: Value) -> VMResult<()> {
        match self.pusher {
            Some(instance_ptr) => {
                let func = unsafe { instance_ptr.as_ref() };
                func.call(ExternRef::new(value))
                    .map_err(|x| VmError::RuntimeErr(x.to_string()))
            }
            None => Err(VmError::InstantiationErr(
                "instance missing, lifecycle error".to_string(),
            )),
        }
    }
}
