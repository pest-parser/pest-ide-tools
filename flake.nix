{
  inputs = {
    utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      utils,
    }:

    utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            rust-overlay.overlays.default
          ];
        };
        rustToolchain = (
          pkgs.rust-bin.stable.latest.default.override {
            extensions = [
              "rust-src"
              "rust-analyzer"
            ];
          }
        );

      in
      {
        packages.pest-ide-tools =
          (pkgs.makeRustPlatform {
            rustc = rustToolchain;
            cargo = rustToolchain;
          }).buildRustPackage
            rec {
              name = "pest-langage-server";
              src = pkgs.lib.cleanSource ./.;
              cargoLock = {
                lockFile = ./Cargo.lock;
                allowBuiltinFetchGit = true;
              };

              meta.mainProgram = name;
            };

        devShell = pkgs.mkShell {
          packages = with pkgs; [
            (lib.hiPrio rust-bin.nightly.latest.rustfmt)
            go-task
            nodejs
            prettier
            rustToolchain
          ];
        };
      }
    );

}
