let import_list =
  let ref_ref__ref = "(param i64 i64) (result i64)" in
  let ref_func__ref = "(param i64 funcref) (result i64)" in
  let ref_ref_ref__ref =
    "(param i64 i64 i64) (result i64)"
  in
  let ref__ref = "(param i64) (result i64)" in
  let ref__i32 = "(param i64) (result i32)" in
  let i32__ref = "(param i32) (result i64)" in
  let i32_ref__ref = "(param i32 i64) (result i64)" in
  let ref__ = "(param i64)" in
  let const = "(result i64)" in
  let func type_ name =
    Printf.sprintf "(import \"env\" \"%s\" (func $%s %s))" name name type_
  in
  [ func ref_ref__ref "pair"
  ; func ref_ref__ref "z_add"
  ; func ref_ref__ref "z_sub"
  ; func ref_ref__ref "compare"
  ; func ref__ref "car"
  ; func ref__ref "cdr"
  ; func ref__ref "some"
  ; func const "nil"
  ; func const "zero"
  ; func const "empty_set"
  ; func const "sender"
  ; func ref_ref__ref "map_get"
  ; func ref_ref__ref "mem"
  ; func ref_ref_ref__ref "update"
  ; func ref_func__ref "iter"
  ; func ref__i32 "is_left"
  ; func ref__i32 "is_none"
  ; func ref__ref "isnat"
  ; func ref__ref "not"
  ; func ref_ref__ref "or"
  ; func ref__i32 "deref_bool"
  ; func ref__ref "neq"
  ; func i32__ref "string"
  ; func ref__ "failwith"
  ; func i32_ref__ref "get_n"
  ; func ref_ref__ref "exec"
  ; func ref_ref__ref "apply"
  ; func i32__ref "const"
  ; func ref__ref "get_some"
  ; func ref__ref "abs"
  ; func ref__ref "lt"
  ; func ref__ref "get_left"
  ; func ref__ref "get_right"
  ; func i32__ref "closure"
  ]
  |> String.concat "\n"

let base t =
  Format.asprintf
    {|
(module
  %s

  (global $mode i32 (i32.const 0))

  (memory 1)
  (global $sp (mut i32) (i32.const 4000)) ;; stack pointer
  (global $sh_sp (mut i32) (i32.const 1000)) ;;shadow_stack stack pointer

  (global $__stack_base i32 (i32.const 32768))

  (func $dip (param $n i32) (result)
    (local $stop i32)
    (local $sp' i32)
    (local $sh_sp' i32)
    (local.set $stop (i32.const 0))
    (local.set $sp'  (global.get $sp))
    (local.tee $sh_sp' (i32.sub (global.get $sh_sp) (local.get $n)))
    global.set $sh_sp
    (loop $l
      (i32.mul (i32.const 8) (i32.add (global.get $__stack_base) (i32.add (local.get $sh_sp') (local.get $stop))))
      (i64.load (i32.mul (i32.const 8) (i32.add (local.get $sp') (local.get $stop))))
      i64.store
      (local.tee $stop (i32.add (local.get $stop) (i32.const 1)))
      (local.get $n)
      i32.ne
      br_if $l)

    (global.set $sp
    (i32.add
      (local.get $sp') (local.get $n))))

  (func $undip (param $n i32) (result)
    (local $stop i32)
    (local $sp' i32)
    (local $sh_sp' i32)
    (local.tee $sp'  (i32.sub (global.get $sp) (local.get $n)))
    global.set $sp
    (local.set $sh_sp' (global.get $sh_sp))
    (local.set $stop (i32.const 0))
    (loop $l
      (i32.mul (i32.const 8) (i32.add (local.get $sp') (local.get $stop)))
      (i64.load
        (i32.add
          (global.get $__stack_base)
          (i32.mul (i32.const 8) (i32.add (local.get $sh_sp') (local.get $stop)))))
      (i64.store)
      (local.tee $stop (i32.add (local.get $stop) (i32.const 1)))
      (local.get $n)
      i32.ne
      br_if $l)
    (global.set $sh_sp (i32.add (local.get $sh_sp') (local.get $n))))

  (func $dup (param $n i32) (result)
    (i64.load (i32.mul (i32.const 8) (i32.add (global.get $sp) (local.get $n))))
    (call $push))

  (func $swap (param) (result)
    (local $v1 i64)
    (local $v2 i64)
    (local.set $v1 (call $pop))
    (local.set $v2 (call $pop))
    (call $push (local.get $v1))
    (call $push (local.get $v2)))

  (func $dug (param $n i32) (result)
    (local $idx i32)
    (local $loop_idx i32)
    (local $sp' i32)
    (local $top i64)
    (local.set $sp' (i32.add (global.get $sp) (local.get $n)))
    (i32.mul (i32.const 8) (local.tee $idx (global.get $sp)))
    (local.tee $loop_idx)
    i64.load
    local.set $top
    (loop $loop
      (i32.mul (i32.const 8) (local.get $idx))
      (i32.mul (i32.const 8) (i32.add (local.get $loop_idx) (i32.const 1)))
      local.tee $loop_idx
      i64.load
      i64.store
      (local.set $idx (i32.add (local.get $idx) (i32.const 1)))
      (local.get $idx)
      (local.get $sp')
      i32.lt_u
      br_if $loop)

    (i64.store (i32.mul (i32.const 8) (local.get $sp')) (local.get $top)))

  (func $dig (param $n i32) (result)
    (local $idx i32)
    (local $loop_idx i32)
    (local $sp' i32)
    (local $digged i64)
    (local.set $sp' (global.get $sp))
    (i32.mul (i32.const 8) (local.tee $idx (i32.add (local.get $sp') (local.get $n))))
    (local.tee $loop_idx)
    (i64.load)
    local.set $digged
    (loop $loop
      (i32.mul (i32.const 8) (local.get $idx))
      (i32.sub (local.get $loop_idx) (i32.const 1))
      local.tee $loop_idx
      i32.const 8
      i32.mul
      i64.load
      i64.store
      (local.set $idx (i32.sub (local.get $idx) (i32.const 1)))
      (local.get $sp')
      (local.get $loop_idx)
      i32.lt_u
      br_if $loop)
    (i64.store (i32.mul (i32.const 8) (global.get $sp)) (local.get $digged)))

  (func $pop (result i64)
    (local $spp i32)
    (i32.mul (i32.const 8) (local.tee $spp (global.get $sp)))
    i64.load
    (global.set $sp (i32.add (local.get $spp) (i32.const 1))))  ;;set stackptr

  (func $push (param $value i64) (result)
    (local $spp i32)
    (i32.mul (i32.const 8) (local.tee $spp (i32.sub (global.get $sp) (i32.const 1)) ))
    (i64.store (local.get $value))
    (global.set $sp (local.get $spp)))  ;;set stackptr

  (func $drop (param $n i32) (result)
    (global.set $sp (i32.add (global.get $sp) (local.get $n))))  ;;set stackptr

  %s

  (func $main (param $v1 i64) (result i64)
    (local $1 i64)
    (call $push (local.get $v1))
    %a
    (call $pop))

  (export "push" (func $push))
  (export "pop" (func $push))
  (export "main" (func $main)))
|}
    import_list t
