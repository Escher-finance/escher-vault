#----------------------------------------------------------------
# Generated CMake target import file for configuration "Release".
#----------------------------------------------------------------

# Commands may need to know the format version.
set(CMAKE_IMPORT_FILE_VERSION 1)

# Import target "TBB::tbb" for configuration "Release"
set_property(TARGET TBB::tbb APPEND PROPERTY IMPORTED_CONFIGURATIONS RELEASE)
set_target_properties(TBB::tbb PROPERTIES
  IMPORTED_LOCATION_RELEASE "/nix/store/35xjn153jbkkh8d6ibndv4winkyfmni9-tbb-2021.11.0/lib/libtbb.12.11.dylib"
  IMPORTED_SONAME_RELEASE "/nix/store/35xjn153jbkkh8d6ibndv4winkyfmni9-tbb-2021.11.0/lib/libtbb.12.dylib"
  )

list(APPEND _cmake_import_check_targets TBB::tbb )
list(APPEND _cmake_import_check_files_for_TBB::tbb "/nix/store/35xjn153jbkkh8d6ibndv4winkyfmni9-tbb-2021.11.0/lib/libtbb.12.11.dylib" )

# Import target "TBB::tbbmalloc" for configuration "Release"
set_property(TARGET TBB::tbbmalloc APPEND PROPERTY IMPORTED_CONFIGURATIONS RELEASE)
set_target_properties(TBB::tbbmalloc PROPERTIES
  IMPORTED_LOCATION_RELEASE "/nix/store/35xjn153jbkkh8d6ibndv4winkyfmni9-tbb-2021.11.0/lib/libtbbmalloc.2.11.dylib"
  IMPORTED_SONAME_RELEASE "/nix/store/35xjn153jbkkh8d6ibndv4winkyfmni9-tbb-2021.11.0/lib/libtbbmalloc.2.dylib"
  )

list(APPEND _cmake_import_check_targets TBB::tbbmalloc )
list(APPEND _cmake_import_check_files_for_TBB::tbbmalloc "/nix/store/35xjn153jbkkh8d6ibndv4winkyfmni9-tbb-2021.11.0/lib/libtbbmalloc.2.11.dylib" )

# Import target "TBB::tbbmalloc_proxy" for configuration "Release"
set_property(TARGET TBB::tbbmalloc_proxy APPEND PROPERTY IMPORTED_CONFIGURATIONS RELEASE)
set_target_properties(TBB::tbbmalloc_proxy PROPERTIES
  IMPORTED_LINK_DEPENDENT_LIBRARIES_RELEASE "TBB::tbbmalloc"
  IMPORTED_LOCATION_RELEASE "/nix/store/35xjn153jbkkh8d6ibndv4winkyfmni9-tbb-2021.11.0/lib/libtbbmalloc_proxy.2.11.dylib"
  IMPORTED_SONAME_RELEASE "/nix/store/35xjn153jbkkh8d6ibndv4winkyfmni9-tbb-2021.11.0/lib/libtbbmalloc_proxy.2.dylib"
  )

list(APPEND _cmake_import_check_targets TBB::tbbmalloc_proxy )
list(APPEND _cmake_import_check_files_for_TBB::tbbmalloc_proxy "/nix/store/35xjn153jbkkh8d6ibndv4winkyfmni9-tbb-2021.11.0/lib/libtbbmalloc_proxy.2.11.dylib" )

# Commands beyond this point should not need to know the version.
set(CMAKE_IMPORT_FILE_VERSION)
