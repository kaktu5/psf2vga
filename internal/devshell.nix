{
  lib,
  pkgs,
  rustfmt,
}: let
  inherit (lib.attrsets) attrValues;
  inherit (pkgs) mkShell;
in
  mkShell {
    name = "psf2vga-devshell";
    packages = attrValues {
      # markdown
      inherit (pkgs) mdformat;

      # nix
      inherit (pkgs) alejandra nixd;

      # rust
      inherit rustfmt;
      inherit
        (pkgs)
        cargo
        clippy
        crate2nix
        rust-analyzer
        rustc
        ;

      # toml
      inherit (pkgs) taplo;
    };
  }
