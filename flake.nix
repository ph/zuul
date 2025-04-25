{
  description = "Pinentry application for the COSMIC desktop environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    nix-filter.url = "github:numtide/nix-filter";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, nix-filter, crane, rust-overlay,  }:
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ] (system:
      let
        overlays = [
          (import rust-overlay)
        ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
        unfiltered = ./.;
        src = nixpkgs.lib.fileset.toSource {
          root = unfiltered;
          fileset = nixpkgs.lib.fileset.unions [
            (craneLib.fileset.commonCargoSources unfiltered)
            (nixpkgs.lib.fileset.fileFilter (file: file.hasExt "md") unfiltered)
            (nixpkgs.lib.fileset.maybeMissing ./resources)
            (nixpkgs.lib.fileset.maybeMissing ./i18n)
          ];
        };

        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
          autoPatchelfHook
        ];

        buildInputs = with pkgs; [
          libxkbcommon
          wayland
        ];

        commonArgs = {
          inherit src buildInputs nativeBuildInputs;
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        zuul = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
        });
      in
        with pkgs;
        {
          packages =
            {
              inherit zuul;
              default = zuul;
            };
          devShells.default = mkShell {
            # instead of passing `buildInputs` / `nativeBuildInputs`,
            # we refer to an existing derivation here
            inputsFrom = [
              zuul
            ];

            buildInputs = with pkgs; [ goreleaser ] ++ buildInputs;

            LD_LIBRARY_PATH = pkgs.lib.strings.makeLibraryPath buildInputs;
          };
        }
    );
}
