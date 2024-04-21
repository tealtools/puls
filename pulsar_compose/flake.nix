{
  description = "Tools needed for pulsar-compose development.";
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

          protoc-gen-grpc-web = pkgs.callPackage ./nix/protoc-gen-grpc-web.nix { };
          protoc-gen-scala = pkgs.callPackage ./nix/protoc-gen-scala.nix { };

          missingSysPkgs =
            if pkgs.stdenv.isDarwin then
              [
                pkgs.darwin.apple_sdk.frameworks.Foundation
                pkgs.darwin.libiconv
              ]
            else
              [ ];

          runtimeLibraryPath = lib.makeLibraryPath ([ pkgs.zlib ]);

          pulsar-compose-dev = pkgs.mkShell {
            shellHook = ''
              export LD_LIBRARY_PATH="${runtimeLibraryPath}"
            '';

            packages = [
              pkgs.gnumake
              pkgs.coreutils
              pkgs.rustup
              pkgs.rustfmt
              pkgs.cargo-cross

              pkgs.git
            ] ++ missingSysPkgs;
          };
        in
        rec {
          packages = { };
          packages.default = pulsar-compose-dev;
          devShells.default = pulsar-compose-dev;
          devShell = devShells.default;
        }
      );
}
