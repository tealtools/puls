{ lib, stdenv, fetchzip }:

let
  src_any_platform = {
    url = "https://www.apache.org/dyn/closer.lua/pulsar/pulsar-3.2.2/apache-pulsar-3.2.2-bin.tar.gz?action=download";
    sha512 = "sha512-T5WTyYqSOp32rNBgEPFDGKbAodige+qnNuDI6KpyHDqS2E3J25gLLEpzNLbhcoxBefp4uT2v3DLyUbSC+ApfcQ==";
  };
in

stdenv.mkDerivation rec {
  pname = "apache-pulsar";
  version = "v0.1.0";

  src = fetchzip (src_any_platform);

  installPhase = ''
    mkdir -p "$out"
    cp -r $src/* $out
  '';

  outputs = [ "out" ];

  meta = with lib; {
    homepage = "https://github.com/apache/pulsar";
    description = "Next-gen data streaming platform";
    license = licenses.asl20;
    platforms = platforms.unix;
  };
}
