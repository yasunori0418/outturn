# outturn の Rust crate（rust/）を buildRustPackage でビルドし、cargo test を走らせる。
#
# テストが ../testdata を読むため、src はリポジトリ全体（nix/pkgs から見た ../..）にし、
# cargoRoot と buildAndTestSubdir で rust/ を指す（outturn-validate.nix と同じ事情）。
# 依存は cargoLock.lockFile に渡した Cargo.lock の版で取得する。
{ pkgs }:
pkgs.rustPlatform.buildRustPackage {
  pname = "outturn-rust";
  version = "0.1.0";
  src = ../..;
  cargoRoot = "rust";
  buildAndTestSubdir = "rust";
  cargoLock.lockFile = ../../rust/Cargo.lock;
  doCheck = true;
}
