{
  lib,
  pkgs,
  self,
  system,
}: let
  inherit (lib.attrsets) attrValues;
  inherit (pkgs) runCommandLocal;
in {
  psf2vga = self.packages.${system}.psf2vga.override {runTests = true;};

  formatting =
    runCommandLocal "psf2vga-formatting-check" {
      nativeBuildInputs = attrValues {
        inherit (pkgs) alejandra fd mdformat rustfmt taplo;
      };
    } ''
      fd . ${self} -t f -e md -X mdformat --check --wrap 120 '{}'
      fd . ${self} -t f -e nix -E Cargo.nix -X alejandra --check '{}'
      fd . ${self} -t f -e rs -X rustfmt --check '{}'
      RUST_LOG='warn' fd . ${self} -t f -e toml -X taplo check '{}'
      touch $out
    '';
}
