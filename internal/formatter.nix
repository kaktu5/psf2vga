{
  lib,
  pkgs,
  rustfmt,
}: let
  inherit (lib.attrsets) attrValues;
  inherit (pkgs) writeShellApplication;
in
  writeShellApplication {
    name = "psf2vga-nix3-fmt-wrapper";
    runtimeInputs = attrValues {
      inherit rustfmt;
      inherit (pkgs) alejandra fd mdformat taplo;
    };
    text = ''
      fd "$@" -t f -e md -X mdformat --wrap 120 '{}'
      fd "$@" -t f -e nix -E Cargo.nix -X alejandra --quiet '{}'
      fd "$@" -t f -e rs -X rustfmt '{}'
      RUST_LOG='warn' fd "$@" -t f -e toml -X taplo format '{}'
    '';
  }
