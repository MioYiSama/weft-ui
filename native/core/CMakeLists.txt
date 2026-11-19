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

include(${CMAKE_CURRENT_SOURCE_DIR}/hermes.cmake)

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
