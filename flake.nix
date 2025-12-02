{
  description = "Photo renamer tool based on EXIF data";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rustfmt" "clippy" ];
        };

        buildInputs = with pkgs; [
          pkg-config
          openssl
        ] ++ lib.optionals stdenv.isDarwin [
          darwin.apple_sdk.frameworks.Security
          darwin.apple_sdk.frameworks.SystemConfiguration
        ];

        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
        ];

      in
      {
        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs;
          
          shellHook = ''
            echo "Photo Renamer development environment"
            echo "Run 'cargo build' to build the project"
            echo "Run 'cargo run -- --help' to see usage"
          '';
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "photo-renamer";
          version = "0.1.0";
          
          src = ./.;
          
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          
          inherit buildInputs nativeBuildInputs;
          
          meta = with pkgs.lib; {
            description = "Rename photos based on EXIF date and GPS location data";
            homepage = "https://github.com/tommd/PhotoRenamer";
            license = licenses.mit;
            maintainers = [ ];
            platforms = platforms.all;
          };
        };

        packages.photo-renamer = self.packages.${system}.default;

        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/photo-renamer";
        };
      });
}