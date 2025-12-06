{
  pkgs,
  self,
  system,
}: let
  inherit (pkgs) callPackage;

  cargoNix = callPackage (self + /Cargo.nix) {};
  packages' = self.packages.${system};
in {
  psf2vga = cargoNix.rootCrate.build;
  default = packages'.psf2vga;
}
