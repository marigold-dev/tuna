
FA2 with only transfer semantics
  $ ../bin/tunacc_test.exe contract fa2_no_metadata.tz
  {
    "module_": "\n(module\n  (import \"env\" \"dup_host\" (func $dup_host (param i64 ) (result)))\n(import \"env\" \"pair\" (func $pair (param i64 i64) (result i64)))\n(import \"env\" \"unpair\" (func $unpair (param i64)))\n(import \"env\" \"z_add\" (func $z_add (param i64 i64) (result i64)))\n(import \"env\" \"z_sub\" (func $z_sub (param i64 i64) (result i64)))\n(import \"env\" \"compare\" (func $compare (param i64 i64) (result i64)))\n(import \"env\" \"car\" (func $car (param i64) (result i64)))\n(import \"env\" \"cdr\" (func $cdr (param i64) (result i64)))\n(import \"env\" \"some\" (func $some (param i64) (result i64)))\n(import \"env\" \"nil\" (func $nil (result i64)))\n(import \"env\" \"none\" (func $none (result i64)))\n(import \"env\" \"unit\" (func $unit (result i64)))\n(import \"env\" \"zero\" (func $zero (result i64)))\n(import \"env\" \"empty_map\" (func $empty_map (result i64)))\n(import \"env\" \"empty_set\" (func $empty_set (result i64)))\n(import \"env\" \"sender\" (func $sender (result i64)))\n(import \"env\" \"map_get\" (func $map_get (param i64 i64) (result i64)))\n(import \"env\" \"mem\" (func $mem (param i64 i64) (result i64)))\n(import \"env\" \"update\" (func $update (param i64 i64 i64) (result i64)))\n(import \"env\" \"iter\" (func $iter (param i64 i32) (result )))\n(import \"env\" \"map\" (func $map (param i64 i32) (result i64)))\n(import \"env\" \"if_left\" (func $if_left (param i64) (result i32)))\n(import \"env\" \"if_none\" (func $if_none (param i64) (result i32)))\n(import \"env\" \"if_cons\" (func $if_cons (param i64) (result i32)))\n(import \"env\" \"isnat\" (func $isnat (param i64) (result i64)))\n(import \"env\" \"not\" (func $not (param i64) (result i64)))\n(import \"env\" \"or\" (func $or (param i64 i64) (result i64)))\n(import \"env\" \"and\" (func $and (param i64 i64) (result i64)))\n(import \"env\" \"deref_bool\" (func $deref_bool (param i64) (result i32)))\n(import \"env\" \"neq\" (func $neq (param i64) (result i64)))\n(import \"env\" \"failwith\" (func $failwith (param i64)))\n(import \"env\" \"get_n\" (func $get_n (param i32 i64) (result i64)))\n(import \"env\" \"exec\" (func $exec (param i64 i64) (result i64)))\n(import \"env\" \"apply\" (func $apply (param i64 i64) (result i64)))\n(import \"env\" \"const\" (func $const (param i32) (result i64)))\n(import \"env\" \"abs\" (func $abs (param i64) (result i64)))\n(import \"env\" \"eq\" (func $eq (param i64) (result i64)))\n(import \"env\" \"gt\" (func $gt (param i64) (result i64)))\n(import \"env\" \"lt\" (func $lt (param i64) (result i64)))\n(import \"env\" \"closure\" (func $closure (param i32) (result i64)))\n(import \"env\" \"left\" (func $left (param i64) (result i64)))\n(import \"env\" \"right\" (func $right (param i64) (result i64)))\n(import \"env\" \"cons\" (func $cons (param i64 i64) (result i64)))\n(import \"env\" \"transfer_tokens\" (func $transfer_tokens (param i64 i64 i64) (result i64)))\n(import \"env\" \"address\" (func $address (param i64) (result i64)))\n(import \"env\" \"self\" (func $self (result i64)))\n(import \"env\" \"get_and_update\" (func $get_and_update (param i64 i64 i64)))\n(import \"env\" \"read_ticket\" (func $read_ticket (param i64)))\n(import \"env\" \"ticket\" (func $ticket (param i64 i64) (result i64)))\n(import \"env\" \"join_tickets\" (func $join_tickets (param i64) (result i64)))\n(import \"env\" \"split_ticket\" (func $split_ticket (param i64 i64) (result i64)))\n(import \"env\" \"amount\" (func $amount (result i64)))\n\n  (global $mode i32 (i32.const 0))\n\n  (memory 4)\n  (global $sp (mut i32) (i32.const 4000)) ;; stack pointer\n  (global $sh_sp (mut i32) (i32.const 1000)) ;;shadow_stack stack pointer\n\n  (global $__stack_base i32 (i32.const 32768))\n\n  (type $callback_t (func (param i64) (result i64)))\n  (func $call_callback (param $arg1 i64) (param $idx i32) (result i64)\n    (call_indirect (type $callback_t) (local.get $arg1) (local.get $idx)))\n\n  (type $callback_t_unit (func (param i64) (result)))\n  (func $call_callback_unit (param $arg1 i64) (param $idx i32) (result )\n    (call_indirect (type $callback_t_unit)\n      (local.get $arg1)\n      (local.get $idx)))\n\n  (func $dip (param $n i32) (result)\n    (local $stop i32)\n    (local $sp' i32)\n    (local $sh_sp' i32)\n    (local.set $stop (i32.const 0))\n    (local.set $sp'  (global.get $sp))\n    (local.tee $sh_sp' (i32.sub (global.get $sh_sp) (local.get $n)))\n    global.set $sh_sp\n    (loop $l\n      (i32.mul (i32.const 8) (i32.add (global.get $__stack_base) (i32.add (local.get $sh_sp') (local.get $stop))))\n      (i64.load (i32.mul (i32.const 8) (i32.add (local.get $sp') (local.get $stop))))\n      i64.store\n      (local.tee $stop (i32.add (local.get $stop) (i32.const 1)))\n      (local.get $n)\n      i32.ne\n      br_if $l)\n\n    (global.set $sp\n    (i32.add\n      (local.get $sp') (local.get $n))))\n\n  (func $undip (param $n i32) (result)\n    (local $stop i32)\n    (local $sp' i32)\n    (local $sh_sp' i32)\n    (local.tee $sp'  (i32.sub (global.get $sp) (local.get $n)))\n    global.set $sp\n    (local.set $sh_sp' (global.get $sh_sp))\n    (local.set $stop (i32.const 0))\n    (loop $l\n      (i32.mul (i32.const 8) (i32.add (local.get $sp') (local.get $stop)))\n      (i64.load\n        (i32.add\n          (global.get $__stack_base)\n          (i32.mul (i32.const 8) (i32.add (local.get $sh_sp') (local.get $stop)))))\n      (i64.store)\n      (local.tee $stop (i32.add (local.get $stop) (i32.const 1)))\n      (local.get $n)\n      i32.ne\n      br_if $l)\n    (global.set $sh_sp (i32.add (local.get $sh_sp') (local.get $n))))\n\n  (func $dup (param $n i32) (result)\n    (i64.load (i32.mul (i32.const 8) (i32.add (global.get $sp) (local.get $n))))\n    (call $dup_host))\n\n  (func $swap (param) (result)\n    (local $v1 i64)\n    (local $v2 i64)\n    (local.set $v1 (call $pop))\n    (local.set $v2 (call $pop))\n    (call $push (local.get $v1))\n    (call $push (local.get $v2)))\n\n  (func $dug (param $n i32) (result)\n    (local $idx i32)\n    (local $loop_idx i32)\n    (local $sp' i32)\n    (local $top i64)\n    (local.set $sp' (i32.add (global.get $sp) (local.get $n)))\n    (local.tee $idx (global.get $sp))\n    (local.tee $loop_idx)\n    (i32.mul (i32.const 8))\n    i64.load\n    local.set $top\n    (loop $loop\n      (i32.mul (i32.const 8) (local.get $idx))\n      (i32.add (local.get $loop_idx) (i32.const 1))\n      local.tee $loop_idx\n      (i32.mul (i32.const 8))\n      i64.load\n      i64.store\n      (local.set $idx (i32.add (local.get $idx) (i32.const 1)))\n      (local.get $idx)\n      (local.get $sp')\n      i32.lt_u\n      br_if $loop)\n\n    (i64.store (i32.mul (i32.const 8) (local.get $sp')) (local.get $top)))\n\n  (func $dig (param $n i32) (result)\n    (local $idx i32) (local $t i32) (local $digged i64)\n\n    (local.set $digged\n      (i64.load\n        (i32.mul (i32.const 8)\n          (local.tee $idx (i32.add (global.get $sp) (local.get $n))))))\n\n    (loop $loop\n      (local.set $t (i32.mul (i32.const 8) (local.get $idx)))\n\n      (i64.store (local.get $t)\n        (i64.load\n          (i32.mul\n            (i32.const 8)\n            (local.tee $idx (i32.sub (local.get $idx) (i32.const 1))))))\n\n      (br_if $loop\n        (i32.lt_u (global.get $sp) (local.get $idx))))\n\n    (i64.store (i32.mul (i32.const 8) (local.get $idx)) (local.get $digged)))\n\n  (func $pop (result i64)\n    (local $spp i32)\n    (i32.mul (i32.const 8) (local.tee $spp (global.get $sp)))\n    i64.load\n    (global.set $sp (i32.add (local.get $spp) (i32.const 1))))  ;;set stackptr\n\n  (func $push (param $value i64) (result)\n    (local $spp i32)\n    (i32.mul (i32.const 8) (local.tee $spp (i32.sub (global.get $sp) (i32.const 1)) ))\n    (i64.store (local.get $value))\n    (global.set $sp (local.get $spp)))  ;;set stackptr\n\n  (func $drop (param $n i32) (result)\n    (global.set $sp (i32.add (global.get $sp) (local.get $n))))  ;;set stackptr\n\n  (table $closures funcref (elem $lambda.0 $iter_lambda.2 $iter_lambda.1))\n(func $iter_lambda.1 (param $arg i64) (result) (local $1 i64) (call $push (local.get $arg)) (call $swap)\n(call $dup (i32.const 0))\n(call $dup (i32.const 2))\n(call $push (call $car (call $pop)))\n(call $push (call $pair (call $pop) (call $pop)))\n(call $dup (i32.const 3))\n(call $swap)\n(call $push (call $exec (call $pop) (call $pop)))\n(call $swap)\n(call $push (call $pair (call $pop) (call $pop)))\n(call $dup (i32.const 1))\n(call $push (call $cdr (call $pop)))\n(call $iter (call $pop) (i32.const 1) (; $iter_lambda.2 ;) )\n(call $unpair (call $pop)) ;; implicit return\n(call $swap)\n(call $push (call $some (call $pop)))\n(call $dig (i32.const 2))\n(call $push (call $car (call $pop)))\n(call $push (call $update (call $pop) (call $pop) (call $pop))) )\n(func $iter_lambda.2 (param $arg i64) (result) (local $1 i64) (call $push (local.get $arg)) (call $swap)\n(call $unpair (call $pop)) ;; implicit return\n(call $dup (i32.const 0))\n(call $dup (i32.const 3))\n(call $push (call $car (call $pop)))\n(call $push (call $pair (call $pop) (call $pop)))\n(call $dup (i32.const 5))\n(call $swap)\n(call $push (call $exec (call $pop) (call $pop)))\n(call $dup (i32.const 3))\n(call $push (call $get_n (i32.const 4) (call $pop)))\n(call $dup (i32.const 0))\n(call $dup (i32.const 4))\n(call $push (call $car (call $pop)))\n(call $push (call $z_sub (call $pop) (call $pop)))\n(call $push (call $zero)) (; 0 ;)\n(call $dup (i32.const 1))\n(call $push (call $compare (call $pop) (call $pop)))\n(call $push (call $lt (call $pop)))\n(call $deref_bool (call $pop)) (if (then (call $push (call $const (i32.const 1))) (; \"FA2_INSUFFICIENT_BALANCE\" ;)\n(call $failwith (call $pop)) unreachable) (else ))\n(call $dig (i32.const 4))\n(call $push (call $cdr (call $pop)))\n(call $swap)\n(call $push (call $abs (call $pop)))\n(call $push (call $pair (call $pop) (call $pop)))\n(call $dup (i32.const 2))\n(call $push (call $cdr (call $pop)))\n(call $dig (i32.const 2))\n(call $dig (i32.const 3))\n(call $push (call $car (call $pop)))\n(call $push (call $z_add (call $pop) (call $pop)))\n(call $push (call $pair (call $pop) (call $pop)))\n(call $push (call $zero)) (; 0 ;)\n(call $dup (i32.const 4))\n(call $push (call $get_n (i32.const 3) (call $pop)))\n(call $push (call $compare (call $pop) (call $pop)))\n(call $push (call $neq (call $pop)))\n(call $deref_bool (call $pop)) (if (then (call $push (call $const (i32.const 2))) (; \"FA2_TOKEN_UNDEFINED\" ;)\n(call $failwith (call $pop)) unreachable) (else ))\n(call $push (call $sender))\n(call $dup (i32.const 3))\n(call $dup (i32.const 6))\n(call $push (call $car (call $pop)))\n(call $push (call $map_get (call $pop) (call $pop)))\n(call $if_none (call $pop)) (if (then (call $dup (i32.const 7))) (else ))\n(call $push (call $cdr (call $pop)))\n(call $dup (i32.const 1))\n(call $push (call $mem (call $pop) (call $pop)))\n(call $push (call $not (call $pop)))\n(call $dup (i32.const 6))\n(call $push (call $car (call $pop)))\n(call $dig (i32.const 2))\n(call $push (call $compare (call $pop) (call $pop)))\n(call $push (call $neq (call $pop)))\n(call $push (call $or (call $pop) (call $pop)))\n(call $deref_bool (call $pop)) (if (then (call $push (call $const (i32.const 3))) (; \"FA2_NOT_OPERATOR\" ;)\n(call $failwith (call $pop)) unreachable) (else ))\n(call $swap)\n(call $dug (i32.const 2))\n(call $push (call $some (call $pop)))\n(call $dig (i32.const 3))\n(call $push (call $car (call $pop)))\n(call $push (call $update (call $pop) (call $pop) (call $pop)))\n(call $push (call $pair (call $pop) (call $pop))) )\n(func $lambda.0 (param $arg i64) (result i64) (local $1 i64) (call $push (local.get $arg)) (call $unpair (call $pop)) ;; implicit return\n(call $swap)\n(call $unpair (call $pop)) ;; implicit return\n(call $push (call $map_get (call $pop) (call $pop)))\n(call $if_none (call $pop)) (if (then ) (else (call $swap)\n(call $drop (i32.const 1)))) (call $pop))\n\n  (func $main (param $v1 i64) (result i64)\n    (local $1 i64)\n    (call $push (local.get $v1))\n    (call $push (call $empty_set))\n(call $push (call $zero)) (; 0 ;)\n(call $push (call $pair (call $pop) (call $pop)))\n(call $push (call $closure (i32.const 0) (; $lambda.0 ;) ))\n(call $dup (i32.const 1))\n(call $push (call $apply (call $pop) (call $pop)))\n(call $dig (i32.const 2))\n(call $unpair (call $pop)) ;; implicit return\n(call $if_left (call $pop)) (if (then (call $if_left (call $pop)) (if (then (call $drop (i32.const 4))\n(call $push (call $const (i32.const 0))) (; \"FA2_NOT_SUPPORTED\" ;)\n(call $failwith (call $pop)) unreachable) (else (call $iter (call $pop) (i32.const 2) (; $iter_lambda.1 ;) )\n(call $swap)\n(call $dig (i32.const 2))\n(call $drop (i32.const 2))\n(call $push (call $nil))\n(call $push (call $pair (call $pop) (call $pop)))))) (else (call $drop (i32.const 4))\n(call $push (call $const (i32.const 0))) (; \"FA2_NOT_SUPPORTED\" ;)\n(call $failwith (call $pop)) unreachable))\n    (call $pop))\n\n  (export \"push\" (func $push))\n  (export \"pop\" (func $push))\n  (export \"main\" (func $main))\n  (export \"closures\" (table $closures))\n  (export \"call_callback\" (func $call_callback))\n  (export \"call_callback_unit\" (func $call_callback_unit))\n  )\n",
    "constants": [
      [ 0, [ "String", "FA2_NOT_SUPPORTED" ] ],
      [ 1, [ "String", "FA2_INSUFFICIENT_BALANCE" ] ],
      [ 2, [ "String", "FA2_TOKEN_UNDEFINED" ] ],
      [ 3, [ "String", "FA2_NOT_OPERATOR" ] ]
    ]
  }
