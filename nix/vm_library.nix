{
  nix-filter,
  lib,
  rustPlatform,
  m4,
  gmp,
}:

rustPlatform.buildRustPackage rec {
  pname = "vm_library";
  version = "1.0.0";

  src = with nix-filter.lib; filter {
    root = ../.;
    include = [
      "crates"
      "Cargo.lock"
      "Cargo.toml"
    ];
  };

  cargoSha256 = "sha256-uhtMKkme6Q7DKOVoiATkV8zAQNaYYigSR+WqUZFbMBk=";

  nativeBuildInputs = [
    m4
  ];

  buildInputs = [
    gmp
  ];

  doCheck = false;
}
