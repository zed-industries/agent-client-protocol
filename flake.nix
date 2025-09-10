{
  description = "Devshell for ACP";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        formatter = pkgs.nixfmt-rfc-style;

        # Rust toolchain derived from rust-toolchain.toml
        # Uses oxalica/rust-overlay to match the pinned channel/components.
        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in
      {
        inherit formatter;

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            # Rust toolchain pinned via rust-toolchain.toml
            rustToolchain
            pkg-config
            openssl

            # Node.js toolchain
            nodejs_24

            # Go toolchain
            go_1_24

            # Nix formatter
            formatter
          ];

          RUST_BACKTRACE = "1";
        };
      }
    );
}
