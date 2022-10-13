{ pkgs ? import <nixpkgs> {} }:
  pkgs.mkShell rec {
    buildInputs = with pkgs; [
      rustup
    ];
    RUSTC_VERSION = pkgs.lib.readFile ./rust-toolchain;
    HISTFILE = toString ./.history;
    RUST_LOG="trace";
  }