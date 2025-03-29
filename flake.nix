{
  description = "paranormal - ain't afraid of no ghost";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];

        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustVersion = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        # zbus_xmlgen =
        #   let
        #     inherit (pkgs) lib fetchFromGitHub rustPlatform;
        #   in 
        #     rustPlatform.buildRustPackage rec {
        #       pname = "zbus_xmlgen";
        #       version = "5.1.0";

        #       src = fetchFromGitHub {
        #         leaveDotGit = true;
        #         owner = "dbus2";
        #         repo = "zbus";
        #         rev = "zbus_xmlgen-${version}";
        #         hahs = "sha256-d1n2YlOHdimMTznSTBVd4NiHt/P9Hln7+3BMSwAcSUM=";
        #       };
        #       cargoBuildFlags = "-p zbus_xmlgen";

        #       # Disable running the tests, a lot of them are failing, I believe it's
        #       # because of my environment where systemd doesn't exists and dbus is handled
        #       # by shepherd.
        #       doCheck = false;
              
        #       useFetchCargoVendor = true;
        #       # source_route = "${pname}";
        #       cargoHash = "sha256-qagwNOiQjTxQ5m8MHI9PjhzlXm1zhJajVz6iIyIaWz4=";

        #       meta = {
        #         description = "D-Bus XML interface code generator";
        #         homepage = "https://github.com/dbus2/zbus";
        #         license = lib.licenses.unlicense;
        #         maintainers = [ ];
        #       };
        #     };
      in {
        devShell = pkgs.mkShell rec {
          buildInputs = [
            (rustVersion.override { extensions = [ "rust-src" "rustfmt" "clippy" ]; })
            pkgs.cargo-deny
            pkgs.cmake
            pkgs.pkg-config
            pkgs.expat
            pkgs.fontconfig
            pkgs.freetype
            pkgs.libxkbcommon
            pkgs.lld             
            pkgs.pkg-config
            pkgs.rust-analyzer
            pkgs.wayland 
            pkgs.vulkan-loader
            # zbus_xmlgen
          ];

          shellHook = ''
              export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${builtins.toString (pkgs.lib.makeLibraryPath buildInputs)}";
            '';

        };
      });
}
