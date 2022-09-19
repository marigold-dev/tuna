let read_file name =
  let f = open_in name in
  let buf = Bytes.create 100000 in
  let size = input f buf 0 100000 in
  Bytes.to_string @@ Bytes.sub buf 0 size

let () =
  let wat, constants = Sys.argv.(1) |> read_file |> Tunac.Compiler.compile in
  let _ = Tunac.Output.make wat constants |> Result.get_ok in
  print_endline wat
