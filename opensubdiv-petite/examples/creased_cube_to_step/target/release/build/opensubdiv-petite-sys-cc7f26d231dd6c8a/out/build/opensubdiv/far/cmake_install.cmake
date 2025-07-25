# Install script for directory: /root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/far

# Set the install prefix
if(NOT DEFINED CMAKE_INSTALL_PREFIX)
  set(CMAKE_INSTALL_PREFIX "/root/repo/opensubdiv-petite/examples/creased_cube_to_step/target/release/build/opensubdiv-petite-sys-cc7f26d231dd6c8a/out")
endif()
string(REGEX REPLACE "/$" "" CMAKE_INSTALL_PREFIX "${CMAKE_INSTALL_PREFIX}")

# Set the install configuration name.
if(NOT DEFINED CMAKE_INSTALL_CONFIG_NAME)
  if(BUILD_TYPE)
    string(REGEX REPLACE "^[^A-Za-z0-9_]+" ""
           CMAKE_INSTALL_CONFIG_NAME "${BUILD_TYPE}")
  else()
    set(CMAKE_INSTALL_CONFIG_NAME "Release")
  endif()
  message(STATUS "Install configuration: \"${CMAKE_INSTALL_CONFIG_NAME}\"")
endif()

# Set the component getting installed.
if(NOT CMAKE_INSTALL_COMPONENT)
  if(COMPONENT)
    message(STATUS "Install component: \"${COMPONENT}\"")
    set(CMAKE_INSTALL_COMPONENT "${COMPONENT}")
  else()
    set(CMAKE_INSTALL_COMPONENT)
  endif()
endif()

# Install shared libraries without execute permission?
if(NOT DEFINED CMAKE_INSTALL_SO_NO_EXE)
  set(CMAKE_INSTALL_SO_NO_EXE "1")
endif()

# Is this installation the result of a crosscompile?
if(NOT DEFINED CMAKE_CROSSCOMPILING)
  set(CMAKE_CROSSCOMPILING "FALSE")
endif()

# Set default install directory permissions.
if(NOT DEFINED CMAKE_OBJDUMP)
  set(CMAKE_OBJDUMP "/usr/bin/objdump")
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/include/opensubdiv/far" TYPE FILE PERMISSIONS OWNER_READ GROUP_READ WORLD_READ FILES
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/far/error.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/far/patchDescriptor.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/far/patchParam.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/far/patchMap.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/far/patchTable.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/far/patchTableFactory.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/far/primvarRefiner.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/far/ptexIndices.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/far/stencilTable.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/far/stencilTableFactory.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/far/topologyDescriptor.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/far/topologyLevel.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/far/topologyRefiner.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/far/topologyRefinerFactory.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/far/types.h"
    )
endif()

