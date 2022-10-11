{
  nix-filter,
  lib,
  stdenv,
  dune,
  ocaml,
  zarith,
  ppx_deriving,
  ppx_yojson_conv,
  yojson,
  wasm,
  data-encoding,
  tezos-micheline,
  core, 
  core_unix, 
  ppx_jane,
  alcotest,
}:

stdenv.mkDerivation rec {
  name = "tunac";
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

  nativeBuildInputs = [
    dune
    ocaml
  ];

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
    core
    core_unix
    ppx_jane
  ];

  checkInputs = [
    alcotest
  ];

  doCheck = true;

  buildPhase = ''
    runHook preBuild
    dune build --profile=release ./packages/tunac/bin/tunacc_test_operation.exe
    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall
    mkdir -p $out/lib $out/bin
    cp _build/default/packages/tunac/bin/tunacc_test_operation.exe $out/bin/tunac
    runHook postInstall
  '';
}
