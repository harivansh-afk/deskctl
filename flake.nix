{
  description = "deskctl";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        lib = pkgs.lib;
        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        crateFetchurl =
          args:
          let
            crate = builtins.match "https://crates.io/api/v1/crates/([^/]+)/([^/]+)/download" args.url;
          in
          pkgs.fetchurl (
            args
            // lib.optionalAttrs (crate != null) {
              url = "https://static.crates.io/crates/${builtins.elemAt crate 0}/${builtins.elemAt crate 1}/download";
            }
          );
        cargoDeps = (pkgs.rustPlatform.importCargoLock.override { fetchurl = crateFetchurl; }) {
          lockFile = ./Cargo.lock;
        };

        deskctl = pkgs.rustPlatform.buildRustPackage {
          pname = cargoToml.package.name;
          version = cargoToml.package.version;
          src = ./.;
          inherit cargoDeps;
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
          packages = [
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
