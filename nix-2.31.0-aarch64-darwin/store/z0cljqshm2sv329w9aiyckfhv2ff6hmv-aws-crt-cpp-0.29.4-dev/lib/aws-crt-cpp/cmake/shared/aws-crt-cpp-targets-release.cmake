#----------------------------------------------------------------
# Generated CMake target import file for configuration "Release".
#----------------------------------------------------------------

# Commands may need to know the format version.
set(CMAKE_IMPORT_FILE_VERSION 1)

# Import target "AWS::aws-crt-cpp" for configuration "Release"
set_property(TARGET AWS::aws-crt-cpp APPEND PROPERTY IMPORTED_CONFIGURATIONS RELEASE)
set_target_properties(AWS::aws-crt-cpp PROPERTIES
  IMPORTED_LOCATION_RELEASE "/nix/store/5mj1d3dd64f8ki9vvnvh0gbj67iqd7hk-aws-crt-cpp-0.29.4/lib/libaws-crt-cpp.dylib"
  IMPORTED_SONAME_RELEASE "/nix/store/5mj1d3dd64f8ki9vvnvh0gbj67iqd7hk-aws-crt-cpp-0.29.4/lib/libaws-crt-cpp.dylib"
  )

list(APPEND _cmake_import_check_targets AWS::aws-crt-cpp )
list(APPEND _cmake_import_check_files_for_AWS::aws-crt-cpp "/nix/store/5mj1d3dd64f8ki9vvnvh0gbj67iqd7hk-aws-crt-cpp-0.29.4/lib/libaws-crt-cpp.dylib" )

# Commands beyond this point should not need to know the version.
set(CMAKE_IMPORT_FILE_VERSION)
