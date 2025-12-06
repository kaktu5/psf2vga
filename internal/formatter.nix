{
  lib,
  pkgs,
}: let
  inherit (lib.attrsets) attrValues;
  inherit (pkgs) writeShellApplication;
in
  writeShellApplication {
    name = "psf2vga-nix3-fmt-wrapper";
    runtimeInputs = attrValues {
      inherit (pkgs) alejandra fd mdformat rustfmt taplo;
    };
    text = ''
      fd "$@" -t f -e md -X mdformat --check --wrap 120 '{}'
      fd "$@" -t f -e nix -E Cargo.nix -X alejandra --quiet '{}'
      fd "$@" -t f -e rs -X rustfmt '{}'
      RUST_LOG='warn' fd "$@" -t f -e toml -X taplo format '{}'
    '';
  }
