let read_file name =
  let f = open_in name in
  let buf = Bytes.create 100000 in
  let size = input f buf 0 100000 in
  Bytes.to_string @@ Bytes.sub buf 0 size

type 'a t =
  { type_ : string
  ; content : 'a
  }
[@@deriving yojson]

type originate_payload =
  { module_ : string
  ; constants : (int * Tunac.Values.t) array
  ; initial_storage : Tunac.Values.t
  ; entrypoints : Tunac.Path.t option
  }
[@@deriving yojson]

type invoke_payload =
  { address : string
  ; argument : Tunac.Values.t
  }
[@@deriving yojson]

let originate contract init =
  let init = Tunac.Compiler.compile_value init |> Result.get_ok in
  let wat, constants, entrypoints =
    contract |> read_file |> Tunac.Compiler.compile |> Result.get_ok
  in
  let out = Tunac.Output.make wat constants entrypoints |> Result.get_ok in
  { type_ = "Originate"
  ; content =
      { module_ = out.Tunac.Output.module_
      ; constants = out.Tunac.Output.constants
      ; initial_storage = init
      ; entrypoints = out.Tunac.Output.entrypoints
      }
  }
  |> yojson_of_t yojson_of_originate_payload
  |> Yojson.Safe.pretty_to_string |> print_endline

let invoke address arg =
  let init = Tunac.Compiler.compile_value arg |> Result.get_ok in

  { type_ = "Invoke"; content = { address; argument = init } }
  |> yojson_of_t yojson_of_invoke_payload
  |> Yojson.Safe.pretty_to_string |> print_endline
(* pub enum Operation {
       Originate {
           module: String,
           constants: Vec<(u32, Value)>,
           initial_storage: Value,
       },
       Invoke {
           address: ContractAddress,
           argument: Value,
           #[serde(default = "def")]
           gas_limit: u64,
       },
       Transfer {
           address: String,
           tickets: Vec<(TicketId, usize)>,
       },
   } *)

let () =
  match (Sys.argv.(1), Sys.argv.(2), Sys.argv.(3)) with
  | "invoke", address, arg -> invoke address arg
  | "originate", contract, init -> originate contract init
  | _ -> failwith "Invalid command"
