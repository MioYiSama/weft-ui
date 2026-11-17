cmake_minimum_required(VERSION 3.28)
project(weft_core LANGUAGES C CXX)

# Hermes release embedded by the kernel. Bytecode is only accepted by the
# VM it was compiled for, so this tag and the `hermes-compiler` version used
# by `@weft-ui/vite` must name the same release.
set(WEFT_HERMES_TAG "hermes-v260318099.0.8")

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
add_library(weft_core
  src/ops.cpp
  src/rows.cpp
  src/runtime.cpp
)
add_library(weft::core ALIAS weft_core)
target_include_directories(weft_core PUBLIC include)
target_compile_definitions(weft_core PRIVATE WEFT_CORE_BUILD)
if(BUILD_SHARED_LIBS)
  target_compile_definitions(weft_core PUBLIC WEFT_CORE_SHARED)
endif()
target_link_libraries(weft_core PRIVATE hermesvmlean_a)

if(WEFT_CORE_TESTS)
  enable_testing()
  add_subdirectory(tests)
endif()
