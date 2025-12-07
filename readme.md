# psf2vga

Convert PC Screen Font to VGA text mode font (with Code Page 437 encoding).

## Features

- Supports both PSF1 and PSF2 font formats
- Handles gzip-compressed PSF files
- Generates preview images of fonts

## CLI

```
Convert a PC Screen Font to VGA text mode font (Code Page 437)

Usage: psf2vga [OPTIONS] <INPUT_PATH> [OUTPUT_PATH]

Arguments:
  <INPUT_PATH>   Input PSF font file
  [OUTPUT_PATH]  Output VGA font file

Options:
      --preview  Generate preview images
  -q, --quiet    Decrease logging verbosity
  -h, --help     Print help
```

## Nix

Available as both a package (`packages.${system}.psf2vga`) and a function (`legacyPackages.${system}.psfToVga`).

```
inputs.psf2vga.packages.${system}.psf2vga

inputs.psf2vga.legacyPackages.${system}.psfToVga
    "ter-v16n.f16"
    (pkgs.terminus_font + /share/consolefonts/ter-v16n.psf.gz)
```
