{ pkgs ? import <nixpkgs> {} }:

with pkgs;

mkShell {
  buildInputs = [
    cargo
    pkg-config
    python3
    rustc
    xorg.libX11
    xorg.libXcursor
    xorg.libxcb
  ];
}
