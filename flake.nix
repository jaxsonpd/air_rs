{
  description = "Dev enviroment for air rs";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            (pkgs.rust-bin.stable.latest.default) # latest stable rust (rustc + cargo)
            nodejs
            nodePackages.npm
            pkg-config
            fontconfig
            soapysdr-with-plugins
          ];

          shellHook = ''
            export SOAPY_SDR_PLUGIN_PATH="${pkgs.soapysdr-with-plugins}/lib/SoapySDR"
          '';
        };
      });
}
