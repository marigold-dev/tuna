use std::ptr::NonNull;

use wasmer::{ExternRef, Instance, Module, NativeFunc};

use crate::{
    env::Context,
    managed::{imports::make_imports, value::Value},
};

// TODO: remove unwraps
pub fn call_module(m: Module, gas_limit: u64, params: Value) -> Option<Value> {
    let mut env = Box::new(Context {
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
    let main: NativeFunc<ExternRef, ExternRef> =
        instance.exports.get_native_function("main").unwrap();

    let called = main.call(ExternRef::new(params)).unwrap();
    called.downcast::<Value>().cloned()
}

#[cfg(test)]
mod test {
    use wasmer::wat2wasm;

    use crate::compile_store;

    use super::*;
    #[test]
    fn testing() {
        let expected = Value::Int(1.into());
        let module = wat2wasm(br#"
        (module
            (import "env" "pair" (func $pair (param externref externref) (result externref)))
         (import "env" "unpair" (func $unpair (param externref) (result externref externref)))
         (import "env" "z_add" (func $z_add (param externref externref) (result externref)))
         (import "env" "z_sub" (func $z_sub (param externref externref) (result externref)))
         (import "env" "compare" (func $compare (param externref externref) (result externref)))
         (import "env" "car" (func $car (param externref) (result externref)))
         (import "env" "cdr" (func $cdr (param externref) (result externref)))
         (import "env" "some" (func $some (param externref) (result externref)))
         (import "env" "nil" (func $nil (result externref)))
         (import "env" "zero" (func $zero (result externref)))
         (import "env" "empty_set" (func $empty_set (result externref)))
         (import "env" "sender" (func $sender (result externref)))
         (import "env" "map_get" (func $map_get (param externref externref) (result externref)))
         (import "env" "mem" (func $mem (param externref externref) (result externref)))
         (import "env" "update" (func $update (param externref externref externref) (result externref)))
         (import "env" "iter" (func $iter (param externref) (result externref externref i32)))
         (import "env" "is_left" (func $is_left (param externref) (result externref i32)))
         (import "env" "is_none" (func $is_none (param externref) (result externref i32)))
         (import "env" "isnat" (func $isnat (param externref) (result externref)))
         (import "env" "not" (func $not (param externref) (result externref)))
         (import "env" "or" (func $or (param externref externref) (result externref)))
         (import "env" "deref_bool" (func $deref_bool (param externref) (result i32)))
         (import "env" "neq" (func $neq (param externref) (result externref)))
         (import "env" "string" (func $string (param i32) (result externref)))
         (import "env" "failwith" (func $failwith (param externref)))
         (import "env" "get_n" (func $get_n (param i32 externref) (result externref)))
            (global $mode (i32) 0)
            (table $stack 4000 externref)
            (global $sp (mut i32) (i32.const 4000)) ;; stack pointer
            (table $shadow_stack 1000 externref)
            (global $sh_sp (mut i32) (i32.const 1000)) ;;shadow_stack stack pointer
              (func $dip (param $n i32) (result)
                  (local $stop i32)
                  (local $sp' i32)
                  (local $sh_sp' i32)
                  (local.set $stop (i32.const 0))
                  (local.set $sp'  (global.get $sp))
                  (local.tee $sh_sp' (i32.sub (global.get $sh_sp) (local.get $n)))
                  global.set $sh_sp
                  (loop $l
                    (i32.add (local.get $sh_sp') (local.get $stop))
                    (table.get $stack (i32.add (local.get $sp') (local.get $stop)))
                    (table.set $shadow_stack)
                    (local.tee $stop (i32.add (local.get $stop) (i32.const 1)))
                    (local.get $n)
                    i32.ne
                    br_if $l
                  )
                (global.set $sp 
                     (i32.add 
                       (local.get $sp') (local.get $n)
                      )
                   )
            )
              (func $undip (param $n i32) (result)
                  (local $stop i32)
                  (local $sp' i32)
                  (local $sh_sp' i32)
                  (local.tee $sp'  (i32.sub (global.get $sp) (local.get $n)))
                  global.set $sp
                  (local.set $sh_sp' (global.get $sh_sp))
                  (local.set $stop (i32.const 0))
                   (loop $l
                    (i32.add (local.get $sp') (local.get $stop))
                    (table.get $shadow_stack (i32.add (local.get $sh_sp') (local.get $stop)))
                    (table.set $stack)
                    (local.tee $stop (i32.add (local.get $stop) (i32.const 1)))
                    (local.get $n)
                    i32.ne
                    br_if $l
                  )
                (global.set $sh_sp (i32.add (global.get $sh_sp') (local.get $n)))
            )
          
            (func $dup (param $n i32) (result)
                  (table.get $stack (i32.add (global.get $sp) (local.get $n)))
                (call $push)
            )
          (func $swap (param) (result)
           (local $v1 externref)
           (local $v2 externref)
           (local.set $v1 (call $pop))
           (local.set $v2 (call $pop))
           (call $push (local.get $v1))
           (call $push (local.get $v2))
         )
          
         (func $dug (param $n i32) (result)
         (local $idx i32)
         (local $loop_idx i32)
         (local $sp' i32)
         (local $top externref)
         (local.set $sp' (i32.add (global.get $sp) (local.get $n)))
         (local.tee $idx (global.get $sp))
         (local.tee $loop_idx)
         table.get $stack
         local.set $top
         (loop $loop
          (local.get $idx)
          (i32.add (local.get $loop_idx) (i32.const 1))
          local.tee $loop_idx
          table.get $stack
          table.set $stack
         (local.set $idx (i32.add (local.get $idx) (i32.const 1)))
          (local.get $idx)
          (local.get $sp')
          i32.lt_u
          br_if $loop
         )
         (table.set $stack (local.get $sp') (local.get $top))
         )
         (func $dig (param $n i32) (result)
         (local $idx i32)
         (local $loop_idx i32)
         (local $sp' i32)
         (local $digged externref)
         (local.set $sp' (global.get $sp))
         (local.tee $idx (i32.add (local.get $sp') (local.get $n)))
         (local.tee $loop_idx)
         table.get $stack
         local.set $digged
         (loop $loop
         (local.get $idx)
         (i32.sub (local.get $loop_idx) (i32.const 1))
         local.tee $loop_idx 
         table.get $stack
         table.set $stack
         (local.set $idx (i32.sub (local.get $idx) (i32.const 1)))
         (local.get $sp')
         (local.get $loop_idx)
         i32.lt_u
         br_if $loop
         )
         (table.set $stack (global.get $sp) (local.get $digged))
         )
            (func $pop (result externref)   
                 (local $spp i32)
                 (local.tee $spp (global.get $sp))
                 table.get $stack
                 (global.set $sp (i32.add (local.get $spp) (i32.const 1)))  ;;set stackptr
            )
              (func $push (param $value externref) (result) 
                 (local $spp i32)
                 (local.tee $spp (i32.sub (global.get $sp) (i32.const 1)) )
                 (table.set $stack (local.get $value))
                 (global.set $sp (local.get $spp) )  ;;set stackptr
            )  
            (func $drop (param $n i32) (result)   
                 (global.set $sp (i32.add (global.get $sp) (local.get $n)))  ;;set stackptr
            )
            
            (func $main (param $v1 externref) (result externref)
             (call $push (local.get $v1))
              (local $1 externref) (call $push (call $cdr (local.tee $1 (call $pop)))) (call $push (call $car (local.get $1)))
         (call $is_left (local.tee $1 (call $pop))) (if (then (call $push (call $get_left (local.get $1))) (call $swap)
         (call $push (call $sub (call $pop) (call $pop)))) (else (call $push (call $get_right (local.get $1))) (call $push (call $add (call $pop) (call $pop)))))
         (call $push (call $nil))
         (call $push (call $pair (call $pop) (call $pop)))
             (call $pop)
             )
            (export "push" (func $push))
            (export "pop" (func $push))
            (export "main" (func $main))
         )
         
        "#).unwrap();
        let store = compile_store::new_compile_store();
        let module = Module::new(&store, module).unwrap();
        let result = call_module(
            module,
            100000000,
            Value::Pair(Box::new((Value::Int(5.into()), Value::Int(5.into())))),
        );
        assert_eq!(result.unwrap(), expected)
    }
}
