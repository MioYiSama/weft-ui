#!/usr/bin/env bash
# Builds build/linux-<arch>/libweft_core.so for the host's architecture: the
# native kernel (native/core) with Hermes linked statically, which the Linux
# shell links. Only the kernel's C ABI is exported. Also builds the Hermes
# compiler of the same release, build/linux-<arch>/bin/hermesc, since the
# hermes-compiler package has no Linux arm64 binary (point WEFT_HERMESC at it).
#
# Needs a C++20 compiler, CMake, Ninja, Python 3 and a stable Rust toolchain.
# Set WEFT_HERMES_SOURCE to reuse a checkout of the kernel's Hermes tag
# instead of cloning it.
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
core=$(cd "$here/../core" && pwd)

case $(uname -m) in
  x86_64 | amd64) arch=x64 ;;
  aarch64 | arm64) arch=arm64 ;;
  *)
    echo "weft: unsupported architecture $(uname -m)" >&2
    exit 1
    ;;
esac
build="$here/build/linux-$arch"

fetch=()
if [[ -n "${WEFT_HERMES_SOURCE:-}" ]]; then
  fetch=(-DFETCHCONTENT_SOURCE_DIR_HERMES="$(cd "$WEFT_HERMES_SOURCE" && pwd)")
fi

echo "weft: building the kernel for Linux $arch"
cmake -S "$core" -B "$build" -G Ninja \
  -DCMAKE_BUILD_TYPE=Release \
  -DWEFT_CORE_SHARED=ON \
  -DWEFT_CORE_TESTS=OFF \
  -DWEFT_CONFORMANCE=OFF \
  "${fetch[@]}"
cmake --build "$build" --target weft_core hermesc
strip --strip-unneeded "$build/libweft_core.so"
echo "weft: wrote $build/libweft_core.so and $build/bin/hermesc"
