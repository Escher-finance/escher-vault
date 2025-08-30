#----------------------------------------------------------------
# Generated CMake target import file for configuration "Release".
#----------------------------------------------------------------

# Commands may need to know the format version.
set(CMAKE_IMPORT_FILE_VERSION 1)

# Import target "AWS::aws-c-s3" for configuration "Release"
set_property(TARGET AWS::aws-c-s3 APPEND PROPERTY IMPORTED_CONFIGURATIONS RELEASE)
set_target_properties(AWS::aws-c-s3 PROPERTIES
  IMPORTED_LOCATION_RELEASE "${_IMPORT_PREFIX}/lib/libaws-c-s3.1.0.0.dylib"
  IMPORTED_SONAME_RELEASE "/nix/store/n432n22hhf69zd7n7f2f4d8pbvx1y01b-aws-c-s3-0.7.1/lib/libaws-c-s3.0unstable.dylib"
  )

list(APPEND _cmake_import_check_targets AWS::aws-c-s3 )
list(APPEND _cmake_import_check_files_for_AWS::aws-c-s3 "${_IMPORT_PREFIX}/lib/libaws-c-s3.1.0.0.dylib" )

# Commands beyond this point should not need to know the version.
set(CMAKE_IMPORT_FILE_VERSION)
