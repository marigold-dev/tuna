open Helpers

module rec V : sig
  type union =
    | Left of t
    | Right of t

  and t =
    | Int of Z.t
    | String of string
    | Bool of int
    | Pair of t * t
    | Union of union
    | List of t list
    | Option of t option
    | Unit
    | Map of t Map.t
    | Set of Set.t
  [@@deriving ord, eq, yojson]
end = struct
  type union =
    | Left of t
    | Right of t

  and t =
    | Int of Z.t
    | String of string
    | Bool of int
    | Pair of t * t
    | Union of union
    | List of t list
    | Option of t option
    | Unit
    | Map of t Map.t
    | Set of Set.t
  [@@deriving ord, eq, yojson]
end

and Map : (Helpers.Map.S_with_yojson with type key = V.t) =
  Helpers.Map.Make_with_yojson (V)

and Set : (Helpers.Set.S_with_yojson with type elt = V.t) =
  Helpers.Set.Make_with_yojson (V)

include V
