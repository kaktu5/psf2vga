{
  lib,
  pkgs,
}: let
  inherit (lib.attrsets) attrValues;
  inherit (pkgs) mkShell;
in
  mkShell {
    name = "psf2vga-devshell";
    packages = attrValues {
      inherit
        (pkgs)
        # rust
        bacon
        cargo
        clippy
        rust-analyzer
        rustc
        rustfmt
        # nix
        alejandra
        crate2nix
        deadnix
        nil
        nixd
        statix
        ;
    };
  }
