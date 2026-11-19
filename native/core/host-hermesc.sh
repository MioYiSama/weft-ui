# Sourced by cross builds to bootstrap hermesc and reuse its Hermes checkout.
# Sets the caller's fetch array with any CMake options needed for that checkout.
weft_host_hermesc() {
  local core=$1 host=$2 source_dir

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
}
