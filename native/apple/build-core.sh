#!/usr/bin/env bash
# Builds build/WeftCore.xcframework: the native kernel (native/core) with
# Hermes, as one static library per Apple SDK (macOS, iOS, iOS simulator),
# which the WeftKit package links as its binary target `WeftCore`.
#
# Cross builds take the Hermes compiler (needed for Hermes' internal
# bytecode) from a host build of native/core; it is configured and built in
# native/core/build when missing. An existing Hermes checkout of that host
# build is reused. Set WEFT_APPLE_SDKS to build a subset, e.g.
# "macosx iphonesimulator".
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
core=$(cd "$here/../core" && pwd)
host="$core/build"
out="$here/build"
sdks=${WEFT_APPLE_SDKS:-"macosx iphonesimulator iphoneos"}

# Minimum supported OS versions; keep in sync with Package.swift.
macos_min=14.0
ios_min=17.0

if [[ ! -x "$host/bin/hermesc" ]]; then
  echo "weft: building the host Hermes compiler in $host"
  cmake -S "$core" -B "$host" -G Ninja -DCMAKE_BUILD_TYPE=Release
  cmake --build "$host" --target hermesc
fi

fetch=()
source_dir=$(sed -n 's/^Hermes_SOURCE_DIR:STATIC=//p' "$host/CMakeCache.txt")
if [[ -n "$source_dir" && -d "$source_dir" ]]; then
  fetch=(-DFETCHCONTENT_SOURCE_DIR_HERMES="$source_dir")
fi

libraries=()
for sdk in $sdks; do
  case $sdk in
    macosx) archs="arm64;x86_64" min=$macos_min mobile=OFF ;;
    iphonesimulator) archs="arm64;x86_64" min=$ios_min mobile=ON ;;
    iphoneos) archs="arm64" min=$ios_min mobile=ON ;;
    *) echo "weft: unknown SDK $sdk" >&2; exit 1 ;;
  esac
  build="$out/$sdk"
  echo "weft: building the kernel for $sdk ($archs)"
  # The SDK goes in as a path: Hermes is a subproject, so the SDK names it
  # resolves itself (HERMES_APPLE_TARGET_PLATFORM) would arrive too late.
  cmake -S "$core" -B "$build" -G Ninja \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_OSX_SYSROOT="$(xcrun --sdk "$sdk" --show-sdk-path)" \
    -DCMAKE_OSX_ARCHITECTURES="$archs" \
    -DCMAKE_OSX_DEPLOYMENT_TARGET="$min" \
    -DHERMES_IS_MOBILE_BUILD="$mobile" \
    -DIMPORT_HOST_COMPILERS="$host/ImportHostCompilers.cmake" \
    -DWEFT_CORE_TESTS=OFF \
    "${fetch[@]}"
  cmake --build "$build" --target weft_core
  # The kernel and the static libraries it links (the link line of
  # weft_core_tests), merged so the package links one archive.
  libtool -static -no_warning_for_no_symbols -o "$build/libWeftCore.a" \
    "$build/libweft_core.a" \
    "$build/_deps/hermes-build/lib/libhermesvmlean_a.a" \
    "$build/_deps/hermes-build/jsi/libjsi.a" \
    "$build/_deps/hermes-build/external/boost/boost_1_86_0/libs/context/libboost_context.a"
  libraries+=(-library "$build/libWeftCore.a" -headers "$out/headers")
done

rm -rf "$out/headers"
mkdir -p "$out/headers/weft"
cp "$core/include/weft/runtime.h" "$out/headers/weft/"
cat > "$out/headers/module.modulemap" <<'EOF'
module WeftCore {
  header "weft/runtime.h"
  link "c++"
  link framework "CoreFoundation"
  export *
}
EOF

rm -rf "$out/WeftCore.xcframework"
xcodebuild -create-xcframework "${libraries[@]}" -output "$out/WeftCore.xcframework"
echo "weft: wrote $out/WeftCore.xcframework"
