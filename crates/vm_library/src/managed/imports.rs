use std::{
    borrow::Borrow,
    ops::{Add, Sub},
};

use im_rc::Vector;
use rug::Integer;
use slotmap::{DefaultKey, Key, KeyData};
use wasmer::{Exports, Function, ImportObject, Store};

use super::value::*;
use crate::conversions;
use crate::{
    env::Context,
    errors::{ffi::FFIError, vm::VmError, VMResult},
};

pub fn compare(env: &Context, value1: Value, value2: Value) -> VMResult<i64> {
    env.update_gas(300)?;
    let cmp_res = (value1).cmp(&value2) as i8;
    let bumped = env.bump(Value::Int(cmp_res.into()));
    conversions::to_i64(bumped)
}

pub fn equal(env: &Context, value1: Value, value2: Value) -> VMResult<i64> {
    env.update_gas(300)?;
    let cmp_res = value1.eq(&value2);
    let bumped = env.bump(Value::Bool(cmp_res));
    conversions::to_i64(bumped)
}
pub fn or(env: &Context, value1: Value, value2: Value) -> VMResult<i64> {
    env.update_gas(300)?;
    let res: VMResult<Value> = match (value1, value2) {
        (Value::Bool(x), Value::Bool(y)) => Ok(Value::Bool(x | y)),
        (x, _) => Err(FFIError::ExternError {
            value: x,
            msg: "type mismatch, expected Two Bools".to_string(),
        }
        .into()),
    };
    let res = res?;
    let bumped = env.bump(res);
    conversions::to_i64(bumped)
}
pub fn neq(env: &Context, value: Value) -> VMResult<i64> {
    env.update_gas(300)?;
    let one: rug::Integer = rug::Integer::from(1);

    let res: VMResult<Value> = match value {
        Value::Int(n) if n == Integer::ZERO => Ok(false),
        Value::Int(n) if n == one => Ok(true),

        _ => Err(FFIError::ExternError {
            value: (value).clone(),
            msg: "type mismatch, expected Pair".to_string(),
        }
        .into()),
    }
    .map(Value::Bool);
    let res = res?;
    let bumped = env.bump(res);
    conversions::to_i64(bumped)
}

pub fn not(env: &Context, value: Value) -> VMResult<i64> {
    env.update_gas(300)?;
    let res: VMResult<Value> = match value {
        Value::Bool(n) => Ok(!n),

        _ => Err(FFIError::ExternError {
            value,
            msg: "type mismatch, expected Pair".to_string(),
        }
        .into()),
    }
    .map(Value::Bool);
    let res = res?;
    let bumped = env.bump(res);
    conversions::to_i64(bumped)
}
pub fn pair(env: &Context, value1: Value, value2: Value) -> VMResult<i64> {
    env.update_gas(300)?;
    let fst = env.bump_raw(value1);
    let snd = env.bump_raw(value2);
    let res = Value::Pair { fst, snd };
    let key = env.bump(res);
    conversions::to_i64(key)
}
pub fn unpair(env: &Context, value: Value) -> VMResult<i64> {
    env.update_gas(300)?;
    match value {
        Value::Pair { fst, snd } => {
            let fst = conversions::to_i64(fst.data().as_ffi())?;
            let snd = conversions::to_i64(snd.data().as_ffi())?;
            env.push_value(snd)?;
            Ok(fst)
        }
        _ => Err(FFIError::ExternError {
            value,
            msg: "type mismatch, expected Pair".to_string(),
        }
        .into()),
    }
}
pub fn car(env: &Context, value: Value) -> VMResult<i64> {
    env.update_gas(300)?;
    match value {
        Value::Pair { fst, snd: _ } => conversions::to_i64(fst.data().as_ffi()),
        _ => Err(FFIError::ExternError {
            value,
            msg: "type mismatch, expected Pair".to_string(),
        }
        .into()),
    }
}
pub fn cdr(env: &Context, value: Value) -> VMResult<i64> {
    env.update_gas(300)?;
    match value {
        Value::Pair { fst: _, snd } => conversions::to_i64(snd.data().as_ffi()),
        _ => Err(FFIError::ExternError {
            value: (value),
            msg: "type mismatch, expected Pair".to_string(),
        }
        .into()),
    }
}
pub fn z_add(env: &Context, value1: Value, value2: Value) -> VMResult<i64> {
    env.update_gas(300)?;
    match (value1, value2) {
        (Value::Int(x), Value::Int(y)) => {
            let res = Value::Int((x).add(y));
            let key = env.bump(res);
            conversions::to_i64(key)
        }
        (Value::Int(_), err) | (err, Value::Int(_)) => Err(FFIError::ExternError {
            value: (err),
            msg: "type mismatch, expected Int".to_string(),
        }
        .into()),
        (x, _) => Err(FFIError::ExternError {
            value: x,
            msg: "type mismatch, expected Int".to_string(),
        }
        .into()),
    }
}
pub fn z_sub(env: &Context, value1: Value, value2: Value) -> VMResult<i64> {
    env.update_gas(300)?;
    match (value1, value2) {
        (Value::Int(x), Value::Int(y)) => {
            let res = Value::Int((x).sub(y));
            let key = env.bump(res);
            conversions::to_i64(key)
        }
        (Value::Int(_), err) | (err, Value::Int(_)) => Err(FFIError::ExternError {
            value: (err),
            msg: "type mismatch, expected Int".to_string(),
        }
        .into()),
        (x, _) => Err(FFIError::ExternError {
            value: x,
            msg: "type mismatch, expected Int".to_string(),
        }
        .into()),
    }
}
pub fn is_left(env: &Context, value: Value) -> VMResult<i32> {
    env.update_gas(300)?;
    match value {
        Value::Union(Union::Left(l)) => {
            let key = conversions::to_i64(l.data().as_ffi())?;
            env.push_value(key)?;
            Ok(1)
        }
        Value::Union(Union::Right(l)) => {
            let key = conversions::to_i64(l.data().as_ffi())?;
            env.push_value(key)?;
            Ok(0)
        }
        _ => Err(FFIError::ExternError {
            value: (value),
            msg: "type mismatch, expected Union".to_string(),
        }
        .into()),
    }
}
pub fn deref_bool(env: &Context, value: Value) -> VMResult<i32> {
    env.update_gas(300)?;
    match value {
        Value::Bool(x) => Ok((x).into()),
        _ => Err(FFIError::ExternError {
            value: (value),
            msg: "type mismatch, expected Bool".to_string(),
        }
        .into()),
    }
}
pub fn failwith(env: &Context, value: Value) -> VMResult<()> {
    env.update_gas(300)?;
    match value {
        Value::String(str) => Err(VmError::RuntimeErr(str)),
        _ => Err(FFIError::ExternError {
            value: (value).clone(),
            msg: "type mismatch, expected String".to_string(),
        }
        .into()),
    }
}
pub fn is_none(env: &Context, value: Value) -> VMResult<i32> {
    env.update_gas(300)?;
    match value {
        Value::Option(x) => (x).map_or_else(
            || {
                let res = Value::Int(0.into());
                let bumped = env.bump(res);
                let key = conversions::to_i64(bumped)?;
                env.push_value(key)?;
                Ok(1)
            },
            |v| {
                let key = conversions::to_i64(v.data().as_ffi())?;
                env.push_value(key)?;
                Ok(0)
            },
        ),
        _ => Err(FFIError::ExternError {
            value: (value),
            msg: "type mismatch, expected Option".to_string(),
        }
        .into()),
    }
}
pub fn is_nat(env: &Context, value: Value) -> VMResult<i64> {
    env.update_gas(300)?;
    match value {
        Value::Int(x) if x >= Integer::ZERO => {
            let bumped = env.bump(Value::Int(x));
            let opt = Value::Option(Some(DefaultKey::from(KeyData::from_ffi(bumped))));
            let bumped = env.bump(opt);
            let key = conversions::to_i64(bumped)?;
            Ok(key)
        }
        Value::Int(_) => {
            let opt = Value::Option(None);
            let bumped = env.bump(opt);
            let key = conversions::to_i64(bumped)?;
            Ok(key)
        }
        _ => Err(FFIError::ExternError {
            value: (value).clone(),
            msg: "type mismatch, expected Nat".to_string(),
        }
        .into()),
    }
}
pub fn some(env: &Context, value: Value) -> VMResult<i64> {
    env.update_gas(300)?;
    let bumped = env.bump(value);
    let opt = Value::Option(Some(DefaultKey::from(KeyData::from_ffi(bumped))));
    let bumped = env.bump(opt);
    let key = conversions::to_i64(bumped)?;
    Ok(key)
}
pub fn get_n(env: &Context, idx: u32, value: Value) -> VMResult<i64> {
    env.update_gas(300 * (idx as u64))?;
    if idx == 0 {
        let bumped = env.bump(value);
        let key = conversions::to_i64(bumped)?;
        return Ok(key);
    }
    let mut current = value;
    let mut loop_idx = idx;
    loop {
        if loop_idx == 0 {
            let bumped = env.bump(current);
            let key = conversions::to_i64(bumped)?;
            return Ok(key);
        }
        match (loop_idx, current) {
            (1, Value::Pair { fst, snd: _ }) => {
                current = env.get(fst)?;
                break;
            }
            (2, Value::Pair { fst: _, snd }) => {
                current = env.get(snd)?;
                break;
            }
            (_, Value::Pair { fst: _, snd }) => {
                current = env.get(snd)?;
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
    let bumped = env.bump(current);
    let key = conversions::to_i64(bumped)?;
    Ok(key)
}

pub fn mem(env: &Context, value1: Value, value2: Value) -> VMResult<i64> {
    env.update_gas(300)?;
    match value1 {
        Value::Map(x) => {
            let res = x.contains_key(&value2);
            let bumped = env.bump(Value::Bool(res));
            conversions::to_i64(bumped)
        }
        Value::Set(x) => {
            let res = x.contains(&value2);
            let bumped = env.bump(Value::Bool(res));
            conversions::to_i64(bumped)
        }
        _ => Err(FFIError::ExternError {
            value: value1,
            msg: "type mismatch, expected Map/Set with a Key".to_string(),
        }
        .into()),
    }
}
pub fn map_get(env: &Context, value1: Value, value2: Value) -> VMResult<i64> {
    env.update_gas(300)?;
    match value1 {
        Value::Map(x) => {
            let res = x.get(&value2);
            let bumped = res.map(|res| env.bump_raw(res.clone()));
            let bumped = env.bump(Value::Option(bumped));
            conversions::to_i64(bumped)
        }
        _ => Err(FFIError::ExternError {
            value: value1,
            msg: "type mismatch, expected Map with a Key".to_string(),
        }
        .into()),
    }
}
pub fn update(env: &Context, map: Value, key: Value, value: Value) -> VMResult<i64> {
    env.update_gas(300)?;
    match (&map, value) {
        (Value::Map(x), Value::Option(boxed)) => {
            let mut map = x.clone();
            match boxed {
                None => {
                    map.remove(&key);
                }
                Some(x) => {
                    let x = env.get(x)?;
                    map.insert(key, x);
                }
            }
            let bumped = env.bump(Value::Map(map));
            conversions::to_i64(bumped)
        }
        _ => Err(FFIError::ExternError {
            value: map.clone(),
            msg: "type mismatch, expected Map with a Option Value".to_string(),
        }
        .into()),
    }
}
pub const fn call1<A, F>(f: F) -> impl Fn(&Context, i64) -> VMResult<A>
where
    F: Fn(&Context, Value) -> VMResult<A>,
{
    move |env, arg| match env.get(DefaultKey::from(KeyData::from_ffi(arg as u64))) {
        Ok(x) => f(env, x),
        Err(x) => Err(x),
    }
}
pub const fn call2<F, A>(f: F) -> impl Fn(&Context, i64, i64) -> VMResult<A>
where
    F: Fn(&Context, Value, Value) -> VMResult<A>,
{
    move |env, arg, arg2| match (
        env.get(DefaultKey::from(KeyData::from_ffi(arg as u64))),
        env.get(DefaultKey::from(KeyData::from_ffi(arg2 as u64))),
    ) {
        (Ok(x), Ok(y)) => f(env, x, y),
        (_, _) => Err(VmError::RuntimeErr("illegal argument".to_string())),
    }
}

pub const fn call2_extra<F, A>(f: F) -> impl Fn(&Context, u32, i64) -> VMResult<A>
where
    F: Fn(&Context, u32, Value) -> VMResult<A>,
{
    move |env, arg, arg2| match env.get(DefaultKey::from(KeyData::from_ffi(arg2 as u64))) {
        Ok(x) => f(env, arg, x),
        _ => Err(VmError::RuntimeErr("illegal argument".to_string())),
    }
}
pub const fn call2_default<F, A>(f: F) -> impl Fn(&Context, u32, i64) -> VMResult<A>
where
    F: Fn(&Context, DefaultKey, DefaultKey) -> VMResult<A>,
{
    move |env, arg, arg2| {
        let (x, y) = (
            DefaultKey::from(KeyData::from_ffi(arg as u64)),
            DefaultKey::from(KeyData::from_ffi(arg2 as u64)),
        );
        f(env, x, y)
    }
}
pub const fn call2_default_value<F, A>(f: F) -> impl Fn(&Context, u32, i64) -> VMResult<A>
where
    F: Fn(&Context, Value, DefaultKey) -> VMResult<A>,
{
    move |env, arg, arg2| {
        let (x, y) = (
            env.get(DefaultKey::from(KeyData::from_ffi(arg as u64)))?,
            DefaultKey::from(KeyData::from_ffi(arg2 as u64)),
        );
        f(env, x, y)
    }
}
pub const fn call3<F, A>(f: F) -> impl Fn(&Context, i64, i64, i64) -> VMResult<A>
where
    F: Fn(&Context, Value, Value, Value) -> VMResult<A>,
{
    move |env, arg, arg2, arg3| match (
        env.get(DefaultKey::from(KeyData::from_ffi(arg as u64))),
        env.get(DefaultKey::from(KeyData::from_ffi(arg3 as u64))),
        env.get(DefaultKey::from(KeyData::from_ffi(arg2 as u64))),
    ) {
        (Ok(x), Ok(y), Ok(z)) => f(env, x, z, y),
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
        "if_left",
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
        "isnat",
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
    // TODO FIXUp
    exports.insert(
        "nil",
        Function::new_native_with_env(store, env.clone(), nil),
    );
    exports.insert(
        "zero",
        Function::new_native_with_env(store, env.clone(), todo_single),
    );
    exports.insert(
        "empty_set",
        Function::new_native_with_env(store, env.clone(), todo_single),
    );
    exports.insert(
        "sender",
        Function::new_native_with_env(store, env.clone(), todo_single),
    );
    exports.insert(
        "source",
        Function::new_native_with_env(store, env.clone(), todo_single),
    );
    exports.insert(
        "iter",
        Function::new_native_with_env(store, env.clone(), todo_single2),
    );
    exports.insert(
        "is_left",
        Function::new_native_with_env(store, env.clone(), todo_single3),
    );
    exports.insert(
        "is_right",
        Function::new_native_with_env(store, env.clone(), todo_single3),
    );
    exports.insert(
        "string",
        Function::new_native_with_env(store, env.clone(), todo_single4),
    );
    exports.insert(
        "exec",
        Function::new_native_with_env(store, env.clone(), todo_single6),
    );
    exports.insert(
        "apply",
        Function::new_native_with_env(store, env.clone(), todo_single6),
    );
    exports.insert(
        "const",
        Function::new_native_with_env(store, env.clone(), todo_single4),
    );
    exports.insert(
        "get_some",
        Function::new_native_with_env(store, env.clone(), todo_single5),
    );
    exports.insert(
        "get_left",
        Function::new_native_with_env(store, env.clone(), todo_single5),
    );
    exports.insert(
        "get_right",
        Function::new_native_with_env(store, env.clone(), todo_single5),
    );
    exports.insert(
        "abs",
        Function::new_native_with_env(store, env.clone(), todo_single5),
    );
    exports.insert(
        "lt",
        Function::new_native_with_env(store, env.clone(), todo_single5),
    );
    exports.insert(
        "gt",
        Function::new_native_with_env(store, env.clone(), todo_single5),
    );
    exports.insert(
        "closure",
        Function::new_native_with_env(store, env.clone(), todo_single4),
    );
    imports.register("env", exports);
    imports
}
fn todo_single(_c: &Context) -> i64 {
    todo!();
}
fn nil(c: &Context) -> VMResult<i64> {
    let v = Value::List(Vector::new());
    let bumped = c.bump(v);
    conversions::to_i64(bumped)
}
fn todo_single2(_c: &Context, _d: i64, _x: i32) -> i64 {
    todo!();
}
fn todo_single3(_c: &Context, _d: i64) -> i32 {
    1
}
fn todo_single4(_c: &Context, _d: i32) -> i64 {
    todo!();
}
fn todo_single5(_c: &Context, _d: i64) -> i64 {
    todo!();
}
fn todo_single6(_c: &Context, _d: i64, _x: i64) -> i64 {
    todo!();
}
