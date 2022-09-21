
FA2 with only transfer semantics
  $ ../bin/tunacc_test.exe fa2_no_metadata.tz
  Module validation error at 210:0 - 210:73: type mismatch: instruction requires [i64] but stack has []Fatal error: exception Invalid_argument("result is Error _")
  Raised at Stdlib.invalid_arg in file "stdlib.ml", line 30, characters 20-45
  Called from Stdlib__Result.get_ok in file "result.ml" (inlined), line 21, characters 45-76
  Called from Dune__exe__Tunacc_test in file "packages/tunac/bin/tunacc_test.ml", line 11, characters 12-60
  [2]


