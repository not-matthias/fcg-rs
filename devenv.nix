{ pkgs, ... }:
{
   languages.rust.enable = true;
   env.RUST_LOG="debug";
}
