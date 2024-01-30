{
  description = "PBGql";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, ... }@inputs: {
    overlays.dev = nixpkgs.lib.composeManyExtensions [
    ];
  } // inputs.utils.lib.eachSystem [
    "x86_64-linux"
  ] (system:
    let pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
          overlays = [ self.overlays.dev ];
        };
    in {
      devShells.default = let
        name = "PBGql";
      in pkgs.mkShell {
        inherit name;

        packages = with pkgs; [
          openssl_3_2
          pkg-config
        ];
      };
    });
}
