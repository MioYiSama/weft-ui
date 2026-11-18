# find_package(fbjni CONFIG) of the Hermes subproject: fbjni is the static
# library that native/android/CMakeLists.txt builds from source.
if(NOT TARGET fbjni::fbjni)
  message(FATAL_ERROR "weft: fbjni::fbjni must be defined before Hermes is configured")
endif()
set(fbjni_FOUND TRUE)
