{
  nix-filter,
  lib,
  buildDunePackage,
  zarith,
  ppx_deriving,
  ppx_yojson_conv,
  yojson,
  wasm,
  data-encoding,
  tezos-micheline,
  core_bench,
  alcotest,
}:
buildDunePackage {
  pname = "tunac";
  version = "1.0.0";

  src = with nix-filter.lib; filter {
    root = ../.;
    include = [
      "packages"
      "tunac.opam"
      "dune-project"
      "dune"
      ".ocamlformat"
    ];
  };

  propagatedBuildInputs = [
    zarith
    ppx_deriving
    ppx_yojson_conv
    data-encoding
    wasm
    tezos-micheline
  ];

  buildInputs = [
    yojson
    core_bench
  ];

  checkInputs = [
    alcotest
  ];

  doCheck = true;
}
