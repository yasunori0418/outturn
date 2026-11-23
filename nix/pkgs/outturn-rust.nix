# outturn の Rust crate（rust/）を buildRustPackage でビルドし、cargo test を走らせる。
#
# テストが ../testdata を読むため、src はリポジトリ全体（nix/pkgs から見た ../..）にし、
# cargoRoot と buildAndTestSubdir で rust/ を指す（outturn-validate.nix と同じ事情）。
# 依存は cargoLock.lockFile に渡した Cargo.lock の版で取得する。
# 例の rust/examples/conformance.rs もビルドして $out/bin に置く。checks.rust-conformance が
# その出力を outturn-validate で検証する。
{ pkgs }:
pkgs.rustPlatform.buildRustPackage {
  pname = "outturn-rust";
  version = "0.1.0";
  src = ../..;
  cargoRoot = "rust";
  buildAndTestSubdir = "rust";
  cargoLock.lockFile = ../../rust/Cargo.lock;
  doCheck = true;
  cargoBuildFlags = [
    "--example"
    "conformance"
  ];
  # cargoInstallHook は releaseDir の直下にある実行ファイルしか拾わないため、例は postInstall で置く。
  postInstall = ''
    install -Dm755 "$releaseDir/examples/conformance" "$out/bin/conformance"
  '';
}
