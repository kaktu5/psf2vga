{
  pkgs,
  self,
  system,
  ...
}: let
  buildWith = pkgs: (pkgs.callPackage (self + /Cargo.nix) {}).rootCrate.build;
in {
  psf2vga = buildWith pkgs;
  psf2vgaStatic = buildWith pkgs.pkgsStatic;
  default = self.packages.${system}.psf2vga;
}
