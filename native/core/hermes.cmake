# Hermes as a subproject of the kernel, with the build options Weft needs.
# CI keys its build caches on this file and the Hermes tag. Keep the Hermes
# build options and its Linux Unicode dependency here.
#
# Hermes is built from source; only the targets the kernel links are built.
# `-DFETCHCONTENT_SOURCE_DIR_HERMES=<checkout>` reuses an existing checkout of
# the same tag.
set(HERMES_ENABLE_TEST_SUITE OFF CACHE BOOL "" FORCE)
set(HERMES_ENABLE_DEBUGGER OFF CACHE BOOL "" FORCE)
set(HERMES_ENABLE_INTL OFF CACHE BOOL "" FORCE)
set(HERMES_BUILD_SHARED_JSI OFF CACHE BOOL "" FORCE)
# Never let the parent's shared-library default turn Hermes dependencies
# into DLLs. The only dynamic boundary is the kernel's C ABI.
set(BUILD_SHARED_LIBS OFF)
if(WIN32)
  # Windows fibers avoid Boost's custom ARMASM language modules, which do
  # not provide the linker inspection modules required by recent CMake.
  set(BOOST_CONTEXT_IMPLEMENTATION winfib CACHE STRING "" FORCE)
endif()
if(WEFT_CORE_SHARED)
  set(CMAKE_POSITION_INDEPENDENT_CODE ON)
endif()
include(FetchContent)
FetchContent_Declare(hermes
  GIT_REPOSITORY https://github.com/meta/hermes.git
  GIT_TAG ${WEFT_HERMES_TAG}
  GIT_SHALLOW TRUE
  GIT_PROGRESS TRUE
  EXCLUDE_FROM_ALL
)
if(CMAKE_SYSTEM_NAME STREQUAL "Linux")
  block()
    # Weft provides its own Unicode backend; Hermes must not find or link ICU.
    set(ICU_FOUND TRUE)
    set(ICU_LIBRARIES "")
    FetchContent_MakeAvailable(hermes)
  endblock()

  # Use the target processor, not the host's, for a Linux cross-build.
  if(CMAKE_SYSTEM_PROCESSOR MATCHES "^(x86_64|AMD64|amd64)$")
    set(icu4x_arch x86_64)
    set(icu4x_archive_sha256 e2537db56bd55147cda4e054700822db29067283f5cc28bf2bc54b56eaf7ffc0)
  elseif(CMAKE_SYSTEM_PROCESSOR MATCHES "^(aarch64|arm64|ARM64)$")
    set(icu4x_arch aarch64)
    set(icu4x_archive_sha256 f1fff9706dfa9356a0dd25763779580c760c0b63e7bd7a5611e79c42e630d9ad)
  else()
    message(FATAL_ERROR "Weft's ICU4X backend does not support Linux ${CMAKE_SYSTEM_PROCESSOR}")
  endif()

  set(WEFT_ICU4X_ARCHIVE "" CACHE FILEPATH "Local ICU4X 2.4.0 static archive for the Linux target")
  set(WEFT_ICU4X_HEADERS "" CACHE FILEPATH "Local ICU4X 2.4.0 codeload source tarball")

  function(weft_icu4x_fetch out override url cached sha256)
    if(NOT "${${override}}" STREQUAL "")
      set(path "${${override}}")
      file(SHA256 "${path}" actual)
      if(NOT actual STREQUAL sha256)
        message(FATAL_ERROR "${override}: expected SHA256 ${sha256} for ${path}, got ${actual}")
      endif()
    else()
      set(path "${cached}")
      file(DOWNLOAD "${url}" "${path}" EXPECTED_HASH "SHA256=${sha256}" TLS_VERIFY ON)
    endif()
    set(${out} "${path}" PARENT_SCOPE)
  endfunction()

  set(icu4x_version 2.4.0)
  set(icu4x_dir "${CMAKE_CURRENT_BINARY_DIR}/_deps/icu4x-${icu4x_version}")
  set(icu4x_asset "icu4x-2-static-with_data-${icu4x_arch}-unknown-linux-gnu")
  file(MAKE_DIRECTORY "${icu4x_dir}")
  weft_icu4x_fetch(icu4x_archive WEFT_ICU4X_ARCHIVE
    "https://github.com/unicode-org/icu4x/releases/download/icu%40${icu4x_version}/${icu4x_asset}"
    "${icu4x_dir}/${icu4x_asset}.a" "${icu4x_archive_sha256}")
  weft_icu4x_fetch(icu4x_tarball WEFT_ICU4X_HEADERS
    "https://codeload.github.com/unicode-org/icu4x/tar.gz/refs/tags/icu%40${icu4x_version}"
    "${icu4x_dir}/source.tar.gz" a21796945e1f797157d18bba04a0458a4eb9e93e215e99fe654679f8ee9142d7)

  set(icu4x_cpp "icu4x-icu-${icu4x_version}/ffi/capi/bindings/cpp")
  if(NOT EXISTS "${icu4x_dir}/headers-extracted")
    file(ARCHIVE_EXTRACT INPUT "${icu4x_tarball}" DESTINATION "${icu4x_dir}"
      PATTERNS "${icu4x_cpp}/icu4x/*" TOUCH)
    file(TOUCH "${icu4x_dir}/headers-extracted")
  endif()

  find_package(Threads REQUIRED)
  add_library(weft_icu4x STATIC IMPORTED)
  set_target_properties(weft_icu4x PROPERTIES IMPORTED_LOCATION "${icu4x_archive}")
  target_link_libraries(weft_icu4x INTERFACE Threads::Threads ${CMAKE_DL_LIBS} m)
  # Drop unused ICU4X code and data in both the kernel and host hermesc.
  target_link_options(weft_icu4x INTERFACE "LINKER:--gc-sections")

  # 6 selects Weft's ICU4X backend; Hermes' backends use values 1 through 5.
  target_compile_definitions(hermesPlatformUnicode_obj PRIVATE HERMES_PLATFORM_UNICODE=6)
  target_sources(hermesPlatformUnicode_obj PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/src/platform_unicode.cpp)
  target_include_directories(hermesPlatformUnicode_obj SYSTEM PRIVATE "${icu4x_dir}/${icu4x_cpp}")
  target_link_libraries(hermesPlatformUnicode_obj weft_icu4x)
else()
  FetchContent_MakeAvailable(hermes)
endif()
