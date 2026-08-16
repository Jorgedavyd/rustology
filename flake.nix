{
    description = "Rustology, studying rust for fun";

    inputs = {
        nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
        flake-utils.url = "github:numtide/flake-utils";
        rust-overlay = {
            url = "github:oxalica/rust-overlay";
            inputs.nixpkgs.follows = "nixpkgs";
        };
        naersk = {
            url = "github:nix-community/naersk";
            inputs.nixpkgs.follows = "nixpkgs";
        };
    };

    outputs = { self, nixpkgs, flake-utils, rust-overlay, naersk,  ... }@inputs:
        flake-utils.lib.eachDefaultSystem (system:
            let
                pkgs = import nixpkgs {
                    inherit system;
                    config.allowUnfree = true;
                    overlays = [ (import rust-overlay) ];
                };
                rustToolchain = pkgs.rust-bin.stable.latest.default;
            in {
                packages.default = (naersk.lib.${system}.override {
                    cargo = rustToolchain;
                    rustc = rustToolchain;
                }).buildPackage {
                    src = self + "/projects/rustology";
                    buildInputs = [ pkgs.glib ];
                };

                devShells.default = pkgs.mkShell {
                    buildInputs = [ rustToolchain ];
                    shellHook = ''
                        export PATH="$PATH:$HOME/.cargo/bin"
                    '';
                };
            }
        );
}
