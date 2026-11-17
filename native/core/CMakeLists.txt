cmake_minimum_required(VERSION 3.28)
# Hermes has an older policy baseline; keep its MSVC runtime selection on
# CMP0091 too, so the DLL never needs the VC++ redistributable.
set(CMAKE_POLICY_DEFAULT_CMP0091 NEW)
set(CMAKE_MSVC_RUNTIME_LIBRARY MultiThreaded)
project(weft_core LANGUAGES C CXX)

# Hermes release embedded by the kernel. Bytecode is only accepted by the
# VM it was compiled for, so this tag and the `hermes-compiler` version used
# by `@weft-ui/vite` must name the same release.
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
FetchContent_MakeAvailable(hermes)

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
endif()
if(MSVC)
  target_compile_options(weft_core PRIVATE /utf-8)
endif()
target_link_libraries(weft_core PRIVATE hermesvmlean_a)

if(WEFT_CORE_TESTS)
  enable_testing()
  add_subdirectory(tests)
endif()
