hashlock
  $ ../bin/tunacc_test.exe contract hashlock.tz
  Module validation error at 261:18 - 261:22: unknown function $nowFatal error: exception Invalid_argument("result is Error _")
  Raised at Stdlib.invalid_arg in file "stdlib.ml", line 30, characters 20-45
  Called from Stdlib__Result.get_ok in file "result.ml" (inlined), line 21, characters 45-76
  Called from Dune__exe__Tunacc_test.compile_contract in file "packages/tunac/bin/tunacc_test.ml", line 12, characters 12-72
  Called from Dune__exe__Tunacc_test in file "packages/tunac/bin/tunacc_test.ml", line 23, characters 18-47
  [2]

