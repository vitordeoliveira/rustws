{
  description = "Rust development environment for InvisibleMatrix Admin";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain
            rustc
            cargo
            rustfmt
            rust-analyzer
            clippy
            cargo-machete
            cargo-edit
            lld_21
            
            # PostgreSQL
            # postgresql_16
            # sqlx-cli
            
            # Development tools
            pkg-config
            openssl
            gcc
            libiconv
            nodejs_24
          ];

          shellHook = ''
            # Add Cargo bin to PATH for imcli and other cargo-installed binaries
            export PATH="$HOME/.cargo/bin:$PATH"
            echo "🦀 Added Cargo bin to PATH: ~/.cargo/bin"

            # Set DATABASE_URL for sqlx and other tools
            
            echo ""
            echo "🦀 Rust + PostgreSQL 16 Development Environment"
            echo "🐚 Shell: To use your preferred shell, run: nix develop -c \$SHELL"
            echo ""
          '';

          # Environment variables
          env = {
            RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          };
        };
      });
} 
