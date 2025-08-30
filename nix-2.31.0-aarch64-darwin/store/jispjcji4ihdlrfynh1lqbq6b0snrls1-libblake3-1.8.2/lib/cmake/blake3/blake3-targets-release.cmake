#----------------------------------------------------------------
# Generated CMake target import file for configuration "Release".
#----------------------------------------------------------------

# Commands may need to know the format version.
set(CMAKE_IMPORT_FILE_VERSION 1)

# Import target "BLAKE3::blake3" for configuration "Release"
set_property(TARGET BLAKE3::blake3 APPEND PROPERTY IMPORTED_CONFIGURATIONS RELEASE)
set_target_properties(BLAKE3::blake3 PROPERTIES
  IMPORTED_LOCATION_RELEASE "/nix/store/jispjcji4ihdlrfynh1lqbq6b0snrls1-libblake3-1.8.2/lib/libblake3.1.8.2.dylib"
  IMPORTED_SONAME_RELEASE "/nix/store/jispjcji4ihdlrfynh1lqbq6b0snrls1-libblake3-1.8.2/lib/libblake3.0.dylib"
  )

list(APPEND _cmake_import_check_targets BLAKE3::blake3 )
list(APPEND _cmake_import_check_files_for_BLAKE3::blake3 "/nix/store/jispjcji4ihdlrfynh1lqbq6b0snrls1-libblake3-1.8.2/lib/libblake3.1.8.2.dylib" )

# Commands beyond this point should not need to know the version.
set(CMAKE_IMPORT_FILE_VERSION)
