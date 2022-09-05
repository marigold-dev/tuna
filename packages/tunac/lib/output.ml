type t =
  { module_ : string
  ; constants : (int * Values.t) array
  }
[@@deriving yojson]

let make m constants =
  let open Wasm.Script in
  let open Wasm.Source in
  try
    let m = Wasm.Parse.string_to_module m in
    match m.it with
    | Textual m ->
      Wasm.Valid.check_module m;
      let m = Wasm.Encode.encode m in
      Ok { module_ = m; constants }
    | Encoded _ | Quoted _ -> Error `Invalid_module
  with Wasm.Parse.Syntax (at, msg) | Wasm.Valid.Invalid (at, msg) ->
    Format.printf "Module validation error at %d:%d - %d:%d: %s" at.left.line
      at.left.column at.right.line at.right.column msg;
    Error `Module_validation_error
