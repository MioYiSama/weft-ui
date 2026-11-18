cmake_minimum_required(VERSION 3.28)
# libweft.so: the native kernel (native/core) with Hermes and the JNI glue of
# the Android shell, one shared library per ABI. Built by build-core.sh with
# the NDK toolchain; the shell module packages build/jniLibs/<abi>/.
project(weft_android LANGUAGES C CXX)

if(NOT ANDROID)
  message(FATAL_ERROR "weft: configure native/android with the NDK toolchain (build-core.sh)")
endif()

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Hermes on Android wraps its finalizer thread in an fbjni ThreadScope and
# does Unicode work through java.text (PlatformUnicodeJava.cpp, Java side in
# src/main/java/com/facebook/hermes/unicode). fbjni is built from source as
# a static library, so libweft.so stays the only native library and links
# the C++ runtime statically. Its own JNI_OnLoad (OnLoad.cpp) is left out:
# the glue initializes fbjni itself.
set(WEFT_FBJNI_TAG "v0.7.0")
include(FetchContent)
FetchContent_Declare(fbjni
  GIT_REPOSITORY https://github.com/facebookincubator/fbjni.git
  GIT_TAG ${WEFT_FBJNI_TAG}
  GIT_SHALLOW TRUE
  # Sources only: the static library is defined below.
  SOURCE_SUBDIR none
)
FetchContent_MakeAvailable(fbjni)
file(GLOB fbjni_sources
  ${fbjni_SOURCE_DIR}/cxx/fbjni/*.cpp
  ${fbjni_SOURCE_DIR}/cxx/fbjni/detail/*.cpp
  ${fbjni_SOURCE_DIR}/cxx/lyra/*.cpp
)
list(FILTER fbjni_sources EXCLUDE REGEX "/OnLoad\\.cpp$")
add_library(fbjni STATIC ${fbjni_sources})
add_library(fbjni::fbjni ALIAS fbjni)
set_target_properties(fbjni PROPERTIES POSITION_INDEPENDENT_CODE ON)
target_compile_options(fbjni PRIVATE -fexceptions -frtti -ffunction-sections -fno-omit-frame-pointer -DBUILDING_FBJNI)
target_include_directories(fbjni PUBLIC ${fbjni_SOURCE_DIR}/cxx)
target_link_libraries(fbjni PUBLIC log)

# Hermes looks fbjni up with find_package; cmake/fbjniConfig.cmake hands it
# the target above.
set(fbjni_DIR ${CMAKE_CURRENT_SOURCE_DIR}/cmake)
set(HERMES_IS_ANDROID ON CACHE BOOL "" FORCE)
# The heap layout of Hermes' own Android builds.
set(HERMESVM_HEAP_HV_MODE HEAP_HV_PREFER32 CACHE STRING "" FORCE)
set(WEFT_CORE_TESTS OFF CACHE BOOL "" FORCE)
set(WEFT_CONFORMANCE OFF CACHE BOOL "" FORCE)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)
add_subdirectory(../core core)

add_library(weft SHARED src/main/cpp/weft_jni.cpp)
set_target_properties(weft PROPERTIES CXX_VISIBILITY_PRESET hidden VISIBILITY_INLINES_HIDDEN ON)
target_compile_options(weft PRIVATE -fexceptions -frtti)
target_link_libraries(weft PRIVATE weft::core fbjni::fbjni log)
# Only the JNI entry points are exported; the static libraries' symbols stay
# local and unused sections are dropped.
target_link_options(weft PRIVATE -Wl,--exclude-libs,ALL -Wl,--gc-sections)
