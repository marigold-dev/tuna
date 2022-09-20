let read_file name =
  let f = open_in name in
  let buf = Bytes.create 100000 in
  let size = input f buf 0 100000 in
  Bytes.to_string @@ Bytes.sub buf 0 size

let () =
  let wat, constants =
    Sys.argv.(1) |> read_file |> Tunac.Compiler.compile |> Result.get_ok
  in
  let out = Tunac.Output.make wat constants |> Result.get_ok in

  print_endline @@ Yojson.Safe.pretty_to_string @@ Tunac.Output.yojson_of_t out
