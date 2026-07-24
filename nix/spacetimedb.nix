# spacetimedb: official prebuilt server + CLI from Clockwork Labs releases.
#
# Copied/adapted (overlay -> callPackage form) from the established pattern at
# ~/.nix-config/overlays/spacetimedb.nix. The experiment repo must provision the
# official *released* 2.7.0 distribution (v2.7.0-hotfix3) through its own
# flake/devshell (spec: "Pin the new pass to the latest published 2.7.0-family
# distribution, `v2.7.0-hotfix3`"); it must NOT build the server from source
# (the upstream SpacetimeDB flake compiles via crane) and must NOT use an ambient
# PATH `spacetime`. nixpkgs' `spacetimedb` is unfree (no cache) and would source
# build, so we fetch the official prebuilt release artifact, pinned by SRI hash:
# a fetch, not a compile. Trust root is Clockwork Labs' release asset.
#
# The tarball has no top-level dir; it extracts two dynamically-linked ELF
# binaries to cwd:
#   - spacetimedb-standalone  (the server)
#   - spacetimedb-cli         (the CLI; upstream/nixpkgs expose it as `spacetime`)
# autoPatchelfHook resolves their interpreter/libc; libstdc++/libgcc_s come from
# stdenv.cc.cc.lib.
#
# Only the platforms in `sources` are pinned/verified; others throw loudly rather
# than silently falling back to a source build. Bump: change `version` + the
# platform `hash` (get it via `nix store prefetch-file <url>`).
{ lib, stdenv, stdenvNoCC, fetchurl, autoPatchelfHook, system }:
let
  version = "2.7.0-hotfix3";

  sources = {
    "aarch64-linux" = {
      asset = "spacetime-aarch64-unknown-linux-gnu.tar.gz";
      hash = "sha256-vv0T95bZPQxbfBdRxz0Vz9CCGZDpXdkFm/FWPTRLqeo=";
    };
  };

  source =
    sources.${system}
      or (throw "spacetimedb: no pinned prebuilt binary for '${system}'; only aarch64-linux is pinned. Add the release asset + SRI hash to nix/spacetimedb.nix to enable it.");
in
stdenvNoCC.mkDerivation {
  pname = "spacetimedb";
  inherit version;

  src = fetchurl {
    url = "https://github.com/clockworklabs/SpacetimeDB/releases/download/v${version}/${source.asset}";
    inherit (source) hash;
  };

  # Tarball has no top-level directory — binaries extract straight to cwd.
  sourceRoot = ".";

  nativeBuildInputs = [ autoPatchelfHook ];
  buildInputs = [ stdenv.cc.cc.lib ];

  dontBuild = true;

  installPhase = ''
    runHook preInstall

    install -Dm755 spacetimedb-standalone $out/bin/spacetimedb-standalone
    install -Dm755 spacetimedb-cli $out/bin/spacetimedb-cli
    # Upstream/nixpkgs expose the CLI as `spacetime`; preserve that interface.
    ln -s spacetimedb-cli $out/bin/spacetime

    runHook postInstall
  '';

  meta = with lib; {
    description = "Relational database that runs your application logic inside the database (official prebuilt binaries)";
    homepage = "https://spacetimedb.com";
    downloadPage = "https://github.com/clockworklabs/SpacetimeDB/releases";
    license = licenses.bsl11;
    platforms = attrNames sources;
    mainProgram = "spacetime";
    sourceProvenance = with sourceTypes; [ binaryNativeCode ];
  };
}
