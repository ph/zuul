# SPDX-FileCopyrightText: 2025 Pier-Hugues Pellerin <ph@heykimo.com>
#
# SPDX-License-Identifier: MIT

{
  description = "zuul";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
    cosmic-icons.url  = "github:pop-os/cosmic-icons";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, cosmic-icons }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];

        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustVersion = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

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
            pkgs.desktop-file-utils
            pkgs.vulkan-loader
            pkgs.reuse
            pkgs.just
            cosmic-icons.packages.x86_64-linux.default
            # zbus_xmlgen
          ];

          shellHook = ''
              export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${builtins.toString (pkgs.lib.makeLibraryPath buildInputs)}";
            '';

        };
      });
}
