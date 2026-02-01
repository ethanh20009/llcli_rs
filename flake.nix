{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1"; # unstable Nixpkgs
    fenix = {
      url = "https://flakehub.com/f/nix-community/fenix/0.1";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {self, ...} @ inputs: let
    supportedSystems = [
      "x86_64-linux"
      "aarch64-linux"
      "x86_64-darwin"
      "aarch64-darwin"
    ];
    forEachSupportedSystem = f:
      inputs.nixpkgs.lib.genAttrs supportedSystems (
        system:
          f {
            pkgs = import inputs.nixpkgs {
              inherit system;
              overlays = [
                inputs.self.overlays.default
              ];
            };
          }
      );
  in {
    overlays.default = final: prev: {
      rustToolchain = with inputs.fenix.packages.${prev.stdenv.hostPlatform.system};
        combine (
          with stable; [
            clippy
            rustc
            cargo
            rustfmt
            rust-src
          ]
        );
    };

    devShells = forEachSupportedSystem (
      {pkgs}: {
        default = pkgs.mkShell {
          # Tools needed at build time (compilers, pkg-config)
          nativeBuildInputs = with pkgs; [
            pkg-config
            rustToolchain
            cargo-deny
            cargo-edit
            cargo-watch
            rust-analyzer
          ];

          # Libraries needed at runtime/link time
          buildInputs = with pkgs; [
            dbus
            openssl
          ];

          env = {
            RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
          };

          shellHook = ''
            # We need both DBus and OpenSSL in the library path for the binary to run
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [pkgs.dbus pkgs.openssl]}:$LD_LIBRARY_PATH"
          '';
        };
      }
    );
  };
}
