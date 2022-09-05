let import_list =
  let ref_ref__ref = "(param externref externref) (result externref)" in
  let ref_ref_ref__ref =
    "(param externref externref externref) (result externref)"
  in
  let ref__ref_ref = "(param externref) (result externref externref)" in
  let ref__ref = "(param externref) (result externref)" in
  let ref__ref_i32 = "(param externref) (result externref i32)" in
  let ref__ref_ref_i32 = "(param externref) (result externref externref i32)" in
  let ref__i32 = "(param externref) (result i32)" in
  let i32__ref = "(param i32) (result externref)" in
  let i32_ref__ref = "(param i32 externref) (result externref)" in
  let ref__ = "(param externref)" in
  let const = "(result externref)" in
  let func type_ name =
    Printf.sprintf "(import \"env\" \"%s\" (func $%s %s))" name name type_
  in
  [ func ref_ref__ref "pair"
  ; func ref__ref_ref "unpair"
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
  ; func ref__ref_ref_i32 "iter"
  ; func ref__ref_i32 "is_left"
  ; func ref__ref_i32 "is_none"
  ; func ref__ref "isnat"
  ; func ref__ref "not"
  ; func ref_ref__ref "or"
  ; func ref__i32 "deref_bool"
  ; func ref__ref "neq"
  ; func i32__ref "string"
  ; func ref__ "failwith"
  ; func i32_ref__ref "get_n"
  ]
  |> String.concat "\n"

let base t =
  Format.asprintf
    {|
(module
   %s
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
     %a
    (call $pop)
    )
   (export "push" (func $push))
   (export "pop" (func $push))
   (export "main" (func $main))
)
|}
    import_list t
