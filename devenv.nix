{pkgs, ...}: {
  packages = with pkgs; [
    cargo-insta
  ];

  languages.rust.enable = true;
  env.RUST_LOG = "debug";
}
