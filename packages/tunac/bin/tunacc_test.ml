
let read_file name =
  let f = open_in name in
  let buf = Bytes.create 10000 in
  let size = input f buf 0 10000 in
  Bytes.to_string @@ Bytes.sub buf 0 size

let () =
  Sys.argv.(1)
  |> read_file
  |> Tunac.Compiler.compile
  |> print_endline
