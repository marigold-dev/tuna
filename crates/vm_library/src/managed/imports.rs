use std::ops::Add;

use rug::Integer;
use slotmap::{DefaultKey, Key};
use wasmer::{Exports, Function, ImportObject, Store};

use super::value::*;
use crate::conversions;
use crate::{
    env::Context,
    errors::{ffi::FFIError, vm::VmError, VMResult},
};

pub fn compare(env: &Context, value1: &Value, value2: &Value) -> VMResult<i64> {
    env.update_gas(300)?;
    let cmp_res = (*value1).cmp(value2) as i64;
    Ok(cmp_res)
}
pub fn equal(env: &Context, value1: &Value, value2: &Value) -> VMResult<i64> {
    env.update_gas(300)?;
    let cmp_res = value1.eq(value2) as i64;
    Ok(cmp_res)
}
pub fn or(env: &Context, value1: &i64, value2: &i64) -> VMResult<i64> {
    env.update_gas(300)?;
    Ok(value1 | value2)
}

pub fn neq(env: &Context, value: &Value) -> VMResult<i64> {
    env.update_gas(300)?;
    match value {
        Value::Int(n) if n == &Integer::ZERO => Ok(0),
        Value::Int(n) if n == &Integer::from(1) => Ok(1),

        _ => Err(FFIError::ExternError {
            value: (*value).clone(),
            msg: "type mismatch, expected Pair".to_string(),
        }
        .into()),
    }
}

pub fn not(env: &Context, value: &i64) -> VMResult<i64> {
    env.update_gas(300)?;
    Ok(!value)
}
pub fn pair(env: &Context, value1: DefaultKey, value2: DefaultKey) -> VMResult<i64> {
    env.update_gas(300)?;
    let res = Value::Pair {
        fst: value1,
        snd: value2,
    };
    let key = env.bump(res);
    conversions::to_i64(key)
}
pub fn unpair(env: &Context, value: &Value) -> VMResult<(ExternRef, ExternRef)> {
    env.update_gas(300)?;
    match value {
        Value::Pair(boxed) => Ok((
            ExternRef::new(boxed.1.clone()),
            ExternRef::new(boxed.0.clone()),
        )),
        _ => Err(FFIError::ExternError {
            value: (*value).clone(),
            msg: "type mismatch, expected Pair".to_string(),
        }
        .into()),
    }
}
pub fn car(env: &Context, value: &Value) -> VMResult<i64> {
    env.update_gas(300)?;
    match value {
        Value::Pair(boxed) => Ok(ExternRef::new(boxed.0.clone())),
        _ => Err(FFIError::ExternError {
            value: (*value).clone(),
            msg: "type mismatch, expected Pair".to_string(),
        }
        .into()),
    }
}
pub fn cdr(env: &Context, value: &Value) -> VMResult<i64> {
    env.update_gas(300)?;
    match value {
        Value::Pair(boxed) => Ok(ExternRef::new(boxed.1.clone())),
        _ => Err(FFIError::ExternError {
            value: (*value).clone(),
            msg: "type mismatch, expected Pair".to_string(),
        }
        .into()),
    }
}
pub fn z_add(env: &Context, value1: &Value, value2: &Value) -> VMResult<i64> {
    env.update_gas(300)?;
    match (value1, value2) {
        (Value::Int(x), Value::Int(y)) => {
            let res = (x.clone()).add(y.clone());
            Ok(ExternRef::new(Value::Int(res)))
        }
        (Value::Int(_), err) | (err, Value::Int(_)) => Err(FFIError::ExternError {
            value: (*err).clone(),
            msg: "type mismatch, expected Int".to_string(),
        }
        .into()),
        (_, _) => Err(FFIError::ExternError {
            value: Value::Pair(Box::new(((*value1).clone(), (*value2).clone()))),
            msg: "type mismatch, expected Int".to_string(),
        }
        .into()),
    }
}
pub fn z_sub(env: &Context, value1: &Value, value2: &Value) -> VMResult<i64> {
    env.update_gas(300)?;
    match (value1, value2) {
        (Value::Int(x), Value::Int(y)) => {
            let res = (x.clone()) - (y.clone());
            Ok(ExternRef::new(Value::Int(res)))
        }
        (Value::Int(_), err) | (err, Value::Int(_)) => Err(FFIError::ExternError {
            value: (*err).clone(),
            msg: "type mismatch, expected Int".to_string(),
        }
        .into()),
        (_, _) => Err(FFIError::ExternError {
            value: Value::Pair(Box::new(((*value1).clone(), (*value2).clone()))),
            msg: "type mismatch, expected Int".to_string(),
        }
        .into()),
    }
}
pub fn is_left(env: &Context, value: &Value) -> VMResult<(i64, i32)> {
    env.update_gas(300)?;
    match value {
        Value::Union(Union::Left(l)) => Ok((ExternRef::new((*l).clone()), 1)),
        _ => Err(FFIError::ExternError {
            value: (*value).clone(),
            msg: "type mismatch, expected Union".to_string(),
        }
        .into()),
    }
}
pub fn deref_bool(env: &Context, value: &Value) -> VMResult<i32> {
    env.update_gas(300)?;
    match value {
        Value::Bool(x) => Ok((*x).into()),
        _ => Err(FFIError::ExternError {
            value: (*value).clone(),
            msg: "type mismatch, expected Bool".to_string(),
        }
        .into()),
    }
}
pub fn failwith(env: &Context, value: &Value) -> VMResult<()> {
    env.update_gas(300)?;
    match value {
        Value::String(str) => Err(VmError::RuntimeErr(str.clone())),
        _ => Err(FFIError::ExternError {
            value: (*value).clone(),
            msg: "type mismatch, expected String".to_string(),
        }
        .into()),
    }
}
pub fn is_none(env: &Context, value: &Value) -> VMResult<(i64, i32)> {
    env.update_gas(300)?;
    match value {
        Value::Option(x) => (*x).clone().map_or_else(
            || Ok((ExternRef::new(Value::Int(0.into())), 1)),
            |v| Ok((ExternRef::new(v), 0)),
        ),
        _ => Err(FFIError::ExternError {
            value: (*value).clone(),
            msg: "type mismatch, expected Option".to_string(),
        }
        .into()),
    }
}
pub fn is_nat(env: &Context, value: &Value) -> VMResult<i64> {
    env.update_gas(300)?;
    match value {
        Value::Int(x) if *x >= Integer::ZERO => Ok(ExternRef::new(Value::Option(Box::new(Some(
            (*value).clone(),
        ))))),
        Value::Int(_) => Ok(ExternRef::new(Value::Option(Box::default()))),
        _ => Err(FFIError::ExternError {
            value: (*value).clone(),
            msg: "type mismatch, expected Nat".to_string(),
        }
        .into()),
    }
}
pub fn some(env: &Context, value: &Value) -> VMResult<i64> {
    env.update_gas(300)?;
    Ok(ExternRef::new(Value::Option(Box::new(Some(
        (*value).clone(),
    )))))
}
pub fn get_n(env: &Context, idx: u32, value: &Value) -> VMResult<i64> {
    env.update_gas(300 * (idx as u64))?;
    if idx == 0 {
        return Ok(ExternRef::new((*value).clone()));
    }
    let mut current = (*value).clone();
    let mut loop_idx = idx;
    loop {
        if loop_idx == 0 {
            return Ok(ExternRef::new(current));
        }
        match (loop_idx, current) {
            (1, Value::Pair(b)) => {
                current = b.0;
                break;
            }
            (2, Value::Pair(b)) => {
                current = b.1;
                break;
            }
            (_, Value::Pair(b)) => {
                current = b.1;
                loop_idx = loop_idx.saturating_sub(2);
            }
            (_, value) => {
                return Err(FFIError::ExternError {
                    value: (value),
                    msg: "type mismatch, expected Pair".to_string(),
                }
                .into())
            }
        }
    }
    Ok(ExternRef::new(current))
}
pub fn mem(env: &Context, value1: &Value, value2: &Value) -> VMResult<i64> {
    env.update_gas(300)?;
    match value1 {
        Value::Map(x) => {
            let res = x.contains_key(value2);
            Ok(ExternRef::new(Value::Bool(res)))
        }
        Value::Set(x) => {
            let res = x.contains(value2);
            Ok(ExternRef::new(Value::Bool(res)))
        }
        _ => Err(FFIError::ExternError {
            value: Value::Pair(Box::new(((*value1).clone(), (*value2).clone()))),
            msg: "type mismatch, expected Map/Set with a Key".to_string(),
        }
        .into()),
    }
}
pub fn map_get(env: &Context, value1: &Value, value2: &Value) -> VMResult<i64> {
    env.update_gas(300)?;
    match value1 {
        Value::Map(x) => {
            let res = x.get(value2);
            Ok(ExternRef::new(res.map_or_else(
                || Value::Option(Box::default()),
                |x| Value::Option(Box::new(Some((*x).clone()))),
            )))
        }
        _ => Err(FFIError::ExternError {
            value: Value::Pair(Box::new(((*value1).clone(), (*value2).clone()))),
            msg: "type mismatch, expected Map with a Key".to_string(),
        }
        .into()),
    }
}
pub fn update(env: &Context, map: &Value, key: &Value, value: &Value) -> VMResult<i64> {
    env.update_gas(300)?;
    match (map, value) {
        (Value::Map(x), Value::Option(boxed)) => {
            let mut map = x.clone();
            (*boxed).clone().map(|x| map.insert((*key).clone(), x));
            Ok(ExternRef::new(Value::Map(map)))
        }
        _ => Err(FFIError::ExternError {
            value: Value::Pair(Box::new((map.clone(), value.clone()))),
            msg: "type mismatch, expected Map with a Option Value".to_string(),
        }
        .into()),
    }
}
pub const fn call1<A, F>(f: F) -> impl Fn(&Context, ExternRef) -> VMResult<A>
where
    F: Fn(&Context, &Value) -> VMResult<A>,
{
    move |env, arg| match arg.downcast::<Value>() {
        Some(x) => f(env, x),
        None => Err(VmError::RuntimeErr("illegal argument".to_string())),
    }
}
pub const fn call2<F, A>(f: F) -> impl Fn(&Context, ExternRef, ExternRef) -> VMResult<A>
where
    F: Fn(&Context, &Value, &Value) -> VMResult<A>,
{
    move |env, arg, arg2| match (arg.downcast::<Value>(), arg2.downcast::<Value>()) {
        (Some(x), Some(y)) => f(env, x, y),
        _ => Err(VmError::RuntimeErr("illegal argument".to_string())),
    }
}

pub const fn call2_extra<F, A>(f: F) -> impl Fn(&Context, u32, ExternRef) -> VMResult<A>
where
    F: Fn(&Context, u32, &Value) -> VMResult<A>,
{
    move |env, arg, arg2| match arg2.downcast::<Value>() {
        Some(x) => f(env, arg, x),
        _ => Err(VmError::RuntimeErr("illegal argument".to_string())),
    }
}
pub const fn call3<F, A>(f: F) -> impl Fn(&Context, ExternRef, ExternRef, ExternRef) -> VMResult<A>
where
    F: Fn(&Context, &Value, &Value, &Value) -> VMResult<A>,
{
    move |env, arg, arg2, arg3| match (
        arg.downcast::<Value>(),
        arg2.downcast::<Value>(),
        arg3.downcast::<Value>(),
    ) {
        (Some(x), Some(y), Some(z)) => f(env, x, y, z),
        _ => Err(VmError::RuntimeErr("illegal argument".to_string())),
    }
}
pub fn make_imports(env: &Context, store: &Store) -> ImportObject {
    let mut imports = ImportObject::new();
    let mut exports = Exports::new();

    exports.insert(
        "compare",
        Function::new_native_with_env(store, env.clone(), call2(compare)),
    );
    exports.insert(
        "equal",
        Function::new_native_with_env(store, env.clone(), call2(equal)),
    );
    exports.insert(
        "or",
        Function::new_native_with_env(store, env.clone(), call2(or)),
    );
    exports.insert(
        "neq",
        Function::new_native_with_env(store, env.clone(), call1(neq)),
    );
    exports.insert(
        "not",
        Function::new_native_with_env(store, env.clone(), call1(not)),
    );
    exports.insert(
        "pair",
        Function::new_native_with_env(store, env.clone(), call2(pair)),
    );
    exports.insert(
        "unpair",
        Function::new_native_with_env(store, env.clone(), call1(unpair)),
    );
    exports.insert(
        "car",
        Function::new_native_with_env(store, env.clone(), call1(car)),
    );
    exports.insert(
        "cdr",
        Function::new_native_with_env(store, env.clone(), call1(cdr)),
    );
    exports.insert(
        "z_add",
        Function::new_native_with_env(store, env.clone(), call2(z_add)),
    );
    exports.insert(
        "z_sub",
        Function::new_native_with_env(store, env.clone(), call2(z_sub)),
    );
    exports.insert(
        "is_left",
        Function::new_native_with_env(store, env.clone(), call1(is_left)),
    );
    exports.insert(
        "deref_bool",
        Function::new_native_with_env(store, env.clone(), call1(deref_bool)),
    );
    exports.insert(
        "failwith",
        Function::new_native_with_env(store, env.clone(), call1(failwith)),
    );
    exports.insert(
        "is_none",
        Function::new_native_with_env(store, env.clone(), call1(is_none)),
    );
    exports.insert(
        "is_nat",
        Function::new_native_with_env(store, env.clone(), call1(is_nat)),
    );
    exports.insert(
        "some",
        Function::new_native_with_env(store, env.clone(), call1(some)),
    );
    exports.insert(
        "get_n",
        Function::new_native_with_env(store, env.clone(), call2_extra(get_n)),
    );
    exports.insert(
        "mem",
        Function::new_native_with_env(store, env.clone(), call2(mem)),
    );
    exports.insert(
        "map_get",
        Function::new_native_with_env(store, env.clone(), call2(map_get)),
    );
    exports.insert(
        "update",
        Function::new_native_with_env(store, env.clone(), call3(update)),
    );
    imports.register("env", exports);
    imports
}
