{ pkgs ? import <nixpkgs> {} }:

let
  stdenv = pkgs.stdenv;
  libPath = with pkgs; with xlibs; stdenv.lib.makeLibraryPath [
    libX11
    libXcursor
    libXi
    libXxf86vm
  ];
in
stdenv.mkDerivation {
  name = "rust-opengl";
  buildInputs = with pkgs; [
    rustup
    wasm-pack
    nodePackages.webpack
    nodePackages.webpack-cli
    cmake
    gcc
    gdb
    linuxPackages.perf
    pkgconfig
    fontconfig
    freetype
    xclip
    openssl
    nodejs
    yarn
  ];
  shellHook = "export LD_LIBRARY_PATH=${stdenv.cc.cc.lib}/lib64/:$LD_LIBRARY_PATH:${libPath}";

  # RUST_BACKTRACE=1;
  RUST_SRC_PATH="${pkgs.rustc.src}";
}
