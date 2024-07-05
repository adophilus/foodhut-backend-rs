{
  description = "The backend for FoodHut";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=release-24.05";
  };

  outputs = { self, nixpkgs }:
  let
    system = "x86_64-linux";
    pkgs = import nixpkgs { inherit system; };
  in {
    packages = {
      cargo = nixpkgs.cargo;
      rust-analyzer = nixpkgs.rust-analyzer;
      rustfmt = nixpkgs.rustfmt;
      clippy = nixpkgs.clippy;
    };

    devShells.${system} = {
      default = pkgs.mkShell {
        buildInputs = with pkgs; [
          cargo
          cargo-watch
          rust-analyzer
          rustfmt
          clippy
          sqlx-cli
        ];
      };
    };
  };
}
