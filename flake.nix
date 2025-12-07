{
  inputs = {
    systems.url = "github:nix-systems/default";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    systems,
    nixpkgs,
    rust-overlay,
  }: let
    inherit (nixpkgs) lib;
    inherit (lib.attrsets) mapAttrs recursiveUpdate;
    inherit (lib.lists) foldl';

    mapSystems = systems: f: (foldl' (acc: system: (f system
      |> mapAttrs (_: value: {${system} = value;})
      |> recursiveUpdate acc)) {}
    systems);
  in
    mapSystems (import systems) (system: let
      pkgs = (nixpkgs.legacyPackages.${system}
          .extend rust-overlay.overlays.default)
          .extend (_: super: {
        inherit (super.rust-bin.nightly.latest) rustfmt;
      });
    in {
      checks = import ./internal/checks.nix {inherit lib pkgs self system;};
      devShells.default = import ./internal/devshell.nix {inherit lib pkgs;};
      formatter = import ./internal/formatter.nix {inherit lib pkgs;};
      legacyPackages = import ./internal/legacy-packages.nix {inherit lib pkgs self system;};
      packages = import ./internal/packages.nix {inherit pkgs self system;};
    });
}
