{
  lib,
  pkgs,
  self,
  system,
}: let
  inherit (lib.meta) getExe;
  inherit (pkgs) runCommandLocal;
in {
  psfToVga = name: input: (runCommandLocal name {} ''
    ${getExe self.packages.${system}.psf2vga} ${input} $out
  '');
}
