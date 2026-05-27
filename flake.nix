{
  description = "Xray Manager CLI utility";

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
        pkgs = nixpkgs.legacyPackages.${system};
        lib = pkgs.lib;

        # fallback chain for git revision
        rev = self.shortRev or self.dirtyShortRev or "";
        revSuffix = lib.optionalString (rev != "") "+${rev}";

        # package meta
        cargoToml = fromTOML (builtins.readFile ./Cargo.toml);
        baseVersion = cargoToml.package.version;
        pname = cargoToml.package.name;
        version = "${baseVersion}${revSuffix}";
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          inherit pname version;

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = [ pkgs.pkg-config ];

          buildInputs = [ ];

          meta = with lib; {
            description = cargoToml.package.description;
            homepage = cargoToml.package.repository;
            license = licenses.mit;
            mainProgram = cargoToml.package.name;
          };
        };
        apps.default =
          flake-utils.lib.mkApp {
            drv = self.packages.${system}.default;
          }
          // {
            meta = self.packages.${system}.default.meta;
          };

        devShells.default = pkgs.mkShell {

          RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

          buildInputs = with pkgs; [
            rustc
            cargo
            rust-analyzer
            rustfmt
            clippy
          ];
        };
      }
    );
}
