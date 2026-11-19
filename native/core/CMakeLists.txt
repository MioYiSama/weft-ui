cmake_minimum_required(VERSION 3.28)
# Hermes has an older policy baseline; keep its MSVC runtime selection on
# CMP0091 too, so the DLL never needs the VC++ redistributable.
set(CMAKE_POLICY_DEFAULT_CMP0091 NEW)
set(CMAKE_MSVC_RUNTIME_LIBRARY MultiThreaded)
project(weft_core LANGUAGES C CXX)

# Hermes release embedded by the kernel. Bytecode is only accepted by the
# VM it was compiled for, so this tag and the `hermes-compiler` version in
# each native shell package must name the same release.
set(WEFT_HERMES_TAG "hermes-v260318099.0.8")

option(WEFT_CORE_SHARED "Build the kernel as a shared library" ${BUILD_SHARED_LIBS})
option(WEFT_CORE_TESTS "Build the kernel tests" ${PROJECT_IS_TOP_LEVEL})
option(WEFT_CONFORMANCE "Run the conformance fixtures in Hermes (needs Node.js and an installed workspace)" ${PROJECT_IS_TOP_LEVEL})

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Hermes is built from source as a subproject; only the targets the kernel
# links are built. `-DFETCHCONTENT_SOURCE_DIR_HERMES=<checkout>` reuses an
# existing checkout of the same tag.
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

# The kernel only runs bytecode, so it links the VM without the compiler.
if(WEFT_CORE_SHARED)
  set(weft_core_type SHARED)
else()
  set(weft_core_type STATIC)
endif()
add_library(weft_core ${weft_core_type}
  src/ops.cpp
  src/rows.cpp
  src/runtime.cpp
)
add_library(weft::core ALIAS weft_core)
target_include_directories(weft_core PUBLIC include)
target_compile_definitions(weft_core PRIVATE WEFT_CORE_BUILD)
if(WEFT_CORE_SHARED)
  target_compile_definitions(weft_core PUBLIC WEFT_CORE_SHARED)
  # Like the Windows DLL, the shared library exports the C ABI only: neither
  # the kernel's C++ helpers nor the statically linked Hermes.
  set_target_properties(weft_core PROPERTIES
    C_VISIBILITY_PRESET hidden
    CXX_VISIBILITY_PRESET hidden
    VISIBILITY_INLINES_HIDDEN ON)
  if(CMAKE_SYSTEM_NAME STREQUAL "Linux")
    # Keep weak C++ instantiations as well as static archives out of the ABI.
    set(exports "${CMAKE_CURRENT_SOURCE_DIR}/src/weft_core.map")
    target_link_options(weft_core PRIVATE "LINKER:--exclude-libs,ALL" "LINKER:--version-script=${exports}")
    set_property(TARGET weft_core APPEND PROPERTY LINK_DEPENDS "${exports}")
  endif()
endif()
if(MSVC)
  target_compile_options(weft_core PRIVATE /utf-8)
endif()
target_link_libraries(weft_core PRIVATE hermesvmlean_a)

if(WEFT_CORE_TESTS)
  enable_testing()
  add_subdirectory(tests)
endif()
