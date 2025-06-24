# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2024.

let
  pinnedNixpkgsSrc = builtins.fetchTarball {
    # `release-25.05` branch of 2025-06-14T18:29:16.000Z
    url = "https://github.com/NixOS/nixpkgs/archive/fcfb773595d5d62a78304cdfe76fd0e6daf428e7.tar.gz";
    sha256 = "sha256:108p56y9vj4j8m955w0nf69g23kssyrn76qxanvn9gsfi9v02g0a";
  };

in
{ pkgs ? import pinnedNixpkgsSrc {} }:

with builtins;
let
  inherit (pkgs) stdenv lib;

  tockloader = import (pkgs.fetchFromGitHub {
    owner = "tock";
    repo = "tockloader";
    rev = "v1.12.0";
    sha256 = "sha256-VgbAKDY/7ZVINDkqSHF7C0zRzVgtk8YG6O/ZmUpsh/g=";
  }) {
    inherit pkgs;
    withUnfreePkgs = false;
  };

  elf2tab = pkgs.rustPlatform.buildRustPackage rec {
    name = "elf2tab-${version}";
    version = "0.12.0";

    src = pkgs.fetchFromGitHub {
      owner = "tock";
      repo = "elf2tab";
      rev = "v${version}";
      sha256 = "sha256-+VeWLBI6md399Oaumt4pJrOkm0Nz7fmpXN2TjglUE34=";
    };

    cargoHash = "sha256-C1hg2/y557jRLkSBvFLxYKH+t8xEJudDvU72kO9sPug=";
  };

  rust_overlay = import "${pkgs.fetchFromGitHub {
    owner = "nix-community";
    repo = "fenix";
    rev = "fd217600040e0e7c7ea844af027f3dc1f4b35e6c";
    sha256 = "sha256-R3mjXc+LF74COXMDfJLuKEUPliXqOqe0wgErgTOFovI=";
  }}/overlay.nix";

  nixpkgs = import <nixpkgs> { overlays = [ rust_overlay ]; };

  # Get a custom cross-compile capable Rust install of a specific channel and
  # build. Tock expects a specific version of Rust with a selection of targets
  # and components to be present.
  rustBuild = (
    nixpkgs.fenix.fromToolchainFile { file = ./rust-toolchain.toml; }
  );

in
  pkgs.mkShell {
    name = "omniglot-tock-dev";

    buildInputs = with pkgs; [
      # --- Toolchains ---
      rustBuild
      openocd
      clang
      llvm
      lld
      pkgsCross.riscv32-embedded.buildPackages.gcc
      elf2tab

      # --- Convenience and support packages ---
      gnumake
      python3Full
      tockloader
      unzip # for libtock prebuilt toolchain download

      # --- CI support packages ---
      qemu

      # --- Flashing tools ---
      # If your board requires J-Link to flash and you are on NixOS,
      # add these lines to your system wide configuration.

      # Enable udev rules from segger-jlink package
      # services.udev.packages = [
      #     pkgs.segger-jlink
      # ];

      # Add "segger-jlink" to your system packages and accept the EULA:
      # nixpkgs.config.segger-jlink.acceptLicense = true;

      # Packages for the OSDI'25 eval reproduction pure nix-shell:
      which envsubst wget cacert perl
    ];

    LD_LIBRARY_PATH="${stdenv.cc.cc.lib}/lib64:$LD_LIBRARY_PATH";
    LIBCLANG_PATH="${pkgs.libclang.lib}/lib";

    shellHook = ''
      unset LD
      unset AS
    '';
  }
