#----------------------------------------------------------------
# Generated CMake target import file for configuration "Release".
#----------------------------------------------------------------

# Commands may need to know the format version.
set(CMAKE_IMPORT_FILE_VERSION 1)

# Import target "AWS::s2n" for configuration "Release"
set_property(TARGET AWS::s2n APPEND PROPERTY IMPORTED_CONFIGURATIONS RELEASE)
set_target_properties(AWS::s2n PROPERTIES
  IMPORTED_LOCATION_RELEASE "/nix/store/nimdy8cyn107g3bkm40cbggj3pxwwa8g-s2n-tls-1.5.17/lib/libs2n.1.0.0.dylib"
  IMPORTED_SONAME_RELEASE "/nix/store/nimdy8cyn107g3bkm40cbggj3pxwwa8g-s2n-tls-1.5.17/lib/libs2n.1.dylib"
  )

list(APPEND _cmake_import_check_targets AWS::s2n )
list(APPEND _cmake_import_check_files_for_AWS::s2n "/nix/store/nimdy8cyn107g3bkm40cbggj3pxwwa8g-s2n-tls-1.5.17/lib/libs2n.1.0.0.dylib" )

# Commands beyond this point should not need to know the version.
set(CMAKE_IMPORT_FILE_VERSION)
