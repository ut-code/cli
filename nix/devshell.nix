{ pkgs }:
pkgs.mkShell {
  # Add build dependencies
  packages = with pkgs; [
    (rust-bin.stable.latest.default.override {
      targets = [ "wasm32-unknown-unknown" ];
    })
    wasm-bindgen-cli
    openssl.dev
    pkg-config
  ];

  # Add environment variables
  env = { };

  # Load custom bash code
  shellHook = ''

  '';
}
