{
  lib,
  pkgs,
  self,
  system,
}: let
  inherit (lib.meta) getExe;
  inherit (pkgs) runCommandLocal;
  inherit (self.packages.${system}) psf2vga;
in {
  psfToVga = name: input: (runCommandLocal name {} ''
    ${getExe psf2vga} ${input} $out
  '');
}
