[@@@warning "-40-4"]

open Tezos_micheline.Micheline
open Michelson_primitives

let gen_symbol_count = ref 0
let gen_symbol name =
  incr gen_symbol_count;
  Printf.sprintf "%s.%d" name !gen_symbol_count

let constant_count = ref 0
let constants = ref []

let lambdas = ref []

let compile_constant value =
  let id = !constant_count in
  constants := (id, value) :: !constants;
  incr constant_count;
  Printf.sprintf "(call $push (call $const (i32.const %d)))" id

let rec compile_instruction instruction =
  match instruction with
  | Prim (_, I_UNPAIR, _, _) ->
    "(call $push (call $cdr (local.tee $1 (call $pop)))) (call $push (call $car (local.get $1)))"

  | Prim (_, I_PAIR, _, _) ->
    "(call $push (call $pair (call $pop) (call $pop)))"

  | Prim (_, I_ADD, _, _) ->
    "(call $push (call $add (call $pop) (call $pop)))"

  | Prim (_, I_AMOUNT, _, _) ->
    "(call $push (call $amount))"

  | Prim (_, I_AND, _, _) ->
    "(call $push (call $and (call $pop) (call $pop)))"

  | Prim (_, I_BALANCE, _, _) ->
    "(call $push (call $balance))"

  | Prim (_, I_CAR, _, _) ->
    "(call $push (call $car (call $pop)))"

  | Prim (_, I_CDR, _, _) ->
    "(call $push (call $cdr (call $pop)))"

  | Prim (_, I_COMPARE, _, _) ->
    "(call $push (call $compare (call $pop) (call $pop)))"

  | Prim (_, I_CONS, _, _) ->
    "(call $push (call $cons (call $pop) (call $pop)))"

  | Prim (_, I_EDIV, _, _) ->
    "(call $push (call $ediv (call $pop) (call $pop)))"

  | Prim (_, I_EMPTY_SET, _, _) ->
    "(call $push (call $empty_set))"

  | Prim (_, I_EMPTY_MAP, _, _) ->
    "(call $push (call $empty_map))"

  | Prim (_, I_EQ, _, _) ->
    "(call $push (call $eq (call $pop)))"

  | Prim (_, I_EXEC, _, _) ->
    "(call $push (call $exec (call $pop) (call $pop)))"

  | Prim (_, I_APPLY, _, _) ->
    "(call $push (call $apply (call $pop) (call $pop)))"

  | Prim (_, I_FAILWITH, _, _) ->
    "(call $failwith (call $pop))"

  | Prim (_, I_GE, _, _) ->
    "(call $push (call $ge))"

  | Prim (_, I_GET, [], _) ->
    "(call $push (call $map_get (call $pop) (call $pop)))"

  | Prim (_, I_GET, [ Int (_, n) ], _) ->
    let n = Z.to_int32 n in
    Printf.sprintf
      "(call $push (call $get_n (i32.const %ld) (call $pop)))"
      n

  | Prim (_, I_IF, [ Seq (_, branch_if); Seq (_, branch_else) ], _) ->
    let branch_if =
      branch_if
      |> List.map compile_instruction
      |> String.concat "\n"
    in
    let branch_else =
      branch_else
      |> List.map compile_instruction
      |> String.concat "\n"
    in
    Printf.sprintf
      "(call $deref_bool (call $pop)) (if (then %s) (else %s))"
      branch_if
      branch_else

  | Prim (_, I_IF_CONS, [ Seq (_, branch_if_cons); Seq (_, branch_if_nil) ], _) ->
    let branch_if_cons =
      branch_if_cons
      |> List.map compile_instruction
      |> String.concat "\n"
    in
    let branch_if_nil =
      branch_if_nil
      |> List.map compile_instruction
      |> String.concat "\n"
    in
    let list_unpack =
      "(call $push (call $tail (local.get $1))) (call $push (call $head (local.get $1)))"
    in
    Printf.sprintf
      "(call $is_cons (local.tee $1 (call $pop))) (if (then %s %s) (else %s))"
        list_unpack
        branch_if_cons
        branch_if_nil

  | Prim (_, I_IF_LEFT, [ Seq (_, branch_if_left); Seq (_, branch_if_right) ], _) ->
    let branch_if_left =
      branch_if_left
      |> List.map compile_instruction
      |> String.concat "\n"
    in
    let branch_if_right =
      branch_if_right
      |> List.map compile_instruction
      |> String.concat "\n"
    in
    let if_body =
      Printf.sprintf
        "(if (then (call $push (call $get_left (local.get $1))) %s) (else (call $push (call $get_right (local.get $1))) %s))"
        branch_if_left
        branch_if_right
    in
    Printf.sprintf
      "(call $is_left (local.tee $1 (call $pop))) %s"
      if_body

  | Prim (_, I_IF_NONE, [ Seq (_, branch_if_none); Seq (_, branch_if_some) ], _) ->
    let branch_if_none =
      branch_if_none
      |> List.map compile_instruction
      |> String.concat "\n"
    in
    let branch_if_some =
      branch_if_some
      |> List.map compile_instruction
      |> String.concat "\n"
    in
    Printf.sprintf
      "(call $is_none (local.tee $1 (call $pop))) (if (then %s) (else (call $push (call $get_some (local.get $1))) %s))"
      branch_if_none
      branch_if_some

  | Prim (_, I_LE, _, _) ->
    "(call $push (call $le (call $pop)))"

  | Prim (_, I_LEFT, _, _) ->
    "(call $push (call $left (call $pop)))"

  | Prim (_, I_LT, _, _) ->
    "(call $push (call $lt (call $pop)))"

  | Prim (_, I_MEM, _, _) ->
    "(call $push (call $mem (call $pop) (call $pop)))"

  | Prim (_, I_MUL, _, _) ->
    "(call $push (call $mul (call $pop) (call $pop)))"

  | Prim (_, I_NEG, _, _) ->
    "(call $push (call $neg (call $pop)))"

  | Prim (_, I_NEQ, _, _) ->
    "(call $push (call $neq (call $pop)))"

  | Prim (_, I_NIL, _, _) ->
    "(call $push (call $nil))"

  | Prim (_, I_NONE, _, _) ->
    "(call $push (call $none))"

  | Prim (_, I_NOT, _, _) ->
    "(call $push (call $not (call $pop)))"

  | Prim (_, I_OR, _, _) ->
    "(call $push (call $or (call $pop) (call $pop)))"

  | Prim (_, I_RIGHT, _, _) ->
    "(call $push (call $right (call $pop)))"

  | Prim (_, I_SIZE, _, _) ->
    "(call $push (call $size (call $pop)))"

  | Prim (_, I_SOME, _, _) ->
    "(call $push (call $some (call $pop)))"

  | Prim (_, I_SOURCE, _, _) ->
    "(call $push (call $source))"

  | Prim (_, I_SUB, _, _) ->
    "(call $push (call $sub (call $pop) (call $pop)))"

  | Prim (_, I_SWAP, _, _) ->
    "(call $swap)"

  | Prim (_, I_UNIT, _, _) ->
    "(call $push (call $unit))"

  | Prim (_, I_UPDATE, _, _) ->
    "(call $push (call $update (call $pop) (call $pop) (call $pop)))"

  | Prim (_, I_XOR, _, _) ->
    "(call $push (call $xor (call $pop) (call $pop)))"

  | Prim (_, I_ISNAT, _, _) ->
    "(call $push (call $isnat (call $pop)))"

  | Prim (_, I_DIG, [ Int (_, n) ], _) ->
    let n = Z.to_int32 n in
    Printf.sprintf "(call $dig (i32.const %ld))" n

  | Prim (_, I_DUG, [ Int (_, n) ], _) ->
    let n = Z.to_int32 n in
    Printf.sprintf "(call $dug (i32.const %ld))" n

  | Prim (_, I_DUP, [ Int (_, n) ], _) ->
    let n = Z.to_int32 n in
    Printf.sprintf "(call $dup (i32.const %ld))" n

  | Prim (loc, I_DUP, [], annot) ->
    compile_instruction (Prim (loc, I_DUP, [ Int (loc, Z.one) ], annot))

  | Prim (_, I_DROP, [ Int (_, n) ], _) ->
    let n = Z.to_int32 n in
    Printf.sprintf "(call $drop (i32.const %ld))" n

  | Prim (loc, I_DROP, [], annot) ->
    compile_instruction (Prim (loc, I_DROP, [ Int (loc, Z.one) ], annot))

  | Prim (_, I_DIP, [ Int (_, n); Seq (_, body) ], _) ->
    let n = Z.to_int32 n in
    let body =
      body
      |> List.map compile_instruction
      |> String.concat "\n"
    in
    Printf.sprintf
      "(block %s (call $dip (i32.const %ld)) %s (call $undip (i32.const %ld)))"
      (gen_symbol "dip") n body n

  | Prim (loc, I_DIP, [], annot) ->
    compile_instruction (Prim (loc, I_DIP, [ Int (loc, Z.one) ], annot))

  | Prim (_, I_ABS, _, _) ->
    "(call $push (call $abs (call $pop)))"

  | Prim (_, I_EMPTY_BIG_MAP, _, _) ->
    "(call $push (call $empty_big_map))"

  | Prim (_, I_GET_AND_UPDATE, _, _) ->
    "(call $push (call $cdr (local.tee $1 (call $get_and_update (call $pop) (call $pop) (call $pop))))) (call $push (call $car (local.get $1)))"

  | Prim (_, I_INT, _, _) ->
    "(call $push (call $int (call $pop)))"

  | Prim (_, I_LSL, _, _) ->
    "(call $push (call $lsl (call $pop) (call $pop)))"

  | Prim (_, I_LSR, _, _) ->
    "(call $push (call $lsr (call $pop) (call $pop)))"

  | Prim (_, I_NOW, _, _) ->
    "(call $push (call $now))"

  | Prim (_, I_SELF, _, _) ->
    "(call $push (call $self))"

  | Prim (_, I_SELF_ADDRESS, _, _) ->
    "(call $push (call $self_address))"

  | Prim (_, I_SENDER, _, _) ->
    "(call $push (call $sender))"

  | Prim (_, I_ADDRESS, _, _) ->
    "(call $push (call $address (call $pop)))"

  | Prim (_, I_CONTRACT, _, _) ->
    "(call $push (call $contract (call $pop)))"

  | Prim (_, I_LOOP, [ Seq (_, body) ], _) ->
    let body =
      body
      |> List.map compile_instruction
      |> String.concat "\n"
    in
    let loop_name = gen_symbol "$loop" in
    Printf.sprintf
     "(loop %s (call $deref_bool (call $pop)) br_if %s %s)"
     loop_name loop_name body

  | Prim (_, I_LOOP_LEFT, [ Seq (_, body) ], _) ->
    let body =
      body
      |> List.map compile_instruction
      |> String.concat "\n"
    in
    let loop_name = gen_symbol "$loop_left" in
    Printf.sprintf
      "(loop %s (call $is_left (local.tee $1 (call $pop))) br_if %s (call $get_left (local.get $1)) %s)"
      loop_name loop_name body

  | Prim (_, I_ITER, [ Seq (_, body) ], _) ->
    let name = gen_symbol "$iter_lambda" in
    let lambda = compile_lambda name body in
    lambdas := lambda :: !lambdas;
    Printf.sprintf
      "(call $push (call $iter (call $pop) (ref.fun %s)))"
      name

  | Prim (_, I_MAP, [ Seq (_, body) ], _) ->
    let name = gen_symbol "$map_lambda" in
    let lambda = compile_lambda name body in
    lambdas := lambda :: !lambdas;
    Printf.sprintf
      "(call $push (call $map (call $pop) (ref.fun %s)))"
      name

  | Prim (_, I_PUSH, [ _; Int (_, z) ], _) ->
    compile_constant (Values.Int z)

  | Prim (_, I_PUSH, [ _; String (_, s) ], _) ->
    compile_constant (Values.String s)

  | Prim (_, I_PUSH, [ _; Bytes (_, b) ], _) ->
    compile_constant (Values.Bytes b)

  | Prim (_, I_LAMBDA, [ _; _; Seq (_, body) ], _) ->
    let name = gen_symbol "$lambda" in
    let lambda = compile_lambda name body in
    lambdas := lambda :: !lambdas;
    Printf.sprintf "(call $push (ref.func %s))" name

  | Prim (_, prim, _, _) ->
    failwith ("Unsupported primitive " ^ (Michelson_primitives.string_of_prim prim))

  | Seq _ | Int _ | String _ | Bytes _ -> assert false

and compile_lambda name body =
  let body =
    body
    |> List.map compile_instruction
    |> String.concat "\n"
  in
  Printf.sprintf
    "(func %s (local $1 externref) %s)"
    name
    body

let compile code =
  let parsed =
    match Parser.parse_expr code with
    | Ok expr -> root expr
    | Error (`Parsing_error _) -> failwith "Parsing error"
    | Error (`Prim_parsing_error _) -> failwith "Primitive parsing error"
  in
  match parsed with
  | Seq (
      _, [ Prim (_, K_parameter, _, _)
         ; Prim (_, K_storage, _, _)
         ; Prim (_, K_code, [ Seq (_, instructions) ], _) ] ) ->
    let body =
      instructions
      |> List.map compile_instruction
      |> String.concat "\n"
    in
    let b = Printf.sprintf "(local $1 externref) %s" body in
    let lambdas = String.concat "\n" !lambdas in
    Template.base lambdas (fun fmt b -> Format.pp_print_string fmt b) b
  | _ -> assert false
