{
  description = "SpacetimeDB 2.6.1 view read-set experiment: official prebuilt server/CLI + pinned Rust toolchain";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    # fenix: pinned Rust toolchains (copied from Papaya's flake pattern).
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, fenix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          # spacetimedb is BSL-1.1 (unfree). Admit only that one package — not a
          # blanket allowUnfree — so nothing broader is silently pulled in.
          config.allowUnfreePredicate = pkg: (nixpkgs.lib.getName pkg) == "spacetimedb";
        };

        # Official prebuilt 2.6.1 CLI + standalone, resolved declaratively from
        # this flake (not ambient PATH, not a /nix/store glob) — spec: "Official
        # 2.6.1 server provisioning".
        spacetimedb = pkgs.callPackage ./nix/spacetimedb.nix { inherit system; };

        # Pinned Rust toolchain with the `wasm32-unknown-unknown` target the
        # module compiles to, so the repo reproducibly builds module + harness
        # (spec success criterion 1). Copied from Papaya's fenix pattern.
        rust-toolchain =
          with fenix.packages.${system};
          combine [
            (stable.withComponents [
              "cargo"
              "clippy"
              "rust-src"
              "rustc"
              "rustfmt"
            ])
            targets.wasm32-unknown-unknown.stable.rust-std
          ];
      in
      {
        packages = {
          inherit spacetimedb;
          default = spacetimedb;
        };

        devShells.default = pkgs.mkShell {
          packages = [
            spacetimedb
            rust-toolchain
          ];

          # The pinned `spacetimedb-sdk` selects `native-tls` on native targets, whose
          # `openssl-sys` build script needs `pkg-config` + OpenSSL to compile. Provide both
          # from Nix (declarative, no ambient PATH) so the native harness builds reproducibly.
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl ];

          # Absolute bin directory of the pinned official binaries. The harness
          # reads this (fail-fast if unset) to resolve `spacetimedb-cli` and
          # `spacetimedb-standalone` by absolute path — no ambient PATH, no glob.
          SPACETIMEDB_2_6_1_BIN = "${spacetimedb}/bin";
        };
      });
}
