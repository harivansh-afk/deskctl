{
  description = "deskctl - Desktop control CLI for AI agents on Linux X11";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        lib = pkgs.lib;
        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);

        deskctl =
          pkgs.rustPlatform.buildRustPackage {
            pname = cargoToml.package.name;
            version = cargoToml.package.version;
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = [ pkgs.pkg-config ];
            buildInputs = lib.optionals pkgs.stdenv.isLinux [
              pkgs.libx11
              pkgs.libxtst
            ];
            doCheck = false;

            meta = with lib; {
              description = cargoToml.package.description;
              homepage = cargoToml.package.homepage;
              license = licenses.mit;
              mainProgram = "deskctl";
              platforms = platforms.linux;
            };
          };
      in
      {
        formatter = pkgs.nixfmt;

        packages = lib.optionalAttrs pkgs.stdenv.isLinux {
          inherit deskctl;
          default = deskctl;
        };

        apps = lib.optionalAttrs pkgs.stdenv.isLinux {
          default = flake-utils.lib.mkApp { drv = deskctl; };
          deskctl = flake-utils.lib.mkApp { drv = deskctl; };
        };

        checks = lib.optionalAttrs pkgs.stdenv.isLinux {
          build = deskctl;
        };

        devShells.default = pkgs.mkShell {
          packages =
            [
              pkgs.cargo
              pkgs.clippy
              pkgs.nodejs
              pkgs.nixfmt
              pkgs.pkg-config
              pkgs.pnpm
              pkgs.rustc
              pkgs.rustfmt
            ]
            ++ lib.optionals pkgs.stdenv.isLinux [
              pkgs.libx11
              pkgs.libxtst
              pkgs.xorg.xorgserver
            ];
        };
      }
    );
}
