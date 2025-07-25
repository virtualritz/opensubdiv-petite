#----------------------------------------------------------------
# Generated CMake target import file for configuration "Debug".
#----------------------------------------------------------------

# Commands may need to know the format version.
set(CMAKE_IMPORT_FILE_VERSION 1)

# Import target "OpenSubdiv::osdCPU_static" for configuration "Debug"
set_property(TARGET OpenSubdiv::osdCPU_static APPEND PROPERTY IMPORTED_CONFIGURATIONS DEBUG)
set_target_properties(OpenSubdiv::osdCPU_static PROPERTIES
  IMPORTED_LINK_INTERFACE_LANGUAGES_DEBUG "CXX"
  IMPORTED_LOCATION_DEBUG "${_IMPORT_PREFIX}/lib/libosdCPU.a"
  )

list(APPEND _cmake_import_check_targets OpenSubdiv::osdCPU_static )
list(APPEND _cmake_import_check_files_for_OpenSubdiv::osdCPU_static "${_IMPORT_PREFIX}/lib/libosdCPU.a" )

# Import target "OpenSubdiv::osdCPU" for configuration "Debug"
set_property(TARGET OpenSubdiv::osdCPU APPEND PROPERTY IMPORTED_CONFIGURATIONS DEBUG)
set_target_properties(OpenSubdiv::osdCPU PROPERTIES
  IMPORTED_LOCATION_DEBUG "${_IMPORT_PREFIX}/lib/libosdCPU.so.3.6.1"
  IMPORTED_SONAME_DEBUG "libosdCPU.so.3.6.1"
  )

list(APPEND _cmake_import_check_targets OpenSubdiv::osdCPU )
list(APPEND _cmake_import_check_files_for_OpenSubdiv::osdCPU "${_IMPORT_PREFIX}/lib/libosdCPU.so.3.6.1" )

# Commands beyond this point should not need to know the version.
set(CMAKE_IMPORT_FILE_VERSION)
