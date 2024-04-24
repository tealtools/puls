{
  description = "Tools needed for puls development.";
  inputs = {
    nixpkgs = {
      url = "nixpkgs/nixos-unstable";
    };
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
    flake-utils = {
      url = "github:numtide/flake-utils";
    };
  };
  outputs =
    { self
    , nixpkgs
    , flake-compat
    , flake-utils
    ,
    } @ inputs:
    flake-utils.lib.eachSystem
      [
        flake-utils.lib.system.x86_64-linux
        flake-utils.lib.system.x86_64-darwin
        flake-utils.lib.system.aarch64-linux
        flake-utils.lib.system.aarch64-darwin
      ]
      (
        system:
        let
          inherit (nixpkgs) lib;

          pkgs = import nixpkgs {
            system = system;
            config.allowBroken = true;
          };

          apache-pulsar = pkgs.callPackage ./nix/apache-pulsar.nix { };

          missingSysPkgs =
            if pkgs.stdenv.isDarwin then
              [
                pkgs.darwin.apple_sdk.frameworks.Foundation
                pkgs.darwin.libiconv
              ]
            else
              [ ];

          runtimeLibraryPath = lib.makeLibraryPath ([ pkgs.zlib ]);

          puls-dev = pkgs.mkShell {
            shellHook = ''
              export JAVA_HOME=$(echo "$(which java)" | sed 's/\/bin\/java//g' )
              export LD_LIBRARY_PATH="${runtimeLibraryPath}"
            '';

            packages = [
              pkgs.gnumake
              pkgs.coreutils
              pkgs.git

              pkgs.rustup
              pkgs.rustfmt
              pkgs.cargo-cross

              apache-pulsar
            ] ++ missingSysPkgs;
          };
        in
        rec {
          packages = { };
          packages.default = puls-dev;
          devShells.default = puls-dev;
          devShell = devShells.default;
        }
      );
}
