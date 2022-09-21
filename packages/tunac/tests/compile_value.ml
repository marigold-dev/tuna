
let value = Alcotest.of_pp Tunac.Values.V.pp
let error = Alcotest.of_pp (fun _fmt _t -> ())

let integers () =
  Alcotest.(check @@ result value error)
    "Same value"
    (Ok (Tunac.Values.V.Int (Z.of_int 42)))
    (Tunac.Compiler.compile_value "42")

let booleans () =
  Alcotest.(check @@ result value error)
    "Same value"
    (Ok (Tunac.Values.V.Bool 0))
    (Tunac.Compiler.compile_value "False");
  Alcotest.(check @@ result value error)
    "Same value"
    (Ok (Tunac.Values.V.Bool 1))
    (Tunac.Compiler.compile_value "True")

let bytes_ () =
  Alcotest.(check @@ result value error)
    "Same value"
    (Ok (Tunac.Values.V.Bytes (Bytes.of_string "ABC")))
    (Tunac.Compiler.compile_value "0x414243");
  Alcotest.(check @@ result value error)
    "Same value"
    (Ok (Tunac.Values.V.Bytes Bytes.empty))
    (Tunac.Compiler.compile_value "0x")

let strings () =
  Alcotest.(check @@ result value error)
    "Same value"
    (Ok (Tunac.Values.V.String "Alcotest"))
    (Tunac.Compiler.compile_value "\"Alcotest\"")

let unit_ () =
  Alcotest.(check @@ result value error)
    "Same value"
    (Ok Tunac.Values.V.Unit)
    (Tunac.Compiler.compile_value "Unit")

let pairs () =
  Alcotest.(check @@ result value error)
    "Same value"
    (Ok Tunac.Values.V.(Pair (Bool 1, Int (Z.of_int 42))))
    (Tunac.Compiler.compile_value "(Pair True 42)")

let unions () =
  Alcotest.(check @@ result value error)
    "Same value"
    (Ok Tunac.Values.V.(Union (Left (Int (Z.of_int 13)))))
    (Tunac.Compiler.compile_value "(Left 13)");
  Alcotest.(check @@ result value error)
    "Same value"
    (Ok Tunac.Values.V.(Union (Right (Int (Z.of_int 45)))))
    (Tunac.Compiler.compile_value "(Right 45)")

let optionals () =
  Alcotest.(check @@ result value error)
    "Same value"
    (Ok Tunac.Values.V.(Option None))
    (Tunac.Compiler.compile_value "None");
  Alcotest.(check @@ result value error)
    "Same value"
    (Ok Tunac.Values.V.(Option (Some (String "Hello world"))))
    (Tunac.Compiler.compile_value "(Some \"Hello world\")")

let lists () =
  Alcotest.(check @@ result value error)
    "Same value"
    (Ok Tunac.Values.V.(List []))
    (Tunac.Compiler.compile_value "{ }");
  Alcotest.(check @@ result value error)
    "Same value"
    (Ok Tunac.Values.V.(List [ Int (Z.of_int 0); Int (Z.of_int 1); Int (Z.of_int 3) ]))
    (Tunac.Compiler.compile_value "{ 0; 1; 3 }")

let maps () =
  Alcotest.(check @@ result value error)
    "Same value"
    (Ok Tunac.Values.(
      V.Map (Map.of_seq (List.to_seq
        [ V.Int (Z.of_int 0), V.String "zero"
        ; V.Int (Z.of_int 1), V.String "one"
        ; V.Int (Z.of_int 3), V.String "three" ]))))
    (Tunac.Compiler.compile_value "{ Elt 0 \"zero\"; Elt 1 \"one\" ; Elt 3 \"three\" }")

let () =
  let open Alcotest in
  run "Compile value" [
    "Values", [ test_case "Integers" `Quick integers
              ; test_case "Booleans" `Quick booleans
              ; test_case "Bytes" `Quick bytes_
              ; test_case "Strings" `Quick strings
              ; test_case "Unit" `Quick unit_
              ; test_case "Pairs" `Quick pairs
              ; test_case "Unions" `Quick unions
              ; test_case "Optionals" `Quick optionals
              ; test_case "Lists" `Quick lists
              ; test_case "Maps" `Quick maps ] ]