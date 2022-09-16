
let read_file name =
  let f = open_in name in
  let buf = Bytes.create 10000 in
  let size = input f buf 0 10000 in
  Bytes.to_string @@ Bytes.sub buf 0 size

let () =
  let wat, constants =
    Sys.argv.(1)
    |> read_file
    |> Tunac.Compiler.compile
  in
  let _ = Tunac.Output.make wat constants in
  print_endline wat
