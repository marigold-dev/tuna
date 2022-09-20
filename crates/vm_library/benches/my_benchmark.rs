use std::{borrow::Cow, cell::RefCell, ptr::NonNull, rc::Rc, sync::Arc};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use mimalloc::MiMalloc;
use serde::{Deserialize, Serialize};
use slotmap::{DefaultKey, Key, KeyData};
use vm_library::{
    arena::ARENA,
    compile_store,
    env::{Context, Inner},
    managed::{
        imports,
        value::{Union, Value},
    },
};
use wasmer::{imports, wat2wasm, Instance, Module};
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
fn deser(json: &str) -> Init {
    serde_json::from_str(json).unwrap()
}
fn deser2(json: &str) -> Init2 {
    serde_json::from_str(json).unwrap()
}
fn ser2(res: &Init2) -> String {
    serde_json::ser::to_string(res).unwrap()
}

fn benchmark2(num: i64, json: &str) -> (Module, String, i64) {
    let x = deser2(json);

    let module =
        unsafe { Module::deserialize(&compile_store::new_headless(), &x.module_).unwrap() };
    let mut env = Context {
        inner: Rc::new(RefCell::new(Inner {
            instance: None,
            pusher: None,
            gas_limit: 10000,
            call_unit: None,
            call: None,
        })),
    };
    let store = module.store();

    let imports = imports::make_imports(&env, store);
    let instance = Box::from(Instance::new(&module, &imports).unwrap());
    let new = NonNull::from(instance.as_ref());
    let pusher = Box::from(
        instance
            .exports
            .get_native_function::<i64, ()>("push")
            .unwrap(),
    );
    let call_unit = Box::from(
        instance
            .exports
            .get_native_function::<i64, ()>("call_unit")
            .unwrap(),
    );
    let call = Box::from(
        instance
            .exports
            .get_native_function::<i64, i64>("call")
            .unwrap(),
    );
    env.set_instance(Some(new));
    env.set_pusher(Some(NonNull::from(pusher.as_ref())));
    env.set_call_unit(Some(NonNull::from(call_unit.as_ref())));
    env.set_call(Some(NonNull::from(call.as_ref())));

    env.set_gas_left(100000);

    let arena = unsafe { &mut ARENA };
    let arg = arena.insert(Value::Int(5.into()));
    let arg = Value::Union(Union::Right(arg));
    let arg = arena.insert(arg);
    let arg = Value::Union(Union::Left(arg));
    let fst = arena.insert(arg);
    let snd = arena.insert(Value::Int(0.into()));
    let arg = Value::Pair { fst, snd };
    let arg = arena.insert(arg).data().as_ffi();

    let caller = instance
        .exports
        .get_native_function::<i64, i64>("main")
        .unwrap();
    let result = caller.call(arg as i64).expect("error");
    let key = DefaultKey::from(KeyData::from_ffi(result as u64));
    let value = arena.get(key);
    match value {
        Some(Value::Pair { fst, snd }) => {
            assert_eq!(arena.get(*snd).unwrap(), &Value::Int(5.into()))
        }
        _ => todo!(),
    };
    let serialized = ser2(&x);
    unsafe { ARENA.clear() };
    (module, serialized, result)
}
//RUSTFLAGS='-C target-cpu=native' cargo bench
#[derive(Deserialize, Serialize)]
struct T<'a> {
    #[serde(borrow)]
    mod_: Cow<'a, str>,
    arg: Value,
    initial_storage: Value,
}
#[derive(Deserialize, Serialize)]
struct Init {
    module_: String,
    constants: Vec<(i32, Value)>,
}
#[derive(Deserialize, Serialize)]
struct Init2 {
    module_: Vec<u8>,
    constants: Vec<(i32, Value)>,
}
fn compile(num: i64, json: &[u8]) -> Module {
    unsafe { Module::deserialize(&compile_store::new_compile_store(), json).unwrap() }
}
fn criterion_benchmark(c: &mut Criterion) {
    let json = r#" {
        "module_": "\n(module\n  (import \"env\" \"dup_host\" (func $dup_host (param i64 ) (result)))\n(import \"env\" \"pair\" (func $pair (param i64 i64) (result i64)))\n(import \"env\" \"unpair\" (func $unpair (param i64)))\n(import \"env\" \"z_add\" (func $z_add (param i64 i64) (result i64)))\n(import \"env\" \"z_sub\" (func $z_sub (param i64 i64) (result i64)))\n(import \"env\" \"compare\" (func $compare (param i64 i64) (result i64)))\n(import \"env\" \"car\" (func $car (param i64) (result i64)))\n(import \"env\" \"cdr\" (func $cdr (param i64) (result i64)))\n(import \"env\" \"some\" (func $some (param i64) (result i64)))\n(import \"env\" \"nil\" (func $nil (result i64)))\n(import \"env\" \"none\" (func $none (result i64)))\n(import \"env\" \"zero\" (func $zero (result i64)))\n(import \"env\" \"empty_set\" (func $empty_set (result i64)))\n(import \"env\" \"sender\" (func $sender (result i64)))\n(import \"env\" \"map_get\" (func $map_get (param i64 i64) (result i64)))\n(import \"env\" \"mem\" (func $mem (param i64 i64) (result i64)))\n(import \"env\" \"update\" (func $update (param i64 i64 i64) (result i64)))\n(import \"env\" \"iter\" (func $iter (param i64 i32) (result i64)))\n(import \"env\" \"map\" (func $map (param i64 i32) (result i64)))\n(import \"env\" \"if_left\" (func $if_left (param i64) (result i32)))\n(import \"env\" \"if_none\" (func $if_none (param i64) (result i32)))\n(import \"env\" \"if_cons\" (func $if_cons (param i64) (result i32)))\n(import \"env\" \"isnat\" (func $isnat (param i64) (result i64)))\n(import \"env\" \"not\" (func $not (param i64) (result i64)))\n(import \"env\" \"or\" (func $or (param i64 i64) (result i64)))\n(import \"env\" \"and\" (func $and (param i64 i64) (result i64)))\n(import \"env\" \"deref_bool\" (func $deref_bool (param i64) (result i32)))\n(import \"env\" \"neq\" (func $neq (param i64) (result i64)))\n(import \"env\" \"failwith\" (func $failwith (param i64)))\n(import \"env\" \"get_n\" (func $get_n (param i32 i64) (result i64)))\n(import \"env\" \"exec\" (func $exec (param i64 i64) (result i64)))\n(import \"env\" \"apply\" (func $apply (param i64 i64) (result i64)))\n(import \"env\" \"const\" (func $const (param i32) (result i64)))\n(import \"env\" \"get_some\" (func $get_some (param i64) (result i64)))\n(import \"env\" \"abs\" (func $abs (param i64) (result i64)))\n(import \"env\" \"eq\" (func $eq (param i64) (result i64)))\n(import \"env\" \"gt\" (func $gt (param i64) (result i64)))\n(import \"env\" \"lt\" (func $lt (param i64) (result i64)))\n(import \"env\" \"closure\" (func $closure (param i32) (result i64)))\n(import \"env\" \"left\" (func $left (param i64) (result i64)))\n(import \"env\" \"right\" (func $right (param i64) (result i64)))\n(import \"env\" \"cons\" (func $cons (param i64 i64) (result i64)))\n(import \"env\" \"transfer_tokens\" (func $transfer_tokens (param i64 i64 i64) (result i64)))\n(import \"env\" \"address\" (func $address (param i64) (result i64)))\n(import \"env\" \"self\" (func $self (result i64)))\n(import \"env\" \"get_and_update\" (func $get_and_update (param i64 i64 i64)))\n(import \"env\" \"read_ticket\" (func $read_ticket (param i64)))\n(import \"env\" \"ticket\" (func $ticket (param i64 i64) (result i64)))\n(import \"env\" \"join_tickets\" (func $join_tickets (param i64) (result i64)))\n(import \"env\" \"split_ticket\" (func $split_ticket (param i64 i64) (result i64)))\n(import \"env\" \"amount\" (func $amount (result i64)))\n\n  (global $mode i32 (i32.const 0))\n\n  (memory 1)\n  (global $sp (mut i32) (i32.const 4000)) ;; stack pointer\n  (global $sh_sp (mut i32) (i32.const 1000)) ;;shadow_stack stack pointer\n\n  (global $__stack_base i32 (i32.const 32768))\n\n  (func $dip (param $n i32) (result)\n    (local $stop i32)\n    (local $sp' i32)\n    (local $sh_sp' i32)\n    (local.set $stop (i32.const 0))\n    (local.set $sp'  (global.get $sp))\n    (local.tee $sh_sp' (i32.sub (global.get $sh_sp) (local.get $n)))\n    global.set $sh_sp\n    (loop $l\n      (i32.mul (i32.const 8) (i32.add (global.get $__stack_base) (i32.add (local.get $sh_sp') (local.get $stop))))\n      (i64.load (i32.mul (i32.const 8) (i32.add (local.get $sp') (local.get $stop))))\n      i64.store\n      (local.tee $stop (i32.add (local.get $stop) (i32.const 1)))\n      (local.get $n)\n      i32.ne\n      br_if $l)\n\n    (global.set $sp\n    (i32.add\n      (local.get $sp') (local.get $n))))\n\n  (func $undip (param $n i32) (result)\n    (local $stop i32)\n    (local $sp' i32)\n    (local $sh_sp' i32)\n    (local.tee $sp'  (i32.sub (global.get $sp) (local.get $n)))\n    global.set $sp\n    (local.set $sh_sp' (global.get $sh_sp))\n    (local.set $stop (i32.const 0))\n    (loop $l\n      (i32.mul (i32.const 8) (i32.add (local.get $sp') (local.get $stop)))\n      (i64.load\n        (i32.add\n          (global.get $__stack_base)\n          (i32.mul (i32.const 8) (i32.add (local.get $sh_sp') (local.get $stop)))))\n      (i64.store)\n      (local.tee $stop (i32.add (local.get $stop) (i32.const 1)))\n      (local.get $n)\n      i32.ne\n      br_if $l)\n    (global.set $sh_sp (i32.add (local.get $sh_sp') (local.get $n))))\n\n  (func $dup (param $n i32) (result)\n    (i64.load (i32.mul (i32.const 8) (i32.add (global.get $sp) (local.get $n))))\n    (call $dup_host))\n\n  (func $swap (param) (result)\n    (local $v1 i64)\n    (local $v2 i64)\n    (local.set $v1 (call $pop))\n    (local.set $v2 (call $pop))\n    (call $push (local.get $v1))\n    (call $push (local.get $v2)))\n\n  (func $dug (param $n i32) (result)\n    (local $idx i32)\n    (local $loop_idx i32)\n    (local $sp' i32)\n    (local $top i64)\n    (local.set $sp' (i32.add (global.get $sp) (local.get $n)))\n    (i32.mul (i32.const 8) (local.tee $idx (global.get $sp)))\n    (local.tee $loop_idx)\n    i64.load\n    local.set $top\n    (loop $loop\n      (i32.mul (i32.const 8) (local.get $idx))\n      (i32.mul (i32.const 8) (i32.add (local.get $loop_idx) (i32.const 1)))\n      local.tee $loop_idx\n      i64.load\n      i64.store\n      (local.set $idx (i32.add (local.get $idx) (i32.const 1)))\n      (local.get $idx)\n      (local.get $sp')\n      i32.lt_u\n      br_if $loop)\n\n    (i64.store (i32.mul (i32.const 8) (local.get $sp')) (local.get $top)))\n\n  (func $dig (param $n i32) (result)\n    (local $idx i32)\n    (local $loop_idx i32)\n    (local $sp' i32)\n    (local $digged i64)\n    (local.set $sp' (global.get $sp))\n    (i32.mul (i32.const 8) (local.tee $idx (i32.add (local.get $sp') (local.get $n))))\n    (local.tee $loop_idx)\n    (i64.load)\n    local.set $digged\n    (loop $loop\n      (i32.mul (i32.const 8) (local.get $idx))\n      (i32.sub (local.get $loop_idx) (i32.const 1))\n      local.tee $loop_idx\n      i32.const 8\n      i32.mul\n      i64.load\n      i64.store\n      (local.set $idx (i32.sub (local.get $idx) (i32.const 1)))\n      (local.get $sp')\n      (local.get $loop_idx)\n      i32.lt_u\n      br_if $loop)\n    (i64.store (i32.mul (i32.const 8) (global.get $sp)) (local.get $digged)))\n\n  (func $pop (result i64)\n    (local $spp i32)\n    (i32.mul (i32.const 8) (local.tee $spp (global.get $sp)))\n    i64.load\n    (global.set $sp (i32.add (local.get $spp) (i32.const 1))))  ;;set stackptr\n\n  (func $push (param $value i64) (result)\n    (local $spp i32)\n    (i32.mul (i32.const 8) (local.tee $spp (i32.sub (global.get $sp) (i32.const 1)) ))\n    (i64.store (local.get $value))\n    (global.set $sp (local.get $spp)))  ;;set stackptr\n\n  (func $drop (param $n i32) (result)\n    (global.set $sp (i32.add (global.get $sp) (local.get $n))))  ;;set stackptr\n\n  (table $closures funcref (elem ))\n\n\n  (func $main (param $v1 i64) (result i64)\n    (local $1 i64)\n    (call $push (local.get $v1))\n    (call $unpair (call $pop)) ;; implicit return\n(call $push (call $zero)) (; 0 ;)\n(call $push (call $amount))\n(call $push (call $compare (call $pop) (call $pop)))\n(call $push (call $neq (call $pop)))\n(call $deref_bool (call $pop)) (if (then (call $push (call $const (i32.const 0))) (; \"DontSendTez\" ;)\n(call $failwith (call $pop)) unreachable) (else ))\n(call $if_left (call $pop)) (if (then (call $if_left (call $pop)) (if (then (call $if_left (call $pop)) (if (then (call $dup (i32.const 2))\n(call $push (call $car (call $pop)))\n(call $push (call $car (call $pop)))\n(call $dup (i32.const 2))\n(call $push (call $car (call $pop)))\n(call $push (call $sender))\n(call $push (call $pair (call $pop) (call $pop)))\n(call $push (call $zero)) (; 0 ;)\n(call $dup (i32.const 4))\n(call $push (call $cdr (call $pop)))\n(call $push (call $compare (call $pop) (call $pop)))\n(call $push (call $gt (call $pop)))\n(call $push (call $zero)) (; 0 ;)\n(call $dup (i32.const 4))\n(call $dup (i32.const 4))\n(call $push (call $map_get (call $pop) (call $pop)))\n(call $if_none (call $pop)) (if (then (call $push (call $zero)) (; 0 ;)) (else ))\n(call $push (call $compare (call $pop) (call $pop)))\n(call $push (call $gt (call $pop)))\n(call $push (call $and (call $pop) (call $pop)))\n(call $deref_bool (call $pop)) (if (then (call $push (call $const (i32.const 1))) (; \"UnsafeAllowanceChange\" ;)\n(call $failwith (call $pop)) unreachable) (else ))\n(call $dup (i32.const 4))\n(call $push (call $cdr (call $pop)))\n(call $dig (i32.const 4))\n(call $push (call $car (call $pop)))\n(call $push (call $cdr (call $pop)))\n(call $dig (i32.const 4))\n(call $push (call $cdr (call $pop)))\n(call $dig (i32.const 4))\n(call $push (call $zero)) (; 0 ;)\n(call $dup (i32.const 3))\n(call $push (call $compare (call $pop) (call $pop)))\n(call $push (call $eq (call $pop)))\n(call $deref_bool (call $pop)) (if (then (call $swap)\n(call $drop (i32.const 1))\n(call $push (call $none))) (else (call $swap)\n(call $push (call $some (call $pop)))))\n(call $dig (i32.const 4))\n(call $push (call $update (call $pop) (call $pop) (call $pop)))\n(call $push (call $pair (call $pop) (call $pop)))\n(call $push (call $pair (call $pop) (call $pop)))\n(call $push (call $nil))) (else (call $dup (i32.const 2))\n(call $push (call $nil))\n(call $dup (i32.const 3))\n(call $push (call $cdr (call $pop)))\n(call $push (call $zero)) (; 0 ;)\n(call $dig (i32.const 5))\n(call $push (call $car (call $pop)))\n(call $push (call $car (call $pop)))\n(call $dig (i32.const 5))\n(call $push (call $car (call $pop)))\n(call $push (call $map_get (call $pop) (call $pop)))\n(call $if_none (call $pop)) (if (then (call $push (call $zero)) (; 0 ;)) (else ))\n(call $push (call $transfer_tokens (call $pop) (call $pop) (call $pop)))\n(call $push (call $cons (call $pop) (call $pop)))))) (else (call $if_left (call $pop)) (if (then (call $dup (i32.const 2))\n(call $push (call $nil))\n(call $dup (i32.const 3))\n(call $push (call $cdr (call $pop)))\n(call $push (call $zero)) (; 0 ;)\n(call $dig (i32.const 5))\n(call $push (call $car (call $pop)))\n(call $push (call $cdr (call $pop)))\n(call $dig (i32.const 5))\n(call $push (call $car (call $pop)))\n(call $push (call $map_get (call $pop) (call $pop)))\n(call $if_none (call $pop)) (if (then (call $push (call $zero)) (; 0 ;)) (else ))\n(call $push (call $transfer_tokens (call $pop) (call $pop) (call $pop)))) (else (call $dup (i32.const 2))\n(call $push (call $nil))\n(call $dig (i32.const 2))\n(call $push (call $cdr (call $pop)))\n(call $push (call $zero)) (; 0 ;)\n(call $dig (i32.const 4))\n(call $push (call $cdr (call $pop)))\n(call $push (call $transfer_tokens (call $pop) (call $pop) (call $pop)))))\n(call $push (call $cons (call $pop) (call $pop)))))) (else (call $dup (i32.const 2))\n(call $push (call $car (call $pop)))\n(call $push (call $car (call $pop)))\n(call $dup (i32.const 3))\n(call $push (call $car (call $pop)))\n(call $push (call $cdr (call $pop)))\n(call $dup (i32.const 3))\n(call $push (call $car (call $pop)))\n(call $push (call $sender))\n(call $push (call $compare (call $pop) (call $pop)))\n(call $push (call $eq (call $pop)))\n(call $deref_bool (call $pop)) (if (then (call $swap)) (else (call $push (call $sender))\n(call $dup (i32.const 4))\n(call $push (call $car (call $pop)))\n(call $push (call $pair (call $pop) (call $pop)))\n(call $dup (i32.const 4))\n(call $push (call $get_n (i32.const 4) (call $pop)))\n(call $dup (i32.const 4))\n(call $dup (i32.const 3))\n(call $push (call $map_get (call $pop) (call $pop)))\n(call $if_none (call $pop)) (if (then (call $push (call $zero)) (; 0 ;)) (else ))\n(call $push (call $z_sub (call $pop) (call $pop)))\n(call $push (call $isnat (call $pop)))\n(call $if_none (call $pop)) (if (then (call $push (call $const (i32.const 2))) (; \"NotEnoughAllowance\" ;)\n(call $failwith (call $pop)) unreachable) (else ))\n(call $dig (i32.const 3))\n(call $push (call $zero)) (; 0 ;)\n(call $dup (i32.const 3))\n(call $push (call $compare (call $pop) (call $pop)))\n(call $push (call $eq (call $pop)))\n(call $deref_bool (call $pop)) (if (then (call $swap)\n(call $drop (i32.const 1))\n(call $push (call $none))) (else (call $swap)\n(call $push (call $some (call $pop)))))\n(call $dig (i32.const 2))\n(call $push (call $update (call $pop) (call $pop) (call $pop)))))\n(call $dup (i32.const 3))\n(call $push (call $get_n (i32.const 4) (call $pop)))\n(call $dup (i32.const 3))\n(call $dup (i32.const 5))\n(call $push (call $car (call $pop)))\n(call $push (call $map_get (call $pop) (call $pop)))\n(call $if_none (call $pop)) (if (then (call $push (call $zero)) (; 0 ;)) (else ))\n(call $push (call $z_sub (call $pop) (call $pop)))\n(call $push (call $isnat (call $pop)))\n(call $if_none (call $pop)) (if (then (call $push (call $const (i32.const 3))) (; \"NotEnoughBalance\" ;)\n(call $failwith (call $pop)) unreachable) (else ))\n(call $dig (i32.const 2))\n(call $push (call $zero)) (; 0 ;)\n(call $dup (i32.const 3))\n(call $push (call $compare (call $pop) (call $pop)))\n(call $push (call $eq (call $pop)))\n(call $deref_bool (call $pop)) (if (then (call $swap)\n(call $drop (i32.const 1))\n(call $push (call $none))) (else (call $swap)\n(call $push (call $some (call $pop)))))\n(call $dup (i32.const 4))\n(call $push (call $car (call $pop)))\n(call $push (call $update (call $pop) (call $pop) (call $pop)))\n(call $dup (i32.const 3))\n(call $push (call $get_n (i32.const 4) (call $pop)))\n(call $dup (i32.const 2))\n(call $dup (i32.const 5))\n(call $push (call $get_n (i32.const 3) (call $pop)))\n(call $push (call $map_get (call $pop) (call $pop)))\n(call $if_none (call $pop)) (if (then (call $push (call $zero)) (; 0 ;)) (else ))\n(call $push (call $z_add (call $pop) (call $pop)))\n(call $dup (i32.const 5))\n(call $push (call $cdr (call $pop)))\n(call $dig (i32.const 2))\n(call $push (call $zero)) (; 0 ;)\n(call $dup (i32.const 4))\n(call $push (call $compare (call $pop) (call $pop)))\n(call $push (call $eq (call $pop)))\n(call $deref_bool (call $pop)) (if (then (call $dig (i32.const 2))\n(call $drop (i32.const 1))\n(call $push (call $none))) (else (call $dig (i32.const 2))\n(call $push (call $some (call $pop)))))\n(call $dig (i32.const 4))\n(call $push (call $get_n (i32.const 3) (call $pop)))\n(call $push (call $update (call $pop) (call $pop) (call $pop)))\n(call $dig (i32.const 3))\n(call $drop (i32.const 1))\n(call $dig (i32.const 2))\n(call $push (call $pair (call $pop) (call $pop)))\n(call $push (call $pair (call $pop) (call $pop)))\n(call $push (call $nil))))\n(call $push (call $pair (call $pop) (call $pop)))\n    (call $pop))\n\n  (export \"push\" (func $push))\n  (export \"pop\" (func $push))\n  (export \"main\" (func $main))\n  (export \"closures\" (table $closures))\n  )\n",
        "constants": [
          [ 3, [ "String", "NotEnoughBalance" ] ],
          [ 2, [ "String", "NotEnoughAllowance" ] ],
          [ 1, [ "String", "UnsafeAllowanceChange" ] ],
          [ 0, [ "String", "DontSendTez" ] ]
        ]
      }
    "#;
    let x = deser(json);
    let module = Module::new(
        &compile_store::new_compile_store(),
        wat2wasm(x.module_.as_bytes()).unwrap(),
    )
    .unwrap()
    .serialize()
    .unwrap();
    let json2 = serde_json::to_string(&Init2 {
        module_: module,
        constants: x.constants,
    })
    .unwrap();
    let input = (json, 100);
    let input2 = (json2, 100);

    // c.bench_with_input(
    //     BenchmarkId::new("test", "simple module"),
    //     &input,
    //     |b, (json, num)| b.iter(|| benchmark(*num, json)),
    // );
    c.bench_with_input(
        BenchmarkId::new("test", "simple module"),
        &input2,
        |b, (json, num)| b.iter(|| benchmark2(*num, json)),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
