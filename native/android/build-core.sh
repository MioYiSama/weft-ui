#!/usr/bin/env bash
# Builds build/jniLibs/<abi>/libweft.so: the native kernel (native/core) with
# Hermes, fbjni and the JNI glue of the shell, one stripped shared library
# per Android ABI, which the shell module packages as its native libraries.
#
# Needs the Android NDK (ANDROID_NDK_HOME, else the newest one under
# $ANDROID_HOME/ndk), CMake, Ninja and Python 3. Cross builds take the
# Hermes compiler (needed for Hermes' internal bytecode) from a host build of
# native/core; it is configured and built in native/core/build when missing.
# An existing Hermes checkout of that host build is reused. Set
# WEFT_ANDROID_ABIS to build a subset, e.g. "arm64-v8a x86_64".
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
core=$(cd "$here/../core" && pwd)
host="$core/build"
out="$here/build"
abis=${WEFT_ANDROID_ABIS:-"arm64-v8a armeabi-v7a x86_64 x86"}

# Minimum supported API level; keep in sync with build.gradle.kts.
min_sdk=24

ndk=${ANDROID_NDK_HOME:-}
if [[ -z "$ndk" ]]; then
  sdk=${ANDROID_HOME:-${ANDROID_SDK_ROOT:-}}
  if [[ -z "$sdk" ]]; then
    for candidate in "$HOME/Library/Android/sdk" "$HOME/Android/Sdk"; do
      [[ -d "$candidate" ]] && sdk=$candidate && break
    done
  fi
  if [[ -n "$sdk" && -d "$sdk/ndk" ]]; then
    newest=$(ls "$sdk/ndk" | sort -V | tail -n 1)
    [[ -n "$newest" ]] && ndk="$sdk/ndk/$newest"
  fi
fi
if [[ -z "$ndk" || ! -f "$ndk/build/cmake/android.toolchain.cmake" ]]; then
  echo "weft: the Android NDK was not found; install it with \`sdkmanager --install \"ndk;<version>\"\` or set ANDROID_NDK_HOME" >&2
  exit 1
fi
strip=$(echo "$ndk"/toolchains/llvm/prebuilt/*/bin/llvm-strip)

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

for abi in $abis; do
  build="$out/$abi"
  echo "weft: building the kernel for $abi with $ndk"
  cmake -S "$here" -B "$build" -G Ninja \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_TOOLCHAIN_FILE="$ndk/build/cmake/android.toolchain.cmake" \
    -DANDROID_ABI="$abi" \
    -DANDROID_PLATFORM="android-$min_sdk" \
    -DANDROID_STL=c++_static \
    -DANDROID_SUPPORT_FLEXIBLE_PAGE_SIZES=ON \
    -DIMPORT_HOST_COMPILERS="$host/ImportHostCompilers.cmake" \
    "${fetch[@]}"
  cmake --build "$build" --target weft
  mkdir -p "$out/jniLibs/$abi"
  "$strip" --strip-unneeded -o "$out/jniLibs/$abi/libweft.so" "$build/libweft.so"
  echo "weft: wrote $out/jniLibs/$abi/libweft.so"
done
