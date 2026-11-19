# Hermes as a subproject of the kernel, with the build options Weft needs.
# CI keys its build caches on this file and the Hermes tag, so keep every
# option that changes how Hermes is compiled here, and nothing else.
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
else()
  FetchContent_MakeAvailable(hermes)
endif()

if(CMAKE_SYSTEM_NAME STREQUAL "Linux")
  # 6 selects Weft's ICU4X backend; Hermes' backends use values 1 through 5.
  target_compile_definitions(hermesPlatformUnicode_obj PRIVATE HERMES_PLATFORM_UNICODE=6)
  target_sources(hermesPlatformUnicode_obj PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/src/platform_unicode.cpp)

  find_program(CARGO cargo HINTS "$ENV{HOME}/.cargo/bin" REQUIRED)
  find_package(Threads REQUIRED)
  get_filename_component(repo "${CMAKE_CURRENT_SOURCE_DIR}/../.." ABSOLUTE)
  set(unicode_crate "${repo}/crates/unicode")
  set(cargo_target_dir "${CMAKE_CURRENT_BINARY_DIR}/cargo")
  set(unicode_archive "${cargo_target_dir}/kernel/libweft_ui_unicode.a")
  file(GLOB_RECURSE unicode_sources CONFIGURE_DEPENDS "${unicode_crate}/src/*.rs")
  add_custom_command(OUTPUT "${unicode_archive}"
    COMMAND "${CARGO}" build --locked --profile kernel -p weft-ui-unicode --target-dir "${cargo_target_dir}"
    DEPENDS "${repo}/Cargo.toml" "${repo}/Cargo.lock" "${unicode_crate}/Cargo.toml" ${unicode_sources}
    WORKING_DIRECTORY "${repo}"
    VERBATIM)
  add_custom_target(weft_unicode_archive DEPENDS "${unicode_archive}")
  add_library(weft_unicode STATIC IMPORTED)
  set_target_properties(weft_unicode PROPERTIES IMPORTED_LOCATION "${unicode_archive}")
  add_dependencies(weft_unicode weft_unicode_archive)
  target_link_libraries(hermesPlatformUnicode_obj weft_unicode Threads::Threads ${CMAKE_DL_LIBS} m)
endif()
