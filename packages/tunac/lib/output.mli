type t [@@deriving yojson]

val make :
     string
  -> (int * Values.t) array
  -> (t, [ `Invalid_module | `Module_validation_error ]) result
