{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
  }: let
    inherit (nixpkgs) lib;
    inherit (lib.attrsets) attrValues mapAttrs recursiveUpdate;
    inherit (lib.lists) foldl';

    mapSystems = systems: f: (foldl' (acc: system: (f system
      |> mapAttrs (_: value: {${system} = value;})
      |> recursiveUpdate acc)) {}
    systems);
    mapSystems' = mapSystems [
      "x86_64-darwin"
      "aarch64-darwin"
      "x86_64-linux"
      "aarch64-linux"
    ];

    cargoToml = lib.importTOML ./Cargo.toml;
  in
    mapSystems' (system: let
      pkgs = (nixpkgs.legacyPackages.${system}
          .extend rust-overlay.overlays.default)
          .extend (_: super: {
        rust-toolchain = super.rust-bin.selectLatestNightlyWith (
          toolchain: toolchain.default.override {extensions = ["rust-analyzer"];}
        );
      });
    in {
      devShells.default = pkgs.mkShell {
        packages = attrValues {
          inherit (pkgs) bacon rust-toolchain;
        };
      };

      formatter = pkgs.writeShellApplication {
        name = "fmt";
        runtimeInputs = attrValues {
          inherit (pkgs) alejandra fd rust-toolchain taplo;
        };
        text = ''
          fd "$@" -t f -e nix -X alejandra --quiet '{}'
          fd "$@" -t f -e rs -X rustfmt '{}'
          RUST_LOG='warn' fd "$@" -t f -e toml -X taplo format '{}'
        '';
      };
    });
}
