{
  inputs = {
    systems = {
      url = "path:internal/systems.nix";
      flake = false;
    };

    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    systems,
    nixpkgs,
    fenix,
  }: let
    inherit (nixpkgs) lib;

    inherit (lib.attrsets) mapAttrs zipAttrsWith;
    inherit (lib.lists) foldl';

    mapSystems = systems: f:
      systems
      |> map (s: f s |> mapAttrs (_: v: {${s} = v;}))
      |> zipAttrsWith (_: foldl' (a: b: a // b) {});
  in
    mapSystems (import systems) (system: let
      pkgs = nixpkgs.legacyPackages.${system};
      rustfmt = fenix.packages.${system}.latest.rustfmt;
    in {
      devShells.default = import ./internal/devshell.nix {inherit lib pkgs rustfmt;};

      formatter = import ./internal/formatter.nix {inherit lib pkgs rustfmt;};

      legacyPackages = import ./internal/legacy-packages.nix {inherit lib pkgs self system;};

      packages = import ./internal/packages.nix {inherit pkgs self system;};
    });
}
