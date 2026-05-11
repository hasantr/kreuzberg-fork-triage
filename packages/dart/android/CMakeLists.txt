cmake_minimum_required(VERSION 3.10)
project(kreuzberg_dart VERSION 1.0)

add_library(kreuzberg_dart SHARED IMPORTED)
set_target_properties(kreuzberg_dart PROPERTIES
  IMPORTED_LOCATION "${CMAKE_CURRENT_SOURCE_DIR}/src/main/jniLibs/${ANDROID_ABI}/libkreuzberg_dart.so"
)
