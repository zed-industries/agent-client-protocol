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

        # Use rustup to manage toolchains so `cargo +nightly` works in dev shell
        rustup = pkgs.rustup;
      in
      {
        inherit formatter;

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            # Rustup manages stable/nightly toolchains according to rust-toolchain.toml
            rustup
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

          # Ensure rustup shims are used and install required toolchains on first entry
          shellHook = ''
            export RUSTUP_HOME="$PWD/.rustup"
            export CARGO_HOME="$PWD/.cargo"
            export PATH="$CARGO_HOME/bin:$PATH"

            if ! command -v rustup >/dev/null 2>&1; then
              echo "rustup not found in PATH" 1>&2
            else
              # Install toolchains if missing; respect pinned channel from rust-toolchain.toml
              if ! rustup toolchain list | grep -q nightly; then
                rustup toolchain install nightly --profile minimal >/dev/null 2>&1 || true
              fi
              # Ensure stable toolchain from rust-toolchain.toml exists (rustup will auto-select it)
              # Attempt to install channel specified in rust-toolchain.toml (fallback to stable)
              TOOLCHAIN_CHANNEL=$(sed -n 's/^channel\s*=\s*"\(.*\)"/\1/p' rust-toolchain.toml || true)
              if [ -n "$TOOLCHAIN_CHANNEL" ]; then
                if ! rustup toolchain list | grep -q "$TOOLCHAIN_CHANNEL"; then
                  rustup toolchain install "$TOOLCHAIN_CHANNEL" --profile minimal --component rustfmt clippy >/dev/null 2>&1 || true
                fi
              fi
            fi
          '';
        };
      }
    );
}
