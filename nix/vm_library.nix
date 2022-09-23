{
  lib,
  rustPlatform,
  m4,
  gmp,
}:

rustPlatform.buildRustPackage rec {
  pname = "vm_library";
  version = "1.0.0";

  src = ../.;

  cargoSha256 = "sha256-iIKqu4sII/ztNRzJqqqzsv1aFIlk3v+F0LSo9yhKt78=";

  nativeBuildInputs = [
    m4
  ];

  buildInputs = [
    gmp
  ];
}
