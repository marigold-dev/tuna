[@@@warning "-40-4"]

open Tezos_micheline.Micheline
open Michelson_primitives

let gen_symbol_count = ref 0

let gen_symbol name =
  incr gen_symbol_count;
  Printf.sprintf "%s.%d" name !gen_symbol_count

let constant_count = ref 0

let constants = ref []

let lambda_count = ref 0

let lambdas = ref []

let compile_constant value =
  let id = !constant_count in
  match value with
  | Values.Int z when Z.equal Z.zero z ->
    Printf.sprintf "(call $push (call $zero))"
  | _ -> (
    match
      List.find_map
        (fun (k, x) -> if Values.equal x value then Some k else None)
        !constants
    with
    | None ->
      constants := (id, value) :: !constants;
      incr constant_count;
      Printf.sprintf "(call $push (call $const (i32.const %d)))" id
    | Some x -> Printf.sprintf "(call $push (call $const (i32.const %d)))" x)

let rec compile_instruction instruction =
  match instruction with
  | Prim (_, I_UNPAIR, _, _) -> "(call $unpair (call $pop)) ;; implicit return"
  | Prim (_, I_PAIR, _, _) ->
    "(call $push (call $pair (call $pop) (call $pop)))"
  | Prim (_, I_ADD, _, _) ->
    "(call $push (call $z_add (call $pop) (call $pop)))"
  | Prim (_, I_AMOUNT, _, _) -> "(call $push (call $amount))"
  | Prim (_, I_AND, _, _) -> "(call $push (call $and (call $pop) (call $pop)))"
  | Prim (_, I_BALANCE, _, _) -> "(call $push (call $balance))"
  | Prim (_, I_CAR, _, _) -> "(call $push (call $car (call $pop)))"
  | Prim (_, I_CDR, _, _) -> "(call $push (call $cdr (call $pop)))"
  | Prim (_, I_COMPARE, _, _) ->
    "(call $push (call $compare (call $pop) (call $pop)))"
  | Prim (_, I_CONS, _, _) ->
    "(call $push (call $cons (call $pop) (call $pop)))"
  | Prim (_, I_EDIV, _, _) ->
    "(call $push (call $ediv (call $pop) (call $pop)))"
  | Prim (_, I_EMPTY_SET, _, _) -> "(call $push (call $empty_set))"
  | Prim (_, I_EMPTY_MAP, _, _) -> "(call $push (call $empty_map))"
  | Prim (_, I_EQ, _, _) -> "(call $push (call $eq (call $pop)))"
  | Prim (_, I_EXEC, _, _) ->
    "(call $push (call $exec (call $pop) (call $pop)))"
  | Prim (_, I_APPLY, _, _) ->
    "(call $push (call $apply (call $pop) (call $pop)))"
  | Prim (_, I_FAILWITH, _, _) -> "(call $failwith (call $pop)) unreachable"
  | Prim (_, I_GE, _, _) -> "(call $push (call $ge (call $pop)))"
  | Prim (_, I_GT, _, _) -> "(call $push (call $gt (call $pop)))"
  | Prim (_, I_GET, [], _) ->
    "(call $push (call $map_get (call $pop) (call $pop)))"
  | Prim (_, I_GET, [ Int (_, n) ], _) ->
    let n = Z.to_int32 n in
    Printf.sprintf "(call $push (call $get_n (i32.const %ld) (call $pop)))" n
  | Prim (_, I_IF, [ Seq (_, branch_if); Seq (_, branch_else) ], _) ->
    let branch_if =
      branch_if |> List.map compile_instruction |> String.concat "\n"
    in
    let branch_else =
      branch_else |> List.map compile_instruction |> String.concat "\n"
    in
    Printf.sprintf "(call $deref_bool (call $pop)) (if (then %s) (else %s))"
      branch_if branch_else
  | Prim (_, I_IF_CONS, [ Seq (_, branch_if_cons); Seq (_, branch_if_nil) ], _)
    ->
    let branch_if_cons =
      branch_if_cons |> List.map compile_instruction |> String.concat "\n"
    in
    let branch_if_nil =
      branch_if_nil |> List.map compile_instruction |> String.concat "\n"
    in
    Printf.sprintf "(call $if_cons (call $pop)) (if (then %s) (else %s))"
      branch_if_cons branch_if_nil
  | Prim (_, I_IF_LEFT, [ Seq (_, branch_if_left); Seq (_, branch_if_right) ], _)
    ->
    let branch_if_left =
      branch_if_left |> List.map compile_instruction |> String.concat "\n"
    in
    let branch_if_right =
      branch_if_right |> List.map compile_instruction |> String.concat "\n"
    in
    let if_body =
      Printf.sprintf "(if (then %s) (else %s))" branch_if_left branch_if_right
    in
    Printf.sprintf "(call $if_left (call $pop)) %s" if_body
  | Prim (_, I_IF_NONE, [ Seq (_, branch_if_none); Seq (_, branch_if_some) ], _)
    ->
    let branch_if_none =
      branch_if_none |> List.map compile_instruction |> String.concat "\n"
    in
    let branch_if_some =
      branch_if_some |> List.map compile_instruction |> String.concat "\n"
    in
    Printf.sprintf "(call $if_none (call $pop)) (if (then %s) (else %s))"
      branch_if_none branch_if_some
  | Prim (_, I_LE, _, _) -> "(call $push (call $le (call $pop)))"
  | Prim (_, I_LEFT, _, _) -> "(call $push (call $left (call $pop)))"
  | Prim (_, I_LT, _, _) -> "(call $push (call $lt (call $pop)))"
  | Prim (_, I_MEM, _, _) -> "(call $push (call $mem (call $pop) (call $pop)))"
  | Prim (_, I_MUL, _, _) -> "(call $push (call $mul (call $pop) (call $pop)))"
  | Prim (_, I_NEG, _, _) -> "(call $push (call $neg (call $pop)))"
  | Prim (_, I_NEQ, _, _) -> "(call $push (call $neq (call $pop)))"
  | Prim (_, I_NIL, _, _) -> "(call $push (call $nil))"
  | Prim (_, I_NONE, _, _) -> "(call $push (call $none))"
  | Prim (_, I_NOT, _, _) -> "(call $push (call $not (call $pop)))"
  | Prim (_, I_OR, _, _) -> "(call $push (call $or (call $pop) (call $pop)))"
  | Prim (_, I_RIGHT, _, _) -> "(call $push (call $right (call $pop)))"
  | Prim (_, I_SIZE, _, _) -> "(call $push (call $size (call $pop)))"
  | Prim (_, I_SOME, _, _) -> "(call $push (call $some (call $pop)))"
  | Prim (_, I_SOURCE, _, _) -> "(call $push (call $source))"
  | Prim (_, I_SUB, _, _) ->
    "(call $push (call $z_sub (call $pop) (call $pop)))"
  | Prim (_, I_SWAP, _, _) -> "(call $swap)"
  | Prim (_, I_UNIT, _, _) -> "(call $push (call $unit))"
  | Prim (_, I_UPDATE, _, _) ->
    "(call $push (call $update (call $pop) (call $pop) (call $pop)))"
  | Prim (_, I_XOR, _, _) -> "(call $push (call $xor (call $pop) (call $pop)))"
  | Prim (_, I_ISNAT, _, _) -> "(call $push (call $isnat (call $pop)))"
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
    let body = body |> List.map compile_instruction |> String.concat "\n" in
    Printf.sprintf
      "(block %s (call $dip (i32.const %ld)) %s (call $undip (i32.const %ld)))"
      (gen_symbol "dip") n body n
  | Prim (loc, I_DIP, [], annot) ->
    compile_instruction (Prim (loc, I_DIP, [ Int (loc, Z.one) ], annot))
  | Prim (_, I_ABS, _, _) -> "(call $push (call $abs (call $pop)))"
  | Prim (_, I_EMPTY_BIG_MAP, _, _) -> "(call $push (call $empty_big_map))"
  | Prim (_, I_GET_AND_UPDATE, _, _) ->
    "(call $get_and_update (call $pop) (call $pop) (call $pop)) ;; implicit \
     update"
  | Prim (_, I_INT, _, _) -> "(call $push (call $int (call $pop)))"
  | Prim (_, I_LSL, _, _) -> "(call $push (call $lsl (call $pop) (call $pop)))"
  | Prim (_, I_LSR, _, _) -> "(call $push (call $lsr (call $pop) (call $pop)))"
  | Prim (_, I_NOW, _, _) -> "(call $push (call $now))"
  | Prim (_, I_SELF, _, _) -> "(call $push (call $self))"
  | Prim (_, I_SELF_ADDRESS, _, _) -> "(call $push (call $self_address))"
  | Prim (_, I_SENDER, _, _) -> "(call $push (call $sender))"
  | Prim (_, I_ADDRESS, _, _) -> "(call $push (call $address (call $pop)))"
  | Prim (_, I_CONTRACT, _, _) -> "(call $push (call $contract (call $pop)))"
  | Prim (_, I_IMPLICIT_ACCOUNT, _, _) ->
    "(call $push (call $implicit_account (call $pop)))"
  | Prim (_, I_LEVEL, _, _) -> "(call $push (call $level))"
  | Prim (_, I_TRANSFER_TOKENS, _, _) ->
    (* 'ty : mutez : contract 'ty : A -> operation : A *)
    "(call $push (call $transfer_tokens (call $pop) (call $pop) (call $pop)))"
  | Prim (_, I_LOOP, [ Seq (_, body) ], _) ->
    let body = body |> List.map compile_instruction |> String.concat "\n" in
    let loop_name = gen_symbol "$loop" in
    Printf.sprintf "(loop %s (call $deref_bool (call $pop)) br_if %s %s)"
      loop_name loop_name body
  | Prim (_, I_LOOP_LEFT, [ Seq (_, body) ], _) ->
    let body = body |> List.map compile_instruction |> String.concat "\n" in
    let loop_name = gen_symbol "$loop_left" in
    Printf.sprintf "(loop %s (call $if_left (call $pop)) br_if %s %s)" loop_name
      loop_name body
  | Prim (_, I_ITER, [ Seq (_, body) ], _) ->
    let name = gen_symbol "$iter_lambda" in
    let lambda = compile_lambda name body in
    Printf.sprintf
      "(call $push (call $iter (call $pop) (i32.const %d) (; %s ;) ))" lambda
      name
  | Prim (_, I_MAP, [ Seq (_, body) ], _) ->
    let name = gen_symbol "$map_lambda" in
    let lambda = compile_lambda name body in
    Printf.sprintf
      "(call $push (call $map (call $pop) (i32.const %d) (; %s ;) ))" lambda
      name
  | Prim (_, I_PUSH, [ _; Int (_, z) ], _) ->
    Printf.sprintf "%s (; %s ;)"
      (compile_constant (Values.Int z))
      (Z.to_string z)
  | Prim (_, I_PUSH, [ _; String (_, s) ], _) ->
    Printf.sprintf "%s (; \"%s\" ;)" (compile_constant (Values.String s)) s
  | Prim (_, I_PUSH, [ _; Bytes (_, b) ], _) ->
    compile_constant (Values.Bytes b)
  | Prim (_, I_LAMBDA, [ _; _; Seq (_, body) ], _) ->
    let name = gen_symbol "$lambda" in
    let lambda = compile_lambda name body in
    Printf.sprintf "(call $push (call $closure (i32.const %d) (; %s ;) ))"
      lambda name
  | Prim (_, I_BLAKE2B, _, _) -> "(call $push (call $blake2b (call $pop)))"
  | Prim (_, I_CHECK_SIGNATURE, _, _) ->
    (* key : signature : bytes : A -> bool : A *)
    "(call $push (call $check_signature (call $pop) (call $pop) (call $pop)))"
  | Prim (_, I_HASH_KEY, _, _) ->
    (* key : A -> key_hash : A *)
    "(call $push (call $hash_key (call $pop)))"
  | Prim (_, I_KECCAK, _, _) ->
    (* bytes : A -> bytes : A *)
    "(call $push (call $keccak (call $pop)))"
  | Prim (_, I_PAIRING_CHECK, _, _) ->
    (* list ( pair bls12_381_g1 bls12_381_g2 ) : A -> bool : A *)
    "(call $push (call $pairing_check (call $pop)))"
  | Prim (_, I_SHA256, _, _) ->
    (* bytes : A -> bytes : A *)
    "(call $push (call $sha256 (call $pop)))"
  | Prim (_, I_SHA3, _, _) ->
    (* bytes : A -> bytes : A *)
    "(call $push (call $sha3 (call $pop)))"
  | Prim (_, I_SHA512, _, _) ->
    (* bytes : A -> bytes : A *)
    "(call $push (call $sha512 (call $pop)))"
  | Prim (_, I_CAST, _, _) -> (* Ignored *) ""
  | Prim (_, I_CONCAT, _, _) ->
    "(local.tee $1 (call $pop)) (call $is_list (local.get $1)) (if (then (call \
     $push (call $concat (local.get $1)))) (else (call $push (call $concat \
     (local.get $1) (call $pop)))))"
  | Prim (_, I_TICKET, _, _) ->
    (* pair ( ticket cty ) ( ticket cty ) : A -> option (ticket cty) : A *)
    "(call $push (call $ticket (call $pop) (call $pop)))"
  | Prim (_, I_SPLIT_TICKET, _, _) ->
    (* ticket cty : pair nat nat : A -> option ( pair ( ticket cty ) ( ticket cty ) ) : A *)
    "(call $push (call $split_ticket (call $pop) (call $pop)))"
  | Prim (_, I_READ_TICKET, _, _) ->
    (* ticket cty : A -> pair address cty nat : A *)
    "(call $read_ticket (call $pop)) ;; implicit return"
  | Prim (_, I_JOIN_TICKETS, _, _) ->
    (* pair ( ticket cty ) ( ticket cty ) : A -> option ( ticket cty ) : A *)
    "(call $push (call $join_tickets (call $pop)))"
  | Prim (_, prim, _, _) ->
    failwith
      ("Unsupported primitive " ^ Michelson_primitives.string_of_prim prim)
  | Seq _ | Int _ | String _ | Bytes _ -> assert false

and compile_lambda name body =
  let body = body |> List.map compile_instruction |> String.concat "\n" in
  let lambda = Printf.sprintf "(func %s (local $1 i64) %s)" name body in
  let id = !lambda_count in
  incr lambda_count;
  lambdas := (id, name, lambda) :: !lambdas;
  id

let compile code =
  let open Helpers.Result.Let_syntax in
  let* parsed =
    match Parser.parse_expr code with
    | Ok expr -> Ok (root expr)
    | (Error (`Parsing_error _) | Error (`Prim_parsing_error _)) as x -> x
  in
  match parsed with
  | Seq
      ( _
      , [ Prim (_, K_parameter, _, _)
        ; Prim (_, K_storage, _, _)
        ; Prim (_, K_code, [ Seq (_, instructions) ], _)
        ] ) ->
    let body =
      instructions |> List.map compile_instruction |> String.concat "\n"
    in
    let lambda_code =
      !lambdas |> List.map (fun (_, _, x) -> x) |> String.concat "\n"
    in
    let lambda_table =
      !lambdas
      |> List.map (fun (_, name, _) -> name)
      |> String.concat " "
      |> Printf.sprintf "(table $closures funcref (elem %s))\n"
    in
    Ok
      ( Template.base
          (lambda_table ^ lambda_code)
          (fun fmt b -> Format.pp_print_string fmt b)
          body
      , Array.of_list !constants )
  | _ -> Error `Unexpected_error

let rec compile_value parsed =
  let open Helpers.Result.Let_syntax in
  let open Values.V in
  match parsed with
  | Prim (_, D_Unit, _, _) -> Ok Unit
  | Prim (_, D_False, _, _) -> Ok (Bool 0)
  | Prim (_, D_True, _, _) -> Ok (Bool 1)
  | Prim (_, D_None, _, _) -> Ok (Option None)
  | Prim (_, D_Some, [ value ], _) ->
    let* value = compile_value value in
    Ok (Option (Some value))
  | Prim (_, D_Left, [ value ], _) ->
    let* value = compile_value value in
    Ok (Union (Left value))
  | Prim (_, D_Right, [ value ], _) ->
    let* value = compile_value value in
    Ok (Union (Right value))
  | Prim (_, D_Pair, [ fst; snd ], _) ->
    let* fst = compile_value fst in
    let* snd = compile_value snd in
    Ok (Pair (fst, snd))
  | Int (_, z) -> Ok (Values.V.Int z)
  | String (_, s) -> Ok (Values.V.String s)
  | Bytes (_, b) -> Ok (Values.V.Bytes b)
  | Seq (_, Prim (_, D_Elt, _, _) :: _) -> compile_map parsed
    (* TODO: sets have the same representation as lists, types should help disambiguate. *)
  | Seq (_, elements) ->
    let rec aux elts =
      match elts with
      | elt :: elts ->
        let* elt = compile_value elt in
        let* lst = aux elts in
        Ok (elt :: lst)
      | [] -> Ok []
    in
    let* elements = aux elements in
    Ok (Values.V.List elements)
  | _ -> Error `Unexpected_error

and compile_map parsed =
  let open Helpers.Result.Let_syntax in
  match parsed with
  | Seq (_, elements) ->
    let rec aux m elts =
      match elts with
      | Prim (_, D_Elt, [ key; value ], _) :: elts ->
        let* key = compile_value key in
        let* value = compile_value value in
        let m = Values.Map.add key value m in
        aux m elts
      | [] -> Ok m
      | _ -> Error `Unexpected_error
    in
    let* m = aux Values.Map.empty elements in
    Ok (Values.V.Map m)
  | _ -> assert false

let compile_value expr =
  let open Helpers.Result.Let_syntax in
  let* parsed =
    match Parser.parse_expr expr with
    | Ok expr -> Ok (root expr)
    | Error (`Parsing_error _ | `Prim_parsing_error _) as err -> err
  in
  compile_value parsed
