{
  pkgs,
  self,
  system,
}: let
  buildRootCrateWith = pkgs: (pkgs.callPackage (self + /Cargo.nix) {}).rootCrate.build;
in {
  psf2vga = buildRootCrateWith pkgs;
  psf2vgaStatic = buildRootCrateWith pkgs.pkgsStatic;
  default = self.packages.${system}.psf2vga;
}
